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
    .with_context(|| format!("Failed to delete worktree '{}'", worktree_name))
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
#[path = "delete_test.rs"]
mod tests;
