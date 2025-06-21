use super::types::{CreateWorktreeOptions, CreateWorktreeSuccess};
use crate::git::backend::GitBackend;
use crate::Result;
use std::marker::PhantomData;
use std::path::Path;
use std::sync::Arc;

/// Type states for the builder
pub mod builder_states {
    /// Initial state - no name set
    pub struct NoName;

    /// Name has been set
    pub struct WithName;

    /// Ready to build (all required fields set)
    pub struct Ready;
}

/// A type-safe builder for creating worktrees
pub struct WorktreeBuilder<State> {
    name: Option<String>,
    branch: Option<String>,
    base: Option<String>,
    copy_files: Vec<String>,
    _state: PhantomData<State>,
}

impl WorktreeBuilder<builder_states::NoName> {
    /// Create a new worktree builder
    pub fn new() -> Self {
        WorktreeBuilder {
            name: None,
            branch: None,
            base: None,
            copy_files: Vec::new(),
            _state: PhantomData,
        }
    }
}

impl Default for WorktreeBuilder<builder_states::NoName> {
    fn default() -> Self {
        Self::new()
    }
}

impl WorktreeBuilder<builder_states::NoName> {
    /// Set the worktree name (required)
    pub fn name(self, name: impl Into<String>) -> WorktreeBuilder<builder_states::WithName> {
        WorktreeBuilder {
            name: Some(name.into()),
            branch: self.branch,
            base: self.base,
            copy_files: self.copy_files,
            _state: PhantomData,
        }
    }
}

impl WorktreeBuilder<builder_states::WithName> {
    /// Set the branch name (optional, defaults to worktree name)
    pub fn branch(mut self, branch: impl Into<String>) -> Self {
        self.branch = Some(branch.into());
        self
    }

    /// Set the base commit/branch (optional, defaults to HEAD)
    pub fn base(mut self, base: impl Into<String>) -> Self {
        self.base = Some(base.into());
        self
    }

    /// Add a file to copy from the source worktree
    pub fn copy_file(mut self, file: impl Into<String>) -> Self {
        self.copy_files.push(file.into());
        self
    }

    /// Add multiple files to copy
    pub fn copy_files(mut self, files: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.copy_files.extend(files.into_iter().map(Into::into));
        self
    }

    /// Validate and prepare for building
    pub fn validate(self) -> Result<WorktreeBuilder<builder_states::Ready>> {
        let name = self.name.clone().unwrap(); // Safe because we're in WithName state

        // Validate worktree name
        super::validate::validate_worktree_name(&name)?;

        Ok(WorktreeBuilder {
            name: self.name,
            branch: self.branch,
            base: self.base,
            copy_files: self.copy_files,
            _state: PhantomData,
        })
    }

    /// Build without validation (use with caution)
    pub fn build_unchecked(self) -> CreateWorktreeOptions {
        let name = self.name.unwrap(); // Safe because we're in WithName state
        CreateWorktreeOptions {
            branch: self.branch.or_else(|| Some(name.clone())),
            commitish: self.base,
            copy_files: if self.copy_files.is_empty() { None } else { Some(self.copy_files) },
        }
    }

    /// Create the worktree directly, validating and building in one step
    pub async fn create(
        self,
        backend: Arc<dyn GitBackend>,
        git_root: &Path,
    ) -> Result<CreateWorktreeSuccess> {
        let validated = self.validate()?;
        validated.create(backend, git_root).await
    }
}

impl WorktreeBuilder<builder_states::Ready> {
    /// Build the worktree creation options
    pub fn build(self) -> CreateWorktreeOptions {
        let name = self.name.unwrap(); // Safe because we validated
        CreateWorktreeOptions {
            branch: self.branch.or_else(|| Some(name.clone())),
            commitish: self.base,
            copy_files: if self.copy_files.is_empty() { None } else { Some(self.copy_files) },
        }
    }

    /// Get the validated name
    pub fn name(&self) -> &str {
        self.name.as_ref().unwrap() // Safe because we validated
    }

    /// Create the worktree directly using a GitBackend
    pub async fn create(
        self,
        backend: Arc<dyn GitBackend>,
        git_root: &Path,
    ) -> Result<CreateWorktreeSuccess> {
        let name = self.name.clone().unwrap(); // Safe because we validated
        let options = self.build();
        super::create::create_worktree_with_backend(backend, git_root, &name, options).await
    }
}

/// Convenience function to start building
pub fn build_worktree() -> WorktreeBuilder<builder_states::NoName> {
    WorktreeBuilder::new()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builder_happy_path() {
        let options = build_worktree()
            .name("feature-123")
            .branch("feature/awesome")
            .base("develop")
            .copy_file(".env")
            .copy_file("config.local.js")
            .validate()
            .unwrap()
            .build();

        assert_eq!(options.branch, Some("feature/awesome".to_string()));
        assert_eq!(options.commitish, Some("develop".to_string()));
        assert_eq!(
            options.copy_files,
            Some(vec![".env".to_string(), "config.local.js".to_string()])
        );
    }

    #[test]
    fn test_builder_minimal() {
        let options = build_worktree().name("minimal").build_unchecked();

        assert_eq!(options.branch, Some("minimal".to_string()));
        assert_eq!(options.commitish, None);
        assert_eq!(options.copy_files, None);
    }

    #[test]
    fn test_builder_validation_fails() {
        let result = build_worktree()
            .name("invalid..name") // Double dots are not allowed
            .validate();

        assert!(result.is_err());
    }

    #[test]
    fn test_builder_with_multiple_files() {
        let files = vec![".env", "config.yml", "secrets.json"];

        let options = build_worktree().name("test").copy_files(files).build_unchecked();

        assert_eq!(
            options.copy_files,
            Some(vec![".env".to_string(), "config.yml".to_string(), "secrets.json".to_string()])
        );
    }

    #[test]
    fn test_builder_wont_compile_without_name() {
        // This test demonstrates compile-time safety
        // Uncomment to see compilation error:

        // let options = build_worktree()
        //     .branch("feature")
        //     .build(); // ERROR: method `build` not found
    }

    #[tokio::test]
    async fn test_builder_create_worktree() {
        use crate::git::factory::create_backend_for_dir;
        use crate::test_utils::TestRepo;

        let repo = TestRepo::new().await.unwrap();
        repo.create_file_and_commit("test.txt", "content", "Initial commit").await.unwrap();

        let backend = create_backend_for_dir(repo.path());

        let result = build_worktree()
            .name("feature-test")
            .branch("feature/test-branch")
            .create(backend, repo.path())
            .await;

        assert!(result.is_ok());
        let success = result.unwrap();
        assert!(success.message.contains("Created worktree 'feature-test'"));
    }

    #[tokio::test]
    async fn test_builder_create_with_validation() {
        use crate::git::factory::create_backend_for_dir;
        use crate::test_utils::TestRepo;

        let repo = TestRepo::new().await.unwrap();
        repo.create_file_and_commit("test.txt", "content", "Initial commit").await.unwrap();

        let backend = create_backend_for_dir(repo.path());

        let result = build_worktree()
            .name("feature-123")
            .base("HEAD")
            .validate()
            .unwrap()
            .create(backend, repo.path())
            .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_builder_create_invalid_name_fails() {
        use crate::git::factory::create_backend_for_dir;
        use crate::test_utils::TestRepo;

        let repo = TestRepo::new().await.unwrap();
        repo.create_file_and_commit("test.txt", "content", "Initial commit").await.unwrap();

        let backend = create_backend_for_dir(repo.path());

        let result = build_worktree().name("invalid..name").create(backend, repo.path()).await;

        assert!(result.is_err());
    }
}
