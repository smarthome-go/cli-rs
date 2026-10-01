use anstream::print;
use log::info;
use smarthome_sdk_rs::{Client, HomescriptArg};

use super::errors::{Error, Result};
use crate::{cli::HmsArg, term};

pub async fn run_script(client: &Client, id: &str, args: &[HmsArg]) -> Result<()> {
    let args = args
        .iter()
        .map(|arg| HomescriptArg {
            key: &arg.key,
            value: &arg.value,
        })
        .collect();
    let response = term::spin(format!("Running {id}"), client.exec_homescript(id, args, false)).await?;

    if !response.success {
        let scripts = client.list_personal_homescripts().await?;
        return Err(Error::RunErrors {
            errors: response.errors,
            code: scripts
                .into_iter()
                .find(|script| script.data.id == id)
                .map(|script| script.data.code)
                .unwrap_or_default(),
            file_contents: response.file_contents,
        });
    }

    info!("Program executed successfully");
    print!("{}", response.output);
    Ok(())
}
