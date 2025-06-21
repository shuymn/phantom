/// Example demonstrating arena allocation for batch operations
///
/// This example shows how to use the BatchProcessor to efficiently
/// process large numbers of worktrees with minimal allocation overhead.
use phantom::core::executors::MockCommandExecutor;
use phantom::worktree::batch_processor::BatchProcessor;
use std::path::PathBuf;
use std::sync::Arc;
use typed_arena::Arena;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Example 1: Using the batch processor directly with arenas
    println!("=== Example 1: Direct arena usage ===");
    example_direct_arena_usage().await?;

    // Example 2: Processing specific worktrees
    println!("\n=== Example 2: Processing specific worktrees ===");
    example_specific_worktrees().await?;

    // Example 3: Memory efficiency comparison
    println!("\n=== Example 3: Memory efficiency demonstration ===");
    example_memory_efficiency().await?;

    Ok(())
}

async fn example_direct_arena_usage() -> Result<(), Box<dyn std::error::Error>> {
    // Create arenas for different allocation types
    let info_arena = Arena::new();
    let error_arena = Arena::new();
    let string_arena = Arena::new();

    // Create batch processor
    let processor = BatchProcessor::new(&info_arena, &error_arena, &string_arena);

    // Set up mock executor
    let mut mock = MockCommandExecutor::new();
    mock_worktree_list(&mut mock);
    mock_status_checks(&mut mock);

    // Process worktrees
    let result =
        processor.process_worktrees_with_status(Arc::new(mock), &PathBuf::from("/repo")).await?;

    println!("Processed {} worktrees successfully", result.success.len());
    println!("Failed to process {} worktrees", result.failed.len());

    // All allocations are cleaned up when arenas go out of scope
    // This is more efficient than individual allocations/deallocations

    Ok(())
}

async fn example_specific_worktrees() -> Result<(), Box<dyn std::error::Error>> {
    let info_arena = Arena::new();
    let error_arena = Arena::new();
    let string_arena = Arena::new();

    let processor = BatchProcessor::new(&info_arena, &error_arena, &string_arena);

    // Mock executor for specific worktree processing
    let mut mock = MockCommandExecutor::new();
    mock_specific_worktrees(&mut mock);

    // Process only specific worktrees
    let names = vec!["feature-1", "feature-2", "hotfix-1"];
    let result = processor
        .process_specific_worktrees(Arc::new(mock), &PathBuf::from("/repo"), &names)
        .await?;

    println!(
        "Specific processing: {} success, {} failed",
        result.success.len(),
        result.failed.len()
    );

    for info in result.success {
        println!("  ✓ {} ({})", info.name, if info.is_clean { "clean" } else { "dirty" });
    }

    for (name, error) in result.failed {
        println!("  ✗ {}: {}", name, error);
    }

    Ok(())
}

async fn example_memory_efficiency() -> Result<(), Box<dyn std::error::Error>> {
    // Simulate processing a large number of worktrees
    let worktree_count = 1000;

    println!("Processing {} worktrees to demonstrate memory efficiency...", worktree_count);

    let start = std::time::Instant::now();

    // Arena-based processing
    let info_arena = Arena::new();
    let error_arena = Arena::new();
    let string_arena = Arena::new();

    let processor = BatchProcessor::new(&info_arena, &error_arena, &string_arena);

    // Create mock with many worktrees
    let mut mock = MockCommandExecutor::new();
    mock_large_worktree_list(&mut mock, worktree_count);

    let result =
        processor.process_worktrees_with_status(Arc::new(mock), &PathBuf::from("/repo")).await?;

    let duration = start.elapsed();

    println!("Arena-based processing completed in {:?}", duration);
    println!("  - Processed: {} worktrees", result.success.len());
    println!("  - Memory efficiency: All allocations in contiguous memory blocks");
    println!("  - Deallocation: Single operation when arenas drop");

    // Compare with theoretical Vec-based approach
    println!("\nComparison with Vec-based approach:");
    println!("  - Vec would require {} individual allocations", worktree_count * 3);
    println!("  - Arena uses only 3 memory regions total");
    println!("  - Reduced memory fragmentation and better cache locality");

    Ok(())
}

// Helper functions to set up mocks

fn mock_worktree_list(mock: &mut MockCommandExecutor) {
    mock.expect_command("git").with_args(&["worktree", "list", "--porcelain"]).returns_output(
        "worktree /repo\nHEAD abc123\nbranch refs/heads/main\n\n\
         worktree /repo/.git/phantom/worktrees/feature-1\nHEAD def456\nbranch refs/heads/feature-1\n\n\
         worktree /repo/.git/phantom/worktrees/feature-2\nHEAD ghi789\nbranch refs/heads/feature-2\n\n\
         worktree /repo/.git/phantom/worktrees/hotfix-1\nHEAD jkl012\nbranch refs/heads/hotfix-1\n",
        "",
        0,
    );
}

fn mock_status_checks(mock: &mut MockCommandExecutor) {
    mock.expect_command("git")
        .with_args(&["status", "--porcelain"])
        .in_dir("/repo/.git/phantom/worktrees/feature-1")
        .returns_output("", "", 0);

    mock.expect_command("git")
        .with_args(&["status", "--porcelain"])
        .in_dir("/repo/.git/phantom/worktrees/feature-2")
        .returns_output("M src/main.rs\n", "", 0);

    mock.expect_command("git")
        .with_args(&["status", "--porcelain"])
        .in_dir("/repo/.git/phantom/worktrees/hotfix-1")
        .returns_output("", "", 0);
}

fn mock_specific_worktrees(mock: &mut MockCommandExecutor) {
    // Mock for get_worktree_info calls
    for name in &["feature-1", "feature-2", "hotfix-1"] {
        let path = format!("/repo/.git/phantom/worktrees/{}", name);

        // Mock worktree list for each specific check
        mock.expect_command("git").with_args(&["worktree", "list", "--porcelain"]).returns_output(
            &format!(
                "worktree /repo\nHEAD abc123\nbranch refs/heads/main\n\n\
                 worktree {}\nHEAD def456\nbranch refs/heads/{}\n",
                path, name
            ),
            "",
            0,
        );

        // Mock status check
        mock.expect_command("git")
            .with_args(&["status", "--porcelain"])
            .in_dir(&path)
            .returns_output("", "", 0);
    }
}

fn mock_large_worktree_list(mock: &mut MockCommandExecutor, count: usize) {
    let mut output = String::from("worktree /repo\nHEAD abc123\nbranch refs/heads/main\n");

    for i in 0..count {
        output.push_str(&format!(
            "\nworktree /repo/.git/phantom/worktrees/feature-{}\nHEAD def{:03}\nbranch refs/heads/feature-{}\n",
            i, i, i
        ));
    }

    mock.expect_command("git")
        .with_args(&["worktree", "list", "--porcelain"])
        .returns_output(&output, "", 0);

    // Mock status checks for all worktrees
    for i in 0..count {
        mock.expect_command("git")
            .with_args(&["status", "--porcelain"])
            .in_dir(&format!("/repo/.git/phantom/worktrees/feature-{}", i))
            .returns_output("", "", 0);
    }
}
