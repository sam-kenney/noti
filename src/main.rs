//! noti: A command line notification tool.
//!
//! Supports desktop notifications, and sending messages over webhooks to
//! Google Chat, Discord, and any supporting plain text using http POST.
//!
//! ## Sending a notification
//!
//! Below shows how to send a notification once a task has finished.
//! ```sh
//! # Some long running task.
//! dbt run --target ... && noti "dbt run complete"
//! ```
//!
//! This can be useful in cases where your task cannot fail, or where you only want
//! to be notified that it has finished.
//!
//! Noti also supports reading from stdin and sending notifications as lines come in.
//! Naturally this can get quite noisy, so it also features an option to filter input
//! using regex.
//!
//! ```yaml
//! # in noti.yaml
//!
//! stream:
//!   enabled: true
//!   matching: "^(WARN:.*)|^(ERROR:.*)"
//!   redirect: stdout
//! ```
//!
//! ```sh
//! # Long running task 
//! dbt run --target ... | noti
//! ```
//!
//! The above will only send notifications for inputs that start with either `WARN:`
//! or `ERROR:`.
#[deny(unsafe_code)]
mod cli;
mod commands;
mod config;
mod error;
use crate::{
    cli::{Cli, Command},
    error::Result,
};
use clap::Parser;

#[tokio::main]
async fn main() {
    let args = Cli::parse();

    let result: Result<()> = match args.command {
        Some(cmd) => match cmd {
            Command::Init { destination } => commands::init(&args.config, &destination).await,
            Command::Destination { command } => commands::destination(&command).await,
        },
        None => commands::execute(args).await,
    };

    if let Err(err) = result {
        println!("ERROR: {err}")
    }
}
