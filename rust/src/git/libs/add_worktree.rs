use crate::core::command_executor::CommandExecutor;
use crate::git::git_executor_adapter::GitExecutor;
use crate::{PhantomError, Result};
use std::path::Path;
use std::sync::Arc;
use tracing::info;

/// Add a new git worktree with executor
pub async fn add_worktree_with_executor(
    executor: Arc<dyn CommandExecutor>,
    repo_path: &Path,
    worktree_path: &Path,
    branch: Option<&str>,
    new_branch: bool,
) -> Result<()> {
    let git_executor = GitExecutor::new(executor).with_cwd(repo_path);

    let mut args = vec!["worktree", "add"];

    // If creating a new branch
    if new_branch {
        if let Some(branch_name) = branch {
            args.push("-b");
            args.push(branch_name);
        } else {
            return Err(PhantomError::InvalidWorktreeName(
                "Branch name required when creating new branch".to_string(),
            ));
        }
    }

    // Add the worktree path
    let path_str = worktree_path.to_string_lossy();
    args.push(&path_str);

    // If checking out existing branch (not creating new)
    if !new_branch {
        if let Some(branch_name) = branch {
            args.push(branch_name);
        }
    }

    info!("Creating worktree at {:?} for branch {:?}", worktree_path, branch);
    git_executor.run(&args).await?;

    Ok(())
}

/// Add a new git worktree using the default executor
pub async fn add_worktree(
    repo_path: &Path,
    worktree_path: &Path,
    branch: Option<&str>,
    new_branch: bool,
) -> Result<()> {
    use crate::core::executors::RealCommandExecutor;
    add_worktree_with_executor(Arc::new(RealCommandExecutor), repo_path, worktree_path, branch, new_branch).await
}

/// Add a new worktree with automatic branch name
pub async fn add_worktree_auto(repo_path: &Path, worktree_name: &str) -> Result<()> {
    let worktree_path = repo_path
        .parent()
        .ok_or_else(|| {
            PhantomError::InvalidWorktreeName("Cannot determine parent directory".to_string())
        })?
        .join(worktree_name);

    add_worktree(repo_path, &worktree_path, Some(worktree_name), true).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::TestRepo;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_add_worktree_new_branch() {
        let repo = TestRepo::new().await.unwrap();
        repo.create_file_and_commit("test.txt", "content", "Initial commit").await.unwrap();

        let temp_dir = tempdir().unwrap();
        let worktree_path = temp_dir.path().join("feature-branch");

        add_worktree(repo.path(), &worktree_path, Some("feature-branch"), true).await.unwrap();

        assert!(worktree_path.exists());
        assert!(worktree_path.join(".git").exists());
        assert!(worktree_path.join("test.txt").exists());
    }

    #[tokio::test]
    async fn test_add_worktree_existing_branch() {
        let repo = TestRepo::new().await.unwrap();
        repo.create_file_and_commit("test.txt", "content", "Initial commit").await.unwrap();
        repo.create_branch("existing-branch").await.unwrap();

        // Switch back to main branch to allow worktree creation
        use crate::core::executors::RealCommandExecutor;
        let executor = GitExecutor::new(Arc::new(RealCommandExecutor)).with_cwd(repo.path());
        executor.run(&["checkout", "main"]).await.ok();

        let temp_dir = tempdir().unwrap();
        let worktree_path = temp_dir.path().join("existing-worktree");

        add_worktree(repo.path(), &worktree_path, Some("existing-branch"), false).await.unwrap();

        assert!(worktree_path.exists());
    }

    #[tokio::test]
    async fn test_add_worktree_auto() {
        let repo = TestRepo::new().await.unwrap();
        repo.create_file_and_commit("test.txt", "content", "Initial commit").await.unwrap();

        // Use a unique name to avoid conflicts in parallel tests
        let unique_name = format!("auto-branch-{}", std::process::id());
        add_worktree_auto(repo.path(), &unique_name).await.unwrap();

        let expected_path = repo.path().parent().unwrap().join(&unique_name);
        assert!(expected_path.exists());
    }

    #[tokio::test]
    async fn test_add_worktree_missing_branch_name() {
        let repo = TestRepo::new().await.unwrap();
        let temp_dir = tempdir().unwrap();
        let worktree_path = temp_dir.path().join("test");

        let result = add_worktree(repo.path(), &worktree_path, None, true).await;
        assert!(result.is_err());

        match result.unwrap_err() {
            PhantomError::InvalidWorktreeName(msg) => {
                assert!(msg.contains("Branch name required"));
            }
            _ => panic!("Expected InvalidWorktreeName error"),
        }
    }
}
