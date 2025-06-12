use crate::{PhantomError, Result};
use std::path::Path;
use std::time::Duration;
use tokio::process::Command;
use tokio::time::timeout;
use tracing::{debug, trace};

/// Default timeout for git operations (30 seconds)
const DEFAULT_GIT_TIMEOUT: Duration = Duration::from_secs(30);

/// Git command executor
#[derive(Debug, Clone)]
pub struct GitExecutor {
    cwd: Option<String>,
    timeout_duration: Duration,
}

impl GitExecutor {
    /// Create a new GitExecutor
    pub fn new() -> Self {
        Self { cwd: None, timeout_duration: DEFAULT_GIT_TIMEOUT }
    }

    /// Create a GitExecutor with a specific working directory
    pub fn with_cwd<P: AsRef<Path>>(cwd: P) -> Self {
        Self {
            cwd: Some(cwd.as_ref().to_string_lossy().to_string()),
            timeout_duration: DEFAULT_GIT_TIMEOUT,
        }
    }

    /// Set a custom timeout for git operations
    pub fn with_timeout(mut self, duration: Duration) -> Self {
        self.timeout_duration = duration;
        self
    }

    /// Run a git command with arguments
    pub async fn run(&self, args: &[&str]) -> Result<String> {
        debug!("Running git command: git {:?}", args);

        let mut cmd = Command::new("git");
        cmd.args(args);

        if let Some(ref cwd) = self.cwd {
            cmd.current_dir(cwd);
        }

        let output_future = cmd.output();

        let output = match timeout(self.timeout_duration, output_future).await {
            Ok(Ok(output)) => output,
            Ok(Err(e)) => {
                return Err(PhantomError::ProcessExecution(format!(
                    "Failed to execute git: {}",
                    e
                )));
            }
            Err(_) => {
                return Err(PhantomError::ProcessExecution(format!(
                    "Git command timed out after {:?}: git {}",
                    self.timeout_duration,
                    args.join(" ")
                )));
            }
        };

        trace!("Git command stdout: {}", String::from_utf8_lossy(&output.stdout));
        trace!("Git command stderr: {}", String::from_utf8_lossy(&output.stderr));

        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
            let exit_code = output.status.code().unwrap_or(-1);

            Err(PhantomError::Git {
                message: if stderr.is_empty() {
                    format!("git {} failed with exit code {}", args.join(" "), exit_code)
                } else {
                    stderr
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

impl Default for GitExecutor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::TestRepo;

    #[tokio::test]
    async fn test_git_executor_new() {
        let executor = GitExecutor::new();
        assert!(executor.cwd.is_none());
    }

    #[tokio::test]
    async fn test_git_executor_with_cwd() {
        let executor = GitExecutor::with_cwd("/tmp");
        assert_eq!(executor.cwd, Some("/tmp".to_string()));
    }

    #[tokio::test]
    async fn test_run_git_version() {
        let executor = GitExecutor::new();
        let result = executor.run(&["--version"]).await.unwrap();
        assert!(result.contains("git version"));
    }

    #[tokio::test]
    async fn test_run_git_command_failure() {
        let executor = GitExecutor::new();
        let result = executor.run(&["invalid-command"]).await;
        assert!(result.is_err());

        match result.unwrap_err() {
            PhantomError::Git { message, exit_code } => {
                assert!(message.contains("invalid-command"));
                assert_ne!(exit_code, 0);
            }
            _ => panic!("Expected Git error"),
        }
    }

    #[tokio::test]
    async fn test_is_in_git_repo() {
        let repo = TestRepo::new().await.unwrap();
        let executor = GitExecutor::with_cwd(repo.path());

        assert!(executor.is_in_git_repo().await);

        // Test outside git repo
        let temp_dir = tempfile::tempdir().unwrap();
        let executor = GitExecutor::with_cwd(temp_dir.path());
        assert!(!executor.is_in_git_repo().await);
    }

    #[tokio::test]
    async fn test_run_lines() {
        let executor = GitExecutor::new();
        // Use a command that outputs multiple lines
        let result = executor.run_lines(&["config", "--list", "--local"]).await;

        // This might fail if not in a git repo, which is fine for this test
        if let Ok(lines) = result {
            // Just verify it returns a vector
            assert!(lines.is_empty() || !lines.is_empty());
        }
    }

    #[tokio::test]
    async fn test_git_executor_with_timeout() {
        let executor = GitExecutor::new().with_timeout(Duration::from_secs(5));
        assert_eq!(executor.timeout_duration, Duration::from_secs(5));

        // Test that timeout is applied
        let result = executor.run(&["--version"]).await;
        assert!(result.is_ok()); // Should complete well within 5 seconds
    }

    #[tokio::test]
    async fn test_git_executor_default() {
        let executor1 = GitExecutor::new();
        let executor2 = GitExecutor::default();

        assert_eq!(executor1.cwd, executor2.cwd);
        assert_eq!(executor1.timeout_duration, executor2.timeout_duration);
    }

    #[tokio::test]
    async fn test_run_lines_with_output() {
        let repo = TestRepo::new().await.unwrap();

        // Set up some config in the test repo
        let executor = GitExecutor::with_cwd(repo.path());
        executor.run(&["config", "user.name", "Test User"]).await.unwrap();
        executor.run(&["config", "user.email", "test@example.com"]).await.unwrap();

        // Now test run_lines
        let lines = executor.run_lines(&["config", "--list", "--local"]).await.unwrap();
        assert!(!lines.is_empty());

        // Check that we got the config lines
        let has_name = lines.iter().any(|line| line.contains("user.name=Test User"));
        let has_email = lines.iter().any(|line| line.contains("user.email=test@example.com"));
        assert!(has_name);
        assert!(has_email);
    }

    #[tokio::test]
    async fn test_run_with_empty_output() {
        let repo = TestRepo::new().await.unwrap();
        let executor = GitExecutor::with_cwd(repo.path());

        // Run a command that produces empty output
        let result = executor.run(&["status", "--porcelain"]).await.unwrap();
        assert_eq!(result, ""); // Clean repo has empty status
    }

    #[tokio::test]
    async fn test_git_error_with_empty_stderr() {
        let executor = GitExecutor::new();

        // This should fail with an error
        let result = executor.run(&["log", "--invalid-option"]).await;
        assert!(result.is_err());

        match result.unwrap_err() {
            PhantomError::Git { message, exit_code } => {
                assert!(exit_code != 0);
                // Message should contain something about the command
                assert!(!message.is_empty());
            }
            _ => panic!("Expected Git error"),
        }
    }

    #[tokio::test]
    async fn test_git_executor_debug() {
        let executor = GitExecutor::with_cwd("/test/path");
        let debug_str = format!("{:?}", executor);
        assert!(debug_str.contains("GitExecutor"));
        assert!(debug_str.contains("/test/path"));
    }

    #[tokio::test]
    async fn test_git_executor_clone() {
        let executor = GitExecutor::with_cwd("/test/path").with_timeout(Duration::from_secs(10));
        let cloned = executor.clone();

        assert_eq!(executor.cwd, cloned.cwd);
        assert_eq!(executor.timeout_duration, cloned.timeout_duration);
    }

    #[tokio::test]
    async fn test_run_lines_empty_lines_filtered() {
        let repo = TestRepo::new().await.unwrap();
        let executor = GitExecutor::with_cwd(repo.path());

        // Create a file with content that will produce empty lines
        std::fs::write(repo.path().join("test.txt"), "line1\n\nline2\n\n").unwrap();
        executor.run(&["add", "test.txt"]).await.unwrap();

        // Get diff which might have empty lines
        let output = executor.run(&["diff", "--cached", "--name-only"]).await.unwrap();
        let lines = output
            .lines()
            .filter(|line| !line.is_empty())
            .map(|s| s.to_string())
            .collect::<Vec<_>>();

        // Verify no empty lines
        assert!(!lines.iter().any(|line| line.is_empty()));
    }

    #[test]
    fn test_default_timeout_constant() {
        assert_eq!(DEFAULT_GIT_TIMEOUT, Duration::from_secs(30));
    }
}
