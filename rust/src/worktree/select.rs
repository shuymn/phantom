use crate::git::libs::list_worktrees::list_worktrees;
use crate::worktree::delete::get_worktree_status;
use crate::{PhantomError, Result};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
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

/// Select a worktree interactively using fzf
pub async fn select_worktree_with_fzf(git_root: &Path) -> Result<Option<SelectWorktreeResult>> {
    select_worktree_with_fzf_and_options(git_root, FzfOptions::default()).await
}

/// Select a worktree interactively using fzf with custom options
pub async fn select_worktree_with_fzf_and_options(
    git_root: &Path,
    options: FzfOptions,
) -> Result<Option<SelectWorktreeResult>> {
    info!("Selecting worktree with fzf");

    // List all worktrees
    let worktrees = list_worktrees(git_root).await?;

    if worktrees.is_empty() {
        debug!("No worktrees found");
        return Ok(None);
    }

    // Get clean status for each worktree
    let mut worktree_statuses = Vec::new();
    for worktree in &worktrees {
        let path = PathBuf::from(&worktree.path);
        let status = get_worktree_status(&path).await;
        worktree_statuses.push(!status.has_uncommitted_changes);
    }

    // Format worktrees for display
    let formatted_worktrees: Vec<String> = worktrees
        .iter()
        .zip(worktree_statuses.iter())
        .map(|(wt, is_clean)| {
            let branch_info = wt.branch.as_ref().map(|b| format!(" ({})", b)).unwrap_or_default();
            let status = if !is_clean { " [dirty]" } else { "" };
            format!("{}{}{}", wt.name, branch_info, status)
        })
        .collect();

    // Run fzf
    let selected = run_fzf(&formatted_worktrees, options)?;

    match selected {
        Some(selection) => {
            // Extract the worktree name from the selection
            let selected_name = selection
                .split(' ')
                .next()
                .ok_or_else(|| PhantomError::Worktree("Invalid fzf selection".to_string()))?;

            // Find the matching worktree and its clean status
            let position = worktrees
                .iter()
                .position(|wt| wt.name == selected_name)
                .ok_or_else(|| PhantomError::Worktree("Selected worktree not found".to_string()))?;

            let selected_worktree = worktrees.into_iter().nth(position).unwrap();
            let is_clean = worktree_statuses[position];

            Ok(Some(SelectWorktreeResult {
                name: selected_worktree.name,
                branch: selected_worktree.branch,
                is_clean,
            }))
        }
        None => {
            debug!("No worktree selected");
            Ok(None)
        }
    }
}

/// Run fzf with the given items and options
fn run_fzf(items: &[String], options: FzfOptions) -> Result<Option<String>> {
    // Check if fzf is available
    if !is_fzf_available() {
        return Err(PhantomError::Validation(
            "fzf command not found. Please install fzf first.".to_string(),
        ));
    }

    let mut cmd = Command::new("fzf");

    // Add options
    if let Some(prompt) = options.prompt {
        cmd.args(["--prompt", &prompt]);
    } else {
        cmd.args(["--prompt", "Select worktree> "]);
    }

    if let Some(header) = options.header {
        cmd.args(["--header", &header]);
    } else {
        cmd.args(["--header", "Git Worktrees"]);
    }

    if let Some(preview) = options.preview_command {
        cmd.args(["--preview", &preview]);
    }

    // Set up pipes
    cmd.stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::piped());

    let mut child = cmd
        .spawn()
        .map_err(|e| PhantomError::ProcessExecution(format!("Failed to spawn fzf: {}", e)))?;

    // Write items to fzf's stdin
    if let Some(mut stdin) = child.stdin.take() {
        let input = items.join("\n");
        stdin.write_all(input.as_bytes()).map_err(|e| {
            PhantomError::ProcessExecution(format!("Failed to write to fzf stdin: {}", e))
        })?;
    }

    // Wait for fzf to complete
    let output = child
        .wait_with_output()
        .map_err(|e| PhantomError::ProcessExecution(format!("Failed to wait for fzf: {}", e)))?;

    match output.status.code() {
        Some(0) => {
            // Success - user selected an item
            let selected = String::from_utf8_lossy(&output.stdout).trim().to_string();
            Ok(if selected.is_empty() { None } else { Some(selected) })
        }
        Some(1) => {
            // No match found
            Ok(None)
        }
        Some(130) => {
            // User pressed Ctrl-C
            Ok(None)
        }
        Some(code) => {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(PhantomError::ProcessExecution(format!(
                "fzf exited with code {}: {}",
                code, stderr
            )))
        }
        None => Err(PhantomError::ProcessExecution("fzf terminated by signal".to_string())),
    }
}

/// Check if fzf command is available
fn is_fzf_available() -> bool {
    Command::new("fzf")
        .arg("--version")
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::types::Worktree;

    #[test]
    fn test_is_fzf_available() {
        // This test will pass or fail depending on whether fzf is installed
        // We just verify that the function doesn't panic
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
        let result = SelectWorktreeResult {
            name: "test".to_string(),
            branch: None,
            is_clean: true,
        };

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
        let result1 = SelectWorktreeResult {
            name: "test1".to_string(),
            branch: None,
            is_clean: true,
        };
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
}
