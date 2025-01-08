use clap::Parser;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::path::PathBuf;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    NoConfig,
    InvalidConfig(serde_yaml::Error),
    MissingConfig(&'static str),
    Http(reqwest::Error),
    Regex(regex::Error),
    Io(std::io::Error),
    NoMessage,
}

impl std::convert::From<std::io::Error> for Error {
    fn from(error: std::io::Error) -> Self {
        match error.kind() {
            std::io::ErrorKind::NotFound => Error::NoConfig,
            _ => Error::Io(error),
        }
    }
}

impl std::convert::From<serde_yaml::Error> for Error {
    fn from(error: serde_yaml::Error) -> Self {
        Self::InvalidConfig(error)
    }
}

impl std::convert::From<reqwest::Error> for Error {
    fn from(error: reqwest::Error) -> Self {
        Self::Http(error)
    }
}

impl std::convert::From<regex::Error> for Error {
    fn from(error: regex::Error) -> Self {
        Self::Regex(error)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Method {
    Webhook,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WebhookFormat {
    PlainText,
    Discord,
    GoogleChat,
}

impl WebhookFormat {
    pub fn as_content_type(&self) -> &'static str {
        match self {
            Self::PlainText => "text/html",
            Self::Discord => "application/json",
            Self::GoogleChat => "application/json",
        }
    }

    pub fn format_message(&self, message: String) -> String {
        match &self {
            Self::PlainText => message,
            Self::Discord => serde_json::to_string(&json!({"content": message}))
                .expect("Serde serialize derive is broken"),
            Self::GoogleChat => serde_json::to_string(&json!({"text": message}))
                .expect("Serde serialize derive is broken"),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct Config {
    pub method: Method,
    pub webhook_url: Option<String>,
    pub webhook_format: Option<WebhookFormat>,
    pub matching: Option<String>,
    pub stream: Option<bool>,
}

impl std::convert::TryFrom<&PathBuf> for Config {
    type Error = Error;
    fn try_from(path: &PathBuf) -> Result<Self> {
        let content = std::fs::read_to_string(&path)?;
        Ok(serde_yaml::from_str(content.as_str())?)
    }
}

#[derive(Debug, Parser)]
pub struct Cli {
    #[arg()]
    pub message: Option<String>,
    #[arg(long, default_value = "noti.yaml")]
    pub config: PathBuf,
}

async fn dispatch_webhook(message: String, config: &Config) -> Result<()> {
    let client = reqwest::Client::new();

    let url = config
        .webhook_url
        .as_ref()
        .ok_or(Error::MissingConfig("webhook_url"))?;

    let webhook = config
        .webhook_format
        .as_ref()
        .ok_or(Error::MissingConfig("webhook_format"))?;

    let resp = client
        .post(url)
        .header(reqwest::header::CONTENT_TYPE, webhook.as_content_type())
        .body(webhook.format_message(message));

    resp.send().await?.error_for_status()?;
    Ok(())
}

pub async fn dispatch(message: String, config: &Config) -> Result<()> {
    match config.method {
        Method::Webhook => dispatch_webhook(message, config).await,
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    #[cfg(not(unix))]
    {
        println!("Only unix systems are supported");
        std::process::exit(1);
    }

    let args = Cli::parse();
    let config = Config::try_from(&args.config)?;

    match config.stream {
        Some(stream) if stream => unimplemented!("Streaming is not supported yet"),
        _ => match args.message {
            Some(message) => dispatch(message, &config).await,
            None => {
                println!("A message must be provided when not streaming notifications");
                Err(Error::NoMessage)
            }
        },
    }
}
