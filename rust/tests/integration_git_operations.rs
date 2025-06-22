use phantom::core::executors::RealCommandExecutor;
use phantom::git::git_executor_adapter::GitExecutor;
use phantom::git::libs::{
    add_worktree::add_worktree, attach_worktree::attach_worktree, branch_exists::branch_exists,
    get_current_branch::get_current_branch, get_current_worktree::get_current_worktree,
    get_git_root::get_git_root, list_worktrees::list_worktrees,
};
use std::fs;
use tempfile::tempdir;

/// Helper to create a real git repository
async fn create_real_git_repo() -> tempfile::TempDir {
    let temp_dir = tempdir().expect("Failed to create temp dir");
    let repo_path = temp_dir.path();

    let executor = GitExecutor::new(RealCommandExecutor::new()).with_cwd(repo_path);

    // Initialize git repo
    executor.run(&["init"]).await.expect("Failed to init git repo");

    // Set user config for commits
    executor.run(&["config", "user.email", "test@example.com"]).await.expect("Failed to set email");
    executor.run(&["config", "user.name", "Test User"]).await.expect("Failed to set name");

    // Create initial commit on main branch
    fs::write(repo_path.join("README.md"), "# Test Repository").expect("Failed to write README");
    executor.run(&["add", "README.md"]).await.expect("Failed to add README");
    executor.run(&["commit", "-m", "Initial commit"]).await.expect("Failed to commit");

    // Ensure we're on main branch (rename if needed)
    let current_branch =
        executor.run(&["branch", "--show-current"]).await.expect("Failed to get branch");
    let current_branch = current_branch.trim();
    if current_branch != "main" {
        executor
            .run(&["branch", "-m", current_branch, "main"])
            .await
            .expect("Failed to rename branch");
    }

    temp_dir
}

#[tokio::test]
#[serial_test::serial]
async fn test_real_git_operations_workflow() {
    let repo = create_real_git_repo().await;
    let repo_path = repo.path();

    // Test get_git_root - it finds git root from current directory
    std::env::set_current_dir(repo_path).expect("Failed to set current dir");
    let git_root = get_git_root(RealCommandExecutor).await.expect("Failed to get git root");
    assert_eq!(git_root.canonicalize().unwrap(), repo_path.canonicalize().unwrap());

    // Test get_current_branch
    let current_branch = get_current_branch(RealCommandExecutor, repo_path)
        .await
        .expect("Failed to get current branch");
    assert_eq!(current_branch, "main");

    // Test branch_exists
    assert!(branch_exists(RealCommandExecutor, repo_path, "main")
        .await
        .expect("Failed to check main branch"));
    assert!(!branch_exists(RealCommandExecutor, repo_path, "nonexistent")
        .await
        .expect("Failed to check nonexistent branch"));

    // Test list_worktrees - should have just the main worktree
    let worktrees =
        list_worktrees(RealCommandExecutor, repo_path).await.expect("Failed to list worktrees");
    assert_eq!(worktrees.len(), 1);
    assert_eq!(worktrees[0].branch, Some("main".to_string()));

    // Test get_current_worktree - should return None for main worktree
    let current_worktree = get_current_worktree(RealCommandExecutor, repo_path)
        .await
        .expect("Failed to get current worktree");
    assert!(current_worktree.is_none());
}

