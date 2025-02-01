use crate::error::{Error, Result};
use crate::models::WebhookFormat;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Redirect {
    Stdout,
    Stderr,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Stream {
    pub enabled: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub matching: Option<String>,
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

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase", tag = "type")]
pub enum Destination {
    Webhook { url: String, format: WebhookFormat },
    Desktop { summary: String, persistent: bool },
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    pub destination: Vec<Destination>,
    #[serde(default)]
    pub stream: Stream,
}

impl Config {
    pub fn default_webhook() -> Self {
        Self {
            destination: vec![Destination::Webhook {
                url: "https://discord.com/api/webhooks/<CHANNEL_ID>/<WEBHOOK_ID>".into(),
                format: WebhookFormat::Discord,
            }],
            stream: Stream::default(),
        }
    }

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

impl std::convert::TryFrom<&PathBuf> for Config {
    type Error = Error;
    fn try_from(path: &PathBuf) -> Result<Self> {
        let content = std::fs::read_to_string(path)?;
        Ok(serde_yaml::from_str(content.as_str())?)
    }
}
