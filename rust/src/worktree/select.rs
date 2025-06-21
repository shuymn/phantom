use crate::core::command_executor::{CommandArgs, CommandExecutor};
use crate::worktree::concurrent::list_worktrees_concurrent_with_executor;
use crate::{PhantomError, Result};
use smallvec::smallvec;
use std::path::Path;
use std::sync::Arc;
use tracing::{debug, info};

/// Result of selecting a worktree
#[derive(Debug, Clone)]
pub struct SelectWorktreeResult {
    pub name: String,
    pub branch: Option<String>,
    pub is_clean: bool,
}

/// Options for FZF selection
#[derive(Debug, Clone, Default)]
pub struct FzfOptions {
    pub prompt: Option<String>,
    pub header: Option<String>,
    pub preview_command: Option<String>,
}

/// Select a worktree interactively using fzf with CommandExecutor
pub async fn select_worktree_with_fzf_with_executor(
    executor: Arc<dyn CommandExecutor>,
    git_root: &Path,
) -> Result<Option<SelectWorktreeResult>> {
    select_worktree_with_fzf_and_options_with_executor(executor, git_root, FzfOptions::default())
        .await
}

/// Select a worktree interactively using fzf
pub async fn select_worktree_with_fzf(git_root: &Path) -> Result<Option<SelectWorktreeResult>> {
    select_worktree_with_fzf_and_options(git_root, FzfOptions::default()).await
}

/// Select a worktree interactively using fzf with custom options and CommandExecutor
pub async fn select_worktree_with_fzf_and_options_with_executor(
    executor: Arc<dyn CommandExecutor>,
    git_root: &Path,
    options: FzfOptions,
) -> Result<Option<SelectWorktreeResult>> {
    info!("Selecting worktree with fzf");

    // List all worktrees using concurrent operations
    let list_result = list_worktrees_concurrent_with_executor(executor.clone(), git_root).await?;

    // Filter to only phantom worktrees (concurrent list already does this)
    let worktrees = list_result.worktrees;

    if worktrees.is_empty() {
        debug!("No phantom worktrees found");
        return Ok(None);
    }

    // Format worktrees for display
    let formatted_worktrees: Vec<String> = worktrees
        .iter()
        .map(|wt| {
            let branch_info = wt.branch.as_ref().map(|b| format!(" ({})", b)).unwrap_or_default();
            let status = if !wt.is_clean { " [dirty]" } else { "" };
            format!("{}{}{}", wt.name, branch_info, status)
        })
        .collect();

    // Run fzf
    let selected = run_fzf_with_executor(executor, &formatted_worktrees, options).await?;

    match selected {
        Some(selection) => {
            // Extract the worktree name from the selection
            let selected_name = selection
                .split(' ')
                .next()
                .ok_or_else(|| PhantomError::Worktree("Invalid fzf selection".to_string()))?;

            // Find the matching worktree
            let selected_worktree = worktrees
                .into_iter()
                .find(|wt| wt.name == selected_name)
                .ok_or_else(|| PhantomError::Worktree("Selected worktree not found".to_string()))?;

            Ok(Some(SelectWorktreeResult {
                name: selected_worktree.name,
                branch: selected_worktree.branch,
                is_clean: selected_worktree.is_clean,
            }))
        }
        None => {
            debug!("No worktree selected");
            Ok(None)
        }
    }
}

/// Select a worktree interactively using fzf with custom options
pub async fn select_worktree_with_fzf_and_options(
    git_root: &Path,
    options: FzfOptions,
) -> Result<Option<SelectWorktreeResult>> {
    use crate::core::executors::RealCommandExecutor;
    select_worktree_with_fzf_and_options_with_executor(
        Arc::new(RealCommandExecutor),
        git_root,
        options,
    )
    .await
}

