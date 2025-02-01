mod cli;
mod commands;
mod config;
mod error;
mod models;
use clap::Parser;
use cli::{Cli, Command};

#[tokio::main]
async fn main() {
    let args = Cli::parse();

    let result = match args.command {
        Some(cmd) => match cmd {
            Command::Init { command } => commands::init(&args.config, &command).await,
        },
        None => commands::execute(args).await,
    };

    if let Err(err) = result {
        println!("ERROR: {err}")
    }
}
