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
            ConfigError::Json(err) => PhantomError::Config(format!("JSON error: {}", err)),
            ConfigError::TomlDe(err) => {
                PhantomError::Config(format!("TOML deserialization error: {}", err))
            }
            ConfigError::TomlSer(err) => {
                PhantomError::Config(format!("TOML serialization error: {}", err))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_error_display() {
        let error = ConfigError::NotFound("/path/to/config".to_string());
        assert_eq!(error.to_string(), "Configuration file not found: /path/to/config");

        let error = ConfigError::ParseError("invalid syntax".to_string());
        assert_eq!(error.to_string(), "Failed to parse configuration: invalid syntax");

        let error = ConfigError::ValidationError("missing field".to_string());
        assert_eq!(error.to_string(), "Invalid configuration: missing field");
    }

    #[test]
    fn test_config_error_from_io() {
        let io_error = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let config_error = ConfigError::from(io_error);
        assert!(matches!(config_error, ConfigError::Io(_)));
    }

    #[test]
    fn test_config_error_from_json() {
        let json_err = serde_json::from_str::<String>("invalid").unwrap_err();
        let config_error = ConfigError::from(json_err);
        assert!(matches!(config_error, ConfigError::Json(_)));
    }

    #[test]
    fn test_config_error_from_toml_de() {
        let toml_err = toml::from_str::<String>("invalid = ").unwrap_err();
        let config_error = ConfigError::from(toml_err);
        assert!(matches!(config_error, ConfigError::TomlDe(_)));
    }

    #[test]
    fn test_config_error_from_toml_ser() {
        use serde::Serialize;
        #[derive(Serialize)]
        struct Invalid {
            #[serde(serialize_with = "invalid_serializer")]
            field: String,
        }

        fn invalid_serializer<S>(_: &String, _: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer,
        {
            Err(serde::ser::Error::custom("test error"))
        }

        let invalid = Invalid { field: "test".to_string() };
        let toml_err = toml::to_string(&invalid).unwrap_err();
        let config_error = ConfigError::from(toml_err);
        assert!(matches!(config_error, ConfigError::TomlSer(_)));
    }

    #[test]
    fn test_config_error_to_phantom_error() {
        // Test NotFound conversion
        let error = ConfigError::NotFound("config.toml".to_string());
        let phantom_error: PhantomError = error.into();
        match phantom_error {
            PhantomError::Config(msg) => assert!(msg.contains("Configuration file not found")),
            _ => panic!("Expected PhantomError::Config"),
        }

        // Test ParseError conversion
        let error = ConfigError::ParseError("syntax error".to_string());
        let phantom_error: PhantomError = error.into();
        match phantom_error {
            PhantomError::Config(msg) => assert!(msg.contains("Failed to parse configuration")),
            _ => panic!("Expected PhantomError::Config"),
        }

        // Test ValidationError conversion
        let error = ConfigError::ValidationError("invalid value".to_string());
        let phantom_error: PhantomError = error.into();
        match phantom_error {
            PhantomError::Validation(msg) => assert!(msg.contains("Invalid configuration")),
            _ => panic!("Expected PhantomError::Validation"),
        }

        // Test Io conversion
        let io_error = std::io::Error::new(std::io::ErrorKind::PermissionDenied, "access denied");
        let error = ConfigError::Io(io_error);
        let phantom_error: PhantomError = error.into();
        assert!(matches!(phantom_error, PhantomError::Io(_)));

        // Test Json conversion
        let json_err = serde_json::from_str::<String>("bad json").unwrap_err();
        let error = ConfigError::Json(json_err);
        let phantom_error: PhantomError = error.into();
        match phantom_error {
            PhantomError::Config(msg) => assert!(msg.contains("JSON error")),
            _ => panic!("Expected PhantomError::Config"),
        }

        // Test TomlDe conversion
        let toml_err = toml::from_str::<String>("bad = ").unwrap_err();
        let error = ConfigError::TomlDe(toml_err);
        let phantom_error: PhantomError = error.into();
        match phantom_error {
            PhantomError::Config(msg) => assert!(msg.contains("TOML deserialization error")),
            _ => panic!("Expected PhantomError::Config"),
        }
    }

    #[test]
    fn test_config_error_debug() {
        let error = ConfigError::NotFound("test.toml".to_string());
        let debug_str = format!("{:?}", error);
        assert!(debug_str.contains("NotFound"));
        assert!(debug_str.contains("test.toml"));
    }
}
