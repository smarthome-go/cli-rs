use std::{fmt::Display, io};

pub type Result<T> = std::result::Result<T, Error>;

use smarthome_sdk_rs::Error as SdkError;

pub enum Error {
    Create(CreateError),
    Delete(DeleteError),
    Ws(WsError),
    Rustyline(rustyline::error::ReadlineError),
    FetchHomescript(SdkError),
}

pub enum DeleteError {
    ScriptDoesNotExist(String),
    ScriptHasDependentAutomations(String),
    IO(io::Error),
    Unknown(SdkError),
}

pub enum CreateError {
    ScriptAlreadyExists(String),
    InvalidData(String),
    IO(io::Error),
    TomlEncode(toml::ser::Error),
    Unknown(SdkError),
}

pub enum WsError {
    NotAWorkspace,
    InvalidWorkspace,
    InvalidHomescript(String),
    IO(io::Error),
    DecodeManifest(toml::de::Error),
    Smarthome(SdkError),
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

impl From<WsError> for Error {
    fn from(err: WsError) -> Self {
        Self::Ws(err)
    }
}

impl From<rustyline::error::ReadlineError> for Error {
    fn from(err: rustyline::error::ReadlineError) -> Self {
        Self::Rustyline(err)
    }
}

impl From<io::Error> for CreateError {
    fn from(err: io::Error) -> Self {
        Self::IO(err)
    }
}

impl From<io::Error> for DeleteError {
    fn from(err: io::Error) -> Self {
        Self::IO(err)
    }
}

impl From<toml::ser::Error> for CreateError {
    fn from(err: toml::ser::Error) -> Self {
        Self::TomlEncode(err)
    }
}

impl From<toml::de::Error> for WsError {
    fn from(err: toml::de::Error) -> Self {
        Self::DecodeManifest(err)
    }
}

impl From<io::Error> for WsError {
    fn from(err: io::Error) -> Self {
        Self::IO(err)
    }
}

impl From<SdkError> for WsError {
    fn from(err: SdkError) -> Self {
        Self::Smarthome(err)
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Create(err) => format!(
                    "{}",
                    match err {
                        CreateError::InvalidData(message) => format!("Invalid data: {message}"),
                        CreateError::ScriptAlreadyExists(id) =>
                            format!("Script `{id}` already exists"),
                        CreateError::IO(err) =>
                            format!("Could not perform IO operation: {err}"),
                        CreateError::TomlEncode(err) =>
                            format!("Could not create manifest: TOML failure: {err}"),
                        CreateError::Unknown(err) => format!("Unknown Smarthome error: {err}"),
                    }
                ),
                Self::Delete(err) => format!(
                    "Could not delete script: {}",
                    match err {
                        DeleteError::ScriptDoesNotExist(id) =>
                            format!("Script `{id}` does not exist or is inaccessible"),
                        DeleteError::ScriptHasDependentAutomations(id) =>
                            format!("Automations depend on script `{id}`"),
                        DeleteError::IO(err) =>
                            format!("Could not perform IO operation: {err}"),
                        DeleteError::Unknown(err) => format!("Unknown Smarthome error: {err}"),
                    }
                ),
                Self::Ws(err) => match err {
                        WsError::InvalidWorkspace => format!("Corrupt Homescript workspace: some files are broken:\n => Clone this script again"),
                        WsError::DecodeManifest(err) => format!("Invalid Homescript manifest (at `.hms.toml`):\n{err}\n => Clone this script again"),
                        WsError::IO(err) => format!("Could not read or write from / to workspace: {err}"),
                        WsError::NotAWorkspace =>
                        format!("Not a valid Homescript directory: (missing files?)"),
                        WsError::Smarthome(err) => format!("Smarthome Error: {err}"),
                        WsError::InvalidHomescript(id) => format!("Cannot perform action on script `{id}`: script does not exist or is inaccessible"),
                },
                Self::Rustyline(err) => format!("REPL error: {err}"),
                Self::FetchHomescript(err) => format!("Could not fetch Homescript: {err}"),
            }
        )
    }
}
