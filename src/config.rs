//! Configuration data for noti.
use crate::error::{Error, Result};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::path::PathBuf;

/// Where to write received stdin back to.
#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Redirect {
    Stdout,
    Stderr,
}

/// Notification streaming configuration.
#[derive(Debug, Deserialize, Serialize)]
pub struct Stream {
    /// Whether to use streaming or not.
    pub enabled: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    /// Optional regular expression to filter lines from stdin to send.
    pub matching: Option<String>,
    /// Where to write input received from stdin back out to.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub redirect: Option<Redirect>,
}

impl Default for Stream {
    fn default() -> Self {
        Self {
            enabled: false,
            matching: None,
            redirect: Some(Redirect::Stdout),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WebhookFormat {
    PlainText,
    Discord,
    GoogleChat,
}

impl WebhookFormat {
    /// Return the required content type for the platform.
    pub fn as_content_type(&self) -> &'static str {
        match self {
            Self::PlainText => "text/html",
            Self::Discord => "application/json",
            Self::GoogleChat => "application/json",
        }
    }

    /// Format a message as needed by the respective platform.
    pub fn format_message(&self, message: String) -> String {
        match &self {
            Self::PlainText => message,
            Self::Discord => serde_json::to_string(&json!({"content": message}))
                .expect("Serde serialize for `serde_json::json`"),
            Self::GoogleChat => serde_json::to_string(&json!({"text": message}))
                .expect("Serde serialize for `serde_json::json`"),
        }
    }
}

/// Where to send notifications to.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase", tag = "type")]
pub enum Destination {
    Webhook { url: String, format: WebhookFormat },
    Desktop { summary: String, persistent: bool },
}

/// A noti configuration file.
#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    pub destination: Vec<Destination>,
    #[serde(default)]
    pub stream: Stream,
}

impl Config {
    /// Generate an example webhook configuration for noti.
    pub fn default_webhook() -> Self {
        Self {
            destination: vec![Destination::Webhook {
                url: "https://discord.com/api/webhooks/<CHANNEL_ID>/<WEBHOOK_ID>".into(),
                format: WebhookFormat::Discord,
            }],
            stream: Stream::default(),
        }
    }

    /// Generate an example desktop configuration for noti.
    pub fn default_desktop() -> Self {
        Self {
            destination: vec![Destination::Desktop {
                summary: "Noti".into(),
                persistent: false,
            }],
            stream: Stream::default(),
        }
    }
}

/// Try to load config from a PathBuf.
impl std::convert::TryFrom<&PathBuf> for Config {
    type Error = Error;
    fn try_from(path: &PathBuf) -> Result<Self> {
        let content = std::fs::read_to_string(path)?;
        Ok(serde_yaml::from_str(content.as_str())?)
    }
}
