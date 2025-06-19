use async_trait::async_trait;
use std::collections::HashMap;
use std::fs::{Metadata, Permissions};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use tokio::fs::DirEntry;

use crate::core::error::PhantomError;
use crate::core::filesystem::FileSystem;
use crate::core::result::Result;

#[derive(Debug, Clone)]
pub enum FileSystemOperation {
    Exists,
    Metadata,
    IsFile,
    IsDir,
    CreateDir,
    CreateDirAll,
    RemoveDirAll,
    ReadDir,
    ReadToString,
    Write,
    Copy,
    SetPermissions,
    CurrentDir,
    SetCurrentDir,
    HomeDir,
    Canonicalize,
}

#[derive(Debug)]
pub struct FileSystemExpectation {
    pub operation: FileSystemOperation,
    pub path: Option<PathBuf>,
    pub from_path: Option<PathBuf>,
    pub to_path: Option<PathBuf>,
    pub contents: Option<String>,
    pub result: Result<MockResult>,
}

#[derive(Debug, Clone)]
pub enum MockResult {
    Bool(bool),
    Metadata(MockMetadata),
    DirEntries(Vec<MockDirEntry>),
    String(String),
    U64(u64),
    Unit,
    PathBuf(PathBuf),
    OptionPathBuf(Option<PathBuf>),
}

#[derive(Debug, Clone)]
pub struct MockMetadata {
    pub is_file: bool,
    pub is_dir: bool,
}

#[derive(Debug, Clone)]
pub struct MockDirEntry {
    pub path: PathBuf,
}

#[derive(Debug, Clone)]
pub struct MockFileSystem {
    expectations: Arc<Mutex<HashMap<String, Vec<FileSystemExpectation>>>>,
    current_dir: Arc<Mutex<PathBuf>>,
}

impl MockFileSystem {
    pub fn new() -> Self {
        Self {
            expectations: Arc::new(Mutex::new(HashMap::new())),
            current_dir: Arc::new(Mutex::new(PathBuf::from("/mock/dir"))),
        }
    }

    pub fn expect(&self, expectation: FileSystemExpectation) {
        let key = self.expectation_key(&expectation);
        let mut expectations = self.expectations.lock().unwrap();
        expectations.entry(key).or_insert_with(Vec::new).push(expectation);
    }

    fn expectation_key(&self, expectation: &FileSystemExpectation) -> String {
        match &expectation.operation {
            FileSystemOperation::Copy => {
                format!(
                    "{:?}:{:?}:{:?}",
                    expectation.operation, expectation.from_path, expectation.to_path
                )
            }
            FileSystemOperation::Write => {
                format!(
                    "{:?}:{:?}:{:?}",
                    expectation.operation, expectation.path, expectation.contents
                )
            }
            FileSystemOperation::CurrentDir | FileSystemOperation::HomeDir => {
                format!("{:?}", expectation.operation)
            }
            _ => format!("{:?}:{:?}", expectation.operation, expectation.path),
        }
    }

    fn find_expectation(
        &self,
        operation: FileSystemOperation,
        path: Option<&Path>,
        from_path: Option<&Path>,
        to_path: Option<&Path>,
        contents: Option<&str>,
    ) -> Result<FileSystemExpectation> {
        let expectation = FileSystemExpectation {
            operation,
            path: path.map(|p| p.to_path_buf()),
            from_path: from_path.map(|p| p.to_path_buf()),
            to_path: to_path.map(|p| p.to_path_buf()),
            contents: contents.map(|s| s.to_string()),
            result: Ok(MockResult::Unit),
        };

        let key = self.expectation_key(&expectation);
        let mut expectations = self.expectations.lock().unwrap();

        if let Some(vec) = expectations.get_mut(&key) {
            if !vec.is_empty() {
                return Ok(vec.remove(0));
            }
        }

        Err(PhantomError::FileOperation(format!(
            "No expectation found for {:?} with path {:?}",
            expectation.operation, path
        )))
    }
}

