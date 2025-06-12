use phantom::cli::commands::list::ListArgs;
use phantom::cli::handlers::list;
use phantom::worktree::create::create_worktree;
use phantom::worktree::types::CreateWorktreeOptions;
use std::env;
use tempfile::TempDir;
use tokio;

#[tokio::test]
async fn test_list_command_empty() {
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

    // List worktrees (should be empty)
    let args = ListArgs { fzf: false, names: false, json: false };

    let result = list::handle(args).await;
    assert!(result.is_ok(), "List command failed: {:?}", result);
}

#[tokio::test]
async fn test_list_command_with_worktrees() {
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

    // Create some worktrees
    let options1 =
        CreateWorktreeOptions { branch: Some("feature-1".to_string()), ..Default::default() };
    create_worktree(repo_path, "feature-1", options1).await.unwrap();

    let options2 =
        CreateWorktreeOptions { branch: Some("feature-2".to_string()), ..Default::default() };
    create_worktree(repo_path, "feature-2", options2).await.unwrap();

    // Change to the repo directory
    env::set_current_dir(&repo_path).unwrap();

    // List worktrees
    let args = ListArgs { fzf: false, names: false, json: false };

    let result = list::handle(args).await;
    assert!(result.is_ok(), "List command failed: {:?}", result);
}

#[tokio::test]
async fn test_list_command_names_only() {
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

    // Create a worktree
    let options =
        CreateWorktreeOptions { branch: Some("test-branch".to_string()), ..Default::default() };
    create_worktree(repo_path, "test-branch", options).await.unwrap();

    // Change to the repo directory
    env::set_current_dir(&repo_path).unwrap();

    // List worktrees with names only
    let args = ListArgs { fzf: false, names: true, json: false };

    let result = list::handle(args).await;
    assert!(result.is_ok(), "List command failed: {:?}", result);
}

#[tokio::test]
async fn test_list_command_json_output() {
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

    // Create some worktrees
    let options1 =
        CreateWorktreeOptions { branch: Some("feature-1".to_string()), ..Default::default() };
    create_worktree(repo_path, "feature-1", options1).await.unwrap();

    let options2 =
        CreateWorktreeOptions { branch: Some("feature-2".to_string()), ..Default::default() };
    create_worktree(repo_path, "feature-2", options2).await.unwrap();

    // Change to the repo directory
    env::set_current_dir(&repo_path).unwrap();

    // List worktrees with JSON output
    let args = ListArgs { fzf: false, names: false, json: true };

    let result = list::handle(args).await;
    assert!(result.is_ok(), "List command failed: {:?}", result);
}

#[tokio::test]
async fn test_list_command_json_empty() {
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

    // List worktrees with JSON output (should be empty)
    let args = ListArgs { fzf: false, names: false, json: true };

    let result = list::handle(args).await;
    assert!(result.is_ok(), "List command failed: {:?}", result);
}
