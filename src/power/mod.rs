use smarthome_sdk_rs::Client;

use crate::{cli::PowerCommand, config::Config};

use self::errors::Error;

mod errors;
mod power;
mod switch;

pub async fn handle_subcommand(
    command: PowerCommand,
    client: &Client,
    config: &Config,
) -> Result<(), Error> {
    match command {
        PowerCommand::Draw => power::power_draw(client, &config.power).await,
        PowerCommand::Toggle { switch_id } => switch::toggle_power(&client, &switch_id).await,
        PowerCommand::On { switch_id } => switch::set_power(&client, &switch_id, true).await,
        PowerCommand::Off { switch_id } => switch::set_power(&client, &switch_id, false).await,
    }
}
