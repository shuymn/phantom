use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Options for creating a new worktree
#[derive(Debug, Clone, Default)]
pub struct CreateWorktreeOptions {
    /// Branch name to create (defaults to worktree name)
    pub branch: Option<String>,
    /// Commit/branch to base the new worktree on (defaults to HEAD)
    pub commitish: Option<String>,
    /// Files to copy from the source worktree
    pub copy_files: Option<Vec<String>>,
}

/// Result of a successful worktree creation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateWorktreeSuccess {
    pub message: String,
    pub path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub copied_files: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub skipped_files: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub copy_error: Option<String>,
}

/// Options for deleting a worktree
#[derive(Debug, Clone, Default)]
pub struct DeleteWorktreeOptions {
    /// Force deletion even if there are untracked/modified files
    pub force: bool,
}

/// Result of a successful worktree deletion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteWorktreeSuccess {
    pub message: String,
    pub path: String,
}

/// Result of worktree validation
#[derive(Debug, Clone)]
pub struct WorktreeExistsSuccess {
    pub path: PathBuf,
}

/// Result of worktree non-existence validation
#[derive(Debug, Clone)]
pub struct WorktreeDoesNotExistSuccess {
    pub path: PathBuf,
}
