use crate::cli::commands::attach::AttachArgs;
use crate::cli::output::output;
use crate::git::libs::branch_exists::branch_exists;
use crate::git::libs::get_git_root::get_git_root;
use crate::process::exec::exec_in_dir;
use crate::process::shell::shell_in_dir;
use crate::worktree::attach::attach_worktree;
use crate::worktree::paths::get_worktree_path;
use crate::worktree::validate::validate_worktree_name;
use crate::{PhantomError, Result};
use tokio::fs;

/// Handle the attach command
pub async fn handle(args: AttachArgs) -> Result<()> {
    // Validate branch name
    validate_worktree_name(&args.branch)?;

    // Get git root
    let git_root = get_git_root().await?;

    // Check if worktree already exists
    let worktree_path = get_worktree_path(&git_root, &args.branch);
    if fs::metadata(&worktree_path).await.is_ok() {
        return Err(PhantomError::WorktreeExists { name: args.branch.clone() });
    }

    // Check if branch exists
    if !branch_exists(&git_root, &args.branch).await? {
        return Err(PhantomError::BranchNotFound { branch: args.branch.clone() });
    }

    // Attach the worktree
    attach_worktree(&git_root, &args.branch).await?;

    output().success(&format!("Attached phantom: {}", args.branch));

    // Handle post-attach actions
    if args.shell {
        shell_in_dir(&worktree_path).await?;
    } else if let Some(exec_cmd) = args.exec {
        let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/sh".to_string());
        exec_in_dir(&worktree_path, &shell, &["-c".to_string(), exec_cmd]).await?;
    }

    Ok(())
}
