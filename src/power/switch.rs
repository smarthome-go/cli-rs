use log::{debug, trace};
use reqwest::StatusCode;
use smarthome_sdk_rs::{Client, Error as SdkError};

use super::errors::Error;

pub async fn toggle_power(client: &Client, switch_ids: &[String]) -> Result<(), Error> {
    let switches = match client.personal_switches().await {
        Ok(response) => response,
        Err(err) => return Err(Error::GetSwitches(err)),
    };
    for switch in switch_ids.iter() {
        let old_state = match switches.iter().find(|sw| sw.id == *switch) {
            Some(switch) => switch.power_on,
            None => return Err(Error::InvalidSwitch(switch.clone())),
        };
        if let Err(err) = set_power_helper(client, switch, !old_state).await {
            return Err(err);
        }
    }
    Ok(())
}

pub async fn set_power(
    client: &Client,
    switch_ids: &Vec<String>,
    power_on: bool,
) -> Result<(), Error> {
    for switch in switch_ids {
        set_power_helper(client, switch, power_on).await?;
    }
    Ok(())
}

pub async fn set_power_helper(
    client: &Client,
    switch_id: &str,
    power_on: bool,
) -> Result<(), Error> {
    trace!(
        "{}ctivating switch `{switch_id}`...",
        if power_on { "A" } else { "Dea" }
    );
    match client.set_power(switch_id, power_on).await {
        Ok(_) => Ok(debug!(
            "Successfully {}ctivated switch `{switch_id}`",
            if power_on { "a" } else { "dea" }
        )),
        Err(err) => Err(match err {
            SdkError::Smarthome(status_code) => match status_code {
                StatusCode::UNPROCESSABLE_ENTITY => Error::InvalidSwitch(switch_id.to_string()),
                StatusCode::FORBIDDEN => Error::PermissionDenied(switch_id.to_string()),
                StatusCode::SERVICE_UNAVAILABLE => Error::ServerError,
                _ => Error::Unknown(err),
            },
            _ => Error::Unknown(err),
        }),
    }
}
