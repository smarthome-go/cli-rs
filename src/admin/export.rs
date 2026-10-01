use std::fs;

use anyhow::Context;
use chrono::Local;
use log::info;
use smarthome_sdk_rs::{Client, ExportRequest};

use crate::term::{self, GOOD, paint};

pub async fn export(
    client: &Client,
    include_profile_pictures: bool,
    include_cache_data: bool,
) -> anyhow::Result<()> {
    let export = term::spin(
        "Exporting configuration",
        client.export_config(&ExportRequest {
            include_profile_pictures,
            include_cache_data,
        }),
    )
    .await?;

    let filename = format!(
        "{}_{}_smarthome_export.json",
        client
            .smarthome_url
            .host_str()
            .expect("URL must have a base when request succeeded"),
        Local::now().to_rfc3339(),
    );
    fs::write(&filename, &export)
        .with_context(|| format!("could not write the export to `{filename}`"))?;

    info!(
        "{} the configuration to `{filename}` ({} bytes)",
        paint(GOOD, "Exported"),
        export.len()
    );
    Ok(())
}
