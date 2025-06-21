/// Example demonstrating the use of extension traits for better ergonomics
use phantom::cli::context::ProductionContext;
use phantom::core::executors::RealCommandExecutor;
use phantom::core::exit_handler::RealExitHandler;
use phantom::core::extension_traits::{CommandExecutorExt, StrExt, WorktreeExt};
use phantom::core::filesystems::RealFileSystem;
use phantom::core::types::Worktree;
use std::sync::Arc;

#[tokio::main]
async fn main() -> phantom::Result<()> {
    // Create a production context
    let context = ProductionContext {
        executor: RealCommandExecutor,
        filesystem: RealFileSystem::new(),
        exit_handler: RealExitHandler,
    };

    // Example 1: Using CommandExecutorExt for simple commands
    println!("=== CommandExecutorExt Example ===");

    // Simple command execution
    let output = context.executor.run_simple("echo", &["Hello, World!"]).await?;
    println!("Echo output: {}", output);

    // Command in specific directory (would fail without valid git repo)
    // let git_version = context.executor.run_in_dir("git", &["--version"], Path::new("/tmp")).await?;

    // Create GitExecutor from CommandExecutor
    let git = Arc::new(context.executor).git();
    // Now you can use git.run(&["status"]).await

    // Example 2: Using WorktreeExt for worktree convenience methods
    println!("\n=== WorktreeExt Example ===");

    let worktree = Worktree {
        name: "feature-branch".to_string(),
        path: "/repo/.git/phantom/worktrees/feature-branch".into(),
        branch: Some("feature/new-ui".to_string()),
        commit: "abc123def456789".to_string(),
        is_bare: false,
        is_detached: false,
        is_locked: false,
        is_prunable: false,
    };

    println!("Worktree display name: {}", worktree.display_name());
    println!("Is main branch: {}", worktree.is_main());
    println!("Relative path: {:?}", worktree.relative_path());

    // Example with detached HEAD (no branch)
    let detached_worktree = Worktree {
        name: "experiment".to_string(),
        path: "/repo/.git/phantom/worktrees/experiment".into(),
        branch: None,
        commit: "def456abc123789".to_string(),
        is_bare: false,
        is_detached: true,
        is_locked: false,
        is_prunable: false,
    };

    println!("\nDetached worktree display: {}", detached_worktree.display_name());

    // Example 3: Using StrExt for string validation and sanitization
    println!("\n=== StrExt Example ===");

    let inputs =
        vec!["feature-branch", "abc123def", "feature/new-ui", "feature branch!", "v1.0.0", "main"];

    for input in inputs {
        println!("\nInput: '{}'", input);
        println!("  Is branch-like: {}", input.is_branch_like());
        println!("  Is commit-like: {}", input.is_commit_like());
        println!("  Sanitized: '{}'", input.sanitize_worktree_name());
    }

    // Example 4: Practical usage in a handler-like function
    println!("\n=== Practical Usage Example ===");

    // Check if a name is valid before creating a worktree
    let proposed_name = "feature@branch#123";
    if !proposed_name.is_branch_like() {
        let sanitized = proposed_name.sanitize_worktree_name();
        println!("'{}' is not a valid branch name, using '{}' instead", proposed_name, sanitized);
    }

    Ok(())
}

// Example function showing how extension traits improve code readability
async fn check_git_status(executor: &impl CommandExecutor) -> phantom::Result<bool> {
    // Without extension trait:
    // let config = CommandConfig::new("git").with_args(vec!["status".to_string(), "--porcelain".to_string()]);
    // let output = executor.execute(config).await?;
    // Ok(output.stdout.trim().is_empty())

    // With extension trait:
    let output = executor.run_simple("git", &["status", "--porcelain"]).await?;
    Ok(output.is_empty())
}
