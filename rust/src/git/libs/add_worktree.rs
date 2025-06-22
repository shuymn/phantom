use crate::core::command_executor::CommandExecutor;
use crate::git::const_utils::{commands, flags};
use crate::git::git_executor_adapter::GitExecutor;
use crate::{PhantomError, Result};
use std::path::Path;
use tracing::info;

/// Add a new git worktree with executor
pub async fn add_worktree<E>(
    executor: E,
    repo_path: &Path,
    worktree_path: &Path,
    branch: Option<&str>,
    new_branch: bool,
    commitish: Option<&str>,
) -> Result<()>
where
    E: CommandExecutor + Clone + 'static,
{
    let git_executor = GitExecutor::new(executor).with_cwd(repo_path);

    let mut args = vec![commands::WORKTREE, commands::ADD];

    // If creating a new branch
    if new_branch {
        if let Some(branch_name) = branch {
            args.push(flags::BRANCH_FLAG);
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
    } else {
        // When creating a new branch, add the commitish if provided
        if let Some(base) = commitish {
            args.push(base);
        }
    }

    info!(
        "Creating worktree at {:?} for branch {:?} from base {:?}",
        worktree_path, branch, commitish
    );
    git_executor.run(&args).await?;

    Ok(())
}

/// Add a new worktree with automatic branch name
pub async fn add_worktree_auto<E>(executor: E, repo_path: &Path, worktree_name: &str) -> Result<()>
where
    E: CommandExecutor + Clone + 'static,
{
    let worktree_path = repo_path
        .parent()
        .ok_or_else(|| {
            PhantomError::InvalidWorktreeName("Cannot determine parent directory".to_string())
        })?
        .join(worktree_name);

    add_worktree(executor, repo_path, &worktree_path, Some(worktree_name), true, None).await
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

        use crate::core::executors::RealCommandExecutor;
        add_worktree(
            RealCommandExecutor,
            repo.path(),
            &worktree_path,
            Some("feature-branch"),
            true,
            None,
        )
        .await
        .unwrap();

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
        let executor = GitExecutor::new(RealCommandExecutor).with_cwd(repo.path());
        executor.run(&["checkout", "main"]).await.ok();

        let temp_dir = tempdir().unwrap();
        let worktree_path = temp_dir.path().join("existing-worktree");

        add_worktree(
            RealCommandExecutor,
            repo.path(),
            &worktree_path,
            Some("existing-branch"),
            false,
            None,
        )
        .await
        .unwrap();

        assert!(worktree_path.exists());
    }

    #[tokio::test]
    async fn test_add_worktree_auto() {
        let repo = TestRepo::new().await.unwrap();
        repo.create_file_and_commit("test.txt", "content", "Initial commit").await.unwrap();

        // Use a unique name to avoid conflicts in parallel tests
        let unique_name = format!("auto-branch-{}", std::process::id());
        use crate::core::executors::RealCommandExecutor;
        add_worktree_auto(RealCommandExecutor, repo.path(), &unique_name).await.unwrap();

        let expected_path = repo.path().parent().unwrap().join(&unique_name);
        assert!(expected_path.exists());
    }

    #[tokio::test]
    async fn test_add_worktree_missing_branch_name() {
        let repo = TestRepo::new().await.unwrap();
        let temp_dir = tempdir().unwrap();
        let worktree_path = temp_dir.path().join("test");

        use crate::core::executors::RealCommandExecutor;
        let result =
            add_worktree(RealCommandExecutor, repo.path(), &worktree_path, None, true, None).await;
        assert!(result.is_err());

        match result.unwrap_err() {
            PhantomError::InvalidWorktreeName(msg) => {
                assert!(msg.contains("Branch name required"));
            }
            _ => panic!("Expected InvalidWorktreeName error"),
        }
    }

    #[tokio::test]
    async fn test_add_worktree_with_commitish() {
        let repo = TestRepo::new().await.unwrap();

        // Create initial commit
        repo.create_file_and_commit("file1.txt", "content1", "First commit").await.unwrap();

        // Get first commit hash
        use crate::core::executors::RealCommandExecutor;
        let executor = GitExecutor::new(RealCommandExecutor).with_cwd(repo.path());
        let first_commit = executor.run(&["rev-parse", "HEAD"]).await.unwrap();
        let first_commit = first_commit.trim();

        // Create second commit
        repo.create_file_and_commit("file2.txt", "content2", "Second commit").await.unwrap();

        // Create worktree based on first commit
        let temp_dir = tempdir().unwrap();
        let worktree_path = temp_dir.path().join("based-on-first");

        add_worktree(
            RealCommandExecutor,
            repo.path(),
            &worktree_path,
            Some("feature-from-first"),
            true,
            Some(first_commit),
        )
        .await
        .unwrap();

        // Verify worktree exists
        assert!(worktree_path.exists());

        // Verify it's at the first commit (file2.txt should not exist)
        assert!(worktree_path.join("file1.txt").exists());
        assert!(!worktree_path.join("file2.txt").exists());

        // Verify the commit hash
        let worktree_executor = GitExecutor::new(RealCommandExecutor).with_cwd(&worktree_path);
        let worktree_commit = worktree_executor.run(&["rev-parse", "HEAD"]).await.unwrap();
        assert_eq!(worktree_commit.trim(), first_commit);
    }
}
