use crate::worktree::errors::WorktreeError;
use crate::worktree::gitignore::{
    default_ignore_patterns, load_gitignore_hierarchy, GitignoreMatcher,
};
use crate::Result;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::fs;
use tokio::sync::Semaphore;
use tokio::task::JoinSet;
use tracing::{debug, info, warn};

/// Configuration for parallel file copying
#[derive(Debug, Clone)]
pub struct ParallelCopyConfig {
    /// Maximum number of concurrent file operations
    pub max_concurrent_ops: usize,
    /// Whether to use gitignore patterns
    pub use_gitignore: bool,
    /// Buffer size for the file queue channel
    pub channel_buffer_size: usize,
}

impl Default for ParallelCopyConfig {
    fn default() -> Self {
        Self { max_concurrent_ops: 32, use_gitignore: true, channel_buffer_size: 1000 }
    }
}

/// Result of a parallel copy operation
#[derive(Debug, Clone)]
pub struct ParallelCopyResult {
    pub copied_files: Vec<String>,
    pub skipped_files: Vec<String>,
    pub errors: Vec<String>,
}

/// File copy task
#[derive(Debug, Clone)]
struct CopyTask {
    source: PathBuf,
    target: PathBuf,
    relative_path: String,
}

/// Copy files in parallel with configurable concurrency
pub async fn copy_directory_parallel(
    source_dir: &Path,
    target_dir: &Path,
    config: ParallelCopyConfig,
) -> Result<ParallelCopyResult> {
    info!(
        "Starting parallel copy from {} to {} with max {} concurrent operations",
        source_dir.display(),
        target_dir.display(),
        config.max_concurrent_ops
    );

    // Load gitignore patterns if enabled
    let gitignore = if config.use_gitignore {
        let mut matcher = load_gitignore_hierarchy(source_dir).await?;
        let defaults = default_ignore_patterns();
        matcher.extend(&defaults);
        Some(Arc::new(matcher))
    } else {
        None
    };

    // Collect all files to copy first
    let mut copy_tasks = Vec::new();
    collect_copy_tasks(source_dir, target_dir, source_dir, gitignore.as_deref(), &mut copy_tasks)
        .await?;

    debug!("Collected {} files to copy", copy_tasks.len());

    // Create semaphore for concurrency control
    let semaphore = Arc::new(Semaphore::new(config.max_concurrent_ops));
    let mut task_set = JoinSet::new();

    // Process tasks with concurrency limit
    for task in copy_tasks {
        let semaphore = semaphore.clone();
        task_set.spawn(async move {
            let _permit = semaphore.acquire().await.unwrap();
            copy_file_task(&task).await
        });
    }

    // Collect results
    let mut copied_files = Vec::new();
    let mut skipped_files = Vec::new();
    let mut errors = Vec::new();

    while let Some(result) = task_set.join_next().await {
        match result {
            Ok(CopyResult::Copied(path)) => copied_files.push(path),
            Ok(CopyResult::Skipped(path)) => skipped_files.push(path),
            Ok(CopyResult::Error(path, err)) => errors.push(format!("{}: {}", path, err)),
            Err(e) => errors.push(format!("Task join error: {}", e)),
        }
    }

    info!(
        "Parallel copy completed: {} copied, {} skipped, {} errors",
        copied_files.len(),
        skipped_files.len(),
        errors.len()
    );

    Ok(ParallelCopyResult { copied_files, skipped_files, errors })
}

/// Result of a single file copy operation
#[derive(Debug)]
enum CopyResult {
    Copied(String),
    #[allow(dead_code)]
    Skipped(String),
    Error(String, String),
}

/// Collect all copy tasks recursively
async fn collect_copy_tasks(
    source_dir: &Path,
    target_dir: &Path,
    base_source_dir: &Path,
    gitignore: Option<&GitignoreMatcher>,
    tasks: &mut Vec<CopyTask>,
) -> Result<()> {
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
        let relative_path = path.strip_prefix(base_source_dir).unwrap_or(&path).to_path_buf();

        let metadata = entry.metadata().await.map_err(|e| {
            WorktreeError::FileOperation(format!(
                "Failed to get metadata for {}: {}",
                path.display(),
                e
            ))
        })?;

        // Check gitignore patterns
        if let Some(gitignore) = gitignore {
            if gitignore.is_ignored(&relative_path, metadata.is_dir()) {
                debug!("Ignoring {} (matched gitignore pattern)", relative_path.display());
                continue;
            }
        }

        if metadata.is_file() {
            // Add file copy task
            let task = CopyTask {
                source: path.clone(),
                target: target_dir.join(&relative_path),
                relative_path: relative_path.to_string_lossy().to_string(),
            };
            tasks.push(task);
        } else if metadata.is_dir() {
            // Create target subdirectory
            let target_subdir = target_dir.join(&relative_path);
            fs::create_dir_all(&target_subdir).await.map_err(|e| {
                WorktreeError::FileOperation(format!(
                    "Failed to create directory {}: {}",
                    target_subdir.display(),
                    e
                ))
            })?;

            // Recursively collect from subdirectory
            Box::pin(collect_copy_tasks(&path, target_dir, base_source_dir, gitignore, tasks))
                .await?;
        }
    }

    Ok(())
}

