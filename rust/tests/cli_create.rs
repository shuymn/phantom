use phantom::cli::commands::create::CreateArgs;
use phantom::cli::handlers::create;
use std::env;
use tempfile::TempDir;
use tokio;

#[tokio::test]
async fn test_create_command_basic() {
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

    // Create the worktree using the CLI handler
    let args = CreateArgs {
        name: "test-worktree".to_string(),
        branch: Some("test-branch".to_string()),
        base: None,
        copy_files: None,
        shell: false,
        tmux: false,
        tmux_vertical: false,
        tmux_v: false,
        tmux_horizontal: false,
        tmux_h: false,
        kitty: false,
        kitty_vertical: false,
        kitty_v: false,
        kitty_horizontal: false,
        kitty_h: false,
        exec: None,
        json: false,
    };

    let result = create::handle(args).await;
    assert!(result.is_ok(), "Create command failed: {:?}", result);

    // Verify the worktree was created
    let worktree_path =
        repo_path.join(".git").join("phantom").join("worktrees").join("test-worktree");
    assert!(worktree_path.exists(), "Worktree directory was not created");
    assert!(worktree_path.join(".git").exists(), "Worktree .git file was not created");
}

#[tokio::test]
async fn test_create_command_with_copy_files() {
    // Create a temporary git repository
    let temp_dir = TempDir::new().unwrap();
    let repo_path = temp_dir.path();

    // Initialize git repo
    std::process::Command::new("git")
        .args(&["init"])
        .current_dir(&repo_path)
        .output()
        .expect("Failed to init git repo");

    // Create files to copy
    std::fs::write(repo_path.join("config.json"), r#"{"test": true}"#).unwrap();
    std::fs::write(repo_path.join(".env"), "TEST=value").unwrap();

    // Create initial commit
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

    // Create the worktree with copy files
    let args = CreateArgs {
        name: "test-worktree-copy".to_string(),
        branch: None,
        base: None,
        copy_files: Some(vec!["config.json".to_string(), ".env".to_string()]),
        shell: false,
        tmux: false,
        tmux_vertical: false,
        tmux_v: false,
        tmux_horizontal: false,
        tmux_h: false,
        kitty: false,
        kitty_vertical: false,
        kitty_v: false,
        kitty_horizontal: false,
        kitty_h: false,
        exec: None,
        json: false,
    };

    let result = create::handle(args).await;
    assert!(result.is_ok(), "Create command failed: {:?}", result);

    // Verify the worktree was created and files were copied
    let worktree_path =
        repo_path.join(".git").join("phantom").join("worktrees").join("test-worktree-copy");
    assert!(worktree_path.exists(), "Worktree directory was not created");
    assert!(worktree_path.join("config.json").exists(), "config.json was not copied");
    assert!(worktree_path.join(".env").exists(), ".env was not copied");

    // Verify file contents
    let config_content = std::fs::read_to_string(worktree_path.join("config.json")).unwrap();
    assert_eq!(config_content, r#"{"test": true}"#);
    let env_content = std::fs::read_to_string(worktree_path.join(".env")).unwrap();
    assert_eq!(env_content, "TEST=value");
}

#[tokio::test]
async fn test_create_command_json_output() {
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

    // Create the worktree with JSON output
    let args = CreateArgs {
        name: "test-worktree-json".to_string(),
        branch: Some("json-branch".to_string()),
        base: None,
        copy_files: None,
        shell: false,
        tmux: false,
        tmux_vertical: false,
        tmux_v: false,
        tmux_horizontal: false,
        tmux_h: false,
        kitty: false,
        kitty_vertical: false,
        kitty_v: false,
        kitty_horizontal: false,
        kitty_h: false,
        exec: None,
        json: true,
    };

    // We can't easily capture the JSON output in this test, but we can verify it doesn't error
    let result = create::handle(args).await;
    assert!(result.is_ok(), "Create command with JSON output failed: {:?}", result);

    // Verify the worktree was created
    let worktree_path =
        repo_path.join(".git").join("phantom").join("worktrees").join("test-worktree-json");
    assert!(worktree_path.exists(), "Worktree directory was not created");
}
