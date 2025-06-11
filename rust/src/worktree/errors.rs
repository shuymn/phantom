use crate::PhantomError;
use thiserror::Error;

/// Base error type for worktree operations
#[derive(Error, Debug)]
pub enum WorktreeError {
    #[error("Worktree '{0}' not found")]
    NotFound(String),

    #[error("Worktree '{0}' already exists")]
    AlreadyExists(String),

    #[error("Invalid worktree name: '{0}'")]
    InvalidName(String),

    #[error("Git {operation} failed: {details}")]
    GitOperation { operation: String, details: String },

    #[error("Branch '{0}' not found")]
    BranchNotFound(String),

    #[error("File operation failed: {0}")]
    FileOperation(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Path error: {0}")]
    Path(String),
}

impl From<WorktreeError> for PhantomError {
    fn from(err: WorktreeError) -> Self {
        match err {
            WorktreeError::NotFound(name) => {
                PhantomError::Worktree(format!("Worktree '{}' not found", name))
            }
            WorktreeError::AlreadyExists(name) => {
                PhantomError::Worktree(format!("Worktree '{}' already exists", name))
            }
            WorktreeError::InvalidName(name) => {
                PhantomError::Validation(format!("Invalid worktree name: '{}'", name))
            }
            WorktreeError::GitOperation { operation, details } => PhantomError::Git {
                message: format!("Git {} failed: {}", operation, details),
                exit_code: 1,
            },
            WorktreeError::BranchNotFound(branch) => PhantomError::Git {
                message: format!("Branch '{}' not found", branch),
                exit_code: 1,
            },
            WorktreeError::FileOperation(msg) => PhantomError::FileOperation(msg),
            WorktreeError::Io(err) => PhantomError::Io(err),
            WorktreeError::Path(msg) => PhantomError::Path(msg),
        }
    }
}
