use crate::cli::commands::create::{CreateArgs, CreateResult};
use crate::cli::context::HandlerContext;
use crate::cli::output::output;
use crate::config::loader::load_config;
use crate::git::libs::get_git_root::get_git_root_with_executor;
use crate::process::exec::exec_in_dir;
use crate::process::multiplexer::{execute_in_multiplexer, MultiplexerOptions, SplitDirection};
use crate::process::shell::shell_in_dir;
use crate::worktree::create::create_worktree;
use crate::worktree::paths::get_worktree_path;
use crate::worktree::types::CreateWorktreeOptions;
use crate::Result;

/// Handle the create command
pub async fn handle(args: CreateArgs, context: HandlerContext) -> Result<()> {
    // Get git root
    let git_root = match get_git_root_with_executor(context.executor.clone()).await {
        Ok(root) => root,
        Err(e) => {
            if args.json {
                let result = CreateResult {
                    success: false,
                    name: args.name.clone(),
                    branch: args.branch.clone().unwrap_or_else(|| args.name.clone()),
                    path: String::new(),
                    copied_files: None,
                    error: Some(e.to_string()),
                };
                output().json(&result)?;
                return Err(e);
            } else {
                return Err(e);
            }
        }
    };

    // Load config for copy files
    let config = load_config(&git_root).await.ok().flatten();
    let copy_files = if let Some(files) = args.copy_files {
        Some(files)
    } else {
        config.and_then(|cfg| cfg.post_create.and_then(|pc| pc.copy_files))
    };

    // Create the worktree
    let branch_name = args.branch.clone().unwrap_or_else(|| args.name.clone());
    let options = CreateWorktreeOptions {
        branch: Some(branch_name.clone()),
        commitish: args.base.clone(),
        copy_files: copy_files.clone(),
    };

    let result = match create_worktree(&git_root, &args.name, options).await {
        Ok(success) => success,
        Err(e) => {
            if args.json {
                let result = CreateResult {
                    success: false,
                    name: args.name.clone(),
                    branch: branch_name,
                    path: String::new(),
                    copied_files: None,
                    error: Some(e.to_string()),
                };
                output().json(&result)?;
            }
            return Err(e);
        }
    };

    let worktree_path = get_worktree_path(&git_root, &args.name);

    // Output result
    if args.json {
        let json_result = CreateResult {
            success: true,
            name: args.name.clone(),
            branch: branch_name.clone(),
            path: worktree_path.to_string_lossy().to_string(),
            copied_files: result.copied_files.clone(),
            error: None,
        };
        output().json(&json_result)?;
    } else {
        output()
            .success(&format!("Created worktree '{}' with branch '{}'", args.name, branch_name));
        if let Some(copied) = &result.copied_files {
            if !copied.is_empty() {
                output().log(&format!("Copied {} files", copied.len()));
            }
        }
    }

    // Handle post-creation actions
    if args.tmux
        || args.tmux_vertical
        || args.tmux_v
        || args.tmux_horizontal
        || args.tmux_h
        || args.kitty
        || args.kitty_vertical
        || args.kitty_v
        || args.kitty_horizontal
        || args.kitty_h
    {
        // Determine split direction
        let direction = if args.tmux_vertical || args.tmux_v || args.kitty_vertical || args.kitty_v
        {
            SplitDirection::Vertical
        } else if args.tmux_horizontal || args.tmux_h || args.kitty_horizontal || args.kitty_h {
            SplitDirection::Horizontal
        } else {
            SplitDirection::New
        };

        // Get shell command
        let shell_cmd = std::env::var("SHELL").unwrap_or_else(|_| "/bin/sh".to_string());

        let options = MultiplexerOptions {
            direction,
            command: shell_cmd,
            args: None,
            cwd: Some(worktree_path.to_string_lossy().to_string()),
            env: None,
            window_name: Some(args.name.clone()),
        };

        execute_in_multiplexer(options).await?;
    } else if args.shell {
        // Open shell in the new worktree
        shell_in_dir(&worktree_path).await?;
    } else if let Some(exec_cmd) = args.exec {
        // Execute command in the new worktree
        exec_in_dir(&worktree_path, &exec_cmd, &[]).await?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::executors::MockCommandExecutor;
    use std::sync::Arc;
    use tempfile::TempDir;

    // Note: These tests demonstrate the challenge of testing handlers that do filesystem operations.
    // The create handler currently mixes command execution with filesystem operations,
    // making it difficult to test with mocks alone. A future refactoring should separate
    // these concerns to enable better testing.

    #[tokio::test]
    async fn test_create_not_in_git_repo() {
        let mut mock = MockCommandExecutor::new();
        
        // Expect git root check to fail
        mock.expect_command("git")
            .with_args(&["rev-parse", "--git-common-dir"])
            .returns_output("", "fatal: not a git repository", 128);
        
        let context = HandlerContext::new(Arc::new(mock));
        let args = CreateArgs {
            name: "test".to_string(),
            branch: None,
            base: None,
            shell: false,
            exec: None,
            copy_files: None,
            json: false,
            tmux: false,
            tmux_vertical: false,
            tmux_v: false,
            tmux_horizontal: false,
            tmux_h: false,
            kitty: false,
            kitty_vertical: false,
            kitty_v: false,
            kitty_horizontal: false,
            kitty_h: false,
        };
        
        let result = handle(args, context).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_create_json_error_output() {
        let mut mock = MockCommandExecutor::new();
        
        // Expect git root check to fail
        mock.expect_command("git")
            .with_args(&["rev-parse", "--git-common-dir"])
            .returns_output("", "fatal: not a git repository", 128);
        
        let context = HandlerContext::new(Arc::new(mock));
        let args = CreateArgs {
            name: "test".to_string(),
            branch: None,
            base: None,
            shell: false,
            exec: None,
            copy_files: None,
            json: true, // JSON output mode
            tmux: false,
            tmux_vertical: false,
            tmux_v: false,
            tmux_horizontal: false,
            tmux_h: false,
            kitty: false,
            kitty_vertical: false,
            kitty_v: false,
            kitty_horizontal: false,
            kitty_h: false,
        };
        
        let result = handle(args, context).await;
        assert!(result.is_err());
        // In JSON mode, errors should be output as JSON before returning
    }

    // Integration test that uses a real temp directory
    #[tokio::test]
    #[ignore = "Integration test - requires filesystem access"]
    async fn test_create_with_temp_directory() {
        let temp_dir = TempDir::new().unwrap();
        let repo_path = temp_dir.path().join(".git");
        std::fs::create_dir(&repo_path).unwrap();
        
        let mut mock = MockCommandExecutor::new();
        
        // Return the temp directory as git root
        mock.expect_command("git")
            .with_args(&["rev-parse", "--git-common-dir"])
            .returns_output(repo_path.to_str().unwrap(), "", 0);
        
        // Expect worktree list - empty initially
        mock.expect_command("git")
            .with_args(&["worktree", "list", "--porcelain"])
            .returns_output(&format!("worktree {}\nHEAD abc123\nbranch refs/heads/main\n\n", temp_dir.path().display()), "", 0);
        
        // Expect worktree add
        let expected_path = temp_dir.path().join("phantoms").join("feature-test");
        mock.expect_command("git")
            .with_args(&["worktree", "add", "-b", "feature-test", expected_path.to_str().unwrap()])
            .returns_success();
        
        let context = HandlerContext::new(Arc::new(mock));
        let args = CreateArgs {
            name: "feature-test".to_string(),
            branch: None,
            base: None,
            shell: false,
            exec: None,
            copy_files: None,
            json: false,
            tmux: false,
            tmux_vertical: false,
            tmux_v: false,
            tmux_horizontal: false,
            tmux_h: false,
            kitty: false,
            kitty_vertical: false,
            kitty_v: false,
            kitty_horizontal: false,
            kitty_h: false,
        };
        
        let result = handle(args, context).await;
        assert!(result.is_ok());
        
        // Verify the phantoms directory was created
        assert!(temp_dir.path().join("phantoms").exists());
    }
}
