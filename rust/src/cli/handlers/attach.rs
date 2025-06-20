use crate::cli::commands::attach::AttachArgs;
use crate::cli::context::HandlerContext;
use crate::cli::output::output;
use crate::git::libs::branch_exists::branch_exists_with_executor;
use crate::git::libs::get_git_root::get_git_root_with_executor;
use crate::process::exec::exec_in_dir;
use crate::process::shell::shell_in_dir;
use crate::worktree::attach::attach_worktree_with_executor;
use crate::worktree::paths::get_worktree_path;
use crate::worktree::validate::validate_worktree_name;
use crate::{PhantomError, Result};
use serde::Serialize;
use tokio::fs;

#[derive(Serialize)]
struct AttachJsonOutput {
    success: bool,
    message: String,
    worktree: String,
    path: String,
}

/// Handle the attach command
pub async fn handle(args: AttachArgs, context: HandlerContext) -> Result<()> {
    // Validate branch name
    validate_worktree_name(&args.branch)?;

    // Get git root
    let git_root = get_git_root_with_executor(context.executor.clone()).await?;

    // Check if worktree already exists
    let worktree_path = get_worktree_path(&git_root, &args.branch);
    if fs::metadata(&worktree_path).await.is_ok() {
        return Err(PhantomError::WorktreeExists { name: args.branch.clone() });
    }

    // Check if branch exists
    if !branch_exists_with_executor(context.executor.clone(), &git_root, &args.branch).await? {
        return Err(PhantomError::BranchNotFound { branch: args.branch.clone() });
    }

    // Attach the worktree
    attach_worktree_with_executor(context.executor.clone(), &git_root, &args.branch).await?;

    if args.json {
        let json_output = AttachJsonOutput {
            success: true,
            message: format!("Attached phantom: {}", args.branch),
            worktree: args.branch.clone(),
            path: worktree_path.to_string_lossy().to_string(),
        };
        output().log(&serde_json::to_string_pretty(&json_output)?);
    } else {
        output().success(&format!("Attached phantom: {}", args.branch));
    }

    // Handle post-attach actions
    if args.shell {
        shell_in_dir(&worktree_path).await?;
    } else if let Some(exec_cmd) = args.exec {
        let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/sh".to_string());
        exec_in_dir(&worktree_path, &shell, &["-c".to_string(), exec_cmd]).await?;
    }

    Ok(())
}

#[cfg(test)]
#[path = "attach_test.rs"]
mod tests;