#[tokio::test]
async fn test_worktree_creation_and_management() {
    let repo = create_real_git_repo().await;
    let repo_path = repo.path();

    // Create a new worktree with a new branch
    let timestamp =
        std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis();
    let worktree_path = repo_path.parent().unwrap().join(format!("test-worktree-{}", timestamp));
    add_worktree(
        RealCommandExecutor,
        repo_path,
        &worktree_path,
        Some("feature-branch"),
        true,
        None,
    )
    .await
    .expect("Failed to add worktree");

    // Verify worktree was created
    assert!(worktree_path.exists());
    assert!(worktree_path.join(".git").exists());

    // Test list_worktrees - should now have two
    let worktrees =
        list_worktrees(RealCommandExecutor, repo_path).await.expect("Failed to list worktrees");
    assert_eq!(worktrees.len(), 2);

    // Find the new worktree
    let new_worktree = worktrees
        .iter()
        .find(|w| w.path.canonicalize().unwrap() == worktree_path.canonicalize().unwrap())
        .expect("New worktree not found");
    assert_eq!(new_worktree.branch, Some("feature-branch".to_string()));

    // Test get_current_branch in the new worktree
    let branch_in_worktree = get_current_branch(RealCommandExecutor, &worktree_path)
        .await
        .expect("Failed to get branch in worktree");
    assert_eq!(branch_in_worktree, "feature-branch");

    // Test get_current_worktree in the new worktree
    let original_dir = std::env::current_dir().ok();
    std::env::set_current_dir(&worktree_path).expect("Failed to change to worktree dir");
    let current_worktree = get_current_worktree(RealCommandExecutor, repo_path)
        .await
        .expect("Failed to get current worktree");
    if let Some(dir) = original_dir {
        let _ = std::env::set_current_dir(&dir); // Ignore error if directory was cleaned up
    }
    assert_eq!(current_worktree, Some("feature-branch".to_string()));

    // Test branch_exists - feature-branch should now exist
    assert!(branch_exists(RealCommandExecutor, repo_path, "feature-branch")
        .await
        .expect("Failed to check feature branch"));
}

#[tokio::test]
async fn test_attach_worktree_to_existing_branch() {
    let repo = create_real_git_repo().await;
    let repo_path = repo.path();

    // Create a new branch
    let executor = GitExecutor::new(RealCommandExecutor::new()).with_cwd(repo_path);
    executor.run(&["checkout", "-b", "existing-branch"]).await.expect("Failed to create branch");
    executor.run(&["checkout", "main"]).await.expect("Failed to switch back to main");

    // Attach a worktree to the existing branch
    let timestamp =
        std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis();
    let worktree_path =
        repo_path.parent().unwrap().join(format!("attached-worktree-{}", timestamp));
    attach_worktree(RealCommandExecutor, repo_path, &worktree_path, "existing-branch")
        .await
        .expect("Failed to attach worktree");

    // Verify worktree was created and attached
    assert!(worktree_path.exists());
    let branch_in_worktree = get_current_branch(RealCommandExecutor, &worktree_path)
        .await
        .expect("Failed to get branch");
    assert_eq!(branch_in_worktree, "existing-branch");

    // Verify it appears in the worktree list
    let worktrees =
        list_worktrees(RealCommandExecutor, repo_path).await.expect("Failed to list worktrees");
    let attached_worktree = worktrees
        .iter()
        .find(|w| w.path.canonicalize().unwrap() == worktree_path.canonicalize().unwrap())
        .expect("Attached worktree not found");
    assert_eq!(attached_worktree.branch, Some("existing-branch".to_string()));
}

#[tokio::test]
async fn test_complex_worktree_scenario() {
    let repo = create_real_git_repo().await;
    let repo_path = repo.path();

    // Get original directory at the start of the test
    let original_dir = std::env::current_dir().ok();

    // Create multiple branches and worktrees
    let executor = GitExecutor::new(RealCommandExecutor::new()).with_cwd(repo_path);

    // Create branches
    for i in 1..=3 {
        executor
            .run(&["checkout", "-b", &format!("feature-{}", i)])
            .await
            .expect("Failed to create branch");
        executor.run(&["checkout", "main"]).await.expect("Failed to switch back");
    }

    // Create worktrees for each branch
    let timestamp =
        std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis();
    for i in 1..=3 {
        let worktree_path =
            repo_path.parent().unwrap().join(format!("worktree-{}-{}", i, timestamp));
        attach_worktree(RealCommandExecutor, repo_path, &worktree_path, &format!("feature-{}", i))
            .await
            .expect("Failed to attach worktree");
    }

    // List all worktrees
    let worktrees =
        list_worktrees(RealCommandExecutor, repo_path).await.expect("Failed to list worktrees");
    assert_eq!(worktrees.len(), 4); // main + 3 feature worktrees

    // Verify each worktree
    for i in 1..=3 {
        let worktree_path =
            repo_path.parent().unwrap().join(format!("worktree-{}-{}", i, timestamp));
        let current_branch = get_current_branch(RealCommandExecutor, &worktree_path)
            .await
            .expect("Failed to get branch");
        assert_eq!(current_branch, format!("feature-{}", i));

        // get_current_worktree needs the git root, not the worktree path
        // Also need to change to the worktree directory first
        std::env::set_current_dir(&worktree_path).expect("Failed to change to worktree dir");
        let current_worktree = get_current_worktree(RealCommandExecutor, repo_path)
            .await
            .expect("Failed to get worktree");
        assert_eq!(current_worktree, Some(format!("feature-{}", i)));
    }

    // Restore original directory after loop
    if let Some(dir) = original_dir {
        let _ = std::env::set_current_dir(&dir); // Ignore error if directory was cleaned up
    }

    // Verify all branches exist
    for i in 1..=3 {
        assert!(branch_exists(RealCommandExecutor, repo_path, &format!("feature-{}", i))
            .await
            .expect("Failed to check branch"));
    }
}

