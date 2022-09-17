use std::{fmt::format, process};

use clap::{Parser, Subcommand};
use log::error;
use reqwest::StatusCode;
use smarthome_sdk_rs::{Auth, Client, User};

mod config;
mod repl;

#[derive(Parser)]
#[clap(author, version, about)]
struct Args {
    /// Selects the target Smarthome server by the provided ID
    #[clap(short, long, value_parser)]
    server: Option<String>,

    /// The path where the configuration file should be located
    #[clap(short, long, value_parser)]
    config_path: Option<String>,

    /// Smarthome subcommands
    #[clap(subcommand)]
    subcommand: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Homescript subcommands
    #[clap(subcommand)]
    Hms(HmsCommand),
}

#[derive(Subcommand)]
enum HmsCommand {
    /// Interactive Homescript live terminal
    Repl,
    /// Script subcommands
    #[clap(subcommand)]
    Script(HmsScriptCommand),
}

#[derive(Subcommand)]
enum HmsScriptCommand {
    /// Create a new Homescript locally and on the remote
    New {
        /// A unique ID for the new script
        id: String,
        #[clap(short, long, value_parser)]
        /// A friendly name for the new script
        name: Option<String>,
        /// A workspace to be associated with the new script
        #[clap(short, long, value_parser)]
        workspace: Option<String>,
    },
    /// Clone an existing script from the server to the local FS
    Clone,
    /// Delete a script from the local FS and the server
    Del,
    /// Push local changes to the server
    Push,
    /// Pull any upstream changes to local FS
    Pull,
}

#[tokio::main]
async fn main() {
    env_logger::init();
    let args = Args::parse();

    let config_path = match args.config_path {
        Some(from_args) => from_args,
        None => "config.toml".to_string(),
    };

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

    // Create a Smarthome client
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
        Command::Hms(sub) => match sub {
            HmsCommand::Repl => repl::start(&client).await.unwrap(),
            HmsCommand::Script(sub) => match sub {
                HmsScriptCommand::New {
                    id,
                    name,
                    workspace,
                } => {
                    println!(
                        "Creating id: `{id}` with name `{}` | Workspace: `{}`",
                        name.unwrap_or_else(|| id.clone()),
                        workspace.unwrap_or_else(|| "default".to_string())
                    )
                }
                HmsScriptCommand::Clone => println!("Clone"),
                HmsScriptCommand::Del => println!("Del"),
                HmsScriptCommand::Push => println!("Push"),
                HmsScriptCommand::Pull => println!("Pull"),
            },
        },
    }
}
