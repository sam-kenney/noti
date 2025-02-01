use crate::cli::{Cli, InitCommand};
use crate::config::{Config, Notification, Redirect, Webhook};
use crate::error::{Error, Result};
use regex::Regex;
use std::{
    io::{self, BufRead},
    path::PathBuf,
};

/// Send a message over webhook.
async fn dispatch_webhook(message: String, webhook: &Webhook) -> Result<()> {
    let client = reqwest::Client::new();

    let url = webhook.url.as_str();

    let format = webhook.format.clone();

    let resp = client
        .post(url)
        .header(reqwest::header::CONTENT_TYPE, format.as_content_type())
        .body(format.format_message(message));

    resp.send().await?.error_for_status()?;
    Ok(())
}

async fn dispatch_notification(message: String, notification: &Notification) -> Result<()> {
    notify_rust::Notification::new()
        .summary(notification.summary.as_str())
        .body(message.as_str())
        .show()?;
    Ok(())
}

/// Send a message to the configured destination.
pub async fn dispatch(message: String, config: &Config) -> Result<()> {
    match config {
        Config {
            webhook: Some(webhook),
            ..
        } => dispatch_webhook(message, webhook).await,
        Config {
            notification: Some(notification),
            ..
        } => dispatch_notification(message, notification).await,
        _ => Err(Error::NoDispatch),
    }
}

/// Dispatch messages by listening to stdin.
///
/// Respects the `stream.matching` config if set by excluding
/// non-matching lines read from stdin.
pub async fn stream_and_dispatch(config: &Config) -> Result<()> {
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
                    dispatch(msg.as_str().to_string(), config).await?;
                }
            }
            None => dispatch(value, config).await?,
        }
    }

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
        (false, Some(message)) => dispatch(message, &config).await,
        (false, None) => Err(Error::NoMessage),
    }
}

/// Initialise a new config file at `path`.
pub async fn init(path: &PathBuf, command: &InitCommand) -> Result<()> {
    if let Ok(true) = tokio::fs::try_exists(&path).await {
        return Err(Error::ConfigConflict(path.clone()));
    }

    let config = match command {
        InitCommand::Notification => Config::default_notification(),
        InitCommand::Webhook => Config::default_webhook(),
    };

    let data = serde_yaml::to_string(&config)?;
    Ok(tokio::fs::write(&path, &data).await?)
}
