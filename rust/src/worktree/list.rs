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

    #[tokio::test]
    async fn test_get_worktree_info() {
        let repo = TestRepo::new().await.unwrap();
        repo.create_file_and_commit("test.txt", "content", "Initial commit").await.unwrap();

        // Create a worktree
        let options =
            CreateWorktreeOptions { branch: Some("info-branch".to_string()), ..Default::default() };
        create_worktree(repo.path(), "test-info", options).await.unwrap();

        let info = get_worktree_info(repo.path(), "test-info").await.unwrap();
        assert_eq!(info.name, "test-info");
        assert!(info.path.contains("test-info"));
        assert_eq!(info.branch, Some("info-branch".to_string()));
        assert!(info.is_clean);
    }

    #[tokio::test]
    async fn test_get_worktree_branch_detached_head() {
        let repo = TestRepo::new().await.unwrap();
        repo.create_file_and_commit("test.txt", "content", "Initial commit").await.unwrap();

        // Get the current commit hash using Command directly
        let output = tokio::process::Command::new("git")
            .args(&["rev-parse", "HEAD"])
            .current_dir(repo.path())
            .output()
            .await
            .unwrap();
        let commit_hash = String::from_utf8_lossy(&output.stdout).trim().to_string();

        // Create a worktree at a specific commit (detached HEAD)
        let worktree_path = get_phantom_directory(repo.path()).join("detached");
        tokio::process::Command::new("git")
            .args(&["worktree", "add", "-d", worktree_path.to_str().unwrap(), &commit_hash])
            .current_dir(repo.path())
            .output()
            .await
            .unwrap();

        let branch = get_worktree_branch(&worktree_path).await.unwrap();
        assert_eq!(branch, "(detached HEAD)");
    }

    #[tokio::test]
    async fn test_get_worktree_branch_nonexistent_path() {
        let temp_dir = tempfile::tempdir().unwrap();
        let nonexistent_path = temp_dir.path().join("nonexistent");

        let branch = get_worktree_branch(&nonexistent_path).await.unwrap();
        assert_eq!(branch, "unknown");
    }

    #[tokio::test]
    async fn test_get_worktree_status_nonexistent_path() {
        let temp_dir = tempfile::tempdir().unwrap();
        let nonexistent_path = temp_dir.path().join("nonexistent");

        let is_clean = get_worktree_status(&nonexistent_path).await.unwrap();
        assert!(is_clean); // Defaults to clean on error
    }

    #[tokio::test]
    async fn test_worktree_info_serialization() {
        let info = WorktreeInfo {
            name: "test".to_string(),
            path: "/path/to/test".to_string(),
            branch: Some("main".to_string()),
            is_clean: true,
        };

        // Test JSON serialization
        let json = serde_json::to_string(&info).unwrap();
        assert!(json.contains("\"name\":\"test\""));
        assert!(json.contains("\"is_clean\":true"));

        // Test deserialization
        let deserialized: WorktreeInfo = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.name, info.name);
        assert_eq!(deserialized.path, info.path);
        assert_eq!(deserialized.branch, info.branch);
        assert_eq!(deserialized.is_clean, info.is_clean);
    }

    #[tokio::test]
    async fn test_list_worktrees_success_serialization() {
        let success = ListWorktreesSuccess {
            worktrees: vec![
                WorktreeInfo {
                    name: "feature1".to_string(),
                    path: "/path/to/feature1".to_string(),
                    branch: Some("feature1".to_string()),
                    is_clean: true,
                },
                WorktreeInfo {
                    name: "feature2".to_string(),
                    path: "/path/to/feature2".to_string(),
                    branch: Some("feature2".to_string()),
                    is_clean: false,
                },
            ],
            message: None,
        };

        let json = serde_json::to_string(&success).unwrap();
        let deserialized: ListWorktreesSuccess = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.worktrees.len(), 2);
        assert_eq!(deserialized.worktrees[0].name, "feature1");
        assert_eq!(deserialized.worktrees[1].name, "feature2");
        assert!(deserialized.message.is_none());
    }

    #[tokio::test]
    async fn test_list_worktrees_with_message() {
        let success = ListWorktreesSuccess {
            worktrees: vec![],
            message: Some("No worktrees found".to_string()),
        };

        let json = serde_json::to_string(&success).unwrap();
        assert!(json.contains("\"message\":\"No worktrees found\""));
    }
}
