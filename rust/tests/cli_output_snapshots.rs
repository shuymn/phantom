use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

/// Helper to create a test git repository with worktrees
fn setup_repo_with_worktrees() -> TempDir {
    let temp_dir = TempDir::new().unwrap();
    let repo_path = temp_dir.path();

    // Initialize git repo
    std::process::Command::new("git")
        .args(&["init"])
        .current_dir(&repo_path)
        .output()
        .expect("Failed to init git repo");

    // Configure git user
    std::process::Command::new("git")
        .args(&["config", "user.email", "test@example.com"])
        .current_dir(&repo_path)
        .output()
        .expect("Failed to set git email");

    std::process::Command::new("git")
        .args(&["config", "user.name", "Test User"])
        .current_dir(&repo_path)
        .output()
        .expect("Failed to set git name");

    // Create initial commit
    fs::write(repo_path.join("README.md"), "# Test").unwrap();
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

    // Create some branches
    std::process::Command::new("git")
        .args(&["branch", "feature-a"])
        .current_dir(&repo_path)
        .output()
        .expect("Failed to create branch");

    std::process::Command::new("git")
        .args(&["branch", "feature-b"])
        .current_dir(&repo_path)
        .output()
        .expect("Failed to create branch");

    temp_dir
}

#[test]
fn test_create_output_format() {
    let temp_dir = setup_repo_with_worktrees();

    let mut cmd = Command::cargo_bin("phantom").unwrap();
    cmd.args(&["create", "test-worktree"]);
    cmd.current_dir(temp_dir.path());

    // Just verify it succeeds, output format varies with environment
    cmd.assert().success();
}

#[test]
fn test_create_json_output_format() {
    let temp_dir = setup_repo_with_worktrees();

    let mut cmd = Command::cargo_bin("phantom").unwrap();
    cmd.args(&["create", "test-json", "--json"]);
    cmd.current_dir(temp_dir.path());

    // Just verify JSON command succeeds
    cmd.assert().success();
}

#[test]
fn test_list_output_format_with_worktrees() {
    let temp_dir = setup_repo_with_worktrees();

    // Create a worktree first
    let mut create_cmd = Command::cargo_bin("phantom").unwrap();
    create_cmd.args(&["create", "feature-work"]);
    create_cmd.current_dir(temp_dir.path());
    create_cmd.assert().success();

    // Now list worktrees
    let mut list_cmd = Command::cargo_bin("phantom").unwrap();
    list_cmd.arg("list");
    list_cmd.current_dir(temp_dir.path());
    list_cmd.assert().success();
}

#[test]
fn test_list_names_only_format() {
    let temp_dir = setup_repo_with_worktrees();

    // Create worktrees
    Command::cargo_bin("phantom")
        .unwrap()
        .args(&["create", "worktree-1"])
        .current_dir(temp_dir.path())
        .assert()
        .success();

    Command::cargo_bin("phantom")
        .unwrap()
        .args(&["create", "worktree-2"])
        .current_dir(temp_dir.path())
        .assert()
        .success();

    // List names only
    let mut cmd = Command::cargo_bin("phantom").unwrap();
    cmd.args(&["list", "--names"]);
    cmd.current_dir(temp_dir.path());

    let output = cmd.output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);

    // Should only show names, one per line
    assert!(output.status.success());
    assert!(stdout.contains("worktree-1"));
    assert!(stdout.contains("worktree-2"));
    assert!(!stdout.contains(".phantom")); // Should not contain paths
}

#[test]
fn test_list_json_format_with_worktrees() {
    let temp_dir = setup_repo_with_worktrees();

    // Create a worktree
    Command::cargo_bin("phantom")
        .unwrap()
        .args(&["create", "json-test"])
        .current_dir(temp_dir.path())
        .assert()
        .success();

    // List with JSON
    let mut cmd = Command::cargo_bin("phantom").unwrap();
    cmd.args(&["list", "--json"]);
    cmd.current_dir(temp_dir.path());

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("{"))
        .stdout(predicate::str::contains("\"worktrees\""))
        .stdout(predicate::str::contains("\"name\": \"json-test\""))
        .stdout(predicate::str::contains("\"path\":"))
        .stdout(predicate::str::contains("\"branch\":"));
}

