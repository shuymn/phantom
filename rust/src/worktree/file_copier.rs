use crate::worktree::errors::WorktreeError;
use crate::Result;
use std::path::Path;
use tokio::fs;
use tracing::debug;

/// Result of a file copy operation
#[derive(Debug, Clone)]
pub struct CopyFileResult {
    pub copied_files: Vec<String>,
    pub skipped_files: Vec<String>,
}

/// Copy multiple files from source directory to target directory
pub async fn copy_files(
    source_dir: &Path,
    target_dir: &Path,
    files: &[String],
) -> Result<CopyFileResult> {
    let mut copied_files = Vec::new();
    let mut skipped_files = Vec::new();

    for file in files {
        let source_path = source_dir.join(file);
        let target_path = target_dir.join(file);

        match copy_single_file(&source_path, &target_path, file).await {
            Ok(true) => {
                copied_files.push(file.clone());
            }
            Ok(false) => {
                skipped_files.push(file.clone());
            }
            Err(e) => {
                return Err(WorktreeError::FileOperation(format!(
                    "Failed to copy {}: {}",
                    file, e
                ))
                .into());
            }
        }
    }

    debug!("Copied {} files, skipped {} files", copied_files.len(), skipped_files.len());

    Ok(CopyFileResult { copied_files, skipped_files })
}

