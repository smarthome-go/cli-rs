use std::{collections::HashMap, fmt::Display, io};

pub type Result<T> = std::result::Result<T, Error>;

use smarthome_sdk_rs::{Error as SdkError, HomescriptExecError};

pub enum Error {
    Rustyline(rustyline::error::ReadlineError),
    ScriptDoesNotExist(String),
    ScriptHasDependentAutomations(String),
    IO(io::Error),
    ScriptAlreadyExists(String),
    InvalidData(String),
    TomlEncode(toml::ser::Error),
    Json(serde_json::Error),
    NotAWorkspace,
    NoWorkspaces,
    LintErrors {
        errors: Vec<HomescriptExecError>,
        code: String,
        file_contents: HashMap<String, String>,
    },
    RunErrors {
        errors: Vec<HomescriptExecError>,
        code: String,
        file_contents: HashMap<String, String>,
    },
    CannotRunDriver(String),
    InvalidHomescript(String),
    DecodeManifest(toml::de::Error),
    CloneDirAlreadyExists(String),
    /// The local workspace and the server both changed, or the operation would discard changes
    Conflict {
        id: String,
        message: String,
    },
    /// Some workspaces of a multi-workspace operation failed (their errors were already reported)
    Failed {
        failed: usize,
        total: usize,
    },
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

impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Self {
        Self::Json(err)
    }
}

impl From<SdkError> for Error {
    fn from(err: SdkError) -> Self {
        Self::Smarthome(err)
    }
}

/// Renders Homescript diagnostics, using the source of the file each one refers to
pub fn render_diagnostics(
    errors: &[HomescriptExecError],
    code: &str,
    file_contents: &HashMap<String, String>,
) -> String {
    errors
        .iter()
        .map(|error| {
            let code = file_contents
                .get(&error.span.filename)
                .map(String::as_str)
                .unwrap_or(code);
            error.display(code)
        })
        .collect::<Vec<String>>()
        .join("\n\n")
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidData(message) => write!(f, "invalid data: {message}"),
            Self::ScriptAlreadyExists(id) => write!(f, "script `{id}` already exists"),
            Self::IO(err) => write!(f, "IO operation failed: {err}"),
            Self::TomlEncode(err) => write!(f, "could not create the manifest: {err}"),
            Self::Json(err) => write!(f, "could not encode JSON: {err}"),
            Self::ScriptDoesNotExist(id) => write!(f, "script `{id}` does not exist or is inaccessible"),
            Self::ScriptHasDependentAutomations(id) => {
                write!(f, "script `{id}` cannot be deleted: automations depend on it")
            }
            Self::DecodeManifest(err) => write!(
                f,
                "invalid workspace manifest (`.hms.toml`):\n{err}\n => Clone this script again"
            ),
            Self::NotAWorkspace => write!(
                f,
                "not a script workspace: `.hms.toml` or the script's code is missing\n => Run this in a directory created by `shome hms script clone`, or use `--all`"
            ),
            Self::NoWorkspaces => write!(
                f,
                "no script workspaces found in or directly below the current directory"
            ),
            Self::InvalidHomescript(id) => write!(
                f,
                "script `{id}` does not exist on the server or is inaccessible"
            ),
            Self::CannotRunDriver(id) => write!(
                f,
                "`{id}` is a driver, which cannot be executed directly"
            ),
            Self::LintErrors {
                errors,
                code,
                file_contents,
            } => write!(
                f,
                "linting discovered problems:\n{}",
                render_diagnostics(errors, code, file_contents)
            ),
            Self::RunErrors {
                errors,
                code,
                file_contents,
            } => write!(
                f,
                "Homescript terminated with errors:\n{}",
                render_diagnostics(errors, code, file_contents)
            ),
            Self::Conflict { id, message } => write!(f, "`{id}`: {message}"),
            Self::Failed { failed, total } => write!(f, "{failed} of {total} scripts failed"),
            Self::Smarthome(err) => write!(f, "{err}"),
            Self::CloneDirAlreadyExists(path) => {
                write!(f, "cannot clone: the directory `./{path}` already exists")
            }
            Self::Rustyline(err) => write!(f, "REPL error: {err}"),
        }
    }
}
