use clap::{Parser, Subcommand};
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
        #[command(subcommand)]
        command: InitCommand,
    },
}

#[derive(Debug, Subcommand)]
pub enum InitCommand {
    Notification,
    Webhook,
}
