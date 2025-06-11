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
}
