use log::{debug, trace};
use reqwest::StatusCode;
use smarthome_sdk_rs::{Client, Error as SdkError};
use tabled::{format::Format, object::Rows, Modify, Style, TableIteratorExt};

use crate::power::draw::TableSwitch;

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
        Ok(_) => {
            debug!(
                "Successfully {}ctivated switch `{switch_id}`",
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
        Err(err) => return Err(Error::GetSwitches(err)),
    };
    let mut table = switches.into_iter().map(TableSwitch::from).table();
    println!(
        "{}",
        table.with(Style::modern().off_horizontal()).with(
            Modify::new(Rows::first()).with(Format::new(|s| format!("\x1b[1;32m{s}\x1b[1;0m")))
        )
    );
    Ok(())
}
