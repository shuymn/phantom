//! DEPRECATED: This module uses dynamic dispatch and is kept only for benchmarking comparisons.
//! Use get_git_root_generic instead for better performance through static dispatch.

use crate::core::command_executor::CommandExecutor;
use crate::git::git_executor_adapter::GitExecutor;
use crate::Result;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tracing::debug;

/// Get the main git repository root (not the worktree root)
pub async fn get_git_root_with_executor(executor: Arc<dyn CommandExecutor>) -> Result<PathBuf> {
    let git_executor = GitExecutor::new(executor);

    // First try to get the git common directory
    let common_dir = git_executor.run(&["rev-parse", "--git-common-dir"]).await?;
    let common_dir = common_dir.trim();

    debug!("Git common dir: {}", common_dir);

    if common_dir.ends_with("/.git") || common_dir == ".git" {
        // We're in a regular repository or worktree
        let path = Path::new(common_dir);
        if let Some(parent) = path.parent() {
            let absolute = if parent.is_relative() {
                std::env::current_dir()?.join(parent)
            } else {
                parent.to_path_buf()
            };
            // Always canonicalize the path to ensure consistency
            return Ok(absolute.canonicalize().unwrap_or(absolute));
        }
    }

    // Fall back to show-toplevel for the main repository
    let toplevel = git_executor.run(&["rev-parse", "--show-toplevel"]).await?;
    let toplevel = toplevel.trim();
    let toplevel_path = PathBuf::from(toplevel);

    // Always canonicalize the path to ensure consistency
    Ok(toplevel_path.canonicalize().unwrap_or(toplevel_path))
}

/// Get the main git repository root using the default executor
pub async fn get_git_root() -> Result<PathBuf> {
    use crate::core::executors::RealCommandExecutor;
    get_git_root_with_executor(Arc::new(RealCommandExecutor)).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::executors::RealCommandExecutor;
    use crate::test_utils::TestRepo;
    use serial_test::serial;
    use std::env;

    #[tokio::test]
    #[serial]
    async fn test_get_git_root_in_main_repo() {
        let repo = TestRepo::new().await.unwrap();
        repo.create_file_and_commit("test.txt", "content", "Initial commit").await.unwrap();

        // Change to the main repo directory
        let _guard = TestWorkingDir::new(repo.path());

        let git_root = get_git_root().await.unwrap();
        assert_eq!(git_root.canonicalize().unwrap(), repo.path().canonicalize().unwrap());
    }

    #[tokio::test]
    #[serial]
    async fn test_get_git_root_in_subdirectory() {
        let repo = TestRepo::new().await.unwrap();
        repo.create_file_and_commit("test.txt", "content", "Initial commit").await.unwrap();

        // Create a subdirectory
        let subdir = repo.path().join("subdir");
        std::fs::create_dir(&subdir).unwrap();

        // Change to the subdirectory
        let _guard = TestWorkingDir::new(&subdir);

        let git_root = get_git_root().await.unwrap();
        assert_eq!(git_root.canonicalize().unwrap(), repo.path().canonicalize().unwrap());
    }

    #[tokio::test]
    #[serial]
    async fn test_get_git_root_in_worktree() {
        let repo = TestRepo::new().await.unwrap();
        repo.create_file_and_commit("test.txt", "content", "Initial commit").await.unwrap();

        // Create a worktree with unique name
        let executor = GitExecutor::new(Arc::new(RealCommandExecutor::new())).with_cwd(repo.path());
        use std::time::{SystemTime, UNIX_EPOCH};
        let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis();
        let unique_name = format!("test-worktree-{}-{}", std::process::id(), timestamp);
        // Create worktree inside a subdirectory of the repo to avoid permission issues
        let worktrees_dir = repo.path().join("test-worktrees");
        std::fs::create_dir_all(&worktrees_dir).unwrap();
        let worktree_path = worktrees_dir.join(&unique_name);
        executor
            .run(&["worktree", "add", "-b", "feature", &worktree_path.to_string_lossy()])
            .await
            .unwrap_or_else(|e| panic!("Failed to create worktree at {:?}: {}", worktree_path, e));

        // Verify worktree was created
        assert!(worktree_path.exists(), "Worktree directory should exist after creation");

        // Change to the worktree directory
        let _guard = TestWorkingDir::new(&worktree_path);

        // From a worktree, get_git_root should return the main repository root
        let git_root = get_git_root().await.unwrap();
        assert_eq!(git_root.canonicalize().unwrap(), repo.path().canonicalize().unwrap());
    }

    #[tokio::test]
    #[serial]
    async fn test_get_git_root_in_worktree_subdirectory() {
        let repo = TestRepo::new().await.unwrap();
        repo.create_file_and_commit("test.txt", "content", "Initial commit").await.unwrap();

        // Create a worktree with unique name
        let executor = GitExecutor::new(Arc::new(RealCommandExecutor::new())).with_cwd(repo.path());
        use std::time::{SystemTime, UNIX_EPOCH};
        let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis();
        let unique_name = format!("test-worktree-sub-{}-{}", std::process::id(), timestamp);
        // Create worktree inside a subdirectory of the repo to avoid permission issues
        let worktrees_dir = repo.path().join("test-worktrees");
        std::fs::create_dir_all(&worktrees_dir).unwrap();
        let worktree_path = worktrees_dir.join(&unique_name);
        executor
            .run(&["worktree", "add", "-b", "feature-sub", &worktree_path.to_string_lossy()])
            .await
            .unwrap_or_else(|e| panic!("Failed to create worktree at {:?}: {}", worktree_path, e));

        // Verify worktree was created
        assert!(worktree_path.exists(), "Worktree directory should exist after creation");

        // Create a subdirectory in the worktree
        let subdir = worktree_path.join("subdir");
        std::fs::create_dir(&subdir).unwrap();

        // Change to the worktree subdirectory
        let _guard = TestWorkingDir::new(&subdir);

        // From a worktree subdirectory, get_git_root should return the main repository root
        let git_root = get_git_root().await.unwrap();
        assert_eq!(git_root.canonicalize().unwrap(), repo.path().canonicalize().unwrap());
    }

    /// Helper struct to temporarily change working directory
    struct TestWorkingDir {
        original: std::path::PathBuf,
    }

    impl TestWorkingDir {
        fn new(path: &Path) -> Self {
            let original = env::current_dir().expect("Failed to get current directory");

            // Ensure the path exists before changing to it
            if !path.exists() {
                panic!("Path does not exist: {:?}", path);
            }

            env::set_current_dir(path)
                .unwrap_or_else(|e| panic!("Failed to set current dir to {:?}: {}", path, e));
            Self { original }
        }
    }

    impl Drop for TestWorkingDir {
        fn drop(&mut self) {
            env::set_current_dir(&self.original).unwrap();
        }
    }
}
