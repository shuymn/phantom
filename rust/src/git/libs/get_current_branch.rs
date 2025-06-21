use crate::core::command_executor::CommandExecutor;
use crate::git::git_executor_adapter::GitExecutor as GitExecutorAdapter;
use crate::Result;
use std::path::Path;
use std::sync::Arc;
use tracing::debug;

/// Get the current branch name with executor
///
/// This function enables mock testing by accepting a CommandExecutor.
/// Use this in handlers that have access to CommandExecutor from HandlerContext.
///
/// # Example
/// ```no_run
/// use phantom::git::libs::get_current_branch::get_current_branch_with_executor;
/// use phantom::cli::context::ProductionContext;
/// use phantom::Result;
/// use std::path::Path;
/// use std::sync::Arc;
///
/// async fn handle_something(context: ProductionContext) -> Result<()> {
///     let branch = get_current_branch_with_executor(
///         Arc::new(context.executor),
///         Path::new("/repo/path")
///     ).await?;
///     println!("Current branch: {}", branch);
///     Ok(())
/// }
/// ```
pub async fn get_current_branch_with_executor(
    executor: Arc<dyn CommandExecutor>,
    repo_path: &Path,
) -> Result<String> {
    let git_executor = GitExecutorAdapter::new(executor).with_cwd(repo_path);

    debug!("Getting current branch in {:?}", repo_path);
    let output = git_executor.run(&["branch", "--show-current"]).await?;

    let branch = output.trim().to_string();
    debug!("Current branch: {}", branch);

    Ok(branch)
}

/// Get the current branch name using the default executor
pub async fn get_current_branch(repo_path: &Path) -> Result<String> {
    use crate::core::executors::RealCommandExecutor;
    get_current_branch_with_executor(Arc::new(RealCommandExecutor), repo_path).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::git::executor::GitExecutor;
    use crate::test_utils::TestRepo;

    #[tokio::test]
    async fn test_get_current_branch_main() {
        let repo = TestRepo::new().await.unwrap();
        repo.create_file_and_commit("test.txt", "content", "Initial commit").await.unwrap();

        let branch = get_current_branch(repo.path()).await.unwrap();
        assert_eq!(branch, "main");
    }

    #[tokio::test]
    async fn test_get_current_branch_after_checkout() {
        let repo = TestRepo::new().await.unwrap();
        repo.create_file_and_commit("test.txt", "content", "Initial commit").await.unwrap();

        // Create and checkout a new branch
        repo.create_branch("feature-branch").await.unwrap();
        let executor = GitExecutor::with_cwd(repo.path());
        executor.run(&["checkout", "feature-branch"]).await.unwrap();

        let branch = get_current_branch(repo.path()).await.unwrap();
        assert_eq!(branch, "feature-branch");
    }

    #[tokio::test]
    async fn test_get_current_branch_detached_head() {
        let repo = TestRepo::new().await.unwrap();
        repo.create_file_and_commit("test.txt", "content", "Initial commit").await.unwrap();

        // Get the current commit
        let executor = GitExecutor::with_cwd(repo.path());
        let commit = executor.run(&["rev-parse", "HEAD"]).await.unwrap();
        let commit = commit.trim();

        // Checkout the commit directly (detached HEAD)
        executor.run(&["checkout", commit]).await.unwrap();

        // In detached HEAD state, --show-current returns empty
        let branch = get_current_branch(repo.path()).await.unwrap();
        assert_eq!(branch, "");
    }

    #[tokio::test]
    async fn test_get_current_branch_with_dashes() {
        let repo = TestRepo::new().await.unwrap();
        repo.create_file_and_commit("test.txt", "content", "Initial commit").await.unwrap();

        // Create and checkout a branch with dashes
        repo.create_branch("feature-with-dashes").await.unwrap();
        let executor = GitExecutor::with_cwd(repo.path());
        executor.run(&["checkout", "feature-with-dashes"]).await.unwrap();

        let branch = get_current_branch(repo.path()).await.unwrap();
        assert_eq!(branch, "feature-with-dashes");
    }

    #[tokio::test]
    async fn test_get_current_branch_with_slashes() {
        let repo = TestRepo::new().await.unwrap();
        repo.create_file_and_commit("test.txt", "content", "Initial commit").await.unwrap();

        // Create and checkout a branch with slashes
        let executor = GitExecutor::with_cwd(repo.path());
        executor.run(&["checkout", "-b", "feature/new-feature"]).await.unwrap();

        let branch = get_current_branch(repo.path()).await.unwrap();
        assert_eq!(branch, "feature/new-feature");
    }

    #[tokio::test]
    async fn test_get_current_branch_with_mock_executor() {
        use crate::core::executors::MockCommandExecutor;

        let mut mock = MockCommandExecutor::new();

        // Mock the branch --show-current command
        mock.expect_command("git")
            .with_args(&["branch", "--show-current"])
            .in_dir("/test")
            .returns_output("feature-branch", "", 0);

        let branch =
            get_current_branch_with_executor(Arc::new(mock), Path::new("/test")).await.unwrap();
        assert_eq!(branch, "feature-branch");
    }

    #[tokio::test]
    async fn test_get_current_branch_detached_with_mock() {
        use crate::core::executors::MockCommandExecutor;

        let mut mock = MockCommandExecutor::new();

        // Mock detached HEAD state (empty output)
        mock.expect_command("git")
            .with_args(&["branch", "--show-current"])
            .in_dir("/test")
            .returns_output("", "", 0);

        let branch =
            get_current_branch_with_executor(Arc::new(mock), Path::new("/test")).await.unwrap();
        assert_eq!(branch, "");
    }
}
