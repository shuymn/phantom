use crate::git::executor::GitExecutor;
use crate::Result;
use std::path::{Path, PathBuf};
use tracing::debug;

/// Get the main git repository root (not the worktree root)
pub async fn get_git_root() -> Result<PathBuf> {
    let executor = GitExecutor::new();

    // First try to get the git common directory
    let common_dir = executor.run(&["rev-parse", "--git-common-dir"]).await?;
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
            return Ok(absolute);
        }
    }

    // Fall back to show-toplevel for the main repository
    let toplevel = executor.run(&["rev-parse", "--show-toplevel"]).await?;
    let toplevel = toplevel.trim();

    Ok(PathBuf::from(toplevel))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::git::executor::GitExecutor;
    use crate::test_utils::TestRepo;
    use std::env;

    #[tokio::test]
    async fn test_get_git_root_in_main_repo() {
        let repo = TestRepo::new().await.unwrap();
        repo.create_file_and_commit("test.txt", "content", "Initial commit").await.unwrap();

        // Change to the main repo directory
        let _guard = TestWorkingDir::new(repo.path());

        let git_root = get_git_root().await.unwrap();
        assert_eq!(git_root.canonicalize().unwrap(), repo.path().canonicalize().unwrap());
    }

    #[tokio::test]
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
    async fn test_get_git_root_in_worktree() {
        let repo = TestRepo::new().await.unwrap();
        repo.create_file_and_commit("test.txt", "content", "Initial commit").await.unwrap();

        // Create a worktree with unique name
        let executor = GitExecutor::with_cwd(repo.path());
        use std::time::{SystemTime, UNIX_EPOCH};
        let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis();
        let unique_name = format!("test-worktree-{}-{}", std::process::id(), timestamp);
        let worktree_path = repo.path().parent().unwrap().join(&unique_name);
        executor
            .run(&["worktree", "add", "-b", "feature", &worktree_path.to_string_lossy()])
            .await
            .unwrap();

        // Change to the worktree directory
        let _guard = TestWorkingDir::new(&worktree_path);

        // From a worktree, get_git_root should return the main repository root
        let git_root = get_git_root().await.unwrap();
        assert_eq!(git_root.canonicalize().unwrap(), repo.path().canonicalize().unwrap());
    }

    #[tokio::test]
    async fn test_get_git_root_in_worktree_subdirectory() {
        let repo = TestRepo::new().await.unwrap();
        repo.create_file_and_commit("test.txt", "content", "Initial commit").await.unwrap();

        // Create a worktree with unique name
        let executor = GitExecutor::with_cwd(repo.path());
        use std::time::{SystemTime, UNIX_EPOCH};
        let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis();
        let unique_name = format!("test-worktree-sub-{}-{}", std::process::id(), timestamp);
        let worktree_path = repo.path().parent().unwrap().join(&unique_name);
        executor
            .run(&["worktree", "add", "-b", "feature-sub", &worktree_path.to_string_lossy()])
            .await
            .unwrap();

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
            let original = env::current_dir().unwrap();
            env::set_current_dir(path).unwrap();
            Self { original }
        }
    }

    impl Drop for TestWorkingDir {
        fn drop(&mut self) {
            env::set_current_dir(&self.original).unwrap();
        }
    }
}
