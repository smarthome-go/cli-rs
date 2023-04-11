use std::{
    fs::{self, File},
    io::Write,
    path::Path,
};

use log::{debug, info, warn};
use smarthome_sdk_rs::{Client, Homescript, HomescriptData};

use super::errors::{Error, Result};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct HomescriptMetadata {
    pub id: String,
}

pub async fn pull(client: &Client) -> Result<()> {
    // Check if the Homescript manifest exists
    let manifest_path = Path::new(".hms.toml");
    if !manifest_path.exists() {
        return Err(Error::NotAWorkspace);
    }
    let manifest: HomescriptMetadata = toml::from_str(&fs::read_to_string(manifest_path)?)?;
    // Check if the required Homescript file exists
    let homescript_path = format!("{}.hms", manifest.id);
    let homescript_path = Path::new(&homescript_path);
    if !homescript_path.exists() {
        return Err(Error::NotAWorkspace);
    }
    // Read the old code
    let old_homescript_code = fs::read_to_string(homescript_path)?;
    debug!("Found valid Homescript workspace. Pulling...");
    debug!("Testing Homescript ID validity...");
    // Download the current data
    let data = match client
        .list_personal_homescripts()
        .await?
        .into_iter()
        .find(|script| script.data.id == manifest.id)
    {
        Some(this_script) => this_script.data,
        None => return Err(Error::InvalidHomescript(manifest.id)),
    };
    // Check if there are changes
    if data.code == old_homescript_code {
        info!("Already up to date.");
        return Ok(());
    }
    // Write the changes to disk
    fs::write(homescript_path, data.code)?;
    info!(
        "Successfully pulled changes of `{}` from {}",
        manifest.id,
        client
            .smarthome_url
            .host()
            .expect("Client can only exist with valid URL")
    );
    Ok(())
}

pub async fn exec_current_script(client: &Client, lint: bool) -> Result<()> {
    // Check if the manifest exists
    let manifest_path = Path::new(".hms.toml");
    if !manifest_path.exists() {
        return Err(Error::NotAWorkspace);
    }
    // Read & parse the manifest
    let manifest: HomescriptMetadata = toml::from_str(&fs::read_to_string(manifest_path)?)?;
    // Check if the Homescript file exists
    let homescript_path = format!("{}.hms", manifest.id);
    let homescript_path = Path::new(&homescript_path);
    if !homescript_path.exists() {
        return Err(Error::NotAWorkspace);
    }
    // Reads the current code
    let homescript_code = fs::read_to_string(homescript_path)?;
    debug!("Found Homescript workspace. Executing...");
    let response = client
        .exec_homescript_code(&homescript_code, vec![], lint)
        .await?;

    match response.success {
        true => {
            println!(
                "{}",
                if lint {
                    "linting discovered no problems.".to_string()
                } else {
                    format!("program completed with exit-code {}.", response.exit_code)
                },
            );
            if !lint && !response.output.is_empty() {
                println!("{}", response.output.trim_end())
            }
            if lint {
                println!(
                    "{}",
                    response
                        .errors
                        .iter()
                        .map(|diagnostic| {
                            let mut code = homescript_code.clone();
                            if let Some(new_code) =
                                response.file_contents.get(&diagnostic.span.filename)
                            {
                                code = new_code.clone();
                            }
                            diagnostic.display(&code)
                        })
                        .collect::<Vec<String>>()
                        .join("\n\n")
                )
            }
        }
        false => {
            return Err(if lint {
                Error::LintErrors {
                    errors: response.errors,
                    code: homescript_code,
                    file_contents: response.file_contents,
                }
            } else {
                Error::RunErrors {
                    errors: response.errors,
                    code: homescript_code,
                    file_contents: response.file_contents,
                }
            })
        }
    }
    Ok(())
}

pub async fn push(client: &Client, lint_hook: bool, force: bool) -> Result<()> {
    // Check if the manifest exists
    let manifest_path = Path::new(".hms.toml");
    if !manifest_path.exists() {
        return Err(Error::NotAWorkspace);
    }
    // Read & parse the manifest
    let manifest: HomescriptMetadata = toml::from_str(&fs::read_to_string(manifest_path)?)?;
    // Check if the Homescript file exists
    let homescript_path = format!("{}.hms", manifest.id);
    let homescript_path = Path::new(&homescript_path);
    if !homescript_path.exists() {
        return Err(Error::NotAWorkspace);
    }
    // Read the current code
    let homescript_code = fs::read_to_string(homescript_path)?;
    debug!("Found valid Homescript workspace. Pushing...");
    debug!("Testing Homescript ID validity...");
    // Get the upstream state of the script
    let old_data = match client
        .list_personal_homescripts()
        .await?
        .into_iter()
        .find(|script| script.data.id == manifest.id)
    {
        Some(this_script) => this_script.data,
        None => return Err(Error::InvalidHomescript(manifest.id)),
    };
    // Check if there are changes
    if old_data.code == homescript_code {
        info!("Already up to date.");
        return Ok(());
    }
    // Running the pre-push lint hook if required
    if lint_hook {
        match client
            .exec_homescript_code(&homescript_code, vec![], true)
            .await
        {
            Ok(response) => match response.success {
                true => debug!("Linting discovered no problems"),
                false if !force => {
                    return Err(Error::LintErrors {
                        errors: response.errors,
                        code: homescript_code,
                        file_contents: response.file_contents,
                    })
                }
                false => warn!("Linting discovered errors: force-pushing to remote"),
            },
            Err(err) => return Err(Error::Smarthome(err)),
        }
    }
    // Push the changes
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

pub async fn clone(script_ids: &Vec<String>, clone_all: bool, client: &Client) -> Result<()> {
    // Fetch the personal scripts
    let personal_scripts: Vec<Homescript> = client
        .list_personal_homescripts()
        .await?
        .into_iter()
        .filter(|script| script_ids.contains(&script.data.id))
        .collect();

    // Clone all scripts if required
    if clone_all {
        for script in &personal_scripts {
            // Clone the current iteration script
            clone_to_fs(&script.data)?
        }
        return Ok(());
    }

    // Iterate over the ids which should be cloned
    for script_id in script_ids {
        // Select the script from the fetched scripts
        let script_to_clone = match personal_scripts
            .iter()
            .find(|item| item.data.id == *script_id)
        {
            Some(script) => script,
            None => return Err(Error::ScriptDoesNotExist(script_id.to_string())),
        };
        // Clone the current iteration script
        clone_to_fs(&script_to_clone.data)?
    }
    Ok(())
}

pub fn clone_to_fs(script_data: &HomescriptData) -> Result<()> {
    debug!("Cloning script `{}`...", script_data.id);
    let path = script_data.id.to_string();
    let path = Path::new(&path);

    if path.exists() {
        return Err(Error::CloneDirAlreadyExists(script_data.id.to_string()));
    }

    fs::create_dir_all(path)?;
    let mut homescript_file = File::create(path.join(format!("{}.hms", script_data.id)))?;
    homescript_file.write_all(script_data.code.as_bytes())?;

    let mut metadate_file = File::create(path.join(".hms.toml"))?;
    metadate_file.write_all(
        toml::to_string_pretty(&HomescriptMetadata {
            id: script_data.id.clone(),
        })?
        .as_bytes(),
    )?;
    info!("Successfully cloned script `{}`", script_data.id);
    Ok(())
}
