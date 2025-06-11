use crate::core::types::Worktree;
use crate::Result;
use async_trait::async_trait;
use std::path::{Path, PathBuf};

/// Trait defining the interface for Git operations
/// This abstraction allows for different implementations (command-line, libgit2, etc.)
#[async_trait]
pub trait GitBackend: Send + Sync {
    /// Initialize a new git repository
    async fn init(&self, path: &Path) -> Result<()>;

    /// Clone a repository
    async fn clone(&self, url: &str, path: &Path) -> Result<()>;

    /// Add files to the staging area
    async fn add(&self, paths: &[&str]) -> Result<()>;

    /// Commit changes
    async fn commit(&self, message: &str) -> Result<String>;

    /// Get the current branch name
    async fn current_branch(&self) -> Result<String>;

    /// List all branches
    async fn list_branches(&self) -> Result<Vec<String>>;

    /// Create a new branch
    async fn create_branch(&self, name: &str) -> Result<()>;

    /// Switch to a branch
    async fn checkout(&self, branch: &str) -> Result<()>;

    /// Check if a branch exists
    async fn branch_exists(&self, name: &str) -> Result<bool>;

    /// Get the repository root
    async fn get_root(&self) -> Result<PathBuf>;

    /// List all worktrees
    async fn list_worktrees(&self) -> Result<Vec<Worktree>>;

    /// Add a new worktree
    async fn add_worktree(&self, path: &Path, branch: Option<&str>, new_branch: bool)
        -> Result<()>;

    /// Attach a worktree to an existing branch
    async fn attach_worktree(&self, path: &Path, branch: &str) -> Result<()>;

    /// Remove a worktree
    async fn remove_worktree(&self, path: &Path) -> Result<()>;

    /// Get the status of the repository
    async fn status(&self) -> Result<String>;

    /// Get the current commit hash
    async fn current_commit(&self) -> Result<String>;

    /// Check if the current directory is inside a git repository
    async fn is_inside_work_tree(&self) -> Result<bool>;

    /// Get the current worktree information (returns None if in main worktree)
    async fn current_worktree(&self) -> Result<Option<String>>;

    /// Execute a raw git command (for operations not covered by the trait)
    async fn execute(&self, args: &[&str]) -> Result<String>;
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
