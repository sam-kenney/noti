use crate::cli::{Cli, DestinationCommand, InitCommand};
use crate::config::{Config, Destination, Redirect};
use crate::error::{Error, Result};
use crate::models::WebhookFormat;
use regex::Regex;
use std::{
    io::{self, BufRead},
    path::PathBuf,
};

/// Send a message over webhook.
async fn dispatch_webhook(message: &str, url: &String, format: WebhookFormat) -> Result<()> {
    let client = reqwest::Client::new();
    let resp = client
        .post(url)
        .header(reqwest::header::CONTENT_TYPE, format.as_content_type())
        .body(format.format_message(message.into()));

    resp.send().await?.error_for_status()?;
    Ok(())
}

/// Send a desktop notification.
async fn dispatch_desktop(message: &str, summary: &str, persistent: bool) -> Result<()> {
    let mut notification = notify_rust::Notification::new()
        .summary(summary)
        .body(message)
        .appname("noti")
        .finalize();

    if persistent {
        notification.timeout(0);
    }

    notification.show_async().await?;
    Ok(())
}

/// Dispatch messages by listening to stdin.
///
/// Respects the `stream.matching` config if set by excluding
/// non-matching lines read from stdin.
async fn stream_and_dispatch(config: &Config) -> Result<()> {
    for line in io::stdin().lock().lines() {
        let value = line?;

        match config.stream.redirect {
            Some(Redirect::Stderr) => eprintln!("{value}"),
            Some(Redirect::Stdout) => println!("{value}"),
            None => (),
        };

        match &config.stream.matching {
            Some(pattern) => {
                let re = Regex::new(pattern)?;
                let Some(captures) = re.captures(&value) else {
                    continue;
                };

                if let Some(msg) = captures.get(0) {
                    dispatch_all(msg.as_str(), config).await?;
                }
            }
            None => dispatch_all(&value, config).await?,
        }
    }

    Ok(())
}

/// Send a message to the configured destination.
async fn dispatch(message: &str, destination: &Destination) -> Result<()> {
    match destination {
        Destination::Webhook { url, format } => {
            dispatch_webhook(message, url, format.clone()).await
        }
        Destination::Desktop {
            summary,
            persistent,
        } => dispatch_desktop(message, summary, *persistent).await,
    }
}

/// Send a message to all configured destinations.
async fn dispatch_all(message: &str, config: &Config) -> Result<()> {
    let tasks = config
        .destination
        .iter()
        .map(|destination| dispatch(message, destination));

    futures::future::try_join_all(tasks).await?;

    Ok(())
}

/// Program's main entrypoint.
///
/// Either sends a message immediately to the configured
/// destination, or start listening for input from stdin.
pub async fn execute(args: Cli) -> Result<()> {
    let config = Config::try_from(&args.config)?;

    match (config.stream.enabled, args.message) {
        (true, None) => stream_and_dispatch(&config).await,
        (true, Some(_)) => Err(Error::StreamAndMessage),
        (false, None) => Err(Error::NoMessage),
        (false, Some(message)) => dispatch_all(&message, &config).await,
    }
}

/// Initialise a new config file at `path`.
pub async fn init(path: &PathBuf, command: &InitCommand) -> Result<()> {
    if let Ok(true) = tokio::fs::try_exists(&path).await {
        return Err(Error::ConfigConflict(path.clone()));
    }

    let config = match command {
        InitCommand::Desktop => Config::default_desktop(),
        InitCommand::Webhook => Config::default_webhook(),
    };

    let data = serde_yaml::to_string(&config)?;
    Ok(tokio::fs::write(&path, &data).await?)
}

pub async fn destination(command: &DestinationCommand) -> Result<()> {
    match command {
        DestinationCommand::List => list_destinations().await,
    }
}

/// Print available destinations.
async fn list_destinations() -> Result<()> {
    println!("desktop");
    println!("webhook");
    Ok(())
}
