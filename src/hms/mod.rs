use smarthome_sdk_rs::Client;

use self::errors::Result;
use crate::cli::{HmsCommand, HmsScriptCommand};

mod crud;
mod errors;
mod listing;
mod repl;
mod workspace;

pub async fn handle_subcommand(command: HmsCommand, client: &Client) -> Result<()> {
    match command {
        HmsCommand::Repl => repl::start(client).await?,
        HmsCommand::Script(sub) => match sub {
            HmsScriptCommand::Ls => listing::list_personal(client).await?,
            HmsScriptCommand::New {
                id,
                name,
                workspace,
            } => {
                crud::create_script(
                    client,
                    id.clone(),
                    name.unwrap_or_else(|| id.clone()),
                    workspace.unwrap_or_else(|| "default".to_string()),
                )
                .await?
            }
            HmsScriptCommand::Del { ids } => {
                for id in &ids {
                    crud::delete_script(client, id).await?
                }
            }
            HmsScriptCommand::Clone { ids } => {
                for id in &ids {
                    println!("Cloning `{id}`...")
                }
            }
            HmsScriptCommand::Push => workspace::push(client).await?,
            HmsScriptCommand::Pull => println!("Pull"),
        },
    }
    Ok(())
}
