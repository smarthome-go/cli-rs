use reqwest::StatusCode;
use smarthome_sdk_rs::{Client, Error as SdkError};

use super::errors::Error;

pub async fn toggle_power(client: &Client, switch_id: &str) -> Result<(), Error> {
    let switches = match client.personal_switches().await {
        Ok(response) => response,
        Err(err) => return Err(Error::GetSwitches(err)),
    };
    let old_state = match switches.iter().find(|switch| switch.id == switch_id) {
        Some(switch) => switch.power_on,
        None => return Err(Error::InvalidSwitch(switch_id.to_string())),
    };
    set_power(client, switch_id, !old_state).await
}

pub async fn set_power(client: &Client, switch: &str, power_on: bool) -> Result<(), Error> {
    match client.set_power(switch, power_on).await {
        Ok(_) => Ok(()),
        Err(err) => Err(match err {
            SdkError::Smarthome(status_code) => match status_code {
                StatusCode::UNPROCESSABLE_ENTITY => Error::InvalidSwitch(switch.to_string()),
                StatusCode::FORBIDDEN => Error::PermissionDenied(switch.to_string()),
                StatusCode::SERVICE_UNAVAILABLE => Error::ServerError,
                _ => Error::Unknown(err),
            },
            _ => Error::Unknown(err),
        }),
    }
}
