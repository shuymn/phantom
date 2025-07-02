use crate::cli::commands::delete::{DeleteArgs, DeleteResult};
use crate::cli::context::HandlerContext;
use crate::cli::output::output;
use crate::core::command_executor::CommandExecutor;
use crate::core::exit_handler::ExitHandler;
use crate::core::filesystem::FileSystem;
use crate::git::libs::get_current_worktree::get_current_worktree;
use crate::git::libs::get_git_root::get_git_root;
use crate::worktree::delete::delete_worktree;
use crate::worktree::select::select_worktree_with_fzf;
use crate::worktree::types::DeleteWorktreeOptions;
use anyhow::{bail, Context, Result};

/// Handle the delete command
pub async fn handle<E, F, H>(args: DeleteArgs, context: HandlerContext<E, F, H>) -> Result<()>
where
    E: CommandExecutor + Clone + 'static,
    F: FileSystem + Clone + 'static,
    H: ExitHandler + Clone + 'static,
{
    // Validate args
    if args.name.is_none() && !args.current && !args.fzf {
        bail!(
            "Please provide a worktree name to delete, use --current to delete the current worktree, or use --fzf for interactive selection"
        );
    }

    if (args.name.is_some() || args.fzf) && args.current {
        bail!("Cannot specify --current with a worktree name or --fzf option");
    }

    if args.name.is_some() && args.fzf {
        bail!("Cannot specify both a worktree name and --fzf option");
    }

    // Get git root
    let git_root = get_git_root(context.executor.clone())
        .await
        .with_context(|| "Failed to determine git repository root")?;

    // Get worktree name
    let worktree_name = if args.current {
        let current = get_current_worktree(context.executor.clone(), &git_root)
            .await
            .with_context(|| "Failed to get current worktree")?;
        match current {
            Some(name) => name,
            None => {
                bail!(
                    "Not in a worktree directory. The --current option can only be used from within a worktree."
                );
            }
        }
    } else if args.fzf {
        match select_worktree_with_fzf(context.executor.clone(), &git_root)
            .await
            .with_context(|| "Failed to select worktree with fzf")?
        {
            Some(worktree) => worktree.name,
            None => {
                // User cancelled selection
                return Ok(());
            }
        }
    } else {
        args.name.unwrap()
    };

    // Delete the worktree
    let options = DeleteWorktreeOptions { force: args.force };

    match delete_worktree(
        context.executor.clone(),
        &git_root,
        &worktree_name,
        options,
        &context.filesystem,
    )
    .await
    .with_context(|| format!("Failed to delete worktree '{worktree_name}'"))
    {
        Ok(result) => {
            if args.json {
                let json_result = DeleteResult {
                    success: true,
                    name: worktree_name,
                    message: result.message.clone(),
                    error: None,
                };
                output().json(&json_result).with_context(|| "Failed to serialize JSON output")?;
            } else {
                output().log(&result.message);
            }
            Ok(())
        }
        Err(e) => {
            if args.json {
                let json_result = DeleteResult {
                    success: false,
                    name: worktree_name,
                    message: String::new(),
                    error: Some(e.to_string()),
                };
                output().json(&json_result).with_context(|| "Failed to serialize JSON output")?;
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
    use crate::cli::commands::delete::DeleteArgs;
    use crate::cli::context::HandlerContext;
    use crate::core::executors::MockCommandExecutor;
    use crate::core::filesystems::mock_filesystem::{FileSystemOperation, MockResult};
    use crate::core::filesystems::{FileSystemExpectation, MockFileSystem};
    use std::path::PathBuf;

    // IMPORTANT: Delete handler testing limitations
    //
    // The delete handler has limited testability with mocks because
    // validate_worktree_exists() uses filesystem operations (fs::metadata)
    // directly rather than git commands. This means we can only test:
    // 1. Early failures (not in git repo, current worktree detection)
    // 2. The overall flow with ignored tests that document the limitation
    //
    // Future work: Abstract filesystem operations to enable full mock testing

    #[tokio::test]
    async fn test_delete_not_in_git_repo() {
        let mut mock = MockCommandExecutor::new();

        // Expect git root check to fail
        mock.expect_command("git").with_args(&["rev-parse", "--git-common-dir"]).returns_output(
            "",
            "fatal: not a git repository",
            128,
        );

        let context = HandlerContext::new(
            mock,
            crate::core::filesystems::MockFileSystem::new(),
            crate::core::exit_handler::MockExitHandler::new(),
        );
        let args = DeleteArgs {
            name: Some("test".to_string()),
            current: false,
            force: false,
            fzf: false,
            json: false,
        };

        let result = handle(args, context).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_delete_with_current_flag_not_in_worktree() {
        let mut mock = MockCommandExecutor::new();

        // Mock git root check
        mock.expect_command("git").with_args(&["rev-parse", "--git-common-dir"]).returns_output(
            "/repo/.git",
            "",
            0,
        );

        // Mock get_current_worktree - not in a worktree
        mock.expect_command("git")
            .with_args(&["rev-parse", "--show-toplevel"])
            .returns_output("/repo", "", 0);

        mock.expect_command("git").with_args(&["worktree", "list", "--porcelain"]).returns_output(
            "worktree /repo\nHEAD abc123\nbranch refs/heads/main\n",
            "",
            0,
        );

        let context = HandlerContext::new(
            mock,
            crate::core::filesystems::MockFileSystem::new(),
            crate::core::exit_handler::MockExitHandler::new(),
        );
        let args = DeleteArgs { name: None, current: true, force: false, fzf: false, json: false };

        let result = handle(args, context).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_delete_worktree_success() {
        let mut mock = MockCommandExecutor::new();
        let mock_fs = MockFileSystem::new();

        // Mock git root check
        mock.expect_command("git").with_args(&["rev-parse", "--git-common-dir"]).returns_output(
            "/repo/.git",
            "",
            0,
        );

        // Mock validate_worktree_exists (via list_worktrees)
        mock.expect_command("git")
                .with_args(&["worktree", "list", "--porcelain"])
                .returns_output(
                    "worktree /repo\nHEAD abc123\nbranch refs/heads/main\n\n\
                     worktree /repo/.git/phantom/worktrees/feature\nHEAD def456\nbranch refs/heads/feature\n",
                    "",
                    0
                );

        // Mock filesystem check for worktree existence
        mock_fs.expect(FileSystemExpectation {
            operation: FileSystemOperation::IsDir,
            path: Some(PathBuf::from("/repo/.git/phantom/worktrees/feature")),
            from_path: None,
            to_path: None,
            contents: None,
            result: Ok(MockResult::Bool(true)), // Directory exists
        });

        // Mock get_worktree_status
        mock.expect_command("git")
            .with_args(&["status", "--porcelain"])
            .in_dir("/repo/.git/phantom/worktrees/feature")
            .returns_output("", "", 0); // Clean status

        // Mock remove_worktree
        mock.expect_command("git")
            .with_args(&["worktree", "remove", "/repo/.git/phantom/worktrees/feature"])
            .in_dir("/repo")
            .returns_success();

        // Mock delete_branch
        mock.expect_command("git")
            .with_args(&["branch", "-D", "feature"])
            .in_dir("/repo")
            .returns_success();

        let context =
            HandlerContext::new(mock, mock_fs, crate::core::exit_handler::MockExitHandler::new());
        let args = DeleteArgs {
            name: Some("feature".to_string()),
            current: false,
            force: false,
            fzf: false,
            json: false,
        };

        let result = handle(args, context).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_delete_worktree_with_uncommitted_changes_no_force() {
        let mut mock = MockCommandExecutor::new();
        let mock_fs = MockFileSystem::new();

        // Mock git root check
        mock.expect_command("git").with_args(&["rev-parse", "--git-common-dir"]).returns_output(
            "/repo/.git",
            "",
            0,
        );

        // Mock validate_worktree_exists
        mock.expect_command("git")
                .with_args(&["worktree", "list", "--porcelain"])
                .returns_output(
                    "worktree /repo\nHEAD abc123\nbranch refs/heads/main\n\n\
                     worktree /repo/.git/phantom/worktrees/feature\nHEAD def456\nbranch refs/heads/feature\n",
                    "",
                    0
                );

        // Mock filesystem check for worktree existence
        mock_fs.expect(FileSystemExpectation {
            operation: FileSystemOperation::IsDir,
            path: Some(PathBuf::from("/repo/.git/phantom/worktrees/feature")),
            from_path: None,
            to_path: None,
            contents: None,
            result: Ok(MockResult::Bool(true)), // Directory exists
        });

        // Mock get_worktree_status - has uncommitted changes
        mock.expect_command("git")
            .with_args(&["status", "--porcelain"])
            .in_dir("/repo/.git/phantom/worktrees/feature")
            .returns_output("M  file.txt\n?? new.txt\n", "", 0);

        let context =
            HandlerContext::new(mock, mock_fs, crate::core::exit_handler::MockExitHandler::new());
        let args = DeleteArgs {
            name: Some("feature".to_string()),
            current: false,
            force: false,
            fzf: false,
            json: false,
        };

        let result = handle(args, context).await;
        assert!(result.is_err());
        match result {
            Err(e) => {
                let error_str = e.to_string();
                assert!(
                    error_str.contains("uncommitted changes")
                        || error_str.contains("Failed to delete worktree"),
                    "Unexpected error message: {error_str}"
                );
            }
            _ => panic!("Expected error about uncommitted changes"),
        }
    }

    #[tokio::test]
    async fn test_delete_worktree_with_force() {
        let mut mock = MockCommandExecutor::new();
        let mock_fs = MockFileSystem::new();

        // Mock git root check
        mock.expect_command("git").with_args(&["rev-parse", "--git-common-dir"]).returns_output(
            "/repo/.git",
            "",
            0,
        );

        // Mock validate_worktree_exists
        mock.expect_command("git")
                .with_args(&["worktree", "list", "--porcelain"])
                .returns_output(
                    "worktree /repo\nHEAD abc123\nbranch refs/heads/main\n\n\
                     worktree /repo/.git/phantom/worktrees/feature\nHEAD def456\nbranch refs/heads/feature\n",
                    "",
                    0
                );

        // Mock filesystem check for worktree existence
        mock_fs.expect(FileSystemExpectation {
            operation: FileSystemOperation::IsDir,
            path: Some(PathBuf::from("/repo/.git/phantom/worktrees/feature")),
            from_path: None,
            to_path: None,
            contents: None,
            result: Ok(MockResult::Bool(true)), // Directory exists
        });

        // Mock get_worktree_status - has uncommitted changes
        mock.expect_command("git")
            .with_args(&["status", "--porcelain"])
            .in_dir("/repo/.git/phantom/worktrees/feature")
            .returns_output("M  file.txt\n", "", 0);

        // Mock remove_worktree - first normal attempt fails
        mock.expect_command("git")
            .with_args(&["worktree", "remove", "/repo/.git/phantom/worktrees/feature"])
            .in_dir("/repo")
            .returns_output("", "worktree has uncommitted changes", 1);

        // Mock remove_worktree - force attempt succeeds
        mock.expect_command("git")
            .with_args(&["worktree", "remove", "--force", "/repo/.git/phantom/worktrees/feature"])
            .in_dir("/repo")
            .returns_success();

        // Mock delete_branch
        mock.expect_command("git")
            .with_args(&["branch", "-D", "feature"])
            .in_dir("/repo")
            .returns_success();

        let context =
            HandlerContext::new(mock, mock_fs, crate::core::exit_handler::MockExitHandler::new());
        let args = DeleteArgs {
            name: Some("feature".to_string()),
            current: false,
            force: true,
            fzf: false,
            json: false,
        };

        let result = handle(args, context).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_delete_json_output_success() {
        let mut mock = MockCommandExecutor::new();
        let mock_fs = MockFileSystem::new();

        // Mock git root check
        mock.expect_command("git").with_args(&["rev-parse", "--git-common-dir"]).returns_output(
            "/repo/.git",
            "",
            0,
        );

        // Mock validate_worktree_exists
        mock.expect_command("git")
                .with_args(&["worktree", "list", "--porcelain"])
                .returns_output(
                    "worktree /repo\nHEAD abc123\nbranch refs/heads/main\n\n\
                     worktree /repo/.git/phantom/worktrees/feature\nHEAD def456\nbranch refs/heads/feature\n",
                    "",
                    0
                );

        // Mock filesystem check for worktree existence
        mock_fs.expect(FileSystemExpectation {
            operation: FileSystemOperation::IsDir,
            path: Some(PathBuf::from("/repo/.git/phantom/worktrees/feature")),
            from_path: None,
            to_path: None,
            contents: None,
            result: Ok(MockResult::Bool(true)), // Directory exists
        });

        // Mock get_worktree_status
        mock.expect_command("git")
            .with_args(&["status", "--porcelain"])
            .in_dir("/repo/.git/phantom/worktrees/feature")
            .returns_output("", "", 0);

        // Mock remove_worktree
        mock.expect_command("git")
            .with_args(&["worktree", "remove", "/repo/.git/phantom/worktrees/feature"])
            .in_dir("/repo")
            .returns_success();

        // Mock delete_branch
        mock.expect_command("git")
            .with_args(&["branch", "-D", "feature"])
            .in_dir("/repo")
            .returns_success();

        let context =
            HandlerContext::new(mock, mock_fs, crate::core::exit_handler::MockExitHandler::new());
        let args = DeleteArgs {
            name: Some("feature".to_string()),
            current: false,
            force: false,
            fzf: false,
            json: true,
        };

        let result = handle(args, context).await;
        assert!(result.is_ok());
        // In JSON mode, success is communicated via JSON output
    }
}
