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
pub struct Webhook {
    pub url: String,
    pub format: WebhookFormat,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Notification {
    pub summary: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub webhook: Option<Webhook>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notification: Option<Notification>,
    #[serde(default)]
    pub stream: Stream,
}

impl Config {
    pub fn default_webhook() -> Self {
        Self {
            webhook: Some(Webhook {
                url: "https://discord.com/api/webhooks/<CHANNEL_ID>/<WEBHOOK_ID>".into(),
                format: WebhookFormat::Discord,
            }),
            notification: None,
            stream: Stream::default(),
        }
    }

    pub fn default_notification() -> Self {
        Self {
            webhook: None,
            notification: Some(Notification {
                summary: "Noti".into(),
            }),
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
