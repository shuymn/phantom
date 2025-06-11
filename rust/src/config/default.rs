use crate::config::types::{Multiplexer, PhantomConfig, PostCreateConfig};
use crate::Result;
use std::path::Path;
use tokio::fs;

/// Generate a default configuration with common settings
pub fn default_config() -> PhantomConfig {
    PhantomConfig {
        post_create: Some(PostCreateConfig {
            copy_files: Some(vec![
                ".env".to_string(),
                ".env.local".to_string(),
                "config.local.json".to_string(),
            ]),
            commands: None,
        }),
        default_multiplexer: None,
    }
}

/// Generate a minimal empty configuration
pub fn minimal_config() -> PhantomConfig {
    PhantomConfig::default()
}

/// Generate a configuration with example values
pub fn example_config() -> PhantomConfig {
    PhantomConfig {
        post_create: Some(PostCreateConfig {
            copy_files: Some(vec![
                ".env".to_string(),
                ".env.local".to_string(),
                "config.local.json".to_string(),
                ".vscode/settings.json".to_string(),
            ]),
            commands: Some(vec![
                "npm install".to_string(),
                "npm run prepare".to_string(),
            ]),
        }),
        default_multiplexer: Some(Multiplexer::Tmux),
    }
}

/// Write a default configuration file
pub async fn write_default_config(path: &Path, format: ConfigFormat) -> Result<()> {
    let config = default_config();
    write_config(path, &config, format).await
}

/// Write an example configuration file with all options
pub async fn write_example_config(path: &Path, format: ConfigFormat) -> Result<()> {
    let config = example_config();
    write_config(path, &config, format).await
}

/// Configuration file format
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ConfigFormat {
    Json,
    Toml,
}

impl ConfigFormat {
    /// Get the file extension for this format
    pub fn extension(&self) -> &'static str {
        match self {
            ConfigFormat::Json => "json",
            ConfigFormat::Toml => "toml",
        }
    }

    /// Get the default filename for this format
    pub fn filename(&self) -> &'static str {
        match self {
            ConfigFormat::Json => crate::config::CONFIG_FILE_NAME,
            ConfigFormat::Toml => crate::config::TOML_CONFIG_FILE_NAME,
        }
    }
}

/// Write a configuration to a file
async fn write_config(path: &Path, config: &PhantomConfig, format: ConfigFormat) -> Result<()> {
    let content = match format {
        ConfigFormat::Json => serde_json::to_string_pretty(config)
            .map_err(|e| crate::config::ConfigError::Json(e))?,
        ConfigFormat::Toml => toml::to_string_pretty(config)
            .map_err(|e| crate::config::ConfigError::TomlSer(e))?,
    };

    fs::write(path, content).await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_default_config() {
        let config = default_config();
        assert!(config.post_create.is_some());
        
        let post_create = config.post_create.unwrap();
        assert!(post_create.copy_files.is_some());
        assert_eq!(post_create.copy_files.unwrap().len(), 3);
        assert!(post_create.commands.is_none());
        assert!(config.default_multiplexer.is_none());
    }

    #[test]
    fn test_minimal_config() {
        let config = minimal_config();
        assert!(config.post_create.is_none());
        assert!(config.default_multiplexer.is_none());
    }

    #[test]
    fn test_example_config() {
        let config = example_config();
        assert!(config.post_create.is_some());
        
        let post_create = config.post_create.unwrap();
        assert!(post_create.copy_files.is_some());
        assert!(post_create.commands.is_some());
        assert_eq!(post_create.commands.unwrap().len(), 2);
        assert_eq!(config.default_multiplexer, Some(Multiplexer::Tmux));
    }

    #[tokio::test]
    async fn test_write_default_config_json() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.json");
        
        write_default_config(&config_path, ConfigFormat::Json).await.unwrap();
        
        let content = fs::read_to_string(&config_path).await.unwrap();
        assert!(content.contains("\"postCreate\""));
        assert!(content.contains("\"copyFiles\""));
        assert!(content.contains("\".env\""));
    }

    #[tokio::test]
    async fn test_write_default_config_toml() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.toml");
        
        write_default_config(&config_path, ConfigFormat::Toml).await.unwrap();
        
        let content = fs::read_to_string(&config_path).await.unwrap();
        assert!(content.contains("[postCreate]"));
        assert!(content.contains("copyFiles"));
        assert!(content.contains("\".env\""));
    }

    #[tokio::test]
    async fn test_write_example_config() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.json");
        
        write_example_config(&config_path, ConfigFormat::Json).await.unwrap();
        
        let content = fs::read_to_string(&config_path).await.unwrap();
        assert!(content.contains("\"commands\""));
        assert!(content.contains("\"npm install\""));
        assert!(content.contains("\"defaultMultiplexer\""));
        assert!(content.contains("\"tmux\""));
    }

    #[test]
    fn test_config_format_extension() {
        assert_eq!(ConfigFormat::Json.extension(), "json");
        assert_eq!(ConfigFormat::Toml.extension(), "toml");
    }

    #[test]
    fn test_config_format_filename() {
        assert_eq!(ConfigFormat::Json.filename(), "phantom.config.json");
        assert_eq!(ConfigFormat::Toml.filename(), "phantom.config.toml");
    }
}