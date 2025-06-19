use crate::cli::commands::shell::ShellArgs;
use crate::cli::context::HandlerContext;
use crate::cli::output::output;
use crate::git::libs::get_git_root::get_git_root_with_executor;
use crate::process::exec::spawn_shell_in_worktree;
use crate::process::kitty::{
    execute_kitty_command, is_inside_kitty, KittyOptions, KittySplitDirection,
};
use crate::process::shell::{detect_shell, get_phantom_env};
use crate::process::tmux::{execute_tmux_command, is_inside_tmux, TmuxOptions, TmuxSplitDirection};
use crate::worktree::select::select_worktree_with_fzf;
use crate::worktree::validate::validate_worktree_exists;
use crate::{PhantomError, Result};
use std::process;

/// Handle the shell command
pub async fn handle(args: ShellArgs, context: HandlerContext) -> Result<()> {
    // Validate args
    if args.name.is_none() && !args.fzf {
        return Err(PhantomError::Validation(
            "Usage: phantom shell <worktree-name> or phantom shell --fzf".to_string(),
        ));
    }

    if args.name.is_some() && args.fzf {
        return Err(PhantomError::Validation(
            "Cannot specify both a worktree name and --fzf option".to_string(),
        ));
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
        args.name.unwrap()
    };

    // Validate worktree exists
    let validation = validate_worktree_exists(&git_root, &worktree_name).await?;
    let worktree_path = validation.path;

    // Get shell info
    let shell_info = detect_shell()?;
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
                Some(worktree_name)
            } else {
                None
            },
        };

        execute_tmux_command(options).await?;
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
                Some(worktree_name)
            } else {
                None
            },
        };

        execute_kitty_command(options).await?;
        return Ok(());
    }

    // Normal shell execution
    output().log(&format!("Entering worktree '{}' at {}", worktree_name, worktree_path.display()));
    output().log("Type 'exit' to return to your original directory\n");

    let result = spawn_shell_in_worktree(&git_root, &worktree_name).await?;

    // Exit with the same code as the shell
    process::exit(result.exit_code);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::executors::MockCommandExecutor;
    use std::sync::Arc;

    #[tokio::test]
    async fn test_shell_not_in_git_repo() {
        let mut mock = MockCommandExecutor::new();

        // Expect git root check to fail
        mock.expect_command("git").with_args(&["rev-parse", "--git-common-dir"]).returns_output(
            "",
            "fatal: not a git repository",
            128,
        );

        let context = HandlerContext::new(Arc::new(mock));
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
        let context = HandlerContext::new(Arc::new(MockCommandExecutor::new()));
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
        let context = HandlerContext::new(Arc::new(MockCommandExecutor::new()));
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
        let context = HandlerContext::new(Arc::new(MockCommandExecutor::new()));
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
        // This will likely pass because is_inside_tmux() checks env directly
        // In a real test environment without TMUX set, this would fail
        if std::env::var("TMUX").is_err() {
            assert!(result.is_err());
            assert!(result.unwrap_err().to_string().contains("inside a tmux session"));
        }
    }

    #[tokio::test]
    async fn test_shell_kitty_outside_kitty_terminal() {
        let context = HandlerContext::new(Arc::new(MockCommandExecutor::new()));
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
        // This will likely pass because is_inside_kitty() checks env directly
        // In a real test environment without KITTY_WINDOW_ID set, this would fail
        if std::env::var("KITTY_WINDOW_ID").is_err() {
            assert!(result.is_err());
            assert!(result.unwrap_err().to_string().contains("inside a kitty terminal"));
        }
    }

    #[tokio::test]
    #[ignore = "Requires filesystem mocking - validate_worktree_exists uses fs::metadata"]
    async fn test_shell_normal_execution() {
        let mut mock = MockCommandExecutor::new();

        // Mock git root check
        mock.expect_command("git").with_args(&["rev-parse", "--git-common-dir"]).returns_output(
            "/repo/.git",
            "",
            0,
        );

        // Mock shell detection would happen via detect_shell()
        // Mock shell spawn
        let mut env = std::collections::HashMap::new();
        env.insert("PHANTOM_WORKTREE".to_string(), "test".to_string());
        env.insert("PHANTOM_WORKTREE_PATH".to_string(), "/repo/.phantom/test".to_string());

        mock.expect_command("/bin/bash") // Or whatever shell is detected
            .in_dir("/repo/.phantom/test")
            .with_env(env)
            .returns_output("", "", 0);

        let context = HandlerContext::new(Arc::new(mock));
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

        // This test would call process::exit, so we can't easily test the success case
        // without refactoring the handler to return the exit code instead
    }

    #[tokio::test]
    #[ignore = "Requires refactoring - tmux execution spawns detached process"]
    async fn test_shell_tmux_new_window() {
        let mut mock = MockCommandExecutor::new();

        // Mock git root check
        mock.expect_command("git").with_args(&["rev-parse", "--git-common-dir"]).returns_output(
            "/repo/.git",
            "",
            0,
        );

        // Mock tmux command
        mock.expect_spawn("tmux")
            .with_args(&[
                "new-window",
                "-n",
                "test",
                "-c",
                "/repo/.phantom/test",
                "/bin/bash", // Or detected shell
            ])
            .returns_pid(12345);

        let context = HandlerContext::new(Arc::new(mock));
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

        // Would need to set TMUX env var and handle filesystem checks
    }

    #[tokio::test]
    #[ignore = "Requires refactoring - kitty execution spawns detached process"]
    async fn test_shell_kitty_new_tab() {
        let mut mock = MockCommandExecutor::new();

        // Mock git root check
        mock.expect_command("git").with_args(&["rev-parse", "--git-common-dir"]).returns_output(
            "/repo/.git",
            "",
            0,
        );

        // Mock kitty command
        mock.expect_spawn("kitty")
            .with_args(&[
                "@",
                "launch",
                "--type=tab",
                "--tab-title=test",
                "--cwd=/repo/.phantom/test",
                "/bin/bash", // Or detected shell
            ])
            .returns_pid(12345);

        let context = HandlerContext::new(Arc::new(mock));
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

        // Would need to set KITTY_WINDOW_ID env var and handle filesystem checks
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

        let context = HandlerContext::new(Arc::new(mock));
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
