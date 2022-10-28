use smarthome_sdk_rs::{Client, Homescript};
use tabled::{format::Format, object::Rows, Modify, Style, TableIteratorExt, Tabled};

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
    #[tabled(
        display_with("Self::display_quick_actions", args),
        rename = "Quick Actions"
    )]
    pub quick_actions_enabled: bool,
    #[tabled(
        display_with("Self::display_scheduler_enabled", args),
        rename = "Selection"
    )]
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
    fn display_quick_actions(&self) -> String {
        if self.quick_actions_enabled {
            "on"
        } else {
            "off"
        }
        .to_string()
    }
    fn display_scheduler_enabled(&self) -> String {
        if self.scheduler_enabled {
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
    let mut table = homescripts.table();
    println!(
        "{}",
        table.with(Style::modern().off_horizontal()).with(
            Modify::new(Rows::first()).with(Format::new(|s| format!("\x1b[1;32m{s}\x1b[1;0m")))
        )
    );
    Ok(())
}
