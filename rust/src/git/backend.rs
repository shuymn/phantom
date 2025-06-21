use crate::core::sealed::Sealed;
use crate::core::types::Worktree;
use crate::Result;
use async_trait::async_trait;
use std::path::{Path, PathBuf};

/// Trait defining the interface for Git operations
/// This abstraction allows for different implementations (command-line, libgit2, etc.)
///
/// This trait is sealed to prevent downstream implementations and maintain API stability
#[async_trait]
pub trait GitBackend: Sealed + Send + Sync {
    /// Get the current branch name
    async fn current_branch(&self) -> Result<String>;

    /// List all branches
    async fn list_branches(&self) -> Result<Vec<String>>;

    /// Create a new branch
    async fn create_branch(&self, name: &str) -> Result<()>;

    /// Check if a branch exists
    async fn branch_exists(&self, name: &str) -> Result<bool>;

    /// Get the repository root
    async fn get_root(&self) -> Result<PathBuf>;

    /// List all worktrees
    async fn list_worktrees(&self) -> Result<Vec<Worktree>>;

    /// Add a new worktree
    async fn add_worktree(
        &self,
        path: &Path,
        branch: Option<&str>,
        new_branch: bool,
        commitish: Option<&str>,
    ) -> Result<()>;

    /// Attach a worktree to an existing branch
    async fn attach_worktree(&self, path: &Path, branch: &str) -> Result<()>;

    /// Remove a worktree
    async fn remove_worktree(&self, path: &Path) -> Result<()>;

    /// Get the current commit hash
    async fn current_commit(&self) -> Result<String>;

    /// Check if the current directory is inside a git repository
    async fn is_inside_work_tree(&self) -> Result<bool>;

    /// Get the current worktree information (returns None if in main worktree)
    async fn current_worktree(&self) -> Result<Option<String>>;
}

/// Configuration for a Git backend
#[derive(Debug, Clone)]
pub struct GitConfig {
    /// Working directory for git operations
    pub cwd: Option<PathBuf>,
    /// Environment variables to set
    pub env: Vec<(String, String)>,
    /// Timeout for operations in seconds
    pub timeout: Option<u64>,
}

impl Default for GitConfig {
    fn default() -> Self {
        Self { cwd: None, env: Vec::new(), timeout: Some(30) }
    }
}

impl GitConfig {
    /// Create a new GitConfig with the specified working directory
    pub fn with_cwd(cwd: impl Into<PathBuf>) -> Self {
        Self { cwd: Some(cwd.into()), ..Default::default() }
    }

    /// Add an environment variable
    pub fn with_env(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.env.push((key.into(), value.into()));
        self
    }

    /// Set the timeout
    pub fn with_timeout(mut self, timeout: u64) -> Self {
        self.timeout = Some(timeout);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_git_config_default() {
        let config = GitConfig::default();
        assert!(config.cwd.is_none());
        assert!(config.env.is_empty());
        assert_eq!(config.timeout, Some(30));
    }

    #[test]
    fn test_git_config_with_cwd() {
        let config = GitConfig::with_cwd("/tmp/repo");
        assert_eq!(config.cwd, Some(PathBuf::from("/tmp/repo")));
        assert!(config.env.is_empty());
        assert_eq!(config.timeout, Some(30));
    }

    #[test]
    fn test_git_config_with_cwd_pathbuf() {
        let path = PathBuf::from("/home/user/project");
        let config = GitConfig::with_cwd(path.clone());
        assert_eq!(config.cwd, Some(path));
    }

    #[test]
    fn test_git_config_with_env() {
        let config = GitConfig::default()
            .with_env("GIT_AUTHOR_NAME", "Test User")
            .with_env("GIT_AUTHOR_EMAIL", "test@example.com");

        assert_eq!(config.env.len(), 2);
        assert!(config.env.contains(&("GIT_AUTHOR_NAME".to_string(), "Test User".to_string())));
        assert!(config
            .env
            .contains(&("GIT_AUTHOR_EMAIL".to_string(), "test@example.com".to_string())));
    }

    #[test]
    fn test_git_config_with_timeout() {
        let config = GitConfig::default().with_timeout(60);
        assert_eq!(config.timeout, Some(60));
    }

    #[test]
    fn test_git_config_builder_chain() {
        let config = GitConfig::with_cwd("/tmp/repo")
            .with_env("KEY1", "value1")
            .with_env("KEY2", "value2")
            .with_timeout(120);

        assert_eq!(config.cwd, Some(PathBuf::from("/tmp/repo")));
        assert_eq!(config.env.len(), 2);
        assert_eq!(config.timeout, Some(120));
    }

    #[test]
    fn test_git_config_debug() {
        let config = GitConfig::default();
        let debug_str = format!("{:?}", config);
        assert!(debug_str.contains("GitConfig"));
        assert!(debug_str.contains("cwd"));
        assert!(debug_str.contains("env"));
        assert!(debug_str.contains("timeout"));
    }

    #[test]
    fn test_git_config_clone() {
        let config = GitConfig::with_cwd("/tmp").with_env("TEST", "value").with_timeout(45);

        let cloned = config.clone();
        assert_eq!(config.cwd, cloned.cwd);
        assert_eq!(config.env, cloned.env);
        assert_eq!(config.timeout, cloned.timeout);
    }
}
