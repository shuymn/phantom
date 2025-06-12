mod common;

use phantom::cli::commands::delete::DeleteArgs;
use phantom::cli::handlers::delete;
use std::env;
use tempfile::TempDir;
use tokio;

#[tokio::test]
async fn test_delete_command_basic() {
    // Create a temporary git repository
    let temp_dir = TempDir::new().unwrap();
    let repo_path = temp_dir.path();

    // Initialize git repo and create initial commit
    common::init_test_repo(&repo_path);
    common::create_initial_commit(&repo_path);

    // Change to the repo directory
    env::set_current_dir(&repo_path).unwrap();

    // Create a worktree using git directly
    let phantom_dir = repo_path.join(".git").join("phantom").join("worktrees");
    std::fs::create_dir_all(&phantom_dir).unwrap();

    std::process::Command::new("git")
        .args(&[
            "worktree",
            "add",
            "-b",
            "test-delete",
            phantom_dir.join("test-delete").to_str().unwrap(),
        ])
        .current_dir(&repo_path)
        .output()
        .expect("Failed to create worktree");

    // Test delete command
    let args = DeleteArgs {
        name: Some("test-delete".to_string()),
        force: false,
        current: false,
        fzf: false,
        json: false,
    };

    let result = delete::handle(args).await;
    assert!(result.is_ok(), "Delete command failed: {:?}", result);

    // Verify the worktree was deleted
    let worktree_path = phantom_dir.join("test-delete");
    assert!(!worktree_path.exists(), "Worktree directory still exists");
}

#[tokio::test]
async fn test_delete_command_with_changes() {
    // Create a temporary git repository
    let temp_dir = TempDir::new().unwrap();
    let repo_path = temp_dir.path();

    // Initialize git repo and create initial commit
    common::init_test_repo(&repo_path);
    common::create_initial_commit(&repo_path);

    // Change to the repo directory
    env::set_current_dir(&repo_path).unwrap();

    // Create a worktree
    let phantom_dir = repo_path.join(".git").join("phantom").join("worktrees");
    std::fs::create_dir_all(&phantom_dir).unwrap();

    std::process::Command::new("git")
        .args(&[
            "worktree",
            "add",
            "-b",
            "test-changes",
            phantom_dir.join("test-changes").to_str().unwrap(),
        ])
        .current_dir(&repo_path)
        .output()
        .expect("Failed to create worktree");

    // Make changes in the worktree
    let worktree_path = phantom_dir.join("test-changes");
    std::fs::write(worktree_path.join("README.md"), "# Modified").unwrap();

    // Test delete command without force - should fail
    let args = DeleteArgs {
        name: Some("test-changes".to_string()),
        force: false,
        current: false,
        fzf: false,
        json: false,
    };

    let result = delete::handle(args).await;
    assert!(result.is_err(), "Expected error for uncommitted changes");

    // Test delete command with force - should succeed
    let args = DeleteArgs {
        name: Some("test-changes".to_string()),
        force: true,
        current: false,
        fzf: false,
        json: false,
    };

    let result = delete::handle(args).await;
    assert!(result.is_ok(), "Delete with force failed: {:?}", result);

    // Verify the worktree was deleted
    assert!(!worktree_path.exists(), "Worktree directory still exists");
}

#[tokio::test]
async fn test_delete_command_not_found() {
    // Create a temporary git repository
    let temp_dir = TempDir::new().unwrap();
    let repo_path = temp_dir.path();

    // Initialize git repo and create initial commit
    common::init_test_repo(&repo_path);
    common::create_initial_commit(&repo_path);

    // Change to the repo directory
    env::set_current_dir(&repo_path).unwrap();

    // Test delete command with non-existent worktree
    let args = DeleteArgs {
        name: Some("non-existent".to_string()),
        force: false,
        current: false,
        fzf: false,
        json: false,
    };

    let result = delete::handle(args).await;
    assert!(result.is_err(), "Expected error for non-existent worktree");

    match result.unwrap_err() {
        phantom::PhantomError::WorktreeNotFound { name } => {
            assert_eq!(name, "non-existent");
        }
        phantom::PhantomError::Worktree(msg) => {
            assert!(msg.contains("non-existent"));
        }
        e => panic!("Expected WorktreeNotFound or Worktree error, got: {:?}", e),
    }
}

#[tokio::test]
async fn test_delete_command_validation() {
    // Test missing args
    let args = DeleteArgs { name: None, force: false, current: false, fzf: false, json: false };
    let result = delete::handle(args).await;
    assert!(result.is_err(), "Expected validation error");

    match result.unwrap_err() {
        phantom::PhantomError::Validation(_) => {}
        _ => panic!("Expected Validation error"),
    }

    // Test both name and current
    let args = DeleteArgs {
        name: Some("test".to_string()),
        force: false,
        current: true,
        fzf: false,
        json: false,
    };
    let result = delete::handle(args).await;
    assert!(result.is_err(), "Expected validation error");

    match result.unwrap_err() {
        phantom::PhantomError::Validation(_) => {}
        _ => panic!("Expected Validation error"),
    }

    // Test both name and fzf
    let args = DeleteArgs {
        name: Some("test".to_string()),
        force: false,
        current: false,
        fzf: true,
        json: false,
    };
    let result = delete::handle(args).await;
    assert!(result.is_err(), "Expected validation error");

    match result.unwrap_err() {
        phantom::PhantomError::Validation(_) => {}
        _ => panic!("Expected Validation error"),
    }
}
