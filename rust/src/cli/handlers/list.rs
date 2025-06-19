use crate::cli::commands::list::ListArgs;
use crate::cli::context::HandlerContext;
use crate::cli::output::output;
use crate::git::libs::get_git_root::get_git_root_with_executor;
use crate::worktree::list::list_worktrees;
use crate::worktree::select::select_worktree_with_fzf;
use crate::Result;
use serde::Serialize;

#[derive(Serialize)]
struct ListJsonOutput {
    worktrees: Vec<WorktreeJsonItem>,
}

#[derive(Serialize)]
struct WorktreeJsonItem {
    name: String,
    branch: Option<String>,
    is_clean: bool,
    path: String,
}

/// Handle the list command
pub async fn handle(args: ListArgs, context: HandlerContext) -> Result<()> {
    let git_root = get_git_root_with_executor(context.executor.clone()).await?;

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
            if args.json {
                let json_output = ListJsonOutput { worktrees: vec![] };
                output().log(&serde_json::to_string_pretty(&json_output)?);
            } else if !args.names {
                output().log(result.message.as_deref().unwrap_or("No worktrees found."));
            }
            return Ok(());
        }

        if args.json {
            // Output as JSON
            let json_worktrees: Vec<WorktreeJsonItem> = result
                .worktrees
                .iter()
                .map(|w| WorktreeJsonItem {
                    name: w.name.clone(),
                    branch: w.branch.clone(),
                    is_clean: w.is_clean,
                    path: w.path.clone(),
                })
                .collect();

            let json_output = ListJsonOutput { worktrees: json_worktrees };

            output().log(&serde_json::to_string_pretty(&json_output)?);
        } else if args.names {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::executors::MockCommandExecutor;
    use std::sync::Arc;

    // IMPORTANT: Mock testing lesson learned
    //
    // The list handler demonstrates why partial migration to CommandExecutor doesn't work.
    // The handler calls list_worktrees() which uses GitExecutor directly, bypassing our mocks.
    // 
    // We can only test paths that fail before reaching unmigrated code (like git root check).
    // Full handler testing requires ALL dependencies to use CommandExecutor.
    //
    // TODO: Complete migration of list_worktrees and its dependencies before adding tests.

    #[tokio::test]
    async fn test_list_not_in_git_repo() {
        let mut mock = MockCommandExecutor::new();
        
        // This test works because it fails early at git root check
        mock.expect_command("git")
            .with_args(&["rev-parse", "--git-common-dir"])
            .returns_output("", "fatal: not a git repository", 128);
        
        let context = HandlerContext::new(Arc::new(mock));
        let args = ListArgs {
            fzf: false,
            json: false,
            names: false,
        };
        
        let result = handle(args, context).await;
        assert!(result.is_err());
    }

    // Future tests to implement after migration:
    // - test_list_empty_worktrees
    // - test_list_with_worktrees  
    // - test_list_json_output
    // - test_list_names_only
    // - test_list_with_dirty_worktrees
    //
    // These tests are documented in the git history if needed for reference.
}
