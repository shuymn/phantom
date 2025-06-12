use crate::git::backend::{GitBackend, GitConfig};
use crate::git::command_backend::CommandBackend;
use std::sync::Arc;

/// Type of Git backend to use
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BackendType {
    /// Use command-line git
    #[default]
    Command,
    /// Use libgit2 (when feature is enabled)
    #[cfg(feature = "libgit2")]
    LibGit2,
}

/// Create a Git backend of the specified type
pub fn create_backend(backend_type: BackendType, config: GitConfig) -> Arc<dyn GitBackend> {
    match backend_type {
        BackendType::Command => Arc::new(CommandBackend::new(config)),
        #[cfg(feature = "libgit2")]
        BackendType::LibGit2 => {
            // TODO: Implement LibGit2Backend when needed
            panic!("LibGit2 backend not yet implemented");
        }
    }
}

/// Create a Git backend with default configuration
pub fn create_default_backend() -> Arc<dyn GitBackend> {
    create_backend(BackendType::default(), GitConfig::default())
}

/// Create a Git backend for a specific directory
pub fn create_backend_for_dir(path: impl Into<std::path::PathBuf>) -> Arc<dyn GitBackend> {
    let config = GitConfig::with_cwd(path);
    create_backend(BackendType::default(), config)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::TestRepo;

    #[test]
    fn test_backend_type_default() {
        assert_eq!(BackendType::default(), BackendType::Command);
    }

    #[tokio::test]
    async fn test_create_backend() {
        let config = GitConfig::default();
        let backend = create_backend(BackendType::Command, config);

        // Verify it's a valid backend by checking if we can use it
        let result = backend.is_inside_work_tree().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_create_default_backend() {
        let backend = create_default_backend();

        // Verify it's a valid backend
        let result = backend.is_inside_work_tree().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_create_backend_for_dir() {
        let repo = TestRepo::new().await.unwrap();
        let backend = create_backend_for_dir(repo.path());

        // Should be inside a work tree
        assert!(backend.is_inside_work_tree().await.unwrap());

        // Should be able to get current branch
        let branch = backend.current_branch().await.unwrap();
        assert_eq!(branch, "main");
    }
}
