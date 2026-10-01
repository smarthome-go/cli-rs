use smarthome_sdk_rs::Client;

mod debug;
mod export;

use crate::cli::AdminCommand;

pub async fn handle_subcommand(command: AdminCommand, client: &Client, json: bool) -> anyhow::Result<()> {
    match command {
        AdminCommand::Debug => debug::debug(client, json).await,
        AdminCommand::Export {
            profile_pictures,
            cache_data,
        } => export::export(client, profile_pictures, cache_data).await,
    }
}
