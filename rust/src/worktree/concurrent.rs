/// Concurrent operations for worktree management
/// These functions use async concurrency to improve performance when dealing with multiple worktrees
use crate::core::command_executor::CommandExecutor;
use crate::git::libs::list_worktrees::list_worktrees as git_list_worktrees;
use crate::worktree::list::{get_worktree_status, ListWorktreesSuccess, WorktreeInfo};
use crate::worktree::paths::get_phantom_directory;
use crate::Result;
use futures::future::join_all;
use std::path::Path;
use tracing::debug;

/// List all phantom worktrees with concurrent status checks
/// This version processes status checks for all worktrees in parallel
pub async fn list_worktrees_concurrent<E>(
    executor: E,
    git_root: &Path,
) -> Result<ListWorktreesSuccess>
where
    E: CommandExecutor + Clone + Send + Sync + 'static,
{
    debug!("Listing worktrees concurrently from git root: {:?}", git_root);

    let git_worktrees = git_list_worktrees(executor.clone(), git_root).await?;
    let phantom_dir = get_phantom_directory(git_root);
    let phantom_dir_canonical = phantom_dir.canonicalize().unwrap_or_else(|_| phantom_dir.clone());
    let phantom_dir_str = phantom_dir_canonical.to_string_lossy();

    // Filter phantom worktrees first
    let phantom_worktrees_filtered: Vec<_> = git_worktrees
        .into_iter()
        .filter_map(|worktree| {
            let worktree_path_canonical =
                worktree.path.canonicalize().unwrap_or_else(|_| worktree.path.clone());
            if worktree_path_canonical.starts_with(&phantom_dir_canonical) {
                let canonical_path_str = worktree_path_canonical.to_string_lossy();
                let name = if let Some(stripped) =
                    canonical_path_str.strip_prefix(&format!("{}/", phantom_dir_str))
                {
                    stripped.to_string()
                } else {
                    worktree.name.clone()
                };
                Some((name, worktree))
            } else {
                None
            }
        })
        .collect();

    // Create futures for concurrent status checks
    let status_futures: Vec<_> = phantom_worktrees_filtered
        .into_iter()
        .map(|(name, worktree)| {
            let executor = executor.clone();
            let path_str = worktree.path.to_string_lossy().to_string();

            async move {
                let is_clean = get_worktree_status(executor, &worktree.path).await.unwrap_or(true);

                WorktreeInfo { name, path: path_str, branch: worktree.branch, is_clean }
            }
        })
        .collect();

    // Execute all status checks concurrently
    let phantom_worktrees = join_all(status_futures).await;

    let message =
        if phantom_worktrees.is_empty() { Some("No worktrees found".to_string()) } else { None };

    Ok(ListWorktreesSuccess { worktrees: phantom_worktrees, message })
}

/// Get information about multiple worktrees concurrently
pub async fn get_worktrees_info_concurrent<E>(
    executor: E,
    git_root: &Path,
    names: &[&str],
) -> Result<Vec<WorktreeInfo>>
where
    E: CommandExecutor + Clone + Send + Sync + 'static,
{
    use crate::worktree::list::get_worktree_info;

    let info_futures: Vec<_> = names
        .iter()
        .map(|&name| {
            let executor = executor.clone();
            async move { get_worktree_info(executor, git_root, name).await }
        })
        .collect();

    let results = join_all(info_futures).await;

    // Collect successful results, skip errors
    Ok(results.into_iter().filter_map(|r| r.ok()).collect())
}

/// Batch check status of multiple worktrees concurrently
pub async fn check_worktrees_status_concurrent<E>(
    executor: E,
    worktree_paths: &[&Path],
) -> Vec<(usize, Result<bool>)>
where
    E: CommandExecutor + Clone + Send + Sync + 'static,
{
    let status_futures: Vec<_> = worktree_paths
        .iter()
        .enumerate()
        .map(|(idx, &path)| {
            let executor = executor.clone();
            let path = path.to_owned();
            async move {
                let result = get_worktree_status(executor, &path).await;
                (idx, result)
            }
        })
        .collect();

    join_all(status_futures).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::executors::MockCommandExecutor;
    use std::path::PathBuf;

    #[tokio::test]
    async fn test_list_worktrees_concurrent_empty() {
        let mut mock = MockCommandExecutor::new();
        let git_root = PathBuf::from("/repo");

        // Mock empty worktree list
        mock.expect_command("git").with_args(&["worktree", "list", "--porcelain"]).returns_output(
            "worktree /repo\nHEAD abc123\nbranch refs/heads/main\n",
            "",
            0,
        );

        let result = list_worktrees_concurrent(mock, &git_root).await.unwrap();

        assert!(result.worktrees.is_empty());
        assert_eq!(result.message, Some("No worktrees found".to_string()));
    }

    #[tokio::test]
    async fn test_concurrent_status_checks() {
        let mut mock = MockCommandExecutor::new();

        // Mock multiple status checks that would run concurrently
        mock.expect_command("git")
            .with_args(&["status", "--porcelain"])
            .in_dir("/repo/worktree1")
            .returns_output("", "", 0); // Clean

        mock.expect_command("git")
            .with_args(&["status", "--porcelain"])
            .in_dir("/repo/worktree2")
            .returns_output("M file.txt\n", "", 0); // Dirty

        mock.expect_command("git")
            .with_args(&["status", "--porcelain"])
            .in_dir("/repo/worktree3")
            .returns_output("", "", 0); // Clean

        let paths = vec![
            Path::new("/repo/worktree1"),
            Path::new("/repo/worktree2"),
            Path::new("/repo/worktree3"),
        ];

        let results = check_worktrees_status_concurrent(
            mock,
            &paths.iter().map(|p| p.as_ref()).collect::<Vec<_>>(),
        )
        .await;

        assert_eq!(results.len(), 3);
        assert!(results[0].1.as_ref().unwrap()); // Clean
        assert!(!results[1].1.as_ref().unwrap()); // Dirty
        assert!(results[2].1.as_ref().unwrap()); // Clean
    }
}
