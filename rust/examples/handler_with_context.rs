use phantom::cli::context::HandlerContext;
use phantom::core::command_executor::CommandConfig;
use phantom::core::executors::MockCommandExecutor;
use std::sync::Arc;

// Example: A handler that needs to execute git commands
struct StatusHandler {
    context: HandlerContext,
}

impl StatusHandler {
    fn new(context: HandlerContext) -> Self {
        Self { context }
    }

    async fn handle(&self) -> Result<String, Box<dyn std::error::Error>> {
        // Use the executor from context
        let config =
            CommandConfig::new("git").with_args(vec!["status".to_string(), "--short".to_string()]);

        let output = self.context.executor.execute(config).await?;

        if output.success() {
            Ok(output.stdout.into_owned())
        } else {
            Err(format!("Git status failed: {}", output.stderr).into())
        }
    }
}

#[tokio::main]
async fn main() {
    println!("=== Handler with Context Example ===\n");

    // Production usage
    println!("1. Production with RealCommandExecutor:");
    let prod_context = HandlerContext::default(); // Uses RealCommandExecutor
    let prod_handler = StatusHandler::new(prod_context);

    match prod_handler.handle().await {
        Ok(status) => println!("Status output:\n{}", status),
        Err(e) => println!("Error: {}", e),
    }

    // Test usage
    println!("\n2. Testing with MockCommandExecutor:");
    let mut mock = MockCommandExecutor::new();
    mock.expect_command("git").with_args(&["status", "--short"]).returns_output(
        "M  src/main.rs\n?? new_file.txt\n",
        "",
        0,
    );

    // Create a simple mock exit handler for the example
    struct SimpleExitHandler;
    impl phantom::core::exit_handler::ExitHandler for SimpleExitHandler {
        fn exit(&self, code: i32) -> ! {
            std::process::exit(code)
        }
    }

    let test_context = HandlerContext::new(
        Arc::new(mock),
        Arc::new(phantom::core::filesystems::MockFileSystem::new()),
        Arc::new(SimpleExitHandler),
    );
    let test_handler = StatusHandler::new(test_context.clone());

    match test_handler.handle().await {
        Ok(status) => println!("Mock status output:\n{}", status),
        Err(e) => println!("Error: {}", e),
    }

    println!("\n=== Example Complete ===");
}
