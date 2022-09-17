use reqwest::StatusCode;
use smarthome_sdk_rs::{Client, Error};

use super::errors::PowerError;

pub async fn toggle_power(client: &Client, switch_id: &str) -> Result<(), PowerError> {
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

pub async fn set_power(client: &Client, switch: &str, power_on: bool) -> Result<(), PowerError> {
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
