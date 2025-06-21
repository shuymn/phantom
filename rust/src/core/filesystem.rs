use async_trait::async_trait;
use std::fs::{Metadata, Permissions};
use std::path::{Path, PathBuf};
use tokio::fs::DirEntry;

use crate::core::result::Result;
use crate::core::sealed::Sealed;

/// File system abstraction trait for testing and modularity
///
/// This trait is sealed to prevent downstream implementations
#[async_trait]
pub trait FileSystem: Sealed + Send + Sync {
    /// Check if a path exists
    async fn exists(&self, path: &Path) -> Result<bool>;

    /// Get metadata for a path
    async fn metadata(&self, path: &Path) -> Result<Metadata>;

    /// Check if a path is a file
    async fn is_file(&self, path: &Path) -> Result<bool>;

    /// Check if a path is a directory
    async fn is_dir(&self, path: &Path) -> Result<bool>;

    /// Create a directory
    async fn create_dir(&self, path: &Path) -> Result<()>;

    /// Create a directory and all its parents
    async fn create_dir_all(&self, path: &Path) -> Result<()>;

    /// Remove a directory and all its contents
    async fn remove_dir_all(&self, path: &Path) -> Result<()>;

    /// Read a directory
    async fn read_dir(&self, path: &Path) -> Result<Vec<DirEntry>>;

    /// Read a file to string
    async fn read_to_string(&self, path: &Path) -> Result<String>;

    /// Write a string to a file
    async fn write(&self, path: &Path, contents: &str) -> Result<()>;

    /// Copy a file
    async fn copy(&self, from: &Path, to: &Path) -> Result<u64>;

    /// Set permissions on a file
    async fn set_permissions(&self, path: &Path, perms: Permissions) -> Result<()>;

    /// Get the current directory
    fn current_dir(&self) -> Result<PathBuf>;

    /// Set the current directory
    fn set_current_dir(&self, path: &Path) -> Result<()>;

    /// Get the user's home directory
    fn home_dir(&self) -> Option<PathBuf>;

    /// Canonicalize a path
    async fn canonicalize(&self, path: &Path) -> Result<PathBuf>;
}
