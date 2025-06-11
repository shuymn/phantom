use crate::{PhantomError, Result};
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use tracing::error;

/// Convert a PhantomError to an exit code
pub fn error_to_exit_code(error: &PhantomError) -> ExitCode {
    match error {
        PhantomError::Git { exit_code, .. } => ExitCode::from(*exit_code as u8),
        PhantomError::NotInGitRepository => ExitCode::from(128),
        PhantomError::WorktreeExists { .. } => ExitCode::from(2),
        PhantomError::WorktreeNotFound { .. } => ExitCode::from(3),
        PhantomError::BranchNotFound { .. } => ExitCode::from(4),
        PhantomError::InvalidWorktreeName(_) => ExitCode::from(5),
        PhantomError::Config(_) => ExitCode::from(6),
        PhantomError::MultiplexerNotFound(_) => ExitCode::from(7),
        PhantomError::ProcessExecution(_) => ExitCode::from(8),
        PhantomError::UnsupportedFeature(_) => ExitCode::from(9),
        PhantomError::Io(_) => ExitCode::from(10),
        PhantomError::Json(_) => ExitCode::from(11),
        PhantomError::Worktree(_) => ExitCode::from(12),
        PhantomError::Validation(_) => ExitCode::from(13),
        PhantomError::FileOperation(_) => ExitCode::from(14),
        PhantomError::Path(_) => ExitCode::from(15),
    }
}

/// Handle and display an error, then exit with appropriate code
pub fn handle_error(error: PhantomError) -> ! {
    error!("{}", error);
    eprintln!("Error: {}", error);
    let exit_code = error_to_exit_code(&error);
    std::process::exit(match exit_code {
        ExitCode::SUCCESS => 0,
        _code => {
            // ExitCode doesn't have a direct conversion to u8/i32, so we have to match the error again
            match &error {
                PhantomError::Git { exit_code, .. } => *exit_code,
                _ => match &error {
                    PhantomError::NotInGitRepository => 128,
                    PhantomError::WorktreeExists { .. } => 2,
                    PhantomError::WorktreeNotFound { .. } => 3,
                    PhantomError::BranchNotFound { .. } => 4,
                    PhantomError::InvalidWorktreeName(_) => 5,
                    PhantomError::Config(_) => 6,
                    PhantomError::MultiplexerNotFound(_) => 7,
                    PhantomError::ProcessExecution(_) => 8,
                    PhantomError::UnsupportedFeature(_) => 9,
                    PhantomError::Io(_) => 10,
                    PhantomError::Json(_) => 11,
                    PhantomError::Worktree(_) => 12,
                    PhantomError::Validation(_) => 13,
                    PhantomError::FileOperation(_) => 14,
                    PhantomError::Path(_) => 15,
                    PhantomError::Git { .. } => unreachable!(),
                },
            }
        }
    });
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
        let error = PhantomError::Git { message: "test".to_string(), exit_code: 128 };
        assert_eq!(error_to_exit_code(&error), ExitCode::from(128));

        let error = PhantomError::NotInGitRepository;
        assert_eq!(error_to_exit_code(&error), ExitCode::from(128));

        let error = PhantomError::WorktreeExists { name: "test".to_string() };
        assert_eq!(error_to_exit_code(&error), ExitCode::from(2));
    }

    #[test]
    fn test_ensure_absolute_path() {
        let abs_path = Path::new("/tmp/test");
        let result = ensure_absolute_path(abs_path).unwrap();
        assert_eq!(result, PathBuf::from("/tmp/test"));

        // For relative paths, we can't test the exact result since it depends on cwd
        let rel_path = Path::new("test");
        let result = ensure_absolute_path(rel_path).unwrap();
        assert!(result.is_absolute());
    }

    #[test]
    fn test_command_exists() {
        // These commands should exist on Unix systems
        assert!(command_exists("ls"));
        assert!(command_exists("echo"));

        // This command should not exist
        assert!(!command_exists("this_command_definitely_does_not_exist"));
    }
}
