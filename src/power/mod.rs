use smarthome_sdk_rs::Client;

use crate::{cli::PowerCommand, config::Config};

use errors::Error;

mod draw;
mod errors;
mod switch;

pub async fn handle_subcommand(
    command: PowerCommand,
    client: &Client,
    config: &Config,
    json: bool,
) -> Result<(), Error> {
    match command {
        PowerCommand::Devices { all } => switch::device_list(client, all, json).await,
        PowerCommand::Draw { simple } => draw::power_draw(client, &config.power, simple, json).await,
        PowerCommand::Toggle { device_ids } => switch::toggle_power(client, &device_ids).await,
        PowerCommand::On { device_ids } => switch::set_power(client, &device_ids, true).await,
        PowerCommand::Off { device_ids } => switch::set_power(client, &device_ids, false).await,
    }
}