#[tokio::test]
async fn test_worktree_with_upstream_branch() {
    let repo = create_real_git_repo().await;
    let repo_path = repo.path();

    // Add worktree with new branch
    let timestamp =
        std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis();
    let worktree_path =
        repo_path.parent().unwrap().join(format!("tracking-worktree-{}", timestamp));
    add_worktree(
        RealCommandExecutor,
        repo_path,
        &worktree_path,
        Some("tracking-branch"),
        true,
        None,
    )
    .await
    .expect("Failed to add worktree with new branch");

    // Verify the worktree was created
    assert!(worktree_path.exists());

    // Check that the branch was created
    assert!(branch_exists(RealCommandExecutor, repo_path, "tracking-branch")
        .await
        .expect("Failed to check branch"));

    // Verify current branch in worktree
    let current_branch = get_current_branch(RealCommandExecutor, &worktree_path)
        .await
        .expect("Failed to get branch");
    assert_eq!(current_branch, "tracking-branch");
}

#[tokio::test]
async fn test_detached_worktree() {
    let repo = create_real_git_repo().await;
    let repo_path = repo.path();

    // Get current commit hash
    let executor = GitExecutor::new(RealCommandExecutor::new()).with_cwd(repo_path);
    let commit_hash = executor.run(&["rev-parse", "HEAD"]).await.expect("Failed to get commit");
    let commit_hash = commit_hash.trim();

    // Create a detached worktree with unique name
    let timestamp =
        std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis();
    let worktree_path =
        repo_path.parent().unwrap().join(format!("detached-worktree-{}", timestamp));
    executor
        .run(&["worktree", "add", "--detach", &worktree_path.to_string_lossy(), commit_hash])
        .await
        .expect("Failed to create detached worktree");

    // List worktrees and find the detached one
    let worktrees =
        list_worktrees(RealCommandExecutor, repo_path).await.expect("Failed to list worktrees");
    let detached_worktree = worktrees
        .iter()
        .find(|w| w.path.canonicalize().unwrap() == worktree_path.canonicalize().unwrap())
        .expect("Detached worktree not found");

    assert!(detached_worktree.is_detached);
    assert!(detached_worktree.branch.is_none());

    // get_current_branch should return empty for detached HEAD
    let current_branch = get_current_branch(RealCommandExecutor, &worktree_path)
        .await
        .expect("Failed to get branch");
    assert_eq!(current_branch, "");

    // get_current_worktree should return None for detached worktree
    let original_dir = std::env::current_dir().ok();
    std::env::set_current_dir(&worktree_path).expect("Failed to change to worktree dir");
    let current_worktree =
        get_current_worktree(RealCommandExecutor, repo_path).await.expect("Failed to get worktree");
    if let Some(dir) = original_dir {
        let _ = std::env::set_current_dir(&dir); // Ignore error if directory was cleaned up
    }
    assert!(current_worktree.is_none());
}
