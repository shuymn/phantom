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
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TerminalConfig {
    #[serde(default = "default_multiplexer")]
    pub multiplexer: String,
}

fn default_multiplexer() -> String {
    "auto".to_string()
}

impl Default for PhantomConfig {
    fn default() -> Self {
        Self {
            copy_files: vec![],
            terminal: TerminalConfig {
                multiplexer: default_multiplexer(),
            },
        }
    }
}