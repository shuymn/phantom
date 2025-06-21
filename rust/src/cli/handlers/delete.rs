use crate::cli::commands::delete::{DeleteArgs, DeleteResult};
use crate::cli::context::HandlerContext;
use crate::cli::output::output;
use crate::core::command_executor::CommandExecutor;
use crate::core::exit_handler::ExitHandler;
use crate::core::filesystem::FileSystem;
use crate::git::libs::get_current_worktree::get_current_worktree_with_executor;
use crate::git::libs::get_git_root_generic::get_git_root;
use crate::worktree::delete::delete_worktree_with_executor;
use crate::worktree::select::select_worktree_with_fzf;
use crate::worktree::types::DeleteWorktreeOptions;
use crate::{PhantomError, Result};

/// Handle the delete command
pub async fn handle<E, F, H>(args: DeleteArgs, context: HandlerContext<E, F, H>) -> Result<()>
where
    E: CommandExecutor + Clone + 'static,
    F: FileSystem + Clone + 'static,
    H: ExitHandler + Clone + 'static,
{
    // Validate args
    if args.name.is_none() && !args.current && !args.fzf {
        return Err(PhantomError::Validation(
            "Please provide a worktree name to delete, use --current to delete the current worktree, or use --fzf for interactive selection".to_string(),
        ));
    }

    if (args.name.is_some() || args.fzf) && args.current {
        return Err(PhantomError::Validation(
            "Cannot specify --current with a worktree name or --fzf option".to_string(),
        ));
    }

    if args.name.is_some() && args.fzf {
        return Err(PhantomError::Validation(
            "Cannot specify both a worktree name and --fzf option".to_string(),
        ));
    }

    // Get git root
    let git_root = get_git_root(context.executor.clone()).await?;

    // Get worktree name
    let worktree_name = if args.current {
        let current = get_current_worktree_with_executor(
            std::sync::Arc::new(context.executor.clone()),
            &git_root,
        )
        .await?;
        match current {
            Some(name) => name,
            None => {
                return Err(PhantomError::Validation(
                    "Not in a worktree directory. The --current option can only be used from within a worktree.".to_string(),
                ));
            }
        }
    } else if args.fzf {
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

    // Delete the worktree
    let options = DeleteWorktreeOptions { force: args.force };

    match delete_worktree_with_executor(
        std::sync::Arc::new(context.executor.clone()),
        &git_root,
        &worktree_name,
        options,
        &context.filesystem,
    )
    .await
    {
        Ok(result) => {
            if args.json {
                let json_result = DeleteResult {
                    success: true,
                    name: worktree_name,
                    message: result.message.clone(),
                    error: None,
                };
                let _ = output().json(&json_result);
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
                let _ = output().json(&json_result);
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
