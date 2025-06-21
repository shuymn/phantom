use crate::core::command_executor::CommandExecutor;
use crate::core::exit_handler::ExitHandler;
use crate::core::filesystem::FileSystem;
use std::sync::Arc;

/// Generic context for CLI handlers with compile-time polymorphism
/// 
/// This provides zero-cost abstractions by using generics instead of dynamic dispatch.
/// For testing, use `TestHandlerContext` type alias.
#[derive(Clone)]
pub struct GenericHandlerContext<E, F, H>
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

impl<E, F, H> GenericHandlerContext<E, F, H>
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

/// Type alias for production use with real implementations
pub type ProductionContext = GenericHandlerContext<
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

/// Legacy context using dynamic dispatch for backward compatibility
/// 
/// This type alias maintains the existing API while we migrate to generics.
/// New code should use `GenericHandlerContext` or `ProductionContext`.
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

/// Type alias for test contexts using mock implementations
#[cfg(test)]
pub type TestHandlerContext = GenericHandlerContext<
    crate::core::executors::MockCommandExecutor,
    crate::core::filesystems::MockFileSystem,
    crate::core::exit_handler::MockExitHandler,
>;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::executors::MockCommandExecutor;

    #[test]
    fn test_generic_handler_context_new() {
        let executor = MockCommandExecutor::new();
        let filesystem = crate::core::filesystems::MockFileSystem::new();
        let exit_handler = crate::core::exit_handler::MockExitHandler::new();
        let context = GenericHandlerContext::new(executor, filesystem, exit_handler);
        
        // Verify context is created successfully with zero-cost abstractions
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
    fn test_generic_context_clone() {
        let context1 = TestHandlerContext::new(
            MockCommandExecutor::new(),
            crate::core::filesystems::MockFileSystem::new(),
            crate::core::exit_handler::MockExitHandler::new(),
        );
        let context2 = context1.clone();
        
        // Both contexts have their own instances (Clone trait)
        // but they're compile-time types, not runtime dispatch
        let _ = &context2.executor;
    }

    // Legacy tests for backward compatibility
    #[test]
    fn test_handler_context_new() {
        let executor: Arc<dyn CommandExecutor> = Arc::new(MockCommandExecutor::new());
        let filesystem: Arc<dyn FileSystem> =
            Arc::new(crate::core::filesystems::MockFileSystem::new());
        let exit_handler: Arc<dyn ExitHandler> =
            Arc::new(crate::core::exit_handler::MockExitHandler::new());
        let context =
            HandlerContext::new(executor.clone(), filesystem.clone(), exit_handler.clone());
        assert!(Arc::ptr_eq(&context.executor, &executor));
        assert!(Arc::ptr_eq(&context.filesystem, &filesystem));
        assert!(Arc::ptr_eq(&context.exit_handler, &exit_handler));
    }

    #[test]
    fn test_handler_context_default() {
        let context = HandlerContext::default();
        // Just verify it creates successfully
        let _ = context.executor;
    }

    #[test]
    fn test_handler_context_clone() {
        let executor = Arc::new(MockCommandExecutor::new());
        let filesystem = Arc::new(crate::core::filesystems::MockFileSystem::new());
        let exit_handler = Arc::new(crate::core::exit_handler::MockExitHandler::new());
        let context1 = HandlerContext::new(executor, filesystem, exit_handler);
        let context2 = context1.clone();

        // Both contexts should share the same executor, filesystem, and exit handler
        assert!(Arc::ptr_eq(&context1.executor, &context2.executor));
        assert!(Arc::ptr_eq(&context1.filesystem, &context2.filesystem));
        assert!(Arc::ptr_eq(&context1.exit_handler, &context2.exit_handler));
    }
}