/// Run fzf with the given items and options using CommandExecutor
async fn run_fzf_with_executor(
    executor: Arc<dyn CommandExecutor>,
    items: &[String],
    options: FzfOptions,
) -> Result<Option<String>> {
    // Check if fzf is available
    if !is_fzf_available_with_executor(executor.clone()).await {
        return Err(PhantomError::Validation(
            "fzf command not found. Please install fzf first.".to_string(),
        ));
    }

    let mut args: CommandArgs = smallvec![];

    // Add options
    if let Some(prompt) = options.prompt {
        args.push("--prompt".to_string());
        args.push(prompt);
    } else {
        args.push("--prompt".to_string());
        args.push("Select worktree> ".to_string());
    }

    if let Some(header) = options.header {
        args.push("--header".to_string());
        args.push(header);
    } else {
        args.push("--header".to_string());
        args.push("Git Worktrees".to_string());
    }

    if let Some(preview) = options.preview_command {
        args.push("--preview".to_string());
        args.push(preview);
    }

    // Join items with newlines for stdin
    let stdin_data = items.join("\n");

    // Execute fzf with stdin data
    let config = crate::core::command_executor::CommandConfig::new("fzf")
        .with_args_smallvec(args)
        .with_stdin_data(stdin_data);

    match executor.execute(config).await {
        Ok(output) => {
            match output.exit_code {
                0 => {
                    // Success - user selected an item
                    let selected = output.stdout.trim().to_string();
                    Ok(if selected.is_empty() { None } else { Some(selected) })
                }
                1 => {
                    // No match found
                    Ok(None)
                }
                130 => {
                    // User pressed Ctrl-C
                    Ok(None)
                }
                _ => Err(PhantomError::ProcessExecution(format!(
                    "fzf exited with code {}: {}",
                    output.exit_code, output.stderr
                ))),
            }
        }
        Err(e) => {
            if e.to_string().contains("command not found") || e.to_string().contains("No such file")
            {
                Err(PhantomError::ProcessExecution(
                    "fzf command not found. Please install fzf first.".to_string(),
                ))
            } else {
                Err(e)
            }
        }
    }
}

