//! Configuration data for noti.
use crate::error::{Error, Result};
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::path::PathBuf;

pub trait AsHeaderMap {
    fn as_header_map(&self) -> Result<reqwest::header::HeaderMap>;
}

/// HeaderMap cannot be serialized, and HeaderMap doesn't implement
/// From<IndexMap>, so convenience method to convert.
impl AsHeaderMap for IndexMap<String, String> {
    fn as_header_map(&self) -> Result<reqwest::header::HeaderMap> {
        use reqwest::header::{HeaderMap, HeaderName, HeaderValue};

        self.iter()
            .map(|(k, v)| {
                let name = HeaderName::from_bytes(k.as_bytes())?;
                let value = HeaderValue::from_bytes(v.as_bytes())?;
                Ok((name, value))
            })
            .collect::<Result<HeaderMap>>()
    }
}

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
    /// Optional regular expression to filter lines from stdin to send.
    #[serde(skip_serializing_if = "Option::is_none")]
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

/// Builtin supported Webhook Formats for common webhook providers.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StandardWebhookFormat {
    /// Send a webhook message to an endpoint that supports plain text requests.
    PlainText,
    /// Send a webhook message to a Discord channel.
    Discord,
    /// Send a webhook message to a Google Chat.
    GoogleChat,
}

/// Subset of http methods useable with webhooks.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HttpMethod {
    POST,
    PATCH,
    PUT,
}

impl std::convert::From<HttpMethod> for reqwest::Method {
    fn from(value: HttpMethod) -> reqwest::Method {
        use reqwest::Method;
        match value {
            HttpMethod::POST => Method::POST,
            HttpMethod::PATCH => Method::PATCH,
            HttpMethod::PUT => Method::PUT,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Http {
    pub headers: IndexMap<String, String>,
    pub method: HttpMethod,
}

/// Enables configuring sending notifications to other webhook providers.
#[derive(Debug, Serialize, Deserialize)]
pub struct CustomWebhookFormat {
    pub http: Http,
    pub template: String,
    pub escape: bool,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum WebhookFormat {
    Standard(StandardWebhookFormat),
    Custom(CustomWebhookFormat),
}

impl WebhookFormat {
    /// Return the required content type for the platform.
    pub fn as_content_type(&self) -> String {
        match self {
            Self::Standard(format) => match format {
                StandardWebhookFormat::PlainText => "text/plain",
                StandardWebhookFormat::Discord => "application/json",
                StandardWebhookFormat::GoogleChat => "application/json",
            }
            .into(),
            Self::Custom(format) => format
                .http
                .headers
                .get(&"Content-Type".to_string())
                .unwrap_or(&"text/plain".to_string())
                .to_owned(),
        }
    }

    /// Format a message as needed by the respective platform.
    pub fn format_message(&self, message: &str) -> String {
        match &self {
            Self::Standard(format) => match format {
                StandardWebhookFormat::PlainText => message.into(),
                StandardWebhookFormat::Discord => {
                    serde_json::to_string(&json!({"content": message}))
                        .expect("Serde serialize for `serde_json::json`")
                }
                StandardWebhookFormat::GoogleChat => {
                    serde_json::to_string(&json!({"text": message}))
                        .expect("Serde serialize for `serde_json::json`")
                }
            },
            Self::Custom(format) => {
                let message = match format.escape {
                    false => message.into(),
                    true => message.escape_default().collect::<String>(),
                };
                format.template.replace("$(message)", message.as_str())
            }
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

impl Destination {
    pub fn default_webhook() -> Self {
        Destination::Webhook {
            url: "https://discord.com/api/webhooks/<CHANNEL_ID>/<WEBHOOK_ID>".into(),
            format: WebhookFormat::Standard(StandardWebhookFormat::Discord),
        }
    }

    pub fn default_custom_webhook() -> Self {
        Destination::Webhook {
            url: "https://discord.com/api/webhooks/<CHANNEL_ID>/<WEBHOOK_ID>".into(),
            format: WebhookFormat::Custom(CustomWebhookFormat {
                http: Http {
                    headers: IndexMap::from([(
                        "Content-Type".to_string(),
                        "application/json".to_string(),
                    )]),
                    method: HttpMethod::POST,
                },
                escape: true,
                template: r#"{"content": "$(message)"}"#.into(),
            }),
        }
    }

    pub fn default_desktop() -> Self {
        Destination::Desktop {
            summary: "Noti".into(),
            persistent: false,
        }
    }
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
            destination: vec![Destination::default_webhook()],
            stream: Stream::default(),
        }
    }

    /// Generate an example custom webhook configuration for noti
    pub fn default_custom_webhook() -> Self {
        Self {
            destination: vec![Destination::default_custom_webhook()],
            stream: Stream::default(),
        }
    }

    /// Generate an example desktop configuration for noti.
    pub fn default_desktop() -> Self {
        Self {
            destination: vec![Destination::default_desktop()],
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
