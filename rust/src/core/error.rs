use std::io;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum PhantomError {
    #[error("Git operation failed: {message}")]
    Git { message: String, exit_code: i32 },

    #[error("IO error: {0}")]
    Io(#[from] io::Error),

    #[error("Worktree '{name}' already exists")]
    WorktreeExists { name: String },

    #[error("Worktree '{name}' not found")]
    WorktreeNotFound { name: String },

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Terminal multiplexer not found: {0}")]
    MultiplexerNotFound(String),

    #[error("Invalid worktree name: {0}")]
    InvalidWorktreeName(String),

    #[error("Not in a git repository")]
    NotInGitRepository,

    #[error("Branch '{branch}' not found")]
    BranchNotFound { branch: String },

    #[error("JSON parsing error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Process execution failed: {0}")]
    ProcessExecution(String),

    #[error("Feature not supported: {0}")]
    UnsupportedFeature(String),
}