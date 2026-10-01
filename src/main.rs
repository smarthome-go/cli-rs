use std::{fmt::Display, io, process};

use anstream::{ColorChoice, println};
use clap::{CommandFactory, Parser};
use cli::{Args, ColorMode, Command};
use log::{error, info};
use smarthome_sdk_rs::{Auth, Client, Error as SdkError, User};

mod admin;
mod cli;
mod config;
mod hms;
mod power;
mod term;

#[tokio::main]
async fn main() {
    let args = Args::parse();

    ColorChoice::write_global(match args.color {
        ColorMode::Auto => ColorChoice::Auto,
        ColorMode::Always => ColorChoice::Always,
        ColorMode::Never => ColorChoice::Never,
    });
    term::init_logger(args.verbose);

    // Commands which neither need the config file nor a server connection
    if let Command::Completions { shell } = args.subcommand {
        clap_complete::generate(shell, &mut Args::command(), "shome", &mut io::stdout());
        return;
    }

    // Select an appropriate configuration file path
    let config_path = match args.config_path {
        Some(from_args) => from_args,
        None => config::file_path().unwrap_or_else(|| {
            fail("your home directory could not be determined\n => Pass the config file path using `--config-path`")
        }),
    };

    if args.subcommand == Command::Config {
        println!("{config_path}");
        return;
    }

    // Read or create the configuration file
    let conf = match config::read_config(&config_path) {
        Ok(Some(conf)) => conf,
        Ok(None) => {
            info!("Created a new configuration file at `{config_path}`\n => Set up your server(s) in this file and run this program again");
            return;
        }
        Err(err) => fail(format!(
            "could not read nor create the config file at `{config_path}`: {}",
            match err {
                config::Error::IO(err) => format!("IO error: {err}"),
                config::Error::Parse(err) => format!("invalid TOML syntax: {err}"),
                config::Error::Validate(err) => format!("validation failed: {err}"),
            }
        )),
    };

    // Select a server profile based on command line arguments or the default
    let profile = match args.server {
        Some(from_args) => conf
            .servers
            .iter()
            .find(|server| server.id == from_args)
            .unwrap_or_else(|| fail(format!("there is no server with the ID `{from_args}` in the config file"))),
        None => &conf.servers[0],
    };

    // Create a Smarthome client
    let auth = match profile.token.is_empty() {
        true => Auth::QueryPassword(User {
            username: profile.username.clone(),
            password: profile.password.clone(),
        }),
        false => Auth::QueryToken(profile.token.clone()),
    };
    let client = match term::spin(
        format!("Connecting to {}", profile.url),
        Client::new(&profile.url, auth, !args.no_version_check),
    )
    .await
    {
        Ok(client) => client,
        Err(err @ SdkError::IncompatibleVersion(_)) => fail(format!(
            "could not connect to `{}`: {err}\n => Use `--no-version-check` to connect anyway",
            profile.url
        )),
        Err(err) => fail(format!("could not connect to `{}`: {err}", profile.url)),
    };

    let result = match args.subcommand {
        Command::Power(sub) => power::handle_subcommand(sub, &client, &conf, args.json)
            .await
            .map_err(|err| err.to_string()),
        Command::Hms(sub) => hms::handle_subcommand(sub, &client, &conf, args.json)
            .await
            .map_err(|err| err.to_string()),
        Command::Admin(sub) => admin::handle_subcommand(sub, &client, args.json)
            .await
            .map_err(|err| format!("{err:#}")),
        Command::Config | Command::Completions { .. } => unreachable!("Handled before connecting"),
    };

    if let Err(err) = result {
        fail(err)
    }
}

/// Reports a fatal error and exits
fn fail(message: impl Display) -> ! {
    error!("{message}");
    process::exit(1)
}
