use crate::cli::commands::attach::AttachArgs;
use crate::cli::context::HandlerContext;
use crate::cli::output::output;
use crate::core::command_executor::CommandExecutor;
use crate::core::exit_handler::ExitHandler;
use crate::core::filesystem::FileSystem;
use crate::git::libs::branch_exists::branch_exists;
use crate::git::libs::get_git_root::get_git_root;
use crate::process::exec::exec_in_dir;
use crate::process::shell::shell_in_dir;
use crate::worktree::attach::attach_worktree;
use crate::worktree::paths::get_worktree_path;
use crate::worktree::validate::validate_worktree_name;
use anyhow::{anyhow, bail, Context, Result};
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
pub async fn handle<E, F, H>(args: AttachArgs, context: HandlerContext<E, F, H>) -> Result<()>
where
    E: CommandExecutor + Clone + 'static,
    F: FileSystem + Clone + 'static,
    H: ExitHandler + Clone + 'static,
{
    // Validate branch name
    validate_worktree_name(&args.branch)?;

    // Get git root
    let git_root = get_git_root(context.executor.clone()).await?;

    // Check if worktree already exists
    let worktree_path = get_worktree_path(&git_root, &args.branch);
    if fs::metadata(&worktree_path).await.is_ok() {
        bail!("Worktree '{}' already exists at path: {}", args.branch, worktree_path.display());
    }

    // Check if branch exists
    if !branch_exists(context.executor.clone(), &git_root, &args.branch)
        .await
        .with_context(|| format!("Failed to check if branch '{}' exists", args.branch))?
    {
        return Err(crate::PhantomError::BranchNotFound { branch: args.branch.clone() }.into());
    }

    // Attach the worktree
    attach_worktree(context.executor.clone(), &git_root, &args.branch)
        .await
        .with_context(|| format!("Failed to attach worktree for branch '{}'", args.branch))?;

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
        shell_in_dir(&context.executor, &worktree_path)
            .await
            .map_err(|e| anyhow!(e))
            .with_context(|| {
                format!("Failed to open shell in worktree path: {}", worktree_path.display())
            })?;
    } else if let Some(exec_cmd) = args.exec {
        let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/sh".to_string());
        exec_in_dir(&worktree_path, &shell, &["-c".to_string(), exec_cmd.clone()])
            .await
            .map_err(|e| anyhow!(e))
            .with_context(|| {
                format!(
                    "Failed to execute command '{}' in worktree path: {}",
                    exec_cmd,
                    worktree_path.display()
                )
            })?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::commands::attach::AttachArgs;
    use crate::cli::context::HandlerContext;
    use crate::core::executors::MockCommandExecutor;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_attach_success() {
        let temp_dir = tempdir().unwrap();
        let git_root = temp_dir.path();
        let git_root_canonical = git_root.canonicalize().unwrap();
        let worktree_path =
            git_root_canonical.join(".git").join("phantom").join("worktrees").join("test-branch");

        let mut mock = MockCommandExecutor::new();

        // Mock git rev-parse --git-common-dir
        mock.expect_command("git").with_args(&["rev-parse", "--git-common-dir"]).returns_output(
            &format!("{}/.git", git_root_canonical.to_string_lossy()),
            "",
            0,
        );

        // Mock git show-ref to check branch exists - with canonicalized path
        mock.expect_command("git")
            .with_args(&["show-ref", "--verify", "--quiet", "refs/heads/test-branch"])
            .in_dir(&git_root_canonical)
            .returns_success();

        // Mock git worktree add - with canonicalized path
        mock.expect_command("git")
            .with_args(&["worktree", "add", &worktree_path.to_string_lossy(), "test-branch"])
            .in_dir(&git_root_canonical)
            .returns_success();

        let args =
            AttachArgs { branch: "test-branch".to_string(), json: false, shell: false, exec: None };

        let context = HandlerContext::new(
            mock,
            crate::core::filesystems::MockFileSystem::new(),
            crate::core::exit_handler::MockExitHandler::new(),
        );

        let result = handle(args, context).await;
        if let Err(e) = &result {
            eprintln!("Handle failed with error: {e:?}");
        }
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_attach_branch_not_found() {
        let temp_dir = tempdir().unwrap();
        let git_root = temp_dir.path();
        let git_root_canonical = git_root.canonicalize().unwrap();

        let mut mock = MockCommandExecutor::new();

        // Mock git rev-parse --git-common-dir
        mock.expect_command("git").with_args(&["rev-parse", "--git-common-dir"]).returns_output(
            &format!("{}/.git", git_root.to_string_lossy()),
            "",
            0,
        );

        // Mock git show-ref to check branch exists (returns false) - with canonicalized path
        mock.expect_command("git")
            .with_args(&["show-ref", "--verify", "--quiet", "refs/heads/nonexistent"])
            .in_dir(&git_root_canonical)
            .returns_output("", "fatal: bad ref for symbolic ref refs/heads/nonexistent\n", 1);

        let args =
            AttachArgs { branch: "nonexistent".to_string(), json: false, shell: false, exec: None };

        let context = HandlerContext::new(
            mock,
            crate::core::filesystems::MockFileSystem::new(),
            crate::core::exit_handler::MockExitHandler::new(),
        );

        let result = handle(args, context).await;
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("Branch 'nonexistent' not found"));
    }

    #[tokio::test]
    async fn test_attach_worktree_already_exists() {
        let temp_dir = tempdir().unwrap();
        let git_root = temp_dir.path();
        let worktree_path =
            git_root.join(".git").join("phantom").join("worktrees").join("existing-branch");

        // Create the worktree directory to simulate it already exists
        std::fs::create_dir_all(&worktree_path).unwrap();

        let mut mock = MockCommandExecutor::new();

        // Mock git rev-parse --git-common-dir
        mock.expect_command("git").with_args(&["rev-parse", "--git-common-dir"]).returns_output(
            &format!("{}/.git", git_root.to_string_lossy()),
            "",
            0,
        );

        let args = AttachArgs {
            branch: "existing-branch".to_string(),
            json: false,
            shell: false,
            exec: None,
        };

        let context = HandlerContext::new(
            mock,
            crate::core::filesystems::MockFileSystem::new(),
            crate::core::exit_handler::MockExitHandler::new(),
        );

        let result = handle(args, context).await;
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("Worktree 'existing-branch' already exists"));
    }

    #[tokio::test]
    async fn test_attach_invalid_worktree_name() {
        let mock = MockCommandExecutor::new();

        let args = AttachArgs { branch: "".to_string(), json: false, shell: false, exec: None };

        let context = HandlerContext::new(
            mock,
            crate::core::filesystems::MockFileSystem::new(),
            crate::core::exit_handler::MockExitHandler::new(),
        );

        let result = handle(args, context).await;
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("empty") || err.to_string().contains("invalid"));
    }

    #[tokio::test]
    async fn test_attach_with_json_output() {
        let temp_dir = tempdir().unwrap();
        let git_root = temp_dir.path();
        let git_root_canonical = git_root.canonicalize().unwrap();
        let worktree_path =
            git_root_canonical.join(".git").join("phantom").join("worktrees").join("json-branch");

        let mut mock = MockCommandExecutor::new();

        // Mock git rev-parse --git-common-dir
        mock.expect_command("git").with_args(&["rev-parse", "--git-common-dir"]).returns_output(
            &format!("{}/.git", git_root_canonical.to_string_lossy()),
            "",
            0,
        );

        // Mock git show-ref to check branch exists - with canonicalized path
        mock.expect_command("git")
            .with_args(&["show-ref", "--verify", "--quiet", "refs/heads/json-branch"])
            .in_dir(&git_root_canonical)
            .returns_success();

        // Mock git worktree add - with canonicalized path
        mock.expect_command("git")
            .with_args(&["worktree", "add", &worktree_path.to_string_lossy(), "json-branch"])
            .in_dir(&git_root_canonical)
            .returns_success();

        let args =
            AttachArgs { branch: "json-branch".to_string(), json: true, shell: false, exec: None };

        let context = HandlerContext::new(
            mock,
            crate::core::filesystems::MockFileSystem::new(),
            crate::core::exit_handler::MockExitHandler::new(),
        );

        let result = handle(args, context).await;
        if let Err(e) = &result {
            eprintln!("Error in test_attach_with_json_output: {e:?}");
        }
        assert!(result.is_ok());
    }
}
