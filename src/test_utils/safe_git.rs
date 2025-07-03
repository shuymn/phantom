use std::path::Path;
use std::process::Command;
use tempfile::TempDir;

/// A safe wrapper for Git commands in tests that ensures isolation from global configuration
pub struct SafeGitCommand {
    temp_home: TempDir,
}

impl SafeGitCommand {
    /// Create a new SafeGitCommand instance with isolated environment
    pub fn new() -> std::io::Result<Self> {
        let temp_home = TempDir::new()?;
        Ok(Self { temp_home })
    }

    /// Create a Git command with safe environment variables
    pub fn command(&self, args: &[&str]) -> Command {
        let mut cmd = Command::new("git");

        // Isolate from global git configuration
        cmd.env("HOME", self.temp_home.path())
            .env("GIT_CONFIG_GLOBAL", "/dev/null")
            .env("GIT_CONFIG_SYSTEM", "/dev/null")
            .env("GIT_CONFIG_NOSYSTEM", "1")
            // Set required git configuration
            .env("GIT_AUTHOR_NAME", "Test Suite")
            .env("GIT_AUTHOR_EMAIL", "test@example.com")
            .env("GIT_COMMITTER_NAME", "Test Suite")
            .env("GIT_COMMITTER_EMAIL", "test@example.com")
            // Disable editor and pager
            .env("GIT_EDITOR", "true")
            .env("GIT_PAGER", "cat")
            // Add the actual git arguments
            .args(args);

        cmd
    }

    /// Initialize a new git repository in the given path
    pub fn init_repo(&self, path: &Path) -> std::io::Result<()> {
        self.command(&["init"]).current_dir(path).output()?;

        // Set local config for the repository
        self.command(&["config", "user.name", "Test Suite"]).current_dir(path).output()?;

        self.command(&["config", "user.email", "test@example.com"]).current_dir(path).output()?;

        Ok(())
    }

    /// Create an initial commit in the repository
    pub fn create_initial_commit(&self, path: &Path) -> std::io::Result<()> {
        // Create a README file
        std::fs::write(path.join("README.md"), "# Test Repository")?;

        // Add and commit
        self.command(&["add", "."]).current_dir(path).output()?;

        self.command(&["commit", "-m", "Initial commit"]).current_dir(path).output()?;

        Ok(())
    }

    /// Add a worktree
    pub fn add_worktree(
        &self,
        repo_path: &Path,
        worktree_path: &Path,
        branch: &str,
    ) -> std::io::Result<()> {
        self.command(&["worktree", "add", "-b", branch, worktree_path.to_str().unwrap()])
            .current_dir(repo_path)
            .output()?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_safe_git_command_creation() {
        let git = SafeGitCommand::new().unwrap();
        let cmd = git.command(&["status"]);

        // Verify environment variables are set
        let env_vars: Vec<_> = cmd.get_envs().collect();
        assert!(env_vars.iter().any(|(k, v)| k == &"HOME" && v.is_some()));
        assert!(env_vars
            .iter()
            .any(|(k, v)| k == &"GIT_CONFIG_GLOBAL" && v == &Some("/dev/null".as_ref())));
    }

    #[test]
    fn test_init_repo() {
        let git = SafeGitCommand::new().unwrap();
        let temp_dir = TempDir::new().unwrap();

        git.init_repo(temp_dir.path()).unwrap();

        // Verify .git directory exists
        assert!(temp_dir.path().join(".git").exists());
    }

    #[test]
    fn test_create_initial_commit() {
        let git = SafeGitCommand::new().unwrap();
        let temp_dir = TempDir::new().unwrap();

        git.init_repo(temp_dir.path()).unwrap();
        git.create_initial_commit(temp_dir.path()).unwrap();

        // Verify README.md exists
        assert!(temp_dir.path().join("README.md").exists());

        // Verify commit was created
        let output =
            git.command(&["log", "--oneline"]).current_dir(temp_dir.path()).output().unwrap();

        let log = String::from_utf8_lossy(&output.stdout);
        assert!(log.contains("Initial commit"));
    }
}
