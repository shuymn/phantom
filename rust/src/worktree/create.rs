use crate::git::backend::GitBackend;
use crate::git::libs::add_worktree::add_worktree;
use crate::worktree::errors::WorktreeError;
use crate::worktree::paths::{get_phantom_directory, get_worktree_path};
use crate::worktree::types::{CreateWorktreeOptions, CreateWorktreeSuccess};
use crate::worktree::validate::{validate_worktree_does_not_exist, validate_worktree_name};
use crate::{PhantomError, Result};
use std::path::Path;
use std::sync::Arc;
use tokio::fs;
use tracing::{debug, info};

/// Create a new worktree
pub async fn create_worktree(
    git_root: &Path,
    name: &str,
    options: CreateWorktreeOptions,
) -> Result<CreateWorktreeSuccess> {
    // Validate the worktree name
    validate_worktree_name(name)?;

    let branch = options.branch.as_deref().unwrap_or(name);
    let _commitish = options.commitish.as_deref().unwrap_or("HEAD");

    let worktrees_path = get_phantom_directory(git_root);
    let worktree_path = get_worktree_path(git_root, name);

    // Create phantom directory if it doesn't exist
    if !fs::metadata(&worktrees_path).await.is_ok() {
        debug!("Creating phantom directory at {:?}", worktrees_path);
        fs::create_dir_all(&worktrees_path).await.map_err(|e| {
            PhantomError::Io(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Failed to create phantom directory: {}", e),
            ))
        })?;
    }

    // Validate worktree doesn't already exist
    validate_worktree_does_not_exist(git_root, name).await?;

    // Add the worktree using the git backend
    info!("Creating worktree '{}' at {:?}", name, worktree_path);

    // For now, we'll use the existing add_worktree function
    // In the future, this should use the GitBackend trait
    add_worktree(git_root, &worktree_path, Some(branch), true).await.map_err(|e| match e {
        PhantomError::Git { message, .. } => {
            WorktreeError::GitOperation { operation: "worktree add".to_string(), details: message }
                .into()
        }
        _ => e,
    })?;

    let mut result = CreateWorktreeSuccess {
        message: format!("Created worktree '{}' at {}", name, worktree_path.display()),
        path: worktree_path.to_string_lossy().to_string(),
        copied_files: None,
        skipped_files: None,
        copy_error: None,
    };

    // Handle file copying if requested
    if let Some(ref files_to_copy) = options.copy_files {
        if !files_to_copy.is_empty() {
            match copy_files(git_root, &worktree_path, files_to_copy).await {
                Ok(copy_result) => {
                    result.copied_files = Some(copy_result.copied_files);
                    result.skipped_files = Some(copy_result.skipped_files);
                }
                Err(e) => {
                    result.copy_error = Some(e.to_string());
                }
            }
        }
    }

    Ok(result)
}

/// Create a new worktree using a GitBackend
pub async fn create_worktree_with_backend(
    backend: Arc<dyn GitBackend>,
    git_root: &Path,
    name: &str,
    options: CreateWorktreeOptions,
) -> Result<CreateWorktreeSuccess> {
    // Validate the worktree name
    validate_worktree_name(name)?;

    let branch = options.branch.as_deref().unwrap_or(name);
    let worktrees_path = get_phantom_directory(git_root);
    let worktree_path = get_worktree_path(git_root, name);

    // Create phantom directory if it doesn't exist
    if !fs::metadata(&worktrees_path).await.is_ok() {
        debug!("Creating phantom directory at {:?}", worktrees_path);
        fs::create_dir_all(&worktrees_path).await.map_err(|e| {
            PhantomError::Io(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Failed to create phantom directory: {}", e),
            ))
        })?;
    }

    // Validate worktree doesn't already exist
    validate_worktree_does_not_exist(git_root, name).await?;

    // Add the worktree using the git backend
    info!("Creating worktree '{}' at {:?}", name, worktree_path);
    backend.add_worktree(&worktree_path, Some(branch), true).await.map_err(|e| match e {
        PhantomError::Git { message, .. } => {
            WorktreeError::GitOperation { operation: "worktree add".to_string(), details: message }
                .into()
        }
        _ => e,
    })?;

    let mut result = CreateWorktreeSuccess {
        message: format!("Created worktree '{}' at {}", name, worktree_path.display()),
        path: worktree_path.to_string_lossy().to_string(),
        copied_files: None,
        skipped_files: None,
        copy_error: None,
    };

    // Handle file copying if requested
    if let Some(ref files_to_copy) = options.copy_files {
        if !files_to_copy.is_empty() {
            match copy_files(git_root, &worktree_path, files_to_copy).await {
                Ok(copy_result) => {
                    result.copied_files = Some(copy_result.copied_files);
                    result.skipped_files = Some(copy_result.skipped_files);
                }
                Err(e) => {
                    result.copy_error = Some(e.to_string());
                }
            }
        }
    }

    Ok(result)
}

