#[cfg(test)]
mod tests {
    use crate::cli::commands::attach::AttachArgs;
    use crate::cli::context::HandlerContext;
    use crate::cli::handlers::attach::handle;
    use crate::core::executors::MockCommandExecutor;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_attach_success() {
        let temp_dir = tempdir().unwrap();
        let git_root = temp_dir.path();
        let git_root_canonical = git_root.canonicalize().unwrap();
        let worktree_path =
            git_root_canonical.join(".git").join("phantom").join("worktrees").join("test-branch");

        let mut mock = MockCommandExecutor::new();

        // Mock git rev-parse --git-common-dir
        mock.expect_command("git").with_args(&["rev-parse", "--git-common-dir"]).returns_output(
            &format!("{}/.git", git_root_canonical.to_string_lossy()),
            "",
            0,
        );

        // Mock git show-ref to check branch exists - with canonicalized path
        mock.expect_command("git")
            .with_args(&["show-ref", "--verify", "--quiet", "refs/heads/test-branch"])
            .in_dir(&git_root_canonical)
            .returns_success();

        // Mock git worktree add - with canonicalized path
        mock.expect_command("git")
            .with_args(&["worktree", "add", &worktree_path.to_string_lossy(), "test-branch"])
            .in_dir(&git_root_canonical)
            .returns_success();

        let args =
            AttachArgs { branch: "test-branch".to_string(), json: false, shell: false, exec: None };

        let context = HandlerContext::new(
            mock,
            crate::core::filesystems::MockFileSystem::new(),
            crate::core::exit_handler::MockExitHandler::new(),
        );

        let result = handle(args, context).await;
        if let Err(e) = &result {
            eprintln!("Handle failed with error: {:?}", e);
        }
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_attach_branch_not_found() {
        let temp_dir = tempdir().unwrap();
        let git_root = temp_dir.path();
        let git_root_canonical = git_root.canonicalize().unwrap();

        let mut mock = MockCommandExecutor::new();

        // Mock git rev-parse --git-common-dir
        mock.expect_command("git").with_args(&["rev-parse", "--git-common-dir"]).returns_output(
            &format!("{}/.git", git_root.to_string_lossy()),
            "",
            0,
        );

        // Mock git show-ref to check branch exists (returns false) - with canonicalized path
        mock.expect_command("git")
            .with_args(&["show-ref", "--verify", "--quiet", "refs/heads/nonexistent"])
            .in_dir(&git_root_canonical)
            .returns_output("", "fatal: bad ref for symbolic ref refs/heads/nonexistent\n", 1);

        let args =
            AttachArgs { branch: "nonexistent".to_string(), json: false, shell: false, exec: None };

        let context = HandlerContext::new(
            mock,
            crate::core::filesystems::MockFileSystem::new(),
            crate::core::exit_handler::MockExitHandler::new(),
        );

        let result = handle(args, context).await;
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("Branch 'nonexistent' not found"));
    }

    #[tokio::test]
    async fn test_attach_worktree_already_exists() {
        let temp_dir = tempdir().unwrap();
        let git_root = temp_dir.path();
        let worktree_path =
            git_root.join(".git").join("phantom").join("worktrees").join("existing-branch");

        // Create the worktree directory to simulate it already exists
        std::fs::create_dir_all(&worktree_path).unwrap();

        let mut mock = MockCommandExecutor::new();

        // Mock git rev-parse --git-common-dir
        mock.expect_command("git").with_args(&["rev-parse", "--git-common-dir"]).returns_output(
            &format!("{}/.git", git_root.to_string_lossy()),
            "",
            0,
        );

        let args = AttachArgs {
            branch: "existing-branch".to_string(),
            json: false,
            shell: false,
            exec: None,
        };

        let context = HandlerContext::new(
            mock,
            crate::core::filesystems::MockFileSystem::new(),
            crate::core::exit_handler::MockExitHandler::new(),
        );

        let result = handle(args, context).await;
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("Worktree 'existing-branch' already exists"));
    }

    #[tokio::test]
    async fn test_attach_invalid_worktree_name() {
        let mock = MockCommandExecutor::new();

        let args = AttachArgs { branch: "".to_string(), json: false, shell: false, exec: None };

        let context = HandlerContext::new(
            mock,
            crate::core::filesystems::MockFileSystem::new(),
            crate::core::exit_handler::MockExitHandler::new(),
        );

        let result = handle(args, context).await;
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("empty") || err.to_string().contains("invalid"));
    }

    #[tokio::test]
    async fn test_attach_with_json_output() {
        let temp_dir = tempdir().unwrap();
        let git_root = temp_dir.path();
        let git_root_canonical = git_root.canonicalize().unwrap();
        let worktree_path =
            git_root_canonical.join(".git").join("phantom").join("worktrees").join("json-branch");

        let mut mock = MockCommandExecutor::new();

        // Mock git rev-parse --git-common-dir
        mock.expect_command("git").with_args(&["rev-parse", "--git-common-dir"]).returns_output(
            &format!("{}/.git", git_root_canonical.to_string_lossy()),
            "",
            0,
        );

        // Mock git show-ref to check branch exists - with canonicalized path
        mock.expect_command("git")
            .with_args(&["show-ref", "--verify", "--quiet", "refs/heads/json-branch"])
            .in_dir(&git_root_canonical)
            .returns_success();

        // Mock git worktree add - with canonicalized path
        mock.expect_command("git")
            .with_args(&["worktree", "add", &worktree_path.to_string_lossy(), "json-branch"])
            .in_dir(&git_root_canonical)
            .returns_success();

        let args =
            AttachArgs { branch: "json-branch".to_string(), json: true, shell: false, exec: None };

        let context = HandlerContext::new(
            mock,
            crate::core::filesystems::MockFileSystem::new(),
            crate::core::exit_handler::MockExitHandler::new(),
        );

        let result = handle(args, context).await;
        if let Err(e) = &result {
            eprintln!("Error in test_attach_with_json_output: {:?}", e);
        }
        assert!(result.is_ok());
    }
}
