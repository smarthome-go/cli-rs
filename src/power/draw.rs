use smarthome_sdk_rs::{Client, PowerDrawPoint, PowerSwitch};

use crate::config::PowerConfig;

use super::errors::{Error, Result};

use tabled::{object::Rows, Format, Modify, Style, TableIteratorExt, Tabled};
#[derive(Tabled)]
struct TableSwitch {
    #[tabled(display_with("Self::display_id", args), rename = "ID")]
    id: String,
    #[tabled(rename = "Name")]
    name: String,
    #[tabled(rename = "Room ID")]
    room_id: String,
    #[tabled(display_with("Self::display_watts", args), rename = "Watts")]
    watts: u16,
    #[tabled(display_with("Self::display_power", args), rename = "Power")]
    power_on: bool,
}

impl TableSwitch {
    fn display_id(&self) -> String {
        match self.power_on {
            true => format!("\x1b[1;32m*\x1b[1;0m {}", self.id),
            false => format!("\x1b[1;31m.\x1b[1;0m {}", self.id),
        }
    }
    fn display_watts(&self) -> String {
        match self.power_on {
            true => format!("\x1b[1;32m{}\x1b[1;0m", self.watts),
            false => self.watts.to_string(),
        }
    }
    fn display_power(&self) -> String {
        match self.power_on {
            true => "on".to_string(),
            false => "off".to_string(),
        }
    }
}

impl From<PowerSwitch> for TableSwitch {
    fn from(source: PowerSwitch) -> Self {
        Self {
            id: source.id,
            name: source.name,
            room_id: source.room_id,
            power_on: source.power_on,
            watts: source.watts,
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
        Err(err) => return Err(Error::GetSwitches(err)),
    };

    let (all, active): (Vec<u32>, Vec<u32>) = switches
        .iter()
        .map(|switch| {
            (switch.watts as u32, {
                if switch.power_on {
                    switch.watts as u32
                } else {
                    0
                }
            })
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
        let table = switches
            .into_iter()
            .map(TableSwitch::from)
            .table()
            .with(Style::modern().off_horizontal())
            .with(
                Modify::new(Rows::first()).with(Format::new(|s| format!("\x1b[1;32m{s}\x1b[1;0m"))),
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
