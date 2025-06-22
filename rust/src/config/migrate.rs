use crate::config::default::ConfigFormat;
use crate::config::loader::load_config_from_file;
use crate::config::types::PhantomConfig;
use crate::{PhantomError, Result};
use std::path::Path;
use tokio::fs;
use tracing::{info, warn};

/// Result of a config migration
#[derive(Debug, Clone)]
pub struct MigrationResult {
    pub source_path: String,
    pub target_path: String,
    pub backup_path: Option<String>,
    pub format_from: ConfigFormat,
    pub format_to: ConfigFormat,
}

/// Migrate a configuration file from one format to another
pub async fn migrate_config(
    source_path: &Path,
    target_format: ConfigFormat,
    create_backup: bool,
) -> Result<MigrationResult> {
    info!("Migrating config from {} to {:?} format", source_path.display(), target_format);

    // Determine source format from file extension
    let source_format = if source_path.extension().and_then(|s| s.to_str()) == Some("json") {
        ConfigFormat::Json
    } else if source_path.extension().and_then(|s| s.to_str()) == Some("toml") {
        ConfigFormat::Toml
    } else {
        return Err(PhantomError::ConfigInvalid {
            reason: "Cannot determine source config format from file extension".to_string(),
        });
    };

    if source_format == target_format {
        return Err(PhantomError::ConfigInvalid {
            reason: "Source and target formats are the same".to_string(),
        });
    }

    // Load the config
    let config = load_config_from_file(source_path).await?;

    // Create backup if requested
    let backup_path = if create_backup {
        let backup_path = source_path.with_extension(format!(
            "{}.backup",
            source_path.extension().and_then(|s| s.to_str()).unwrap_or("config")
        ));

        info!("Creating backup at {}", backup_path.display());
        fs::copy(source_path, &backup_path).await.map_err(|e| {
            PhantomError::FileOperationFailed {
                operation: "create backup".to_string(),
                path: backup_path.clone(),
                reason: e.to_string(),
            }
        })?;

        Some(backup_path.to_string_lossy().to_string())
    } else {
        None
    };

    // Determine target path
    let target_path = source_path.with_extension(match target_format {
        ConfigFormat::Json => "json",
        ConfigFormat::Toml => "toml",
    });

    // Save in new format
    save_config(&config, &target_path, target_format).await?;

    info!("Successfully migrated config from {:?} to {:?}", source_format, target_format);

    Ok(MigrationResult {
        source_path: source_path.to_string_lossy().to_string(),
        target_path: target_path.to_string_lossy().to_string(),
        backup_path,
        format_from: source_format,
        format_to: target_format,
    })
}

/// Migrate JSON config to TOML format
pub async fn migrate_json_to_toml(
    json_path: &Path,
    create_backup: bool,
) -> Result<MigrationResult> {
    migrate_config(json_path, ConfigFormat::Toml, create_backup).await
}

/// Migrate TOML config to JSON format
pub async fn migrate_toml_to_json(
    toml_path: &Path,
    create_backup: bool,
) -> Result<MigrationResult> {
    migrate_config(toml_path, ConfigFormat::Json, create_backup).await
}

/// Save config in the specified format
async fn save_config(config: &PhantomConfig, path: &Path, format: ConfigFormat) -> Result<()> {
    let content = match format {
        ConfigFormat::Json => {
            serde_json::to_string_pretty(config).map_err(|e| PhantomError::ConfigInvalid {
                reason: format!("Failed to serialize config to JSON: {}", e),
            })?
        }
        ConfigFormat::Toml => {
            toml::to_string_pretty(config).map_err(|e| PhantomError::ConfigInvalid {
                reason: format!("Failed to serialize config to TOML: {}", e),
            })?
        }
    };

    fs::write(path, content).await.map_err(|e| PhantomError::FileOperationFailed {
        operation: "write".to_string(),
        path: path.to_path_buf(),
        reason: e.to_string(),
    })?;

    Ok(())
}

