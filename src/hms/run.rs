use smarthome_sdk_rs::{Client, HomescriptArg};

use super::errors::{Error, Result};
use crate::cli::HmsArg;

pub async fn run_script(client: &Client, id: &str, args: &[HmsArg]) -> Result<()> {
    let response = client
        .exec_homescript(
            id,
            args.iter()
                .map(|arg| HomescriptArg {
                    key: &arg.key,
                    value: &arg.value,
                })
                .collect(),
            false,
        )
        .await?;

    match response.success {
        true => {
            println!("program completed with exit-code {}.", response.exit_code,);
            if !response.output.is_empty() {
                println!("{}", response.output.trim_end())
            }
        }
        false => {
            let script = client.list_personal_homescripts().await?;

            return Err(Error::RunErrors {
                errors: response.errors,
                code: script
                    .into_iter()
                    .find(|script| script.data.id == id)
                    .expect("Executed script can always be found")
                    .data
                    .code,
                filename: id.to_string(),
            });
        }
    }
    Ok(())
}
