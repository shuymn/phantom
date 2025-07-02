use super::*;
use crate::core::sealed::Sealed;
use std::sync::{Arc, Mutex};

/// Mock exit handler for testing
#[derive(Clone)]
pub struct MockExitHandler {
    exits: Arc<Mutex<Vec<i32>>>,
}

impl Default for MockExitHandler {
    fn default() -> Self {
        Self::new()
    }
}

impl MockExitHandler {
    pub fn new() -> Self {
        Self { exits: Arc::new(Mutex::new(Vec::new())) }
    }

    /// Get the exit codes that were called
    pub fn get_exits(&self) -> Vec<i32> {
        self.exits.lock().unwrap().clone()
    }

    /// Check if exit was called with a specific code
    pub fn was_exit_called(&self, code: i32) -> bool {
        self.exits.lock().unwrap().contains(&code)
    }

    /// Check if exit was called at all
    pub fn exit_called(&self) -> bool {
        !self.exits.lock().unwrap().is_empty()
    }
}

// Implement the sealed trait
impl Sealed for MockExitHandler {}

#[async_trait]
impl ExitHandler for MockExitHandler {
    fn exit(&self, code: i32) -> ! {
        self.exits.lock().unwrap().push(code);
        panic!("MockExitHandler::exit called with code {}", code);
    }
}
