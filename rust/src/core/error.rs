use std::io;
use std::path::PathBuf;
use std::time::Duration;
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

/// Context information for command execution errors
#[derive(Debug, Clone)]
pub struct CommandContext {
    pub command: String,
    pub args: Vec<String>,
    pub working_dir: Option<PathBuf>,
    pub duration: Option<Duration>,
}

impl CommandContext {
    /// Create a new command context
    pub fn new(command: impl Into<String>) -> Self {
        Self {
            command: command.into(),
            args: Vec::new(),
            working_dir: None,
            duration: None,
        }
    }

    /// Add arguments to the context
    pub fn with_args(mut self, args: Vec<String>) -> Self {
        self.args = args;
        self
    }

    /// Set the working directory
    pub fn with_dir(mut self, dir: PathBuf) -> Self {
        self.working_dir = Some(dir);
        self
    }

    /// Set the duration
    pub fn with_duration(mut self, duration: Duration) -> Self {
        self.duration = Some(duration);
        self
    }
}

/// Extension trait for adding context to errors
pub trait ErrorContext {
    /// Add context information to the error message
    fn context(self, msg: &str) -> Self;
}

impl ErrorContext for PhantomError {
    fn context(self, msg: &str) -> Self {
        match self {
            PhantomError::Git { message, exit_code } => PhantomError::Git {
                message: format!("{}: {}", msg, message),
                exit_code,
            },
            PhantomError::ProcessExecution(message) => {
                PhantomError::ProcessExecution(format!("{}: {}", msg, message))
            }
            PhantomError::Worktree(message) => {
                PhantomError::Worktree(format!("{}: {}", msg, message))
            }
            PhantomError::FileOperation(message) => {
                PhantomError::FileOperation(format!("{}: {}", msg, message))
            }
            _ => self,
        }
    }
}

/// Extension trait for Results to add context
pub trait ResultContext<T> {
    /// Add context to an error
    fn context(self, msg: &str) -> Result<T, PhantomError>;
    
    /// Add context with a closure that's only called on error
    fn with_context<F>(self, f: F) -> Result<T, PhantomError>
    where
        F: FnOnce() -> String;
}

impl<T, E> ResultContext<T> for Result<T, E>
where
    E: Into<PhantomError>,
{
    fn context(self, msg: &str) -> Result<T, PhantomError> {
        self.map_err(|e| e.into().context(msg))
    }

    fn with_context<F>(self, f: F) -> Result<T, PhantomError>
    where
        F: FnOnce() -> String,
    {
        self.map_err(|e| e.into().context(&f()))
    }
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

    #[test]
    fn test_error_context() {
        let err = PhantomError::Git { message: "command failed".to_string(), exit_code: 1 };
        let err_with_context = err.context("While creating worktree");
        assert_eq!(
            err_with_context.to_string(),
            "Git operation failed: While creating worktree: command failed"
        );
    }

    #[test]
    fn test_result_context() {
        let result: Result<(), io::Error> =
            Err(io::Error::new(io::ErrorKind::NotFound, "file not found"));
        let result_with_context = result.context("While reading config");
        assert!(result_with_context.is_err());
        
        // IO errors get converted to PhantomError::Io which doesn't support context
        // So we'll test with a different error type
        let result: Result<(), PhantomError> = Err(PhantomError::ProcessExecution("failed".to_string()));
        let result_with_context = result.context("While running command");
        assert_eq!(
            result_with_context.unwrap_err().to_string(),
            "Process execution failed: While running command: failed"
        );
    }

    #[test]
    fn test_command_context() {
        let context = CommandContext::new("git")
            .with_args(vec!["status".to_string()])
            .with_dir(PathBuf::from("/tmp"))
            .with_duration(Duration::from_secs(2));

        assert_eq!(context.command, "git");
        assert_eq!(context.args, vec!["status"]);
        assert_eq!(context.working_dir, Some(PathBuf::from("/tmp")));
        assert_eq!(context.duration, Some(Duration::from_secs(2)));
    }
}
