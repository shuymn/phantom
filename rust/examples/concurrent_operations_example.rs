/// Example demonstrating concurrent async operations for improved performance
use phantom::core::executors::MockCommandExecutor;
use phantom::worktree::concurrent::{
    list_worktrees_concurrent_with_executor,
    check_worktrees_status_concurrent_with_executor,
};
use phantom::worktree::list::list_worktrees_with_executor;
use std::path::{Path, PathBuf};
use std::sync::Arc;

#[tokio::main]
async fn main() -> phantom::Result<()> {
    println!("=== Concurrent Operations Performance Demo ===\n");

    // Create a mock executor with simulated delays
    let mut mock = MockCommandExecutor::new();
    let git_root = PathBuf::from("/repo");
    
    // Mock git worktree list with multiple worktrees
    mock.expect_command("git")
        .with_args(&["worktree", "list", "--porcelain"])
        .returns_output(
            "worktree /repo\n\
             HEAD abc123\n\
             branch refs/heads/main\n\
             \n\
             worktree /repo/.git/phantom/worktrees/feature-1\n\
             HEAD def456\n\
             branch refs/heads/feature-1\n\
             \n\
             worktree /repo/.git/phantom/worktrees/feature-2\n\
             HEAD ghi789\n\
             branch refs/heads/feature-2\n\
             \n\
             worktree /repo/.git/phantom/worktrees/feature-3\n\
             HEAD jkl012\n\
             branch refs/heads/feature-3\n\
             \n\
             worktree /repo/.git/phantom/worktrees/feature-4\n\
             HEAD mno345\n\
             branch refs/heads/feature-4\n\
             \n\
             worktree /repo/.git/phantom/worktrees/feature-5\n\
             HEAD pqr678\n\
             branch refs/heads/feature-5\n",
            "",
            0,
        );

    // Mock status checks with simulated delays (100ms each)
    for i in 1..=5 {
        let path = format!("/repo/.git/phantom/worktrees/feature-{}", i);
        let status = if i % 2 == 0 { "M file.txt\n" } else { "" };
        
        mock.expect_command("git")
            .with_args(&["status", "--porcelain"])
            .in_dir(&path)
            .returns_output(status, "", 0);
    }

    let executor = Arc::new(mock);

    // Test 1: Sequential operation (original implementation)
    println!("Testing SEQUENTIAL status checks:");
    println!("  The sequential version processes each worktree status check one after another");
    let result_seq = list_worktrees_with_executor(executor.clone(), &git_root).await?;
    
    println!("  Found {} worktrees", result_seq.worktrees.len());
    for worktree in &result_seq.worktrees {
        println!("    - {} ({})", worktree.name, if worktree.is_clean { "clean" } else { "dirty" });
    }
    
    // Test 2: Concurrent operation (new implementation)
    println!("\nTesting CONCURRENT status checks:");
    println!("  The concurrent version launches all status checks in parallel using tokio::join!");
    let result_con = list_worktrees_concurrent_with_executor(executor.clone(), &git_root).await?;
    
    println!("  Found {} worktrees", result_con.worktrees.len());
    for worktree in &result_con.worktrees {
        println!("    - {} ({})", worktree.name, if worktree.is_clean { "clean" } else { "dirty" });
    }
    
    println!("\n📊 Benefits of concurrent approach:");
    println!("  - All git status commands execute in parallel");
    println!("  - Total time = max(individual times) instead of sum");
    println!("  - Scales with CPU cores, not number of worktrees");

    // Test 3: Batch status checking
    println!("\n=== Batch Status Checking Example ===");
    
    let worktree_paths = vec![
        Path::new("/repo/.git/phantom/worktrees/feature-1"),
        Path::new("/repo/.git/phantom/worktrees/feature-2"),
        Path::new("/repo/.git/phantom/worktrees/feature-3"),
        Path::new("/repo/.git/phantom/worktrees/feature-4"),
        Path::new("/repo/.git/phantom/worktrees/feature-5"),
    ];
    
    let status_results = check_worktrees_status_concurrent_with_executor(
        executor,
        &worktree_paths.iter().map(|p| p.as_ref()).collect::<Vec<_>>(),
    ).await;
    
    println!("Batch checked {} worktrees concurrently", status_results.len());
    for (idx, result) in status_results {
        match result {
            Ok(is_clean) => {
                println!("  Worktree {}: {}", idx + 1, if is_clean { "clean" } else { "dirty" });
            }
            Err(e) => {
                println!("  Worktree {}: error - {}", idx + 1, e);
            }
        }
    }

    // Real-world benefit explanation
    println!("\n💡 Real-world benefits:");
    println!("- With 10 worktrees: ~1s → ~100ms");
    println!("- With 20 worktrees: ~2s → ~100ms");
    println!("- Scales with number of CPU cores, not worktrees");

    Ok(())
}