// Placeholder for file copying - will be implemented later
async fn copy_files(
    _source: &Path,
    _dest: &Path,
    _files: &[String],
) -> Result<crate::worktree::types::FileCopyResult> {
    // TODO: Implement file copying
    Ok(crate::worktree::types::FileCopyResult { copied_files: vec![], skipped_files: vec![] })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::git::factory::create_backend_for_dir;
    use crate::test_utils::TestRepo;

    #[tokio::test]
    async fn test_create_worktree_basic() {
        let repo = TestRepo::new().await.unwrap();
        repo.create_file_and_commit("test.txt", "content", "Initial commit").await.unwrap();

        let options = CreateWorktreeOptions::default();
        let result = create_worktree(repo.path(), "feature", options).await;

        assert!(result.is_ok());
        let success = result.unwrap();
        assert!(success.message.contains("Created worktree 'feature'"));
        assert!(success.path.contains("feature"));
        assert!(Path::new(&success.path).exists());
    }

    #[tokio::test]
    async fn test_create_worktree_with_branch_name() {
        let repo = TestRepo::new().await.unwrap();
        repo.create_file_and_commit("test.txt", "content", "Initial commit").await.unwrap();

        let options = CreateWorktreeOptions {
            branch: Some("custom-branch".to_string()),
            ..Default::default()
        };
        let result = create_worktree(repo.path(), "feature", options).await;

        assert!(result.is_ok());
        let success = result.unwrap();
        assert!(Path::new(&success.path).exists());
    }

    #[tokio::test]
    async fn test_create_worktree_duplicate_fails() {
        let repo = TestRepo::new().await.unwrap();
        repo.create_file_and_commit("test.txt", "content", "Initial commit").await.unwrap();

        let options = CreateWorktreeOptions::default();

        // Create first worktree
        let result1 = create_worktree(repo.path(), "feature", options.clone()).await;
        assert!(result1.is_ok());

        // Try to create duplicate
        let result2 = create_worktree(repo.path(), "feature", options).await;
        assert!(result2.is_err());

        match result2.unwrap_err() {
            PhantomError::Worktree(msg) => assert!(msg.contains("already exists")),
            _ => panic!("Expected WorktreeError"),
        }
    }

    #[tokio::test]
    async fn test_create_worktree_invalid_name() {
        let repo = TestRepo::new().await.unwrap();

        let options = CreateWorktreeOptions::default();
        let result = create_worktree(repo.path(), "feature branch", options).await;

        assert!(result.is_err());
        match result.unwrap_err() {
            PhantomError::Validation(msg) => assert!(msg.contains("can only contain")),
            _ => panic!("Expected ValidationError"),
        }
    }

    #[tokio::test]
    async fn test_create_worktree_with_backend() {
        let repo = TestRepo::new().await.unwrap();
        repo.create_file_and_commit("test.txt", "content", "Initial commit").await.unwrap();

        let backend = create_backend_for_dir(repo.path());
        let options = CreateWorktreeOptions::default();
        let result = create_worktree_with_backend(backend, repo.path(), "feature", options).await;

        assert!(result.is_ok());
        let success = result.unwrap();
        assert!(success.message.contains("Created worktree 'feature'"));
        assert!(Path::new(&success.path).exists());
    }
}
