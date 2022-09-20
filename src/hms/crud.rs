use std::{
    fs::{self, File},
    io::Write,
    path::Path,
};

use crate::hms::{errors::CreateError, workspace::HomescriptMetadata};

use super::errors::DeleteError;
use log::{debug, info};
use reqwest::StatusCode;
use smarthome_sdk_rs::{Client, Error as SdkError, HomescriptData};


pub async fn create_script(
    client: &Client,
    id: String,
    name: String,
    workspace: String,
) -> Result<(), CreateError> {
    let path = format!("{id}");
    let path = Path::new(&path);

    if path.exists() {
        return Err(CreateError::ScriptAlreadyExists(id));
    }

    if id.contains(' ') || id.len() > 30 {
        return Err(CreateError::InvalidData(
            "id must not contain whitespaces and shall not exceed 30 characters".to_string(),
        ));
    }
    if name.len() > 30 {
        return Err(CreateError::InvalidData(
            "name must not exceed 30 characters".to_string(),
        ));
    }
    if workspace.len() > 50 {
        return Err(CreateError::InvalidData(
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
            fs::create_dir_all(&path)?;
            let mut homescript_file = File::create(path.join(format!("{id}.hms")))?;
            homescript_file.write_fmt(format_args!("# Homescript `{id}`\n"))?;

            let mut metadate_file = File::create(path.join(".hms.toml"))?;
            metadate_file.write_all(&toml::to_vec(&HomescriptMetadata { id: id.clone() })?)?;

            info!("Successfully created script `{id}`");
            Ok(())
        }
        Err(err) => Err(match err {
            SdkError::Smarthome(code) => match code {
                StatusCode::UNPROCESSABLE_ENTITY => {
                    CreateError::ScriptAlreadyExists(id.to_string())
                }
                code => CreateError::Unknown(SdkError::Smarthome(code)),
            },
            _ => CreateError::Unknown(err),
        }),
    }
}

pub async fn delete_script(client: &Client, id: &str) -> Result<(), DeleteError> {
    debug!("Deleting script `{id}`...");
    match client.delete_homescript(id).await {
        Ok(_) => {
            let path = format!("./{id}");
            let path = Path::new(path.as_str());

            if path.exists() {
                fs::remove_dir_all(path)?;
            }
            info!("Successfully deleted script `{id}`");
            Ok(())
        }
        Err(err) => Err(match err {
            SdkError::Smarthome(code) => match code {
                StatusCode::UNPROCESSABLE_ENTITY => DeleteError::ScriptDoesNotExist(id.to_string()),
                StatusCode::CONFLICT => DeleteError::ScriptHasDependentAutomations(id.to_string()),
                code => DeleteError::Unknown(SdkError::Smarthome(code)),
            },
            _ => DeleteError::Unknown(err),
        }),
    }
}
