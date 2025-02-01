use serde::{Deserialize, Serialize};
use serde_json::json;

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
                .expect("Serde serialize for `serde_json::json`"),
            Self::GoogleChat => serde_json::to_string(&json!({"text": message}))
                .expect("Serde serialize for `serde_json::json`"),
        }
    }
}
