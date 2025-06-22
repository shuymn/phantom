#[cfg(test)]
mod select_integration_tests {
    use crate::test_utils::TestRepo;
    use crate::worktree::create::create_worktree;
    use crate::worktree::types::CreateWorktreeOptions;

    #[tokio::test]
    async fn test_select_worktree_with_fzf_empty_list() {
        use crate::core::executors::MockCommandExecutor;
        use crate::worktree::select::select_worktree_with_fzf;

        let repo = TestRepo::new().await.unwrap();
        repo.create_file_and_commit("test.txt", "content", "Initial commit").await.unwrap();

        let mut mock = MockCommandExecutor::new();

        // Mock git worktree list - only main worktree exists
        mock.expect_command("git")
            .with_args(&["worktree", "list", "--porcelain"])
            .in_dir(repo.path().to_path_buf())
            .returns_output(
                &format!(
                    "worktree {}\nHEAD abc123\nbranch refs/heads/main\n",
                    repo.path().display()
                ),
                "",
                0,
            );

        let result = select_worktree_with_fzf(mock, repo.path()).await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[tokio::test]
    async fn test_select_worktree_with_fzf_with_worktrees() {
        use crate::core::executors::MockCommandExecutor;
        use crate::worktree::select::select_worktree_with_fzf;

        let repo = TestRepo::new().await.unwrap();
        repo.create_file_and_commit("test.txt", "content", "Initial commit").await.unwrap();

        // Create some worktrees
        let options = CreateWorktreeOptions::default();
        create_worktree(repo.path(), "feature-1", options.clone()).await.unwrap();
        create_worktree(repo.path(), "feature-2", options.clone()).await.unwrap();
        create_worktree(repo.path(), "bugfix-1", options).await.unwrap();

        let mut mock = MockCommandExecutor::new();

        // Mock git worktree list
        mock.expect_command("git")
            .with_args(&["worktree", "list", "--porcelain"])
            .in_dir(repo.path().to_path_buf())
            .returns_output(
                &format!(
                    "worktree {}\nHEAD abc123\nbranch refs/heads/main\n\n\
                     worktree {}\nHEAD def456\nbranch refs/heads/feature-1\n\n\
                     worktree {}\nHEAD ghi789\nbranch refs/heads/feature-2\n\n\
                     worktree {}\nHEAD jkl012\nbranch refs/heads/bugfix-1\n",
                    repo.path().display(),
                    repo.path().join(".git/phantom/worktrees").join("feature-1").display(),
                    repo.path().join(".git/phantom/worktrees").join("feature-2").display(),
                    repo.path().join(".git/phantom/worktrees").join("bugfix-1").display()
                ),
                "",
                0,
            );

        // Mock git status for each worktree
        mock.expect_command("git")
            .with_args(&["status", "--porcelain"])
            .in_dir(repo.path().to_path_buf())
            .returns_output("", "", 0);

        mock.expect_command("git")
            .with_args(&["status", "--porcelain"])
            .in_dir(repo.path().join(".git/phantom/worktrees").join("feature-1"))
            .returns_output("", "", 0);

        mock.expect_command("git")
            .with_args(&["status", "--porcelain"])
            .in_dir(repo.path().join(".git/phantom/worktrees").join("feature-2"))
            .returns_output("M file.txt\n", "", 0); // dirty

        mock.expect_command("git")
            .with_args(&["status", "--porcelain"])
            .in_dir(repo.path().join(".git/phantom/worktrees").join("bugfix-1"))
            .returns_output("", "", 0);

        // Mock fzf availability check
        mock.expect_command("fzf").with_args(&["--version"]).returns_output("0.42.0", "", 0);

        // Mock fzf selection - user selects feature-2
        mock.expect_command("fzf")
            .with_args(&["--prompt", "Select worktree> ", "--header", "Git Worktrees"])
            .with_stdin_data(
                "feature-1 (feature-1)\nfeature-2 (feature-2) [dirty]\nbugfix-1 (bugfix-1)",
            )
            .returns_output("feature-2 (feature-2) [dirty]\n", "", 0);

        let result = select_worktree_with_fzf(mock, repo.path()).await;
        assert!(result.is_ok());

        let selected = result.unwrap();
        assert!(selected.is_some());

        let selected_worktree = selected.unwrap();
        assert_eq!(selected_worktree.name, "feature-2");
        assert_eq!(selected_worktree.branch, Some("feature-2".to_string()));
        assert!(!selected_worktree.is_clean); // Should be dirty
    }
}
