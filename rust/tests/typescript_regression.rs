use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

/// Helper to create a test git repository
fn setup_git_repo() -> TempDir {
    let temp_dir = TempDir::new().unwrap();
    let repo_path = temp_dir.path();

    // Initialize git repo
    std::process::Command::new("git")
        .args(["init"])
        .current_dir(repo_path)
        .output()
        .expect("Failed to init git repo");

    // Configure git user
    std::process::Command::new("git")
        .args(["config", "user.email", "test@example.com"])
        .current_dir(repo_path)
        .output()
        .expect("Failed to set git email");

    std::process::Command::new("git")
        .args(["config", "user.name", "Test User"])
        .current_dir(repo_path)
        .output()
        .expect("Failed to set git name");

    // Create initial commit
    fs::write(repo_path.join("README.md"), "# Test").unwrap();
    std::process::Command::new("git")
        .args(["add", "."])
        .current_dir(repo_path)
        .output()
        .expect("Failed to add files");
    std::process::Command::new("git")
        .args(["commit", "-m", "Initial commit"])
        .current_dir(repo_path)
        .output()
        .expect("Failed to commit");

    temp_dir
}

#[test]
fn test_typescript_compat_phantom_directory_structure() {
    // TypeScript version creates worktrees in .git/phantom/worktrees/
    let temp_dir = setup_git_repo();
    let repo_path = temp_dir.path();

    Command::cargo_bin("phantom")
        .unwrap()
        .args(["create", "test-structure"])
        .current_dir(repo_path)
        .assert()
        .success();

    // Check directory structure matches TypeScript version
    let phantom_dir = repo_path.join(".git/phantom/worktrees/test-structure");
    assert!(phantom_dir.exists());
    assert!(phantom_dir.is_dir());
}

#[test]
fn test_typescript_compat_branch_naming() {
    // TypeScript version creates branch with same name as worktree if not specified
    let temp_dir = setup_git_repo();
    let repo_path = temp_dir.path();

    Command::cargo_bin("phantom")
        .unwrap()
        .args(["create", "feature-xyz"])
        .current_dir(repo_path)
        .assert()
        .success();

    // Check branch was created with same name
    let output = std::process::Command::new("git")
        .args(["branch", "--list"])
        .current_dir(repo_path)
        .output()
        .expect("Failed to list branches");

    let branches = String::from_utf8_lossy(&output.stdout);
    assert!(branches.contains("feature-xyz"));
}

#[test]
fn test_typescript_compat_custom_branch_name() {
    // TypeScript version allows specifying different branch name
    let temp_dir = setup_git_repo();
    let repo_path = temp_dir.path();

    Command::cargo_bin("phantom")
        .unwrap()
        .args(["create", "my-worktree", "--branch", "custom-branch"])
        .current_dir(repo_path)
        .assert()
        .success();

    // Check branch was created with custom name
    let output = std::process::Command::new("git")
        .args(["branch", "--list"])
        .current_dir(repo_path)
        .output()
        .expect("Failed to list branches");

    let branches = String::from_utf8_lossy(&output.stdout);
    assert!(branches.contains("custom-branch"));
}

#[test]
fn test_typescript_compat_validation_rules() {
    // TypeScript version validates worktree names
    let temp_dir = setup_git_repo();
    let repo_path = temp_dir.path();

    // Valid names (should pass)
    let valid_names = [
        "feature-123",
        "bugfix_456",
        "test.branch",
        "user/feature",
        "UPPERCASE",
        "MixedCase",
        "with-multiple-dashes",
        "with_multiple_underscores",
        "123-starts-with-number",
    ];

    for name in &valid_names {
        Command::cargo_bin("phantom")
            .unwrap()
            .args(["create", name])
            .current_dir(repo_path)
            .assert()
            .success();
    }

    // Invalid names (should fail)
    let invalid_names = [
        "",
        "with spaces",
        "with\ttab",
        "with\nnewline",
        "with@special",
        "with#hash",
        "with$dollar",
        "with%percent",
        "with^caret",
        "with&ampersand",
        "with*asterisk",
        "with(parens)",
        "with[brackets]",
        "with{braces}",
        "with|pipe",
        "with\\backslash",
        "with:colon",
        "with;semicolon",
        "with'quote",
        "with\"doublequote",
        "with<less",
        "with>greater",
        "with?question",
        "with!exclamation",
    ];

    for name in &invalid_names {
        Command::cargo_bin("phantom")
            .unwrap()
            .args(["create", name])
            .current_dir(repo_path)
            .assert()
            .failure()
            .code(2); // VALIDATION_ERROR
    }
}

