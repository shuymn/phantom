use crate::{PhantomError, Result};
use std::io::Write;
use std::process::{Command, Stdio};
use tracing::{debug, error};

/// Options for FZF selection
#[derive(Debug, Clone, Default)]
pub struct FzfOptions {
    pub prompt: Option<String>,
    pub header: Option<String>,
    pub preview_command: Option<String>,
}

/// Select an item from a list using fzf
pub async fn select_with_fzf(items: Vec<String>, options: FzfOptions) -> Result<Option<String>> {
    debug!("Starting fzf selection with {} items", items.len());

    if items.is_empty() {
        debug!("No items to select from");
        return Ok(None);
    }

    let mut cmd = Command::new("fzf");

    // Add options
    if let Some(prompt) = &options.prompt {
        cmd.arg("--prompt").arg(prompt);
    }

    if let Some(header) = &options.header {
        cmd.arg("--header").arg(header);
    }

    if let Some(preview_command) = &options.preview_command {
        cmd.arg("--preview").arg(preview_command);
    }

    // Configure stdio
    cmd.stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::piped());

    // Spawn the process
    let mut child = cmd.spawn().map_err(|e| {
        if e.kind() == std::io::ErrorKind::NotFound {
            PhantomError::ProcessExecution(
                "fzf command not found. Please install fzf first.".to_string(),
            )
        } else {
            PhantomError::Io(e)
        }
    })?;

    // Write items to stdin
    if let Some(mut stdin) = child.stdin.take() {
        let input = items.join("\n");
        stdin.write_all(input.as_bytes()).map_err(|e| {
            error!("Failed to write to fzf stdin: {}", e);
            PhantomError::Io(e)
        })?;
    }

    // Wait for the process to complete
    let output = child.wait_with_output().map_err(|e| {
        error!("Failed to wait for fzf: {}", e);
        PhantomError::Io(e)
    })?;

    // Check exit status
    match output.status.code() {
        Some(0) => {
            // Success - user selected an item
            let selected = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if selected.is_empty() {
                Ok(None)
            } else {
                debug!("User selected: {}", selected);
                Ok(Some(selected))
            }
        }
        Some(1) => {
            // No match found
            debug!("No match found in fzf");
            Ok(None)
        }
        Some(130) => {
            // User pressed Ctrl+C
            debug!("User cancelled fzf selection");
            Ok(None)
        }
        Some(code) => {
            // Other error
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(PhantomError::ProcessExecution(format!(
                "fzf exited with code {}: {}",
                code, stderr
            )))
        }
        None => {
            // Process terminated by signal
            Err(PhantomError::ProcessExecution("fzf terminated by signal".to_string()))
        }
    }
}

/// Check if fzf is available in the system
pub fn is_fzf_available() -> bool {
    Command::new("fzf")
        .arg("--version")
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_fzf_available() {
        // This test will pass or fail depending on whether fzf is installed
        let available = is_fzf_available();
        // We can't assert a specific value as it depends on the environment
        if available {
            println!("fzf is available");
        } else {
            println!("fzf is not available");
        }
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
            prompt: Some("Select an item: ".to_string()),
            header: Some("Available items".to_string()),
            preview_command: Some("echo {}".to_string()),
        };

        assert_eq!(options.prompt.unwrap(), "Select an item: ");
        assert_eq!(options.header.unwrap(), "Available items");
        assert_eq!(options.preview_command.unwrap(), "echo {}");
    }
}
