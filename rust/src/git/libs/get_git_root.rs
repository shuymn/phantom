use crate::core::command_executor::CommandExecutor;
use crate::core::const_utils::dirs;
use crate::git::git_executor_adapter::GitExecutor;
use crate::Result;
use std::path::{Path, PathBuf};
use tracing::debug;

/// Get the main git repository root (not the worktree root) with generic executor
/// This version avoids dynamic dispatch for better performance
pub async fn get_git_root<E>(executor: E) -> Result<PathBuf>
where
    E: CommandExecutor + Clone + 'static,
{
    let git_executor = GitExecutor::new(executor);

    // First try to get the git common directory
    let common_dir = match git_executor.run(&["rev-parse", "--git-common-dir"]).await {
        Ok(output) => output.trim().to_string(),
        Err(crate::PhantomError::Git { exit_code: 128, stderr, .. })
            if stderr.contains("not a git repository") =>
        {
            return Err(crate::PhantomError::NotInGitRepository);
        }
        Err(e) => return Err(e),
    };

    debug!("Git common dir: {}", common_dir);

    if common_dir.ends_with(&format!("/{}", dirs::GIT)) || common_dir == dirs::GIT {
        // We're in a regular repository or worktree
        let path = Path::new(&common_dir);
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

    // If we get here, we might be in a bare repository or the main worktree
    // Try to get the top-level directory
    let top_level = git_executor.run(&["rev-parse", "--show-toplevel"]).await?;
    let top_level = top_level.trim();

    debug!("Git top level: {}", top_level);

    let path = Path::new(top_level);
    let absolute =
        if path.is_relative() { std::env::current_dir()?.join(path) } else { path.to_path_buf() };

    // Always canonicalize the path to ensure consistency
    Ok(absolute.canonicalize().unwrap_or(absolute))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::executors::MockCommandExecutor;
    use crate::test_utils::TestRepo;
    use serial_test::serial;

    #[tokio::test]
    async fn test_get_git_root_generic() {
        let mut mock = MockCommandExecutor::new();
        mock.expect_command("git").with_args(&["rev-parse", "--git-common-dir"]).returns_output(
            "/home/user/project/.git",
            "",
            0,
        );

        let result = get_git_root(mock).await.unwrap();
        assert_eq!(result, PathBuf::from("/home/user/project"));
    }

    #[tokio::test]
    async fn test_get_git_root_bare_repo() {
        let mut mock = MockCommandExecutor::new();
        mock.expect_command("git")
            .with_args(&["rev-parse", "--git-common-dir"])
            .returns_output(".", "", 0);
        mock.expect_command("git").with_args(&["rev-parse", "--show-toplevel"]).returns_output(
            "/home/user/bare-repo",
            "",
            0,
        );

        let result = get_git_root(mock).await.unwrap();
        assert_eq!(result, PathBuf::from("/home/user/bare-repo"));
    }

    #[tokio::test]
    #[serial]
    async fn test_get_git_root_real_repo() {
        let repo = TestRepo::new().await.unwrap();
        repo.create_file_and_commit("test.txt", "content", "Initial commit").await.unwrap();

        // Change to the repo directory for the test
        let original_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(repo.path()).unwrap();

        let executor = crate::core::executors::RealCommandExecutor::new();
        let result = get_git_root(executor).await.unwrap();

        // Restore original directory
        std::env::set_current_dir(original_dir).unwrap();

        assert_eq!(result.canonicalize().unwrap(), repo.path().canonicalize().unwrap());
    }
}
