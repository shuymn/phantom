use crate::core::command_executor::CommandExecutor;
use crate::core::filesystem::FileSystem;
use std::sync::Arc;

/// Context for CLI handlers containing dependencies
#[derive(Clone)]
pub struct HandlerContext {
    /// Command executor for running external commands
    pub executor: Arc<dyn CommandExecutor>,
    /// Filesystem abstraction for file operations
    pub filesystem: Arc<dyn FileSystem>,
}

impl HandlerContext {
    /// Create a new handler context with the given executor and filesystem
    pub fn new(executor: Arc<dyn CommandExecutor>, filesystem: Arc<dyn FileSystem>) -> Self {
        Self { executor, filesystem }
    }
}

impl Default for HandlerContext {
    fn default() -> Self {
        Self::new(
            Arc::new(crate::core::executors::RealCommandExecutor),
            Arc::new(crate::core::filesystems::RealFileSystem::new()),
        )
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
        let context = HandlerContext::new(executor.clone(), filesystem.clone());
        assert!(Arc::ptr_eq(&context.executor, &executor));
        assert!(Arc::ptr_eq(&context.filesystem, &filesystem));
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
        let context1 = HandlerContext::new(executor, filesystem);
        let context2 = context1.clone();

        // Both contexts should share the same executor and filesystem
        assert!(Arc::ptr_eq(&context1.executor, &context2.executor));
        assert!(Arc::ptr_eq(&context1.filesystem, &context2.filesystem));
    }
}