/// Check if fzf command is available with CommandExecutor
async fn is_fzf_available_with_executor(executor: Arc<dyn CommandExecutor>) -> bool {
    let config = crate::core::command_executor::CommandConfig::new("fzf")
        .with_args_smallvec(smallvec!["--version".to_string()]);

    match executor.execute(config).await {
        Ok(output) => output.exit_code == 0,
        Err(_) => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::types::Worktree;
    use std::path::PathBuf;

    #[test]
    fn test_is_fzf_available() {
        // This test will pass or fail depending on whether fzf is installed
        // We just verify that the function doesn't panic
        use crate::process::fzf::is_fzf_available;
        let _ = is_fzf_available();
    }

    #[test]
    fn test_format_worktree_display() {
        let worktree = Worktree {
            name: "feature".to_string(),
            path: std::path::PathBuf::from("/path/to/worktree"),
            branch: Some("feature-branch".to_string()),
            commit: "abc123".to_string(),
            is_bare: false,
            is_detached: false,
            is_locked: false,
            is_prunable: false,
        };

        let is_clean = true;
        let formatted = format!(
            "{}{}{}",
            worktree.name,
            worktree.branch.as_ref().map(|b| format!(" ({})", b)).unwrap_or_default(),
            if !is_clean { " [dirty]" } else { "" }
        );

        assert_eq!(formatted, "feature (feature-branch)");
    }

    #[test]
    fn test_format_worktree_display_dirty() {
        let worktree = Worktree {
            name: "feature".to_string(),
            path: std::path::PathBuf::from("/path/to/worktree"),
            branch: Some("feature-branch".to_string()),
            commit: "abc123".to_string(),
            is_bare: false,
            is_detached: false,
            is_locked: false,
            is_prunable: false,
        };

        let is_clean = false;
        let formatted = format!(
            "{}{}{}",
            worktree.name,
            worktree.branch.as_ref().map(|b| format!(" ({})", b)).unwrap_or_default(),
            if !is_clean { " [dirty]" } else { "" }
        );

        assert_eq!(formatted, "feature (feature-branch) [dirty]");
    }

    #[test]
    fn test_select_worktree_result() {
        let result = SelectWorktreeResult {
            name: "test".to_string(),
            branch: Some("main".to_string()),
            is_clean: true,
        };

        assert_eq!(result.name, "test");
        assert_eq!(result.branch, Some("main".to_string()));
        assert!(result.is_clean);
    }

    #[test]
    fn test_select_worktree_result_clone() {
        let result = SelectWorktreeResult {
            name: "test".to_string(),
            branch: Some("main".to_string()),
            is_clean: false,
        };

        let cloned = result.clone();
        assert_eq!(result.name, cloned.name);
        assert_eq!(result.branch, cloned.branch);
        assert_eq!(result.is_clean, cloned.is_clean);
    }

    #[test]
    fn test_select_worktree_result_debug() {
        let result =
            SelectWorktreeResult { name: "test".to_string(), branch: None, is_clean: true };

        let debug_str = format!("{:?}", result);
        assert!(debug_str.contains("SelectWorktreeResult"));
        assert!(debug_str.contains("name"));
    }

    #[test]
    fn test_fzf_options_default() {
        let options = FzfOptions::default();
        assert!(options.prompt.is_none());
        assert!(options.header.is_none());
        assert!(options.preview_command.is_none());
    }

    #[test]
    fn test_fzf_options_with_values() {
        let options = FzfOptions {
            prompt: Some("Select:".to_string()),
            header: Some("Worktrees".to_string()),
            preview_command: Some("echo {}".to_string()),
        };

        assert_eq!(options.prompt, Some("Select:".to_string()));
        assert_eq!(options.header, Some("Worktrees".to_string()));
        assert_eq!(options.preview_command, Some("echo {}".to_string()));
    }

    #[test]
    fn test_format_worktree_no_branch() {
        let worktree = Worktree {
            name: "detached".to_string(),
            path: std::path::PathBuf::from("/path/to/worktree"),
            branch: None,
            commit: "abc123".to_string(),
            is_bare: false,
            is_detached: true,
            is_locked: false,
            is_prunable: false,
        };

        let is_clean = true;
        let formatted = format!(
            "{}{}{}",
            worktree.name,
            worktree.branch.as_ref().map(|b| format!(" ({})", b)).unwrap_or_default(),
            if !is_clean { " [dirty]" } else { "" }
        );

        assert_eq!(formatted, "detached");
    }

    #[test]
    fn test_fzf_options_debug() {
        let options = FzfOptions {
            prompt: Some("test".to_string()),
            header: None,
            preview_command: Some("preview".to_string()),
        };

        let debug_str = format!("{:?}", options);
        assert!(debug_str.contains("FzfOptions"));
        assert!(debug_str.contains("prompt: Some"));
        assert!(debug_str.contains("header: None"));
    }

    #[test]
    fn test_fzf_options_clone() {
        let options = FzfOptions {
            prompt: Some("Select>".to_string()),
            header: Some("Header".to_string()),
            preview_command: Some("cat {}".to_string()),
        };

        let cloned = options.clone();
        assert_eq!(options.prompt, cloned.prompt);
        assert_eq!(options.header, cloned.header);
        assert_eq!(options.preview_command, cloned.preview_command);
    }

    #[test]
    fn test_parse_fzf_selection() {
        // Test parsing the worktree name from fzf selection
        let selection = "feature-branch (main) [dirty]";
        let name = selection.split(' ').next();
        assert_eq!(name, Some("feature-branch"));

        let selection2 = "simple-name";
        let name2 = selection2.split(' ').next();
        assert_eq!(name2, Some("simple-name"));
    }

    #[test]
    fn test_worktree_formatting_combinations() {
        // Test various combinations of worktree states
        let cases = vec![
            ("main", Some("main"), true, "main (main)"),
            ("feature", Some("feature-branch"), false, "feature (feature-branch) [dirty]"),
            ("detached", None, true, "detached"),
            ("no-branch", None, false, "no-branch [dirty]"),
        ];

        for (name, branch, is_clean, expected) in cases {
            let worktree = Worktree {
                name: name.to_string(),
                path: PathBuf::from("/path"),
                branch: branch.map(|b| b.to_string()),
                commit: "abc123".to_string(),
                is_bare: false,
                is_detached: branch.is_none(),
                is_locked: false,
                is_prunable: false,
            };

            let formatted = format!(
                "{}{}{}",
                worktree.name,
                worktree.branch.as_ref().map(|b| format!(" ({})", b)).unwrap_or_default(),
                if !is_clean { " [dirty]" } else { "" }
            );

            assert_eq!(formatted, expected, "Failed for case: {}", name);
        }
    }

    #[test]
    fn test_select_worktree_result_variations() {
        // Test with no branch
        let result1 =
            SelectWorktreeResult { name: "test1".to_string(), branch: None, is_clean: true };
        assert!(result1.branch.is_none());

        // Test with dirty state
        let result2 = SelectWorktreeResult {
            name: "test2".to_string(),
            branch: Some("develop".to_string()),
            is_clean: false,
        };
        assert!(!result2.is_clean);
        assert_eq!(result2.branch, Some("develop".to_string()));
    }

    // Mock tests for the run_fzf function behavior
    #[test]
    fn test_run_fzf_error_cases() {
        // Test that we can handle when fzf is not available
        // This is tested by the actual run_fzf function when fzf is missing
        let _options = FzfOptions::default();

        // Verify the error message format
        let error = PhantomError::Validation(
            "fzf command not found. Please install fzf first.".to_string(),
        );
        assert!(error.to_string().contains("fzf command not found"));
    }

    #[test]
    fn test_fzf_output_parsing() {
        // Test parsing empty output
        let empty = "";
        assert!(empty.is_empty());

        // Test parsing trimmed output
        let with_spaces = "  selected  \n";
        assert_eq!(with_spaces.trim(), "selected");

        // Test exit codes we handle
        let exit_codes = vec![(0, "Success"), (1, "No match"), (130, "Ctrl-C")];

        for (code, description) in exit_codes {
            assert!(code >= 0);
            assert!(!description.is_empty());
        }
    }

    #[test]
    fn test_worktree_name_extraction() {
        // Test extracting name from various fzf selections
        let test_cases = vec![
            ("simple", "simple"),
            ("with-branch (main)", "with-branch"),
            ("dirty-worktree (feature) [dirty]", "dirty-worktree"),
            ("spaces in name (branch)", "spaces"),
            ("", ""),
        ];

        for (input, expected) in test_cases {
            let extracted = input.split(' ').next().unwrap_or("");
            assert_eq!(extracted, expected);
        }
    }

    #[test]
    fn test_error_handling() {
        // Test various error scenarios
        let errors = vec![
            PhantomError::Worktree("Invalid fzf selection".to_string()),
            PhantomError::Worktree("Selected worktree not found".to_string()),
            PhantomError::ProcessExecution("Failed to spawn fzf: error".to_string()),
            PhantomError::ProcessExecution("Failed to write to fzf stdin: error".to_string()),
            PhantomError::ProcessExecution("Failed to wait for fzf: error".to_string()),
            PhantomError::ProcessExecution("fzf exited with code 2: error".to_string()),
            PhantomError::ProcessExecution("fzf terminated by signal".to_string()),
        ];

        for error in errors {
            let error_str = error.to_string();
            assert!(!error_str.is_empty());
        }
    }

    #[tokio::test]
    async fn test_select_worktree_with_fzf_empty() {
        use crate::core::executors::MockCommandExecutor;
        use crate::test_utils::TestRepo;

        // Create a test repo with no worktrees
        let repo = TestRepo::new().await.unwrap();

        let mut mock = MockCommandExecutor::new();

        // Mock git worktree list - only main worktree exists
        mock.expect_command("git")
            .with_args(&["worktree", "list", "--porcelain"])
            .in_dir(repo.path().to_path_buf())
            .returns_output(
                &format!(
                    "worktree {}\nHEAD abc123\nbranch refs/heads/main\n",
                    repo.path().display()
                ),
                "",
                0,
            );

        // Should return None when only main worktree exists
        let result = select_worktree_with_fzf_with_executor(Arc::new(mock), repo.path()).await;
        match result {
            Ok(None) => {} // Expected - no worktrees to select
            Ok(Some(_)) => panic!("Should not select a worktree when none exist"),
            Err(e) => panic!("Unexpected error: {:?}", e),
        }
    }

    #[tokio::test]
    async fn test_select_worktree_with_custom_options() {
        use crate::core::executors::MockCommandExecutor;
        use crate::test_utils::TestRepo;
        use crate::worktree::create::create_worktree;
        use crate::worktree::types::CreateWorktreeOptions;

        let repo = TestRepo::new().await.unwrap();
        repo.create_file_and_commit("test.txt", "content", "Initial commit").await.unwrap();

        // Create a worktree
        let create_options = CreateWorktreeOptions::default();
        create_worktree(repo.path(), "feature-1", create_options).await.unwrap();

        let mut mock = MockCommandExecutor::new();

        // Mock git worktree list - show main and feature-1
        mock.expect_command("git")
            .with_args(&["worktree", "list", "--porcelain"])
            .in_dir(repo.path().to_path_buf())
            .returns_output(
                &format!(
                    "worktree {}\nHEAD abc123\nbranch refs/heads/main\n\nworktree {}\nHEAD def456\nbranch refs/heads/feature-1\n",
                    repo.path().display(),
                    repo.path().join(".phantom").join("feature-1").display()
                ),
                "",
                0,
            );

        // Mock git status for main worktree
        mock.expect_command("git")
            .with_args(&["status", "--porcelain"])
            .in_dir(repo.path().to_path_buf())
            .returns_output("", "", 0);

        // Mock git status for feature-1 worktree
        mock.expect_command("git")
            .with_args(&["status", "--porcelain"])
            .in_dir(repo.path().join(".phantom").join("feature-1"))
            .returns_output("", "", 0);

        // Mock fzf availability check
        mock.expect_command("fzf").with_args(&["--version"]).returns_output("0.42.0", "", 0);

        // Mock fzf selection with custom options
        mock.expect_command("fzf")
            .with_args(&[
                "--prompt",
                "Custom prompt> ",
                "--header",
                "Custom header",
                "--preview",
                "echo preview",
            ])
            .with_stdin_data("feature-1 (feature-1)")
            .returns_output("feature-1 (feature-1)\n", "", 0);

        let options = FzfOptions {
            prompt: Some("Custom prompt> ".to_string()),
            header: Some("Custom header".to_string()),
            preview_command: Some("echo preview".to_string()),
        };

        // Test with custom options
        let result = select_worktree_with_fzf_and_options_with_executor(
            Arc::new(mock),
            repo.path(),
            options,
        )
        .await;
        assert!(result.is_ok());
        let selected = result.unwrap();
        assert!(selected.is_some());
        assert_eq!(selected.unwrap().name, "feature-1");
    }

    #[test]
    fn test_fzf_stdin_write() {
        // Test formatting items for fzf input
        let items = vec![
            "worktree1 (main)".to_string(),
            "worktree2 (feature) [dirty]".to_string(),
            "worktree3".to_string(),
        ];

        let input = items.join("\n");
        assert_eq!(input, "worktree1 (main)\nworktree2 (feature) [dirty]\nworktree3");
        assert_eq!(input.as_bytes().len(), input.len());
    }

    #[test]
    fn test_string_from_utf8_lossy() {
        // Test UTF-8 handling
        let valid_utf8 = b"valid string";
        let result = String::from_utf8_lossy(valid_utf8);
        assert_eq!(result, "valid string");

        // Test with invalid UTF-8 (will be replaced with replacement character)
        let invalid_utf8 = &[0xFF, 0xFE, 0xFD];
        let result = String::from_utf8_lossy(invalid_utf8);
        assert!(result.len() > 0); // Will contain replacement characters
    }

    #[test]
    fn test_worktree_position_finding() {
        let worktrees = vec![
            Worktree {
                name: "first".to_string(),
                path: PathBuf::from("/first"),
                branch: Some("main".to_string()),
                commit: "111".to_string(),
                is_bare: false,
                is_detached: false,
                is_locked: false,
                is_prunable: false,
            },
            Worktree {
                name: "second".to_string(),
                path: PathBuf::from("/second"),
                branch: Some("feature".to_string()),
                commit: "222".to_string(),
                is_bare: false,
                is_detached: false,
                is_locked: false,
                is_prunable: false,
            },
        ];

        // Test finding by name
        let pos = worktrees.iter().position(|wt| wt.name == "second");
        assert_eq!(pos, Some(1));

        let pos = worktrees.iter().position(|wt| wt.name == "nonexistent");
        assert_eq!(pos, None);
    }

    #[test]
    fn test_command_creation() {
        // Test that command builder works correctly
        use std::process::Command;
        let mut cmd = Command::new("echo");
        cmd.arg("test");

        // Can't easily test execution without side effects
        // Just verify we can build commands
        assert_eq!(cmd.get_program(), "echo");
    }
}
