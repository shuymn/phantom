use crate::core::command_executor::CommandExecutor;
use crate::git::git_executor_adapter::GitExecutor;
use crate::Result;
use std::path::Path;
use std::sync::Arc;
use tracing::debug;

/// Remove a worktree using a provided executor
pub async fn remove_worktree_with_executor(
    executor: Arc<dyn CommandExecutor>,
    cwd: &Path,
    worktree_path: &Path,
) -> Result<()> {
    let git_executor = GitExecutor::new(executor).with_cwd(cwd);

    debug!("Removing worktree at {:?}", worktree_path);
    git_executor.run(&["worktree", "remove", &worktree_path.to_string_lossy()]).await?;
    debug!("Worktree removed successfully");

    Ok(())
}

/// Remove a worktree using the default executor
pub async fn remove_worktree(cwd: &Path, worktree_path: &Path) -> Result<()> {
    use crate::core::executors::RealCommandExecutor;
    remove_worktree_with_executor(Arc::new(RealCommandExecutor), cwd, worktree_path).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::executors::MockCommandExecutor;

    #[tokio::test]
    async fn test_remove_worktree_with_mock_executor() {
        let mut mock = MockCommandExecutor::new();
        mock.expect_command("git")
            .with_args(&["worktree", "remove", "/test/repo/worktrees/feature"])
            .in_dir("/test/repo")
            .returns_output("", "", 0);

        let result = remove_worktree_with_executor(
            Arc::new(mock),
            Path::new("/test/repo"),
            Path::new("/test/repo/worktrees/feature"),
        )
        .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_remove_worktree_not_found_with_mock() {
        let mut mock = MockCommandExecutor::new();
        mock.expect_command("git")
            .with_args(&["worktree", "remove", "/test/repo/worktrees/nonexistent"])
            .in_dir("/test/repo")
            .returns_output(
                "",
                "fatal: '/test/repo/worktrees/nonexistent' is not a working tree\n",
                128,
            );

        let result = remove_worktree_with_executor(
            Arc::new(mock),
            Path::new("/test/repo"),
            Path::new("/test/repo/worktrees/nonexistent"),
        )
        .await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_remove_worktree_with_changes_with_mock() {
        let mut mock = MockCommandExecutor::new();
        mock.expect_command("git")
            .with_args(&["worktree", "remove", "/test/repo/worktrees/feature"])
            .in_dir("/test/repo")
            .returns_output(
                "",
                "fatal: '/test/repo/worktrees/feature' contains modified or untracked files, use --force to delete it\n",
                128
            );

        let result = remove_worktree_with_executor(
            Arc::new(mock),
            Path::new("/test/repo"),
            Path::new("/test/repo/worktrees/feature"),
        )
        .await;

        assert!(result.is_err());
    }
}
