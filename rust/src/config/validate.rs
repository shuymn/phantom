use crate::config::errors::ConfigError;
use crate::config::types::{Multiplexer, PhantomConfig, PostCreateConfig};
use crate::Result;

/// Validate a PhantomConfig instance
pub fn validate_config(config: &PhantomConfig) -> Result<()> {
    // Validate post_create if present
    if let Some(ref post_create) = config.post_create {
        validate_post_create(post_create)?;
    }

    // Validate default_multiplexer if present
    if let Some(ref multiplexer) = config.default_multiplexer {
        validate_multiplexer(multiplexer)?;
    }

    Ok(())
}

/// Validate post-create configuration
fn validate_post_create(post_create: &PostCreateConfig) -> Result<()> {
    // Validate copy_files
    if let Some(ref copy_files) = post_create.copy_files {
        for file in copy_files {
            if file.trim().is_empty() {
                return Err(ConfigError::ValidationError(
                    "postCreate.copyFiles cannot contain empty strings".to_string(),
                )
                .into());
            }

            // Disallow absolute paths for security
            if file.starts_with('/') || file.starts_with('\\') {
                return Err(ConfigError::ValidationError(format!(
                    "postCreate.copyFiles cannot contain absolute paths: {}",
                    file
                ))
                .into());
            }

            // Disallow parent directory references
            if file.contains("..") {
                return Err(ConfigError::ValidationError(format!(
                    "postCreate.copyFiles cannot contain parent directory references: {}",
                    file
                ))
                .into());
            }
        }
    }

    // Validate commands
    if let Some(ref commands) = post_create.commands {
        for command in commands {
            if command.trim().is_empty() {
                return Err(ConfigError::ValidationError(
                    "postCreate.commands cannot contain empty strings".to_string(),
                )
                .into());
            }
        }
    }

    Ok(())
}

/// Validate multiplexer configuration
fn validate_multiplexer(_multiplexer: &Multiplexer) -> Result<()> {
    // Multiplexer enum values are already constrained by the type system
    // Additional validation can be added here if needed
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_valid_config() {
        let config = PhantomConfig {
            post_create: Some(PostCreateConfig {
                copy_files: Some(vec![".env".to_string(), "config.json".to_string()]),
                commands: Some(vec!["npm install".to_string()]),
            }),
            default_multiplexer: Some(Multiplexer::Tmux),
        };

        assert!(validate_config(&config).is_ok());
    }

    #[test]
    fn test_validate_empty_config() {
        let config = PhantomConfig::default();
        assert!(validate_config(&config).is_ok());
    }

    #[test]
    fn test_validate_empty_copy_file() {
        let config = PhantomConfig {
            post_create: Some(PostCreateConfig {
                copy_files: Some(vec!["".to_string()]),
                commands: None,
            }),
            default_multiplexer: None,
        };

        let result = validate_config(&config);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("cannot contain empty strings"));
    }

    #[test]
    fn test_validate_absolute_path_copy_file() {
        let config = PhantomConfig {
            post_create: Some(PostCreateConfig {
                copy_files: Some(vec!["/etc/passwd".to_string()]),
                commands: None,
            }),
            default_multiplexer: None,
        };

        let result = validate_config(&config);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("cannot contain absolute paths"));
    }

    #[test]
    fn test_validate_parent_directory_copy_file() {
        let config = PhantomConfig {
            post_create: Some(PostCreateConfig {
                copy_files: Some(vec!["../secret.txt".to_string()]),
                commands: None,
            }),
            default_multiplexer: None,
        };

        let result = validate_config(&config);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("cannot contain parent directory references"));
    }

    #[test]
    fn test_validate_empty_command() {
        let config = PhantomConfig {
            post_create: Some(PostCreateConfig {
                copy_files: None,
                commands: Some(vec!["   ".to_string()]),
            }),
            default_multiplexer: None,
        };

        let result = validate_config(&config);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("cannot contain empty strings"));
    }

    #[test]
    fn test_validate_all_multiplexers() {
        for multiplexer in [Multiplexer::Tmux, Multiplexer::Kitty, Multiplexer::None] {
            let config =
                PhantomConfig { post_create: None, default_multiplexer: Some(multiplexer) };
            assert!(validate_config(&config).is_ok());
        }
    }
}
