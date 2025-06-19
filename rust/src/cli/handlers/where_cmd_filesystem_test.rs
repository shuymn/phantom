#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::context::HandlerContext;
    use crate::cli::commands::where_cmd::WhereArgs;
    use crate::core::executors::MockCommandExecutor;
    use crate::core::filesystems::{
        MockFileSystem, FileSystemExpectation, FileSystemOperation, MockResult,
    };
    use crate::cli::handlers::where_cmd::handle;
    use std::sync::Arc;
    use std::path::PathBuf;

    #[tokio::test]
    async fn test_where_with_filesystem_mocking() {
        let mut mock = MockCommandExecutor::new();
        let mock_fs = MockFileSystem::new();

        // Mock git root check
        mock.expect_command("git")
            .with_args(&["rev-parse", "--git-common-dir"])
            .returns_output("/repo/.git", "", 0);

        // Mock filesystem check for worktree existence
        mock_fs.expect(FileSystemExpectation {
            operation: FileSystemOperation::Metadata,
            path: Some(PathBuf::from("/repo/.git/phantom/worktrees/test")),
            from_path: None,
            to_path: None,
            contents: None,
            result: Ok(MockResult::Unit), // Metadata exists
        });

        let context = HandlerContext::new(
            Arc::new(mock),
            Arc::new(mock_fs),
            Arc::new(crate::core::exit_handler::MockExitHandler::new()),
        );
        let args = WhereArgs { 
            name: Some("test".to_string()), 
            fzf: false, 
            json: false 
        };

        let result = handle(args, context).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_where_worktree_not_found_with_filesystem_mocking() {
        let mut mock = MockCommandExecutor::new();
        let mock_fs = MockFileSystem::new();

        // Mock git root check
        mock.expect_command("git")
            .with_args(&["rev-parse", "--git-common-dir"])
            .returns_output("/repo/.git", "", 0);

        // Mock filesystem check - worktree doesn't exist
        mock_fs.expect(FileSystemExpectation {
            operation: FileSystemOperation::Metadata,
            path: Some(PathBuf::from("/repo/.git/phantom/worktrees/nonexistent")),
            from_path: None,
            to_path: None,
            contents: None,
            result: Err(crate::PhantomError::FileOperation("File not found".to_string())),
        });

        let context = HandlerContext::new(
            Arc::new(mock),
            Arc::new(mock_fs),
            Arc::new(crate::core::exit_handler::MockExitHandler::new()),
        );
        let args = WhereArgs { 
            name: Some("nonexistent".to_string()), 
            fzf: false, 
            json: false 
        };

        let result = handle(args, context).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            crate::PhantomError::WorktreeNotFound { name } => {
                assert_eq!(name, "nonexistent");
            }
            e => panic!("Expected WorktreeNotFound error, got: {:?}", e),
        }
    }
}