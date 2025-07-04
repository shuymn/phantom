use async_trait::async_trait;
use std::fs::{Metadata, Permissions};
use std::path::{Path, PathBuf};
use tokio::fs::{self, DirEntry};

use crate::core::error::PhantomError;
use crate::core::filesystem::FileSystem;
use crate::core::result::Result;
use crate::core::sealed::Sealed;

#[derive(Debug, Clone)]
pub struct RealFileSystem;

impl RealFileSystem {
    pub fn new() -> Self {
        Self
    }
}

// Implement the sealed trait
impl Sealed for RealFileSystem {}

#[async_trait]
impl FileSystem for RealFileSystem {
    async fn exists(&self, path: &Path) -> Result<bool> {
        Ok(path.exists())
    }

    async fn metadata(&self, path: &Path) -> Result<Metadata> {
        fs::metadata(path).await.map_err(|e| PhantomError::FileOperationFailed {
            operation: "metadata".to_string(),
            path: path.to_path_buf(),
            reason: e.to_string(),
        })
    }

    async fn is_file(&self, path: &Path) -> Result<bool> {
        match self.metadata(path).await {
            Ok(meta) => Ok(meta.is_file()),
            Err(_) => Ok(false),
        }
    }

    async fn is_dir(&self, path: &Path) -> Result<bool> {
        match self.metadata(path).await {
            Ok(meta) => Ok(meta.is_dir()),
            Err(_) => Ok(false),
        }
    }

    async fn create_dir(&self, path: &Path) -> Result<()> {
        fs::create_dir(path).await.map_err(|e| PhantomError::FileOperationFailed {
            operation: "create_dir".to_string(),
            path: path.to_path_buf(),
            reason: e.to_string(),
        })
    }

    async fn create_dir_all(&self, path: &Path) -> Result<()> {
        fs::create_dir_all(path).await.map_err(|e| PhantomError::FileOperationFailed {
            operation: "create_dir".to_string(),
            path: path.to_path_buf(),
            reason: e.to_string(),
        })
    }

    async fn remove_dir_all(&self, path: &Path) -> Result<()> {
        fs::remove_dir_all(path).await.map_err(|e| PhantomError::FileOperationFailed {
            operation: "remove_dir_all".to_string(),
            path: path.to_path_buf(),
            reason: e.to_string(),
        })
    }

    async fn read_dir(&self, path: &Path) -> Result<Vec<DirEntry>> {
        let mut entries = Vec::new();
        let mut dir = fs::read_dir(path).await.map_err(|e| PhantomError::FileOperationFailed {
            operation: "read_dir".to_string(),
            path: path.to_path_buf(),
            reason: e.to_string(),
        })?;

        while let Some(entry) =
            dir.next_entry().await.map_err(|e| PhantomError::FileOperationFailed {
                operation: "read_dir_entry".to_string(),
                path: path.to_path_buf(),
                reason: e.to_string(),
            })?
        {
            entries.push(entry);
        }

        Ok(entries)
    }

    async fn read_to_string(&self, path: &Path) -> Result<String> {
        fs::read_to_string(path).await.map_err(|e| PhantomError::FileOperationFailed {
            operation: "read_to_string".to_string(),
            path: path.to_path_buf(),
            reason: e.to_string(),
        })
    }

    async fn write(&self, path: &Path, contents: &str) -> Result<()> {
        fs::write(path, contents).await.map_err(|e| PhantomError::FileOperationFailed {
            operation: "write".to_string(),
            path: path.to_path_buf(),
            reason: e.to_string(),
        })
    }

    async fn copy(&self, from: &Path, to: &Path) -> Result<u64> {
        fs::copy(from, to).await.map_err(|e| PhantomError::FileOperationFailed {
            operation: format!("copy from {from:?} to {to:?}"),
            path: from.to_path_buf(),
            reason: e.to_string(),
        })
    }

    async fn set_permissions(&self, path: &Path, perms: Permissions) -> Result<()> {
        fs::set_permissions(path, perms).await.map_err(|e| PhantomError::FileOperationFailed {
            operation: "set_permissions".to_string(),
            path: path.to_path_buf(),
            reason: e.to_string(),
        })
    }

    fn current_dir(&self) -> Result<PathBuf> {
        std::env::current_dir().map_err(|e| PhantomError::FileOperationFailed {
            operation: "current_dir".to_string(),
            path: PathBuf::from("."),
            reason: e.to_string(),
        })
    }

    fn set_current_dir(&self, path: &Path) -> Result<()> {
        std::env::set_current_dir(path).map_err(|e| PhantomError::FileOperationFailed {
            operation: "set_current_dir".to_string(),
            path: path.to_path_buf(),
            reason: e.to_string(),
        })
    }

    fn home_dir(&self) -> Option<PathBuf> {
        // For now, return None as we don't have a standard way to get home directory
        // This could be replaced with proper home directory detection
        None
    }

    async fn canonicalize(&self, path: &Path) -> Result<PathBuf> {
        fs::canonicalize(path).await.map_err(|e| PhantomError::FileOperationFailed {
            operation: "canonicalize".to_string(),
            path: path.to_path_buf(),
            reason: e.to_string(),
        })
    }
}

impl Default for RealFileSystem {
    fn default() -> Self {
        Self::new()
    }
}
