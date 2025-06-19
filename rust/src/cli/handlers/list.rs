use crate::cli::commands::list::ListArgs;
use crate::cli::context::HandlerContext;
use crate::cli::output::output;
use crate::git::libs::get_git_root::get_git_root_with_executor;
use crate::worktree::list::list_worktrees_with_executor;
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
        let result = list_worktrees_with_executor(context.executor.clone(), &git_root).await?;

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
        mock.expect_command("git").with_args(&["rev-parse", "--git-common-dir"]).returns_output(
            "",
            "fatal: not a git repository",
            128,
        );

        let context = HandlerContext::new(
            Arc::new(mock),
            Arc::new(crate::core::filesystems::MockFileSystem::new()),
            Arc::new(crate::core::exit_handler::MockExitHandler::new()),
        );
        let args = ListArgs { fzf: false, json: false, names: false };

        let result = handle(args, context).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_list_empty_worktrees() {
        let mut mock = MockCommandExecutor::new();

        // Mock git root check
        mock.expect_command("git").with_args(&["rev-parse", "--git-common-dir"]).returns_output(
            "/home/user/project/.git",
            "",
            0,
        );

        // Mock worktree list - empty
        mock.expect_command("git").with_args(&["worktree", "list", "--porcelain"]).returns_output(
            "worktree /home/user/project\nHEAD abcd1234\nbranch refs/heads/main\n",
            "",
            0,
        );

        let context = HandlerContext::new(
            Arc::new(mock),
            Arc::new(crate::core::filesystems::MockFileSystem::new()),
            Arc::new(crate::core::exit_handler::MockExitHandler::new()),
        );
        let args = ListArgs { fzf: false, json: false, names: false };

        let result = handle(args, context).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_list_with_worktrees() {
        let mut mock = MockCommandExecutor::new();

        // Mock git root check
        mock.expect_command("git").with_args(&["rev-parse", "--git-common-dir"]).returns_output(
            "/home/user/project/.git",
            "",
            0,
        );

        // Mock worktree list - with phantoms
        mock.expect_command("git").with_args(&["worktree", "list", "--porcelain"]).returns_output(
            "worktree /home/user/project\n\
                HEAD abcd1234\n\
                branch refs/heads/main\n\
                \n\
                worktree /home/user/project/.phantom/feature-1\n\
                HEAD efgh5678\n\
                branch refs/heads/feature-1\n\
                \n\
                worktree /home/user/project/.phantom/feature-2\n\
                HEAD ijkl9012\n\
                branch refs/heads/feature-2\n",
            "",
            0,
        );

        // Mock status checks for each phantom worktree
        mock.expect_command("git")
            .with_args(&["status", "--porcelain"])
            .in_dir("/home/user/project/.phantom/feature-1")
            .returns_output("", "", 0); // Clean

        mock.expect_command("git")
            .with_args(&["status", "--porcelain"])
            .in_dir("/home/user/project/.phantom/feature-2")
            .returns_output("M README.md\n", "", 0); // Dirty

        let context = HandlerContext::new(
            Arc::new(mock),
            Arc::new(crate::core::filesystems::MockFileSystem::new()),
            Arc::new(crate::core::exit_handler::MockExitHandler::new()),
        );
        let args = ListArgs { fzf: false, json: false, names: false };

        let result = handle(args, context).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_list_json_output() {
        let mut mock = MockCommandExecutor::new();

        // Mock git root check
        mock.expect_command("git").with_args(&["rev-parse", "--git-common-dir"]).returns_output(
            "/home/user/project/.git",
            "",
            0,
        );

        // Mock worktree list
        mock.expect_command("git").with_args(&["worktree", "list", "--porcelain"]).returns_output(
            "worktree /home/user/project\n\
                HEAD abcd1234\n\
                branch refs/heads/main\n\
                \n\
                worktree /home/user/project/.phantom/feature-1\n\
                HEAD efgh5678\n\
                branch refs/heads/feature-1\n",
            "",
            0,
        );

        // Mock status check
        mock.expect_command("git")
            .with_args(&["status", "--porcelain"])
            .in_dir("/home/user/project/.phantom/feature-1")
            .returns_output("", "", 0);

        let context = HandlerContext::new(
            Arc::new(mock),
            Arc::new(crate::core::filesystems::MockFileSystem::new()),
            Arc::new(crate::core::exit_handler::MockExitHandler::new()),
        );
        let args = ListArgs { fzf: false, json: true, names: false };

        let result = handle(args, context).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_list_names_only() {
        let mut mock = MockCommandExecutor::new();

        // Mock git root check
        mock.expect_command("git").with_args(&["rev-parse", "--git-common-dir"]).returns_output(
            "/home/user/project/.git",
            "",
            0,
        );

        // Mock worktree list
        mock.expect_command("git").with_args(&["worktree", "list", "--porcelain"]).returns_output(
            "worktree /home/user/project\n\
                HEAD abcd1234\n\
                branch refs/heads/main\n\
                \n\
                worktree /home/user/project/.phantom/feature-1\n\
                HEAD efgh5678\n\
                branch refs/heads/feature-1\n",
            "",
            0,
        );

        // Mock status check
        mock.expect_command("git")
            .with_args(&["status", "--porcelain"])
            .in_dir("/home/user/project/.phantom/feature-1")
            .returns_output("", "", 0);

        let context = HandlerContext::new(
            Arc::new(mock),
            Arc::new(crate::core::filesystems::MockFileSystem::new()),
            Arc::new(crate::core::exit_handler::MockExitHandler::new()),
        );
        let args = ListArgs { fzf: false, json: false, names: true };

        let result = handle(args, context).await;
        assert!(result.is_ok());
    }
}
