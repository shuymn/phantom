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

    #[error("Worktree error: {0}")]
    Worktree(String),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("File operation error: {0}")]
    FileOperation(String),

    #[error("Path error: {0}")]
    Path(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_phantom_error_display() {
        let err =
            PhantomError::Git { message: "failed to create worktree".to_string(), exit_code: 128 };
        assert_eq!(err.to_string(), "Git operation failed: failed to create worktree");

        let err = PhantomError::WorktreeExists { name: "feature-branch".to_string() };
        assert_eq!(err.to_string(), "Worktree 'feature-branch' already exists");

        let err = PhantomError::WorktreeNotFound { name: "missing-branch".to_string() };
        assert_eq!(err.to_string(), "Worktree 'missing-branch' not found");

        let err = PhantomError::Config("invalid configuration".to_string());
        assert_eq!(err.to_string(), "Configuration error: invalid configuration");

        let err = PhantomError::MultiplexerNotFound("tmux".to_string());
        assert_eq!(err.to_string(), "Terminal multiplexer not found: tmux");

        let err = PhantomError::InvalidWorktreeName("invalid/name".to_string());
        assert_eq!(err.to_string(), "Invalid worktree name: invalid/name");

        let err = PhantomError::NotInGitRepository;
        assert_eq!(err.to_string(), "Not in a git repository");

        let err = PhantomError::BranchNotFound { branch: "missing".to_string() };
        assert_eq!(err.to_string(), "Branch 'missing' not found");

        let err = PhantomError::ProcessExecution("command failed".to_string());
        assert_eq!(err.to_string(), "Process execution failed: command failed");

        let err = PhantomError::UnsupportedFeature("Windows".to_string());
        assert_eq!(err.to_string(), "Feature not supported: Windows");
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
        let err = PhantomError::Git { message: "test".to_string(), exit_code: 1 };
        let debug_str = format!("{:?}", err);
        assert!(debug_str.contains("Git"));
        assert!(debug_str.contains("message"));
        assert!(debug_str.contains("exit_code"));
    }
}
