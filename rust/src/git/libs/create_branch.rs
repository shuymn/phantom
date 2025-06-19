use crate::core::command_executor::CommandExecutor;
use crate::git::git_executor_adapter;
use crate::Result;
use std::path::Path;
use std::sync::Arc;
use tracing::debug;

/// Create a new branch in the repository using a provided executor
pub async fn create_branch_with_executor(
    executor: Arc<dyn CommandExecutor>,
    git_root: &Path,
    branch_name: &str,
) -> Result<()> {
    let git_executor = git_executor_adapter::GitExecutor::new(executor).with_cwd(git_root);

    debug!("Creating branch '{}' in {:?}", branch_name, git_root);

    // Create the new branch
    git_executor.run(&["branch", branch_name]).await?;

    debug!("Successfully created branch '{}'", branch_name);
    Ok(())
}

/// Create a new branch in the repository
pub async fn create_branch(git_root: &Path, branch_name: &str) -> Result<()> {
    use crate::core::executors::RealCommandExecutor;
    create_branch_with_executor(Arc::new(RealCommandExecutor), git_root, branch_name).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::executors::MockCommandExecutor;
    use crate::test_utils::TestRepo;
    use crate::PhantomError;

    #[tokio::test]
    async fn test_create_branch_success() {
        let repo = TestRepo::new().await.unwrap();
        repo.create_file_and_commit("test.txt", "content", "Initial commit").await.unwrap();

        // Create a new branch
        create_branch(repo.path(), "feature-branch").await.unwrap();

        // Verify the branch was created using branch_exists
        use crate::git::libs::branch_exists::branch_exists;
        let exists = branch_exists(repo.path(), "feature-branch").await.unwrap();
        assert!(exists);
    }

    #[tokio::test]
    async fn test_create_branch_with_slashes() {
        let repo = TestRepo::new().await.unwrap();
        repo.create_file_and_commit("test.txt", "content", "Initial commit").await.unwrap();

        // Create a branch with slashes
        create_branch(repo.path(), "feature/new-feature").await.unwrap();

        // Verify the branch was created using branch_exists
        use crate::git::libs::branch_exists::branch_exists;
        let exists = branch_exists(repo.path(), "feature/new-feature").await.unwrap();
        assert!(exists);
    }

    #[tokio::test]
    async fn test_create_branch_duplicate_error() {
        let repo = TestRepo::new().await.unwrap();
        repo.create_file_and_commit("test.txt", "content", "Initial commit").await.unwrap();

        // Create a branch
        create_branch(repo.path(), "feature-branch").await.unwrap();

        // Try to create the same branch again
        let result = create_branch(repo.path(), "feature-branch").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_create_branch_with_mock_executor_success() {
        let mut mock = MockCommandExecutor::new();
        mock.expect_command("git")
            .with_args(&["branch", "new-feature"])
            .in_dir("/test/repo")
            .returns_success();

        let result =
            create_branch_with_executor(Arc::new(mock), Path::new("/test/repo"), "new-feature")
                .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_create_branch_with_mock_executor_duplicate() {
        let mut mock = MockCommandExecutor::new();
        mock.expect_command("git")
            .with_args(&["branch", "existing-branch"])
            .in_dir("/test/repo")
            .returns_output("", "fatal: A branch named 'existing-branch' already exists.", 128);

        let result =
            create_branch_with_executor(Arc::new(mock), Path::new("/test/repo"), "existing-branch")
                .await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), PhantomError::Git { exit_code: 128, .. }));
    }

    #[tokio::test]
    async fn test_create_branch_with_mock_executor_invalid_name() {
        let mut mock = MockCommandExecutor::new();
        mock.expect_command("git")
            .with_args(&["branch", "refs/heads/invalid"])
            .in_dir("/test/repo")
            .returns_output("", "fatal: 'refs/heads/invalid' is not a valid branch name.", 128);

        let result = create_branch_with_executor(
            Arc::new(mock),
            Path::new("/test/repo"),
            "refs/heads/invalid",
        )
        .await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), PhantomError::Git { exit_code: 128, .. }));
    }

    #[tokio::test]
    async fn test_create_branch_with_mock_executor_not_git_repo() {
        let mut mock = MockCommandExecutor::new();
        mock.expect_command("git")
            .with_args(&["branch", "feature"])
            .in_dir("/not/a/repo")
            .returns_output("", "fatal: not a git repository", 128);

        let result =
            create_branch_with_executor(Arc::new(mock), Path::new("/not/a/repo"), "feature").await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), PhantomError::Git { exit_code: 128, .. }));
    }
}
