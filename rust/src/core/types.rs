use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Represents a Git worktree
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Worktree {
    pub name: String,
    pub path: PathBuf,
    pub branch: Option<String>,
    pub commit: String,
    pub is_bare: bool,
    pub is_detached: bool,
    pub is_prunable: bool,
}

/// Git configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitConfig {
    pub user_name: Option<String>,
    pub user_email: Option<String>,
}

/// Phantom configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhantomConfig {
    #[serde(default)]
    pub copy_files: Vec<String>,
    #[serde(default)]
    pub terminal: TerminalConfig,
}

/// Terminal configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerminalConfig {
    #[serde(default = "default_multiplexer")]
    pub multiplexer: String,
}

impl Default for TerminalConfig {
    fn default() -> Self {
        Self { multiplexer: default_multiplexer() }
    }
}

fn default_multiplexer() -> String {
    "auto".to_string()
}

impl Default for PhantomConfig {
    fn default() -> Self {
        Self { copy_files: vec![], terminal: TerminalConfig { multiplexer: default_multiplexer() } }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn test_worktree_creation() {
        let worktree = Worktree {
            name: "feature-branch".to_string(),
            path: PathBuf::from("/path/to/worktree"),
            branch: Some("feature-branch".to_string()),
            commit: "abc123".to_string(),
            is_bare: false,
            is_detached: false,
            is_prunable: false,
        };

        assert_eq!(worktree.name, "feature-branch");
        assert_eq!(worktree.path, Path::new("/path/to/worktree"));
        assert_eq!(worktree.branch, Some("feature-branch".to_string()));
        assert_eq!(worktree.commit, "abc123");
        assert!(!worktree.is_bare);
        assert!(!worktree.is_detached);
        assert!(!worktree.is_prunable);
    }

    #[test]
    fn test_worktree_serialization() {
        let worktree = Worktree {
            name: "test".to_string(),
            path: PathBuf::from("/test/path"),
            branch: None,
            commit: "def456".to_string(),
            is_bare: true,
            is_detached: true,
            is_prunable: true,
        };

        // Serialize
        let json = serde_json::to_string(&worktree).unwrap();
        assert!(json.contains("\"name\":\"test\""));
        assert!(json.contains("\"is_bare\":true"));

        // Deserialize
        let deserialized: Worktree = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.name, worktree.name);
        assert_eq!(deserialized.path, worktree.path);
        assert_eq!(deserialized.is_bare, worktree.is_bare);
    }

    #[test]
    fn test_git_config() {
        let config = GitConfig {
            user_name: Some("John Doe".to_string()),
            user_email: Some("john@example.com".to_string()),
        };

        assert_eq!(config.user_name, Some("John Doe".to_string()));
        assert_eq!(config.user_email, Some("john@example.com".to_string()));

        let empty_config = GitConfig { user_name: None, user_email: None };

        assert!(empty_config.user_name.is_none());
        assert!(empty_config.user_email.is_none());
    }

    #[test]
    fn test_phantom_config_default() {
        let config = PhantomConfig::default();

        assert!(config.copy_files.is_empty());
        assert_eq!(config.terminal.multiplexer, "auto");
    }

    #[test]
    fn test_phantom_config_serialization() {
        let config = PhantomConfig {
            copy_files: vec![".env".to_string(), "config.toml".to_string()],
            terminal: TerminalConfig { multiplexer: "tmux".to_string() },
        };

        // Serialize
        let json = serde_json::to_string_pretty(&config).unwrap();
        assert!(json.contains("\"copy_files\""));
        assert!(json.contains("\".env\""));
        assert!(json.contains("\"multiplexer\": \"tmux\""));

        // Deserialize
        let deserialized: PhantomConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.copy_files.len(), 2);
        assert_eq!(deserialized.copy_files[0], ".env");
        assert_eq!(deserialized.terminal.multiplexer, "tmux");
    }

    #[test]
    fn test_terminal_config_default() {
        let config = TerminalConfig::default();
        assert_eq!(config.multiplexer, "auto");
    }

    #[test]
    fn test_terminal_config_custom() {
        let config = TerminalConfig { multiplexer: "kitty".to_string() };
        assert_eq!(config.multiplexer, "kitty");
    }

    #[test]
    fn test_partial_config_deserialization() {
        // Test with minimal JSON
        let json = "{}";
        let config: PhantomConfig = serde_json::from_str(json).unwrap();
        assert!(config.copy_files.is_empty());
        assert_eq!(config.terminal.multiplexer, "auto");

        // Test with only copy_files
        let json = r#"{"copy_files": [".env"]}"#;
        let config: PhantomConfig = serde_json::from_str(json).unwrap();
        assert_eq!(config.copy_files.len(), 1);
        assert_eq!(config.copy_files[0], ".env");
        assert_eq!(config.terminal.multiplexer, "auto");
    }
}
