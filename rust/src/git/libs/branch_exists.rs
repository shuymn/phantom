use crate::core::command_executor::CommandExecutor;
use crate::git::git_executor_adapter;
use crate::{PhantomError, Result};
use std::path::Path;
use std::sync::Arc;
use tracing::debug;

/// Check if a branch exists in the repository using a provided executor
pub async fn branch_exists_with_executor(
    executor: Arc<dyn CommandExecutor>,
    git_root: &Path,
    branch_name: &str,
) -> Result<bool> {
    let git_executor = git_executor_adapter::GitExecutor::new(executor).with_cwd(git_root);

    debug!("Checking if branch '{}' exists in {:?}", branch_name, git_root);

    // Use show-ref to check if the branch exists
    let result = git_executor
        .run(&["show-ref", "--verify", "--quiet", &format!("refs/heads/{}", branch_name)])
        .await;

    match result {
        Ok(_) => {
            debug!("Branch '{}' exists", branch_name);
            Ok(true)
        }
        Err(PhantomError::Git { exit_code: 1, .. }) => {
            // Exit code 1 means the branch doesn't exist
            debug!("Branch '{}' does not exist", branch_name);
            Ok(false)
        }
        Err(e) => {
            // Any other error is a real error
            Err(e)
        }
    }
}

/// Check if a branch exists in the repository
pub async fn branch_exists(git_root: &Path, branch_name: &str) -> Result<bool> {
    use crate::core::executors::RealCommandExecutor;
    branch_exists_with_executor(Arc::new(RealCommandExecutor), git_root, branch_name).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::executors::MockCommandExecutor;
    use crate::test_utils::TestRepo;

    #[tokio::test]
    async fn test_branch_exists_main() {
        let repo = TestRepo::new().await.unwrap();
        repo.create_file_and_commit("test.txt", "content", "Initial commit").await.unwrap();

        let exists = branch_exists(repo.path(), "main").await.unwrap();
        assert!(exists);
    }

    #[tokio::test]
    async fn test_branch_exists_nonexistent() {
        let repo = TestRepo::new().await.unwrap();
        repo.create_file_and_commit("test.txt", "content", "Initial commit").await.unwrap();

        let exists = branch_exists(repo.path(), "nonexistent-branch").await.unwrap();
        assert!(!exists);
    }

    #[tokio::test]
    async fn test_branch_exists_created_branch() {
        let repo = TestRepo::new().await.unwrap();
        repo.create_file_and_commit("test.txt", "content", "Initial commit").await.unwrap();

        // Create a new branch
        repo.create_branch("feature-branch").await.unwrap();

        let exists = branch_exists(repo.path(), "feature-branch").await.unwrap();
        assert!(exists);
    }

    #[tokio::test]
    async fn test_branch_exists_with_slashes() {
        let repo = TestRepo::new().await.unwrap();
        repo.create_file_and_commit("test.txt", "content", "Initial commit").await.unwrap();

        // Create a branch with slashes
        repo.create_branch("feature/new-feature").await.unwrap();

        let exists = branch_exists(repo.path(), "feature/new-feature").await.unwrap();
        assert!(exists);
    }

    #[tokio::test]
    async fn test_branch_exists_case_sensitive() {
        let repo = TestRepo::new().await.unwrap();
        repo.create_file_and_commit("test.txt", "content", "Initial commit").await.unwrap();

        // Create a branch
        repo.create_branch("Feature-Branch").await.unwrap();

        // On case-insensitive filesystems (like macOS), this test may behave differently
        // So we'll check if the filesystem is case-sensitive first
        let exists_lowercase = branch_exists(repo.path(), "feature-branch").await.unwrap();
        let exists_correct = branch_exists(repo.path(), "Feature-Branch").await.unwrap();

        // The correct case should always exist
        assert!(exists_correct);

        // On case-sensitive systems, lowercase should not exist
        // On case-insensitive systems, both will exist
        // We can't make strong assertions here due to filesystem differences
        if !exists_lowercase {
            println!("Running on case-sensitive filesystem");
        } else {
            println!("Running on case-insensitive filesystem");
        }
    }

    #[tokio::test]
    async fn test_branch_exists_with_mock_executor() {
        let mut mock = MockCommandExecutor::new();
        mock.expect_command("git")
            .with_args(&["show-ref", "--verify", "--quiet", "refs/heads/main"])
            .in_dir("/test/repo")
            .returns_success();

        let result = branch_exists_with_executor(
            Arc::new(mock),
            Path::new("/test/repo"),
            "main"
        )
        .await
        .unwrap();
        assert!(result);
    }

    #[tokio::test]
    async fn test_branch_exists_with_mock_executor_nonexistent() {
        let mut mock = MockCommandExecutor::new();
        mock.expect_command("git")
            .with_args(&["show-ref", "--verify", "--quiet", "refs/heads/nonexistent"])
            .in_dir("/test/repo")
            .returns_output("", "error: reference 'refs/heads/nonexistent' not found", 1);

        let result = branch_exists_with_executor(
            Arc::new(mock),
            Path::new("/test/repo"),
            "nonexistent"
        )
        .await
        .unwrap();
        assert!(!result);
    }

    #[tokio::test]
    async fn test_branch_exists_with_mock_executor_git_error() {
        let mut mock = MockCommandExecutor::new();
        mock.expect_command("git")
            .with_args(&["show-ref", "--verify", "--quiet", "refs/heads/broken"])
            .in_dir("/test/repo")
            .returns_output("", "fatal: not a git repository", 128);

        let result = branch_exists_with_executor(
            Arc::new(mock),
            Path::new("/test/repo"),
            "broken"
        )
        .await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), PhantomError::Git { exit_code: 128, .. }));
    }
}
