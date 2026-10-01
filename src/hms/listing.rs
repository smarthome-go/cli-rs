use anstream::println;
use log::info;
use serde::Serialize;
use smarthome_sdk_rs::{Client, Homescript, HomescriptType};
use tabled::Tabled;

use crate::{
    hms::errors::{Error, Result, render_diagnostics},
    term::{self, ACCENT, paint},
};

#[derive(Tabled, Serialize)]
pub struct ScriptRow {
    #[tabled(rename = "ID")]
    pub id: String,
    #[tabled(rename = "Name")]
    pub name: String,
    #[tabled(display = "display_driver", rename = "Type")]
    pub is_driver: bool,
    #[tabled(rename = "Icon")]
    #[serde(rename = "icon")]
    pub md_icon: String,
    #[tabled(rename = "Workspace")]
    pub workspace: String,
    #[tabled(display = "display_on_off", rename = "Quick Actions")]
    pub quick_actions_enabled: bool,
    #[tabled(display = "display_shown_hidden", rename = "Selection")]
    pub scheduler_enabled: bool,
}

impl From<Homescript> for ScriptRow {
    fn from(source: Homescript) -> Self {
        Self {
            id: source.data.id,
            name: source.data.name,
            is_driver: source.data.type_ == HomescriptType::Driver,
            md_icon: source.data.md_icon,
            workspace: source.data.workspace,
            quick_actions_enabled: source.data.quick_actions_enabled,
            scheduler_enabled: source.data.scheduler_enabled,
        }
    }
}

fn display_driver(is_driver: &bool) -> &'static str {
    if *is_driver { "driver" } else { "script" }
}

fn display_on_off(enabled: &bool) -> &'static str {
    if *enabled { "on" } else { "off" }
}

fn display_shown_hidden(shown: &bool) -> &'static str {
    if *shown { "shown" } else { "hidden" }
}

pub async fn list_personal(client: &Client, json: bool) -> Result<()> {
    let scripts = term::spin("Fetching scripts", client.list_personal_homescripts()).await?;
    let rows = scripts.into_iter().map(ScriptRow::from);
    match json {
        true => println!("{}", serde_json::to_string_pretty(&rows.collect::<Vec<_>>())?),
        false => println!("{}", term::table(rows)),
    }
    Ok(())
}

pub async fn lint_personal(client: &Client) -> Result<()> {
    let scripts = term::spin("Fetching scripts", client.list_personal_homescripts()).await?;
    let total = scripts.len();
    let (mut failed, mut with_diagnostics) = (0, 0);

    for (index, script) in scripts.iter().enumerate() {
        let response = term::spin(
            format!("Linting {} ({}/{total})", script.data.id, index + 1),
            client.exec_homescript(&script.data.id, vec![], true),
        )
        .await?;

        if !response.success {
            failed += 1;
        }
        if response.errors.is_empty() {
            continue;
        }
        with_diagnostics += 1;
        println!(
            "{}\n{}\n",
            paint(ACCENT, format_args!("=== {} ({}) ===", script.data.id, script.data.name)),
            render_diagnostics(&response.errors, &script.data.code, &response.file_contents)
        );
    }

    info!("Linted {total} scripts: {failed} with errors, {with_diagnostics} with diagnostics");
    match failed {
        0 => Ok(()),
        failed => Err(Error::Failed { failed, total }),
    }
}
