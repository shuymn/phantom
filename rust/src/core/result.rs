use crate::core::error::PhantomError;

/// Type alias for Result with PhantomError as the error type
pub type Result<T> = std::result::Result<T, PhantomError>;