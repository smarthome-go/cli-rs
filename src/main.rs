use std::process;

use clap::Parser;
use cli::{Args, Command};
use log::{error, Level};
use loggerv::Logger;
use reqwest::StatusCode;
use smarthome_sdk_rs::{Auth, Client, User};

mod cli;
mod config;
mod hms;
mod power;

#[tokio::main]
async fn main() {
    let args = Args::parse();
    // Intialize logger
    Logger::new()
        .max_level(if args.verbose {
            Level::Debug
        } else {
            Level::Info
        })
        .colors(true)
        .level(true)
        .module_path_filters(vec![env!("CARGO_PKG_NAME").replace("-", "_")])
        .module_path(false)
        .init()
        .unwrap();

    // Select an appropriate configuration file path
    let config_path = match args.config_path {
        Some(from_args) => from_args,
        None => config::file_path().unwrap_or_else(|| {
            error!("Your home directory could not be determined.\nHINT: To use this program, please use the manual config file path command-line-flag");
            process::exit(1);
        }),
    };

    if args.subcommand == Command::Config {
        println!("Configuration file is located at `{config_path}`");
        process::exit(0);
    }

    // Read or create the configuration file
    let conf = match config::read_config(&config_path) {
        Ok(conf) => match conf {
            Some(conf) => conf,
            None => {
                println!("Created a new configuration file (at `{config_path}`).\nHINT: To get started, edit this file to set up your server(s) and run this program again.");
                process::exit(0);
            }
        },
        Err(err) => {
            error!(
                "Could not read nor create config file (at {config_path}): {}",
                match err {
                    config::Error::IO(err) => format!("IO error: {err}"),
                    config::Error::Parse(err) => format!("invalid TOML syntax: {err}"),
                    config::Error::Validate(err) => format!("Validation failed: {err}"),
                }
            );
            process::exit(1);
        }
    };

    // Select a server profile based on command line arguments or the default
    let profile = match args.server {
        Some(from_args) => match conf.servers.iter().find(|server| server.id == from_args) {
            Some(found) => found,
            None => {
                error!("Invalid server id from args: the id `{from_args}` was not found in the server list");
                process::exit(1);
            }
        },
        None => &conf.servers[0],
    };

    // Create a Smarthome client
    let client = match Client::new(
        &profile.url,
        match profile.token.is_empty() {
            true => Auth::QueryPassword(User {
                username: profile.username.clone(),
                password: profile.password.clone(),
            }),
            false => Auth::QueryToken(profile.token.clone()),
        },
    )
    .await
    {
        Ok(client) => client,
        Err(err) => {
            eprintln!(
                "Could not connect to Smarthome: {}",
                match err {
                    smarthome_sdk_rs::Error::UrlParse(err) =>
                        format!("Could not parse URL of server `{}`: {}", profile.id, err),
                    smarthome_sdk_rs::Error::Reqwest(err) => format!("Network error: {err}"),
                    smarthome_sdk_rs::Error::Smarthome(status_code) => format!("Smarthome error ({status_code}):\n{}", match status_code {
                        StatusCode::UNAUTHORIZED => "Login failed: invalid credentials\n => Validate your username + password or access token",
                        StatusCode::SERVICE_UNAVAILABLE => "Smarthome is currently unavailable\n => The server has significant issues and was unable to respond properly",
                        _ => "Unknown status code: please open an issue on Github"
                    }),
                    smarthome_sdk_rs::Error::VersionParse(err) => format!("Internal error: a version is invalid and could not be parsed: this is a bug and not your fault: {err}"),
                    smarthome_sdk_rs::Error::IncompatibleVersion(server_version) => format!("Incompatible server version: the server version is `{server_version}` but this program requires `{}`", smarthome_sdk_rs::SERVER_VERSION_REQUIREMENT)
                }
            );
            process::exit(1);
        }
    };

    match args.subcommand {
        Command::Power(sub) => power::handle_subcommand(sub, &client, &conf)
            .await
            .unwrap_or_else(|err| {
                error!("{err}");
                process::exit(1);
            }),
        Command::Hms(sub) => hms::handle_subcommand(sub, &client)
            .await
            .unwrap_or_else(|err| {
                error!("{err}");
                process::exit(1);
            }),
        Command::Config => unreachable!("Config should have been covered before"),
    };
}
