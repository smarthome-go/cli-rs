use anstream::println;
use log::{debug, info, trace};
use smarthome_sdk_rs::{Client, DeviceCapability, StatusCode};

use super::{
    draw::{DeviceRow, ParsedDevice},
    errors::Error,
};
use crate::term::{self, BAD, GOOD, paint};

pub async fn toggle_power(client: &Client, device_ids: &[String]) -> Result<(), Error> {
    let devices = term::spin("Fetching devices", client.personal_switches())
        .await
        .map_err(Error::GetDevices)?;

    for id in device_ids {
        let device = devices
            .iter()
            .find(|device| device.shallow.id == *id)
            .ok_or_else(|| Error::InvalidDevice(id.clone()))?;

        let has_power_capability = device
            .extractions
            .config
            .capabilities
            .contains(&DeviceCapability::Power);
        let old_state = match (has_power_capability, &device.extractions.power_information) {
            (true, Some(power)) => power.state,
            _ => return Err(Error::NoPowerCapability(id.clone())),
        };
        set_power_helper(client, id, !old_state).await?
    }
    Ok(())
}

pub async fn set_power(client: &Client, device_ids: &[String], power_on: bool) -> Result<(), Error> {
    for id in device_ids {
        set_power_helper(client, id, power_on).await?;
    }
    Ok(())
}

async fn set_power_helper(client: &Client, device_id: &str, power_on: bool) -> Result<(), Error> {
    let state = if power_on { "on" } else { "off" };
    trace!("Turning device `{device_id}` {state}...");
    match term::spin(
        format!("Turning {device_id} {state}"),
        client.set_power(device_id, power_on),
    )
    .await
    {
        Ok(_) => {
            debug!("Turned device `{device_id}` {state}");
            info!(
                "{device_id}: {}",
                if power_on { paint(GOOD, "ON") } else { paint(BAD, "OFF") }
            );
            Ok(())
        }
        Err(err) => Err(match err.status() {
            Some(StatusCode::UNPROCESSABLE_ENTITY) => Error::InvalidDevice(device_id.to_string()),
            Some(StatusCode::FORBIDDEN) => Error::PermissionDenied(device_id.to_string()),
            _ => Error::Smarthome(err),
        }),
    }
}

pub async fn device_list(client: &Client, show_all: bool, json: bool) -> Result<(), Error> {
    let devices = term::spin("Fetching devices", async {
        match show_all {
            true => client.all_switches().await,
            false => client.personal_switches().await,
        }
    })
    .await
    .map_err(Error::GetDevices)?;

    let devices = devices.into_iter().map(ParsedDevice::from);
    match json {
        true => println!("{}", serde_json::to_string_pretty(&devices.collect::<Vec<_>>())?),
        false => println!("{}", term::table(devices.map(DeviceRow::from))),
    }
    Ok(())
}
