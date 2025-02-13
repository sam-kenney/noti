use clap::{Parser, Subcommand, ValueEnum};
use std::path::PathBuf;

#[derive(Debug, Parser)]
pub struct Cli {
    /// The message to send when using as `cmd && noti 'Message'`.
    #[arg()]
    pub message: Option<String>,

    /// The path to the config to use.
    #[arg(long, default_value = "noti.yaml", env = "NOTI_CONFIG")]
    pub config: PathBuf,

    /// Optional subcommands.
    #[command(subcommand)]
    pub command: Option<Command>,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Initialise a new `noti.yaml` configuration file.
    Init {
        /// Where to send notifications to.
        destination: InitDestination,
        /// Initialise a custom webhook destination. Has no effect on desktop destination.
        #[arg(long)]
        custom: bool,
    },
    /// Commands about supported notification destinations.
    Destination {
        #[command(subcommand)]
        command: DestinationCommand,
    },
}

#[derive(Debug, Clone, ValueEnum)]
pub enum InitDestination {
    /// Create a new `noti.yaml` file for desktop notifications.
    Desktop,
    /// Create a new `noti.yaml` file for webhook notifications.
    Webhook,
}

#[derive(Debug, Subcommand)]
pub enum DestinationCommand {
    /// List all available destinations.
    List,
}
