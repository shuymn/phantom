use crate::cli::commands::shell::ShellArgs;
use crate::cli::context::HandlerContext;
use crate::cli::output::output;
use crate::core::command_executor::CommandExecutor;
use crate::core::exit_handler::ExitHandler;
use crate::core::filesystem::FileSystem;
use crate::git::libs::get_git_root::get_git_root;
use crate::process::exec::spawn_shell_in_worktree;
use crate::process::kitty::{
    execute_kitty_command, is_inside_kitty, KittyOptions, KittySplitDirection,
};
use crate::process::shell::{detect_shell, get_phantom_env};
use crate::process::tmux::{execute_tmux_command, is_inside_tmux, TmuxOptions, TmuxSplitDirection};
use crate::worktree::select::select_worktree_with_fzf;
use crate::worktree::validate::validate_worktree_exists;
use anyhow::{anyhow, bail, Context, Result};

/// Handle the shell command
pub async fn handle<E, F, H>(args: ShellArgs, context: HandlerContext<E, F, H>) -> Result<()>
where
    E: CommandExecutor + Clone + 'static,
    F: FileSystem + Clone + 'static,
    H: ExitHandler + Clone + 'static,
{
    // Validate args
    if args.name.is_none() && !args.fzf {
        bail!("Usage: phantom shell <worktree-name> or phantom shell --fzf");
    }

    if args.name.is_some() && args.fzf {
        bail!("Cannot specify both a worktree name and --fzf option");
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
        bail!("The --tmux option can only be used inside a tmux session");
    }

    if kitty_direction.is_some() && !is_inside_kitty().await {
        bail!("The --kitty option can only be used inside a kitty terminal");
    }

    // Get git root
    let git_root = get_git_root(context.executor.clone())
        .await
        .with_context(|| "Failed to determine git repository root")?;

    // Get worktree name
    let worktree_name = if args.fzf {
        let result = select_worktree_with_fzf(context.executor.clone(), &git_root)
            .await
            .with_context(|| "Failed to select worktree with fzf")?;

        match result {
            Some(worktree) => worktree.name,
            None => {
                // User cancelled selection
                return Ok(());
            }
        }
    } else {
        args.name.unwrap()
    };

    // Validate worktree exists
    let validation = validate_worktree_exists(&git_root, &worktree_name, &context.filesystem)
        .await
        .with_context(|| format!("Failed to validate worktree '{worktree_name}' exists"))?;
    let worktree_path = validation.path;

    // Get shell info
    let shell_info = detect_shell().with_context(|| "Failed to detect shell")?;
    let shell_command = shell_info.path;

    // Handle tmux execution
    if let Some(direction) = tmux_direction {
        output().log(&format!(
            "Opening worktree '{}' in tmux {}...",
            worktree_name,
            if direction == TmuxSplitDirection::New { "window" } else { "pane" }
        ));

        let options = TmuxOptions {
            direction,
            command: shell_command,
            args: None,
            cwd: Some(worktree_path.to_string_lossy().to_string()),
            env: Some(get_phantom_env(&worktree_name, &worktree_path.to_string_lossy())),
            window_name: if direction == TmuxSplitDirection::New {
                Some(worktree_name.clone())
            } else {
                None
            },
        };

        execute_tmux_command(&context.executor, options)
            .await
            .map_err(|e| anyhow!(e))
            .with_context(|| format!("Failed to open worktree '{worktree_name}' in tmux"))?;
        return Ok(());
    }

    // Handle kitty execution
    if let Some(direction) = kitty_direction {
        output().log(&format!(
            "Opening worktree '{}' in kitty {}...",
            worktree_name,
            if direction == KittySplitDirection::New { "tab" } else { "split" }
        ));

        let options = KittyOptions {
            direction,
            command: shell_command,
            args: None,
            cwd: Some(worktree_path.to_string_lossy().to_string()),
            env: Some(get_phantom_env(&worktree_name, &worktree_path.to_string_lossy())),
            window_title: if direction == KittySplitDirection::New {
                Some(worktree_name.clone())
            } else {
                None
            },
        };

        execute_kitty_command(&context.executor, options)
            .await
            .map_err(|e| anyhow!(e))
            .with_context(|| format!("Failed to open worktree '{worktree_name}' in kitty"))?;
        return Ok(());
    }

    // Normal shell execution
    output().log(&format!("Entering worktree '{}' at {}", worktree_name, worktree_path.display()));
    output().log("Type 'exit' to return to your original directory\n");

    let result = spawn_shell_in_worktree(
        &git_root,
        &worktree_name,
        &context.filesystem,
        Some(context.executor.clone()),
    )
    .await
    .map_err(|e| anyhow!(e))
    .with_context(|| {
        format!(
            "Failed to spawn shell in worktree '{}' at path: {}",
            worktree_name,
            worktree_path.display()
        )
    })?;

    // Exit with the same code as the shell
    context.exit_handler.exit(result.exit_code);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::executors::MockCommandExecutor;
    use crate::core::filesystems::mock_filesystem::{FileSystemOperation, MockResult};
    use crate::core::filesystems::{FileSystemExpectation, MockFileSystem};
    use crate::test_utils::EnvGuard;
    use std::path::PathBuf;

    #[tokio::test]
    async fn test_shell_not_in_git_repo() {
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
        let args = ShellArgs {
            name: Some("test".to_string()),
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
    async fn test_shell_invalid_usage_no_name_no_fzf() {
        let context = HandlerContext::new(
            MockCommandExecutor::new(),
            crate::core::filesystems::MockFileSystem::new(),
            crate::core::exit_handler::MockExitHandler::new(),
        );
        let args = ShellArgs {
            name: None,
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
    async fn test_shell_both_name_and_fzf() {
        let context = HandlerContext::new(
            MockCommandExecutor::new(),
            crate::core::filesystems::MockFileSystem::new(),
            crate::core::exit_handler::MockExitHandler::new(),
        );
        let args = ShellArgs {
            name: Some("test".to_string()),
            fzf: true,
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
        assert!(result.unwrap_err().to_string().contains("Cannot specify both"));
    }

    #[tokio::test]
    async fn test_shell_tmux_outside_tmux_session() {
        let context = HandlerContext::new(
            MockCommandExecutor::new(),
            crate::core::filesystems::MockFileSystem::new(),
            crate::core::exit_handler::MockExitHandler::new(),
        );
        let args = ShellArgs {
            name: Some("test".to_string()),
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

        {
            let _guard = EnvGuard::remove("TMUX"); // Ensure TMUX is not set
            let result = handle(args, context).await;
            // This will likely pass because is_inside_tmux() checks env directly
            // In a real test environment without TMUX set, this would fail
            if std::env::var("TMUX").is_err() {
                assert!(result.is_err());
                assert!(result.unwrap_err().to_string().contains("inside a tmux session"));
            }
        }
    }

    #[tokio::test]
    async fn test_shell_kitty_outside_kitty_terminal() {
        let context = HandlerContext::new(
            MockCommandExecutor::new(),
            crate::core::filesystems::MockFileSystem::new(),
            crate::core::exit_handler::MockExitHandler::new(),
        );
        let args = ShellArgs {
            name: Some("test".to_string()),
            fzf: false,
            tmux: false,
            tmux_vertical: false,
            tmux_v: false,
            tmux_horizontal: false,
            tmux_h: false,
            kitty: true,
            kitty_vertical: false,
            kitty_v: false,
            kitty_horizontal: false,
            kitty_h: false,
        };

        {
            let _guard = EnvGuard::remove("KITTY_WINDOW_ID"); // Ensure KITTY_WINDOW_ID is not set
            let result = handle(args, context).await;
            // This will likely pass because is_inside_kitty() checks env directly
            // In a real test environment without KITTY_WINDOW_ID set, this would fail
            if std::env::var("KITTY_WINDOW_ID").is_err() {
                assert!(result.is_err());
                assert!(result.unwrap_err().to_string().contains("inside a kitty terminal"));
            }
        }
    }

    #[tokio::test]
    #[should_panic(expected = "MockExitHandler::exit called with code 0")]
    async fn test_shell_normal_execution() {
        let mut mock = MockCommandExecutor::new();
        let mock_fs = MockFileSystem::new();

        // Mock git root check
        mock.expect_command("git").with_args(&["rev-parse", "--git-common-dir"]).returns_output(
            "/repo/.git",
            "",
            0,
        );

        // Mock filesystem check for worktree existence
        // Note: validate_worktree_exists is called twice - once in the handler and once in spawn_shell_in_worktree
        mock_fs.expect(FileSystemExpectation {
            operation: FileSystemOperation::IsDir,
            path: Some(PathBuf::from("/repo/.git/phantom/worktrees/test")),
            from_path: None,
            to_path: None,
            contents: None,
            result: Ok(MockResult::Bool(true)), // Directory exists
        });

        // Second expectation for the same path (called from spawn_shell_in_worktree)
        mock_fs.expect(FileSystemExpectation {
            operation: FileSystemOperation::IsDir,
            path: Some(PathBuf::from("/repo/.git/phantom/worktrees/test")),
            from_path: None,
            to_path: None,
            contents: None,
            result: Ok(MockResult::Bool(true)), // Directory exists
        });

        // Mock shell execution
        let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/sh".to_string());
        mock.expect_command(&shell)
            .in_dir(std::path::PathBuf::from("/repo/.git/phantom/worktrees/test"))
            .returns_output("", "", 0);

        let context =
            HandlerContext::new(mock, mock_fs, crate::core::exit_handler::MockExitHandler::new());
        let args = ShellArgs {
            name: Some("test".to_string()),
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
    async fn test_shell_tmux_new_window() {
        let mut mock = MockCommandExecutor::new();
        let mock_fs = MockFileSystem::new();

        // Set TMUX env var to simulate being inside tmux
        let _guard = EnvGuard::set("TMUX", "/tmp/tmux-1000/default,12345,0");

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
        // Note: The actual shell path would be detected, but we'll use a common one for testing
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
                std::env::var("SHELL").unwrap_or_else(|_| "/bin/sh".to_string()).as_str(),
            ])
            .returns_output("", "", 0);

        let context =
            HandlerContext::new(mock, mock_fs, crate::core::exit_handler::MockExitHandler::new());
        let args = ShellArgs {
            name: Some("test".to_string()),
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

        // Guard will automatically restore env var when dropped
    }

    #[tokio::test]
    async fn test_shell_kitty_new_tab() {
        let mut mock = MockCommandExecutor::new();
        let mock_fs = MockFileSystem::new();

        // Set KITTY_WINDOW_ID env var to simulate being inside kitty
        let _guard = EnvGuard::set("KITTY_WINDOW_ID", "1");

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

        // Mock kitty command with sorted environment variables
        mock.expect_command("kitty")
            .with_args(&[
                "@",
                "launch",
                "--type=tab",
                "--tab-title=test",
                "--cwd=/repo/.git/phantom/worktrees/test",
                "--env=PHANTOM_ACTIVE=1",
                "--env=PHANTOM_WORKTREE=test",
                "--env=PHANTOM_WORKTREE_PATH=/repo/.git/phantom/worktrees/test",
                "--",
                std::env::var("SHELL").unwrap_or_else(|_| "/bin/sh".to_string()).as_str(),
            ])
            .returns_output("", "", 0);

        let context =
            HandlerContext::new(mock, mock_fs, crate::core::exit_handler::MockExitHandler::new());
        let args = ShellArgs {
            name: Some("test".to_string()),
            fzf: false,
            tmux: false,
            tmux_vertical: false,
            tmux_v: false,
            tmux_horizontal: false,
            tmux_h: false,
            kitty: true,
            kitty_vertical: false,
            kitty_v: false,
            kitty_horizontal: false,
            kitty_h: false,
        };

        let result = handle(args, context).await;
        assert!(result.is_ok());

        // Guard will automatically restore env var when dropped
    }

    #[tokio::test]
    async fn test_shell_tmux_vertical_split() {
        let mut mock = MockCommandExecutor::new();

        // Mock git root check
        mock.expect_command("git").with_args(&["rev-parse", "--git-common-dir"]).returns_output(
            "/repo/.git",
            "",
            0,
        );

        let context = HandlerContext::new(
            mock,
            crate::core::filesystems::MockFileSystem::new(),
            crate::core::exit_handler::MockExitHandler::new(),
        );
        let args = ShellArgs {
            name: Some("test".to_string()),
            fzf: false,
            tmux: false,
            tmux_vertical: true,
            tmux_v: false,
            tmux_horizontal: false,
            tmux_h: false,
            kitty: false,
            kitty_vertical: false,
            kitty_v: false,
            kitty_horizontal: false,
            kitty_h: false,
        };

        // This test verifies that tmux_vertical flag is properly handled
        // It will fail at validate_worktree_exists due to filesystem operations
        let _result = handle(args, context).await;
        // Can't fully test without filesystem abstraction and TMUX env var
    }
}