/// Copy a single file, creating parent directories as needed
async fn copy_single_file(source: &Path, target: &Path, file_name: &str) -> Result<bool> {
    // Check if source exists and is a file
    match fs::metadata(source).await {
        Ok(metadata) => {
            if !metadata.is_file() {
                debug!("Skipping '{}': not a file", file_name);
                return Ok(false);
            }
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            debug!("Skipping '{}': file not found", file_name);
            return Ok(false);
        }
        Err(e) => {
            return Err(WorktreeError::FileOperation(format!(
                "Failed to check metadata for '{}': {}",
                file_name, e
            ))
            .into());
        }
    }

    // Create parent directory if needed
    if let Some(parent) = target.parent() {
        fs::create_dir_all(parent).await.map_err(|e| {
            WorktreeError::FileOperation(format!(
                "Failed to create directory for '{}': {}",
                file_name, e
            ))
        })?;
    }

    // Copy the file
    fs::copy(source, target).await.map_err(|e| {
        WorktreeError::FileOperation(format!("Failed to copy '{}': {}", file_name, e))
    })?;

    debug!("Copied file: {}", file_name);
    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use tokio::fs;

    #[tokio::test]
    async fn test_copy_files_basic() {
        let source_dir = TempDir::new().unwrap();
        let target_dir = TempDir::new().unwrap();

        // Create test files
        let file1 = source_dir.path().join("file1.txt");
        let file2 = source_dir.path().join("file2.txt");
        fs::write(&file1, "content1").await.unwrap();
        fs::write(&file2, "content2").await.unwrap();

        let files = vec!["file1.txt".to_string(), "file2.txt".to_string()];
        let result = copy_files(source_dir.path(), target_dir.path(), &files).await.unwrap();

        assert_eq!(result.copied_files.len(), 2);
        assert_eq!(result.skipped_files.len(), 0);

        // Verify files were copied
        let target1 = target_dir.path().join("file1.txt");
        let target2 = target_dir.path().join("file2.txt");
        assert!(target1.exists());
        assert!(target2.exists());
        assert_eq!(fs::read_to_string(target1).await.unwrap(), "content1");
        assert_eq!(fs::read_to_string(target2).await.unwrap(), "content2");
    }

    #[tokio::test]
    async fn test_copy_files_with_subdirectories() {
        let source_dir = TempDir::new().unwrap();
        let target_dir = TempDir::new().unwrap();

        // Create test files in subdirectories
        let subdir = source_dir.path().join("subdir");
        fs::create_dir(&subdir).await.unwrap();
        let file = subdir.join("file.txt");
        fs::write(&file, "content").await.unwrap();

        let files = vec!["subdir/file.txt".to_string()];
        let result = copy_files(source_dir.path(), target_dir.path(), &files).await.unwrap();

        assert_eq!(result.copied_files.len(), 1);
        assert_eq!(result.skipped_files.len(), 0);

        // Verify file was copied with directory structure
        let target_file = target_dir.path().join("subdir/file.txt");
        assert!(target_file.exists());
        assert_eq!(fs::read_to_string(target_file).await.unwrap(), "content");
    }

    #[tokio::test]
    async fn test_copy_files_skip_nonexistent() {
        let source_dir = TempDir::new().unwrap();
        let target_dir = TempDir::new().unwrap();

        // Create one file, reference two
        let file1 = source_dir.path().join("exists.txt");
        fs::write(&file1, "content").await.unwrap();

        let files = vec!["exists.txt".to_string(), "missing.txt".to_string()];
        let result = copy_files(source_dir.path(), target_dir.path(), &files).await.unwrap();

        assert_eq!(result.copied_files.len(), 1);
        assert_eq!(result.copied_files[0], "exists.txt");
        assert_eq!(result.skipped_files.len(), 1);
        assert_eq!(result.skipped_files[0], "missing.txt");
    }

    #[tokio::test]
    async fn test_copy_files_skip_directories() {
        let source_dir = TempDir::new().unwrap();
        let target_dir = TempDir::new().unwrap();

        // Create a directory instead of a file
        let dir = source_dir.path().join("dir");
        fs::create_dir(&dir).await.unwrap();

        let files = vec!["dir".to_string()];
        let result = copy_files(source_dir.path(), target_dir.path(), &files).await.unwrap();

        assert_eq!(result.copied_files.len(), 0);
        assert_eq!(result.skipped_files.len(), 1);
    }

    #[tokio::test]
    async fn test_copy_files_empty_list() {
        let source_dir = TempDir::new().unwrap();
        let target_dir = TempDir::new().unwrap();

        let files = vec![];
        let result = copy_files(source_dir.path(), target_dir.path(), &files).await.unwrap();

        assert_eq!(result.copied_files.len(), 0);
        assert_eq!(result.skipped_files.len(), 0);
    }

    #[tokio::test]
    async fn test_copy_files_overwrite_existing() {
        let source_dir = TempDir::new().unwrap();
        let target_dir = TempDir::new().unwrap();

        // Create source file
        let file = source_dir.path().join("file.txt");
        fs::write(&file, "new content").await.unwrap();

        // Create existing target file
        let target_file = target_dir.path().join("file.txt");
        fs::write(&target_file, "old content").await.unwrap();

        let files = vec!["file.txt".to_string()];
        let result = copy_files(source_dir.path(), target_dir.path(), &files).await.unwrap();

        assert_eq!(result.copied_files.len(), 1);
        assert_eq!(result.skipped_files.len(), 0);

        // Verify file was overwritten
        assert_eq!(fs::read_to_string(target_file).await.unwrap(), "new content");
    }

    #[tokio::test]
    async fn test_copy_files_nested_directory_creation() {
        let source_dir = TempDir::new().unwrap();
        let target_dir = TempDir::new().unwrap();

        // Create deeply nested file
        let nested_dir = source_dir.path().join("a/b/c");
        fs::create_dir_all(&nested_dir).await.unwrap();
        let file = nested_dir.join("file.txt");
        fs::write(&file, "nested content").await.unwrap();

        let files = vec!["a/b/c/file.txt".to_string()];
        let result = copy_files(source_dir.path(), target_dir.path(), &files).await.unwrap();

        assert_eq!(result.copied_files.len(), 1);
        assert_eq!(result.skipped_files.len(), 0);

        // Verify nested structure was created
        let target_file = target_dir.path().join("a/b/c/file.txt");
        assert!(target_file.exists());
        assert_eq!(fs::read_to_string(target_file).await.unwrap(), "nested content");
    }

    #[tokio::test]
    async fn test_copy_file_result_debug() {
        let result = CopyFileResult {
            copied_files: vec!["file1.txt".to_string(), "file2.txt".to_string()],
            skipped_files: vec!["skip.txt".to_string()],
        };

        let debug_str = format!("{:?}", result);
        assert!(debug_str.contains("CopyFileResult"));
        assert!(debug_str.contains("copied_files"));
        assert!(debug_str.contains("file1.txt"));
        assert!(debug_str.contains("skipped_files"));
    }

    #[tokio::test]
    async fn test_copy_file_result_clone() {
        let result = CopyFileResult {
            copied_files: vec!["file1.txt".to_string()],
            skipped_files: vec!["file2.txt".to_string()],
        };

        let cloned = result.clone();
        assert_eq!(result.copied_files, cloned.copied_files);
        assert_eq!(result.skipped_files, cloned.skipped_files);
    }

    #[tokio::test]
    async fn test_copy_files_with_special_characters() {
        let source_dir = TempDir::new().unwrap();
        let target_dir = TempDir::new().unwrap();

        // Create file with special characters in name
        let file = source_dir.path().join("file-with-dashes.txt");
        fs::write(&file, "content").await.unwrap();

        let files = vec!["file-with-dashes.txt".to_string()];
        let result = copy_files(source_dir.path(), target_dir.path(), &files).await.unwrap();

        assert_eq!(result.copied_files.len(), 1);
        assert_eq!(result.copied_files[0], "file-with-dashes.txt");

        let target_file = target_dir.path().join("file-with-dashes.txt");
        assert!(target_file.exists());
    }

    #[tokio::test]
    async fn test_copy_files_permission_preserved() {
        use std::os::unix::fs::PermissionsExt;

        let source_dir = TempDir::new().unwrap();
        let target_dir = TempDir::new().unwrap();

        // Create file with specific permissions
        let file = source_dir.path().join("executable.sh");
        fs::write(&file, "#!/bin/bash\necho test").await.unwrap();
        
        // Set executable permission
        let mut perms = fs::metadata(&file).await.unwrap().permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&file, perms).await.unwrap();

        let files = vec!["executable.sh".to_string()];
        let result = copy_files(source_dir.path(), target_dir.path(), &files).await.unwrap();

        assert_eq!(result.copied_files.len(), 1);

        // Note: fs::copy preserves permissions on Unix systems
        let target_file = target_dir.path().join("executable.sh");
        let target_perms = fs::metadata(&target_file).await.unwrap().permissions();
        assert_eq!(target_perms.mode() & 0o777, 0o755);
    }
}
