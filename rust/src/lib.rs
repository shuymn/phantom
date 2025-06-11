pub mod cli;
pub mod core;
pub mod git;
pub mod process;
pub mod worktree;

pub use crate::core::error::PhantomError;
pub use crate::core::result::Result;