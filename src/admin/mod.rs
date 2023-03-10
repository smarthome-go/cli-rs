use smarthome_sdk_rs::{Client, Error};

mod debug;
mod export;

use crate::cli::AdminCommand;

pub async fn handle_subcommand(command: AdminCommand, client: &Client) -> Result<(), Error> {
    match command {
        AdminCommand::Debug => debug::debug(client).await,
        AdminCommand::Export => export::export(client).await,
    }
}
