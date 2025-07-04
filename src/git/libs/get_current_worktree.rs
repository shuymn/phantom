use crate::core::command_executor::CommandExecutor;
use crate::git::libs::list_worktrees::list_worktrees;
use crate::Result;
use std::path::Path;
use tracing::debug;

/// Get the current worktree branch name (returns None if in main worktree)
pub async fn get_current_worktree<E>(executor: E, git_root: &Path) -> Result<Option<String>>
where
    E: CommandExecutor + Clone + 'static,
{
    // Get the current working directory's git root
    let git_executor = crate::git::git_executor_adapter::GitExecutor::new(executor.clone());
    let current_path = git_executor.run(&["rev-parse", "--show-toplevel"]).await?;
    let current_path = current_path.trim();
    let current_path = Path::new(current_path);
    // Canonicalize the current path for consistent comparison
    let current_path_canonical = current_path.canonicalize().unwrap_or(current_path.to_path_buf());

    debug!("Current worktree path: {:?}", current_path_canonical);

    // Get all worktrees
    let worktrees = list_worktrees(executor, git_root).await?;

    // Find the current worktree by comparing canonical paths
    let current_worktree = worktrees.into_iter().find(|wt| {
        let wt_path = Path::new(&wt.path);
        let wt_canonical = wt_path.canonicalize().unwrap_or(wt_path.to_path_buf());
        wt_canonical == current_path_canonical
    });

    match current_worktree {
        Some(wt) => {
            // Use the already canonicalized current path
            let git_root_canonical = git_root.canonicalize().unwrap_or(git_root.to_path_buf());

            if current_path_canonical != git_root_canonical {
                debug!("Current worktree branch: {:?}", wt.branch);
                Ok(wt.branch)
            } else {
                debug!("In main worktree");
                Ok(None)
            }
        }
        None => {
            debug!("Worktree not found");
            Ok(None)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::executors::MockCommandExecutor;
    use crate::git::git_executor_adapter::GitExecutor;
    use crate::test_utils::TestRepo;
    use serial_test::serial;
    use std::env;

    #[tokio::test]
    async fn test_get_current_worktree_with_executor_main_worktree() {
        let mut mock = MockCommandExecutor::new();

        // Mock rev-parse --show-toplevel (returns main worktree path)
        mock.expect_command("git")
            .with_args(&["rev-parse", "--show-toplevel"])
            .returns_output("/repo", "", 0);

        // Mock worktree list for list_worktrees_with_executor
        mock.expect_command("git").with_args(&["worktree", "list", "--porcelain"]).returns_output(
            "worktree /repo\nHEAD abc123\nbranch refs/heads/main\n",
            "",
            0,
        );

        let result = get_current_worktree(mock, Path::new("/repo")).await.unwrap();

        assert_eq!(result, None); // Main worktree returns None
    }

    #[tokio::test]
    async fn test_get_current_worktree_with_executor_in_feature_worktree() {
        let mut mock = MockCommandExecutor::new();

        // Mock rev-parse --show-toplevel (returns feature worktree path)
        mock.expect_command("git").with_args(&["rev-parse", "--show-toplevel"]).returns_output(
            "/repo-feature",
            "",
            0,
        );

        // Mock worktree list for list_worktrees_with_executor
        mock.expect_command("git").with_args(&["worktree", "list", "--porcelain"]).returns_output(
            "worktree /repo\nHEAD abc123\nbranch refs/heads/main\n\n\
                 worktree /repo-feature\nHEAD def456\nbranch refs/heads/feature\n",
            "",
            0,
        );

        let result = get_current_worktree(mock, Path::new("/repo")).await.unwrap();

        assert_eq!(result, Some("feature".to_string()));
    }

    #[tokio::test]
    async fn test_get_current_worktree_with_executor_detached() {
        let mut mock = MockCommandExecutor::new();

        // Mock rev-parse --show-toplevel (returns detached worktree path)
        mock.expect_command("git").with_args(&["rev-parse", "--show-toplevel"]).returns_output(
            "/repo-detached",
            "",
            0,
        );

        // Mock worktree list for list_worktrees_with_executor
        mock.expect_command("git").with_args(&["worktree", "list", "--porcelain"]).returns_output(
            "worktree /repo\nHEAD abc123\nbranch refs/heads/main\n\n\
                 worktree /repo-detached\nHEAD def456\ndetached\n",
            "",
            0,
        );

        let result = get_current_worktree(mock, Path::new("/repo")).await.unwrap();

        assert_eq!(result, None); // Detached worktree has no branch
    }

    #[tokio::test]
    async fn test_get_current_worktree_with_executor_not_found() {
        let mut mock = MockCommandExecutor::new();

        // Mock rev-parse --show-toplevel (returns a path not in worktree list)
        mock.expect_command("git").with_args(&["rev-parse", "--show-toplevel"]).returns_output(
            "/some/other/path",
            "",
            0,
        );

        // Mock worktree list for list_worktrees_with_executor
        mock.expect_command("git").with_args(&["worktree", "list", "--porcelain"]).returns_output(
            "worktree /repo\nHEAD abc123\nbranch refs/heads/main\n",
            "",
            0,
        );

        let result = get_current_worktree(mock, Path::new("/repo")).await.unwrap();

        assert_eq!(result, None);
    }

    #[tokio::test]
    async fn test_get_current_worktree_main() {
        let repo = TestRepo::new().await.unwrap();
        repo.create_file_and_commit("test.txt", "content", "Initial commit").await.unwrap();

        // Change to the main repo directory
        let _guard = TestWorkingDir::new(repo.path());

        use crate::core::executors::RealCommandExecutor;
        let result = get_current_worktree(RealCommandExecutor, repo.path()).await.unwrap();
        assert_eq!(result, None);
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_get_current_worktree_in_worktree() {
        let repo = TestRepo::new().await.unwrap();
        repo.create_file_and_commit("test.txt", "content", "Initial commit").await.unwrap();

        // Create a worktree with unique name
        let executor = GitExecutor::new(RealCommandExecutor::new()).with_cwd(repo.path());
        use std::time::{SystemTime, UNIX_EPOCH};
        let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis();
        let unique_name = format!("test-worktree-{}-{}", std::process::id(), timestamp);
        let worktree_path = repo.path().parent().unwrap().join(&unique_name);
        executor
            .run(&["worktree", "add", "-b", "feature-branch", &worktree_path.to_string_lossy()])
            .await
            .unwrap();

        // Change to the worktree directory
        let _guard = TestWorkingDir::new(&worktree_path);

        use crate::core::executors::RealCommandExecutor;
        let result = get_current_worktree(RealCommandExecutor, repo.path()).await.unwrap();
        assert_eq!(result, Some("feature-branch".to_string()));
    }

    #[tokio::test]
    #[serial]
    async fn test_get_current_worktree_detached() {
        let repo = TestRepo::new().await.unwrap();
        repo.create_file_and_commit("test.txt", "content", "Initial commit").await.unwrap();

        // Get the current commit
        let executor = GitExecutor::new(RealCommandExecutor::new()).with_cwd(repo.path());
        let commit = executor.run(&["rev-parse", "HEAD"]).await.unwrap();
        let commit = commit.trim();

        // Create a detached worktree with unique name
        use std::time::{SystemTime, UNIX_EPOCH};
        let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis();
        let unique_name = format!("detached-worktree-{}-{}", std::process::id(), timestamp);
        let worktree_path = repo.path().parent().unwrap().join(&unique_name);
        executor
            .run(&["worktree", "add", "--detach", &worktree_path.to_string_lossy(), commit])
            .await
            .unwrap();

        // Save current directory before changing
        let original_dir = env::current_dir().unwrap();

        // Test from the main repository directory
        env::set_current_dir(repo.path()).unwrap();
        use crate::core::executors::RealCommandExecutor;
        let result = get_current_worktree(RealCommandExecutor, repo.path()).await.unwrap();
        assert_eq!(result, None);

        // Verify the worktree was created properly by checking from within it
        let worktree_executor =
            GitExecutor::new(RealCommandExecutor::new()).with_cwd(&worktree_path);
        let worktree_result = worktree_executor.run(&["branch", "--show-current"]).await.unwrap();
        assert_eq!(worktree_result.trim(), ""); // Detached HEAD has no branch name

        // Restore original directory
        env::set_current_dir(&original_dir).unwrap();
    }

    /// Helper struct to temporarily change working directory
    struct TestWorkingDir {
        original: std::path::PathBuf,
    }

    impl TestWorkingDir {
        fn new(path: &Path) -> Self {
            let original = env::current_dir().unwrap();
            env::set_current_dir(path).unwrap();
            Self { original }
        }
    }

    impl Drop for TestWorkingDir {
        fn drop(&mut self) {
            // Try to restore the original directory, but don't panic if it no longer exists
            // This can happen when testing with temporary directories that get cleaned up
            let _ = env::set_current_dir(&self.original);
        }
    }
}
