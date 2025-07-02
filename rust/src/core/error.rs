use std::io;
use std::path::PathBuf;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum PhantomError {
    #[error("Git command '{command}' failed with exit code {exit_code}")]
    Git { command: String, args: Vec<String>, exit_code: i32, stderr: String },

    #[error("IO error: {0}")]
    Io(#[from] io::Error),

    #[error("Worktree '{name}' already exists")]
    WorktreeExists { name: String },

    #[error("Worktree '{name}' not found")]
    WorktreeNotFound { name: String },

    #[error("Cannot delete current worktree '{name}'")]
    CannotDeleteCurrent { name: String },

    #[error("Worktree '{name}' has uncommitted changes")]
    WorktreeHasUncommittedChanges { name: String },

    #[error("Failed to create worktree directory at {path}")]
    WorktreeDirectoryCreationFailed { path: PathBuf },

    #[error("Configuration file not found at {path}")]
    ConfigNotFound { path: PathBuf },

    #[error("Invalid configuration: {reason}")]
    ConfigInvalid { reason: String },

    #[error("Terminal multiplexer '{name}' not found")]
    MultiplexerNotFound { name: String },

    #[error("Invalid worktree name '{name}': {reason}")]
    InvalidWorktreeName { name: String, reason: String },

    #[error("Not in a git repository")]
    NotInGitRepository,

    #[error("Branch '{branch}' not found")]
    BranchNotFound { branch: String },

    #[error("JSON parsing error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Command '{command}' not found in PATH")]
    CommandNotFound { command: String },

    #[error("Process '{command}' exited with code {code}")]
    ProcessFailed { command: String, code: i32 },

    #[error("Process execution failed: {reason}")]
    ProcessExecutionError { reason: String },

    #[error("Feature '{feature}' not supported on {platform}")]
    UnsupportedFeature { feature: String, platform: String },

    #[error("File operation '{operation}' failed on '{path}': {reason}")]
    FileOperationFailed { operation: String, path: PathBuf, reason: String },

    #[error("Path '{path}' is not valid: {reason}")]
    InvalidPath { path: String, reason: String },

    #[error("Validation failed: {reason}")]
    ValidationFailed { reason: String },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_phantom_error_display() {
        let err = PhantomError::Git {
            command: "git".to_string(),
            args: vec!["worktree".to_string(), "add".to_string()],
            exit_code: 128,
            stderr: "error: failed to create worktree".to_string(),
        };
        assert_eq!(err.to_string(), "Git command 'git' failed with exit code 128");

        let err = PhantomError::WorktreeExists { name: "feature-branch".to_string() };
        assert_eq!(err.to_string(), "Worktree 'feature-branch' already exists");

        let err = PhantomError::WorktreeNotFound { name: "missing-branch".to_string() };
        assert_eq!(err.to_string(), "Worktree 'missing-branch' not found");

        let err = PhantomError::CannotDeleteCurrent { name: "current".to_string() };
        assert_eq!(err.to_string(), "Cannot delete current worktree 'current'");

        let err = PhantomError::WorktreeHasUncommittedChanges { name: "dirty".to_string() };
        assert_eq!(err.to_string(), "Worktree 'dirty' has uncommitted changes");

        let err = PhantomError::ConfigInvalid { reason: "missing field".to_string() };
        assert_eq!(err.to_string(), "Invalid configuration: missing field");

        let err = PhantomError::MultiplexerNotFound { name: "tmux".to_string() };
        assert_eq!(err.to_string(), "Terminal multiplexer 'tmux' not found");

        let err = PhantomError::InvalidWorktreeName {
            name: "invalid/name".to_string(),
            reason: "contains slash".to_string(),
        };
        assert_eq!(err.to_string(), "Invalid worktree name 'invalid/name': contains slash");

        let err = PhantomError::NotInGitRepository;
        assert_eq!(err.to_string(), "Not in a git repository");

        let err = PhantomError::BranchNotFound { branch: "missing".to_string() };
        assert_eq!(err.to_string(), "Branch 'missing' not found");

        let err = PhantomError::CommandNotFound { command: "phantom".to_string() };
        assert_eq!(err.to_string(), "Command 'phantom' not found in PATH");

        let err = PhantomError::ProcessFailed { command: "ls".to_string(), code: 1 };
        assert_eq!(err.to_string(), "Process 'ls' exited with code 1");

        let err = PhantomError::UnsupportedFeature {
            feature: "symlinks".to_string(),
            platform: "Windows".to_string(),
        };
        assert_eq!(err.to_string(), "Feature 'symlinks' not supported on Windows");
    }

    #[test]
    fn test_io_error_conversion() {
        use std::io::ErrorKind;

        let io_err = io::Error::new(ErrorKind::NotFound, "file not found");
        let phantom_err: PhantomError = io_err.into();

        match phantom_err {
            PhantomError::Io(_) => {
                assert!(phantom_err.to_string().contains("IO error"));
            }
            _ => panic!("Expected PhantomError::Io"),
        }
    }

    #[test]
    fn test_json_error_conversion() {
        let json_str = "{ invalid json";
        let result: Result<serde_json::Value, _> = serde_json::from_str(json_str);

        match result {
            Err(json_err) => {
                let phantom_err: PhantomError = json_err.into();
                match phantom_err {
                    PhantomError::Json(_) => {
                        assert!(phantom_err.to_string().contains("JSON parsing error"));
                    }
                    _ => panic!("Expected PhantomError::Json"),
                }
            }
            Ok(_) => panic!("Expected JSON parsing to fail"),
        }
    }

    #[test]
    fn test_error_debug_format() {
        let err = PhantomError::Git {
            command: "git".to_string(),
            args: vec!["status".to_string()],
            exit_code: 1,
            stderr: "error".to_string(),
        };
        let debug_str = format!("{err:?}");
        assert!(debug_str.contains("Git"));
        assert!(debug_str.contains("command"));
        assert!(debug_str.contains("args"));
        assert!(debug_str.contains("exit_code"));
    }

    #[test]
    fn test_file_operation_error() {
        let err = PhantomError::FileOperationFailed {
            operation: "read".to_string(),
            path: PathBuf::from("/tmp/test.txt"),
            reason: "permission denied".to_string(),
        };
        assert_eq!(
            err.to_string(),
            "File operation 'read' failed on '/tmp/test.txt': permission denied"
        );
    }

    #[test]
    fn test_path_error() {
        let err = PhantomError::InvalidPath {
            path: "../../../etc/passwd".to_string(),
            reason: "path traversal detected".to_string(),
        };
        assert_eq!(
            err.to_string(),
            "Path '../../../etc/passwd' is not valid: path traversal detected"
        );
    }
}
