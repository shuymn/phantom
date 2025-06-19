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
