use smarthome_sdk_rs::{Client, Error, HardwareNode};
use tabled::{object::Rows, format::Format, Modify, Style, TableIteratorExt, Tabled};

#[derive(Tabled)]
struct TableHardwareNode {
    #[tabled(rename = "Name")]
    pub name: String,
    #[tabled(display_with("Self::display_online", args), rename = "Status")]
    pub online: bool,
    #[tabled(display_with("Self::display_enabled", args), rename = "Enabled")]
    pub enabled: bool,
    #[tabled(rename = "URL")]
    pub url: String,
    #[tabled(rename = "Token")]
    pub token: String,
}

impl From<HardwareNode> for TableHardwareNode {
    fn from(source: HardwareNode) -> Self {
        Self {
            name: source.name,
            online: source.online,
            enabled: source.enabled,
            url: source.url,
            token: source.token,
        }
    }
}

impl TableHardwareNode {
    fn display_online(&self) -> String {
        {
            if self.online {
                "\x1b[1;32mONLINE\x1b[1;0m"
            } else {
                "\x1b[1;31mOFFLINE\x1b[1;0m"
            }
        }
        .to_string()
    }
    fn display_enabled(&self) -> String {
        {
            if self.enabled {
                "\x1b[1;32mENABLED\x1b[1;0m"
            } else {
                "\x1b[1;31mDISABLED\x1b[1;0m"
            }
        }
        .to_string()
    }
}

pub async fn debug(client: &Client) -> Result<(), Error> {
    let debug_info = client.debug_info().await?;
    println!(
        "{}",
        debug_info
            .hardware_nodes
            .into_iter()
            .map(TableHardwareNode::from)
            .table()
            .with(Style::modern().off_horizontal())
            .with(
                Modify::new(Rows::first()).with(Format::new(|s| format!("\x1b[1;32m{s}\x1b[1;0m")))
            )
    );

    Ok(())
}
