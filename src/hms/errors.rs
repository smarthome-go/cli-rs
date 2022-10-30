use std::{fmt::Display, io};

pub type Result<T> = std::result::Result<T, Error>;

use smarthome_sdk_rs::{Error as SdkError, HomescriptExecError};

pub enum Error {
    Rustyline(rustyline::error::ReadlineError),
    FetchHomescript(SdkError),
    ScriptDoesNotExist(String),
    ScriptHasDependentAutomations(String),
    IO(io::Error),
    ScriptAlreadyExists(String),
    InvalidData(String),
    TomlEncode(toml::ser::Error),
    NotAWorkspace,
    LintErrors {
        errors: Vec<HomescriptExecError>,
        code: String,
        filename: String,
    },
    RunErrors(Vec<HomescriptExecError>),
    InvalidHomescript(String),
    DecodeManifest(toml::de::Error),
    CloneDirAlreadyExists(String),
    Smarthome(SdkError),
}

impl From<rustyline::error::ReadlineError> for Error {
    fn from(err: rustyline::error::ReadlineError) -> Self {
        Self::Rustyline(err)
    }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Self {
        Self::IO(err)
    }
}

impl From<toml::ser::Error> for Error {
    fn from(err: toml::ser::Error) -> Self {
        Self::TomlEncode(err)
    }
}

impl From<toml::de::Error> for Error {
    fn from(err: toml::de::Error) -> Self {
        Self::DecodeManifest(err)
    }
}

impl From<SdkError> for Error {
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
                        Self::InvalidData(message) => format!("Invalid data: {message}"),
                        Self::ScriptAlreadyExists(id) =>
                            format!("Script `{id}` already exists"),
                        Self::IO(err) =>
                            format!("Could not perform IO operation: {err}"),
                        Self::TomlEncode(err) =>
                            format!("Could not create manifest: TOML failure: {err}"),
                        Self::ScriptDoesNotExist(id) =>
                            format!("Script `{id}` does not exist or is inaccessible"),
                        Self::ScriptHasDependentAutomations(id) =>
                            format!("Automations depend on script `{id}`"),
                        Self::DecodeManifest(err) => format!("Invalid Homescript manifest (at `.hms.toml`):\n{err}\n => Clone this script again"),
                        Self::NotAWorkspace =>
                        "Not a valid Homescript directory: (missing files?)".to_string(),
                        Self::InvalidHomescript(id) => format!("Cannot perform action on script `{id}`: script does not exist or is inaccessible"),
                        Self::LintErrors{errors, code, filename} => format!("Linting discovered problems:\n{}", errors.iter().map(|error|error.display(code, filename)).collect::<Vec<String>>().join("\n\n")),
                        Self::RunErrors(errors) => format!("Homescript terminated with errors:\n{}", errors.iter().map(|error|error.to_string()).collect::<Vec<String>>().join("\n")),
                        Self::Smarthome(err) => format!("Smarthome Error: {err}"),
                        Self::CloneDirAlreadyExists(path) => format!("Cannot clone: directory at `./{path}` already exists."),
                Self::Rustyline(err) => format!("REPL error: {err}"),
                Self::FetchHomescript(err) => format!("Could not fetch Homescript: {err}"),
            }
        )
    }
}
