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

#[cfg(test)]
mod tests {
    use super::*;
    use std::io;

    #[test]
    fn test_worktree_error_display() {
        let err = WorktreeError::NotFound("test-worktree".to_string());
        assert_eq!(err.to_string(), "Worktree 'test-worktree' not found");

        let err = WorktreeError::AlreadyExists("existing".to_string());
        assert_eq!(err.to_string(), "Worktree 'existing' already exists");

        let err = WorktreeError::InvalidName("bad/name".to_string());
        assert_eq!(err.to_string(), "Invalid worktree name: 'bad/name'");

        let err = WorktreeError::GitOperation {
            operation: "checkout".to_string(),
            details: "failed to checkout".to_string(),
        };
        assert_eq!(err.to_string(), "Git checkout failed: failed to checkout");

        let err = WorktreeError::BranchNotFound("missing-branch".to_string());
        assert_eq!(err.to_string(), "Branch 'missing-branch' not found");

        let err = WorktreeError::FileOperation("permission denied".to_string());
        assert_eq!(err.to_string(), "File operation failed: permission denied");

        let err = WorktreeError::Path("/invalid/path".to_string());
        assert_eq!(err.to_string(), "Path error: /invalid/path");
    }

    #[test]
    fn test_worktree_error_to_phantom_error() {
        // Test NotFound conversion
        let err = WorktreeError::NotFound("test".to_string());
        let phantom_err: PhantomError = err.into();
        match phantom_err {
            PhantomError::Worktree(msg) => {
                assert!(msg.contains("test") && msg.contains("not found"))
            }
            _ => panic!("Expected PhantomError::Worktree"),
        }

        // Test AlreadyExists conversion
        let err = WorktreeError::AlreadyExists("existing".to_string());
        let phantom_err: PhantomError = err.into();
        match phantom_err {
            PhantomError::Worktree(msg) => {
                assert!(msg.contains("existing") && msg.contains("already exists"))
            }
            _ => panic!("Expected PhantomError::Worktree"),
        }

        // Test InvalidName conversion
        let err = WorktreeError::InvalidName("bad".to_string());
        let phantom_err: PhantomError = err.into();
        match phantom_err {
            PhantomError::Validation(msg) => assert!(msg.contains("bad")),
            _ => panic!("Expected PhantomError::Validation"),
        }

        // Test GitOperation conversion
        let err = WorktreeError::GitOperation {
            operation: "push".to_string(),
            details: "remote rejected".to_string(),
        };
        let phantom_err: PhantomError = err.into();
        match phantom_err {
            PhantomError::Git { message, exit_code } => {
                assert!(message.contains("push") && message.contains("remote rejected"));
                assert_eq!(exit_code, 1);
            }
            _ => panic!("Expected PhantomError::Git"),
        }

        // Test BranchNotFound conversion
        let err = WorktreeError::BranchNotFound("feature".to_string());
        let phantom_err: PhantomError = err.into();
        match phantom_err {
            PhantomError::Git { message, exit_code } => {
                assert!(message.contains("feature") && message.contains("not found"));
                assert_eq!(exit_code, 1);
            }
            _ => panic!("Expected PhantomError::Git"),
        }

        // Test FileOperation conversion
        let err = WorktreeError::FileOperation("access denied".to_string());
        let phantom_err: PhantomError = err.into();
        match phantom_err {
            PhantomError::FileOperation(msg) => assert_eq!(msg, "access denied"),
            _ => panic!("Expected PhantomError::FileOperation"),
        }

        // Test Path conversion
        let err = WorktreeError::Path("invalid path".to_string());
        let phantom_err: PhantomError = err.into();
        match phantom_err {
            PhantomError::Path(msg) => assert_eq!(msg, "invalid path"),
            _ => panic!("Expected PhantomError::Path"),
        }
    }

    #[test]
    fn test_worktree_error_from_io_error() {
        let io_err = io::Error::new(io::ErrorKind::NotFound, "file not found");
        let worktree_err = WorktreeError::Io(io_err);
        assert!(worktree_err.to_string().contains("IO error"));

        // Test conversion to PhantomError
        let io_err = io::Error::new(io::ErrorKind::PermissionDenied, "denied");
        let worktree_err = WorktreeError::Io(io_err);
        let phantom_err: PhantomError = worktree_err.into();
        match phantom_err {
            PhantomError::Io(_) => {}
            _ => panic!("Expected PhantomError::Io"),
        }
    }

    #[test]
    fn test_worktree_error_debug() {
        let err = WorktreeError::NotFound("test".to_string());
        let debug_str = format!("{:?}", err);
        assert!(debug_str.contains("NotFound"));
        assert!(debug_str.contains("test"));
    }
}
