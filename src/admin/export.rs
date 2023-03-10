use std::{fs, process};

use chrono::Local;
use smarthome_sdk_rs::{Client, Error};

pub async fn export(client: &Client) -> Result<(), Error> {
    let export = client.export_config().await?;
    let now = Local::now().to_rfc3339();
    let filename = format!(
        "{}_{now}_smarthome_export.json",
        client
            .smarthome_url
            .host_str()
            .expect("URL must have a base when request succeeded")
    );

    fs::write(&filename, &export).unwrap_or_else(|err| {
        eprintln!("Could not write configuration export to file: {err}");
        process::exit(1);
    });

    println!("Successfully written export to file `{filename}`, len: {} chars", export.len());

    Ok(())
}
