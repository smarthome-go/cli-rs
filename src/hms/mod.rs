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
    json: bool,
) -> Result<()> {
    match command {
        HmsCommand::Repl => repl::start(client, config.homescript.use_repl_history).await?,
        HmsCommand::Run { script_id, args } => run::run_script(client, &script_id, &args).await?,
        HmsCommand::Script(sub) => match sub {
            HmsScriptCommand::Run => workspace::exec_current_script(client, false).await?,
            HmsScriptCommand::Lint { all } => match all {
                true => listing::lint_personal(client).await?,
                false => workspace::exec_current_script(client, true).await?,
            },
            HmsScriptCommand::Ls => listing::list_personal(client, json).await?,
            HmsScriptCommand::New {
                id,
                name,
                workspace,
            } => {
                let name = name.unwrap_or_else(|| id.clone());
                crud::create_script(client, id, name, workspace).await?
            }
            HmsScriptCommand::Del { ids } => {
                for script_id in &ids {
                    crud::delete_script(client, script_id).await?
                }
            }
            HmsScriptCommand::Clone { ids, all } => workspace::clone(&ids, all, client).await?,
            HmsScriptCommand::Status { all } => workspace::status(client, all, json).await?,
            HmsScriptCommand::Diff { all } => workspace::diff(client, all).await?,
            HmsScriptCommand::Push { force, all } => {
                workspace::push(client, config.homescript.lint_on_push, force, all).await?
            }
            HmsScriptCommand::Pull { force, all } => workspace::pull(client, force, all).await?,
        },
    }
    Ok(())
}
