use crate::cli::{HmsCommand, HmsScriptCommand};
use smarthome_sdk_rs::Client;

use self::errors::Error;

mod errors;
mod repl;

pub async fn handle_subcommand(command: HmsCommand, client: Client) -> Result<(), Error> {
    match command {
        HmsCommand::Repl => repl::start(&client).await?,
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
                );
            }
            HmsScriptCommand::Clone => println!("Clone"),
            HmsScriptCommand::Del => println!("Del"),
            HmsScriptCommand::Push => println!("Push"),
            HmsScriptCommand::Pull => println!("Pull"),
        },
    }
    Ok(())
}