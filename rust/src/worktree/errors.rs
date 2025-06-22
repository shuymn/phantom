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
            WorktreeError::NotFound(name) => PhantomError::WorktreeNotFound { name },
            WorktreeError::AlreadyExists(name) => PhantomError::WorktreeExists { name },
            WorktreeError::InvalidName(reason) => {
                PhantomError::InvalidWorktreeName { name: String::new(), reason }
            }
            WorktreeError::GitOperation { operation, details } => PhantomError::Git {
                command: "git".to_string(),
                args: vec![operation.clone()],
                exit_code: 1,
                stderr: details,
            },
            WorktreeError::BranchNotFound(branch) => PhantomError::BranchNotFound { branch },
            WorktreeError::FileOperation(msg) => PhantomError::FileOperationFailed {
                operation: "unknown".to_string(),
                path: std::path::PathBuf::new(),
                reason: msg,
            },
            WorktreeError::Io(err) => PhantomError::Io(err),
            WorktreeError::Path(msg) => {
                PhantomError::InvalidPath { path: String::new(), reason: msg }
            }
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
            PhantomError::WorktreeNotFound { name } => {
                assert_eq!(name, "test");
            }
            _ => panic!("Expected PhantomError::WorktreeNotFound"),
        }

        // Test AlreadyExists conversion
        let err = WorktreeError::AlreadyExists("existing".to_string());
        let phantom_err: PhantomError = err.into();
        match phantom_err {
            PhantomError::WorktreeExists { name } => {
                assert_eq!(name, "existing");
            }
            _ => panic!("Expected PhantomError::WorktreeExists"),
        }

        // Test InvalidName conversion
        let err = WorktreeError::InvalidName("bad name".to_string());
        let phantom_err: PhantomError = err.into();
        match phantom_err {
            PhantomError::InvalidWorktreeName { name: _, reason } => {
                assert_eq!(reason, "bad name");
            }
            _ => panic!("Expected PhantomError::InvalidWorktreeName"),
        }

        // Test GitOperation conversion
        let err = WorktreeError::GitOperation {
            operation: "push".to_string(),
            details: "remote rejected".to_string(),
        };
        let phantom_err: PhantomError = err.into();
        match phantom_err {
            PhantomError::Git { command, args, exit_code, stderr } => {
                assert_eq!(command, "git");
                assert_eq!(args, vec!["push"]);
                assert_eq!(exit_code, 1);
                assert_eq!(stderr, "remote rejected");
            }
            _ => panic!("Expected PhantomError::Git"),
        }

        // Test BranchNotFound conversion
        let err = WorktreeError::BranchNotFound("feature".to_string());
        let phantom_err: PhantomError = err.into();
        match phantom_err {
            PhantomError::BranchNotFound { branch } => {
                assert_eq!(branch, "feature");
            }
            _ => panic!("Expected PhantomError::BranchNotFound"),
        }

        // Test FileOperation conversion
        let err = WorktreeError::FileOperation("access denied".to_string());
        let phantom_err: PhantomError = err.into();
        match phantom_err {
            PhantomError::FileOperationFailed { operation, path: _, reason } => {
                assert_eq!(operation, "unknown");
                assert_eq!(reason, "access denied");
            }
            _ => panic!("Expected PhantomError::FileOperationFailed"),
        }

        // Test Path conversion
        let err = WorktreeError::Path("invalid path".to_string());
        let phantom_err: PhantomError = err.into();
        match phantom_err {
            PhantomError::InvalidPath { path: _, reason } => {
                assert_eq!(reason, "invalid path");
            }
            _ => panic!("Expected PhantomError::InvalidPath"),
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
