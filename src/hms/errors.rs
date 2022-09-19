use std::{fmt::Display, io};

pub type Result<T> = std::result::Result<T, Error>;

use smarthome_sdk_rs::Error as SdkError;

pub enum Error {
    Rustyline(rustyline::error::ReadlineError),
    ScriptAlreadyExists(String),
    ScriptDoesNotExist(String),
    InvalidData(String),
    IoError(io::Error),
    FetchHomescript(SdkError),
    Unknown(SdkError),
}

impl From<rustyline::error::ReadlineError> for Error {
    fn from(err: rustyline::error::ReadlineError) -> Self {
        Self::Rustyline(err)
    }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Self {
        Self::IoError(err)
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
                Self::InvalidData(message) => format!("Invalid data: {message}"),
                Self::IoError(err) => format!("IO error: {err}"),
                Self::FetchHomescript(err) => format!("Could not fetch Homescript: {err}"),
                Self::Unknown(err) => format!("Unknown Smarthome error: {err}"),
            }
        )
    }
}
