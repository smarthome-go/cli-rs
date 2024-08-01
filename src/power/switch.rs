use log::{debug, trace};
use reqwest::StatusCode;
use smarthome_sdk_rs::{Client, DeviceCapability, Error as SdkError};
use tabled::{
    settings::{format::Format, object::Rows, Modify, Style},
    Table,
};

use crate::power::draw::{ParsedDevice, TableDevice};

use super::errors::Error;

pub async fn toggle_power(client: &Client, switch_ids: &[String]) -> Result<(), Error> {
    let switches = match client.personal_switches().await {
        Ok(response) => response,
        Err(err) => return Err(Error::GetDevices(err)),
    };
    for switch in switch_ids.iter() {
        let old_state = match switches.iter().find(|sw| sw.shallow.id == *switch) {
            Some(switch) => {
                let has_power_capability = switch
                    .extractions
                    .config
                    .capabilities
                    .contains(&DeviceCapability::Power);

                match (has_power_capability, &switch.extractions.power_information) {
                    (_, None) | (false, _) => {
                        return Err(Error::InvalidSwitch(
                            "device does not support power".to_string(),
                        ));
                    }
                    (true, Some(power)) => power.state,
                }
            }
            None => return Err(Error::InvalidSwitch(switch.clone())),
        };
        set_power_helper(client, switch, !old_state).await?
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
        "{}ctivating device `{switch_id}` power...",
        if power_on { "A" } else { "Dea" }
    );
    match client.set_power(switch_id, power_on).await {
        Ok(_) => {
            debug!(
                "Successfully {}ctivated device `{switch_id}` power",
                if power_on { "a" } else { "dea" }
            );
            Ok(())
        }
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

pub async fn switch_list(client: &Client, show_all: bool) -> Result<(), Error> {
    let switches = match if show_all {
        client.all_switches().await
    } else {
        client.personal_switches().await
    } {
        Ok(response) => response,
        Err(err) => return Err(Error::GetDevices(err)),
    };
    let mut table = Table::new(
        switches
            .into_iter()
            .map(|f| TableDevice::from(ParsedDevice::from(f))),
    );
    println!(
        "{}",
        table.with(Style::modern().remove_horizontal()).with(
            Modify::new(Rows::first()).with(Format::content(|s| format!("\x1b[1;32m{s}\x1b[1;0m")))
        )
    );
    Ok(())
}