#[test]
fn test_typescript_compat_list_output_empty() {
    // TypeScript version outputs "No worktrees found" when empty
    let temp_dir = setup_git_repo();
    let repo_path = temp_dir.path();

    Command::cargo_bin("phantom")
        .unwrap()
        .arg("list")
        .current_dir(repo_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("No worktrees found"));
}

#[test]
fn test_typescript_compat_json_empty_array() {
    // TypeScript version outputs empty array for JSON when no worktrees
    let temp_dir = setup_git_repo();
    let repo_path = temp_dir.path();

    let output = Command::cargo_bin("phantom")
        .unwrap()
        .args(["list", "--json"])
        .current_dir(repo_path)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let json_str = String::from_utf8_lossy(&output);
    if !json_str.is_empty() {
        assert!(json_str.contains("[]") || json_str.contains("\"worktrees\": []"));
    }
}

#[test]
fn test_typescript_compat_delete_force_flag() {
    // TypeScript version supports --force flag for delete
    let temp_dir = setup_git_repo();
    let repo_path = temp_dir.path();

    // Create a worktree
    Command::cargo_bin("phantom")
        .unwrap()
        .args(["create", "to-delete"])
        .current_dir(repo_path)
        .assert()
        .success();

    // Make changes in the worktree
    let worktree_path = repo_path.join(".git/phantom/worktrees/to-delete");
    fs::write(worktree_path.join("new-file.txt"), "changes").unwrap();

    // Delete with force flag
    Command::cargo_bin("phantom")
        .unwrap()
        .args(["delete", "to-delete", "--force"])
        .current_dir(repo_path)
        .assert()
        .success();

    // Verify it's deleted
    assert!(!worktree_path.exists());
}

#[test]
fn test_typescript_compat_create_base_option() {
    // TypeScript version supports --base option for create
    // NOTE: Current Rust implementation doesn't use the --base option yet
    // This test documents the expected behavior once implemented
    let temp_dir = setup_git_repo();
    let repo_path = temp_dir.path();

    // Create a second commit
    fs::write(repo_path.join("file2.txt"), "content").unwrap();
    std::process::Command::new("git")
        .args(["add", "."])
        .current_dir(repo_path)
        .output()
        .expect("Failed to add files");
    std::process::Command::new("git")
        .args(["commit", "-m", "Second commit"])
        .current_dir(repo_path)
        .output()
        .expect("Failed to commit");

    // Get the first commit hash
    let output = std::process::Command::new("git")
        .args(["log", "--format=%H", "-n", "2"])
        .current_dir(repo_path)
        .output()
        .expect("Failed to get commits");

    let commits = String::from_utf8_lossy(&output.stdout);
    let first_commit = commits.lines().nth(1).unwrap();

    // Create worktree based on first commit
    Command::cargo_bin("phantom")
        .unwrap()
        .args(["create", "based-on-first", "--base", first_commit])
        .current_dir(repo_path)
        .assert()
        .success();

    // Verify the worktree exists (base option not implemented yet)
    let worktree_path = repo_path.join(".git/phantom/worktrees/based-on-first");
    assert!(worktree_path.exists());

    // Verify the worktree is at the correct commit
    let output = std::process::Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(worktree_path)
        .output()
        .expect("Failed to get worktree commit");

    let worktree_commit = String::from_utf8_lossy(&output.stdout).trim().to_string();
    assert_eq!(worktree_commit, first_commit, "Worktree should be based on the specified commit");
}

