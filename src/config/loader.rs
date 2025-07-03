use crate::config::errors::ConfigError;
use crate::config::types::PhantomConfig;
use crate::config::validate::validate_config;
use crate::Result;
use std::path::{Path, PathBuf};
use tokio::fs;
use tracing::{debug, info};

/// Default configuration file name
pub const CONFIG_FILE_NAME: &str = "phantom.config.json";

/// Alternative TOML configuration file name
pub const TOML_CONFIG_FILE_NAME: &str = "phantom.config.toml";

/// Load configuration from a git repository root
pub async fn load_config(git_root: &Path) -> Result<Option<PhantomConfig>> {
    // Try JSON first (for backward compatibility)
    let json_path = git_root.join(CONFIG_FILE_NAME);
    if let Ok(config) = load_json_config(&json_path).await {
        return Ok(Some(config));
    }

    // Try TOML as alternative
    let toml_path = git_root.join(TOML_CONFIG_FILE_NAME);
    if let Ok(config) = load_toml_config(&toml_path).await {
        return Ok(Some(config));
    }

    // No configuration found
    debug!("No configuration file found in {}", git_root.display());
    Ok(None)
}

/// Load configuration from a specific file
pub async fn load_config_from_file(path: &Path) -> Result<PhantomConfig> {
    match path.extension().and_then(|s| s.to_str()) {
        Some("json") => load_json_config(path).await,
        Some("toml") => load_toml_config(path).await,
        _ => Err(ConfigError::ValidationError(
            "Configuration file must have .json or .toml extension".to_string(),
        )
        .into()),
    }
}

/// Generic configuration loader
async fn load_config_generic<F>(path: &Path, format_name: &str, parser: F) -> Result<PhantomConfig>
where
    F: Fn(&str) -> std::result::Result<PhantomConfig, String>,
{
    debug!("Loading {} configuration from {}", format_name, path.display());

    let content = fs::read_to_string(path).await.map_err(|e| {
        if e.kind() == std::io::ErrorKind::NotFound {
            ConfigError::NotFound(path.display().to_string())
        } else {
            ConfigError::Io(e)
        }
    })?;

    let config = parser(&content)
        .map_err(|e| ConfigError::ParseError(format!("{format_name} error: {e}")))?;

    validate_config(&config)?;

    info!("Loaded configuration from {}", path.display());
    Ok(config)
}

/// Load JSON configuration
async fn load_json_config(path: &Path) -> Result<PhantomConfig> {
    load_config_generic(path, "JSON", |content| {
        serde_json::from_str(content).map_err(|e| e.to_string())
    })
    .await
}

/// Load TOML configuration
async fn load_toml_config(path: &Path) -> Result<PhantomConfig> {
    load_config_generic(path, "TOML", |content| toml::from_str(content).map_err(|e| e.to_string()))
        .await
}

