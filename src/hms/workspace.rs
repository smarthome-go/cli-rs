use std::{fs, path::Path};

use log::{debug, info};
use smarthome_sdk_rs::{Client, HomescriptData};

use super::errors::WsError;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct HomescriptMetadata {
    pub id: String,
}

pub async fn push(client: &Client, lint_hook: bool) -> Result<(), WsError> {
    let manifest_path = Path::new(".hms.toml");
    if !manifest_path.exists() {
        return Err(WsError::NotAWorkspace);
    }
    let manifest: HomescriptMetadata = toml::from_str(&fs::read_to_string(&manifest_path)?)?;

    let script_path = format!("{}.hms", manifest.id);
    let script_path = Path::new(&script_path);
    if !script_path.exists() {
        return Err(WsError::NotAWorkspace);
    }
    let homescript_code = fs::read_to_string(script_path)?;
    debug!("Found valid Homescript workspace. Pushing...");
    debug!("Testing Homescript ID validity...");

    let old_data = match client
        .list_personal_homescripts()
        .await?
        .into_iter()
        .find(|script| script.data.id == manifest.id)
    {
        Some(this_script) => this_script.data,
        None => return Err(WsError::InvalidHomescript(manifest.id)),
    };

    // Running the pre-push lint hook if required
    if lint_hook {
         w q
    }

    client
        .modify_homescript(&HomescriptData {
            code: homescript_code,
            ..old_data
        })
        .await?;

    info!(
        "Successfully pushed script `{}` to {}",
        manifest.id,
        client
            .smarthome_url
            .host()
            .expect("Client can only exist with valid URL")
    );
    Ok(())
}
