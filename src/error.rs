use derive_more::{Error, From};
use std::path::PathBuf;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, From, Error)]
pub enum Error {
    NoConfig,
    NoMessage,
    StreamAndMessage,
    Io(std::io::Error),
    ConfigConflict {
        path: PathBuf,
    },

    #[from]
    InvalidConfig(serde_yaml::Error),

    Http(reqwest::Error),

    #[from]
    UnknownHttpHeader(reqwest::header::InvalidHeaderName),

    #[from]
    InvalidHttpHeader(reqwest::header::InvalidHeaderValue),

    #[from]
    Regex(regex::Error),

    #[from]
    NotifyRust(notify_rust::error::Error),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let message = match self {
            Self::NoConfig => {
                "No config file found, please create one or provide the path with --config".into()
            }
            Self::ConfigConflict { path } => {
                format!("Config file `{}` already exists", path.to_string_lossy())
            }
            Self::InvalidConfig(e) => format!("Invalid config file: {e}"),
            Self::Http(e) => format!("An error occurred when sending a request: {e}"),
            Self::UnknownHttpHeader(e) => format!("{e}"),
            Self::InvalidHttpHeader(e) => format!("{e}"),
            Self::Regex(e) => format!("Failed to parse regex: {e}"),
            Self::Io(e) => format!("IO: {e}"),
            Error::NoMessage => {
                "A message must be provided when not streaming notifications".into()
            }
            Error::StreamAndMessage => "A message cannot be provided when using streaming".into(),
            Error::NotifyRust(e) => format!("Failed to send desktop notification: {e}"),
        };

        write!(f, "{message}")
    }
}

impl std::convert::From<std::io::Error> for Error {
    fn from(error: std::io::Error) -> Self {
        match error.kind() {
            std::io::ErrorKind::NotFound => Error::NoConfig,
            _ => Error::Io(error),
        }
    }
}

impl std::convert::From<reqwest::Error> for Error {
    fn from(error: reqwest::Error) -> Self {
        let error = error.without_url();
        Self::Http(error)
    }
}
