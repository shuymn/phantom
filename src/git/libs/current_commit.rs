use crate::core::command_executor::CommandExecutor;
use crate::git::git_executor_adapter::GitExecutor as GitExecutorAdapter;
use crate::Result;
use std::path::Path;
use tracing::debug;

/// Get the current commit SHA with executor
///
/// This function enables mock testing by accepting a CommandExecutor.
/// Use this in handlers that have access to CommandExecutor from HandlerContext.
///
/// # Example
/// ```no_run
/// use phantom::git::libs::current_commit::current_commit;
/// use phantom::cli::context::ProductionContext;
/// use phantom::Result;
/// use std::path::Path;
///
/// async fn handle_something(context: ProductionContext) -> Result<()> {
///     let commit = current_commit(
///         context.executor,
///         Path::new("/repo/path")
///     ).await?;
///     println!("Current commit: {}", commit);
///     Ok(())
/// }
/// ```
pub async fn current_commit<E>(executor: E, repo_path: &Path) -> Result<String>
where
    E: CommandExecutor + Clone + 'static,
{
    let git_executor = GitExecutorAdapter::new(executor).with_cwd(repo_path);

    debug!("Getting current commit in {:?}", repo_path);
    let output = git_executor.run(&["rev-parse", "HEAD"]).await?;

    let commit = output.trim().to_string();
    debug!("Current commit: {}", commit);

    Ok(commit)
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::git::git_executor_adapter::GitExecutor;
    use crate::test_utils::TestRepo;

    #[tokio::test]
    async fn test_current_commit_initial() {
        let repo = TestRepo::new().await.unwrap();
        repo.create_file_and_commit("test.txt", "content", "Initial commit").await.unwrap();

        use crate::core::executors::RealCommandExecutor;
        let commit = current_commit(RealCommandExecutor, repo.path()).await.unwrap();
        // SHA-1 hash should be 40 characters
        assert_eq!(commit.len(), 40);
        // Should be all hex characters
        assert!(commit.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[tokio::test]
    async fn test_current_commit_after_multiple_commits() {
        let repo = TestRepo::new().await.unwrap();
        repo.create_file_and_commit("test.txt", "content", "Initial commit").await.unwrap();

        // Get first commit
        use crate::core::executors::RealCommandExecutor;
        let first_commit = current_commit(RealCommandExecutor, repo.path()).await.unwrap();

        // Make another commit
        repo.create_file_and_commit("test2.txt", "content2", "Second commit").await.unwrap();

        // Get second commit
        let second_commit = current_commit(RealCommandExecutor, repo.path()).await.unwrap();

        // Should be different commits
        assert_ne!(first_commit, second_commit);
        assert_eq!(second_commit.len(), 40);
    }

    #[tokio::test]
    async fn test_current_commit_same_across_branches() {
        let repo = TestRepo::new().await.unwrap();
        repo.create_file_and_commit("test.txt", "content", "Initial commit").await.unwrap();

        // Get commit on main branch
        use crate::core::executors::RealCommandExecutor;
        let main_commit = current_commit(RealCommandExecutor, repo.path()).await.unwrap();

        // Create and checkout new branch (without new commits)
        repo.create_branch("feature-branch").await.unwrap();
        let executor = GitExecutor::new(RealCommandExecutor::new()).with_cwd(repo.path());
        executor.run(&["checkout", "feature-branch"]).await.unwrap();

        // Should be same commit
        let feature_commit = current_commit(RealCommandExecutor, repo.path()).await.unwrap();
        assert_eq!(main_commit, feature_commit);
    }

    #[tokio::test]
    async fn test_current_commit_detached_head() {
        let repo = TestRepo::new().await.unwrap();
        repo.create_file_and_commit("test.txt", "content", "Initial commit").await.unwrap();

        // Get the current commit
        use crate::core::executors::RealCommandExecutor;
        let commit_sha = current_commit(RealCommandExecutor, repo.path()).await.unwrap();

        // Checkout the commit directly (detached HEAD)
        let executor = GitExecutor::new(RealCommandExecutor::new()).with_cwd(repo.path());
        executor.run(&["checkout", &commit_sha]).await.unwrap();

        // Should still return the same commit
        let detached_commit = current_commit(RealCommandExecutor, repo.path()).await.unwrap();
        assert_eq!(commit_sha, detached_commit);
    }

    #[tokio::test]
    async fn test_current_commit_with_mock_executor() {
        use crate::core::executors::MockCommandExecutor;

        let mut mock = MockCommandExecutor::new();

        // Mock the rev-parse HEAD command
        mock.expect_command("git")
            .with_args(&["rev-parse", "HEAD"])
            .in_dir("/test")
            .returns_output("abc123def456789012345678901234567890abcd", "", 0);

        let commit = current_commit(mock, Path::new("/test")).await.unwrap();
        assert_eq!(commit, "abc123def456789012345678901234567890abcd");
    }

    #[tokio::test]
    async fn test_current_commit_error_with_mock() {
        use crate::core::executors::MockCommandExecutor;

        let mut mock = MockCommandExecutor::new();

        // Mock error case - no commits yet
        mock.expect_command("git")
            .with_args(&["rev-parse", "HEAD"])
            .in_dir("/test")
            .returns_output(
            "",
            "fatal: ambiguous argument 'HEAD': unknown revision or path not in the working tree.",
            128,
        );

        let result = current_commit(mock, Path::new("/test")).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_current_commit_short_sha_with_mock() {
        use crate::core::executors::MockCommandExecutor;

        let mut mock = MockCommandExecutor::new();

        // Some systems might return short SHA
        mock.expect_command("git")
            .with_args(&["rev-parse", "HEAD"])
            .in_dir("/test")
            .returns_output("abc123d", "", 0);

        let commit = current_commit(mock, Path::new("/test")).await.unwrap();
        assert_eq!(commit, "abc123d");
    }
}
