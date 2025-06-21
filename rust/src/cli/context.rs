use crate::core::command_executor::CommandExecutor;
use crate::core::exit_handler::ExitHandler;
use crate::core::filesystem::FileSystem;
use std::sync::Arc;

/// Context for CLI handlers
///
/// Uses Arc for shared ownership and dynamic dispatch to support
/// both production and testing scenarios.
#[derive(Clone)]
pub struct HandlerContext {
    /// Command executor for running external commands
    pub executor: Arc<dyn CommandExecutor>,
    /// Filesystem abstraction for file operations
    pub filesystem: Arc<dyn FileSystem>,
    /// Exit handler for process termination
    pub exit_handler: Arc<dyn ExitHandler>,
}

impl HandlerContext {
    /// Create a new handler context with the given executor, filesystem, and exit handler
    pub fn new(
        executor: Arc<dyn CommandExecutor>,
        filesystem: Arc<dyn FileSystem>,
        exit_handler: Arc<dyn ExitHandler>,
    ) -> Self {
        Self { executor, filesystem, exit_handler }
    }
}

impl Default for HandlerContext {
    fn default() -> Self {
        Self::new(
            Arc::new(crate::core::executors::RealCommandExecutor),
            Arc::new(crate::core::filesystems::RealFileSystem::new()),
            Arc::new(crate::core::exit_handler::RealExitHandler::new()),
        )
    }
}

/// Helper for creating test contexts with mock implementations
#[cfg(test)]
pub struct TestHandlerContext;

#[cfg(test)]
impl TestHandlerContext {
    /// Create a new test handler context with mock implementations
    pub fn create(
        executor: crate::core::executors::MockCommandExecutor,
        filesystem: crate::core::filesystems::MockFileSystem,
        exit_handler: crate::core::exit_handler::MockExitHandler,
    ) -> HandlerContext {
        HandlerContext::new(Arc::new(executor), Arc::new(filesystem), Arc::new(exit_handler))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::executors::MockCommandExecutor;

    #[test]
    fn test_handler_context_new() {
        let executor: Arc<dyn CommandExecutor> = Arc::new(MockCommandExecutor::new());
        let filesystem: Arc<dyn FileSystem> =
            Arc::new(crate::core::filesystems::MockFileSystem::new());
        let exit_handler: Arc<dyn ExitHandler> =
            Arc::new(crate::core::exit_handler::MockExitHandler::new());
        let context =
            HandlerContext::new(executor.clone(), filesystem.clone(), exit_handler.clone());

        // Verify context is created successfully
        assert!(Arc::ptr_eq(&context.executor, &executor));
        assert!(Arc::ptr_eq(&context.filesystem, &filesystem));
        assert!(Arc::ptr_eq(&context.exit_handler, &exit_handler));
    }

    #[test]
    fn test_handler_context_default() {
        let context = HandlerContext::default();
        // Verify production context with real implementations
        let _ = &context.executor;
        let _ = &context.filesystem;
        let _ = &context.exit_handler;
    }

    #[test]
    fn test_test_context() {
        let context = TestHandlerContext::create(
            MockCommandExecutor::new(),
            crate::core::filesystems::MockFileSystem::new(),
            crate::core::exit_handler::MockExitHandler::new(),
        );

        // Test context works with mock implementations
        let _ = &context.executor;
        let _ = &context.filesystem;
        let _ = &context.exit_handler;
    }

    #[test]
    fn test_handler_context_clone() {
        let executor = Arc::new(MockCommandExecutor::new());
        let filesystem = Arc::new(crate::core::filesystems::MockFileSystem::new());
        let exit_handler = Arc::new(crate::core::exit_handler::MockExitHandler::new());
        let context1 = HandlerContext::new(executor, filesystem, exit_handler);
        let context2 = context1.clone();

        // Both contexts should share the same Arc instances
        assert!(Arc::ptr_eq(&context1.executor, &context2.executor));
        assert!(Arc::ptr_eq(&context1.filesystem, &context2.filesystem));
        assert!(Arc::ptr_eq(&context1.exit_handler, &context2.exit_handler));
    }
}
