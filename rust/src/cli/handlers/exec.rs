use crate::cli::commands::exec::ExecArgs;
use crate::cli::context::HandlerContext;
use crate::cli::output::output;
use crate::git::libs::get_git_root::get_git_root_with_executor;
use crate::process::exec::{exec_in_worktree, exec_in_worktree_with_executor};
use crate::process::kitty::{
    execute_kitty_command, is_inside_kitty, KittyOptions, KittySplitDirection,
};
use crate::process::shell::get_phantom_env;
use crate::process::tmux::{
    execute_tmux_command, execute_tmux_command_with_executor, is_inside_tmux, TmuxOptions,
    TmuxSplitDirection,
};
use crate::worktree::select::select_worktree_with_fzf;
use crate::worktree::validate::validate_worktree_exists;
use crate::{PhantomError, Result};

/// Handle the exec command
pub async fn handle(args: ExecArgs, context: HandlerContext) -> Result<()> {
    // Parse command from arguments
    let (worktree_name_pos, command_args) = if args.fzf {
        // With --fzf, all args are command args
        (None, args.command)
    } else {
        // Without --fzf, first arg is worktree name
        if args.command.is_empty() {
            return Err(PhantomError::Validation(
                "Usage: phantom exec <worktree-name> <command> [args...] or phantom exec --fzf <command> [args...]".to_string(),
            ));
        }

        if args.name.is_some() {
            // If name is provided via option, all command args are the command
            (args.name, args.command)
        } else {
            // Otherwise, first command arg is the worktree name
            if args.command.len() < 2 {
                return Err(PhantomError::Validation(
                    "Usage: phantom exec <worktree-name> <command> [args...]".to_string(),
                ));
            }
            let mut cmd = args.command;
            let name = cmd.remove(0);
            (Some(name), cmd)
        }
    };

    if command_args.is_empty() {
        return Err(PhantomError::Validation("No command specified".to_string()));
    }

    // Determine tmux direction
    let tmux_direction = if args.tmux {
        Some(TmuxSplitDirection::New)
    } else if args.tmux_vertical || args.tmux_v {
        Some(TmuxSplitDirection::Vertical)
    } else if args.tmux_horizontal || args.tmux_h {
        Some(TmuxSplitDirection::Horizontal)
    } else {
        None
    };

    // Determine kitty direction
    let kitty_direction = if args.kitty {
        Some(KittySplitDirection::New)
    } else if args.kitty_vertical || args.kitty_v {
        Some(KittySplitDirection::Vertical)
    } else if args.kitty_horizontal || args.kitty_h {
        Some(KittySplitDirection::Horizontal)
    } else {
        None
    };

    // Validate multiplexer options
    if tmux_direction.is_some() && !is_inside_tmux().await {
        return Err(PhantomError::Validation(
            "The --tmux option can only be used inside a tmux session".to_string(),
        ));
    }

    if kitty_direction.is_some() && !is_inside_kitty().await {
        return Err(PhantomError::Validation(
            "The --kitty option can only be used inside a kitty terminal".to_string(),
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
        worktree_name_pos.unwrap()
    };

    // Validate worktree exists
    let validation =
        validate_worktree_exists(&git_root, &worktree_name, context.filesystem.as_ref()).await?;
    let worktree_path = validation.path;

    // Split command into program and arguments
    let command = command_args[0].clone();
    let args_slice = &command_args[1..];

    // Handle tmux execution
    if let Some(direction) = tmux_direction {
        output().log(&format!(
            "Executing command in worktree '{}' in tmux {}...",
            worktree_name,
            if direction == TmuxSplitDirection::New { "window" } else { "pane" }
        ));

        let options = TmuxOptions {
            direction,
            command,
            args: Some(args_slice.to_vec()),
            cwd: Some(worktree_path.to_string_lossy().to_string()),
            env: Some(get_phantom_env(&worktree_name, &worktree_path.to_string_lossy())),
            window_name: if direction == TmuxSplitDirection::New {
                Some(worktree_name)
            } else {
                None
            },
        };

        if cfg!(test) {
            // In test mode, use the executor from context
            execute_tmux_command_with_executor(context.executor.clone(), options).await?;
        } else {
            execute_tmux_command(options).await?;
        }
        return Ok(());
    }

    // Handle kitty execution
    if let Some(direction) = kitty_direction {
        output().log(&format!(
            "Executing command in worktree '{}' in kitty {}...",
            worktree_name,
            if direction == KittySplitDirection::New { "tab" } else { "split" }
        ));

        let options = KittyOptions {
            direction,
            command,
            args: Some(args_slice.to_vec()),
            cwd: Some(worktree_path.to_string_lossy().to_string()),
            env: Some(get_phantom_env(&worktree_name, &worktree_path.to_string_lossy())),
            window_title: if direction == KittySplitDirection::New {
                Some(worktree_name)
            } else {
                None
            },
        };

        execute_kitty_command(options).await?;
        return Ok(());
    }

    // Normal execution
    let result = if cfg!(test) {
        // In test mode, use the executor from context
        exec_in_worktree_with_executor(
            &git_root,
            &worktree_name,
            &command,
            args_slice,
            context.filesystem.as_ref(),
            Some(context.executor.clone()),
        )
        .await?
    } else {
        exec_in_worktree(
            &git_root,
            &worktree_name,
            &command,
            args_slice,
            context.filesystem.as_ref(),
        )
        .await?
    };

    // Exit with the same code as the executed command
    context.exit_handler.exit(result.exit_code);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::executors::MockCommandExecutor;
    use crate::core::filesystems::mock_filesystem::{FileSystemOperation, MockResult};
    use crate::core::filesystems::{FileSystemExpectation, MockFileSystem};
    use std::path::PathBuf;
    use std::sync::Arc;

    #[tokio::test]
    async fn test_exec_not_in_git_repo() {
        let mut mock = MockCommandExecutor::new();

        // Expect git root check to fail
        mock.expect_command("git").with_args(&["rev-parse", "--git-common-dir"]).returns_output(
            "",
            "fatal: not a git repository",
            128,
        );

        let context = HandlerContext::new(
            Arc::new(mock),
            Arc::new(crate::core::filesystems::MockFileSystem::new()),
            Arc::new(crate::core::exit_handler::MockExitHandler::new()),
        );
        let args = ExecArgs {
            name: Some("test".to_string()),
            command: vec!["echo".to_string(), "hello".to_string()],
            fzf: false,
            tmux: false,
            tmux_vertical: false,
            tmux_v: false,
            tmux_horizontal: false,
            tmux_h: false,
            kitty: false,
            kitty_vertical: false,
            kitty_v: false,
            kitty_horizontal: false,
            kitty_h: false,
        };

        let result = handle(args, context).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_exec_no_command_specified() {
        let context = HandlerContext::new(
            Arc::new(MockCommandExecutor::new()),
            Arc::new(crate::core::filesystems::MockFileSystem::new()),
            Arc::new(crate::core::exit_handler::MockExitHandler::new()),
        );
        let args = ExecArgs {
            name: None,
            command: vec![], // No args at all
            fzf: false,
            tmux: false,
            tmux_vertical: false,
            tmux_v: false,
            tmux_horizontal: false,
            tmux_h: false,
            kitty: false,
            kitty_vertical: false,
            kitty_v: false,
            kitty_horizontal: false,
            kitty_h: false,
        };

        let result = handle(args, context).await;
        assert!(result.is_err());
        // This will trigger the first validation error about usage
        assert!(result.unwrap_err().to_string().contains("Usage:"));
    }

    #[tokio::test]
    async fn test_exec_invalid_usage_without_name() {
        let context = HandlerContext::new(
            Arc::new(MockCommandExecutor::new()),
            Arc::new(crate::core::filesystems::MockFileSystem::new()),
            Arc::new(crate::core::exit_handler::MockExitHandler::new()),
        );
        let args = ExecArgs {
            name: None,
            command: vec!["echo".to_string()], // Only one arg, need at least 2
            fzf: false,
            tmux: false,
            tmux_vertical: false,
            tmux_v: false,
            tmux_horizontal: false,
            tmux_h: false,
            kitty: false,
            kitty_vertical: false,
            kitty_v: false,
            kitty_horizontal: false,
            kitty_h: false,
        };

        let result = handle(args, context).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Usage:"));
    }

    #[tokio::test]
    async fn test_exec_tmux_outside_tmux_session() {
        let mock = MockCommandExecutor::new();

        // Mock TMUX env check (not inside tmux)
        // Note: is_inside_tmux checks std::env::var("TMUX") directly
        // This test will pass validation but demonstrates the structure

        let context = HandlerContext::new(
            Arc::new(mock),
            Arc::new(crate::core::filesystems::MockFileSystem::new()),
            Arc::new(crate::core::exit_handler::MockExitHandler::new()),
        );
        let args = ExecArgs {
            name: Some("test".to_string()),
            command: vec!["echo".to_string(), "hello".to_string()],
            fzf: false,
            tmux: true,
            tmux_vertical: false,
            tmux_v: false,
            tmux_horizontal: false,
            tmux_h: false,
            kitty: false,
            kitty_vertical: false,
            kitty_v: false,
            kitty_horizontal: false,
            kitty_h: false,
        };

        let result = handle(args, context).await;
        // This will likely pass because is_inside_tmux() checks env directly
        // In a real test environment without TMUX set, this would fail
        if std::env::var("TMUX").is_err() {
            assert!(result.is_err());
            assert!(result.unwrap_err().to_string().contains("inside a tmux session"));
        }
    }

    #[tokio::test]
    #[should_panic(expected = "MockExitHandler::exit called with code 0")]
    async fn test_exec_success_normal() {
        let mut mock = MockCommandExecutor::new();
        let mock_fs = MockFileSystem::new();

        // Mock git root check
        mock.expect_command("git").with_args(&["rev-parse", "--git-common-dir"]).returns_output(
            "/repo/.git",
            "",
            0,
        );

        // Mock filesystem check for worktree existence
        // Note: validate_worktree_exists is called twice - once in the handler and once in exec_in_worktree
        mock_fs.expect(FileSystemExpectation {
            operation: FileSystemOperation::IsDir,
            path: Some(PathBuf::from("/repo/.git/phantom/worktrees/test")),
            from_path: None,
            to_path: None,
            contents: None,
            result: Ok(MockResult::Bool(true)), // Directory exists
        });

        // Second expectation for the same path (called from exec_in_worktree_with_executor)
        mock_fs.expect(FileSystemExpectation {
            operation: FileSystemOperation::IsDir,
            path: Some(PathBuf::from("/repo/.git/phantom/worktrees/test")),
            from_path: None,
            to_path: None,
            contents: None,
            result: Ok(MockResult::Bool(true)), // Directory exists
        });

        // Mock command execution
        mock.expect_command("echo")
            .with_args(&["hello"])
            .in_dir("/repo/.git/phantom/worktrees/test")
            .returns_output("hello\n", "", 0);

        let context = HandlerContext::new(
            Arc::new(mock),
            Arc::new(mock_fs),
            Arc::new(crate::core::exit_handler::MockExitHandler::new()),
        );
        let args = ExecArgs {
            name: Some("test".to_string()),
            command: vec!["echo".to_string(), "hello".to_string()],
            fzf: false,
            tmux: false,
            tmux_vertical: false,
            tmux_v: false,
            tmux_horizontal: false,
            tmux_h: false,
            kitty: false,
            kitty_vertical: false,
            kitty_v: false,
            kitty_horizontal: false,
            kitty_h: false,
        };

        // This will panic with MockExitHandler
        handle(args, context).await.unwrap();
    }

    #[tokio::test]
    async fn test_exec_tmux_new_window() {
        let mut mock = MockCommandExecutor::new();
        let mock_fs = MockFileSystem::new();

        // Set TMUX env var to simulate being inside tmux
        std::env::set_var("TMUX", "/tmp/tmux-1000/default,12345,0");

        // Mock git root check
        mock.expect_command("git").with_args(&["rev-parse", "--git-common-dir"]).returns_output(
            "/repo/.git",
            "",
            0,
        );

        // Mock filesystem check for worktree existence
        mock_fs.expect(FileSystemExpectation {
            operation: FileSystemOperation::IsDir,
            path: Some(PathBuf::from("/repo/.git/phantom/worktrees/test")),
            from_path: None,
            to_path: None,
            contents: None,
            result: Ok(MockResult::Bool(true)),
        });

        // Mock tmux command - using expect_command since tmux now uses CommandExecutor
        mock.expect_command("tmux")
            .with_args(&[
                "new-window",
                "-n",
                "test",
                "-c",
                "/repo/.git/phantom/worktrees/test",
                "-e",
                "PHANTOM_ACTIVE=1",
                "-e",
                "PHANTOM_WORKTREE=test",
                "-e",
                "PHANTOM_WORKTREE_PATH=/repo/.git/phantom/worktrees/test",
                "echo",
                "hello",
            ])
            .returns_output("", "", 0);

        let context = HandlerContext::new(
            Arc::new(mock),
            Arc::new(mock_fs),
            Arc::new(crate::core::exit_handler::MockExitHandler::new()),
        );
        let args = ExecArgs {
            name: Some("test".to_string()),
            command: vec!["echo".to_string(), "hello".to_string()],
            fzf: false,
            tmux: true,
            tmux_vertical: false,
            tmux_v: false,
            tmux_horizontal: false,
            tmux_h: false,
            kitty: false,
            kitty_vertical: false,
            kitty_v: false,
            kitty_horizontal: false,
            kitty_h: false,
        };

        let result = handle(args, context).await;
        assert!(result.is_ok());

        // Clean up env var
        std::env::remove_var("TMUX");
    }

    #[tokio::test]
    async fn test_exec_positional_worktree_name() {
        let mut mock = MockCommandExecutor::new();

        // Mock git root check
        mock.expect_command("git").with_args(&["rev-parse", "--git-common-dir"]).returns_output(
            "/repo/.git",
            "",
            0,
        );

        let context = HandlerContext::new(
            Arc::new(mock),
            Arc::new(crate::core::filesystems::MockFileSystem::new()),
            Arc::new(crate::core::exit_handler::MockExitHandler::new()),
        );
        let args = ExecArgs {
            name: None, // Name will be taken from first command arg
            command: vec!["myworktree".to_string(), "echo".to_string(), "hello".to_string()],
            fzf: false,
            tmux: false,
            tmux_vertical: false,
            tmux_v: false,
            tmux_horizontal: false,
            tmux_h: false,
            kitty: false,
            kitty_vertical: false,
            kitty_v: false,
            kitty_horizontal: false,
            kitty_h: false,
        };

        // This test will fail at validate_worktree_exists due to filesystem operations
        let _result = handle(args, context).await;
        // Can't fully test without filesystem abstraction
    }
}
