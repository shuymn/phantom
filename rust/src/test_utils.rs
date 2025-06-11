use crate::Result;
use std::path::{Path, PathBuf};
use tempfile::{tempdir, TempDir};
use tokio::process::Command;

/// Test utility for creating temporary git repositories
pub struct TestRepo {
    pub dir: TempDir,
    pub path: PathBuf,
}

impl TestRepo {
    /// Create a new test repository
    pub async fn new() -> Result<Self> {
        let dir = tempdir().map_err(crate::PhantomError::Io)?;
        let path = dir.path().to_path_buf();

        // Initialize git repo
        Command::new("git").args(["init"]).current_dir(&path).output().await.map_err(|e| {
            crate::PhantomError::ProcessExecution(format!("Failed to init git repo: {}", e))
        })?;

        // Configure git user for tests
        Command::new("git")
            .args(["config", "user.name", "Test User"])
            .current_dir(&path)
            .output()
            .await
            .map_err(|e| {
                crate::PhantomError::ProcessExecution(format!("Failed to set user.name: {}", e))
            })?;

        Command::new("git")
            .args(["config", "user.email", "test@example.com"])
            .current_dir(&path)
            .output()
            .await
            .map_err(|e| {
                crate::PhantomError::ProcessExecution(format!("Failed to set user.email: {}", e))
            })?;

        Ok(Self { dir, path })
    }

    /// Create a test file and commit it
    pub async fn create_file_and_commit(
        &self,
        filename: &str,
        content: &str,
        message: &str,
    ) -> Result<()> {
        let file_path = self.path.join(filename);
        tokio::fs::write(&file_path, content).await.map_err(crate::PhantomError::Io)?;

        Command::new("git")
            .args(["add", filename])
            .current_dir(&self.path)
            .output()
            .await
            .map_err(|e| {
                crate::PhantomError::ProcessExecution(format!("Failed to add file: {}", e))
            })?;

        Command::new("git")
            .args(["commit", "-m", message])
            .current_dir(&self.path)
            .output()
            .await
            .map_err(|e| {
                crate::PhantomError::ProcessExecution(format!("Failed to commit: {}", e))
            })?;

        Ok(())
    }

    /// Create a new branch
    pub async fn create_branch(&self, branch_name: &str) -> Result<()> {
        Command::new("git")
            .args(["checkout", "-b", branch_name])
            .current_dir(&self.path)
            .output()
            .await
            .map_err(|e| {
                crate::PhantomError::ProcessExecution(format!("Failed to create branch: {}", e))
            })?;

        Ok(())
    }

    /// Get the path to the test repository
    pub fn path(&self) -> &Path {
        &self.path
    }
}

/// Create a test configuration file
pub async fn create_test_config(path: &Path, content: &str) -> Result<()> {
    let config_path = path.join("phantom.config.json");
    tokio::fs::write(&config_path, content).await.map_err(crate::PhantomError::Io)?;
    Ok(())
}

/// Assert that a command succeeds
#[macro_export]
macro_rules! assert_cmd_success {
    ($output:expr) => {
        assert!(
            $output.status.success(),
            "Command failed with exit code: {}. stderr: {}",
            $output.status.code().unwrap_or(-1),
            String::from_utf8_lossy(&$output.stderr)
        );
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_create_test_repo() {
        let repo = TestRepo::new().await.unwrap();
        assert!(repo.path.exists());
        assert!(repo.path.join(".git").exists());
    }

    #[tokio::test]
    async fn test_create_file_and_commit() {
        let repo = TestRepo::new().await.unwrap();
        repo.create_file_and_commit("test.txt", "Hello, world!", "Initial commit").await.unwrap();

        assert!(repo.path.join("test.txt").exists());
    }

    #[tokio::test]
    async fn test_create_branch() {
        let repo = TestRepo::new().await.unwrap();
        repo.create_file_and_commit("test.txt", "Hello", "Initial commit").await.unwrap();
        repo.create_branch("feature-branch").await.unwrap();

        // Verify branch was created
        let output =
            Command::new("git").args(["branch"]).current_dir(&repo.path).output().await.unwrap();

        let branches = String::from_utf8_lossy(&output.stdout);
        assert!(branches.contains("feature-branch"));
    }
}
