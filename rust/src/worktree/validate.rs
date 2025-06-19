use crate::core::filesystem::FileSystem;
use crate::worktree::errors::WorktreeError;
use crate::worktree::paths::{get_phantom_directory, get_worktree_path};
use crate::worktree::types::{WorktreeDoesNotExistSuccess, WorktreeExistsSuccess};
use crate::Result;
use std::path::Path;

/// Validate that a worktree exists
pub async fn validate_worktree_exists(
    git_root: &Path,
    name: &str,
    filesystem: &dyn FileSystem,
) -> Result<WorktreeExistsSuccess> {
    let worktree_path = get_worktree_path(git_root, name);

    match filesystem.is_dir(&worktree_path).await {
        Ok(true) => Ok(WorktreeExistsSuccess { path: worktree_path }),
        Ok(false) => Err(WorktreeError::NotFound(name.to_string()).into()),
        Err(_) => Err(WorktreeError::NotFound(name.to_string()).into()),
    }
}

/// Validate that a worktree does not exist
pub async fn validate_worktree_does_not_exist(
    git_root: &Path,
    name: &str,
    filesystem: &dyn FileSystem,
) -> Result<WorktreeDoesNotExistSuccess> {
    let worktree_path = get_worktree_path(git_root, name);

    match filesystem.is_dir(&worktree_path).await {
        Ok(true) => Err(WorktreeError::AlreadyExists(name.to_string()).into()),
        Ok(false) => Ok(WorktreeDoesNotExistSuccess { path: worktree_path }),
        Err(_) => Ok(WorktreeDoesNotExistSuccess { path: worktree_path }),
    }
}

/// Validate that the phantom directory exists
pub async fn validate_phantom_directory_exists(
    git_root: &Path,
    filesystem: &dyn FileSystem,
) -> bool {
    let phantom_dir = get_phantom_directory(git_root);
    filesystem.is_dir(&phantom_dir).await.unwrap_or(false)
}

/// Validate worktree name
pub fn validate_worktree_name(name: &str) -> Result<()> {
    if name.trim().is_empty() {
        return Err(WorktreeError::InvalidName("Phantom name cannot be empty".to_string()).into());
    }

    // Only allow alphanumeric, hyphen, underscore, dot, and slash
    let valid_name_pattern = regex::Regex::new(r"^[a-zA-Z0-9\-_.\/]+$").unwrap();
    if !valid_name_pattern.is_match(name) {
        return Err(WorktreeError::InvalidName(
            "Phantom name can only contain letters, numbers, hyphens, underscores, dots, and slashes".to_string(),
        ).into());
    }

    if name.contains("..") {
        return Err(WorktreeError::InvalidName(
            "Phantom name cannot contain consecutive dots".to_string(),
        )
        .into());
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::filesystems::RealFileSystem;
    use crate::test_utils::TestRepo;

    #[tokio::test]
    async fn test_validate_worktree_exists() {
        let repo = TestRepo::new().await.unwrap();
        let filesystem = RealFileSystem::new();
        let result = validate_worktree_exists(repo.path(), "nonexistent", &filesystem).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_validate_worktree_does_not_exist() {
        let repo = TestRepo::new().await.unwrap();
        let filesystem = RealFileSystem::new();
        let result =
            validate_worktree_does_not_exist(repo.path(), "nonexistent", &filesystem).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_validate_phantom_directory_exists() {
        let repo = TestRepo::new().await.unwrap();
        let filesystem = RealFileSystem::new();
        let exists = validate_phantom_directory_exists(repo.path(), &filesystem).await;
        assert!(!exists); // Should not exist in a fresh repo
    }

    #[test]
    fn test_validate_worktree_name_valid() {
        assert!(validate_worktree_name("feature-branch").is_ok());
        assert!(validate_worktree_name("feature/sub-feature").is_ok());
        assert!(validate_worktree_name("feature_123").is_ok());
        assert!(validate_worktree_name("v1.0.0").is_ok());
    }

    #[test]
    fn test_validate_worktree_name_invalid() {
        assert!(validate_worktree_name("").is_err());
        assert!(validate_worktree_name("   ").is_err());
        assert!(validate_worktree_name("feature branch").is_err()); // Contains space
        assert!(validate_worktree_name("feature@branch").is_err()); // Contains @
        assert!(validate_worktree_name("feature..branch").is_err()); // Contains ..
        assert!(validate_worktree_name("feature!branch").is_err()); // Contains !
    }
}
