use crate::core::command_executor::CommandExecutor;
use crate::core::filesystem::FileSystem;
use crate::git::git_executor_adapter::GitExecutor;
use crate::worktree::errors::WorktreeError;
use crate::worktree::types::DeleteWorktreeOptions;
use crate::worktree::types::DeleteWorktreeSuccess;
use crate::worktree::validate::validate_worktree_exists;
use crate::{PhantomError, Result};
use std::path::Path;
use tracing::{debug, info};

/// Status of a worktree regarding uncommitted changes
#[derive(Debug, Clone)]
pub struct WorktreeStatus {
    pub has_uncommitted_changes: bool,
    pub changed_files: usize,
}

/// Helper to create a GitExecutor with the given executor and working directory
fn create_git_executor<E>(executor: E, cwd: &Path) -> GitExecutor<E>
where
    E: CommandExecutor + Clone + 'static,
{
    GitExecutor::new(executor).with_cwd(cwd)
}

/// Helper to convert git operation errors
fn git_operation_error(operation: &str, error: impl std::fmt::Display) -> PhantomError {
    WorktreeError::GitOperation { operation: operation.to_string(), details: error.to_string() }
        .into()
}

/// Get the status of a worktree (uncommitted changes) with executor
pub async fn get_worktree_status<E>(executor: E, worktree_path: &Path) -> WorktreeStatus
where
    E: CommandExecutor + Clone + 'static,
{
    let git_executor = create_git_executor(executor, worktree_path);

    match git_executor.run(&["status", "--porcelain"]).await {
        Ok(output) => {
            let output = output.trim();
            if output.is_empty() {
                WorktreeStatus { has_uncommitted_changes: false, changed_files: 0 }
            } else {
                let changed_files = output.lines().count();
                WorktreeStatus { has_uncommitted_changes: true, changed_files }
            }
        }
        Err(_) => {
            // If git status fails, assume no changes
            WorktreeStatus { has_uncommitted_changes: false, changed_files: 0 }
        }
    }
}

/// Remove a worktree using git commands with executor
async fn remove_worktree<E>(
    executor: E,
    git_root: &Path,
    worktree_path: &Path,
    force: bool,
) -> Result<()>
where
    E: CommandExecutor + Clone + 'static,
{
    let git_executor = create_git_executor(executor, git_root);

    // First try normal removal
    let result = git_executor.run(&["worktree", "remove", &worktree_path.to_string_lossy()]).await;

    match result {
        Ok(_) => Ok(()),
        Err(_) if force => {
            // If normal removal fails and force is true, try force removal
            git_executor
                .run(&["worktree", "remove", "--force", &worktree_path.to_string_lossy()])
                .await
                .map(|_| ())
                .map_err(|e| git_operation_error("worktree remove", e))
        }
        Err(e) => Err(git_operation_error("worktree remove", e)),
    }
}

/// Delete a branch with executor
async fn delete_branch<E>(executor: E, git_root: &Path, branch_name: &str) -> Result<bool>
where
    E: CommandExecutor + Clone + 'static,
{
    let git_executor = create_git_executor(executor, git_root);

    match git_executor.run(&["branch", "-D", branch_name]).await {
        Ok(_) => Ok(true),
        Err(e) => {
            debug!("Failed to delete branch '{branch_name}': {e}");
            Ok(false)
        }
    }
}

