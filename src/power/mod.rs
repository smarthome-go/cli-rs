use smarthome_sdk_rs::Client;

use crate::cli::PowerCommand;

use self::errors::PowerError;

mod errors;
mod switch;

pub async fn handle_subcommand(command: PowerCommand, client: Client) -> Result<(), PowerError> {
    match command {
        PowerCommand::Toggle { switch_id } => switch::toggle_power(&client, &switch_id).await,
        PowerCommand::On { switch_id } => switch::set_power(&client, &switch_id, true).await,
        PowerCommand::Off { switch_id } => switch::set_power(&client, &switch_id, false).await,
    }
}
