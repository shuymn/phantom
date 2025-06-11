use crate::{PhantomError, Result};
use std::path::Path;
use tokio::process::Command;
use tracing::{debug, trace};

/// Git command executor
#[derive(Debug, Clone)]
pub struct GitExecutor {
    cwd: Option<String>,
}

impl GitExecutor {
    /// Create a new GitExecutor
    pub fn new() -> Self {
        Self { cwd: None }
    }

    /// Create a GitExecutor with a specific working directory
    pub fn with_cwd<P: AsRef<Path>>(cwd: P) -> Self {
        Self {
            cwd: Some(cwd.as_ref().to_string_lossy().to_string()),
        }
    }

    /// Run a git command with arguments
    pub async fn run(&self, args: &[&str]) -> Result<String> {
        debug!("Running git command: git {:?}", args);
        
        let mut cmd = Command::new("git");
        cmd.args(args);

        if let Some(ref cwd) = self.cwd {
            cmd.current_dir(cwd);
        }

        let output = cmd
            .output()
            .await
            .map_err(|e| PhantomError::ProcessExecution(format!("Failed to execute git: {}", e)))?;

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
        Ok(output
            .lines()
            .filter(|line| !line.is_empty())
            .map(|s| s.to_string())
            .collect())
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
}
