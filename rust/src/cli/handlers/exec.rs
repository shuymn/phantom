use crate::cli::commands::exec::ExecArgs;
use crate::cli::output::output;
use crate::git::libs::get_git_root::get_git_root;
use crate::process::exec::exec_in_worktree;
use crate::process::kitty::{
    execute_kitty_command, is_inside_kitty, KittyOptions, KittySplitDirection,
};
use crate::process::shell::get_phantom_env;
use crate::process::tmux::{execute_tmux_command, is_inside_tmux, TmuxOptions, TmuxSplitDirection};
use crate::worktree::select::select_worktree_with_fzf;
use crate::worktree::validate::validate_worktree_exists;
use crate::{PhantomError, Result};
use std::process;

/// Handle the exec command
pub async fn handle(args: ExecArgs) -> Result<()> {
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
    let git_root = get_git_root().await?;

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
    let validation = validate_worktree_exists(&git_root, &worktree_name).await?;
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

        execute_tmux_command(options).await?;
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
    let result = exec_in_worktree(&git_root, &worktree_name, &command, args_slice).await?;

    // Exit with the same code as the executed command
    process::exit(result.exit_code);
}
