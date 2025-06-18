use std::path::Path;
use std::sync::OnceLock;
use phantom::test_utils::safe_git::SafeGitCommand;

/// Global SafeGitCommand instance for tests
static SAFE_GIT: OnceLock<SafeGitCommand> = OnceLock::new();

/// Get or create a SafeGitCommand instance
fn get_safe_git() -> &'static SafeGitCommand {
    SAFE_GIT.get_or_init(|| {
        SafeGitCommand::new().expect("Failed to create SafeGitCommand")
    })
}

/// Initialize a git repository with proper configuration
pub fn init_test_repo(path: &Path) {
    let git = get_safe_git();
    
    // Initialize git repo with explicit main branch
    git.command(&["init", "-b", "main"])
        .current_dir(path)
        .output()
        .expect("Failed to init git repo");

    // Configure git user
    git.command(&["config", "user.name", "Test User"])
        .current_dir(path)
        .output()
        .expect("Failed to configure git user name");
    
    git.command(&["config", "user.email", "test@example.com"])
        .current_dir(path)
        .output()
        .expect("Failed to configure git user email");
}

/// Create an initial commit with a README file
pub fn create_initial_commit(path: &Path) {
    let git = get_safe_git();
    
    std::fs::write(path.join("README.md"), "# Test Repository").unwrap();
    
    git.command(&["add", "."])
        .current_dir(path)
        .output()
        .expect("Failed to add files");
        
    git.command(&["commit", "-m", "Initial commit"])
        .current_dir(path)
        .output()
        .expect("Failed to commit");
}