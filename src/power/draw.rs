use std::usize;

use super::errors::{Error, Result};
use crate::config::PowerConfig;
use smarthome_sdk_rs::{Client, DeviceCapability, HydratedDeviceResponse, PowerDrawPoint};
use tabled::{
    settings::{format::Format, object::Rows, Modify, Style},
    Table, Tabled,
};

pub struct ParsedDevice {
    pub id: String,
    pub name: String,
    pub room_id: String,
    pub power: Option<ParsedPower>,
}

pub struct ParsedPower {
    pub status: bool,
    pub watts: usize,
}

#[derive(Tabled)]
pub struct TableDevice {
    #[tabled(rename = "ID")]
    id: String,
    #[tabled(rename = "Name")]
    name: String,
    #[tabled(rename = "Room ID")]
    room_id: String,
    #[tabled(display_with("Self::display_watts"), rename = "Watts")]
    watts: Option<usize>,
    #[tabled(display_with("Self::display_power"), rename = "Power")]
    power_on: Option<bool>,
}

// #[derive(Tabled)]
// pub struct TablePower {
//     #[tabled(rename = "Watts")]
//     watts: usize,
//     #[tabled(display_with("Self::display_power"), rename = "Power")]
//     power_on: bool,
// }

impl TableDevice {
    fn display_power(power_on: &Option<bool>) -> String {
        match *power_on {
            Some(true) => "\x1b[1;32mON\x1b[1;0m".to_string(),
            Some(false) => "\x1b[1;31mOFF\x1b[1;0m".to_string(),
            None => "\x1b[1;30mN/A\x1b[1;0m".to_string(),
        }
    }

    fn display_watts(watts: &Option<usize>) -> String {
        match watts {
            Some(watts) => watts.to_string(),
            None => "\x1b[1;30mN/A\x1b[1;0m".to_string(),
        }
    }
}

impl From<HydratedDeviceResponse> for ParsedDevice {
    fn from(source: HydratedDeviceResponse) -> Self {
        let has_power_capability = source
            .extractions
            .config
            .capabilities
            .contains(&DeviceCapability::Power);

        let power_info = match (has_power_capability, source.extractions.power_information) {
            (true, Some(power)) => Some(ParsedPower {
                watts: power.power_draw_watts,
                status: power.state,
            }),
            (false, _) | (_, None) => None,
        };

        Self {
            id: source.shallow.id,
            name: source.shallow.name,
            room_id: source.shallow.room_id,
            power: power_info,
        }
    }
}

impl From<ParsedDevice> for TableDevice {
    fn from(source: ParsedDevice) -> Self {
        Self {
            id: source.id,
            name: source.name,
            room_id: source.room_id,
            watts: source.power.as_ref().map(|p| p.watts),
            power_on: source.power.map(|p| p.status),
        }
    }
}

pub async fn power_draw(
    client: &Client,
    config: &PowerConfig,
    use_simple_display: bool,
) -> Result<()> {
    let switches = match client.all_switches().await {
        Ok(response) => response,
        Err(err) => return Err(Error::GetDevices(err)),
    };

    let (all, active): (Vec<u32>, Vec<u32>) = switches
        .clone()
        .into_iter()
        .map(|switch| {
            let switch_alt = ParsedDevice::from(switch);

            match switch_alt.power {
                Some(power) => (
                    power.watts as u32,
                    if power.status {
                        power.watts as u32
                    } else {
                        0u32
                    },
                ),
                None => (0u32, 0u32),
            }
        })
        .unzip();

    let power_total = all.into_iter().sum::<u32>();
    let power_active = active.into_iter().sum::<u32>();
    let power_passive = power_total - power_active;

    let historic_data = match client.power_usage(false).await {
        Ok(response) => response,
        Err(err) => return Err(Error::GetPowerDrawData(err)),
    };

    let kwh_24_hours = kwh_total(&historic_data)?;
    let peak_24_hours = match historic_data
        .iter()
        .max_by_key(|measurement| measurement.on.watts)
    {
        Some(max) => max.on.watts,
        None => return Err(Error::NotEnoughPowerDrawData),
    };

    // Only print the table if the simple display is turned off
    if !use_simple_display {
        let mut table = Table::new(
            switches
                .into_iter()
                .map(|f| TableDevice::from(ParsedDevice::from(f)))
                .collect::<Vec<TableDevice>>(),
        );
        table.with(Style::modern().remove_horizontal()).with(
            Modify::new(Rows::first()).with(Format::content(|s| format!("\x1b[1;32m{s}\x1b[1;0m"))),
        );
        println!("{}", table);
    }

    println!(
        "=== Current Power Draw ===
  Active  \x1b[1;32m*\x1b[1;0m {:>4} W ({:>3.0} %)
  Passive \x1b[1;31m.\x1b[1;0m {:>4} W ({:>3.0} %)
  Total   Σ {:>4} W (100 %)
  ",
        power_active,
        power_active as f64 * 100.0 / power_total as f64,
        power_passive,
        power_passive as f64 * 100.0 / power_total as f64,
        power_total,
    );

    println!(
        "\n=== 24-Hour Metrics    ===
  Used    Σ {:>3.2} KWh
  Cost      {:>3.2} {}
  Peak      {:>3} W",
        kwh_24_hours,
        kwh_24_hours * config.cost_per_kwh,
        config.unit_symbol,
        peak_24_hours,
    );

    Ok(())
}

/// Analyzes the data and returns how much power has been used during the timespan of the input
/// Only accounts for the on-power (which has been used)
fn kwh_total(data: &[PowerDrawPoint]) -> Result<f64> {
    let mut data = data.iter();

    let mut prev: &PowerDrawPoint = match data.next() {
        Some(p) => p,
        None => return Err(Error::NotEnoughPowerDrawData),
    };

    let mut sum = 0.0;

    for point in data {
        let duration_minutes = (point.time - prev.time) / 1000 / 60;
        sum += point.on.watts as f64 * (duration_minutes as f64 / 60.0) / 1000.0;
        prev = point;
    }

    Ok(sum)
}
