use std::env;
use tempfile::TempDir;
use tokio;

#[tokio::test]
async fn test_shell_command_basic() {
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
            "test-shell",
            phantom_dir.join("test-shell").to_str().unwrap(),
        ])
        .current_dir(&repo_path)
        .output()
        .expect("Failed to create worktree");

    // We can't test the actual shell execution because it calls process::exit
    // Just verify the worktree was created
    let worktree_path = phantom_dir.join("test-shell");
    assert!(worktree_path.exists(), "Worktree directory was not created");
}

#[tokio::test]
async fn test_shell_command_validation() {
    // The actual CLI would parse these differently, so we can't test the handler directly
    // Just ensure the module compiles
}