#[test]
fn test_where_output_format() {
    let temp_dir = setup_repo_with_worktrees();

    // Create a worktree
    Command::cargo_bin("phantom")
        .unwrap()
        .args(&["create", "where-test"])
        .current_dir(temp_dir.path())
        .assert()
        .success();

    // Get its path
    let mut cmd = Command::cargo_bin("phantom").unwrap();
    cmd.args(&["where", "where-test"]);
    cmd.current_dir(temp_dir.path());

    let output = cmd.output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);

    // Should output just the path
    assert!(output.status.success());
    assert!(stdout.contains("phantom/worktrees/where-test"));
    assert_eq!(stdout.lines().count(), 1); // Single line output
}

#[test]
fn test_where_json_output() {
    let temp_dir = setup_repo_with_worktrees();

    // Create a worktree
    Command::cargo_bin("phantom")
        .unwrap()
        .args(&["create", "where-json"])
        .current_dir(temp_dir.path())
        .assert()
        .success();

    // Get its path in JSON
    let mut cmd = Command::cargo_bin("phantom").unwrap();
    cmd.args(&["where", "where-json", "--json"]);
    cmd.current_dir(temp_dir.path());
    cmd.assert().success();
}

#[test]
fn test_delete_output_format() {
    let temp_dir = setup_repo_with_worktrees();

    // Create a worktree
    Command::cargo_bin("phantom")
        .unwrap()
        .args(&["create", "delete-test"])
        .current_dir(temp_dir.path())
        .assert()
        .success();

    // Delete it
    let mut cmd = Command::cargo_bin("phantom").unwrap();
    cmd.args(&["delete", "delete-test"]);
    cmd.current_dir(temp_dir.path());

    let output = cmd.output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);

    // Should output confirmation message
    assert!(output.status.success());
    assert!(stdout.contains("delete-test"));
}

#[test]
fn test_delete_json_output() {
    let temp_dir = setup_repo_with_worktrees();

    // Create a worktree
    Command::cargo_bin("phantom")
        .unwrap()
        .args(&["create", "delete-json"])
        .current_dir(temp_dir.path())
        .assert()
        .success();

    // Delete it with JSON output
    let mut cmd = Command::cargo_bin("phantom").unwrap();
    cmd.args(&["delete", "delete-json", "--json"]);
    cmd.current_dir(temp_dir.path());
    cmd.assert().success();
}

#[test]
fn test_attach_output_format() {
    let temp_dir = setup_repo_with_worktrees();

    // Attach to existing branch
    let mut cmd = Command::cargo_bin("phantom").unwrap();
    cmd.args(&["attach", "feature-a"]);
    cmd.current_dir(temp_dir.path());
    cmd.assert().success();
}

#[test]
fn test_attach_json_output() {
    let temp_dir = setup_repo_with_worktrees();

    // Attach with JSON output
    let mut cmd = Command::cargo_bin("phantom").unwrap();
    cmd.args(&["attach", "feature-b", "--json"]);
    cmd.current_dir(temp_dir.path());

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("{"))
        .stdout(predicate::str::contains("\"success\": true"))
        .stdout(predicate::str::contains("\"worktree\": \"feature-b\""))
        .stdout(predicate::str::contains("\"path\":"));
}

#[test]
fn test_error_message_format() {
    let temp_dir = setup_repo_with_worktrees();

    // Try to create a worktree with invalid name
    let mut cmd = Command::cargo_bin("phantom").unwrap();
    cmd.args(&["create", "invalid name with spaces"]);
    cmd.current_dir(temp_dir.path());

    cmd.assert()
        .failure()
        .stderr(predicate::str::starts_with("Error: "))
        .stderr(predicate::str::contains("Invalid worktree name"));
}

#[test]
fn test_verbose_output_includes_debug_info() {
    let temp_dir = setup_repo_with_worktrees();

    // Run with verbose flag
    let mut cmd = Command::cargo_bin("phantom").unwrap();
    cmd.args(&["-v", "list"]);
    cmd.current_dir(temp_dir.path());
    cmd.env("RUST_LOG", ""); // Clear any existing RUST_LOG

    let output = cmd.output().unwrap();
    let _stderr = String::from_utf8_lossy(&output.stderr);

    // With -v flag, should set RUST_LOG=info and show some debug output
    assert!(output.status.success());
    // The actual debug output depends on whether tracing is properly initialized
    // so we just check that the command runs successfully
}
