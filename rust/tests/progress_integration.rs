#[cfg(test)]
mod tests {
    use phantom::worktree::create::create_worktree;
    use phantom::worktree::types::CreateWorktreeOptions;
    use std::path::Path;
    use std::process::Command;
    use tempfile::TempDir;
    use tokio::fs;

    // Helper to create a test git repository
    async fn create_test_repo() -> TempDir {
        let dir = TempDir::new().unwrap();

        // Initialize git repo
        Command::new("git")
            .args(["init"])
            .current_dir(dir.path())
            .output()
            .expect("Failed to init git repo");

        // Configure git user
        Command::new("git")
            .args(["config", "user.name", "Test User"])
            .current_dir(dir.path())
            .output()
            .expect("Failed to configure git user");

        Command::new("git")
            .args(["config", "user.email", "test@example.com"])
            .current_dir(dir.path())
            .output()
            .expect("Failed to configure git email");

        // Create initial commit
        fs::write(dir.path().join("README.md"), "# Test Repo").await.unwrap();

        Command::new("git")
            .args(["add", "."])
            .current_dir(dir.path())
            .output()
            .expect("Failed to add files");

        Command::new("git")
            .args(["commit", "-m", "Initial commit"])
            .current_dir(dir.path())
            .output()
            .expect("Failed to commit");

        dir
    }

    #[tokio::test]
    #[ignore = "Requires git to be installed"]
    async fn test_create_worktree_with_progress() {
        let repo = create_test_repo().await;

        // Create some test files in the repo
        let file1 = repo.path().join("file1.txt");
        let file2 = repo.path().join("src/file2.rs");
        fs::create_dir_all(repo.path().join("src")).await.unwrap();
        fs::write(&file1, "content1").await.unwrap();
        fs::write(&file2, "content2").await.unwrap();

        // Create worktree with progress reporting enabled
        let options = CreateWorktreeOptions {
            branch: Some("feature-test".to_string()),
            commitish: None,
            copy_files: None,
            copy_directory: true,
            show_progress: true,
        };

        let result = create_worktree(repo.path(), "test-worktree", options).await.unwrap();

        // Verify worktree was created
        assert!(result.message.contains("Created worktree"));
        assert!(Path::new(&result.path).exists());

        // Verify files were copied
        if let Some(copied) = result.copied_files {
            // Should have copied some files
            assert!(!copied.is_empty());
            println!("Copied {} files", copied.len());
        }
    }

    #[tokio::test]
    #[ignore = "Requires git to be installed"]
    async fn test_create_worktree_without_progress() {
        let repo = create_test_repo().await;

        // Create worktree without progress reporting
        let options = CreateWorktreeOptions {
            branch: Some("quiet-feature".to_string()),
            commitish: None,
            copy_files: None,
            copy_directory: false,
            show_progress: false,
        };

        let result = create_worktree(repo.path(), "quiet-worktree", options).await.unwrap();

        // Verify worktree was created
        assert!(result.message.contains("Created worktree"));
        assert!(Path::new(&result.path).exists());

        // No files copied since copy_directory is false
        assert!(result.copied_files.is_none());
    }

    #[tokio::test]
    #[ignore = "Requires git to be installed"]
    async fn test_progress_with_large_file_count() {
        let repo = create_test_repo().await;

        // Create many files to test progress reporting
        for i in 0..100 {
            let dir = repo.path().join(format!("dir{}", i / 10));
            fs::create_dir_all(&dir).await.unwrap();
            let file = dir.join(format!("file{}.txt", i));
            fs::write(&file, format!("content {}", i)).await.unwrap();
        }

        // Create worktree with progress reporting
        let options = CreateWorktreeOptions {
            branch: Some("many-files".to_string()),
            commitish: None,
            copy_files: None,
            copy_directory: true,
            show_progress: true,
        };

        let result = create_worktree(repo.path(), "large-worktree", options).await.unwrap();

        // Verify files were copied
        if let Some(copied) = result.copied_files {
            println!("Copied {} files with progress reporting", copied.len());
            // Should have copied many files
            assert!(copied.len() > 50); // At least some of the files
        }
    }
}
