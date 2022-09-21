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

    /// If set, more information will be printed to the console
    #[clap(short, long, value_parser, global = true)]
    pub verbose: bool,

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
    /// Shows the user's personal switches
    Switches {
        #[clap(short, long, value_parser)]
        /// Shows all switches which are present on the Smarthome-server
        all: bool,
    },
    /// Displays power current power draw and a historic summary
    Draw {
        #[clap(short, long, value_parser)]
        /// Hides the table and only shows the most relevant information
        simple: bool,
    },
    /// Toggles the power state of a switch
    Toggle {
        /// A list of switch-ids to toggle (individually)
        #[clap(required = true)]
        switch_ids: Vec<String>,
    },
    /// Activates a switch
    On {
        /// A list of switch-ids to activate
        #[clap(required = true)]
        switch_ids: Vec<String>,
    },
    /// Deactivates a switch
    Off {
        /// A list of switch-id,s to deactivate
        #[clap(required = true)]
        switch_ids: Vec<String>,
    },
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
    /// Displays a list of personal Homescripts
    Ls,
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
    Clone {
        /// The ID(s) of the script(s) to be cloned
        #[clap(required = true, conflicts_with = "all")]
        ids: Vec<String>,

        #[clap(short, long, value_parser)]
        all: bool,
    },
    /// Delete a script from the local FS and the server
    Del {
        /// The ID(s) of the script(s) to be deleted
        #[clap(required = true)]
        ids: Vec<String>,
    },
    /// Push local changes to the server
    Push,
    /// Pull any upstream changes to local FS
    Pull,
    /// Runs the Homescript code of a local script
    Run,
    /// Lints the Homescript code of a local script
    Lint,
}
