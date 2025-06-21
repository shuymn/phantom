/// Extension traits for common types to improve ergonomics
/// These traits add convenience methods while keeping core types simple
use crate::core::command_executor::{CommandConfig, CommandExecutor};
use crate::core::types::Worktree;
use crate::git::git_executor_adapter::GitExecutor as GitExecutorAdapter;
use crate::{PhantomError, Result};
use std::path::Path;
use std::sync::Arc;

/// Extension trait for CommandExecutor with convenience methods
#[allow(async_fn_in_trait)]
pub trait CommandExecutorExt: CommandExecutor {
    /// Execute a simple command with just program and args
    async fn run_simple(&self, program: &str, args: &[&str]) -> Result<String> {
        let config =
            CommandConfig::new(program).with_args(args.iter().map(|s| s.to_string()).collect());

        let output = self.execute(config).await?;
        if output.success() {
            Ok(output.stdout.trim().to_string())
        } else {
            Err(PhantomError::ProcessExecution(format!(
                "Command failed: {} {}",
                program,
                args.join(" ")
            )))
        }
    }

    /// Execute a command in a specific directory
    async fn run_in_dir(&self, program: &str, args: &[&str], dir: &Path) -> Result<String> {
        let config = CommandConfig::new(program)
            .with_args(args.iter().map(|s| s.to_string()).collect())
            .with_cwd(dir.to_path_buf());

        let output = self.execute(config).await?;
        if output.success() {
            Ok(output.stdout.trim().to_string())
        } else {
            Err(PhantomError::ProcessExecution(format!(
                "Command failed: {} {} in {:?}",
                program,
                args.join(" "),
                dir
            )))
        }
    }

    /// Create a GitExecutor from this CommandExecutor
    fn git(self: Arc<Self>) -> GitExecutorAdapter
    where
        Self: Sized + 'static,
    {
        GitExecutorAdapter::new(self as Arc<dyn CommandExecutor>)
    }
}

/// Implement CommandExecutorExt for all CommandExecutor types
impl<T: CommandExecutor + ?Sized> CommandExecutorExt for T {}

/// Extension trait for Worktree with convenience methods
pub trait WorktreeExt {
    /// Check if this is the main worktree
    fn is_main(&self) -> bool;

    /// Get a display name (branch name or commit short hash)
    fn display_name(&self) -> String;

    /// Check if worktree has uncommitted changes
    fn is_dirty(&self) -> bool;

    /// Get the relative path from git root
    fn relative_path(&self) -> Option<String>;
}

impl WorktreeExt for Worktree {
    fn is_main(&self) -> bool {
        self.branch.as_ref().is_some_and(|b| b == "main" || b == "master")
    }

    fn display_name(&self) -> String {
        self.branch.as_ref().cloned().unwrap_or_else(|| {
            // Use first 7 chars of commit hash if no branch
            self.commit.chars().take(7).collect()
        })
    }

    fn is_dirty(&self) -> bool {
        // This would need to be async in a real implementation
        // For now, just return false
        false
    }

    fn relative_path(&self) -> Option<String> {
        // Extract relative path from absolute path
        self.path.file_name().and_then(|name| name.to_str()).map(|s| s.to_string())
    }
}

/// Extension trait for Result types to add context
pub trait ResultExt<T> {
    /// Add command context to errors
    fn with_command_context(self, command: &str, args: &[&str]) -> Result<T>;

    /// Add path context to errors
    fn with_path_context(self, path: &Path) -> Result<T>;
}

impl<T> ResultExt<T> for Result<T> {
    fn with_command_context(self, command: &str, args: &[&str]) -> Result<T> {
        self.map_err(|e| {
            PhantomError::ProcessExecution(format!("{}: {} {}", e, command, args.join(" ")))
        })
    }

    fn with_path_context(self, path: &Path) -> Result<T> {
        self.map_err(|e| PhantomError::FileOperation(format!("{}: {:?}", e, path)))
    }
}

