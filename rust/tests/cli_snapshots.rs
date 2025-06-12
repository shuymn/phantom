use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::TempDir;

/// Helper to create a test git repository
fn setup_test_repo() -> TempDir {
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

    temp_dir
}

#[test]
fn test_phantom_version_output() {
    let mut cmd = Command::cargo_bin("phantom").unwrap();
    cmd.arg("version");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("phantom"))
        .stdout(predicate::str::contains("0.1.0"));
}

#[test]
fn test_phantom_version_json_output() {
    let mut cmd = Command::cargo_bin("phantom").unwrap();
    cmd.args(&["version", "--json"]);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("\"name\": \"phantom\""))
        .stdout(predicate::str::contains("\"version\": \"0.1.0\""));
}

#[test]
fn test_phantom_help_output() {
    let mut cmd = Command::cargo_bin("phantom").unwrap();
    cmd.arg("--help");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Phantom is a CLI tool"))
        .stdout(predicate::str::contains("Commands:"))
        .stdout(predicate::str::contains("create"))
        .stdout(predicate::str::contains("attach"))
        .stdout(predicate::str::contains("list"))
        .stdout(predicate::str::contains("delete"));
}

#[test]
fn test_phantom_list_empty() {
    let temp_dir = setup_test_repo();

    let mut cmd = Command::cargo_bin("phantom").unwrap();
    cmd.arg("list");
    cmd.current_dir(temp_dir.path());
    cmd.assert().success().stdout(predicate::str::contains("No worktrees found"));
}

#[test]
fn test_phantom_list_json_empty() {
    let temp_dir = setup_test_repo();

    let mut cmd = Command::cargo_bin("phantom").unwrap();
    cmd.args(&["list", "--json"]);
    cmd.current_dir(temp_dir.path());
    cmd.assert().success().stdout(predicate::str::contains("[]"));
}

#[test]
fn test_phantom_create_validation_error() {
    let temp_dir = setup_test_repo();

    let mut cmd = Command::cargo_bin("phantom").unwrap();
    cmd.args(&["create", ""]);
    cmd.current_dir(temp_dir.path());
    cmd.assert().failure().stderr(predicate::str::contains("Error")).code(2); // VALIDATION_ERROR
}

#[test]
fn test_phantom_delete_not_found() {
    let temp_dir = setup_test_repo();

    let mut cmd = Command::cargo_bin("phantom").unwrap();
    cmd.args(&["delete", "nonexistent"]);
    cmd.current_dir(temp_dir.path());
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("Error"))
        .stderr(predicate::str::contains("Worktree"))
        .stderr(predicate::str::contains("not found"))
        .code(1); // GENERAL_ERROR
}

#[test]
fn test_phantom_where_not_found() {
    let temp_dir = setup_test_repo();

    let mut cmd = Command::cargo_bin("phantom").unwrap();
    cmd.args(&["where", "nonexistent"]);
    cmd.current_dir(temp_dir.path());
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("Error"))
        .stderr(predicate::str::contains("Worktree"))
        .stderr(predicate::str::contains("not found"))
        .code(1); // GENERAL_ERROR
}

#[test]
fn test_phantom_attach_invalid_branch() {
    let temp_dir = setup_test_repo();

    let mut cmd = Command::cargo_bin("phantom").unwrap();
    cmd.args(&["attach", "nonexistent-branch"]);
    cmd.current_dir(temp_dir.path());
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("Error"))
        .stderr(predicate::str::contains("Branch"))
        .stderr(predicate::str::contains("not found"))
        .code(6); // BRANCH_NOT_FOUND
}

#[test]
fn test_phantom_not_in_git_repo() {
    let temp_dir = TempDir::new().unwrap();

    let mut cmd = Command::cargo_bin("phantom").unwrap();
    cmd.arg("list");
    cmd.current_dir(temp_dir.path());
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("Error"))
        .stderr(predicate::str::contains("not a git repository"))
        .code(1); // GENERAL_ERROR
}

#[test]
fn test_phantom_completion_bash() {
    let mut cmd = Command::cargo_bin("phantom").unwrap();
    cmd.args(&["completion", "bash"]);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("_phantom_completions"))
        .stdout(predicate::str::contains("complete -F"));
}

#[test]
fn test_phantom_completion_zsh() {
    let mut cmd = Command::cargo_bin("phantom").unwrap();
    cmd.args(&["completion", "zsh"]);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("#compdef phantom"))
        .stdout(predicate::str::contains("_phantom()"));
}

#[test]
fn test_phantom_completion_fish() {
    let mut cmd = Command::cargo_bin("phantom").unwrap();
    cmd.args(&["completion", "fish"]);
    cmd.assert().success().stdout(predicate::str::contains("complete -c phantom"));
}

#[test]
fn test_phantom_invalid_command() {
    let mut cmd = Command::cargo_bin("phantom").unwrap();
    cmd.arg("invalid-command");
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("error"))
        .stderr(predicate::str::contains("unrecognized subcommand"));
}

#[test]
fn test_phantom_verbose_flag() {
    let temp_dir = setup_test_repo();

    let mut cmd = Command::cargo_bin("phantom").unwrap();
    cmd.args(&["-v", "list"]);
    cmd.current_dir(temp_dir.path());
    cmd.env("RUST_LOG", ""); // Clear any existing RUST_LOG
    cmd.assert().success();
}

#[test]
fn test_phantom_quiet_flag() {
    let temp_dir = setup_test_repo();

    let mut cmd = Command::cargo_bin("phantom").unwrap();
    cmd.args(&["-q", "list"]);
    cmd.current_dir(temp_dir.path());
    cmd.assert().success();
}
