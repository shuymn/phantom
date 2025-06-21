/// Batch processor for worktree operations using arena allocation
///
/// This module provides efficient batch processing of worktree operations
/// by using arena allocation to reduce allocation overhead in bulk operations.
use crate::core::command_executor::CommandExecutor;
use crate::core::types::Worktree;
use crate::git::libs::list_worktrees::list_worktrees_with_executor;
use crate::worktree::list::{get_worktree_status_with_executor, WorktreeInfo};
use crate::worktree::paths::get_phantom_directory;
use crate::Result;
use futures::stream::{FuturesUnordered, StreamExt};
use std::path::Path;
use std::sync::Arc;
use tracing::{debug, instrument};
use typed_arena::Arena;

/// Result of a batch operation
#[derive(Debug)]
pub struct BatchResult<'a> {
    /// Successfully processed items
    pub success: Vec<&'a WorktreeInfo>,
    /// Failed items with their error messages
    pub failed: Vec<&'a (String, String)>,
}

/// Batch processor for worktree operations
///
/// Uses arena allocation to minimize allocation overhead when processing
/// large numbers of worktrees.
pub struct BatchProcessor<'a> {
    /// Arena for allocating WorktreeInfo structs
    info_arena: &'a Arena<WorktreeInfo>,
    /// Arena for allocating error tuples
    error_arena: &'a Arena<(String, String)>,
    /// Arena for temporary string allocations
    string_arena: &'a Arena<String>,
}

impl<'a> BatchProcessor<'a> {
    /// Create a new batch processor with the given arenas
    pub fn new(
        info_arena: &'a Arena<WorktreeInfo>,
        error_arena: &'a Arena<(String, String)>,
        string_arena: &'a Arena<String>,
    ) -> Self {
        Self { info_arena, error_arena, string_arena }
    }

    /// Process a batch of worktrees with status checks
    ///
    /// This method processes all worktrees concurrently and allocates
    /// results in the arena for efficient memory usage.
    #[instrument(skip(self, executor))]
    pub async fn process_worktrees_with_status<E>(
        &self,
        executor: Arc<E>,
        git_root: &Path,
    ) -> Result<BatchResult<'a>>
    where
        E: CommandExecutor + 'static,
    {
        debug!("Starting batch processing of worktrees");

        // Get all worktrees
        let git_worktrees = list_worktrees_with_executor(executor.clone(), git_root).await?;
        let phantom_dir = get_phantom_directory(git_root);
        let phantom_dir_canonical =
            phantom_dir.canonicalize().unwrap_or_else(|_| phantom_dir.clone());

        // Filter phantom worktrees
        let phantom_worktrees: Vec<_> = git_worktrees
            .into_iter()
            .filter_map(|worktree| {
                let worktree_path_canonical =
                    worktree.path.canonicalize().unwrap_or_else(|_| worktree.path.clone());
                if worktree_path_canonical.starts_with(&phantom_dir_canonical) {
                    Some(worktree)
                } else {
                    None
                }
            })
            .collect();

        debug!("Found {} phantom worktrees to process", phantom_worktrees.len());

        // Process worktrees concurrently with rate limiting
        let mut futures = FuturesUnordered::new();
        let mut success: Vec<&WorktreeInfo> = Vec::new();
        let mut failed: Vec<&(String, String)> = Vec::new();

        for worktree in phantom_worktrees {
            let executor = executor.clone();
            let name = self.extract_worktree_name(&worktree, &phantom_dir_canonical);
            let path_str = self.string_arena.alloc(worktree.path.to_string_lossy().to_string());

            futures.push(async move {
                let status_result =
                    get_worktree_status_with_executor(executor, &worktree.path).await;
                (name, path_str, worktree.branch, status_result)
            });
        }

        // Collect results with controlled concurrency
        while let Some((name, path_str, branch, status_result)) = futures.next().await {
            match status_result {
                Ok(is_clean) => {
                    let info = self.info_arena.alloc(WorktreeInfo {
                        name,
                        path: path_str.clone(),
                        branch,
                        is_clean,
                    });
                    success.push(info as &WorktreeInfo);
                }
                Err(e) => {
                    let error_tuple = self.error_arena.alloc((name, e.to_string()));
                    failed.push(error_tuple as &(String, String));
                }
            }
        }

        debug!("Batch processing complete: {} success, {} failed", success.len(), failed.len());

        Ok(BatchResult { success, failed })
    }

    /// Process multiple specific worktrees by name
    #[instrument(skip(self, executor))]
    pub async fn process_specific_worktrees<E>(
        &self,
        executor: Arc<E>,
        git_root: &Path,
        names: &[&str],
    ) -> Result<BatchResult<'a>>
    where
        E: CommandExecutor + 'static,
    {
        use crate::worktree::list::get_worktree_info_with_executor;

        debug!("Processing {} specific worktrees", names.len());

        let mut futures = FuturesUnordered::new();
        let mut success: Vec<&WorktreeInfo> = Vec::new();
        let mut failed: Vec<&(String, String)> = Vec::new();

        for &name in names {
            let executor = executor.clone();
            let name_owned = self.string_arena.alloc(name.to_string());

            futures.push(async move {
                let result = get_worktree_info_with_executor(executor, git_root, name).await;
                (name_owned.clone(), result)
            });
        }

        // Process with controlled concurrency
        while let Some((name, result)) = futures.next().await {
            match result {
                Ok(info) => {
                    let arena_info = self.info_arena.alloc(info);
                    success.push(arena_info as &WorktreeInfo);
                }
                Err(e) => {
                    let error_tuple = self.error_arena.alloc((name, e.to_string()));
                    failed.push(error_tuple as &(String, String));
                }
            }
        }

        Ok(BatchResult { success, failed })
    }

