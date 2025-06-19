use phantom::core::command_executor::{CommandConfig, CommandExecutor};
use phantom::core::executors::{MockCommandExecutor, RealCommandExecutor};
use std::path::Path;
use std::sync::Arc;

// Example: Refactored Git operations using CommandExecutor
struct GitOperations {
    executor: Arc<dyn CommandExecutor>,
    cwd: Option<String>,
}

impl GitOperations {
    fn new(executor: Arc<dyn CommandExecutor>) -> Self {
        Self { executor, cwd: None }
    }

    fn with_cwd(mut self, cwd: impl Into<String>) -> Self {
        self.cwd = Some(cwd.into());
        self
    }

    async fn status(&self) -> Result<String, Box<dyn std::error::Error>> {
        let mut config = CommandConfig::new("git")
            .with_args(vec!["status".to_string(), "--short".to_string()]);

        if let Some(ref cwd) = self.cwd {
            config = config.with_cwd(cwd.into());
        }

        let output = self.executor.execute(config).await?;

        if output.success() {
            Ok(output.stdout)
        } else {
            Err(format!("Git status failed: {}", output.stderr).into())
        }
    }

    async fn add_worktree(
        &self,
        path: &str,
        branch: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut config = CommandConfig::new("git").with_args(vec![
            "worktree".to_string(),
            "add".to_string(),
            "-b".to_string(),
            branch.to_string(),
            path.to_string(),
        ]);

        if let Some(ref cwd) = self.cwd {
            config = config.with_cwd(cwd.into());
        }

        let output = self.executor.execute(config).await?;

        if output.success() {
            Ok(())
        } else {
            Err(format!("Failed to add worktree: {}", output.stderr).into())
        }
    }

    async fn list_worktrees(&self) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        let mut config = CommandConfig::new("git")
            .with_args(vec!["worktree".to_string(), "list".to_string()]);

        if let Some(ref cwd) = self.cwd {
            config = config.with_cwd(cwd.into());
        }

        let output = self.executor.execute(config).await?;

        if output.success() {
            Ok(output.stdout.lines().map(String::from).collect())
        } else {
            Err(format!("Failed to list worktrees: {}", output.stderr).into())
        }
    }
}

#[tokio::main]
async fn main() {
    println!("=== Git Operations with CommandExecutor ===\n");

    // Test scenario with mocks
    println!("1. Testing git operations with mocks:");
    
    let mut mock = MockCommandExecutor::new();
    
    // Set up expectations
    mock.expect_command("git")
        .with_args(&["status", "--short"])
        .returns_output("M  file.txt\n", "", 0);
    
    mock.expect_command("git")
        .with_args(&["worktree", "add", "-b", "feature/test", "phantoms/feature-test"])
        .returns_success();
    
    mock.expect_command("git")
        .with_args(&["worktree", "list"])
        .returns_output("/path/to/main  abc123 [main]\n/path/to/feature-test  def456 [feature/test]\n", "", 0);

    let git = GitOperations::new(Arc::new(mock));
    
    // Test status
    match git.status().await {
        Ok(status) => println!("  Status: {}", status.trim()),
        Err(e) => println!("  Status error: {}", e),
    }
    
    // Test add worktree
    match git.add_worktree("phantoms/feature-test", "feature/test").await {
        Ok(()) => println!("  ✓ Worktree added successfully"),
        Err(e) => println!("  ✗ Add worktree error: {}", e),
    }
    
    // Test list worktrees
    match git.list_worktrees().await {
        Ok(worktrees) => {
            println!("  Worktrees:");
            for wt in worktrees {
                println!("    - {}", wt);
            }
        }
        Err(e) => println!("  List error: {}", e),
    }

    // Production scenario (commented out to avoid actual git commands)
    println!("\n2. Production usage pattern:");
    println!("   In production, you would use:");
    println!("   let git = GitOperations::new(Arc::new(RealCommandExecutor::new()));");
    println!("   // Then use the same API as above");

    println!("\n=== Example Complete ===");
}