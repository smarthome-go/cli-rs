use anstream::println;
use serde::{Serialize, Serializer};
use smarthome_sdk_rs::{Client, DeviceCapability, DeviceColor, HydratedDeviceResponse, PowerDrawPoint};
use tabled::Tabled;

use super::errors::{Error, Result};
use crate::{
    config::PowerConfig,
    term::{self, BAD, BOLD, DIM, GOOD, paint},
};

#[derive(Serialize)]
pub struct ParsedDevice {
    pub id: String,
    pub name: String,
    pub room_id: String,
    pub power: Option<ParsedPower>,
    #[serde(serialize_with = "serialize_color")]
    pub color: Option<DeviceColor>,
}

#[derive(Serialize)]
pub struct ParsedPower {
    #[serde(rename = "on")]
    pub status: bool,
    pub watts: usize,
}

fn hex(color: &DeviceColor) -> String {
    format!("#{:02x}{:02x}{:02x}", color.r, color.g, color.b)
}

fn serialize_color<S: Serializer>(color: &Option<DeviceColor>, serializer: S) -> std::result::Result<S::Ok, S::Error> {
    color.as_ref().map(hex).serialize(serializer)
}

#[derive(Tabled)]
pub struct DeviceRow {
    #[tabled(rename = "ID")]
    id: String,
    #[tabled(rename = "Name")]
    name: String,
    #[tabled(rename = "Room ID")]
    room_id: String,
    #[tabled(display = "display_watts", rename = "Watts")]
    watts: Option<usize>,
    #[tabled(display = "display_power", rename = "Power")]
    power_on: Option<bool>,
    #[tabled(display = "display_color", rename = "Color")]
    color: Option<DeviceColor>,
}

fn not_available() -> String {
    paint(DIM, "N/A")
}

fn display_power(power_on: &Option<bool>) -> String {
    match *power_on {
        Some(true) => paint(GOOD, "ON"),
        Some(false) => paint(BAD, "OFF"),
        None => not_available(),
    }
}

fn display_watts(watts: &Option<usize>) -> String {
    match watts {
        Some(watts) => watts.to_string(),
        None => not_available(),
    }
}

fn display_color(color: &Option<DeviceColor>) -> String {
    match color {
        Some(color) => format!("{}██\x1b[0m {}", color_escape(color), hex(color)),
        None => not_available(),
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
        let capabilities = &source.extractions.config.capabilities;
        let has_power_capability = capabilities.contains(&DeviceCapability::Power);
        let has_color_capability = capabilities.contains(&DeviceCapability::Color);

        let power = match (has_power_capability, source.extractions.power_information) {
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
            power,
            color: match has_color_capability {
                true => source.extractions.color,
                false => None,
            },
        }
    }
}

impl From<ParsedDevice> for DeviceRow {
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

#[derive(Serialize)]
struct PowerDrawReport {
    devices: Vec<ParsedDevice>,
    current: CurrentDraw,
    last_24_hours: Last24Hours,
}

#[derive(Serialize)]
struct CurrentDraw {
    active_watts: usize,
    passive_watts: usize,
    total_watts: usize,
}

#[derive(Serialize)]
struct Last24Hours {
    kwh: f64,
    cost: f64,
    currency: char,
    peak_watts: usize,
}

pub async fn power_draw(
    client: &Client,
    config: &PowerConfig,
    use_simple_display: bool,
    json: bool,
) -> Result<()> {
    let devices: Vec<ParsedDevice> = term::spin("Fetching devices", client.all_switches())
        .await
        .map_err(Error::GetDevices)?
        .into_iter()
        .map(ParsedDevice::from)
        .collect();

    let total_watts: usize = devices.iter().filter_map(|d| d.power.as_ref()).map(|p| p.watts).sum();
    let active_watts: usize = devices
        .iter()
        .filter_map(|d| d.power.as_ref())
        .filter(|p| p.status)
        .map(|p| p.watts)
        .sum();
    let current = CurrentDraw {
        active_watts,
        passive_watts: total_watts - active_watts,
        total_watts,
    };

    let historic_data = term::spin("Fetching power usage", client.power_usage(false))
        .await
        .map_err(Error::GetPowerDrawData)?;
    let kwh = kwh_total(&historic_data)?;
    let last_24_hours = Last24Hours {
        kwh,
        cost: kwh * config.cost_per_kwh,
        currency: config.unit_symbol,
        peak_watts: historic_data
            .iter()
            .map(|measurement| measurement.on.watts)
            .max()
            .ok_or(Error::NotEnoughPowerDrawData)?,
    };

    if json {
        let report = PowerDrawReport {
            devices,
            current,
            last_24_hours,
        };
        println!("{}", serde_json::to_string_pretty(&report)?);
        return Ok(());
    }

    // Only print the table if the simple display is turned off
    if !use_simple_display {
        println!("{}\n", term::table(devices.into_iter().map(DeviceRow::from)));
    }

    let percent = |watts: usize| match current.total_watts {
        0 => 0.0,
        total => watts as f64 * 100.0 / total as f64,
    };
    println!(
        "{}
  Active  {} {:>4} W ({:>3.0} %)
  Passive {} {:>4} W ({:>3.0} %)
  Total   Σ {:>4} W (100 %)

{}
  Used    Σ {:>3.2} kWh
  Cost      {:>3.2} {}
  Peak      {:>3} W",
        paint(BOLD, "Current power draw"),
        paint(GOOD, "*"),
        current.active_watts,
        percent(current.active_watts),
        paint(BAD, "."),
        current.passive_watts,
        percent(current.passive_watts),
        current.total_watts,
        paint(BOLD, "Last 24 hours"),
        last_24_hours.kwh,
        last_24_hours.cost,
        last_24_hours.currency,
        last_24_hours.peak_watts,
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
