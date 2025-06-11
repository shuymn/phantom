pub mod errors;
pub mod types;
pub mod validate;

// Re-export commonly used types
pub use errors::ConfigError;
pub use types::{Multiplexer, PhantomConfig, PostCreateConfig};
pub use validate::validate_config;