/// Find configuration file in directory hierarchy
pub async fn find_config_file(start_dir: &Path) -> Option<PathBuf> {
    let mut current = start_dir;

    loop {
        // Check for JSON config
        let json_path = current.join(CONFIG_FILE_NAME);
        if fs::metadata(&json_path).await.is_ok() {
            return Some(json_path);
        }

        // Check for TOML config
        let toml_path = current.join(TOML_CONFIG_FILE_NAME);
        if fs::metadata(&toml_path).await.is_ok() {
            return Some(toml_path);
        }

        // Move up to parent directory
        match current.parent() {
            Some(parent) => current = parent,
            None => break,
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::types::Multiplexer;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_load_json_config() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("phantom.config.json");

        let json = r#"{
            "postCreate": {
                "copyFiles": [".env", "config.json"],
                "commands": ["npm install"]
            },
            "defaultMultiplexer": "tmux"
        }"#;

        fs::write(&config_path, json).await.unwrap();

        let config = load_json_config(&config_path).await.unwrap();
        assert!(config.post_create.is_some());

        let post_create = config.post_create.unwrap();
        assert_eq!(post_create.copy_files.unwrap().len(), 2);
        assert_eq!(post_create.commands.unwrap()[0], "npm install");
        assert_eq!(config.default_multiplexer, Some(Multiplexer::Tmux));
    }

    #[tokio::test]
    async fn test_load_toml_config() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("phantom.config.toml");

        let toml = r#"
defaultMultiplexer = "kitty"

[postCreate]
copyFiles = [".env", "config.toml"]
commands = ["cargo build"]
"#;

        fs::write(&config_path, toml).await.unwrap();

        let config = load_toml_config(&config_path).await.unwrap();
        assert!(config.post_create.is_some());

        let post_create = config.post_create.unwrap();
        assert_eq!(post_create.copy_files.unwrap().len(), 2);
        assert_eq!(post_create.commands.unwrap()[0], "cargo build");
        assert_eq!(config.default_multiplexer, Some(Multiplexer::Kitty));
    }

    #[tokio::test]
    async fn test_load_config_json_priority() {
        let temp_dir = TempDir::new().unwrap();

        // Create both JSON and TOML configs
        let json_path = temp_dir.path().join("phantom.config.json");
        let toml_path = temp_dir.path().join("phantom.config.toml");

        fs::write(&json_path, r#"{"defaultMultiplexer": "tmux"}"#).await.unwrap();
        fs::write(&toml_path, r#"defaultMultiplexer = "kitty""#).await.unwrap();

        // JSON should take priority
        let config = load_config(temp_dir.path()).await.unwrap().unwrap();
        assert_eq!(config.default_multiplexer, Some(Multiplexer::Tmux));
    }

    #[tokio::test]
    async fn test_load_config_not_found() {
        let temp_dir = TempDir::new().unwrap();
        let result = load_config(temp_dir.path()).await.unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_load_invalid_json() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("phantom.config.json");

        fs::write(&config_path, "{ invalid json").await.unwrap();

        let result = load_json_config(&config_path).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("JSON error"));
    }

    #[tokio::test]
    async fn test_load_invalid_toml() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("phantom.config.toml");

        fs::write(&config_path, "invalid = toml =").await.unwrap();

        let result = load_toml_config(&config_path).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("TOML error"));
    }

    #[tokio::test]
    async fn test_load_config_with_validation_error() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("phantom.config.json");

        let json = r#"{
            "postCreate": {
                "copyFiles": ["/etc/passwd"]
            }
        }"#;

        fs::write(&config_path, json).await.unwrap();

        let result = load_json_config(&config_path).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("absolute paths"));
    }

    #[tokio::test]
    async fn test_find_config_file() {
        let temp_dir = TempDir::new().unwrap();
        let sub_dir = temp_dir.path().join("sub").join("dir");
        fs::create_dir_all(&sub_dir).await.unwrap();

        let config_path = temp_dir.path().join("phantom.config.json");
        fs::write(&config_path, "{}").await.unwrap();

        // Should find config in parent directory
        let found = find_config_file(&sub_dir).await;
        assert!(found.is_some());
        assert_eq!(found.unwrap(), config_path);
    }

    #[tokio::test]
    async fn test_find_config_file_not_found() {
        let temp_dir = TempDir::new().unwrap();
        let found = find_config_file(temp_dir.path()).await;
        assert!(found.is_none());
    }

    #[tokio::test]
    async fn test_load_config_from_file_by_extension() {
        let temp_dir = TempDir::new().unwrap();

        // Test JSON
        let json_path = temp_dir.path().join("config.json");
        fs::write(&json_path, r#"{"defaultMultiplexer": "tmux"}"#).await.unwrap();
        let config = load_config_from_file(&json_path).await.unwrap();
        assert_eq!(config.default_multiplexer, Some(Multiplexer::Tmux));

        // Test TOML
        let toml_path = temp_dir.path().join("config.toml");
        fs::write(&toml_path, r#"defaultMultiplexer = "kitty""#).await.unwrap();
        let config = load_config_from_file(&toml_path).await.unwrap();
        assert_eq!(config.default_multiplexer, Some(Multiplexer::Kitty));

        // Test unsupported extension
        let txt_path = temp_dir.path().join("config.txt");
        fs::write(&txt_path, "some content").await.unwrap();
        let result = load_config_from_file(&txt_path).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains(".json or .toml"));
    }
}
