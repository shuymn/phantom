use phantom::cli::commands::attach::AttachArgs;
use phantom::cli::handlers::attach;
use std::env;
use tempfile::TempDir;
use tokio;

#[tokio::test]
async fn test_attach_command_basic() {
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

    // Create a branch
    std::process::Command::new("git")
        .args(&["checkout", "-b", "existing-branch"])
        .current_dir(&repo_path)
        .output()
        .expect("Failed to create branch");

    // Switch back to main
    std::process::Command::new("git")
        .args(&["checkout", "main"])
        .current_dir(&repo_path)
        .output()
        .expect("Failed to switch to main");

    // Change to the repo directory
    env::set_current_dir(&repo_path).unwrap();

    // Attach to the existing branch
    let args = AttachArgs { branch: "existing-branch".to_string(), shell: false, exec: None };

    let result = attach::handle(args).await;
    assert!(result.is_ok(), "Attach command failed: {:?}", result);

    // Verify the worktree was created
    let worktree_path =
        repo_path.join(".git").join("phantom").join("worktrees").join("existing-branch");
    assert!(worktree_path.exists(), "Worktree directory was not created");
    assert!(worktree_path.join(".git").exists(), "Worktree .git file was not created");
}

#[tokio::test]
async fn test_attach_command_branch_not_found() {
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

    // Try to attach to a non-existent branch
    let args = AttachArgs { branch: "non-existent-branch".to_string(), shell: false, exec: None };

    let result = attach::handle(args).await;
    assert!(result.is_err(), "Expected error for non-existent branch");

    match result.unwrap_err() {
        phantom::PhantomError::BranchNotFound { branch } => {
            assert_eq!(branch, "non-existent-branch");
        }
        _ => panic!("Expected BranchNotFound error"),
    }
}

#[tokio::test]
async fn test_attach_command_already_exists() {
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

    // Create a branch
    std::process::Command::new("git")
        .args(&["checkout", "-b", "existing-branch"])
        .current_dir(&repo_path)
        .output()
        .expect("Failed to create branch");

    // Switch back to main
    std::process::Command::new("git")
        .args(&["checkout", "main"])
        .current_dir(&repo_path)
        .output()
        .expect("Failed to switch to main");

    // Change to the repo directory
    env::set_current_dir(&repo_path).unwrap();

    // Attach to the existing branch
    let args = AttachArgs { branch: "existing-branch".to_string(), shell: false, exec: None };

    // First attach should succeed
    let result = attach::handle(args).await;
    assert!(result.is_ok(), "First attach failed: {:?}", result);

    // Second attach should fail
    let args2 = AttachArgs { branch: "existing-branch".to_string(), shell: false, exec: None };
    let result = attach::handle(args2).await;
    assert!(result.is_err(), "Expected error for already existing worktree");

    match result.unwrap_err() {
        phantom::PhantomError::WorktreeExists { name } => {
            assert_eq!(name, "existing-branch");
        }
        _ => panic!("Expected WorktreeExists error"),
    }
}
