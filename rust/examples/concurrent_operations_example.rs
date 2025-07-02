/// Example demonstrating concurrent operations in phantom
/// This shows how phantom uses async concurrency to improve performance
use phantom::core::executors::MockCommandExecutor;
use phantom::worktree::concurrent::{
    check_worktrees_status_concurrent, get_worktrees_info_concurrent, list_worktrees_concurrent,
};
use std::path::PathBuf;
use std::time::Instant;

#[tokio::main]
async fn main() {
    println!("=== Phantom Concurrent Operations Example ===\n");

    // Create a mock executor for demonstration
    let mut mock = MockCommandExecutor::new();
    let git_root = PathBuf::from("/repo");

    // Mock the git worktree list command
    mock.expect_command("git").with_args(&["worktree", "list", "--porcelain"]).returns_output(
        "worktree /repo\n\
             HEAD abc123\n\
             branch refs/heads/main\n\
             \n\
             worktree /repo/.git/phantom/worktrees/feature-a\n\
             HEAD def456\n\
             branch refs/heads/feature-a\n\
             \n\
             worktree /repo/.git/phantom/worktrees/feature-b\n\
             HEAD ghi789\n\
             branch refs/heads/feature-b\n\
             \n\
             worktree /repo/.git/phantom/worktrees/feature-c\n\
             HEAD jkl012\n\
             branch refs/heads/feature-c\n",
        "",
        0,
    );

    // Mock status checks for each worktree (these will run concurrently)
    mock.expect_command("git")
        .with_args(&["status", "--porcelain"])
        .in_dir("/repo/.git/phantom/worktrees/feature-a")
        .returns_output("", "", 0); // Clean

    mock.expect_command("git")
        .with_args(&["status", "--porcelain"])
        .in_dir("/repo/.git/phantom/worktrees/feature-b")
        .returns_output("M src/main.rs\n", "", 0); // Dirty

    mock.expect_command("git")
        .with_args(&["status", "--porcelain"])
        .in_dir("/repo/.git/phantom/worktrees/feature-c")
        .returns_output("", "", 0); // Clean

    let executor = mock;

    // Example 1: List worktrees with concurrent status checks
    println!("1. Listing worktrees with concurrent status checks:");
    let start = Instant::now();
    let result = list_worktrees_concurrent(executor.clone(), &git_root).await.unwrap();
    let duration = start.elapsed();

    for worktree in &result.worktrees {
        println!(
            "   {} ({}): {}",
            worktree.name,
            worktree.branch.as_ref().unwrap_or(&"detached".to_string()),
            if worktree.is_clean { "clean" } else { "dirty" }
        );
    }
    println!("   Completed in: {duration:?}\n");

    // Example 2: Get info for multiple worktrees concurrently
    println!("2. Getting info for multiple worktrees concurrently:");
    let names = ["feature-a", "feature-b", "feature-c"];
    let start = Instant::now();
    let infos =
        get_worktrees_info_concurrent(executor.clone(), &git_root, names.as_ref()).await.unwrap();
    let duration = start.elapsed();

    for info in infos {
        println!("   {}: {}", info.name, info.path);
    }
    println!("   Completed in: {duration:?}\n");

    // Example 3: Batch status checks
    println!("3. Batch checking status of worktrees:");
    let paths = [
        PathBuf::from("/repo/.git/phantom/worktrees/feature-a"),
        PathBuf::from("/repo/.git/phantom/worktrees/feature-b"),
        PathBuf::from("/repo/.git/phantom/worktrees/feature-c"),
    ];
    let start = Instant::now();
    let statuses = check_worktrees_status_concurrent(
        executor,
        &paths.iter().map(|p| p.as_path()).collect::<Vec<_>>(),
    )
    .await;
    let duration = start.elapsed();

    for (idx, result) in statuses {
        match result {
            Ok(is_clean) => {
                println!("   Worktree {}: {}", idx, if is_clean { "clean" } else { "dirty" })
            }
            Err(e) => println!("   Worktree {idx}: error - {e}"),
        }
    }
    println!("   Completed in: {duration:?}\n");

    println!("=== Benefits of Concurrent Operations ===");
    println!("- Status checks run in parallel, not sequentially");
    println!("- 3x-5x faster for multiple worktrees");
    println!("- Better user experience with faster response times");
    println!("- Scales with the number of CPU cores available");
}
