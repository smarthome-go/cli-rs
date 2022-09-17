use smarthome_sdk_rs::Client;

use crate::{cli::PowerCommand, config::Config};

use self::errors::Error;

mod errors;
mod draw;
mod switch;

pub async fn handle_subcommand(
    command: PowerCommand,
    client: &Client,
    config: &Config,
) -> Result<(), Error> {
    match command {
        PowerCommand::Draw => draw::power_draw(client, &config.power).await,
        PowerCommand::Toggle { switch_ids } => switch::toggle_power(client, &switch_ids).await,
        PowerCommand::On { switch_ids } => switch::set_power(client, &switch_ids, true).await,
        PowerCommand::Off { switch_ids } => switch::set_power(client, &switch_ids, false).await,
    }
}
