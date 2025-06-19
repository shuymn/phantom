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

    // Note: These tests demonstrate the challenge of partial migration.
    // The list handler calls list_worktrees() which still uses the old GitExecutor
    // directly, not our CommandExecutor. Until list_worktrees is refactored to
    // accept a CommandExecutor, we cannot effectively test the list handler with mocks.
    //
    // This is tracked in TODO: "Update list_worktrees git operation to use CommandExecutor"

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

    #[tokio::test]
    #[ignore = "Requires list_worktrees to be refactored to use CommandExecutor"]
    async fn test_list_empty_worktrees() {
        let mut mock = MockCommandExecutor::new();
        
        // Expect git root check
        mock.expect_command("git")
            .with_args(&["rev-parse", "--git-common-dir"])
            .returns_output("/path/to/repo/.git", "", 0);
        
        // Expect worktree list - only main
        mock.expect_command("git")
            .with_args(&["worktree", "list", "--porcelain"])
            .returns_output("worktree /path/to/repo\nHEAD abc123\nbranch refs/heads/main\n\n", "", 0);
        
        let context = HandlerContext::new(Arc::new(mock));
        let args = ListArgs {
            fzf: false,
            json: false,
            names: false,
        };
        
        let result = handle(args, context).await;
        if let Err(e) = &result {
            eprintln!("Test failed with error: {:?}", e);
        }
        assert!(result.is_ok());
        // Note: Should output "No phantom worktrees found in the main repository."
    }

    #[tokio::test]
    #[ignore = "Requires list_worktrees to be refactored to use CommandExecutor"]
    async fn test_list_with_worktrees() {
        let mut mock = MockCommandExecutor::new();
        
        // Expect git root check
        mock.expect_command("git")
            .with_args(&["rev-parse", "--git-common-dir"])
            .returns_output("/path/to/repo/.git", "", 0);
        
        // Expect worktree list with multiple worktrees
        mock.expect_command("git")
            .with_args(&["worktree", "list", "--porcelain"])
            .returns_output(concat!(
                "worktree /path/to/repo\n",
                "HEAD abc123\n",
                "branch refs/heads/main\n",
                "\n",
                "worktree /path/to/repo/phantoms/feature-1\n",
                "HEAD def456\n", 
                "branch refs/heads/feature-1\n",
                "\n",
                "worktree /path/to/repo/phantoms/feature-2\n",
                "HEAD ghi789\n",
                "branch refs/heads/feature-2\n",
                "\n"
            ), "", 0);
        
        // Expect status checks for each worktree
        mock.expect_command("git")
            .with_args(&["branch", "--show-current"])
            .in_dir("/path/to/repo/phantoms/feature-1")
            .returns_output("feature-1", "", 0);
        
        mock.expect_command("git")
            .with_args(&["status", "--porcelain"])
            .in_dir("/path/to/repo/phantoms/feature-1")
            .returns_output("", "", 0); // Clean
        
        mock.expect_command("git")
            .with_args(&["branch", "--show-current"])
            .in_dir("/path/to/repo/phantoms/feature-2")
            .returns_output("feature-2", "", 0);
        
        mock.expect_command("git")
            .with_args(&["status", "--porcelain"])
            .in_dir("/path/to/repo/phantoms/feature-2")
            .returns_output("M file.txt\n", "", 0); // Dirty
        
        let context = HandlerContext::new(Arc::new(mock));
        let args = ListArgs {
            fzf: false,
            json: false,
            names: false,
        };
        
        let result = handle(args, context).await;
        assert!(result.is_ok());
        // Should list both worktrees with their status
    }

    #[tokio::test]
    #[ignore = "Requires list_worktrees to be refactored to use CommandExecutor"]
    async fn test_list_json_output() {
        let mut mock = MockCommandExecutor::new();
        
        // Expect git root check
        mock.expect_command("git")
            .with_args(&["rev-parse", "--git-common-dir"])
            .returns_output("/path/to/repo/.git", "", 0);
        
        // Expect worktree list
        mock.expect_command("git")
            .with_args(&["worktree", "list", "--porcelain"])
            .returns_output(concat!(
                "worktree /path/to/repo\n",
                "HEAD abc123\n",
                "branch refs/heads/main\n",
                "\n",
                "worktree /path/to/repo/phantoms/feature-json\n",
                "HEAD def456\n",
                "branch refs/heads/feature-json\n",
                "\n"
            ), "", 0);
        
        // Expect status checks
        mock.expect_command("git")
            .with_args(&["branch", "--show-current"])
            .in_dir("/path/to/repo/phantoms/feature-json")
            .returns_output("feature-json", "", 0);
        
        mock.expect_command("git")
            .with_args(&["status", "--porcelain"])
            .in_dir("/path/to/repo/phantoms/feature-json")
            .returns_output("", "", 0);
        
        let context = HandlerContext::new(Arc::new(mock));
        let args = ListArgs {
            fzf: false,
            json: true,
            names: false,
        };
        
        let result = handle(args, context).await;
        assert!(result.is_ok());
        // Should output JSON format
    }

    #[tokio::test]
    #[ignore = "Requires list_worktrees to be refactored to use CommandExecutor"]
    async fn test_list_names_only() {
        let mut mock = MockCommandExecutor::new();
        
        // Expect git root check
        mock.expect_command("git")
            .with_args(&["rev-parse", "--git-common-dir"])
            .returns_output("/path/to/repo/.git", "", 0);
        
        // Expect worktree list
        mock.expect_command("git")
            .with_args(&["worktree", "list", "--porcelain"])
            .returns_output(concat!(
                "worktree /path/to/repo\n",
                "HEAD abc123\n",
                "branch refs/heads/main\n",
                "\n",
                "worktree /path/to/repo/phantoms/name-1\n",
                "HEAD def456\n",
                "branch refs/heads/name-1\n",
                "\n",
                "worktree /path/to/repo/phantoms/name-2\n",
                "HEAD ghi789\n",
                "branch refs/heads/name-2\n",
                "\n"
            ), "", 0);
        
        // No status checks needed for names-only mode
        
        let context = HandlerContext::new(Arc::new(mock));
        let args = ListArgs {
            fzf: false,
            json: false,
            names: true,
        };
        
        let result = handle(args, context).await;
        assert!(result.is_ok());
        // Should output only names: name-1, name-2
    }

    #[tokio::test]
    #[ignore = "Requires list_worktrees to be refactored to use CommandExecutor"]
    async fn test_list_json_empty() {
        let mut mock = MockCommandExecutor::new();
        
        // Expect git root check
        mock.expect_command("git")
            .with_args(&["rev-parse", "--git-common-dir"])
            .returns_output("/path/to/repo/.git", "", 0);
        
        // Expect worktree list - only main
        mock.expect_command("git")
            .with_args(&["worktree", "list", "--porcelain"])
            .returns_output("worktree /path/to/repo\nHEAD abc123\nbranch refs/heads/main\n\n", "", 0);
        
        let context = HandlerContext::new(Arc::new(mock));
        let args = ListArgs {
            fzf: false,
            json: true,
            names: false,
        };
        
        let result = handle(args, context).await;
        assert!(result.is_ok());
        // Should output empty JSON array: {"worktrees":[]}
    }
}
