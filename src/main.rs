#[doc = include_str!("../README.md")]
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
            Command::Init {
                destination,
                custom,
            } => commands::init(&args.config, &destination, custom).await,
            Command::Destination { command } => commands::destination(&args.config, &command).await,
        },
        None => commands::execute(args).await,
    };

    if let Err(err) = result {
        println!("ERROR: {err}")
    }
}
