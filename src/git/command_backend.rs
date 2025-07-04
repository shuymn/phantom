use crate::core::command_executor::CommandExecutor;
use crate::core::sealed::Sealed;
use crate::core::types::Worktree;
use crate::git::backend::{GitBackend, GitConfig};
use crate::git::libs::{
    add_worktree::add_worktree, attach_worktree::attach_worktree, branch_exists::branch_exists,
    create_branch::create_branch, current_commit::current_commit,
    get_current_branch::get_current_branch, get_current_worktree::get_current_worktree,
    get_git_root::get_git_root, is_inside_work_tree::is_inside_work_tree,
    list_branches::list_branches, list_worktrees::list_worktrees, remove_worktree::remove_worktree,
};
use crate::Result;
use async_trait::async_trait;
use std::path::{Path, PathBuf};

/// Git backend implementation using command-line git
pub struct CommandBackend<E> {
    config: GitConfig,
    executor: E,
}

impl<E> CommandBackend<E>
where
    E: CommandExecutor + Clone + 'static,
{
    /// Create a new CommandBackend with the given configuration and executor
    pub fn new(config: GitConfig, executor: E) -> Self {
        Self { config, executor }
    }
}

impl<E> Default for CommandBackend<E>
where
    E: CommandExecutor + Clone + Default + 'static,
{
    fn default() -> Self {
        Self::new(GitConfig::default(), E::default())
    }
}

// Implement the sealed trait to allow CommandBackend to implement GitBackend
impl<E> Sealed for CommandBackend<E> where E: CommandExecutor + Clone + 'static {}

#[async_trait]
impl<E> GitBackend for CommandBackend<E>
where
    E: CommandExecutor + Clone + Send + Sync + 'static,
{
    async fn current_branch(&self) -> Result<String> {
        let cwd = self.config.cwd.as_deref().unwrap_or(Path::new("."));
        get_current_branch(self.executor.clone(), cwd).await
    }

    async fn list_branches(&self) -> Result<Vec<String>> {
        let cwd = self.config.cwd.as_deref().unwrap_or(Path::new("."));
        list_branches(self.executor.clone(), cwd).await
    }

    async fn create_branch(&self, name: &str) -> Result<()> {
        let cwd = self.config.cwd.as_deref().unwrap_or(Path::new("."));
        create_branch(self.executor.clone(), cwd, name).await
    }

    async fn branch_exists(&self, name: &str) -> Result<bool> {
        let cwd = self.config.cwd.as_deref().unwrap_or(Path::new("."));
        branch_exists(self.executor.clone(), cwd, name).await
    }

    async fn get_root(&self) -> Result<PathBuf> {
        get_git_root(self.executor.clone()).await
    }

    async fn list_worktrees(&self) -> Result<Vec<Worktree>> {
        let cwd = self.config.cwd.as_deref().unwrap_or(Path::new("."));
        list_worktrees(self.executor.clone(), cwd).await
    }

    async fn add_worktree(
        &self,
        path: &Path,
        branch: Option<&str>,
        new_branch: bool,
        commitish: Option<&str>,
    ) -> Result<()> {
        let cwd = self.config.cwd.as_deref().unwrap_or(Path::new("."));
        add_worktree(self.executor.clone(), cwd, path, branch, new_branch, commitish).await
    }

    async fn attach_worktree(&self, path: &Path, branch: &str) -> Result<()> {
        let cwd = self.config.cwd.as_deref().unwrap_or(Path::new("."));
        attach_worktree(self.executor.clone(), cwd, path, branch).await
    }

    async fn remove_worktree(&self, path: &Path) -> Result<()> {
        let cwd = self.config.cwd.as_deref().unwrap_or(Path::new("."));
        remove_worktree(self.executor.clone(), cwd, path).await
    }

    async fn current_commit(&self) -> Result<String> {
        let cwd = self.config.cwd.as_deref().unwrap_or(Path::new("."));
        current_commit(self.executor.clone(), cwd).await
    }

    async fn is_inside_work_tree(&self) -> Result<bool> {
        is_inside_work_tree(self.executor.clone(), self.config.cwd.as_deref()).await
    }

    async fn current_worktree(&self) -> Result<Option<String>> {
        let cwd = self.config.cwd.as_deref().unwrap_or(Path::new("."));
        get_current_worktree(self.executor.clone(), cwd).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::TestRepo;

    #[tokio::test]
    async fn test_command_backend_basic_operations() {
        let repo = TestRepo::new().await.unwrap();
        repo.create_file_and_commit("test.txt", "content", "Initial commit").await.unwrap();

        use crate::core::executors::RealCommandExecutor;
        let config = GitConfig::with_cwd(repo.path());
        let backend = CommandBackend::new(config, RealCommandExecutor);

        // Test current_branch
        let branch = backend.current_branch().await.unwrap();
        assert_eq!(branch, "main");

        // Test is_inside_work_tree
        assert!(backend.is_inside_work_tree().await.unwrap());

        // Test list_branches
        let branches = backend.list_branches().await.unwrap();
        assert_eq!(branches, vec!["main"]);

        // Test create_branch
        backend.create_branch("test-branch").await.unwrap();
        assert!(backend.branch_exists("test-branch").await.unwrap());
    }

    #[tokio::test]
    async fn test_command_backend_worktree_operations() {
        let repo = TestRepo::new().await.unwrap();
        repo.create_file_and_commit("test.txt", "content", "Initial commit").await.unwrap();

        use crate::core::executors::RealCommandExecutor;
        let config = GitConfig::with_cwd(repo.path());
        let backend = CommandBackend::new(config, RealCommandExecutor);

        // Test list_worktrees
        let worktrees = backend.list_worktrees().await.unwrap();
        assert_eq!(worktrees.len(), 1);

        // Test add_worktree
        let timestamp =
            std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis();
        let worktree_path =
            repo.path().parent().unwrap().join(format!("test-worktree-{timestamp}"));
        backend.add_worktree(&worktree_path, Some("feature"), true, None).await.unwrap();

        let worktrees = backend.list_worktrees().await.unwrap();
        assert_eq!(worktrees.len(), 2);

        // Test current_worktree
        let current = backend.current_worktree().await.unwrap();
        assert!(current.is_none()); // We're in the main worktree

        // Test remove_worktree
        backend.remove_worktree(&worktree_path).await.unwrap();
        let worktrees = backend.list_worktrees().await.unwrap();
        assert_eq!(worktrees.len(), 1);
    }

    #[tokio::test]
    async fn test_command_backend_current_commit() {
        let repo = TestRepo::new().await.unwrap();
        repo.create_file_and_commit("test.txt", "content", "Initial commit").await.unwrap();

        use crate::core::executors::RealCommandExecutor;
        let config = GitConfig::with_cwd(repo.path());
        let backend = CommandBackend::new(config, RealCommandExecutor);

        // Test current_commit
        let commit = backend.current_commit().await.unwrap();
        assert_eq!(commit.len(), 40); // SHA-1 hash length
    }
}
