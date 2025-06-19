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
