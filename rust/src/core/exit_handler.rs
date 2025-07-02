use crate::core::sealed::Sealed;
use async_trait::async_trait;

/// Trait for handling process exits
///
/// This trait is sealed to prevent downstream implementations
#[async_trait]
pub trait ExitHandler: Sealed + Send + Sync {
    /// Exit the process with the given code
    fn exit(&self, code: i32) -> !;
}

/// Real exit handler that calls std::process::exit
#[derive(Clone)]
pub struct RealExitHandler;

impl RealExitHandler {
    pub fn new() -> Self {
        Self
    }
}

impl Default for RealExitHandler {
    fn default() -> Self {
        Self::new()
    }
}

// Implement the sealed trait
impl Sealed for RealExitHandler {}

#[async_trait]
impl ExitHandler for RealExitHandler {
    fn exit(&self, code: i32) -> ! {
        std::process::exit(code);
    }
}

#[cfg(test)]
mod mock_exit_handler;
#[cfg(test)]
pub use mock_exit_handler::MockExitHandler;
