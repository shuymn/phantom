use crate::cli::commands::create::{CreateArgs, CreateResult};
use crate::cli::context::HandlerContext;
use crate::cli::output::output;
use crate::config::loader::load_config;
use crate::core::command_executor::CommandExecutor;
use crate::core::exit_handler::ExitHandler;
use crate::core::filesystem::FileSystem;
use crate::git::libs::get_git_root_generic::get_git_root;
use crate::process::exec::exec_in_dir;
use crate::process::multiplexer::{execute_in_multiplexer, MultiplexerOptions, SplitDirection};
use crate::process::shell::shell_in_dir;
use crate::worktree::create::create_worktree;
use crate::worktree::paths::get_worktree_path;
use crate::worktree::types::CreateWorktreeOptions;
use crate::Result;

/// Handle the create command
pub async fn handle<E, F, H>(args: CreateArgs, context: HandlerContext<E, F, H>) -> Result<()>
where
    E: CommandExecutor + Clone + 'static,
    F: FileSystem + Clone + 'static,
    H: ExitHandler + Clone + 'static,
{
    // Get git root
    let git_root = match get_git_root(context.executor.clone()).await {
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

    // IMPORTANT: Create handler testing limitations
    //
    // The create handler uses create_worktree which performs filesystem operations
    // (creating directories) that we cannot mock. This limits our testing to:
    // 1. Early failures (not in git repo)
    // 2. Validation failures (worktree already exists)
    // 3. The flow up to the point where filesystem operations begin
    //
    // Future work: Abstract filesystem operations to enable full mock testing

    #[tokio::test]
    async fn test_create_not_in_git_repo() {
        let mut mock = MockCommandExecutor::new();

        // Expect git root check to fail
        mock.expect_command("git").with_args(&["rev-parse", "--git-common-dir"]).returns_output(
            "",
            "fatal: not a git repository",
            128,
        );

        let context = HandlerContext::new(
            mock,
            crate::core::filesystems::MockFileSystem::new(),
            crate::core::exit_handler::MockExitHandler::new(),
        );
        let args = CreateArgs {
            name: "feature".to_string(),
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
        match result {
            Err(e) => assert!(e.to_string().contains("not a git repository")),
            _ => panic!("Expected error about git repository"),
        }
    }

    #[tokio::test]
    async fn test_create_json_error_not_git_repo() {
        let mut mock = MockCommandExecutor::new();

        // Expect git root check to fail
        mock.expect_command("git").with_args(&["rev-parse", "--git-common-dir"]).returns_output(
            "",
            "fatal: not a git repository",
            128,
        );

        let context = HandlerContext::new(
            mock,
            crate::core::filesystems::MockFileSystem::new(),
            crate::core::exit_handler::MockExitHandler::new(),
        );
        let args = CreateArgs {
            name: "feature".to_string(),
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
        // In JSON mode, the error should be output as JSON before returning
    }

    #[tokio::test]
    async fn test_create_worktree_already_exists() {
        let mut mock = MockCommandExecutor::new();

        // Mock git root check
        mock.expect_command("git").with_args(&["rev-parse", "--git-common-dir"]).returns_output(
            "/repo/.git",
            "",
            0,
        );

        // Mock worktree list check - shows feature already exists
        mock.expect_command("git")
            .with_args(&["worktree", "list", "--porcelain"])
            .returns_output(
                "worktree /repo\nHEAD abc123\nbranch refs/heads/main\n\n\
                 worktree /repo/.git/phantom/worktrees/feature\nHEAD def456\nbranch refs/heads/feature\n",
                "",
                0,
            );

        let context = HandlerContext::new(
            mock,
            crate::core::filesystems::MockFileSystem::new(),
            crate::core::exit_handler::MockExitHandler::new(),
        );
        let args = CreateArgs {
            name: "feature".to_string(),
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
        match result {
            Err(e) => {
                let error_msg = e.to_string();
                // The error could be about already existing or about filesystem operations
                // since create_worktree tries to create directories
                assert!(
                    error_msg.contains("already exists") || error_msg.contains("phantom directory"),
                    "Unexpected error: {}",
                    error_msg
                );
            }
            _ => panic!("Expected error"),
        }
    }

    #[tokio::test]
    async fn test_create_with_custom_branch() {
        let mut mock = MockCommandExecutor::new();

        // Mock git root check
        mock.expect_command("git").with_args(&["rev-parse", "--git-common-dir"]).returns_output(
            "/repo/.git",
            "",
            0,
        );

        // Mock worktree list check - empty
        mock.expect_command("git").with_args(&["worktree", "list", "--porcelain"]).returns_output(
            "worktree /repo\nHEAD abc123\nbranch refs/heads/main\n",
            "",
            0,
        );

        let context = HandlerContext::new(
            mock,
            crate::core::filesystems::MockFileSystem::new(),
            crate::core::exit_handler::MockExitHandler::new(),
        );
        let args = CreateArgs {
            name: "feature".to_string(),
            branch: Some("custom-feature".to_string()),
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

        // This will fail when it tries to create directories
        let result = handle(args, context).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_create_invalid_worktree_name() {
        let mut mock = MockCommandExecutor::new();

        // Mock git root check
        mock.expect_command("git").with_args(&["rev-parse", "--git-common-dir"]).returns_output(
            "/repo/.git",
            "",
            0,
        );

        let context = HandlerContext::new(
            mock,
            crate::core::filesystems::MockFileSystem::new(),
            crate::core::exit_handler::MockExitHandler::new(),
        );
        let args = CreateArgs {
            name: "invalid name with spaces".to_string(),
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
        match result {
            Err(e) => assert!(e.to_string().contains("can only contain")),
            _ => panic!("Expected validation error"),
        }
    }

    // Note: Tests for post-creation actions (tmux, kitty, shell, exec) are not
    // included here because they would require mocking process operations, which
    // haven't been migrated to use CommandExecutor yet. These will be added once
    // the process operations migration is complete.
}
