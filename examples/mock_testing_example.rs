use phantom::core::command_executor::{CommandConfig, CommandExecutor};
use phantom::core::executors::{MockCommandExecutor, RealCommandExecutor};
use std::sync::Arc;

// Example: A service that uses git commands
struct GitService {
    executor: Arc<dyn CommandExecutor>,
}

impl GitService {
    fn new(executor: Arc<dyn CommandExecutor>) -> Self {
        Self { executor }
    }

    async fn get_current_branch(&self) -> Result<String, Box<dyn std::error::Error>> {
        let config = CommandConfig::new("git")
            .with_args(vec!["branch".to_string(), "--show-current".to_string()]);

        let output = self.executor.execute(config).await?;

        if output.success() {
            Ok(output.stdout.trim().to_string())
        } else {
            Err(format!("Git command failed: {}", output.stderr).into())
        }
    }

    async fn commit_changes(&self, message: &str) -> Result<(), Box<dyn std::error::Error>> {
        // First add all changes
        let add_config =
            CommandConfig::new("git").with_args(vec!["add".to_string(), ".".to_string()]);

        self.executor.execute(add_config).await?;

        // Then commit with message
        let commit_config = CommandConfig::new("git").with_args(vec![
            "commit".to_string(),
            "-m".to_string(),
            message.to_string(),
        ]);

        let output = self.executor.execute(commit_config).await?;

        if output.success() {
            Ok(())
        } else {
            Err(format!("Commit failed: {}", output.stderr).into())
        }
    }
}

#[tokio::main]
async fn main() {
    println!("=== Mock Testing Example ===\n");

    // Example 1: Using real executor (for production)
    println!("1. Production usage with RealCommandExecutor:");
    let real_executor = Arc::new(RealCommandExecutor::new());
    let git_service = GitService::new(real_executor);

    match git_service.get_current_branch().await {
        Ok(branch) => println!("   Current branch: {branch}"),
        Err(e) => println!("   Error: {e}"),
    }

    // Example 2: Using mock executor for testing
    println!("\n2. Testing with MockCommandExecutor:");

    // Set up expectations
    let mut mock = MockCommandExecutor::new();

    // Expect the branch command
    mock.expect_command("git").with_args(&["branch", "--show-current"]).times(1).returns_output(
        "feature/mock-testing\n",
        "",
        0,
    );

    // Expect add and commit commands
    mock.expect_command("git").with_args(&["add", "."]).times(1).returns_success();

    mock.expect_command("git")
        .with_args(&["commit", "-m", "feat: implement mock testing"])
        .times(1)
        .returns_output("", "[feature/mock-testing abc123] feat: implement mock testing\n", 0);

    let mock_executor = Arc::new(mock);
    let test_service = GitService::new(mock_executor.clone());

    // Test getting current branch
    let branch = test_service.get_current_branch().await.unwrap();
    println!("   Mocked branch: {branch}");

    // Test committing changes
    test_service.commit_changes("feat: implement mock testing").await.unwrap();
    println!("   Mocked commit successful");

    // Verify all expectations were met
    match mock_executor.verify() {
        Ok(()) => println!("   ✓ All expectations verified!"),
        Err(e) => println!("   ✗ Verification failed: {e}"),
    }

    // Example 3: Testing error scenarios
    println!("\n3. Testing error scenarios:");

    let mut error_mock = MockCommandExecutor::new();
    error_mock
        .expect_command("git")
        .with_args(&["branch", "--show-current"])
        .returns_error("fatal: not a git repository");

    let error_service = GitService::new(Arc::new(error_mock));

    match error_service.get_current_branch().await {
        Ok(_) => println!("   Unexpected success"),
        Err(e) => println!("   ✓ Expected error: {e}"),
    }

    println!("\n=== Example Complete ===");
}
