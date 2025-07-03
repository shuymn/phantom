pub mod cli;
pub mod config;
pub mod core;
pub mod git;
pub mod process;
pub mod worktree;

#[cfg(test)]
pub mod test_utils;

pub use crate::core::error::PhantomError;
pub use crate::core::result::Result;