/// Detect and migrate legacy config files
pub async fn auto_migrate_config(config_dir: &Path) -> Result<Option<MigrationResult>> {
    let json_path = config_dir.join("phantom.config.json");
    let toml_path = config_dir.join("phantom.config.toml");

    // Check if both exist
    let json_exists = json_path.exists();
    let toml_exists = toml_path.exists();

    if json_exists && !toml_exists {
        // Migrate JSON to TOML
        info!("Found legacy JSON config, migrating to TOML format");
        let result = migrate_json_to_toml(&json_path, true).await?;

        // Optionally remove the old JSON file after successful migration
        // For now, we keep it as backup
        warn!(
            "Migration complete. Old JSON config kept at {}. You can delete it if no longer needed.",
            json_path.display()
        );

        Ok(Some(result))
    } else if json_exists && toml_exists {
        // Both exist, prefer TOML
        warn!("Both JSON and TOML configs exist. Using TOML config at {}", toml_path.display());
        Ok(None)
    } else {
        // No migration needed
        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::types::{Multiplexer, PostCreateConfig};
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_migrate_json_to_toml() {
        let temp_dir = TempDir::new().unwrap();
        let json_path = temp_dir.path().join("phantom.config.json");

        // Create a test JSON config
        let config = PhantomConfig {
            post_create: Some(PostCreateConfig {
                copy_files: Some(vec![".env".to_string(), "config.toml".to_string()]),
                commands: Some(vec!["npm install".to_string()]),
            }),
            default_multiplexer: Some(Multiplexer::Tmux),
        };

        let json_content = serde_json::to_string_pretty(&config).unwrap();
        fs::write(&json_path, json_content).await.unwrap();

        // Migrate to TOML
        let result = migrate_json_to_toml(&json_path, true).await.unwrap();

        // Verify migration
        assert_eq!(result.format_from, ConfigFormat::Json);
        assert_eq!(result.format_to, ConfigFormat::Toml);
        assert!(result.backup_path.is_some());

        // Check that TOML file was created
        let toml_path = temp_dir.path().join("phantom.config.toml");
        assert!(toml_path.exists());

        // Load and verify the migrated config
        let migrated_config = load_config_from_file(&toml_path).await.unwrap();
        assert_eq!(migrated_config.post_create, config.post_create);
        assert_eq!(migrated_config.default_multiplexer, config.default_multiplexer);

        // Check backup was created
        let backup_path = temp_dir.path().join("phantom.config.json.backup");
        assert!(backup_path.exists());
    }

    #[tokio::test]
    async fn test_migrate_toml_to_json() {
        let temp_dir = TempDir::new().unwrap();
        let toml_path = temp_dir.path().join("phantom.config.toml");

        // Create a test TOML config
        let config = PhantomConfig {
            post_create: Some(PostCreateConfig {
                copy_files: Some(vec!["Gemfile".to_string(), "Gemfile.lock".to_string()]),
                commands: Some(vec!["bundle install".to_string()]),
            }),
            default_multiplexer: Some(Multiplexer::Kitty),
        };

        let toml_content = toml::to_string_pretty(&config).unwrap();
        fs::write(&toml_path, toml_content).await.unwrap();

        // Migrate to JSON
        let result = migrate_toml_to_json(&toml_path, false).await.unwrap();

        // Verify migration
        assert_eq!(result.format_from, ConfigFormat::Toml);
        assert_eq!(result.format_to, ConfigFormat::Json);
        assert!(result.backup_path.is_none()); // No backup requested

        // Check that JSON file was created
        let json_path = temp_dir.path().join("phantom.config.json");
        assert!(json_path.exists());

        // Load and verify the migrated config
        let migrated_config = load_config_from_file(&json_path).await.unwrap();
        assert_eq!(migrated_config.post_create, config.post_create);
        assert_eq!(migrated_config.default_multiplexer, config.default_multiplexer);
    }

    #[tokio::test]
    async fn test_migrate_same_format_error() {
        let temp_dir = TempDir::new().unwrap();
        let json_path = temp_dir.path().join("phantom.config.json");

        // Create a dummy file
        fs::write(&json_path, "{}").await.unwrap();

        // Try to migrate JSON to JSON
        let result = migrate_config(&json_path, ConfigFormat::Json, false).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("same"));
    }

    #[tokio::test]
    async fn test_auto_migrate_json_to_toml() {
        let temp_dir = TempDir::new().unwrap();
        let json_path = temp_dir.path().join("phantom.config.json");

        // Create a test JSON config
        let config = PhantomConfig::default();
        let json_content = serde_json::to_string_pretty(&config).unwrap();
        fs::write(&json_path, json_content).await.unwrap();

        // Run auto migration
        let result = auto_migrate_config(temp_dir.path()).await.unwrap();
        assert!(result.is_some());

        let migration = result.unwrap();
        assert_eq!(migration.format_from, ConfigFormat::Json);
        assert_eq!(migration.format_to, ConfigFormat::Toml);

        // Check that TOML file was created
        let toml_path = temp_dir.path().join("phantom.config.toml");
        assert!(toml_path.exists());
    }

    #[tokio::test]
    async fn test_auto_migrate_both_exist() {
        let temp_dir = TempDir::new().unwrap();
        let json_path = temp_dir.path().join("phantom.config.json");
        let toml_path = temp_dir.path().join("phantom.config.toml");

        // Create both files
        fs::write(&json_path, "{}").await.unwrap();
        fs::write(&toml_path, "").await.unwrap();

        // Run auto migration
        let result = auto_migrate_config(temp_dir.path()).await.unwrap();
        assert!(result.is_none()); // No migration performed
    }

    #[tokio::test]
    async fn test_auto_migrate_none_exist() {
        let temp_dir = TempDir::new().unwrap();

        // Run auto migration
        let result = auto_migrate_config(temp_dir.path()).await.unwrap();
        assert!(result.is_none()); // No migration needed
    }
}
