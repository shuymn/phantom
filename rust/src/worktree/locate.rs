use crate::worktree::validate::validate_worktree_exists;
use crate::Result;
use std::path::Path;

/// Result of where worktree operation
pub struct WhereWorktreeSuccess {
    pub path: String,
}

/// Get the path of a worktree
pub async fn where_worktree(git_root: &Path, name: &str) -> Result<WhereWorktreeSuccess> {
    let validation = validate_worktree_exists(git_root, name).await?;

    Ok(WhereWorktreeSuccess { path: validation.path.to_string_lossy().to_string() })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::TestRepo;
    use crate::worktree::create::create_worktree;
    use crate::worktree::types::CreateWorktreeOptions;

    #[tokio::test]
    async fn test_where_worktree_exists() {
        let repo = TestRepo::new().await.unwrap();
        repo.create_file_and_commit("test.txt", "content", "Initial commit").await.unwrap();

        // Create a worktree
        let options = CreateWorktreeOptions::default();
        create_worktree(repo.path(), "test-worktree", options).await.unwrap();

        // Get the path
        let result = where_worktree(repo.path(), "test-worktree").await.unwrap();
        assert!(result.path.contains("test-worktree"));
        assert!(result.path.contains(".git/phantom/worktrees"));
    }

    #[tokio::test]
    async fn test_where_worktree_not_found() {
        let repo = TestRepo::new().await.unwrap();
        repo.create_file_and_commit("test.txt", "content", "Initial commit").await.unwrap();

        // Try to get path of non-existent worktree
        let result = where_worktree(repo.path(), "non-existent").await;
        assert!(result.is_err());
    }
}
