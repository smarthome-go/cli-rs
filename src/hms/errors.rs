use std::{fmt::Display, io};

pub type Result<T> = std::result::Result<T, Error>;

use smarthome_sdk_rs::Error as SdkError;

pub enum Error {
    Create(CreateError),
    Delete(DeleteError),
    Rustyline(rustyline::error::ReadlineError),
    FetchHomescript(SdkError),
    DecodeManifest(toml::de::Error),
}

pub enum DeleteError {
    ScriptDoesNotExist(String),
    ScriptHasDependentAutomations(String),
    IoError(io::Error),
    Unknown(SdkError),
}

pub enum CreateError {
    ScriptAlreadyExists(String),
    InvalidData(String),
    IoError(io::Error),
    TomlEncode(toml::ser::Error),
    Unknown(SdkError),
}

impl From<CreateError> for Error {
    fn from(err: CreateError) -> Self {
        Self::Create(err)
    }
}

impl From<DeleteError> for Error {
    fn from(err: DeleteError) -> Self {
        Self::Delete(err)
    }
}

impl From<rustyline::error::ReadlineError> for Error {
    fn from(err: rustyline::error::ReadlineError) -> Self {
        Self::Rustyline(err)
    }
}

impl From<io::Error> for CreateError {
    fn from(err: io::Error) -> Self {
        Self::IoError(err)
    }
}

impl From<io::Error> for DeleteError {
    fn from(err: io::Error) -> Self {
        Self::IoError(err)
    }
}

impl From<toml::ser::Error> for CreateError {
    fn from(err: toml::ser::Error) -> Self {
        Self::TomlEncode(err)
    }
}

impl From<toml::de::Error> for Error {
    fn from(err: toml::de::Error) -> Self {
        Self::DecodeManifest(err)
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Create(err) => format!("{}", match err {
                    CreateError::InvalidData(message) => format!("Invalid data: {message}"),
                    CreateError::ScriptAlreadyExists(id) => format!("Script `{id}` already exists"),
                    CreateError::IoError(err) => format!("Could not perform IO operation: {err}"),
                    CreateError::TomlEncode(err) =>
                        format!("Could not create manifest: TOML failure: {err}"),
                    CreateError::Unknown(err) => format!("Unknown Smarthome error: {err}"),
                }),
                Self::Delete(err) => format!(
                    "Could not delete script: {}",
                    match err {
                        DeleteError::ScriptDoesNotExist(id) =>
                            format!("Script `{id}` does not exist or is inaccessible"),
                        DeleteError::ScriptHasDependentAutomations(id) =>
                            format!("Automations depend on script `{id}`"),
                        DeleteError::IoError(err) =>
                            format!("Could not perform IO operation: {err}"),
                        DeleteError::Unknown(err) => format!("Unknown Smarthome error: {err}"),
                    }
                ),
                Self::Rustyline(err) => format!("REPL error: {err}"),
                Self::FetchHomescript(err) => format!("Could not fetch Homescript: {err}"),
                Self::DecodeManifest(err) => format!("Bad manifest (at `.hms.toml`): {err}"),
            }
        )
    }
}
