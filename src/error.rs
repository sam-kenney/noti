use std::path::PathBuf;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    NoConfig,
    ConfigConflict(PathBuf),
    InvalidConfig(serde_yaml::Error),
    Http(reqwest::Error),
    Regex(regex::Error),
    Io(std::io::Error),
    NoMessage,
    StreamAndMessage,
    NotifyRust(notify_rust::error::Error),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let message = match self {
            Self::NoConfig => {
                "No config file found, please create one or provide the path with --config".into()
            }
            Self::ConfigConflict(path) => format!("`{}` already exists", path.to_string_lossy()),
            Self::InvalidConfig(e) => e.to_string(),
            Self::Http(e) => e.to_string(),
            Self::Regex(e) => e.to_string(),
            Self::Io(e) => e.to_string(),
            Error::NoMessage => {
                "A message must be provided when not streaming notifications".into()
            }
            Error::StreamAndMessage => "A message cannot be provided when using streaming".into(),
            Error::NotifyRust(e) => e.to_string(),
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

impl std::convert::From<notify_rust::error::Error> for Error {
    fn from(error: notify_rust::error::Error) -> Self {
        Self::NotifyRust(error)
    }
}
