use std::fmt::Display;

pub type Result<T> = std::result::Result<T, Error>;

use smarthome_sdk_rs::Error as SdkError;

pub enum Error {
    Rustyline(rustyline::error::ReadlineError),
    ScriptAlreadyExists(String),
    ScriptDoesNotExist(String),
    InvalidData(String),
    Unknown(SdkError),
}

impl From<rustyline::error::ReadlineError> for Error {
    fn from(err: rustyline::error::ReadlineError) -> Self {
        Self::Rustyline(err)
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Rustyline(err) => format!("REPL error: {err}"),
                Self::ScriptDoesNotExist(id) =>
                    format!("Script `{id}` does not exist or is inaccessible"),
                Self::ScriptAlreadyExists(id) => format!("Script `{id}` already exists"),
                Self::Unknown(err) => format!("Unknown Smarthome error: {err}"),
                Self::InvalidData(message) => format!("Invalid data: {message}"),
            }
        )
    }
}