#[async_trait]
impl FileSystem for MockFileSystem {
    async fn exists(&self, path: &Path) -> Result<bool> {
        let expectation =
            self.find_expectation(FileSystemOperation::Exists, Some(path), None, None, None)?;
        match expectation.result? {
            MockResult::Bool(b) => Ok(b),
            _ => Err(PhantomError::FileOperation("Unexpected result type for exists".to_string())),
        }
    }

    async fn metadata(&self, path: &Path) -> Result<Metadata> {
        let expectation =
            self.find_expectation(FileSystemOperation::Metadata, Some(path), None, None, None)?;
        match expectation.result {
            Ok(_) => {
                // We can't create real Metadata, so we'll return an error for now
                // In real usage, we might need to refactor to return our own metadata type
                Err(PhantomError::FileOperation("Mock metadata not fully implemented".to_string()))
            }
            Err(e) => Err(e),
        }
    }

    async fn is_file(&self, path: &Path) -> Result<bool> {
        let expectation =
            self.find_expectation(FileSystemOperation::IsFile, Some(path), None, None, None)?;
        match expectation.result? {
            MockResult::Bool(b) => Ok(b),
            _ => Err(PhantomError::FileOperation("Unexpected result type for is_file".to_string())),
        }
    }

    async fn is_dir(&self, path: &Path) -> Result<bool> {
        let expectation =
            self.find_expectation(FileSystemOperation::IsDir, Some(path), None, None, None)?;
        match expectation.result? {
            MockResult::Bool(b) => Ok(b),
            _ => Err(PhantomError::FileOperation("Unexpected result type for is_dir".to_string())),
        }
    }

    async fn create_dir(&self, path: &Path) -> Result<()> {
        let expectation =
            self.find_expectation(FileSystemOperation::CreateDir, Some(path), None, None, None)?;
        match expectation.result? {
            MockResult::Unit => Ok(()),
            _ => Err(PhantomError::FileOperation(
                "Unexpected result type for create_dir".to_string(),
            )),
        }
    }

    async fn create_dir_all(&self, path: &Path) -> Result<()> {
        let expectation =
            self.find_expectation(FileSystemOperation::CreateDirAll, Some(path), None, None, None)?;
        match expectation.result? {
            MockResult::Unit => Ok(()),
            _ => Err(PhantomError::FileOperation(
                "Unexpected result type for create_dir_all".to_string(),
            )),
        }
    }

    async fn remove_dir_all(&self, path: &Path) -> Result<()> {
        let expectation =
            self.find_expectation(FileSystemOperation::RemoveDirAll, Some(path), None, None, None)?;
        match expectation.result? {
            MockResult::Unit => Ok(()),
            _ => Err(PhantomError::FileOperation(
                "Unexpected result type for remove_dir_all".to_string(),
            )),
        }
    }

    async fn read_dir(&self, path: &Path) -> Result<Vec<DirEntry>> {
        let expectation =
            self.find_expectation(FileSystemOperation::ReadDir, Some(path), None, None, None)?;
        match expectation.result {
            Ok(_) => {
                // We can't create real DirEntry objects, so we'll return an error for now
                // In real usage, we might need to refactor to return our own directory entry type
                Err(PhantomError::FileOperation("Mock read_dir not fully implemented".to_string()))
            }
            Err(e) => Err(e),
        }
    }

    async fn read_to_string(&self, path: &Path) -> Result<String> {
        let expectation =
            self.find_expectation(FileSystemOperation::ReadToString, Some(path), None, None, None)?;
        match expectation.result? {
            MockResult::String(s) => Ok(s),
            _ => Err(PhantomError::FileOperation(
                "Unexpected result type for read_to_string".to_string(),
            )),
        }
    }

    async fn write(&self, path: &Path, contents: &str) -> Result<()> {
        let expectation = self.find_expectation(
            FileSystemOperation::Write,
            Some(path),
            None,
            None,
            Some(contents),
        )?;
        match expectation.result? {
            MockResult::Unit => Ok(()),
            _ => Err(PhantomError::FileOperation("Unexpected result type for write".to_string())),
        }
    }