/// Extension trait for string slices with git-specific helpers
pub trait StrExt {
    /// Check if this looks like a git branch name
    fn is_branch_like(&self) -> bool;

    /// Check if this looks like a commit hash
    fn is_commit_like(&self) -> bool;

    /// Sanitize for use as a worktree name
    fn sanitize_worktree_name(&self) -> String;
}

/// Extension trait for PhantomConfig with convenience methods
pub trait PhantomConfigExt {
    /// Check if any files are configured to be copied
    fn has_copy_files(&self) -> bool;

    /// Add a file to the copy list
    fn add_copy_file(&mut self, file: impl Into<String>);

    /// Remove a file from the copy list
    fn remove_copy_file(&mut self, file: &str) -> bool;

    /// Check if a specific file is in the copy list
    fn should_copy_file(&self, file: &str) -> bool;

    /// Get terminal multiplexer or default
    fn multiplexer_or_default(&self) -> &str;
}

/// Extension trait for GitConfig with convenience methods  
pub trait GitConfigExt {
    /// Check if user configuration is complete
    fn is_user_configured(&self) -> bool;

    /// Get user display name (name or email or "unknown")
    fn user_display(&self) -> String;

    /// Create environment variables for git commands
    fn to_env_vars(&self) -> Vec<(String, String)>;
}

impl StrExt for str {
    fn is_branch_like(&self) -> bool {
        !self.is_empty()
            && !self.is_commit_like()
            && self.chars().all(|c| c.is_alphanumeric() || "/-_.".contains(c))
    }

    fn is_commit_like(&self) -> bool {
        self.len() >= 7 && self.chars().all(|c| c.is_ascii_hexdigit())
    }

    fn sanitize_worktree_name(&self) -> String {
        self.chars()
            .map(|c| if c.is_alphanumeric() || "/-_.".contains(c) { c } else { '-' })
            .collect()
    }
}

impl PhantomConfigExt for crate::core::types::PhantomConfig {
    fn has_copy_files(&self) -> bool {
        !self.copy_files.is_empty()
    }

    fn add_copy_file(&mut self, file: impl Into<String>) {
        let file = file.into();
        if !self.copy_files.contains(&file) {
            self.copy_files.push(file);
        }
    }

    fn remove_copy_file(&mut self, file: &str) -> bool {
        if let Some(pos) = self.copy_files.iter().position(|f| f == file) {
            self.copy_files.remove(pos);
            true
        } else {
            false
        }
    }

    fn should_copy_file(&self, file: &str) -> bool {
        self.copy_files.iter().any(|f| f == file)
    }

    fn multiplexer_or_default(&self) -> &str {
        &self.terminal.multiplexer
    }
}

impl GitConfigExt for crate::core::types::GitConfig {
    fn is_user_configured(&self) -> bool {
        self.user_name.is_some() && self.user_email.is_some()
    }

    fn user_display(&self) -> String {
        self.user_name
            .as_ref()
            .or(self.user_email.as_ref())
            .map(|s| s.to_string())
            .unwrap_or_else(|| "unknown".to_string())
    }

