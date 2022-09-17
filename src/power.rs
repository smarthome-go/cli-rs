use std::fmt::Display;

use reqwest::StatusCode;
use smarthome_sdk_rs::{Client, Error};

use crate::cli::PowerCommand;

pub enum PowerError {
    GetSwitches(Error),
    InvalidSwitch(String),
    PermissionDenied(String),
    ServerError,
    Unknown(Error),
}

impl Display for PowerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::InvalidSwitch(switch_id) =>
                    format!("The switch `{switch_id}` does not exist"),
                Self::PermissionDenied(switch_id) => format!("You are either lacking permission to use switches or you do not have access to the switch `{switch_id}`"),
                Self::ServerError => format!("The server was unable to handle this switch"),
                Self::GetSwitches(err) => format!("Could not get switches: {err}"),
                Self::Unknown(err) => format!("Unknown error: {err}")
            }
        )
    }
}

pub async fn handle_subcommand(command: PowerCommand, client: Client) -> Result<(), PowerError> {
    match command {
        PowerCommand::Toggle { switch_id } => toggle_power(&client, &switch_id).await,
        PowerCommand::On { switch_id } => set_power(&client, &switch_id, true).await,
        PowerCommand::Off { switch_id } => set_power(&client, &switch_id, false).await,
    }
}

async fn toggle_power(client: &Client, switch_id: &str) -> Result<(), PowerError> {
    let switches = match client.personal_switches().await {
        Ok(response) => response,
        Err(err) => return Err(PowerError::GetSwitches(err)),
    };
    let old_state = match switches.iter().find(|switch| switch.id == switch_id) {
        Some(switch) => switch.power_on,
        None => return Err(PowerError::InvalidSwitch(switch_id.to_string())),
    };
    set_power(client, switch_id, !old_state).await
}

async fn set_power(client: &Client, switch: &str, power_on: bool) -> Result<(), PowerError> {
    match client.set_power(switch, power_on).await {
        Ok(_) => Ok(()),
        Err(err) => Err(match err {
            Error::Smarthome(status_code) => match status_code {
                StatusCode::UNPROCESSABLE_ENTITY => PowerError::InvalidSwitch(switch.to_string()),
                StatusCode::FORBIDDEN => PowerError::PermissionDenied(switch.to_string()),
                StatusCode::SERVICE_UNAVAILABLE => PowerError::ServerError,
                _ => PowerError::Unknown(err),
            },
            _ => PowerError::Unknown(err),
        }),
    }
}