/// Copy a single file
async fn copy_file_task(task: &CopyTask) -> CopyResult {
    // Create parent directory if needed
    if let Some(parent) = task.target.parent() {
        if let Err(e) = fs::create_dir_all(parent).await {
            return CopyResult::Error(
                task.relative_path.clone(),
                format!("Failed to create directory: {}", e),
            );
        }
    }

    // Copy the file
    match fs::copy(&task.source, &task.target).await {
        Ok(_) => {
            debug!("Copied: {}", task.relative_path);
            CopyResult::Copied(task.relative_path.clone())
        }
        Err(e) => {
            warn!("Failed to copy {}: {}", task.relative_path, e);
            CopyResult::Error(task.relative_path.clone(), e.to_string())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_parallel_copy_basic() {
        let source_dir = TempDir::new().unwrap();
        let target_dir = TempDir::new().unwrap();

        // Create test files
        for i in 0..10 {
            fs::write(source_dir.path().join(format!("file{}.txt", i)), format!("content{}", i))
                .await
                .unwrap();
        }

        let config = ParallelCopyConfig {
            max_concurrent_ops: 4,
            use_gitignore: false,
            channel_buffer_size: 100,
        };

        let result =
            copy_directory_parallel(source_dir.path(), target_dir.path(), config).await.unwrap();

        assert_eq!(result.copied_files.len(), 10);
        assert_eq!(result.skipped_files.len(), 0);
        assert_eq!(result.errors.len(), 0);

        // Verify all files were copied
        for i in 0..10 {
            let target_file = target_dir.path().join(format!("file{}.txt", i));
            assert!(target_file.exists());
            let content = fs::read_to_string(target_file).await.unwrap();
            assert_eq!(content, format!("content{}", i));
        }
    }

    #[tokio::test]
    async fn test_parallel_copy_with_subdirs() {
        let source_dir = TempDir::new().unwrap();
        let target_dir = TempDir::new().unwrap();

        // Create nested structure
        let sub1 = source_dir.path().join("sub1");
        let sub2 = source_dir.path().join("sub1/sub2");
        fs::create_dir_all(&sub2).await.unwrap();

        fs::write(source_dir.path().join("root.txt"), "root").await.unwrap();
        fs::write(sub1.join("sub1.txt"), "sub1").await.unwrap();
        fs::write(sub2.join("sub2.txt"), "sub2").await.unwrap();

        let config = ParallelCopyConfig {
            max_concurrent_ops: 2,
            use_gitignore: false,
            channel_buffer_size: 50,
        };

        let result =
            copy_directory_parallel(source_dir.path(), target_dir.path(), config).await.unwrap();

        assert_eq!(result.copied_files.len(), 3);
        assert!(target_dir.path().join("root.txt").exists());
        assert!(target_dir.path().join("sub1/sub1.txt").exists());
        assert!(target_dir.path().join("sub1/sub2/sub2.txt").exists());
    }

    #[tokio::test]
    async fn test_parallel_copy_with_gitignore() {
        let source_dir = TempDir::new().unwrap();
        let target_dir = TempDir::new().unwrap();

        // Create test files
        fs::write(source_dir.path().join("keep.txt"), "keep").await.unwrap();
        fs::write(source_dir.path().join("test.log"), "log").await.unwrap();
        fs::write(source_dir.path().join(".env"), "secret").await.unwrap();

        // Create .gitignore
        fs::write(source_dir.path().join(".gitignore"), "*.log\n.env\n").await.unwrap();

        let config = ParallelCopyConfig::default();

        let result =
            copy_directory_parallel(source_dir.path(), target_dir.path(), config).await.unwrap();

        // Should only copy non-ignored files
        assert!(result.copied_files.contains(&"keep.txt".to_string()));
        assert!(result.copied_files.contains(&".gitignore".to_string()));
        assert!(!result.copied_files.contains(&"test.log".to_string()));
        assert!(!result.copied_files.contains(&".env".to_string()));

        assert!(target_dir.path().join("keep.txt").exists());
        assert!(!target_dir.path().join("test.log").exists());
        assert!(!target_dir.path().join(".env").exists());
    }

    #[tokio::test]
    async fn test_parallel_copy_stress() {
        let source_dir = TempDir::new().unwrap();
        let target_dir = TempDir::new().unwrap();

        // Create many files to test concurrency
        for i in 0..100 {
            let subdir = source_dir.path().join(format!("dir{}", i % 10));
            fs::create_dir_all(&subdir).await.unwrap();
            fs::write(subdir.join(format!("file{}.txt", i)), format!("content{}", i))
                .await
                .unwrap();
        }

        let config = ParallelCopyConfig {
            max_concurrent_ops: 16,
            use_gitignore: false,
            channel_buffer_size: 200,
        };

        let result =
            copy_directory_parallel(source_dir.path(), target_dir.path(), config).await.unwrap();

        assert_eq!(result.copied_files.len(), 100);
        assert_eq!(result.errors.len(), 0);
    }
}
