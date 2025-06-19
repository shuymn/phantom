use crate::core::command_executor::CommandExecutor;
use std::sync::Arc;

/// Context for CLI handlers containing dependencies
#[derive(Clone)]
pub struct HandlerContext {
    /// Command executor for running external commands
    pub executor: Arc<dyn CommandExecutor>,
}

impl HandlerContext {
    /// Create a new handler context with the given executor
    pub fn new(executor: Arc<dyn CommandExecutor>) -> Self {
        Self { executor }
    }
}

impl Default for HandlerContext {
    fn default() -> Self {
        Self::new(Arc::new(crate::core::executors::RealCommandExecutor))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::executors::MockCommandExecutor;

    #[test]
    fn test_handler_context_new() {
        let executor: Arc<dyn CommandExecutor> = Arc::new(MockCommandExecutor::new());
        let context = HandlerContext::new(executor.clone());
        assert!(Arc::ptr_eq(&context.executor, &executor));
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
        let context1 = HandlerContext::new(executor);
        let context2 = context1.clone();

        // Both contexts should share the same executor
        assert!(Arc::ptr_eq(&context1.executor, &context2.executor));
    }
}
