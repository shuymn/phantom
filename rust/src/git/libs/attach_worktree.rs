use crate::core::command_executor::CommandExecutor;
use crate::git::git_executor_adapter::GitExecutor;
use crate::Result;
use std::path::Path;
use std::sync::Arc;
use tracing::info;

/// Attach a worktree to an existing branch with executor
pub async fn attach_worktree_with_executor(
    executor: Arc<dyn CommandExecutor>,
    git_root: &Path,
    worktree_path: &Path,
    branch_name: &str,
) -> Result<()> {
    let git_executor = GitExecutor::new(executor).with_cwd(git_root);

    info!("Attaching worktree at {:?} to branch '{}'", worktree_path, branch_name);

    let worktree_path_str = worktree_path.to_string_lossy();
    git_executor.run(&["worktree", "add", &worktree_path_str, branch_name]).await?;

    Ok(())
}

/// Attach a worktree to an existing branch using the default executor
pub async fn attach_worktree(
    git_root: &Path,
    worktree_path: &Path,
    branch_name: &str,
) -> Result<()> {
    use crate::core::executors::RealCommandExecutor;
    attach_worktree_with_executor(
        Arc::new(RealCommandExecutor),
        git_root,
        worktree_path,
        branch_name,
    )
    .await
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::executors::RealCommandExecutor;
    use crate::test_utils::TestRepo;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_attach_worktree_existing_branch() {
        let repo = TestRepo::new().await.unwrap();
        repo.create_file_and_commit("test.txt", "content", "Initial commit").await.unwrap();

        // Create a branch
        repo.create_branch("existing-branch").await.unwrap();

        // Switch back to main to allow worktree creation
        let executor = GitExecutor::new(Arc::new(RealCommandExecutor::new())).with_cwd(repo.path());
        executor.run(&["checkout", "main"]).await.unwrap();

        // Attach worktree to existing branch
        let temp_dir = tempdir().unwrap();
        let worktree_path = temp_dir.path().join("my-worktree");

        attach_worktree(repo.path(), &worktree_path, "existing-branch").await.unwrap();

        assert!(worktree_path.exists());
        assert!(worktree_path.join(".git").exists());
        assert!(worktree_path.join("test.txt").exists());

        // Verify the worktree is on the correct branch
        let executor_wt =
            GitExecutor::new(Arc::new(RealCommandExecutor::new())).with_cwd(&worktree_path);
        let branch = executor_wt.run(&["branch", "--show-current"]).await.unwrap();
        assert_eq!(branch.trim(), "existing-branch");
    }

    #[tokio::test]
    async fn test_attach_worktree_nonexistent_branch() {
        let repo = TestRepo::new().await.unwrap();
        repo.create_file_and_commit("test.txt", "content", "Initial commit").await.unwrap();

        let temp_dir = tempdir().unwrap();
        let worktree_path = temp_dir.path().join("my-worktree");

        // Should fail when trying to attach to non-existent branch
        let result = attach_worktree(repo.path(), &worktree_path, "nonexistent-branch").await;
        assert!(result.is_err());
        assert!(!worktree_path.exists());
    }

    #[tokio::test]
    async fn test_attach_worktree_already_checked_out() {
        let repo = TestRepo::new().await.unwrap();
        repo.create_file_and_commit("test.txt", "content", "Initial commit").await.unwrap();

        let temp_dir = tempdir().unwrap();
        let worktree_path = temp_dir.path().join("my-worktree");

        // Should fail when trying to attach to currently checked out branch
        let result = attach_worktree(repo.path(), &worktree_path, "main").await;
        assert!(result.is_err());

        // Error should mention branch is already in use
        let err_msg = result.unwrap_err().to_string();
        // Git error messages can vary by version, check for common patterns
        assert!(
            err_msg.contains("already checked out")
                || err_msg.contains("is already checked out")
                || err_msg.contains("is checked out")
                || err_msg.contains("is already used by worktree")
        );
    }

    #[tokio::test]
    async fn test_attach_worktree_path_already_exists() {
        let repo = TestRepo::new().await.unwrap();
        repo.create_file_and_commit("test.txt", "content", "Initial commit").await.unwrap();

        // Create a branch
        repo.create_branch("test-branch").await.unwrap();

        let executor = GitExecutor::new(Arc::new(RealCommandExecutor::new())).with_cwd(repo.path());
        executor.run(&["checkout", "main"]).await.unwrap();

        // Create a directory that already exists
        let temp_dir = tempdir().unwrap();
        let worktree_path = temp_dir.path().join("existing-dir");
        std::fs::create_dir(&worktree_path).unwrap();

        // Should fail when path already exists
        let result = attach_worktree(repo.path(), &worktree_path, "test-branch").await;

        // On some systems, git might create the worktree in the existing directory
        // Let's check if it's an error, and if so, check the message
        if result.is_err() {
            let err_msg = result.unwrap_err().to_string();
            assert!(err_msg.contains("already exists") || err_msg.contains("not empty"));
        } else {
            // If it succeeded, the directory should now be a worktree
            assert!(worktree_path.join(".git").exists());
        }
    }
}
