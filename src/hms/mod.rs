use smarthome_sdk_rs::Client;

use crate::{
    cli::{HmsCommand, HmsScriptCommand},
    config::Config,
};
use errors::Result;

mod crud;
mod errors;
mod listing;
mod repl;
mod run;
mod workspace;

pub async fn handle_subcommand(
    command: HmsCommand,
    client: &Client,
    config: &Config,
) -> Result<()> {
    match command {
        HmsCommand::Repl => repl::start(client).await?,
        HmsCommand::Run { scipt_id, args } => run::run_script(client, &scipt_id, &args).await?,
        HmsCommand::Script(sub) => match sub {
            HmsScriptCommand::Run => workspace::exec_current_script(client, false).await?,
            HmsScriptCommand::Lint { all } => match all {
                true => listing::lint_personal(client).await?,
                false => workspace::exec_current_script(client, true).await?,
            },
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
                for script_id in &ids {
                    crud::delete_script(client, script_id).await?
                }
            }
            HmsScriptCommand::Clone { ids, all } => workspace::clone(&ids, all, client).await?,
            HmsScriptCommand::Push { force } => {
                workspace::push(client, config.homescript.lint_on_push, force).await?
            }
            HmsScriptCommand::Pull => workspace::pull(client).await?,
        },
    }
    Ok(())
}
