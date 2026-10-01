use anstream::println;
use smarthome_sdk_rs::Client;

use crate::term::{self, BAD, BOLD, GOOD, paint};

pub async fn debug(client: &Client, json: bool) -> anyhow::Result<()> {
    let info = term::spin("Fetching debug information", client.debug_info()).await?;

    if json {
        println!("{}", serde_json::to_string_pretty(&info)?);
        return Ok(());
    }

    let db = &info.database_stats;
    let rows = [
        ("Server", format!("{} ({})", info.server_version, info.go_version)),
        (
            "Runtime",
            format!(
                "{} CPU cores, {} goroutines, {} MiB memory",
                info.cpu_cores, info.goroutines, info.memory_usage
            ),
        ),
        (
            "Database",
            format!(
                "{} ({} open connections, {} in use, {} idle)",
                match info.database_online {
                    true => paint(GOOD, "online"),
                    false => paint(BAD, "offline"),
                },
                db.open_connections,
                db.in_use,
                db.idle
            ),
        ),
        ("Homescript", format!("{} running jobs", info.homescript_job_count)),
        (
            "Time",
            format!("{:02}:{:02}:{:02}", info.time.hours, info.time.minutes, info.time.seconds),
        ),
    ];
    for (label, value) in rows {
        println!("{} {value}", paint(BOLD, format_args!("{label:<10}")));
    }
    Ok(())
}
