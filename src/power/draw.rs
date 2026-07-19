use std::usize;

use super::errors::{Error, Result};
use crate::config::PowerConfig;
use smarthome_sdk_rs::{Client, DeviceCapability, DeviceColor, HydratedDeviceResponse, PowerDrawPoint};
use tabled::{
    settings::{format::Format, object::Rows, Modify, Style},
    Table, Tabled,
};

pub struct ParsedDevice {
    pub id: String,
    pub name: String,
    pub room_id: String,
    pub power: Option<ParsedPower>,
    pub color: Option<DeviceColor>,
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
    #[tabled(display_with("Self::display_color"), rename = "Color")]
    color: Option<DeviceColor>,
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

    fn display_color(color: &Option<DeviceColor>) -> String {
        match color {
            Some(c) => format!(
                "{}██\x1b[0m #{:02x}{:02x}{:02x}",
                color_escape(c),
                c.r,
                c.g,
                c.b
            ),
            None => "\x1b[1;30mN/A\x1b[1;0m".to_string(),
        }
    }
}

/// Returns a foreground escape sequence for the given color.
/// Uses 24-bit truecolor when the terminal advertises it, otherwise
/// approximates via the 256-color cube.
fn color_escape(color: &DeviceColor) -> String {
    let truecolor = std::env::var("COLORTERM")
        .map(|v| v.contains("truecolor") || v.contains("24bit"))
        .unwrap_or(false);

    if truecolor {
        format!("\x1b[38;2;{};{};{}m", color.r, color.g, color.b)
    } else {
        format!("\x1b[38;5;{}m", ansi_256_approx(color))
    }
}

/// Maps an RGB color to the closest entry of the xterm 256-color palette
/// (using the 6x6x6 color cube and the grayscale ramp).
fn ansi_256_approx(color: &DeviceColor) -> u8 {
    fn to_cube(v: u8) -> (u8, u8) {
        // Cube channel values are 0, 95, 135, 175, 215, 255.
        let idx = if v < 48 {
            0
        } else if v < 115 {
            1
        } else {
            (v as u16 - 35) as u8 / 40
        };
        (idx, [0u8, 95, 135, 175, 215, 255][idx as usize])
    }

    let (ri, rv) = to_cube(color.r);
    let (gi, gv) = to_cube(color.g);
    let (bi, bv) = to_cube(color.b);
    let cube_dist = dist(color, rv, gv, bv);

    // Grayscale ramp: indices 232..=255 with values 8, 18, ..., 238.
    let gray_avg = (color.r as u16 + color.g as u16 + color.b as u16) / 3;
    let gray_idx = if gray_avg > 238 {
        23
    } else {
        (gray_avg.saturating_sub(3) / 10) as u8
    };
    let gray_val = 8 + 10 * gray_idx;
    let gray_dist = dist(color, gray_val, gray_val, gray_val);

    if gray_dist < cube_dist {
        232 + gray_idx
    } else {
        16 + 36 * ri + 6 * gi + bi
    }
}

fn dist(color: &DeviceColor, r: u8, g: u8, b: u8) -> u32 {
    let dr = color.r as i32 - r as i32;
    let dg = color.g as i32 - g as i32;
    let db = color.b as i32 - b as i32;
    (dr * dr + dg * dg + db * db) as u32
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

        let has_color_capability = source
            .extractions
            .config
            .capabilities
            .contains(&DeviceCapability::Color);

        Self {
            id: source.shallow.id,
            name: source.shallow.name,
            room_id: source.shallow.room_id,
            power: power_info,
            color: if has_color_capability {
                source.extractions.color
            } else {
                None
            },
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
            color: source.color,
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
