use std::{fs, path::Path};

use log::{debug, info};
use smarthome_sdk_rs::{Client, HomescriptData, HomescriptType, StatusCode};

use super::{
    errors::{Error, Result},
    workspace::Workspace,
};
use crate::term::{self, GOOD, paint};

pub async fn create_script(
    client: &Client,
    id: String,
    name: String,
    workspace: String,
) -> Result<()> {
    if Path::new(&id).exists() {
        return Err(Error::ScriptAlreadyExists(id));
    }

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

    let script = HomescriptData {
        id: id.clone(),
        name,
        description: "Created through the CLI".to_string(),
        quick_actions_enabled: false,
        scheduler_enabled: false,
        is_widget: false,
        code: String::new(),
        md_icon: "code".to_string(),
        workspace,
        type_: HomescriptType::Normal,
    };

    debug!("Creating script `{id}` at `./{id}`...");
    // The server explains validation failures (such as a duplicate ID) itself
    term::spin(format!("Creating {id}"), client.create_homescript(&script)).await?;

    // The local starter code counts as a local change, so the next push uploads it
    Workspace::create(&script, &format!("// Homescript `{id}`\n"))?;
    info!("{} script `{id}` in `./{id}`", paint(GOOD, "Created"));
    Ok(())
}

pub async fn delete_script(client: &Client, id: &str) -> Result<()> {
    debug!("Deleting script `{id}`...");
    match term::spin(format!("Deleting {id}"), client.delete_homescript(id)).await {
        Ok(_) => {
            let path = Path::new(id);
            if path.exists() {
                fs::remove_dir_all(path)?;
            }
            info!("{} script `{id}`", paint(GOOD, "Deleted"));
            Ok(())
        }
        Err(err) => Err(match err.status() {
            Some(StatusCode::UNPROCESSABLE_ENTITY) => Error::ScriptDoesNotExist(id.to_string()),
            Some(StatusCode::CONFLICT) => Error::ScriptHasDependentAutomations(id.to_string()),
            _ => Error::Smarthome(err),
        }),
    }
}
