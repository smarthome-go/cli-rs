use clap::{Parser, Subcommand};

#[derive(Parser)]
#[clap(author, version, about)]
pub struct Args {
    /// Selects the target Smarthome server by the provided ID
    #[clap(short, long, value_parser)]
    pub server: Option<String>,

    /// The path where the configuration file should be located
    #[clap(short, long, value_parser)]
    pub config_path: Option<String>,

    /// Smarthome subcommands
    #[clap(subcommand)]
    pub subcommand: Command,
}

#[derive(Subcommand, PartialEq, Eq)]
pub enum Command {
    /// Power subcommands
    #[clap(subcommand)]
    Power(PowerCommand),

    /// Homescript subcommands
    #[clap(subcommand)]
    Hms(HmsCommand),

    /// Displays the file path of the CLI's configuration file
    Config,
}

#[derive(Subcommand, PartialEq, Eq)]
pub enum PowerCommand {
    /// Toggles the power state of a switch
    Toggle { switch_id: String },
    /// Activates a switch
    On { switch_id: String },
    /// Deactivates a switch
    Off { switch_id: String },
}

#[derive(Subcommand, PartialEq, Eq)]
pub enum HmsCommand {
    /// Interactive Homescript live terminal
    Repl,
    /// Script subcommands
    #[clap(subcommand)]
    Script(HmsScriptCommand),
}

#[derive(Subcommand, PartialEq, Eq)]
pub enum HmsScriptCommand {
    /// Create a new Homescript locally and on the remote
    New {
        /// A unique ID for the new script
        id: String,
        #[clap(short, long, value_parser)]
        /// A friendly name for the new script
        name: Option<String>,
        /// A workspace to be associated with the new script
        #[clap(short, long, value_parser)]
        workspace: Option<String>,
    },
    /// Clone an existing script from the server to the local FS
    Clone,
    /// Delete a script from the local FS and the server
    Del,
    /// Push local changes to the server
    Push,
    /// Pull any upstream changes to local FS
    Pull,
}
