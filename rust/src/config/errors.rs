use crate::PhantomError;
use thiserror::Error;

/// Configuration-related errors
#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("Configuration file not found: {0}")]
    NotFound(String),

    #[error("Failed to parse configuration: {0}")]
    ParseError(String),

    #[error("Invalid configuration: {0}")]
    ValidationError(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("TOML deserialization error: {0}")]
    TomlDe(#[from] toml::de::Error),

    #[error("TOML serialization error: {0}")]
    TomlSer(#[from] toml::ser::Error),
}

impl From<ConfigError> for PhantomError {
    fn from(err: ConfigError) -> Self {
        match err {
            ConfigError::NotFound(path) => {
                PhantomError::Config(format!("Configuration file not found: {}", path))
            }
            ConfigError::ParseError(msg) => {
                PhantomError::Config(format!("Failed to parse configuration: {}", msg))
            }
            ConfigError::ValidationError(msg) => {
                PhantomError::Validation(format!("Invalid configuration: {}", msg))
            }
            ConfigError::Io(err) => PhantomError::Io(err),
            ConfigError::Json(err) => {
                PhantomError::Config(format!("JSON error: {}", err))
            }
            ConfigError::TomlDe(err) => {
                PhantomError::Config(format!("TOML deserialization error: {}", err))
            }
            ConfigError::TomlSer(err) => {
                PhantomError::Config(format!("TOML serialization error: {}", err))
            }
        }
    }
}