#[cfg(test)]
mod select_integration_tests {
    use crate::test_utils::TestRepo;
    use crate::worktree::create::create_worktree;
    use crate::worktree::select::select_worktree_with_fzf;
    use crate::worktree::types::CreateWorktreeOptions;

    #[tokio::test]
    #[ignore] // This test requires fzf to be installed
    async fn test_select_worktree_with_fzf_empty_list() {
        let repo = TestRepo::new().await.unwrap();
        repo.create_file_and_commit("test.txt", "content", "Initial commit").await.unwrap();

        let result = select_worktree_with_fzf(repo.path()).await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[tokio::test]
    #[ignore] // This test requires fzf to be installed and manual interaction
    async fn test_select_worktree_with_fzf_with_worktrees() {
        let repo = TestRepo::new().await.unwrap();
        repo.create_file_and_commit("test.txt", "content", "Initial commit").await.unwrap();

        // Create some worktrees
        let options = CreateWorktreeOptions::default();
        create_worktree(repo.path(), "feature-1", options.clone()).await.unwrap();
        create_worktree(repo.path(), "feature-2", options.clone()).await.unwrap();
        create_worktree(repo.path(), "bugfix-1", options).await.unwrap();

        // This would require manual selection in fzf
        println!("Manual test: select a worktree from the fzf interface");
        let result = select_worktree_with_fzf(repo.path()).await;
        assert!(result.is_ok());

        if let Some(selected) = result.unwrap() {
            println!("Selected worktree: {:?}", selected);
            assert!(!selected.name.is_empty());
        }
    }
}
