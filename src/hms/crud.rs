use std::{fs, path::Path};

use super::errors::{Error, Result};
use log::{debug, info};
use reqwest::StatusCode;
use smarthome_sdk_rs::{Client, Error as SdkError, HomescriptData};

pub async fn delete_script(client: &Client, id: &str) -> Result<()> {
    debug!("Deleting script `{id}`...");
    match client.delete_homescript(id).await {
        Ok(_) => {
            info!("Successfully deleted script `{id}`");

            let path = format!("./{id}");
            let path = Path::new(path.as_str());

            if path.exists() {
                fs::remove_dir_all(path)?;
            }
            Ok(())
        }
        Err(err) => Err(match err {
            SdkError::Smarthome(code) => match code {
                StatusCode::UNPROCESSABLE_ENTITY => Error::ScriptDoesNotExist(id.to_string()),
                code => Error::Unknown(SdkError::Smarthome(code)),
            },
            _ => Error::Unknown(err),
        }),
    }
}

pub async fn create_script(
    client: &Client,
    id: String,
    name: String,
    workspace: String,
) -> Result<()> {
    if id.contains(' ') || id.len() > 30 {
        return Err(Error::InvalidData(
            "id must not contain whitespaces and shall not exceed 30 characters".to_string(),
        ));
    }
    if name.len() > 30 {
        return Err(Error::InvalidData(
            "name must not exceed 30 characters".to_string(),
        ));
    }
    if workspace.len() > 50 {
        return Err(Error::InvalidData(
            "workspace must not exceed 50 characters".to_string(),
        ));
    }
    debug!("Creating script `{id}` at `./{id}`...");
    match client
        .create_homescript(&HomescriptData {
            id: id.clone(),
            name,
            description: "".to_string(),
            quick_actions_enabled: false,
            scheduler_enabled: false,
            code: "".to_string(),
            md_icon: "code".to_string(),
            workspace,
        })
        .await
    {
        Ok(_) => {
            info!("Successfully created script `{id}`");
            Ok(())
        }
        Err(err) => Err(match err {
            SdkError::Smarthome(code) => match code {
                StatusCode::UNPROCESSABLE_ENTITY => Error::ScriptAlreadyExists(id.to_string()),
                code => Error::Unknown(SdkError::Smarthome(code)),
            },
            _ => Error::Unknown(err),
        }),
    }
}
