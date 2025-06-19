use std::process;

/// Exit codes for the CLI
pub struct ExitCode;

impl ExitCode {
    pub const SUCCESS: i32 = 0;
    pub const GENERAL_ERROR: i32 = 1;
    pub const VALIDATION_ERROR: i32 = 2;
    pub const NOT_IN_GIT_REPO: i32 = 3;
    pub const WORKTREE_EXISTS: i32 = 4;
    pub const WORKTREE_NOT_FOUND: i32 = 5;
    pub const BRANCH_NOT_FOUND: i32 = 6;
    pub const CONFIG_ERROR: i32 = 7;
    pub const EXEC_ERROR: i32 = 8;
}

/// Exit with an error message and code
pub fn exit_with_error(message: &str, code: i32) -> ! {
    eprintln!("Error: {}", message);
    process::exit(code);
}

/// Exit with success
pub fn exit_with_success() -> ! {
    process::exit(ExitCode::SUCCESS);
}

/// Convert a PhantomError to an appropriate exit code
pub fn error_to_exit_code(error: &crate::PhantomError) -> i32 {
    use crate::PhantomError;

    match error {
        PhantomError::NotInGitRepository => ExitCode::NOT_IN_GIT_REPO,
        PhantomError::WorktreeExists { .. } => ExitCode::WORKTREE_EXISTS,
        PhantomError::WorktreeNotFound { .. } => ExitCode::WORKTREE_NOT_FOUND,
        PhantomError::BranchNotFound { .. } => ExitCode::BRANCH_NOT_FOUND,
        PhantomError::Config(_) => ExitCode::CONFIG_ERROR,
        PhantomError::ProcessExecution(_) => ExitCode::EXEC_ERROR,
        PhantomError::Validation(_) => ExitCode::VALIDATION_ERROR,
        _ => ExitCode::GENERAL_ERROR,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::PhantomError;

    #[test]
    fn test_exit_codes() {
        assert_eq!(ExitCode::SUCCESS, 0);
        assert_eq!(ExitCode::GENERAL_ERROR, 1);
        assert_eq!(ExitCode::VALIDATION_ERROR, 2);
        assert_eq!(ExitCode::NOT_IN_GIT_REPO, 3);
        assert_eq!(ExitCode::WORKTREE_EXISTS, 4);
        assert_eq!(ExitCode::WORKTREE_NOT_FOUND, 5);
        assert_eq!(ExitCode::BRANCH_NOT_FOUND, 6);
        assert_eq!(ExitCode::CONFIG_ERROR, 7);
        assert_eq!(ExitCode::EXEC_ERROR, 8);
    }

    #[test]
    fn test_error_to_exit_code() {
        assert_eq!(
            error_to_exit_code(&PhantomError::NotInGitRepository),
            ExitCode::NOT_IN_GIT_REPO
        );

        assert_eq!(
            error_to_exit_code(&PhantomError::WorktreeExists { name: "test".to_string() }),
            ExitCode::WORKTREE_EXISTS
        );

        assert_eq!(
            error_to_exit_code(&PhantomError::WorktreeNotFound { name: "test".to_string() }),
            ExitCode::WORKTREE_NOT_FOUND
        );

        assert_eq!(
            error_to_exit_code(&PhantomError::BranchNotFound { branch: "test".to_string() }),
            ExitCode::BRANCH_NOT_FOUND
        );

        assert_eq!(
            error_to_exit_code(&PhantomError::Config("test error".to_string())),
            ExitCode::CONFIG_ERROR
        );

        assert_eq!(
            error_to_exit_code(&PhantomError::ProcessExecution("test error".to_string())),
            ExitCode::EXEC_ERROR
        );

        assert_eq!(
            error_to_exit_code(&PhantomError::Validation("test error".to_string())),
            ExitCode::VALIDATION_ERROR
        );

        // Test general error fallback
        assert_eq!(
            error_to_exit_code(&PhantomError::Io(std::io::Error::new(
                std::io::ErrorKind::Other,
                "test"
            ))),
            ExitCode::GENERAL_ERROR
        );
    }
}
