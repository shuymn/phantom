use crate::core::command_executor::CommandExecutor;
use crate::git::libs::attach_worktree::attach_worktree as git_attach_worktree;
use crate::worktree::paths::get_worktree_path;
use crate::worktree::validate::validate_worktree_name;
use crate::{PhantomError, Result};
use std::path::Path;
use tokio::fs;
use tracing::info;

/// Attach a worktree to an existing branch with executor
pub async fn attach_worktree<E>(executor: E, git_root: &Path, branch_name: &str) -> Result<()>
where
    E: CommandExecutor + Clone + 'static,
{
    // Validate the branch name
    validate_worktree_name(branch_name)?;

    let worktree_path = get_worktree_path(git_root, branch_name);

    // Check if worktree already exists
    if fs::metadata(&worktree_path).await.is_ok() {
        return Err(PhantomError::WorktreeExists { name: branch_name.to_string() });
    }

    // Create phantom directory if it doesn't exist
    let phantom_dir = worktree_path.parent().ok_or_else(|| PhantomError::InvalidPath {
        path: worktree_path.to_string_lossy().to_string(),
        reason: "Invalid worktree path - no parent directory".to_string(),
    })?;

    if fs::metadata(&phantom_dir).await.is_err() {
        fs::create_dir_all(&phantom_dir).await.map_err(|e| {
            PhantomError::Io(std::io::Error::other(format!(
                "Failed to create phantom directory: {e}"
            )))
        })?;
    }

    // Attach the worktree using the git backend
    info!("Attaching worktree '{}' at {:?}", branch_name, worktree_path);
    git_attach_worktree(executor, git_root, &worktree_path, branch_name).await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::git::git_executor_adapter::GitExecutor;
    use crate::test_utils::TestRepo;

    #[tokio::test]
    async fn test_attach_worktree_success() {
        let repo = TestRepo::new().await.unwrap();
        repo.create_file_and_commit("test.txt", "content", "Initial commit").await.unwrap();

        // Create a branch
        repo.create_branch("existing-branch").await.unwrap();

        // Switch back to main
        let executor = GitExecutor::new(RealCommandExecutor::new()).with_cwd(repo.path());
        executor.run(&["checkout", "main"]).await.unwrap();

        // Attach worktree
        use crate::core::executors::RealCommandExecutor;
        attach_worktree(RealCommandExecutor, repo.path(), "existing-branch").await.unwrap();

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
        let executor = GitExecutor::new(RealCommandExecutor::new()).with_cwd(repo.path());
        executor.run(&["checkout", "main"]).await.unwrap();

        // First attach should succeed
        use crate::core::executors::RealCommandExecutor;
        attach_worktree(RealCommandExecutor, repo.path(), "existing-branch").await.unwrap();

        // Second attach should fail
        let result = attach_worktree(RealCommandExecutor, repo.path(), "existing-branch").await;
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

        use crate::core::executors::RealCommandExecutor;
        let result = attach_worktree(RealCommandExecutor, repo.path(), "").await;
        assert!(result.is_err());
        match result.unwrap_err() {
            PhantomError::InvalidWorktreeName { .. } | PhantomError::ValidationFailed { .. } => {}
            e => panic!("Expected InvalidWorktreeName or Validation error, got: {e:?}"),
        }
    }
}