    async fn copy(&self, from: &Path, to: &Path) -> Result<u64> {
        let expectation =
            self.find_expectation(FileSystemOperation::Copy, None, Some(from), Some(to), None)?;
        match expectation.result? {
            MockResult::U64(n) => Ok(n),
            _ => Err(PhantomError::FileOperation("Unexpected result type for copy".to_string())),
        }
    }

    async fn set_permissions(&self, path: &Path, _perms: Permissions) -> Result<()> {
        let expectation = self.find_expectation(
            FileSystemOperation::SetPermissions,
            Some(path),
            None,
            None,
            None,
        )?;
        match expectation.result? {
            MockResult::Unit => Ok(()),
            _ => Err(PhantomError::FileOperation(
                "Unexpected result type for set_permissions".to_string(),
            )),
        }
    }

    fn current_dir(&self) -> Result<PathBuf> {
        let expectation =
            self.find_expectation(FileSystemOperation::CurrentDir, None, None, None, None)?;
        match expectation.result? {
            MockResult::PathBuf(p) => Ok(p),
            _ => Err(PhantomError::FileOperation(
                "Unexpected result type for current_dir".to_string(),
            )),
        }
    }

    fn set_current_dir(&self, path: &Path) -> Result<()> {
        let expectation = self.find_expectation(
            FileSystemOperation::SetCurrentDir,
            Some(path),
            None,
            None,
            None,
        )?;
        match expectation.result? {
            MockResult::Unit => {
                *self.current_dir.lock().unwrap() = path.to_path_buf();
                Ok(())
            }
            _ => Err(PhantomError::FileOperation(
                "Unexpected result type for set_current_dir".to_string(),
            )),
        }
    }

    fn home_dir(&self) -> Option<PathBuf> {
        match self.find_expectation(FileSystemOperation::HomeDir, None, None, None, None) {
            Ok(expectation) => match expectation.result {
                Ok(MockResult::OptionPathBuf(opt)) => opt,
                _ => None,
            },
            Err(_) => None,
        }
    }

    async fn canonicalize(&self, path: &Path) -> Result<PathBuf> {
        let expectation =
            self.find_expectation(FileSystemOperation::Canonicalize, Some(path), None, None, None)?;
        match expectation.result? {
            MockResult::PathBuf(p) => Ok(p),
            _ => Err(PhantomError::FileOperation(
                "Unexpected result type for canonicalize".to_string(),
            )),
        }
    }
}

impl Default for MockFileSystem {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_mock_filesystem_exists() {
        let mock_fs = MockFileSystem::new();

        // Set up expectation
        mock_fs.expect(FileSystemExpectation {
            operation: FileSystemOperation::Exists,
            path: Some(PathBuf::from("/test/path")),
            from_path: None,
            to_path: None,
            contents: None,
            result: Ok(MockResult::Bool(true)),
        });

        // Test
        let result = mock_fs.exists(Path::new("/test/path")).await;
        assert!(result.unwrap());
    }

    #[tokio::test]
    async fn test_mock_filesystem_write() {
        let mock_fs = MockFileSystem::new();

        // Set up expectation
        mock_fs.expect(FileSystemExpectation {
            operation: FileSystemOperation::Write,
            path: Some(PathBuf::from("/test/file.txt")),
            from_path: None,
            to_path: None,
            contents: Some("Hello, world!".to_string()),
            result: Ok(MockResult::Unit),
        });

        // Test
        let result = mock_fs.write(Path::new("/test/file.txt"), "Hello, world!").await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_mock_filesystem_copy() {
        let mock_fs = MockFileSystem::new();

        // Set up expectation
        mock_fs.expect(FileSystemExpectation {
            operation: FileSystemOperation::Copy,
            path: None,
            from_path: Some(PathBuf::from("/source/file.txt")),
            to_path: Some(PathBuf::from("/dest/file.txt")),
            contents: None,
            result: Ok(MockResult::U64(1024)),
        });

        // Test
        let result = mock_fs.copy(Path::new("/source/file.txt"), Path::new("/dest/file.txt")).await;
        assert_eq!(result.unwrap(), 1024);
    }
}
