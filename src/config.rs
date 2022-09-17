use std::{
    env,
    fmt::Display,
    fs::{self, File},
    io::{self, Write},
    path::Path,
};

use log::debug;
use serde::{Deserialize, Serialize};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    IO(io::Error),
    Parse(toml::de::Error),
    Validate(ValidateError),
}

impl From<io::Error> for Error {
    fn from(e: io::Error) -> Self {
        Self::IO(e)
    }
}

impl From<toml::de::Error> for Error {
    fn from(e: toml::de::Error) -> Self {
        Self::Parse(e)
    }
}

impl From<ValidateError> for Error {
    fn from(err: ValidateError) -> Self {
        Self::Validate(err)
    }
}

#[derive(Debug)]
pub enum ValidateError {
    // If a token and username or password is specified at once
    // The string contains the server's id in order to display it in an error message
    AmbiguousAuth(String),
    // An ID is used more than one time thus defeating the purpose of an ID
    // Holds the value of the duplicate ID
    DuplicateID(String),
    // The token is not well-formed and is likey bad, holds the token and the ID of the server
    InvalidToken {
        server_id: String,
        token: String,
        message: &'static str,
    },
    // No Username is present whilst the token is empty, holds the ID of the affected server
    EmptyUserName(String),
    NoServers,
}

impl Display for ValidateError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self {
            Self::AmbiguousAuth(id) => format!("Ambiguous authentication at server `{id}`: username or password specified whilst token is not empty"),
            Self::DuplicateID(id) => format!("Duplicate server ID: the ID `{id}` must be unique"),
            Self::InvalidToken { server_id, token, message } => format!("Malformed access token: token `{token}` at server `{server_id}` is invalid: {message}"),
            Self::EmptyUserName(id) => format!("No authentication provided for server `{id}`: token and username are both empty"),
            Self::NoServers => "No servers specified: at least one (default) server must be specified to use this CLI".to_string(),
        })
    }
}

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub homescript: HomescriptConfig,
    pub power: PowerConfig,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub servers: Vec<SmarthomeServer>,
}

#[derive(Serialize, Deserialize)]
pub struct PowerConfig {
    pub unit_symbol: char,
    pub cost_per_kwh: f64,
}

#[derive(Serialize, Deserialize)]
pub struct HomescriptConfig {
    pub lint_on_push: bool,
    pub use_repl_history: bool,
}

#[derive(Serialize, Deserialize)]
pub struct SmarthomeServer {
    pub id: String,
    pub url: String,
    pub username: String,
    pub password: String,
    pub token: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            homescript: HomescriptConfig::default(),
            power: PowerConfig::default(),
            servers: vec![SmarthomeServer::default()],
        }
    }
}

impl Default for HomescriptConfig {
    fn default() -> Self {
        Self {
            lint_on_push: true,
            use_repl_history: true,
        }
    }
}

impl Default for PowerConfig {
    fn default() -> Self {
        Self {
            unit_symbol: '€',
            cost_per_kwh: 0.3,
        }
    }
}

impl Default for SmarthomeServer {
    fn default() -> Self {
        Self {
            id: "default".to_string(),
            url: "http://smarthome.box".to_string(),
            username: String::new(),
            password: String::new(),
            token: "-".repeat(32),
        }
    }
}

pub fn file_path() -> Option<String> {
    match env::var("HOME") {
        Ok(home) => {
            if let Ok(xdg_home) = env::var("XDG_CONFIG_HOME") {
                Some(format!("{}/smarthome-cli-rs/config.toml", xdg_home))
            } else {
                Some(format!("{}/.config/smarthome-cli-rs/config.toml", home))
            }
        }
        Err(_) => None,
    }
}

pub fn read_config(file_path: &str) -> Result<Option<Config>> {
    // Either read or create a configuration file based on it's current existence
    let path = Path::new(file_path);
    match &path.exists() {
        true => {
            // The file exists, it can be read
            debug!("Found existing config file at {file_path}");
            let content = fs::read_to_string(&path)?;
            let config = toml::from_str(&content)?;
            // Validate the contents of the config file
            Ok(Some(validate_config(config)?))
        }
        false => {
            // The file does not exist, therefore create a new one
            fs::create_dir_all(&path.parent().unwrap())?;
            let mut file = File::create(path)?;
            file.write_all(include_bytes!("default_config.toml"))?;
            // In case a few new struct fields must be serialized
            /* file.write_all(
                toml::to_string_pretty(&Config::default())
                    .unwrap()
                    .as_bytes(),
            );
            */
            Ok(None)
        }
    }
}

fn validate_config(config: Config) -> std::result::Result<Config, ValidateError> {
    let mut ids: Vec<&str> = Vec::with_capacity(config.servers.len());
    if config.servers.is_empty() {
        return Err(ValidateError::NoServers);
    }
    for server in &config.servers {
        // Validate that every ID is unique
        if ids.contains(&server.id.as_str()) {
            return Err(ValidateError::DuplicateID(server.id.clone()));
        }
        ids.push(&server.id);
        match server.token.is_empty() {
            true => {
                // Validate that there is some form of authentication
                if server.username.is_empty() {
                    return Err(ValidateError::EmptyUserName(server.id.clone()));
                }
            }
            false => {
                // Validate that the token is well-formed
                if server.token.len() != 32 {
                    return Err(ValidateError::InvalidToken {
                        token: server.token.clone(),
                        server_id: server.id.clone(),
                        message: "Token does not have the length of 32 characters.",
                    });
                }
                if server.token.contains(' ') || !server.token.is_ascii() {
                    return Err(ValidateError::InvalidToken {
                        token: server.token.clone(),
                        server_id: server.id.clone(),
                        message: "Token may not contain whitespaces or non-ASCII characters",
                    });
                }

                // Validate that the authentication mode is unambiguous
                if !server.username.is_empty() || !server.password.is_empty() {
                    return Err(ValidateError::AmbiguousAuth(server.id.clone()));
                }
            }
        }
    }
    Ok(config)
}
