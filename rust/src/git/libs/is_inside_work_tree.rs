use crate::core::command_executor::CommandExecutor;
use crate::git::git_executor_adapter;
use crate::{PhantomError, Result};
use std::path::Path;
use std::sync::Arc;
use tracing::debug;

/// Check if the current directory is inside a git work tree using a provided executor
pub async fn is_inside_work_tree_with_executor(
    executor: Arc<dyn CommandExecutor>,
    cwd: Option<&Path>,
) -> Result<bool> {
    let git_executor = match cwd {
        Some(path) => git_executor_adapter::GitExecutor::new(executor).with_cwd(path),
        None => git_executor_adapter::GitExecutor::new(executor),
    };

    debug!("Checking if inside work tree in {:?}", cwd);

    match git_executor.run(&["rev-parse", "--is-inside-work-tree"]).await {
        Ok(output) => {
            let is_inside = output.trim() == "true";
            debug!("Is inside work tree: {}", is_inside);
            Ok(is_inside)
        }
        Err(PhantomError::Git { exit_code: 128, .. }) => {
            // Exit code 128 means not in a git repository
            debug!("Not in a git repository");
            Ok(false)
        }
        Err(e) => {
            // Any other error is a real error
            Err(e)
        }
    }
}

/// Check if the current directory is inside a git work tree
pub async fn is_inside_work_tree(cwd: Option<&Path>) -> Result<bool> {
    use crate::core::executors::RealCommandExecutor;
    is_inside_work_tree_with_executor(Arc::new(RealCommandExecutor), cwd).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::executors::MockCommandExecutor;
    use crate::test_utils::TestRepo;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_is_inside_work_tree_true() {
        let repo = TestRepo::new().await.unwrap();

        let result = is_inside_work_tree(Some(repo.path())).await.unwrap();
        assert!(result);
    }

    #[tokio::test]
    async fn test_is_inside_work_tree_false() {
        let temp_dir = tempdir().unwrap();

        let result = is_inside_work_tree(Some(temp_dir.path())).await.unwrap();
        assert!(!result);
    }

    #[tokio::test]
    async fn test_is_inside_work_tree_subdirectory() {
        let repo = TestRepo::new().await.unwrap();

        // Create a subdirectory
        let sub_dir = repo.path().join("subdir");
        std::fs::create_dir(&sub_dir).unwrap();

        let result = is_inside_work_tree(Some(&sub_dir)).await.unwrap();
        assert!(result);
    }

    #[tokio::test]
    async fn test_is_inside_work_tree_with_mock_executor_true() {
        let mut mock = MockCommandExecutor::new();
        mock.expect_command("git")
            .with_args(&["rev-parse", "--is-inside-work-tree"])
            .in_dir("/test/repo")
            .returns_output("true\n", "", 0);

        let result =
            is_inside_work_tree_with_executor(Arc::new(mock), Some(Path::new("/test/repo")))
                .await
                .unwrap();
        assert!(result);
    }

    #[tokio::test]
    async fn test_is_inside_work_tree_with_mock_executor_false() {
        let mut mock = MockCommandExecutor::new();
        mock.expect_command("git")
            .with_args(&["rev-parse", "--is-inside-work-tree"])
            .in_dir("/test/repo")
            .returns_output("false\n", "", 0);

        let result =
            is_inside_work_tree_with_executor(Arc::new(mock), Some(Path::new("/test/repo")))
                .await
                .unwrap();
        assert!(!result);
    }

    #[tokio::test]
    async fn test_is_inside_work_tree_with_mock_executor_not_a_repo() {
        let mut mock = MockCommandExecutor::new();
        mock.expect_command("git")
            .with_args(&["rev-parse", "--is-inside-work-tree"])
            .in_dir("/not/a/repo")
            .returns_output(
                "",
                "fatal: not a git repository (or any of the parent directories): .git\n",
                128,
            );

        let result =
            is_inside_work_tree_with_executor(Arc::new(mock), Some(Path::new("/not/a/repo")))
                .await
                .unwrap();
        assert!(!result);
    }

    #[tokio::test]
    async fn test_is_inside_work_tree_with_mock_executor_no_cwd() {
        let mut mock = MockCommandExecutor::new();
        mock.expect_command("git")
            .with_args(&["rev-parse", "--is-inside-work-tree"])
            .returns_output("true\n", "", 0);

        let result = is_inside_work_tree_with_executor(Arc::new(mock), None).await.unwrap();
        assert!(result);
    }

    #[tokio::test]
    async fn test_is_inside_work_tree_with_mock_executor_unexpected_error() {
        let mut mock = MockCommandExecutor::new();
        mock.expect_command("git")
            .with_args(&["rev-parse", "--is-inside-work-tree"])
            .in_dir("/test/repo")
            .returns_output("", "fatal: some unexpected error", 2);

        let result =
            is_inside_work_tree_with_executor(Arc::new(mock), Some(Path::new("/test/repo"))).await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), PhantomError::Git { exit_code: 2, .. }));
    }
}