#[test]
fn test_typescript_compat_config_file_formats() {
    // TypeScript version supports both .phantom.json and .phantom.toml
    let temp_dir = setup_git_repo();
    let repo_path = temp_dir.path();

    // Test JSON config
    let json_config = r#"
{
    "copyFiles": ["README.md"]
}
"#;
    fs::write(repo_path.join(".phantom.json"), json_config).unwrap();

    Command::cargo_bin("phantom")
        .unwrap()
        .args(["create", "with-json-config"])
        .current_dir(repo_path)
        .assert()
        .success();

    // Verify file was copied
    let worktree1 = repo_path.join(".git/phantom/worktrees/with-json-config");
    assert!(worktree1.join("README.md").exists());

    // Remove JSON config and test TOML config
    fs::remove_file(repo_path.join(".phantom.json")).unwrap();

    let toml_config = r#"
copyFiles = ["README.md"]
"#;
    fs::write(repo_path.join(".phantom.toml"), toml_config).unwrap();

    Command::cargo_bin("phantom")
        .unwrap()
        .args(["create", "with-toml-config"])
        .current_dir(repo_path)
        .assert()
        .success();

    // Verify file was copied
    let worktree2 = repo_path.join(".git/phantom/worktrees/with-toml-config");
    assert!(worktree2.join("README.md").exists());
}

#[test]
fn test_typescript_compat_attach_error_messages() {
    // TypeScript version has specific error messages
    let temp_dir = setup_git_repo();
    let repo_path = temp_dir.path();

    // Try to attach to non-existent branch
    Command::cargo_bin("phantom")
        .unwrap()
        .args(["attach", "no-such-branch"])
        .current_dir(repo_path)
        .assert()
        .failure()
        .stderr(predicate::str::contains("Branch"))
        .stderr(predicate::str::contains("not found"));

    // Create a branch and attach
    std::process::Command::new("git")
        .args(["branch", "existing-branch"])
        .current_dir(repo_path)
        .output()
        .expect("Failed to create branch");

    Command::cargo_bin("phantom")
        .unwrap()
        .args(["attach", "existing-branch"])
        .current_dir(repo_path)
        .assert()
        .success();

    // Try to attach again
    Command::cargo_bin("phantom")
        .unwrap()
        .args(["attach", "existing-branch"])
        .current_dir(repo_path)
        .assert()
        .failure()
        .stderr(predicate::str::contains("already exists"));
}

#[test]
fn test_typescript_compat_where_command() {
    // TypeScript version's where command outputs just the path
    let temp_dir = setup_git_repo();
    let repo_path = temp_dir.path();

    Command::cargo_bin("phantom")
        .unwrap()
        .args(["create", "test-where"])
        .current_dir(repo_path)
        .assert()
        .success();

    let output = Command::cargo_bin("phantom")
        .unwrap()
        .args(["where", "test-where"])
        .current_dir(repo_path)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let path = String::from_utf8_lossy(&output);
    // Should be a single line with just the path
    assert_eq!(path.lines().count(), 1);
    assert!(path.contains("phantom/worktrees/test-where"));
}

#[test]
fn test_typescript_compat_list_names_flag() {
    // TypeScript version supports --names flag
    let temp_dir = setup_git_repo();
    let repo_path = temp_dir.path();

    // Create some worktrees
    Command::cargo_bin("phantom")
        .unwrap()
        .args(["create", "wt1"])
        .current_dir(repo_path)
        .assert()
        .success();

    Command::cargo_bin("phantom")
        .unwrap()
        .args(["create", "wt2"])
        .current_dir(repo_path)
        .assert()
        .success();

    let output = Command::cargo_bin("phantom")
        .unwrap()
        .args(["list", "--names"])
        .current_dir(repo_path)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let names = String::from_utf8_lossy(&output);
    let lines: Vec<&str> = names.lines().collect();

    // Should only contain names, no paths
    assert_eq!(lines.len(), 2);
    assert!(lines.contains(&"wt1"));
    assert!(lines.contains(&"wt2"));
    for line in lines {
        assert!(!line.contains("/"));
    }
}