/// Delete a worktree with executor
pub async fn delete_worktree<E>(
    executor: E,
    git_root: &Path,
    name: &str,
    options: DeleteWorktreeOptions,
    filesystem: &dyn FileSystem,
) -> Result<DeleteWorktreeSuccess>
where
    E: CommandExecutor + Clone + 'static,
{
    // Validate worktree exists
    let validation = validate_worktree_exists(git_root, name, filesystem).await?;
    let worktree_path = validation.path;

    // Get worktree status
    let status = get_worktree_status(executor.clone(), &worktree_path).await;

    // Check for uncommitted changes
    if status.has_uncommitted_changes && !options.force {
        return Err(PhantomError::WorktreeHasUncommittedChanges { name: name.to_string() });
    }

    // Remove the worktree
    info!("Removing worktree '{}' at {:?}", name, worktree_path);
    remove_worktree(executor.clone(), git_root, &worktree_path, options.force).await?;

    // Try to delete the branch
    let branch_deleted = delete_branch(executor, git_root, name).await?;

    // Build the success message
    let mut message = if branch_deleted {
        format!("Deleted worktree '{name}' and its branch '{name}'")
    } else {
        format!("Deleted worktree '{name}'")
    };

    if status.has_uncommitted_changes {
        message = format!(
            "Warning: Worktree '{name}' had uncommitted changes ({} files)\n{message}",
            status.changed_files
        );
    }

    Ok(DeleteWorktreeSuccess { message, path: worktree_path.to_string_lossy().to_string() })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::TestRepo;
    use crate::worktree::create::create_worktree;
    use crate::worktree::types::CreateWorktreeOptions;
    use crate::PhantomError;

    #[tokio::test]
    async fn test_get_worktree_status_clean() {
        let repo = TestRepo::new().await.unwrap();
        repo.create_file_and_commit("test.txt", "content", "Initial commit").await.unwrap();

        use crate::core::executors::RealCommandExecutor;
        let status = get_worktree_status(RealCommandExecutor::new(), repo.path()).await;
        assert!(!status.has_uncommitted_changes);
        assert_eq!(status.changed_files, 0);
    }

    #[tokio::test]
    async fn test_get_worktree_status_with_changes() {
        let repo = TestRepo::new().await.unwrap();
        repo.create_file_and_commit("test.txt", "content", "Initial commit").await.unwrap();

        // Create an uncommitted file
        std::fs::write(repo.path().join("new.txt"), "new content").unwrap();

        use crate::core::executors::RealCommandExecutor;
        let status = get_worktree_status(RealCommandExecutor::new(), repo.path()).await;
        assert!(status.has_uncommitted_changes);
        assert_eq!(status.changed_files, 1);
    }

    #[tokio::test]
    async fn test_delete_worktree_basic() {
        let repo = TestRepo::new().await.unwrap();
        repo.create_file_and_commit("test.txt", "content", "Initial commit").await.unwrap();

        // Create a worktree
        let create_options = CreateWorktreeOptions::default();
        use crate::core::executors::RealCommandExecutor;
        create_worktree(RealCommandExecutor, repo.path(), "feature", create_options).await.unwrap();

        // Delete the worktree
        use crate::core::filesystems::RealFileSystem;
        let filesystem = RealFileSystem::new();
        let delete_options = DeleteWorktreeOptions::default();
        let result = delete_worktree(
            RealCommandExecutor,
            repo.path(),
            "feature",
            delete_options,
            &filesystem,
        )
        .await;

        assert!(result.is_ok());
        let success = result.unwrap();
        assert!(success.message.contains("Deleted worktree 'feature'"));
    }

    #[tokio::test]
    async fn test_delete_worktree_with_uncommitted_changes() {
        let repo = TestRepo::new().await.unwrap();
        repo.create_file_and_commit("test.txt", "content", "Initial commit").await.unwrap();

        // Create a worktree
        let create_options = CreateWorktreeOptions::default();
        use crate::core::executors::RealCommandExecutor;
        let create_result =
            create_worktree(RealCommandExecutor, repo.path(), "feature", create_options)
                .await
                .unwrap();

        // Add uncommitted changes to the worktree
        let worktree_path = Path::new(&create_result.path);
        std::fs::write(worktree_path.join("new.txt"), "uncommitted content").unwrap();

        // Try to delete without force
        use crate::core::filesystems::RealFileSystem;
        let filesystem = RealFileSystem::new();
        let delete_options = DeleteWorktreeOptions { force: false };
        let result = delete_worktree(
            RealCommandExecutor,
            repo.path(),
            "feature",
            delete_options,
            &filesystem,
        )
        .await;

        assert!(result.is_err());
        match result.unwrap_err() {
            PhantomError::WorktreeHasUncommittedChanges { name } => {
                assert_eq!(name, "feature");
            }
            _ => panic!("Expected WorktreeHasUncommittedChanges error"),
        }

        // Delete with force
        let delete_options = DeleteWorktreeOptions { force: true };
        let result = delete_worktree(
            RealCommandExecutor,
            repo.path(),
            "feature",
            delete_options,
            &filesystem,
        )
        .await;

        assert!(result.is_ok());
        let success = result.unwrap();
        assert!(success.message.contains("Warning: Worktree 'feature' had uncommitted changes"));
    }

    #[tokio::test]
    async fn test_delete_worktree_not_found() {
        let repo = TestRepo::new().await.unwrap();

        use crate::core::executors::RealCommandExecutor;
        use crate::core::filesystems::RealFileSystem;
        let filesystem = RealFileSystem::new();
        let delete_options = DeleteWorktreeOptions::default();
        let result = delete_worktree(
            RealCommandExecutor,
            repo.path(),
            "nonexistent",
            delete_options,
            &filesystem,
        )
        .await;

        assert!(result.is_err());
        match result.unwrap_err() {
            PhantomError::WorktreeNotFound { name } => assert_eq!(name, "nonexistent"),
            _ => panic!("Expected Worktree error"),
        }
    }
}
