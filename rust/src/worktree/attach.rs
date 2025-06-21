use crate::core::command_executor::CommandExecutor;
use crate::git::libs::attach_worktree::attach_worktree_with_executor as git_attach_worktree_with_executor;
use crate::worktree::paths::get_worktree_path;
use crate::worktree::validate::validate_worktree_name;
use crate::{PhantomError, Result};
use std::path::Path;
use std::sync::Arc;
use tokio::fs;
use tracing::info;

/// Attach a worktree to an existing branch with executor
pub async fn attach_worktree_with_executor(
    executor: Arc<dyn CommandExecutor>,
    git_root: &Path,
    branch_name: &str,
) -> Result<()> {
    // Validate the branch name
    validate_worktree_name(branch_name)?;

    let worktree_path = get_worktree_path(git_root, branch_name);

    // Check if worktree already exists
    if fs::metadata(&worktree_path).await.is_ok() {
        return Err(PhantomError::WorktreeExists { name: branch_name.to_string() });
    }

    // Create phantom directory if it doesn't exist
    let phantom_dir = worktree_path
        .parent()
        .ok_or_else(|| PhantomError::InvalidWorktreeName("Invalid worktree path".to_string()))?;

    if fs::metadata(&phantom_dir).await.is_err() {
        fs::create_dir_all(&phantom_dir).await.map_err(|e| {
            PhantomError::Io(std::io::Error::other(format!(
                "Failed to create phantom directory: {}",
                e
            )))
        })?;
    }

    // Attach the worktree using the git backend
    info!("Attaching worktree '{}' at {:?}", branch_name, worktree_path);
    git_attach_worktree_with_executor(executor, git_root, &worktree_path, branch_name).await?;

    Ok(())
}

/// Attach a worktree to an existing branch using the default executor
pub async fn attach_worktree(git_root: &Path, branch_name: &str) -> Result<()> {
    use crate::core::executors::RealCommandExecutor;
    attach_worktree_with_executor(Arc::new(RealCommandExecutor), git_root, branch_name).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::executors::RealCommandExecutor;
    use crate::git::git_executor_adapter::GitExecutor;
    use crate::test_utils::TestRepo;

    #[tokio::test]
    async fn test_attach_worktree_success() {
        let repo = TestRepo::new().await.unwrap();
        repo.create_file_and_commit("test.txt", "content", "Initial commit").await.unwrap();

        // Create a branch
        repo.create_branch("existing-branch").await.unwrap();

        // Switch back to main
        let executor = GitExecutor::new(Arc::new(RealCommandExecutor::new())).with_cwd(repo.path());
        executor.run(&["checkout", "main"]).await.unwrap();

        // Attach worktree
        attach_worktree(repo.path(), "existing-branch").await.unwrap();

        // Verify worktree was created
        let worktree_path = get_worktree_path(repo.path(), "existing-branch");
        assert!(worktree_path.exists());
        assert!(worktree_path.join(".git").exists());
    }

    #[tokio::test]
    async fn test_attach_worktree_already_exists() {
        let repo = TestRepo::new().await.unwrap();
        repo.create_file_and_commit("test.txt", "content", "Initial commit").await.unwrap();

        // Create a branch and worktree
        repo.create_branch("existing-branch").await.unwrap();
        let executor = GitExecutor::new(Arc::new(RealCommandExecutor::new())).with_cwd(repo.path());
        executor.run(&["checkout", "main"]).await.unwrap();

        // First attach should succeed
        attach_worktree(repo.path(), "existing-branch").await.unwrap();

        // Second attach should fail
        let result = attach_worktree(repo.path(), "existing-branch").await;
        assert!(result.is_err());
        match result.unwrap_err() {
            PhantomError::WorktreeExists { name } => {
                assert_eq!(name, "existing-branch");
            }
            _ => panic!("Expected WorktreeExists error"),
        }
    }

    #[tokio::test]
    async fn test_attach_worktree_invalid_name() {
        let repo = TestRepo::new().await.unwrap();

        let result = attach_worktree(repo.path(), "").await;
        assert!(result.is_err());
        match result.unwrap_err() {
            PhantomError::InvalidWorktreeName(_) | PhantomError::Validation(_) => {}
            e => panic!("Expected InvalidWorktreeName or Validation error, got: {:?}", e),
        }
    }
}
