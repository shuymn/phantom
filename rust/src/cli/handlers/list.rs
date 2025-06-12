use crate::cli::commands::list::ListArgs;
use crate::cli::output::output;
use crate::git::libs::get_git_root::get_git_root;
use crate::worktree::list::list_worktrees;
use crate::worktree::select::select_worktree_with_fzf;
use crate::Result;

/// Handle the list command
pub async fn handle(args: ListArgs) -> Result<()> {
    let git_root = get_git_root().await?;

    if args.fzf {
        // Use fzf for interactive selection
        match select_worktree_with_fzf(&git_root).await? {
            Some(worktree) => {
                output().log(&worktree.name);
            }
            None => {
                // User cancelled selection
            }
        }
    } else {
        // List all worktrees
        let result = list_worktrees(&git_root).await?;

        if result.worktrees.is_empty() {
            if !args.names {
                output().log(result.message.as_deref().unwrap_or("No worktrees found."));
            }
            return Ok(());
        }

        if args.names {
            // Output only names
            for worktree in &result.worktrees {
                output().log(&worktree.name);
            }
        } else {
            // Output formatted list
            let max_name_length = result.worktrees.iter().map(|w| w.name.len()).max().unwrap_or(0);

            for worktree in &result.worktrees {
                let padded_name = format!("{:<width$}", worktree.name, width = max_name_length + 2);
                let branch_info =
                    worktree.branch.as_ref().map(|b| format!("({})", b)).unwrap_or_default();
                let status = if !worktree.is_clean { " [dirty]" } else { "" };

                output().log(&format!("{}{}{}", padded_name, branch_info, status));
            }
        }
    }

    Ok(())
}
