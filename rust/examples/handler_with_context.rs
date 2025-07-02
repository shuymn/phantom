use phantom::cli::context::{HandlerContext, ProductionContext};
use phantom::core::command_executor::{CommandConfig, CommandExecutor};
use phantom::core::executors::MockCommandExecutor;
use phantom::core::exit_handler::ExitHandler;
use phantom::core::filesystem::FileSystem;

// Example: A handler that needs to execute git commands
struct StatusHandler<E, F, H>
where
    E: CommandExecutor,
    F: FileSystem,
    H: ExitHandler,
{
    context: HandlerContext<E, F, H>,
}

impl<E, F, H> StatusHandler<E, F, H>
where
    E: CommandExecutor,
    F: FileSystem,
    H: ExitHandler,
{
    fn new(context: HandlerContext<E, F, H>) -> Self {
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
    let prod_context = ProductionContext::default();
    let prod_handler = StatusHandler::new(prod_context);

    match prod_handler.handle().await {
        Ok(status) => println!("Status output:\n{status}"),
        Err(e) => println!("Error: {e}"),
    }

    // Test usage
    println!("\n2. Testing with MockCommandExecutor:");
    let mut mock = MockCommandExecutor::new();
    mock.expect_command("git").with_args(&["status", "--short"]).returns_output(
        "M  src/main.rs\n?? new_file.txt\n",
        "",
        0,
    );

    // Use RealExitHandler for the example (MockExitHandler is only available in tests)
    let test_context = HandlerContext::new(
        mock,
        phantom::core::filesystems::MockFileSystem::new(),
        phantom::core::exit_handler::RealExitHandler::new(),
    );
    let test_handler = StatusHandler::new(test_context);

    match test_handler.handle().await {
        Ok(status) => println!("Mock status output:\n{status}"),
        Err(e) => println!("Error: {e}"),
    }

    println!("\n=== Example Complete ===");
}
