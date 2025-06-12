use std::path::Path;

/// Initialize a git repository with proper configuration
pub fn init_test_repo(path: &Path) {
    // Initialize git repo with explicit main branch
    std::process::Command::new("git")
        .args(&["init", "-b", "main"])
        .current_dir(path)
        .output()
        .expect("Failed to init git repo");

    // Configure git user
    std::process::Command::new("git")
        .args(&["config", "user.name", "Test User"])
        .current_dir(path)
        .output()
        .expect("Failed to configure git user name");
    
    std::process::Command::new("git")
        .args(&["config", "user.email", "test@example.com"])
        .current_dir(path)
        .output()
        .expect("Failed to configure git user email");
}

/// Create an initial commit with a README file
pub fn create_initial_commit(path: &Path) {
    std::fs::write(path.join("README.md"), "# Test Repository").unwrap();
    
    std::process::Command::new("git")
        .args(&["add", "."])
        .current_dir(path)
        .output()
        .expect("Failed to add files");
        
    std::process::Command::new("git")
        .args(&["commit", "-m", "Initial commit"])
        .current_dir(path)
        .output()
        .expect("Failed to commit");
}