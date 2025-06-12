use crate::git::executor::GitExecutor;
use crate::git::libs::list_worktrees::list_worktrees as git_list_worktrees;
use crate::worktree::paths::get_phantom_directory;
use crate::Result;
use serde::{Deserialize, Serialize};
use std::path::Path;
use tracing::debug;

/// Information about a worktree
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorktreeInfo {
    pub name: String,
    pub path: String,
    pub branch: Option<String>,
    pub is_clean: bool,
}

/// Result of listing worktrees
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListWorktreesSuccess {
    pub worktrees: Vec<WorktreeInfo>,
    pub message: Option<String>,
}

/// Get the current branch of a worktree
pub async fn get_worktree_branch(worktree_path: &Path) -> Result<String> {
    let executor = GitExecutor::with_cwd(worktree_path);
    match executor.run(&["branch", "--show-current"]).await {
        Ok(output) => {
            let branch = output.trim().to_string();
            Ok(if branch.is_empty() { "(detached HEAD)".to_string() } else { branch })
        }
        Err(_) => Ok("unknown".to_string()),
    }
}

/// Get the status of a worktree (clean/dirty)
pub async fn get_worktree_status(worktree_path: &Path) -> Result<bool> {
    let executor = GitExecutor::with_cwd(worktree_path);
    match executor.run(&["status", "--porcelain"]).await {
        Ok(output) => Ok(output.trim().is_empty()), // Clean if no output
        Err(_) => Ok(true),                         // If git status fails, assume clean
    }
}

/// Get detailed information about a worktree
pub async fn get_worktree_info(git_root: &Path, name: &str) -> Result<WorktreeInfo> {
    let worktree_path = get_phantom_directory(git_root).join(name);

    let (branch, is_clean) =
        tokio::join!(get_worktree_branch(&worktree_path), get_worktree_status(&worktree_path));

    Ok(WorktreeInfo {
        name: name.to_string(),
        path: worktree_path.to_string_lossy().to_string(),
        branch: Some(branch.unwrap_or_else(|_| "unknown".to_string())),
        is_clean: is_clean.unwrap_or(true),
    })
}

/// List all phantom worktrees
pub async fn list_worktrees(git_root: &Path) -> Result<ListWorktreesSuccess> {
    debug!("Listing worktrees from git root: {:?}", git_root);

    let git_worktrees = git_list_worktrees(git_root).await?;
    let phantom_dir = get_phantom_directory(git_root);
    let phantom_dir_str = phantom_dir.to_string_lossy();

    // Filter worktrees to only include those in the phantom directory
    let mut phantom_worktrees = Vec::new();
    for worktree in git_worktrees {
        if worktree.path.starts_with(&phantom_dir) {
            // Extract the name from the path
            let path_str = worktree.path.to_string_lossy();
            let name =
                if let Some(stripped) = path_str.strip_prefix(&format!("{}/", phantom_dir_str)) {
                    stripped.to_string()
                } else {
                    worktree.name.clone()
                };

            let is_clean = get_worktree_status(&worktree.path).await.unwrap_or(true);

            phantom_worktrees.push(WorktreeInfo {
                name,
                path: worktree.path.to_string_lossy().to_string(),
                branch: worktree.branch,
                is_clean,
            });
        }
    }

    let message =
        if phantom_worktrees.is_empty() { Some("No worktrees found".to_string()) } else { None };

    Ok(ListWorktreesSuccess { worktrees: phantom_worktrees, message })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::TestRepo;
    use crate::worktree::create::create_worktree;
    use crate::worktree::types::CreateWorktreeOptions;

    #[tokio::test]
    async fn test_list_empty_worktrees() {
        let repo = TestRepo::new().await.unwrap();
        repo.create_file_and_commit("test.txt", "content", "Initial commit").await.unwrap();

        let result = list_worktrees(repo.path()).await.unwrap();
        assert!(result.worktrees.is_empty());
        assert_eq!(result.message, Some("No worktrees found".to_string()));
    }

    // TODO: This test is flaky in the test environment.
    // The worktrees are created successfully but git worktree list
    // doesn't always reflect them immediately.
    // This functionality is tested in integration tests instead.

    #[tokio::test]
    async fn test_get_worktree_branch() {
        let repo = TestRepo::new().await.unwrap();
        repo.create_file_and_commit("test.txt", "content", "Initial commit").await.unwrap();

        // Create a worktree
        let options =
            CreateWorktreeOptions { branch: Some("test-branch".to_string()), ..Default::default() };
        create_worktree(repo.path(), "test-branch", options).await.unwrap();

        let worktree_path = get_phantom_directory(repo.path()).join("test-branch");
        let branch = get_worktree_branch(&worktree_path).await.unwrap();
        assert_eq!(branch, "test-branch");
    }

    #[tokio::test]
    async fn test_get_worktree_status_clean() {
        let repo = TestRepo::new().await.unwrap();
        repo.create_file_and_commit("test.txt", "content", "Initial commit").await.unwrap();

        // Create a worktree
        let options = CreateWorktreeOptions::default();
        create_worktree(repo.path(), "test-status", options).await.unwrap();

        let worktree_path = get_phantom_directory(repo.path()).join("test-status");
        let is_clean = get_worktree_status(&worktree_path).await.unwrap();
        assert!(is_clean);
    }

    #[tokio::test]
    async fn test_get_worktree_status_dirty() {
        let repo = TestRepo::new().await.unwrap();
        repo.create_file_and_commit("test.txt", "content", "Initial commit").await.unwrap();

        // Create a worktree
        let options = CreateWorktreeOptions::default();
        create_worktree(repo.path(), "test-dirty", options).await.unwrap();

        let worktree_path = get_phantom_directory(repo.path()).join("test-dirty");

        // Make the worktree dirty by modifying a file
        tokio::fs::write(worktree_path.join("test.txt"), "modified content").await.unwrap();

        let is_clean = get_worktree_status(&worktree_path).await.unwrap();
        assert!(!is_clean);
    }
}
