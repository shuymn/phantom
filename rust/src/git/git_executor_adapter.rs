use crate::core::command_executor::{CommandConfig, CommandExecutor};
use crate::git::const_utils::commands;
use crate::worktree::const_validate::timeouts::GIT_OPERATION_TIMEOUT;
use crate::{PhantomError, Result};
use std::path::Path;
use std::time::Duration;
use tracing::{debug, trace};

/// Git command executor that uses CommandExecutor internally
#[derive(Clone)]
pub struct GitExecutor<E>
where
    E: CommandExecutor + Clone,
{
    executor: E,
    cwd: Option<String>,
    timeout_duration: Duration,
}

impl<E> GitExecutor<E>
where
    E: CommandExecutor + Clone,
{
    /// Create a new GitExecutor with a CommandExecutor
    pub fn new(executor: E) -> Self {
        Self { executor, cwd: None, timeout_duration: GIT_OPERATION_TIMEOUT }
    }

    /// Create a GitExecutor with a specific working directory
    pub fn with_cwd<P: AsRef<Path>>(mut self, cwd: P) -> Self {
        self.cwd = Some(cwd.as_ref().to_string_lossy().to_string());
        self
    }

    /// Set a custom timeout for git operations
    pub fn with_timeout(mut self, duration: Duration) -> Self {
        self.timeout_duration = duration;
        self
    }

    /// Run a git command with arguments
    pub async fn run(&self, args: &[&str]) -> Result<String> {
        debug!("Running git command: git {:?}", args);

        let mut config = CommandConfig::new(commands::GIT)
            .with_args(args.iter().map(|s| s.to_string()).collect())
            .with_timeout(self.timeout_duration);

        if let Some(ref cwd) = self.cwd {
            config = config.with_cwd(cwd.into());
        }

        let output = self.executor.execute(config).await?;

        trace!("Git command stdout: {}", output.stdout);
        trace!("Git command stderr: {}", output.stderr);

        if output.success() {
            Ok(output.stdout.trim().to_string())
        } else {
            let exit_code = output.exit_code;

            Err(PhantomError::Git {
                message: if output.stderr.is_empty() {
                    format!("git {} failed with exit code {}", args.join(" "), exit_code)
                } else {
                    output.stderr.trim().to_string()
                },
                exit_code,
            })
        }
    }

    /// Run a git command and return the output lines
    pub async fn run_lines(&self, args: &[&str]) -> Result<Vec<String>> {
        let output = self.run(args).await?;
        Ok(output.lines().filter(|line| !line.is_empty()).map(|s| s.to_string()).collect())
    }

    /// Check if we're in a git repository
    pub async fn is_in_git_repo(&self) -> bool {
        self.run(&["rev-parse", "--git-dir"]).await.is_ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::executors::MockCommandExecutor;

    #[tokio::test]
    async fn test_git_executor_with_mock() {
        let mut mock = MockCommandExecutor::new();
        mock.expect_command("git").with_args(&["status", "--short"]).returns_output(
            "M file.txt\n",
            "",
            0,
        );

        let executor = GitExecutor::new(mock);
        let result = executor.run(&["status", "--short"]).await.unwrap();
        assert_eq!(result, "M file.txt");
    }

    #[tokio::test]
    async fn test_git_executor_with_cwd() {
        let mut mock = MockCommandExecutor::new();
        mock.expect_command("git").with_args(&["status"]).in_dir("/test/repo").returns_success();

        let executor = GitExecutor::new(mock).with_cwd("/test/repo");
        let result = executor.run(&["status"]).await.unwrap();
        assert_eq!(result, "");
    }

    #[tokio::test]
    async fn test_git_executor_error() {
        let mut mock = MockCommandExecutor::new();
        mock.expect_command("git").with_args(&["invalid"]).returns_output(
            "",
            "git: 'invalid' is not a git command",
            1,
        );

        let executor = GitExecutor::new(mock);
        let result = executor.run(&["invalid"]).await;
        assert!(result.is_err());

        match result.unwrap_err() {
            PhantomError::Git { message, exit_code } => {
                assert_eq!(message, "git: 'invalid' is not a git command");
                assert_eq!(exit_code, 1);
            }
            _ => panic!("Expected Git error"),
        }
    }

    #[tokio::test]
    async fn test_run_lines() {
        let mut mock = MockCommandExecutor::new();
        mock.expect_command("git").with_args(&["branch", "-a"]).returns_output(
            "main\n  feature/test\n  feature/another\n",
            "",
            0,
        );

        let executor = GitExecutor::new(mock);
        let lines = executor.run_lines(&["branch", "-a"]).await.unwrap();
        assert_eq!(lines, vec!["main", "  feature/test", "  feature/another"]);
    }

    #[tokio::test]
    async fn test_is_in_git_repo() {
        let mut mock = MockCommandExecutor::new();
        mock.expect_command("git")
            .with_args(&["rev-parse", "--git-dir"])
            .returns_output(".git", "", 0);

        let executor = GitExecutor::new(mock);
        assert!(executor.is_in_git_repo().await);
    }

    #[tokio::test]
    async fn test_is_not_in_git_repo() {
        let mut mock = MockCommandExecutor::new();
        mock.expect_command("git").with_args(&["rev-parse", "--git-dir"]).returns_output(
            "",
            "fatal: not a git repository",
            128,
        );

        let executor = GitExecutor::new(mock);
        assert!(!executor.is_in_git_repo().await);
    }
}
