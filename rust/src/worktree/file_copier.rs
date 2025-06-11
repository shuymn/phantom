use crate::worktree::errors::WorktreeError;
use crate::Result;
use std::path::Path;
use tokio::fs;
use tracing::{debug, warn};

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

    debug!(
        "Copied {} files, skipped {} files",
        copied_files.len(),
        skipped_files.len()
    );

    Ok(CopyFileResult {
        copied_files,
        skipped_files,
    })
}

/// Copy a single file, creating parent directories as needed
async fn copy_single_file(source: &Path, target: &Path, file_name: &str) -> Result<bool> {
    // Check if source exists and is a file
    match fs::metadata(source).await {
        Ok(metadata) => {
            if !metadata.is_file() {
                debug!("Skipping non-file: {}", file_name);
                return Ok(false);
            }
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            debug!("Source file not found: {}", file_name);
            return Ok(false);
        }
        Err(e) => {
            return Err(e.into());
        }
    }

    // Create parent directory
    if let Some(parent) = target.parent() {
        fs::create_dir_all(parent).await.map_err(|e| {
            WorktreeError::FileOperation(format!(
                "Failed to create directory {}: {}",
                parent.display(),
                e
            ))
        })?;
    }

    // Copy the file
    fs::copy(source, target).await.map_err(|e| {
        WorktreeError::FileOperation(format!("Failed to copy file {}: {}", file_name, e))
    })?;

    debug!("Copied file: {}", file_name);
    Ok(true)
}

/// Copy all files from source directory to target directory recursively
pub async fn copy_directory(source_dir: &Path, target_dir: &Path) -> Result<CopyFileResult> {
    let mut copied_files = Vec::new();
    let mut skipped_files = Vec::new();

    copy_directory_recursive(
        source_dir,
        target_dir,
        source_dir,
        &mut copied_files,
        &mut skipped_files,
    )
    .await?;

    debug!(
        "Copied {} files, skipped {} files",
        copied_files.len(),
        skipped_files.len()
    );

    Ok(CopyFileResult {
        copied_files,
        skipped_files,
    })
}

/// Recursively copy directory contents
async fn copy_directory_recursive(
    source_dir: &Path,
    target_dir: &Path,
    base_source_dir: &Path,
    copied_files: &mut Vec<String>,
    skipped_files: &mut Vec<String>,
) -> Result<()> {
    // Create the target directory
    fs::create_dir_all(target_dir).await.map_err(|e| {
        WorktreeError::FileOperation(format!(
            "Failed to create directory {}: {}",
            target_dir.display(),
            e
        ))
    })?;

    // Read directory entries
    let mut entries = fs::read_dir(source_dir).await.map_err(|e| {
        WorktreeError::FileOperation(format!(
            "Failed to read directory {}: {}",
            source_dir.display(),
            e
        ))
    })?;

    while let Some(entry) = entries.next_entry().await.map_err(|e| {
        WorktreeError::FileOperation(format!("Failed to read directory entry: {}", e))
    })? {
        let path = entry.path();
        let file_name = match path.file_name() {
            Some(name) => name.to_string_lossy().to_string(),
            None => continue,
        };

        // Skip .git directory
        if file_name == ".git" {
            continue;
        }

        let metadata = entry.metadata().await.map_err(|e| {
            WorktreeError::FileOperation(format!("Failed to get metadata for {}: {}", file_name, e))
        })?;

        if metadata.is_file() {
            let target_path = target_dir.join(&file_name);
            let relative_path = path
                .strip_prefix(base_source_dir)
                .unwrap_or(&path)
                .to_string_lossy()
                .to_string();

            match fs::copy(&path, &target_path).await {
                Ok(_) => {
                    copied_files.push(relative_path);
                }
                Err(e) => {
                    warn!("Failed to copy {}: {}", relative_path, e);
                    skipped_files.push(relative_path);
                }
            }
        } else if metadata.is_dir() {
            let target_subdir = target_dir.join(&file_name);
            Box::pin(copy_directory_recursive(
                &path,
                &target_subdir,
                base_source_dir,
                copied_files,
                skipped_files,
            ))
            .await?;
        }
    }

    Ok(())
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
    async fn test_copy_directory_basic() {
        let source_dir = TempDir::new().unwrap();
        let target_dir = TempDir::new().unwrap();

        // Create test structure
        fs::write(source_dir.path().join("file1.txt"), "content1").await.unwrap();
        fs::write(source_dir.path().join("file2.txt"), "content2").await.unwrap();
        let subdir = source_dir.path().join("subdir");
        fs::create_dir(&subdir).await.unwrap();
        fs::write(subdir.join("file3.txt"), "content3").await.unwrap();

        let result = copy_directory(source_dir.path(), target_dir.path()).await.unwrap();

        assert_eq!(result.copied_files.len(), 3);
        assert_eq!(result.skipped_files.len(), 0);

        // Verify all files were copied
        assert!(target_dir.path().join("file1.txt").exists());
        assert!(target_dir.path().join("file2.txt").exists());
        assert!(target_dir.path().join("subdir/file3.txt").exists());
    }

    #[tokio::test]
    async fn test_copy_directory_skip_git() {
        let source_dir = TempDir::new().unwrap();
        let target_dir = TempDir::new().unwrap();

        // Create test structure with .git directory
        fs::write(source_dir.path().join("file.txt"), "content").await.unwrap();
        let git_dir = source_dir.path().join(".git");
        fs::create_dir(&git_dir).await.unwrap();
        fs::write(git_dir.join("config"), "git config").await.unwrap();

        let result = copy_directory(source_dir.path(), target_dir.path()).await.unwrap();

        assert_eq!(result.copied_files.len(), 1);
        assert_eq!(result.copied_files[0], "file.txt");
        assert_eq!(result.skipped_files.len(), 0);

        // Verify .git directory was not copied
        assert!(target_dir.path().join("file.txt").exists());
        assert!(!target_dir.path().join(".git").exists());
    }
}