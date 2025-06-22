use crate::{PhantomError, Result};
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use tracing::error;

/// Get the exit code for a PhantomError as an integer
fn error_to_exit_code_int(error: &PhantomError) -> i32 {
    match error {
        PhantomError::Git { exit_code, .. } => *exit_code,
        PhantomError::NotInGitRepository => 128,
        PhantomError::WorktreeExists { .. } => 2,
        PhantomError::WorktreeNotFound { .. } => 3,
        PhantomError::BranchNotFound { .. } => 4,
        PhantomError::InvalidWorktreeName { .. } => 5,
        PhantomError::ConfigNotFound { .. } => 6,
        PhantomError::ConfigInvalid { .. } => 6,
        PhantomError::MultiplexerNotFound { .. } => 7,
        PhantomError::CommandNotFound { .. } => 8,
        PhantomError::ProcessFailed { .. } => 8,
        PhantomError::ProcessExecutionError { .. } => 8,
        PhantomError::UnsupportedFeature { .. } => 9,
        PhantomError::Io(_) => 10,
        PhantomError::Json(_) => 11,
        PhantomError::WorktreeDirectoryCreationFailed { .. } => 12,
        PhantomError::WorktreeHasUncommittedChanges { .. } => 12,
        PhantomError::CannotDeleteCurrent { .. } => 12,
        PhantomError::ValidationFailed { .. } => 13,
        PhantomError::FileOperationFailed { .. } => 14,
        PhantomError::InvalidPath { .. } => 15,
    }
}

/// Convert a PhantomError to an exit code
pub fn error_to_exit_code(error: &PhantomError) -> ExitCode {
    ExitCode::from(error_to_exit_code_int(error) as u8)
}

/// Handle and display an error, then exit with appropriate code
pub fn handle_error(error: PhantomError) -> ! {
    error!("{}", error);
    eprintln!("Error: {}", error);
    std::process::exit(error_to_exit_code_int(&error));
}

/// Ensure a path is absolute
pub fn ensure_absolute_path(path: &Path) -> Result<PathBuf> {
    if path.is_absolute() {
        Ok(path.to_path_buf())
    } else {
        std::env::current_dir().map(|cwd| cwd.join(path)).map_err(PhantomError::Io)
    }
}

