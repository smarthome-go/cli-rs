use std::str::FromStr;

use anyhow::bail;
use clap::{Parser, Subcommand, ValueEnum};
use clap_complete::Shell;

#[derive(Parser)]
#[command(name = "shome", author, version, about)]
pub struct Args {
    /// Selects the target Smarthome server by the ID it has in the config file
    #[arg(short, long)]
    pub server: Option<String>,

    /// Path of the config file [default: $XDG_CONFIG_HOME/smarthome-cli-rs/config.toml]
    #[arg(short, long)]
    pub config_path: Option<String>,

    /// Print debug information
    #[arg(short, long, global = true)]
    pub verbose: bool,

    /// Skip the server version compatibility check when connecting
    #[arg(short, long, global = true)]
    pub no_version_check: bool,

    /// Print machine-readable JSON instead of tables (listing and status commands)
    #[arg(long, global = true)]
    pub json: bool,

    /// When to use colors
    #[arg(long, global = true, value_enum, default_value_t = ColorMode::Auto)]
    pub color: ColorMode,

    #[command(subcommand)]
    pub subcommand: Command,
}

#[derive(ValueEnum, Clone, Copy, PartialEq, Eq)]
pub enum ColorMode {
    /// Use colors when printing to a terminal and `NO_COLOR` is not set
    Auto,
    Always,
    Never,
}

#[derive(Subcommand, PartialEq, Eq)]
pub enum Command {
    /// Devices and power usage
    #[command(subcommand)]
    Power(PowerCommand),

    /// Homescript scripts, workspaces and the live REPL
    #[command(subcommand)]
    Hms(HmsCommand),

    /// Server administration
    #[command(subcommand)]
    Admin(AdminCommand),

    /// Prints the path of the CLI's configuration file
    Config,

    /// Prints a shell completion script, e.g. `shome completions zsh > ~/.zfunc/_shome`
    Completions {
        /// The shell to generate completions for
        shell: Shell,
    },
}

#[derive(Subcommand, PartialEq, Eq)]
pub enum PowerCommand {
    /// Lists your devices
    Devices {
        /// List all devices on the server instead of only yours
        #[arg(short, long)]
        all: bool,
    },
    /// Shows the current power draw and the usage of the last 24 hours
    Draw {
        /// Only show the summary, not the device table
        #[arg(short, long)]
        simple: bool,
    },
    /// Toggles the power state of devices
    Toggle {
        /// IDs of the devices to toggle (individually)
        #[arg(required = true)]
        device_ids: Vec<String>,
    },
    /// Turns devices on
    On {
        /// IDs of the devices to turn on
        #[arg(required = true)]
        device_ids: Vec<String>,
    },
    /// Turns devices off
    Off {
        /// IDs of the devices to turn off
        #[arg(required = true)]
        device_ids: Vec<String>,
    },
}

#[derive(Subcommand, PartialEq, Eq)]
pub enum HmsCommand {
    /// Interactive Homescript live terminal
    Repl,
    /// Manage scripts and local script workspaces
    #[command(subcommand)]
    Script(HmsScriptCommand),
    /// Runs a script on the server by its ID
    Run {
        /// The ID of the script to execute
        script_id: String,
        /// Arguments as `key:value` pairs, separated by commas
        #[arg(short, long, value_delimiter = ',')]
        args: Vec<HmsArg>,
    },
}

#[derive(PartialEq, Eq, Clone)]
pub struct HmsArg {
    pub key: String,
    pub value: String,
}

impl FromStr for HmsArg {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // Only split at the first colon so that values may contain colons (e.g. URLs)
        let Some((key, value)) = s.split_once(':') else {
            bail!("expected an argument of the form `key:value`")
        };
        Ok(Self {
            key: key.to_string(),
            value: value.to_string(),
        })
    }
}

#[derive(Subcommand, PartialEq, Eq)]
pub enum HmsScriptCommand {
    /// Lists your scripts on the server
    Ls,
    /// Creates a new script on the server and a workspace for it in `./<id>`
    New {
        /// A unique ID for the new script
        id: String,
        /// A friendly name for the new script [default: the ID]
        #[arg(long)]
        name: Option<String>,
        /// The workspace (group) the script belongs to on the server
        #[arg(short, long, default_value = "default")]
        workspace: String,
    },
    /// Clones scripts from the server into local workspaces (`./<id>`)
    Clone {
        /// The ID(s) of the script(s) to clone
        #[arg(required_unless_present = "all", conflicts_with = "all")]
        ids: Vec<String>,
        /// Clone all of your scripts
        #[arg(short, long)]
        all: bool,
    },
    /// Deletes scripts from the server and their local workspaces
    Del {
        /// The ID(s) of the script(s) to delete
        #[arg(required = true)]
        ids: Vec<String>,
    },
    /// Shows whether local workspaces and the server are in sync
    ///
    /// Without `--all`, the current workspace is checked; outside of a workspace,
    /// all workspaces directly below the current directory are.
    Status {
        /// Check all workspaces directly below the current directory
        #[arg(short, long)]
        all: bool,
    },
    /// Shows the changes between the server's copy and the local code
    Diff {
        /// Diff all workspaces directly below the current directory
        #[arg(short, long)]
        all: bool,
    },
    /// Uploads local changes to the server
    Push {
        /// Push even if linting fails or the server's copy changed since the last sync
        #[arg(short, long)]
        force: bool,
        /// Push all workspaces directly below the current directory
        #[arg(short, long)]
        all: bool,
    },
    /// Downloads the server's changes into the local workspace
    Pull {
        /// Pull even if this discards local changes
        #[arg(short, long)]
        force: bool,
        /// Pull all workspaces directly below the current directory
        #[arg(short, long)]
        all: bool,
    },
    /// Runs the local code of the current workspace on the server
    Run,
    /// Lints the local code of the current workspace
    Lint {
        /// Lint all of your scripts on the server instead
        #[arg(short, long)]
        all: bool,
    },
}

#[derive(Subcommand, PartialEq, Eq)]
pub enum AdminCommand {
    /// Shows the server's version, runtime and database status
    Debug,
    /// Exports the server's configuration into a JSON file in the current directory
    Export {
        /// Include profile pictures in the export
        #[arg(short, long)]
        profile_pictures: bool,
        /// Include cache data in the export
        #[arg(short, long)]
        cache_data: bool,
    },
}

#[cfg(test)]
mod tests {
    use clap::CommandFactory;

    use super::*;

    #[test]
    fn cli_definition_is_valid() {
        // Catches conflicting flags (e.g. a global short flag reused by a subcommand)
        Args::command().debug_assert();
    }

    #[test]
    fn args_may_contain_colons() {
        let arg: HmsArg = "url:http://home.edu:80".parse().unwrap();
        assert_eq!(arg.key, "url");
        assert_eq!(arg.value, "http://home.edu:80");
        assert!("no-colon".parse::<HmsArg>().is_err());
    }
}
