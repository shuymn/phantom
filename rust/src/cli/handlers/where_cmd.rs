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