/// Check if a command exists in PATH
pub fn command_exists(command: &str) -> bool {
    which::which(command).is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_to_exit_code() {
        // Test Git error with custom exit code
        let error = PhantomError::Git {
            command: "git".to_string(),
            args: vec!["status".to_string()],
            exit_code: 128,
            stderr: "test".to_string(),
        };
        assert_eq!(error_to_exit_code(&error), ExitCode::from(128));

        let error = PhantomError::Git {
            command: "git".to_string(),
            args: vec!["status".to_string()],
            exit_code: 64,
            stderr: "test".to_string(),
        };
        assert_eq!(error_to_exit_code(&error), ExitCode::from(64));

        // Test all error variants
        let error = PhantomError::NotInGitRepository;
        assert_eq!(error_to_exit_code(&error), ExitCode::from(128));

        let error = PhantomError::WorktreeExists { name: "test".to_string() };
        assert_eq!(error_to_exit_code(&error), ExitCode::from(2));

        let error = PhantomError::WorktreeNotFound { name: "test".to_string() };
        assert_eq!(error_to_exit_code(&error), ExitCode::from(3));

        let error = PhantomError::BranchNotFound { branch: "test".to_string() };
        assert_eq!(error_to_exit_code(&error), ExitCode::from(4));

        let error = PhantomError::InvalidWorktreeName {
            name: "test".to_string(),
            reason: "invalid".to_string(),
        };
        assert_eq!(error_to_exit_code(&error), ExitCode::from(5));

        let error = PhantomError::ConfigInvalid { reason: "test".to_string() };
        assert_eq!(error_to_exit_code(&error), ExitCode::from(6));

        let error = PhantomError::MultiplexerNotFound { name: "tmux".to_string() };
        assert_eq!(error_to_exit_code(&error), ExitCode::from(7));

        let error = PhantomError::ProcessExecutionError { reason: "test".to_string() };
        assert_eq!(error_to_exit_code(&error), ExitCode::from(8));

        let error = PhantomError::UnsupportedFeature {
            feature: "test".to_string(),
            platform: "linux".to_string(),
        };
        assert_eq!(error_to_exit_code(&error), ExitCode::from(9));

        let error = PhantomError::Io(std::io::Error::new(std::io::ErrorKind::Other, "test"));
        assert_eq!(error_to_exit_code(&error), ExitCode::from(10));

        let error = PhantomError::Json(serde_json::from_str::<String>("invalid").unwrap_err());
        assert_eq!(error_to_exit_code(&error), ExitCode::from(11));

        let error = PhantomError::WorktreeHasUncommittedChanges { name: "test".to_string() };
        assert_eq!(error_to_exit_code(&error), ExitCode::from(12));

        let error = PhantomError::ValidationFailed { reason: "test".to_string() };
        assert_eq!(error_to_exit_code(&error), ExitCode::from(13));

        let error = PhantomError::FileOperationFailed {
            operation: "read".to_string(),
            path: PathBuf::from("/test"),
            reason: "test".to_string(),
        };
        assert_eq!(error_to_exit_code(&error), ExitCode::from(14));

        let error =
            PhantomError::InvalidPath { path: "test".to_string(), reason: "invalid".to_string() };
        assert_eq!(error_to_exit_code(&error), ExitCode::from(15));
    }

    #[test]
    fn test_ensure_absolute_path() {
        // Test with absolute path
        let abs_path = Path::new("/tmp/test");
        let result = ensure_absolute_path(abs_path).unwrap();
        assert_eq!(result, PathBuf::from("/tmp/test"));

        // Test with another absolute path
        let abs_path = Path::new("/usr/local/bin");
        let result = ensure_absolute_path(abs_path).unwrap();
        assert_eq!(result, PathBuf::from("/usr/local/bin"));

        // For relative paths, we can't test the exact result since it depends on cwd
        let rel_path = Path::new("test");
        let result = ensure_absolute_path(rel_path).unwrap();
        assert!(result.is_absolute());
        assert!(result.ends_with("test"));

        // Test with relative path containing subdirectories
        let rel_path = Path::new("test/subdir");
        let result = ensure_absolute_path(rel_path).unwrap();
        assert!(result.is_absolute());
        assert!(result.ends_with("test/subdir"));

        // Test with current directory
        let rel_path = Path::new(".");
        let result = ensure_absolute_path(rel_path).unwrap();
        assert!(result.is_absolute());

        // Test with parent directory
        let rel_path = Path::new("..");
        let result = ensure_absolute_path(rel_path).unwrap();
        assert!(result.is_absolute());
    }

    #[test]
    fn test_command_exists() {
        // These commands should exist on Unix systems
        assert!(command_exists("ls"));
        assert!(command_exists("echo"));
        assert!(command_exists("cat"));
        assert!(command_exists("pwd"));

        // Git should exist in test environment
        assert!(command_exists("git"));

        // This command should not exist
        assert!(!command_exists("this_command_definitely_does_not_exist"));
        assert!(!command_exists("fake_command_12345"));
        assert!(!command_exists(""));
    }

    #[test]
    fn test_error_display() {
        // Test that errors display correctly
        let error = PhantomError::NotInGitRepository;
        let display = format!("{}", error);
        assert!(display.contains("git repository"));

        let error = PhantomError::WorktreeExists { name: "test-worktree".to_string() };
        let display = format!("{}", error);
        assert!(display.contains("test-worktree"));

        let error = PhantomError::ConfigInvalid { reason: "Invalid config".to_string() };
        let display = format!("{}", error);
        assert!(display.contains("Invalid config"));
    }

    #[test]
    fn test_phantom_error_all_variants() {
        // Ensure all error variants have proper display implementations
        let errors: Vec<PhantomError> = vec![
            PhantomError::Git {
                command: "git".to_string(),
                args: vec!["status".to_string()],
                exit_code: 1,
                stderr: "git error".to_string(),
            },
            PhantomError::NotInGitRepository,
            PhantomError::WorktreeExists { name: "wt".to_string() },
            PhantomError::WorktreeNotFound { name: "wt".to_string() },
            PhantomError::BranchNotFound { branch: "br".to_string() },
            PhantomError::InvalidWorktreeName {
                name: "invalid".to_string(),
                reason: "test".to_string(),
            },
            PhantomError::ConfigInvalid { reason: "cfg".to_string() },
            PhantomError::MultiplexerNotFound { name: "tmux".to_string() },
            PhantomError::ProcessExecutionError { reason: "proc".to_string() },
            PhantomError::UnsupportedFeature {
                feature: "feat".to_string(),
                platform: "linux".to_string(),
            },
            PhantomError::Io(std::io::Error::new(std::io::ErrorKind::NotFound, "io")),
            PhantomError::Json(serde_json::from_str::<String>("bad").unwrap_err()),
            PhantomError::WorktreeHasUncommittedChanges { name: "wt".to_string() },
            PhantomError::ValidationFailed { reason: "val".to_string() },
            PhantomError::FileOperationFailed {
                operation: "test".to_string(),
                path: PathBuf::from("/test"),
                reason: "file".to_string(),
            },
            PhantomError::InvalidPath { path: "path".to_string(), reason: "test".to_string() },
        ];

        for error in errors {
            // Test that display doesn't panic
            let _ = format!("{}", error);
            // Test that exit code conversion doesn't panic
            let _ = error_to_exit_code(&error);
        }
    }
}