    /// Extract worktree name from path
    fn extract_worktree_name(&self, worktree: &Worktree, phantom_dir: &Path) -> String {
        let phantom_dir_str = phantom_dir.to_string_lossy();
        let worktree_path_canonical =
            worktree.path.canonicalize().unwrap_or_else(|_| worktree.path.clone());
        let canonical_path_str = worktree_path_canonical.to_string_lossy();

        if let Some(stripped) = canonical_path_str.strip_prefix(&format!("{}/", phantom_dir_str)) {
            self.string_arena.alloc(stripped.to_string()).clone()
        } else {
            worktree.name.clone()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::executors::MockCommandExecutor;
    use std::path::PathBuf;

    #[tokio::test]
    async fn test_batch_processor_empty_worktrees() {
        let mut mock = MockCommandExecutor::new();
        let git_root = PathBuf::from("/repo");

        // Mock empty worktree list
        mock.expect_command("git").with_args(&["worktree", "list", "--porcelain"]).returns_output(
            "worktree /repo\nHEAD abc123\nbranch refs/heads/main\n",
            "",
            0,
        );

        let info_arena = Arena::new();
        let error_arena = Arena::new();
        let string_arena = Arena::new();

        let processor = BatchProcessor::new(&info_arena, &error_arena, &string_arena);
        let result =
            processor.process_worktrees_with_status(Arc::new(mock), &git_root).await.unwrap();

        assert_eq!(result.success.len(), 0);
        assert_eq!(result.failed.len(), 0);
    }

    #[tokio::test]
    async fn test_batch_processor_with_worktrees() {
        let mut mock = MockCommandExecutor::new();
        let git_root = PathBuf::from("/repo");

        // Mock worktree list with phantom worktrees
        mock.expect_command("git").with_args(&["worktree", "list", "--porcelain"]).returns_output(
            "worktree /repo\nHEAD abc123\nbranch refs/heads/main\n\n\
             worktree /repo/.git/phantom/worktrees/feature-1\nHEAD def456\nbranch refs/heads/feature-1\n\n\
             worktree /repo/.git/phantom/worktrees/feature-2\nHEAD ghi789\nbranch refs/heads/feature-2\n",
            "",
            0,
        );

        // Mock status checks
        mock.expect_command("git")
            .with_args(&["status", "--porcelain"])
            .in_dir("/repo/.git/phantom/worktrees/feature-1")
            .returns_output("", "", 0);

        mock.expect_command("git")
            .with_args(&["status", "--porcelain"])
            .in_dir("/repo/.git/phantom/worktrees/feature-2")
            .returns_output("M file.txt\n", "", 0);

        let info_arena = Arena::new();
        let error_arena = Arena::new();
        let string_arena = Arena::new();

        let processor = BatchProcessor::new(&info_arena, &error_arena, &string_arena);
        let result =
            processor.process_worktrees_with_status(Arc::new(mock), &git_root).await.unwrap();

        assert_eq!(result.success.len(), 2);
        assert_eq!(result.failed.len(), 0);

        // Check results
        let feature1 = result.success.iter().find(|w| w.name == "feature-1").unwrap();
        assert!(feature1.is_clean);

        let feature2 = result.success.iter().find(|w| w.name == "feature-2").unwrap();
        assert!(!feature2.is_clean);
    }

    #[tokio::test]
    async fn test_batch_processor_memory_efficiency() {
        // Test that we can process many worktrees efficiently
        let mut mock = MockCommandExecutor::new();
        let git_root = PathBuf::from("/repo");

        // Create a large worktree list
        let mut output = String::from("worktree /repo\nHEAD abc123\nbranch refs/heads/main\n");
        for i in 0..100 {
            output.push_str(&format!(
                "\nworktree /repo/.git/phantom/worktrees/feature-{}\nHEAD def{:03}\nbranch refs/heads/feature-{}\n",
                i, i, i
            ));
        }

        mock.expect_command("git")
            .with_args(&["worktree", "list", "--porcelain"])
            .returns_output(&output, "", 0);

        // Mock status checks for all 100 worktrees
        for i in 0..100 {
            mock.expect_command("git")
                .with_args(&["status", "--porcelain"])
                .in_dir(&format!("/repo/.git/phantom/worktrees/feature-{}", i))
                .returns_output(if i % 2 == 0 { "" } else { "M file.txt\n" }, "", 0);
        }

        let info_arena = Arena::new();
        let error_arena = Arena::new();
        let string_arena = Arena::new();

        let processor = BatchProcessor::new(&info_arena, &error_arena, &string_arena);
        let result =
            processor.process_worktrees_with_status(Arc::new(mock), &git_root).await.unwrap();

        assert_eq!(result.success.len(), 100);
        assert_eq!(result.failed.len(), 0);

        // Verify some results
        let clean_count = result.success.iter().filter(|w| w.is_clean).count();
        let dirty_count = result.success.iter().filter(|w| !w.is_clean).count();
        assert_eq!(clean_count, 50);
        assert_eq!(dirty_count, 50);
    }
}
