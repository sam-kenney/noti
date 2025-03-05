use derive_more::From;
use std::path::PathBuf;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, From)]
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

    VarNotSet(String),
    InvalidVar(String),
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(e) => Some(e),
            Self::InvalidConfig(e) => Some(e),
            Self::Http(e) => Some(e),
            Self::UnknownHttpHeader(e) => Some(e),
            Self::InvalidHttpHeader(e) => Some(e),
            Self::Regex(e) => Some(e),
            Self::NotifyRust(e) => Some(e),
            _ => None,
        }
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let message = match self {
            Self::NoConfig => "no config file found".into(),
            Self::ConfigConflict { path } => {
                format!("config file `{}` already exists", path.display())
            }

            Self::InvalidConfig(e) => format!("invalid config file: {e}"),
            Self::Http(e) => format!("an error occurred when sending a request: {e}"),
            Self::UnknownHttpHeader(e) => format!("{e}"),
            Self::InvalidHttpHeader(e) => format!("{e}"),
            Self::Regex(e) => format!("failed to parse regex: {e}"),
            Self::Io(e) => format!("io: {e}"),
            Error::NoMessage => {
                "a message must be provided when not streaming notifications".into()
            }
            Error::StreamAndMessage => "a message cannot be provided when using streaming".into(),
            Error::NotifyRust(e) => format!("failed to send desktop notification: {e}"),
            Error::VarNotSet(var) => {
                format!("env var {var} was specified in noti.yaml but has not been set")
            }
            Error::InvalidVar(var) => {
                format!("env var {var} is not UTF-8 and cannot be parsed")
            }
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
