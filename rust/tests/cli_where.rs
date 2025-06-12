use phantom::cli::commands::where_cmd::WhereArgs;
use phantom::cli::handlers::where_cmd;
use std::env;
use tempfile::TempDir;
use tokio;

#[tokio::test]
async fn test_where_command_basic() {
    // Create a temporary git repository
    let temp_dir = TempDir::new().unwrap();
    let repo_path = temp_dir.path();

    // Initialize git repo
    std::process::Command::new("git")
        .args(&["init"])
        .current_dir(&repo_path)
        .output()
        .expect("Failed to init git repo");

    // Create initial commit
    std::fs::write(repo_path.join("README.md"), "# Test").unwrap();
    std::process::Command::new("git")
        .args(&["add", "."])
        .current_dir(&repo_path)
        .output()
        .expect("Failed to add files");
    std::process::Command::new("git")
        .args(&["commit", "-m", "Initial commit"])
        .current_dir(&repo_path)
        .output()
        .expect("Failed to commit");

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
            "test-where",
            phantom_dir.join("test-where").to_str().unwrap(),
        ])
        .current_dir(&repo_path)
        .output()
        .expect("Failed to create worktree");

    // Test where command
    let args = WhereArgs { name: Some("test-where".to_string()), fzf: false, json: false };

    let result = where_cmd::handle(args).await;
    assert!(result.is_ok(), "Where command failed: {:?}", result);
}

#[tokio::test]
async fn test_where_command_not_found() {
    // Create a temporary git repository
    let temp_dir = TempDir::new().unwrap();
    let repo_path = temp_dir.path();

    // Initialize git repo
    std::process::Command::new("git")
        .args(&["init"])
        .current_dir(&repo_path)
        .output()
        .expect("Failed to init git repo");

    // Create initial commit
    std::fs::write(repo_path.join("README.md"), "# Test").unwrap();
    std::process::Command::new("git")
        .args(&["add", "."])
        .current_dir(&repo_path)
        .output()
        .expect("Failed to add files");
    std::process::Command::new("git")
        .args(&["commit", "-m", "Initial commit"])
        .current_dir(&repo_path)
        .output()
        .expect("Failed to commit");

    // Change to the repo directory
    env::set_current_dir(&repo_path).unwrap();

    // Test where command with non-existent worktree
    let args = WhereArgs { name: Some("non-existent".to_string()), fzf: false, json: false };

    let result = where_cmd::handle(args).await;
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
async fn test_where_command_validation() {
    // Test missing args
    let args = WhereArgs { name: None, fzf: false, json: false };
    let result = where_cmd::handle(args).await;
    assert!(result.is_err(), "Expected validation error");

    match result.unwrap_err() {
        phantom::PhantomError::Validation(_) => {}
        _ => panic!("Expected Validation error"),
    }

    // Test both name and fzf
    let args = WhereArgs { name: Some("test".to_string()), fzf: true, json: false };
    let result = where_cmd::handle(args).await;
    assert!(result.is_err(), "Expected validation error");

    match result.unwrap_err() {
        phantom::PhantomError::Validation(_) => {}
        _ => panic!("Expected Validation error"),
    }
}
