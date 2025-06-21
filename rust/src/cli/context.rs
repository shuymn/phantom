use crate::core::command_executor::CommandExecutor;
use crate::core::exit_handler::ExitHandler;
use crate::core::filesystem::FileSystem;

/// Context for CLI handlers with zero-cost abstractions
///
/// Uses generics for compile-time polymorphism, eliminating dynamic dispatch overhead.
/// This results in better performance through monomorphization and inlining.
#[derive(Clone)]
pub struct HandlerContext<E, F, H>
where
    E: CommandExecutor,
    F: FileSystem,
    H: ExitHandler,
{
    /// Command executor for running external commands
    pub executor: E,
    /// Filesystem abstraction for file operations
    pub filesystem: F,
    /// Exit handler for process termination
    pub exit_handler: H,
}

impl<E, F, H> HandlerContext<E, F, H>
where
    E: CommandExecutor,
    F: FileSystem,
    H: ExitHandler,
{
    /// Create a new handler context with the given executor, filesystem, and exit handler
    pub fn new(executor: E, filesystem: F, exit_handler: H) -> Self {
        Self { executor, filesystem, exit_handler }
    }
}

/// Type alias for production context using real implementations
pub type ProductionContext = HandlerContext<
    crate::core::executors::RealCommandExecutor,
    crate::core::filesystems::RealFileSystem,
    crate::core::exit_handler::RealExitHandler,
>;

impl Default for ProductionContext {
    fn default() -> Self {
        Self::new(
            crate::core::executors::RealCommandExecutor,
            crate::core::filesystems::RealFileSystem::new(),
            crate::core::exit_handler::RealExitHandler::new(),
        )
    }
}

/// Type alias for test context using mock implementations
#[cfg(test)]
pub type TestContext = HandlerContext<
    crate::core::executors::MockCommandExecutor,
    crate::core::filesystems::MockFileSystem,
    crate::core::exit_handler::MockExitHandler,
>;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::executors::MockCommandExecutor;
    use crate::core::exit_handler::MockExitHandler;
    use crate::core::filesystems::MockFileSystem;

    #[test]
    fn test_handler_context_new() {
        let executor = MockCommandExecutor::new();
        let filesystem = MockFileSystem::new();
        let exit_handler = MockExitHandler::new();
        let context = HandlerContext::new(executor, filesystem, exit_handler);

        // Verify context is created successfully
        // With generics, we no longer need to check Arc pointer equality
        let _ = &context.executor;
        let _ = &context.filesystem;
        let _ = &context.exit_handler;
    }

    #[test]
    fn test_production_context_default() {
        let context = ProductionContext::default();
        // Verify production context with real implementations
        let _ = &context.executor;
        let _ = &context.filesystem;
        let _ = &context.exit_handler;
    }

    #[test]
    fn test_test_context() {
        let context: TestContext = HandlerContext::new(
            MockCommandExecutor::new(),
            MockFileSystem::new(),
            MockExitHandler::new(),
        );

        // Test context works with mock implementations
        let _ = &context.executor;
        let _ = &context.filesystem;
        let _ = &context.exit_handler;
    }

    #[test]
    fn test_handler_context_clone() {
        let context1 = HandlerContext::new(
            MockCommandExecutor::new(),
            MockFileSystem::new(),
            MockExitHandler::new(),
        );
        let context2 = context1.clone();

        // Both contexts have their own cloned values
        // (MockCommandExecutor implements Clone)
        let _ = &context2.executor;
        let _ = &context2.filesystem;
        let _ = &context2.exit_handler;
    }

    #[test]
    fn test_generic_context_zero_cost() {
        // This test demonstrates that the generic context has zero runtime cost
        // The compiler will monomorphize the context for each concrete type combination
        fn process_with_context<E, F, H>(context: HandlerContext<E, F, H>)
        where
            E: CommandExecutor,
            F: FileSystem,
            H: ExitHandler,
        {
            // This function will be optimized and inlined for each concrete type
            let _ = &context.executor;
            let _ = &context.filesystem;
            let _ = &context.exit_handler;
        }

        let test_context: TestContext = HandlerContext::new(
            MockCommandExecutor::new(),
            MockFileSystem::new(),
            MockExitHandler::new(),
        );

        process_with_context(test_context);

        let prod_context = ProductionContext::default();
        process_with_context(prod_context);
    }
}
