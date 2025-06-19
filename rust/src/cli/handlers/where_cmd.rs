use crate::cli::commands::where_cmd::{WhereArgs, WhereResult};
use crate::cli::context::HandlerContext;
use crate::cli::output::output;
use crate::git::libs::get_git_root::get_git_root_with_executor;
use crate::worktree::locate::where_worktree;
use crate::worktree::select::select_worktree_with_fzf;
use crate::{PhantomError, Result};

/// Handle the where command
pub async fn handle(args: WhereArgs, context: HandlerContext) -> Result<()> {
    // Validate args
    if args.name.is_none() && !args.fzf {
        return Err(PhantomError::Validation(
            "Usage: phantom where <worktree-name> or phantom where --fzf".to_string(),
        ));
    }

    if args.name.is_some() && args.fzf {
        return Err(PhantomError::Validation(
            "Cannot specify both a worktree name and --fzf option".to_string(),
        ));
    }

    // Get git root
    let git_root = get_git_root_with_executor(context.executor.clone()).await?;

    // Get worktree name
    let worktree_name = if args.fzf {
        match select_worktree_with_fzf(&git_root).await? {
            Some(worktree) => worktree.name,
            None => {
                // User cancelled selection
                return Ok(());
            }
        }
    } else {
        args.name.unwrap()
    };

    // Get the worktree path
    match where_worktree(&git_root, &worktree_name).await {
        Ok(result) => {
            if args.json {
                let json_result = WhereResult {
                    success: true,
                    name: worktree_name,
                    path: result.path,
                    error: None,
                };
                let _ = output().json(&json_result);
            } else {
                output().log(&result.path);
            }
            Ok(())
        }
        Err(e) => {
            if args.json {
                let json_result = WhereResult {
                    success: false,
                    name: worktree_name,
                    path: String::new(),
                    error: Some(e.to_string()),
                };
                let _ = output().json(&json_result);
                Ok(())
            } else {
                Err(e)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::executors::MockCommandExecutor;
    use std::sync::Arc;

    #[tokio::test]
    async fn test_where_not_in_git_repo() {
        let mut mock = MockCommandExecutor::new();

        // Expect git root check to fail
        mock.expect_command("git").with_args(&["rev-parse", "--git-common-dir"]).returns_output(
            "",
            "fatal: not a git repository",
            128,
        );

        let context = HandlerContext::new(Arc::new(mock));
        let args = WhereArgs { name: Some("test".to_string()), fzf: false, json: false };

        let result = handle(args, context).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_where_invalid_usage_no_name_no_fzf() {
        let context = HandlerContext::new(Arc::new(MockCommandExecutor::new()));
        let args = WhereArgs { name: None, fzf: false, json: false };

        let result = handle(args, context).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Usage:"));
    }

    #[tokio::test]
    async fn test_where_both_name_and_fzf() {
        let context = HandlerContext::new(Arc::new(MockCommandExecutor::new()));
        let args = WhereArgs { name: Some("test".to_string()), fzf: true, json: false };

        let result = handle(args, context).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Cannot specify both"));
    }

    #[tokio::test]
    #[ignore = "Requires filesystem mocking - where_worktree uses validate_worktree_exists which uses fs::metadata"]
    async fn test_where_worktree_exists() {
        let mut mock = MockCommandExecutor::new();

        // Mock git root check
        mock.expect_command("git").with_args(&["rev-parse", "--git-common-dir"]).returns_output(
            "/repo/.git",
            "",
            0,
        );

        // Mock worktree list for where_worktree
        mock.expect_command("git").with_args(&["worktree", "list", "--porcelain"]).returns_output(
            "worktree /repo\n\
                 HEAD abc123\n\
                 branch refs/heads/main\n\
                 \n\
                 worktree /repo/.phantom/test\n\
                 HEAD def456\n\
                 branch refs/heads/test\n",
            "",
            0,
        );

        let context = HandlerContext::new(Arc::new(mock));
        let args = WhereArgs { name: Some("test".to_string()), fzf: false, json: false };

        let result = handle(args, context).await;
        assert!(result.is_ok());
        // In non-json mode, the path would be output to stdout
    }

    #[tokio::test]
    #[ignore = "Requires filesystem mocking - where_worktree uses validate_worktree_exists which uses fs::metadata"]
    async fn test_where_worktree_not_found() {
        let mut mock = MockCommandExecutor::new();

        // Mock git root check
        mock.expect_command("git").with_args(&["rev-parse", "--git-common-dir"]).returns_output(
            "/repo/.git",
            "",
            0,
        );

        // Mock worktree list - worktree doesn't exist
        mock.expect_command("git").with_args(&["worktree", "list", "--porcelain"]).returns_output(
            "worktree /repo\n\
                 HEAD abc123\n\
                 branch refs/heads/main\n",
            "",
            0,
        );

        let context = HandlerContext::new(Arc::new(mock));
        let args = WhereArgs { name: Some("nonexistent".to_string()), fzf: false, json: false };

        let result = handle(args, context).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    #[ignore = "Requires filesystem mocking - where_worktree uses validate_worktree_exists which uses fs::metadata"]
    async fn test_where_json_output_success() {
        let mut mock = MockCommandExecutor::new();

        // Mock git root check
        mock.expect_command("git").with_args(&["rev-parse", "--git-common-dir"]).returns_output(
            "/repo/.git",
            "",
            0,
        );

        // Mock worktree list
        mock.expect_command("git").with_args(&["worktree", "list", "--porcelain"]).returns_output(
            "worktree /repo\n\
                 HEAD abc123\n\
                 branch refs/heads/main\n\
                 \n\
                 worktree /repo/.phantom/test\n\
                 HEAD def456\n\
                 branch refs/heads/test\n",
            "",
            0,
        );

        let context = HandlerContext::new(Arc::new(mock));
        let args = WhereArgs { name: Some("test".to_string()), fzf: false, json: true };

        let result = handle(args, context).await;
        assert!(result.is_ok());
        // In JSON mode, the result would be output as JSON
    }

    #[tokio::test]
    #[ignore = "Requires filesystem mocking - where_worktree uses validate_worktree_exists which uses fs::metadata"]
    async fn test_where_json_output_not_found() {
        let mut mock = MockCommandExecutor::new();

        // Mock git root check
        mock.expect_command("git").with_args(&["rev-parse", "--git-common-dir"]).returns_output(
            "/repo/.git",
            "",
            0,
        );

        // Mock worktree list - worktree doesn't exist
        mock.expect_command("git").with_args(&["worktree", "list", "--porcelain"]).returns_output(
            "worktree /repo\n\
                 HEAD abc123\n\
                 branch refs/heads/main\n",
            "",
            0,
        );

        let context = HandlerContext::new(Arc::new(mock));
        let args = WhereArgs { name: Some("nonexistent".to_string()), fzf: false, json: true };

        let result = handle(args, context).await;
        // In JSON mode, errors are still reported as success with error field
        assert!(result.is_ok());
    }

    #[tokio::test]
    #[ignore = "Requires fzf command mocking - select_worktree_with_fzf uses process execution"]
    async fn test_where_with_fzf_selection() {
        let mut mock = MockCommandExecutor::new();

        // Mock git root check
        mock.expect_command("git").with_args(&["rev-parse", "--git-common-dir"]).returns_output(
            "/repo/.git",
            "",
            0,
        );

        // Mock worktree list for fzf selection
        mock.expect_command("git").with_args(&["worktree", "list", "--porcelain"]).returns_output(
            "worktree /repo\n\
                 HEAD abc123\n\
                 branch refs/heads/main\n\
                 \n\
                 worktree /repo/.phantom/test\n\
                 HEAD def456\n\
                 branch refs/heads/test\n",
            "",
            0,
        );

        // Mock fzf selection
        mock.expect_command("fzf").returns_output("test", "", 0);

        // Mock second worktree list for where_worktree
        mock.expect_command("git").with_args(&["worktree", "list", "--porcelain"]).returns_output(
            "worktree /repo\n\
                 HEAD abc123\n\
                 branch refs/heads/main\n\
                 \n\
                 worktree /repo/.phantom/test\n\
                 HEAD def456\n\
                 branch refs/heads/test\n",
            "",
            0,
        );

        let _context = HandlerContext::new(Arc::new(mock));
        let _args = WhereArgs { name: None, fzf: true, json: false };

        // This test requires fzf process mocking
        // let result = handle(args, context).await;
        // assert!(result.is_ok());
    }
}
