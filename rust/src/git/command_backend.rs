use crate::core::types::Worktree;
use crate::git::backend::{GitBackend, GitConfig};
use crate::git::executor::GitExecutor;
use crate::git::libs::{
    add_worktree::add_worktree, attach_worktree::attach_worktree, branch_exists::branch_exists,
    create_branch::create_branch, current_commit::current_commit,
    get_current_branch::get_current_branch, get_current_worktree::get_current_worktree,
    get_git_root::get_git_root, is_inside_work_tree::is_inside_work_tree,
    list_worktrees::list_worktrees,
};
use crate::Result;
use async_trait::async_trait;
use std::path::{Path, PathBuf};

/// Git backend implementation using command-line git
pub struct CommandBackend {
    config: GitConfig,
}

impl CommandBackend {
    /// Create a new CommandBackend with the given configuration
    pub fn new(config: GitConfig) -> Self {
        Self { config }
    }

    /// Create a GitExecutor for the current configuration
    fn executor(&self) -> GitExecutor {
        match &self.config.cwd {
            Some(cwd) => GitExecutor::with_cwd(cwd),
            None => GitExecutor::new(),
        }
    }
}

impl Default for CommandBackend {
    fn default() -> Self {
        Self::new(GitConfig::default())
    }
}

#[async_trait]
impl GitBackend for CommandBackend {
    async fn init(&self, path: &Path) -> Result<()> {
        let executor = GitExecutor::with_cwd(path);
        executor.run(&["init"]).await?;
        Ok(())
    }

    async fn clone(&self, url: &str, path: &Path) -> Result<()> {
        let executor = self.executor();
        executor.run(&["clone", url, &path.to_string_lossy()]).await?;
        Ok(())
    }

    async fn add(&self, paths: &[&str]) -> Result<()> {
        let executor = self.executor();
        let mut args = vec!["add"];
        args.extend_from_slice(paths);
        executor.run(&args).await?;
        Ok(())
    }

    async fn commit(&self, message: &str) -> Result<String> {
        let executor = self.executor();
        executor.run(&["commit", "-m", message]).await?;

        // Get the commit hash
        let hash = executor.run(&["rev-parse", "HEAD"]).await?;
        Ok(hash.trim().to_string())
    }

    async fn current_branch(&self) -> Result<String> {
        let cwd = self.config.cwd.as_deref().unwrap_or(Path::new("."));
        get_current_branch(cwd).await
    }

    async fn list_branches(&self) -> Result<Vec<String>> {
        let executor = self.executor();
        let output = executor.run(&["branch", "--format=%(refname:short)"]).await?;

        let branches: Vec<String> = output
            .lines()
            .map(|line| line.trim().to_string())
            .filter(|line| !line.is_empty())
            .collect();

        Ok(branches)
    }

    async fn create_branch(&self, name: &str) -> Result<()> {
        let cwd = self.config.cwd.as_deref().unwrap_or(Path::new("."));
        create_branch(cwd, name).await
    }

    async fn checkout(&self, branch: &str) -> Result<()> {
        let executor = self.executor();
        executor.run(&["checkout", branch]).await?;
        Ok(())
    }

    async fn branch_exists(&self, name: &str) -> Result<bool> {
        let cwd = self.config.cwd.as_deref().unwrap_or(Path::new("."));
        branch_exists(cwd, name).await
    }

    async fn get_root(&self) -> Result<PathBuf> {
        get_git_root().await
    }

    async fn list_worktrees(&self) -> Result<Vec<Worktree>> {
        let cwd = self.config.cwd.as_deref().unwrap_or(Path::new("."));
        list_worktrees(cwd).await
    }

    async fn add_worktree(
        &self,
        path: &Path,
        branch: Option<&str>,
        new_branch: bool,
    ) -> Result<()> {
        let cwd = self.config.cwd.as_deref().unwrap_or(Path::new("."));
        add_worktree(cwd, path, branch, new_branch).await
    }

    async fn attach_worktree(&self, path: &Path, branch: &str) -> Result<()> {
        let cwd = self.config.cwd.as_deref().unwrap_or(Path::new("."));
        attach_worktree(cwd, path, branch).await
    }

    async fn remove_worktree(&self, path: &Path) -> Result<()> {
        let executor = self.executor();
        executor.run(&["worktree", "remove", &path.to_string_lossy()]).await?;
        Ok(())
    }

    async fn status(&self) -> Result<String> {
        let executor = self.executor();
        executor.run(&["status", "--porcelain"]).await
    }

    async fn current_commit(&self) -> Result<String> {
        let cwd = self.config.cwd.as_deref().unwrap_or(Path::new("."));
        current_commit(cwd).await
    }

    async fn is_inside_work_tree(&self) -> Result<bool> {
        is_inside_work_tree(self.config.cwd.as_deref()).await
    }

    async fn current_worktree(&self) -> Result<Option<String>> {
        let cwd = self.config.cwd.as_deref().unwrap_or(Path::new("."));
        get_current_worktree(cwd).await
    }

    async fn execute(&self, args: &[&str]) -> Result<String> {
        let executor = self.executor();
        executor.run(args).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::TestRepo;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_command_backend_init() {
        let temp_dir = tempdir().unwrap();
        let backend = CommandBackend::default();

        backend.init(temp_dir.path()).await.unwrap();

        assert!(temp_dir.path().join(".git").exists());
    }

    #[tokio::test]
    async fn test_command_backend_basic_operations() {
        let repo = TestRepo::new().await.unwrap();
        repo.create_file_and_commit("test.txt", "content", "Initial commit").await.unwrap();

        let config = GitConfig::with_cwd(repo.path());
        let backend = CommandBackend::new(config);

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

        // Test checkout
        backend.checkout("test-branch").await.unwrap();
        let current = backend.current_branch().await.unwrap();
        assert_eq!(current, "test-branch");
    }

    #[tokio::test]
    async fn test_command_backend_worktree_operations() {
        let repo = TestRepo::new().await.unwrap();
        repo.create_file_and_commit("test.txt", "content", "Initial commit").await.unwrap();

        let config = GitConfig::with_cwd(repo.path());
        let backend = CommandBackend::new(config);

        // Test list_worktrees
        let worktrees = backend.list_worktrees().await.unwrap();
        assert_eq!(worktrees.len(), 1);

        // Test add_worktree
        let timestamp =
            std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis();
        let worktree_path =
            repo.path().parent().unwrap().join(format!("test-worktree-{}", timestamp));
        backend.add_worktree(&worktree_path, Some("feature"), true).await.unwrap();

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
    async fn test_command_backend_commit_operations() {
        let repo = TestRepo::new().await.unwrap();
        let config = GitConfig::with_cwd(repo.path());
        let backend = CommandBackend::new(config);

        // Create a file
        std::fs::write(repo.path().join("test.txt"), "content").unwrap();

        // Test add
        backend.add(&["test.txt"]).await.unwrap();

        // Test status
        let status = backend.status().await.unwrap();
        assert!(status.contains("test.txt"));

        // Test commit
        let hash = backend.commit("Test commit").await.unwrap();
        assert_eq!(hash.len(), 40); // SHA-1 hash length

        // Test current_commit
        let current = backend.current_commit().await.unwrap();
        assert_eq!(current, hash);
    }

    #[tokio::test]
    async fn test_command_backend_execute() {
        let repo = TestRepo::new().await.unwrap();
        let config = GitConfig::with_cwd(repo.path());
        let backend = CommandBackend::new(config);

        // Test execute with arbitrary command - use a command that always works
        let output = backend.execute(&["--version"]).await.unwrap();
        assert!(output.contains("git version"));
    }
}
