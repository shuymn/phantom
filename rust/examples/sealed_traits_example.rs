/// Example demonstrating the sealed trait pattern for API stability
///
/// This example shows how sealed traits prevent downstream users from
/// implementing our core traits while still allowing them to use them.
use phantom::core::command_executor::CommandExecutor;
use phantom::core::exit_handler::ExitHandler;
use phantom::core::filesystem::FileSystem;
use phantom::git::backend::GitBackend;

// This module represents "external" code trying to use phantom

// ❌ THESE WOULD NOT COMPILE - Cannot implement sealed traits
//
// struct MyCommandExecutor;
//
// impl CommandExecutor for MyCommandExecutor {
//     async fn execute(&self, config: CommandConfig) -> Result<CommandOutput> {
//         // Error: the trait `phantom::core::sealed::Sealed` is not implemented
//     }
// }
//
// struct MyFileSystem;
//
// impl FileSystem for MyFileSystem {
//     // Error: the trait `phantom::core::sealed::Sealed` is not implemented
// }

// ✅ THESE WORK - Using the traits is allowed

use phantom::core::executors::RealCommandExecutor;
use phantom::core::exit_handler::RealExitHandler;
use phantom::core::filesystems::RealFileSystem;
use phantom::git::command_backend::CommandBackend;

fn accept_command_executor<E: CommandExecutor>(_executor: &E) {
    println!("Received a command executor");
    // Can use the trait methods
    // executor.execute(...).await
}

fn accept_filesystem<F: FileSystem>(_fs: &F) {
    println!("Received a filesystem");
    // Can use the trait methods
    // fs.exists(...).await
}

fn accept_exit_handler<H: ExitHandler>(_handler: &H) {
    println!("Received an exit handler");
    // Can use the trait methods
    // handler.exit(0)
}

async fn accept_git_backend<G: GitBackend>(_backend: &G) {
    println!("Received a git backend");
    // Can use the trait methods
    // backend.current_branch().await
}

fn main() {
    println!("=== Sealed Traits Example ===\n");

    // Create instances of the provided implementations
    let executor = RealCommandExecutor::new();
    let filesystem = RealFileSystem::new();
    let exit_handler = RealExitHandler::new();
    let git_backend = CommandBackend::<RealCommandExecutor>::default();

    // Pass them to functions expecting the traits
    accept_command_executor(&executor);
    accept_filesystem(&filesystem);
    accept_exit_handler(&exit_handler);

    // For async trait
    let runtime = tokio::runtime::Runtime::new().unwrap();
    runtime.block_on(async {
        accept_git_backend(&git_backend).await;
    });

    println!("\n✅ All phantom implementations work correctly!");
    println!("❌ But external implementations are prevented!");

    println!("\n📝 Benefits of sealed traits:");
    println!("1. API stability - we can change trait methods without breaking users");
    println!("2. Implementation control - we ensure all implementations meet our standards");
    println!("3. Future flexibility - we can add new methods or change internals");
    println!("4. Security - prevents malicious implementations of critical traits");
}

// Example: Creating a wrapper is still possible
#[allow(dead_code)]
struct MyExecutorWrapper {
    inner: RealCommandExecutor,
}

// But you can only wrap, not implement the trait directly
#[allow(dead_code)]
impl MyExecutorWrapper {
    fn new() -> Self {
        Self { inner: RealCommandExecutor::new() }
    }

    // Can delegate to the inner implementation
    async fn run_command(&self, program: &str) -> phantom::Result<String> {
        use phantom::core::command_executor::CommandConfig;
        let config = CommandConfig::new(program);
        let output = self.inner.execute(config).await?;
        Ok(output.stdout.into_owned())
    }
}
