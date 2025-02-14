use crate::{
    cli::{Cli, DestinationCommand, DestinationType},
    config::{AsHeaderMap, Config, Destination, Redirect, WebhookFormat},
    error::{Error, Result},
};
use regex::Regex;
use std::{
    io::{self, BufRead},
    path::PathBuf,
};
use tokio::fs;

/// Send a message over webhook.
async fn dispatch_webhook(message: &str, url: &str, format: &WebhookFormat) -> Result<()> {
    let client = reqwest::Client::builder().build()?;

    let resp = match format {
        WebhookFormat::Custom(fmt) => client
            .request(fmt.http.method.clone().into(), url)
            .headers(fmt.http.headers.as_header_map()?)
            .body(format.format_message(message)),
        _ => client
            .post(url)
            .header(reqwest::header::CONTENT_TYPE, format.as_content_type())
            .body(format.format_message(message)),
    };

    resp.send().await?.error_for_status()?;
    Ok(())
}

/// Send a desktop notification.
fn dispatch_desktop(message: &str, summary: &str, persistent: bool) -> Result<()> {
    let mut notification = notify_rust::Notification::new()
        .summary(summary)
        .body(message)
        .appname("noti")
        .finalize();

    if persistent {
        notification.timeout(0);
    }

    notification.show()?;
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
        Destination::Webhook { url, format } => dispatch_webhook(message, url, format).await,
        Destination::Desktop {
            summary,
            persistent,
        } => dispatch_desktop(message, summary, *persistent),
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
pub async fn init(path: &PathBuf, destination: &DestinationType, custom: bool) -> Result<()> {
    if let Ok(true) = tokio::fs::try_exists(&path).await {
        return Err(Error::ConfigConflict {
            path: path.to_owned(),
        });
    }

    let config = match destination {
        DestinationType::Desktop => Config::default_desktop(),
        DestinationType::Webhook if custom => Config::default_custom_webhook(),
        DestinationType::Webhook => Config::default_webhook(),
    };

    let data = serde_yaml::to_string(&config)?;
    Ok(tokio::fs::write(&path, &data).await?)
}

/// Handle destination commands.
pub async fn destination(config: &PathBuf, command: &DestinationCommand) -> Result<()> {
    match command {
        DestinationCommand::List => list_destinations().await,
        DestinationCommand::Add {
            destination,
            custom,
        } => add_default_destination(config, &destination, *custom).await,
    }
}

/// Print available destinations.
async fn list_destinations() -> Result<()> {
    println!("desktop");
    println!("webhook");
    Ok(())
}

/// Add a default destination to existing config.
async fn add_default_destination(
    config_path: &PathBuf,
    destination: &DestinationType,
    custom: bool,
) -> Result<()> {
    let file = fs::read_to_string(&config_path).await?;
    let config: Config = serde_yaml::from_str(file.as_str())?;

    let dest = match destination {
        DestinationType::Webhook if custom => Destination::default_custom_webhook(),
        DestinationType::Webhook => Destination::default_webhook(),
        DestinationType::Desktop => Destination::default_desktop(),
    };

    let mut destination = vec![dest];
    destination.extend(config.destination);

    let new_config = Config {
        destination,
        ..config
    };

    let content = serde_yaml::to_string(&new_config)?;
    Ok(fs::write(&config_path, content).await?)
}

#[cfg(test)]
mod test {
    use super::{dispatch_webhook, Result, WebhookFormat};
    use crate::config::{CustomWebhookFormat, Http, HttpMethod, StandardWebhookFormat};
    use indexmap::IndexMap;

    const MESSAGE: &str = "noti test execution.";

    #[cfg(feature = "integration_tests")]
    #[tokio::test]
    pub async fn dispatch_webhook_discord_test() -> Result<()> {
        let url = std::env::var("NOTI_TEST_DISCORD_WEBHOOK_URL")
            .expect("NOTI_TEST_DISCORD_WEBHOOK_URL not set in environment");

        dispatch_webhook(
            MESSAGE,
            url.as_str(),
            &WebhookFormat::Standard(StandardWebhookFormat::Discord),
        )
        .await?;

        Ok(())
    }

    #[cfg(feature = "integration_tests")]
    #[tokio::test]
    pub async fn dispatch_webhook_google_chat_test() -> Result<()> {
        let url = std::env::var("NOTI_TEST_GOOGLE_CHAT_WEBHOOK_URL")
            .expect("NOTI_TEST_GOOGLE_CHAT_WEBHOOK_URL not set in environment");

        dispatch_webhook(
            MESSAGE,
            url.as_str(),
            &WebhookFormat::Standard(StandardWebhookFormat::GoogleChat),
        )
        .await?;

        Ok(())
    }

    #[cfg(feature = "integration_tests")]
    #[tokio::test]
    pub async fn dispatch_webhook_plaintext_test() -> Result<()> {
        let url = std::env::var("NOTI_TEST_PLAINTEXT_WEBHOOK_URL")
            .expect("NOTI_TEST_PLAINTEXT_WEBHOOK_URL not set in environment");

        dispatch_webhook(
            MESSAGE,
            url.as_str(),
            &WebhookFormat::Standard(StandardWebhookFormat::PlainText),
        )
        .await?;

        Ok(())
    }

    #[cfg(feature = "integration_tests")]
    #[tokio::test]
    pub async fn dispatch_webhook_custom_test() -> Result<()> {
        let url = std::env::var("NOTI_TEST_CUSTOM_WEBHOOK_URL")
            .expect("NOTI_TEST_CUSTOM_WEBHOOK_URL not set in environment");

        dispatch_webhook(
            MESSAGE,
            url.as_str(),
            &WebhookFormat::Custom(CustomWebhookFormat {
                http: Http {
                    headers: IndexMap::from([("Content-Type".into(), "application/json".into())]),
                    method: HttpMethod::Put,
                },
                template: r#"{"message":"$(message)"}"#.into(),
                escape: true,
            }),
        )
        .await?;

        Ok(())
    }
}
