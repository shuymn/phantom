use crate::git::backend::GitBackend;
use crate::git::libs::add_worktree::add_worktree;
use crate::worktree::errors::WorktreeError;
use crate::worktree::file_copier::copy_files_concurrent;
use crate::worktree::paths::{get_phantom_directory, get_worktree_path};
use crate::worktree::types::{CreateWorktreeOptions, CreateWorktreeSuccess};
use crate::worktree::validate::{validate_worktree_does_not_exist, validate_worktree_name};
use crate::{PhantomError, Result};
use std::path::Path;
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
    let commitish = options.commitish.as_deref();

    let worktrees_path = get_phantom_directory(git_root);
    let worktree_path = get_worktree_path(git_root, name);

    // Create phantom directory if it doesn't exist
    if fs::metadata(&worktrees_path).await.is_err() {
        debug!("Creating phantom directory at {:?}", worktrees_path);
        fs::create_dir_all(&worktrees_path).await.map_err(|e| {
            PhantomError::Io(std::io::Error::other(format!(
                "Failed to create phantom directory: {}",
                e
            )))
        })?;
    }

    // Validate worktree doesn't already exist
    use crate::core::filesystems::RealFileSystem;
    let filesystem = RealFileSystem::new();
    validate_worktree_does_not_exist(git_root, name, &filesystem).await?;

    // Add the worktree using the git backend
    info!("Creating worktree '{}' at {:?}", name, worktree_path);

    // For now, we'll use the existing add_worktree function
    // In the future, this should use the GitBackend trait
    add_worktree(git_root, &worktree_path, Some(branch), true, commitish).await.map_err(
        |e| match e {
            PhantomError::Git { message, .. } => WorktreeError::GitOperation {
                operation: "worktree add".to_string(),
                details: message,
            }
            .into(),
            _ => e,
        },
    )?;

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
            match copy_files_concurrent(git_root, &worktree_path, files_to_copy).await {
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
pub async fn create_worktree_with_backend<B>(
    backend: &B,
    git_root: &Path,
    name: &str,
    options: CreateWorktreeOptions,
) -> Result<CreateWorktreeSuccess>
where
    B: GitBackend,
{
    // Validate the worktree name
    validate_worktree_name(name)?;

    let branch = options.branch.as_deref().unwrap_or(name);
    let commitish = options.commitish.as_deref();
    let worktrees_path = get_phantom_directory(git_root);
    let worktree_path = get_worktree_path(git_root, name);

    // Create phantom directory if it doesn't exist
    if fs::metadata(&worktrees_path).await.is_err() {
        debug!("Creating phantom directory at {:?}", worktrees_path);
        fs::create_dir_all(&worktrees_path).await.map_err(|e| {
            PhantomError::Io(std::io::Error::other(format!(
                "Failed to create phantom directory: {}",
                e
            )))
        })?;
    }

    // Validate worktree doesn't already exist
    use crate::core::filesystems::RealFileSystem;
    let filesystem = RealFileSystem::new();
    validate_worktree_does_not_exist(git_root, name, &filesystem).await?;

    // Add the worktree using the git backend
    info!("Creating worktree '{}' at {:?}", name, worktree_path);
    backend.add_worktree(&worktree_path, Some(branch), true, commitish).await.map_err(
        |e| match e {
            PhantomError::Git { message, .. } => WorktreeError::GitOperation {
                operation: "worktree add".to_string(),
                details: message,
            }
            .into(),
            _ => e,
        },
    )?;

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
            match copy_files_concurrent(git_root, &worktree_path, files_to_copy).await {
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
        let result = create_worktree_with_backend(&backend, repo.path(), "feature", options).await;

        assert!(result.is_ok());
        let success = result.unwrap();
        assert!(success.message.contains("Created worktree 'feature'"));
        assert!(Path::new(&success.path).exists());
    }

    #[tokio::test]
    async fn test_create_worktree_with_copy_files() {
        let repo = TestRepo::new().await.unwrap();
        repo.create_file_and_commit("test.txt", "content", "Initial commit").await.unwrap();
        repo.create_file_and_commit("config.json", "{}", "Add config").await.unwrap();
        repo.create_file_and_commit(".env", "KEY=value", "Add env").await.unwrap();

        let options = CreateWorktreeOptions {
            copy_files: Some(vec!["config.json".to_string(), ".env".to_string()]),
            ..Default::default()
        };
        let result = create_worktree(repo.path(), "feature", options).await;

        assert!(result.is_ok());
        let success = result.unwrap();
        assert!(success.copied_files.is_some());
        assert_eq!(success.copied_files.unwrap().len(), 2);
        assert!(success.skipped_files.is_some());
        assert_eq!(success.skipped_files.unwrap().len(), 0);
        assert!(success.copy_error.is_none());

        // Verify files were copied
        let worktree_path = Path::new(&success.path);
        assert!(worktree_path.join("config.json").exists());
        assert!(worktree_path.join(".env").exists());
    }

    #[tokio::test]
    async fn test_create_worktree_with_copy_files_some_missing() {
        let repo = TestRepo::new().await.unwrap();
        repo.create_file_and_commit("test.txt", "content", "Initial commit").await.unwrap();
        repo.create_file_and_commit("config.json", "{}", "Add config").await.unwrap();

        let options = CreateWorktreeOptions {
            copy_files: Some(vec!["config.json".to_string(), "missing.txt".to_string()]),
            ..Default::default()
        };
        let result = create_worktree(repo.path(), "feature-missing", options).await;

        assert!(result.is_ok());
        let success = result.unwrap();
        assert!(success.copied_files.is_some());
        assert_eq!(success.copied_files.unwrap().len(), 1);
        assert!(success.skipped_files.is_some());
        assert_eq!(success.skipped_files.unwrap().len(), 1);
        assert!(success.copy_error.is_none());
    }

    #[tokio::test]
    async fn test_create_worktree_with_empty_copy_files() {
        let repo = TestRepo::new().await.unwrap();
        repo.create_file_and_commit("test.txt", "content", "Initial commit").await.unwrap();

        let options = CreateWorktreeOptions { copy_files: Some(vec![]), ..Default::default() };
        let result = create_worktree(repo.path(), "feature-empty", options).await;

        assert!(result.is_ok());
        let success = result.unwrap();
        assert!(success.copied_files.is_none());
        assert!(success.skipped_files.is_none());
        assert!(success.copy_error.is_none());
    }

    #[tokio::test]
    async fn test_create_worktree_success_fields() {
        let repo = TestRepo::new().await.unwrap();
        repo.create_file_and_commit("test.txt", "content", "Initial commit").await.unwrap();

        let options = CreateWorktreeOptions::default();
        let result = create_worktree(repo.path(), "verify-fields", options).await.unwrap();

        assert!(result.message.contains("Created worktree 'verify-fields'"));
        assert!(result.path.ends_with("verify-fields"));
        assert!(result.copied_files.is_none());
        assert!(result.skipped_files.is_none());
        assert!(result.copy_error.is_none());
    }

    #[tokio::test]
    async fn test_create_worktree_with_backend_and_copy_files() {
        let repo = TestRepo::new().await.unwrap();
        repo.create_file_and_commit("test.txt", "content", "Initial commit").await.unwrap();
        repo.create_file_and_commit("data.json", "[]", "Add data").await.unwrap();

        let backend = create_backend_for_dir(repo.path());
        let options = CreateWorktreeOptions {
            copy_files: Some(vec!["data.json".to_string()]),
            ..Default::default()
        };
        let result =
            create_worktree_with_backend(&backend, repo.path(), "backend-copy", options).await;

        assert!(result.is_ok());
        let success = result.unwrap();
        assert!(success.copied_files.is_some());
        assert_eq!(success.copied_files.unwrap().len(), 1);

        // Verify file was copied
        let worktree_path = Path::new(&success.path);
        assert!(worktree_path.join("data.json").exists());
    }

    #[tokio::test]
    async fn test_create_worktree_success_serialization() {
        use crate::worktree::types::CreateWorktreeSuccess;

        // Test CreateWorktreeSuccess serialization
        let success = CreateWorktreeSuccess {
            message: "Created".to_string(),
            path: "/path/to/worktree".to_string(),
            copied_files: Some(vec!["file1".to_string()]),
            skipped_files: Some(vec!["file2".to_string()]),
            copy_error: Some("Error".to_string()),
        };
        let json = serde_json::to_string(&success).unwrap();
        let deserialized: CreateWorktreeSuccess = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.message, success.message);
        assert_eq!(deserialized.path, success.path);
        assert_eq!(deserialized.copied_files, success.copied_files);
        assert_eq!(deserialized.skipped_files, success.skipped_files);
        assert_eq!(deserialized.copy_error, success.copy_error);

        // Test with skip_serializing_if
        let success_minimal = CreateWorktreeSuccess {
            message: "Created".to_string(),
            path: "/path".to_string(),
            copied_files: None,
            skipped_files: None,
            copy_error: None,
        };
        let json = serde_json::to_string(&success_minimal).unwrap();
        assert!(!json.contains("copied_files"));
        assert!(!json.contains("skipped_files"));
        assert!(!json.contains("copy_error"));
    }
}
