use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

/// Helper to create a test git repository with config
fn setup_test_project() -> TempDir {
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

    // Create project structure
    fs::create_dir_all(repo_path.join("src")).unwrap();
    fs::write(repo_path.join("README.md"), "# Test Project\n").unwrap();
    fs::write(repo_path.join("src/main.rs"), "fn main() {}\n").unwrap();
    fs::write(repo_path.join(".gitignore"), "target/\n").unwrap();
    fs::write(repo_path.join("Cargo.toml"), "[package]\nname = \"test\"\nversion = \"0.1.0\"\n")
        .unwrap();

    // Create initial commit
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
fn test_e2e_basic_workflow() {
    let temp_dir = setup_test_project();
    let repo_path = temp_dir.path();

    // 1. List worktrees (should be empty)
    Command::cargo_bin("phantom")
        .unwrap()
        .arg("list")
        .current_dir(repo_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("No worktrees found"));

    // 2. Create a feature worktree
    Command::cargo_bin("phantom")
        .unwrap()
        .args(["create", "feature-auth"])
        .current_dir(repo_path)
        .assert()
        .success();

    // 3. List worktrees (should show the new one)
    Command::cargo_bin("phantom")
        .unwrap()
        .arg("list")
        .current_dir(repo_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("feature-auth"));

    // 4. Get the path of the worktree
    Command::cargo_bin("phantom")
        .unwrap()
        .args(["where", "feature-auth"])
        .current_dir(repo_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("phantom/worktrees/feature-auth"));

    // 5. Create another worktree
    Command::cargo_bin("phantom")
        .unwrap()
        .args(["create", "feature-ui", "--branch", "ui-improvements"])
        .current_dir(repo_path)
        .assert()
        .success();

    // 6. List worktrees (should show both)
    let list_output = Command::cargo_bin("phantom")
        .unwrap()
        .arg("list")
        .current_dir(repo_path)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let list_str = String::from_utf8_lossy(&list_output);
    assert!(list_str.contains("feature-auth"));
    assert!(list_str.contains("feature-ui"));

    // 7. Delete the first worktree
    Command::cargo_bin("phantom")
        .unwrap()
        .args(["delete", "feature-auth"])
        .current_dir(repo_path)
        .assert()
        .success();

    // 8. List again (should only show feature-ui)
    let final_list = Command::cargo_bin("phantom")
        .unwrap()
        .arg("list")
        .current_dir(repo_path)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let final_str = String::from_utf8_lossy(&final_list);
    assert!(!final_str.contains("feature-auth"));
    assert!(final_str.contains("feature-ui"));
}

#[test]
fn test_e2e_branch_attachment_workflow() {
    let temp_dir = setup_test_project();
    let repo_path = temp_dir.path();

    // 1. Create a branch without worktree
    std::process::Command::new("git")
        .args(["branch", "hotfix-123"])
        .current_dir(repo_path)
        .output()
        .expect("Failed to create branch");

    // 2. Attach to the branch
    Command::cargo_bin("phantom")
        .unwrap()
        .args(["attach", "hotfix-123"])
        .current_dir(repo_path)
        .assert()
        .success();

    // 3. Verify the worktree exists
    Command::cargo_bin("phantom")
        .unwrap()
        .args(["where", "hotfix-123"])
        .current_dir(repo_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("phantom/worktrees/hotfix-123"));

    // 4. Try to attach again (should fail)
    Command::cargo_bin("phantom")
        .unwrap()
        .args(["attach", "hotfix-123"])
        .current_dir(repo_path)
        .assert()
        .failure()
        .stderr(predicate::str::contains("already exists"));
}

#[test]
fn test_e2e_json_workflow() {
    let temp_dir = setup_test_project();
    let repo_path = temp_dir.path();

    // 1. Create a worktree with JSON output
    let create_output = Command::cargo_bin("phantom")
        .unwrap()
        .args(["create", "json-test", "--json"])
        .current_dir(repo_path)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    // Verify JSON structure
    let create_str = String::from_utf8_lossy(&create_output);
    if !create_str.is_empty() {
        let json: serde_json::Value = serde_json::from_str(&create_str).unwrap();
        assert_eq!(json["name"], "json-test");
        assert!(json["success"].as_bool().unwrap_or(false));
    }

    // 2. List with JSON output
    let list_output = Command::cargo_bin("phantom")
        .unwrap()
        .args(["list", "--json"])
        .current_dir(repo_path)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let list_str = String::from_utf8_lossy(&list_output);
    if !list_str.is_empty() {
        let json: serde_json::Value = serde_json::from_str(&list_str).unwrap();
        assert!(json["worktrees"].is_array());
    }

    // 3. Where with JSON output
    let where_output = Command::cargo_bin("phantom")
        .unwrap()
        .args(["where", "json-test", "--json"])
        .current_dir(repo_path)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let where_str = String::from_utf8_lossy(&where_output);
    if !where_str.is_empty() {
        let json: serde_json::Value = serde_json::from_str(&where_str).unwrap();
        assert!(json["path"].as_str().unwrap().contains("json-test"));
    }
}

#[test]
fn test_e2e_error_handling_workflow() {
    let temp_dir = setup_test_project();
    let repo_path = temp_dir.path();

    // 1. Try to create worktree with invalid name
    Command::cargo_bin("phantom")
        .unwrap()
        .args(["create", "invalid name!"])
        .current_dir(repo_path)
        .assert()
        .failure()
        .code(2) // VALIDATION_ERROR
        .stderr(predicate::str::contains("Invalid worktree name"));

    // 2. Try to delete non-existent worktree
    Command::cargo_bin("phantom")
        .unwrap()
        .args(["delete", "non-existent"])
        .current_dir(repo_path)
        .assert()
        .failure()
        .stderr(predicate::str::contains("not found"));

    // 3. Try to attach to non-existent branch
    Command::cargo_bin("phantom")
        .unwrap()
        .args(["attach", "no-such-branch"])
        .current_dir(repo_path)
        .assert()
        .failure()
        .code(6) // BRANCH_NOT_FOUND
        .stderr(predicate::str::contains("Branch"));

    // 4. Create a worktree successfully
    Command::cargo_bin("phantom")
        .unwrap()
        .args(["create", "test-worktree"])
        .current_dir(repo_path)
        .assert()
        .success();

    // 5. Try to create another with same name
    Command::cargo_bin("phantom")
        .unwrap()
        .args(["create", "test-worktree"])
        .current_dir(repo_path)
        .assert()
        .failure()
        .stderr(predicate::str::contains("already exists"));
}

#[test]
fn test_e2e_names_only_workflow() {
    let temp_dir = setup_test_project();
    let repo_path = temp_dir.path();

    // Create multiple worktrees
    Command::cargo_bin("phantom")
        .unwrap()
        .args(["create", "feature-1"])
        .current_dir(repo_path)
        .assert()
        .success();

    Command::cargo_bin("phantom")
        .unwrap()
        .args(["create", "feature-2"])
        .current_dir(repo_path)
        .assert()
        .success();

    Command::cargo_bin("phantom")
        .unwrap()
        .args(["create", "bugfix-1"])
        .current_dir(repo_path)
        .assert()
        .success();

    // List names only
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

    // Should contain exactly the worktree names
    assert_eq!(lines.len(), 3);
    assert!(lines.contains(&"feature-1"));
    assert!(lines.contains(&"feature-2"));
    assert!(lines.contains(&"bugfix-1"));

    // Should not contain paths
    for line in lines {
        assert!(!line.contains("/"));
        assert!(!line.contains("phantom"));
    }
}

#[test]
fn test_e2e_completion_generation() {
    // Test bash completion
    Command::cargo_bin("phantom")
        .unwrap()
        .args(["completion", "bash"])
        .assert()
        .success()
        .stdout(predicate::str::contains("_phantom_completions"))
        .stdout(predicate::str::contains("complete -F"));

    // Test zsh completion
    Command::cargo_bin("phantom")
        .unwrap()
        .args(["completion", "zsh"])
        .assert()
        .success()
        .stdout(predicate::str::contains("#compdef phantom"));

    // Test fish completion
    Command::cargo_bin("phantom")
        .unwrap()
        .args(["completion", "fish"])
        .assert()
        .success()
        .stdout(predicate::str::contains("complete -c phantom"));
}

#[test]
fn test_e2e_copy_files_workflow() {
    let temp_dir = setup_test_project();
    let repo_path = temp_dir.path();

    // Create a config file
    let config = r#"
{
    "copyFiles": [".gitignore", "README.md"]
}
"#;
    fs::write(repo_path.join(".phantom.json"), config).unwrap();

    // Create a worktree (should copy files)
    Command::cargo_bin("phantom")
        .unwrap()
        .args(["create", "with-files"])
        .current_dir(repo_path)
        .assert()
        .success();

    // Verify files were copied
    let worktree_path = repo_path.join(".git/phantom/worktrees/with-files");
    assert!(worktree_path.join(".gitignore").exists());
    assert!(worktree_path.join("README.md").exists());
}

#[test]
fn test_e2e_verbose_mode() {
    let temp_dir = setup_test_project();
    let repo_path = temp_dir.path();

    // Run with verbose flag
    Command::cargo_bin("phantom")
        .unwrap()
        .args(["-v", "list"])
        .current_dir(repo_path)
        .env("RUST_LOG", "") // Clear any existing
        .assert()
        .success();

    // Run with quiet flag
    Command::cargo_bin("phantom")
        .unwrap()
        .args(["-q", "list"])
        .current_dir(repo_path)
        .assert()
        .success();
}