    fn to_env_vars(&self) -> Vec<(String, String)> {
        let mut vars = Vec::new();
        if let Some(name) = &self.user_name {
            vars.push(("GIT_AUTHOR_NAME".to_string(), name.clone()));
            vars.push(("GIT_COMMITTER_NAME".to_string(), name.clone()));
        }
        if let Some(email) = &self.user_email {
            vars.push(("GIT_AUTHOR_EMAIL".to_string(), email.clone()));
            vars.push(("GIT_COMMITTER_EMAIL".to_string(), email.clone()));
        }
        vars
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::executors::MockCommandExecutor;

    #[tokio::test]
    async fn test_command_executor_ext() {
        let mut mock = MockCommandExecutor::new();

        mock.expect_command("echo").with_args(&["hello"]).returns_output("hello", "", 0);

        let result = mock.run_simple("echo", &["hello"]).await.unwrap();
        assert_eq!(result, "hello");
    }

    #[test]
    fn test_worktree_ext() {
        let worktree = Worktree {
            name: "feature".to_string(),
            path: "/repo/.git/phantom/worktrees/feature".into(),
            branch: Some("feature/new-thing".to_string()),
            commit: "abc123def456".to_string(),
            is_bare: false,
            is_detached: false,
            is_locked: false,
            is_prunable: false,
        };

        assert!(!worktree.is_main());
        assert_eq!(worktree.display_name(), "feature/new-thing");
        assert_eq!(worktree.relative_path(), Some("feature".to_string()));
    }

    #[test]
    fn test_str_ext() {
        // Branch-like strings
        assert!("feature-branch".is_branch_like());
        assert!("feature/sub".is_branch_like());
        assert!("v1.0.0".is_branch_like());
        assert!(!"".is_branch_like());
        assert!(!"feature branch".is_branch_like());

        // Commit-like strings
        assert!("abc123def456".is_commit_like());
        assert!("1234567".is_commit_like());
        assert!(!"feature".is_commit_like());
        assert!(!"abc12".is_commit_like()); // Too short

        // Sanitization
        assert_eq!("feature branch!".sanitize_worktree_name(), "feature-branch-");
        assert_eq!("feature@branch".sanitize_worktree_name(), "feature-branch");
        assert_eq!("feature/sub".sanitize_worktree_name(), "feature/sub");
    }

    #[test]
    fn test_phantom_config_ext() {
        use crate::core::types::PhantomConfig;

        let mut config = PhantomConfig::default();
        assert!(!config.has_copy_files());

        // Add files
        config.add_copy_file(".env");
        config.add_copy_file("config.json");
        config.add_copy_file(".env"); // Duplicate, should not be added

        assert!(config.has_copy_files());
        assert_eq!(config.copy_files.len(), 2);
        assert!(config.should_copy_file(".env"));
        assert!(config.should_copy_file("config.json"));
        assert!(!config.should_copy_file("other.txt"));

        // Remove file
        assert!(config.remove_copy_file(".env"));
        assert!(!config.remove_copy_file(".env")); // Already removed
        assert_eq!(config.copy_files.len(), 1);
        assert!(!config.should_copy_file(".env"));

        // Multiplexer
        assert_eq!(config.multiplexer_or_default(), "auto");
    }

    #[test]
    fn test_git_config_ext() {
        use crate::core::types::GitConfig;

        // Empty config
        let config = GitConfig { user_name: None, user_email: None };
        assert!(!config.is_user_configured());
        assert_eq!(config.user_display(), "unknown");
        assert!(config.to_env_vars().is_empty());

        // Partial config (name only)
        let config = GitConfig { user_name: Some("Alice".to_string()), user_email: None };
        assert!(!config.is_user_configured());
        assert_eq!(config.user_display(), "Alice");
        let vars = config.to_env_vars();
        assert_eq!(vars.len(), 2);
        assert!(vars.contains(&("GIT_AUTHOR_NAME".to_string(), "Alice".to_string())));
        assert!(vars.contains(&("GIT_COMMITTER_NAME".to_string(), "Alice".to_string())));

        // Partial config (email only)
        let config =
            GitConfig { user_name: None, user_email: Some("alice@example.com".to_string()) };
        assert!(!config.is_user_configured());
        assert_eq!(config.user_display(), "alice@example.com");
        let vars = config.to_env_vars();
        assert_eq!(vars.len(), 2);
        assert!(vars.contains(&("GIT_AUTHOR_EMAIL".to_string(), "alice@example.com".to_string())));

        // Full config
        let config = GitConfig {
            user_name: Some("Alice".to_string()),
            user_email: Some("alice@example.com".to_string()),
        };
        assert!(config.is_user_configured());
        assert_eq!(config.user_display(), "Alice");
        let vars = config.to_env_vars();
        assert_eq!(vars.len(), 4);
    }
}
