use std::vec;

use smarthome_sdk_rs::{Client, HmsRunMode, Homescript};
use tabled::{
    settings::{object::Rows, Format, Modify, Style},
    Table, Tabled,
};

use crate::hms::errors::{Error, Result};

#[derive(Tabled)]
pub struct TableHomescriptData {
    #[tabled(rename = "ID")]
    pub id: String,
    #[tabled(rename = "Name")]
    pub name: String,
    #[tabled(rename = "Icon")]
    pub md_icon: String,
    #[tabled(rename = "Worspace")]
    pub workspace: String,
    #[tabled(display_with("Self::display_quick_actions"), rename = "Quick Actions")]
    pub quick_actions_enabled: bool,
    #[tabled(display_with("Self::display_scheduler_enabled"), rename = "Selection")]
    pub scheduler_enabled: bool,
}

impl From<Homescript> for TableHomescriptData {
    fn from(source: Homescript) -> Self {
        Self {
            id: source.data.id,
            name: source.data.name,
            quick_actions_enabled: source.data.quick_actions_enabled,
            scheduler_enabled: source.data.scheduler_enabled,
            md_icon: source.data.md_icon,
            workspace: source.data.workspace,
        }
    }
}

impl TableHomescriptData {
    fn display_quick_actions(quick_actions_enabled: &bool) -> String {
        if *quick_actions_enabled { "on" } else { "off" }.to_string()
    }
    fn display_scheduler_enabled(scheduler_enabled: &bool) -> String {
        if *scheduler_enabled {
            "shown"
        } else {
            "hidden"
        }
        .to_string()
    }
}

pub async fn list_personal(client: &Client) -> Result<()> {
    let homescripts = match client.list_personal_homescripts().await {
        Ok(response) => response.into_iter().map(TableHomescriptData::from),
        Err(err) => return Err(Error::FetchHomescript(err)),
    };
    let mut table = Table::new(homescripts);
    println!(
        "{}",
        table.with(Style::modern().remove_horizontal()).with(
            Modify::new(Rows::first()).with(Format::content(|s| format!("\x1b[1;32m{s}\x1b[1;0m")))
        )
    );
    Ok(())
}

pub async fn lint_personal(client: &Client) -> Result<()> {
    let homescripts = client.list_personal_homescripts().await?;
    for script in homescripts {
        let res = client
            .exec_homescript(&script.data.id, vec![], true)
            .await?;
        println!(
            "\x1b[1;32m=== {} / {} === \x1b[0m \n{}",
            script.data.id,
            script.data.name,
            res.errors
                .iter()
                .map(|diagnostic| {
                    let mut code = script.data.code.clone();
                    if let Some(new_code) = res.file_contents.get(&diagnostic.span.filename) {
                        code = new_code.clone();
                    }
                    diagnostic.display(&code)
                })
                .collect::<Vec<String>>()
                .join("\n\n"),
        );
        if res.errors.iter().any(|diagnostic| {
            diagnostic.syntax_error.is_some() || {
                if let Some(diagnostic) = diagnostic.diagnostic_error.clone() {
                    diagnostic.kind >= 2
                } else {
                    false
                }
            }
        }) {
            break;
        }
    }
    Ok(())
}
