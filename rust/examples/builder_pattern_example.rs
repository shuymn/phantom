//! Example demonstrating the type-safe WorktreeBuilder pattern
//!
//! The builder pattern with type states ensures compile-time safety:
//! - You must set a name before building
//! - Optional fields can be set in any order
//! - Invalid states are impossible to express

use phantom::git::factory::create_backend_for_dir;
use phantom::worktree::builder::build_worktree;
use std::path::Path;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Example 1: Basic usage - minimal configuration
    println!("Example 1: Basic worktree creation");
    let backend = create_backend_for_dir(Path::new("."));

    let _worktree = build_worktree()
        .name("feature-branch") // Required - transitions to WithName state
        .create(&backend, Path::new("."))
        .await?;

    // Example 2: Full configuration with all options
    println!("\nExample 2: Worktree with all options");
    let _worktree = build_worktree()
        .name("complex-feature")
        .branch("feature/complex-123") // Optional: custom branch name
        .base("develop") // Optional: base commit/branch
        .copy_file(".env") // Optional: copy individual files
        .copy_file("config.json")
        .create(&backend, Path::new("."))
        .await?;

    // Example 3: Using copy_files for batch operations
    println!("\nExample 3: Batch file copying");
    let files_to_copy = vec![".env", "config.json", "secrets.yaml"];

    let _worktree = build_worktree()
        .name("batch-copy-example")
        .copy_files(files_to_copy) // Copy multiple files at once
        .create(&backend, Path::new("."))
        .await?;

    // Example 4: Two-step process with validation
    println!("\nExample 4: Explicit validation step");
    let validated =
        build_worktree().name("validated-feature").branch("feature/validated").validate()?; // Returns Ready state only if valid

    // Now we can access the validated name
    println!("Creating worktree: {}", validated.name());

    let _worktree = validated.create(&backend, Path::new(".")).await?;

    // Example 5: Building options without creating
    println!("\nExample 5: Building options struct");
    let options = build_worktree()
        .name("options-only")
        .branch("feature/options")
        .base("main")
        .copy_files(vec!["package.json", "yarn.lock"])
        .build_unchecked(); // Get CreateWorktreeOptions without validation

    println!(
        "Options: branch={:?}, base={:?}, files={:?}",
        options.branch, options.commitish, options.copy_files
    );

    // Example 6: Type safety demonstration
    println!("\nExample 6: Type safety");

    // This won't compile - no build() method available without name:
    // let _invalid = build_worktree()
    //     .branch("feature")
    //     .build();  // Error: method `build` not found

    // This won't compile - can't call name() after validate():
    // let _invalid = build_worktree()
    //     .name("test")
    //     .validate()?
    //     .name("different");  // Error: method `name` not found

    println!("Type safety prevents invalid states at compile time!");

    Ok(())
}

// Helper function demonstrating how to use the builder in practice
#[allow(dead_code)]
async fn create_feature_worktree(
    name: &str,
    files: Vec<&str>,
) -> Result<(), Box<dyn std::error::Error>> {
    let backend = create_backend_for_dir(Path::new("."));

    let result = build_worktree()
        .name(name)
        .branch(format!("feature/{}", name))
        .copy_files(files)
        .create(&backend, Path::new("."))
        .await?;

    println!("Created worktree: {}", result.message);
    if let Some(copied) = result.copied_files {
        println!("Copied {} files", copied.len());
    }

    Ok(())
}
