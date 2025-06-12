use crate::{PhantomError, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use std::path::Path;

use super::spawn::{spawn_process, SpawnConfig, SpawnSuccess, execute_command};

/// Direction for tmux split operations
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TmuxSplitDirection {
    New,
    Vertical,
    Horizontal,
}

/// Options for tmux command execution
#[derive(Debug, Clone)]
pub struct TmuxOptions {
    pub direction: TmuxSplitDirection,
    pub command: String,
    pub args: Option<Vec<String>>,
    pub cwd: Option<String>,
    pub env: Option<HashMap<String, String>>,
    pub window_name: Option<String>,
}

/// Success result for tmux operations
pub type TmuxSuccess = SpawnSuccess;

/// Check if we're running inside a tmux session
pub async fn is_inside_tmux() -> bool {
    env::var("TMUX").is_ok()
}

/// Execute a command in tmux
pub async fn execute_tmux_command(options: TmuxOptions) -> Result<TmuxSuccess> {
    let mut tmux_args = Vec::new();

    // Set up the tmux command based on direction
    match options.direction {
        TmuxSplitDirection::New => {
            tmux_args.push("new-window".to_string());
            if let Some(window_name) = &options.window_name {
                tmux_args.push("-n".to_string());
                tmux_args.push(window_name.clone());
            }
        }
        TmuxSplitDirection::Vertical => {
            tmux_args.push("split-window".to_string());
            tmux_args.push("-v".to_string());
        }
        TmuxSplitDirection::Horizontal => {
            tmux_args.push("split-window".to_string());
            tmux_args.push("-h".to_string());
        }
    }

    // Add working directory if specified
    if let Some(cwd) = &options.cwd {
        tmux_args.push("-c".to_string());
        tmux_args.push(cwd.clone());
    }

    // Add environment variables
    if let Some(env_vars) = &options.env {
        for (key, value) in env_vars {
            tmux_args.push("-e".to_string());
            tmux_args.push(format!("{}={}", key, value));
        }
    }

    // Add the command
    tmux_args.push(options.command.clone());

    // Add command arguments
    if let Some(args) = &options.args {
        tmux_args.extend(args.clone());
    }

    // Execute the tmux command
    let config = SpawnConfig {
        command: "tmux".to_string(),
        args: tmux_args,
        cwd: None,
        env: None,
        inherit_stdio: true,
        timeout_ms: None,
    };
    spawn_process(config).await
}

/// Create a new tmux session
pub async fn create_tmux_session(session_name: &str, cwd: Option<&Path>) -> Result<TmuxSuccess> {
    let mut args = vec!["new-session".to_string(), "-d".to_string(), "-s".to_string(), session_name.to_string()];

    if let Some(cwd) = cwd {
        args.push("-c".to_string());
        args.push(cwd.to_string_lossy().to_string());
    }

    let config = SpawnConfig {
        command: "tmux".to_string(),
        args,
        cwd: None,
        env: None,
        inherit_stdio: true,
        timeout_ms: None,
    };
    spawn_process(config).await
}

/// Attach to a tmux session
pub async fn attach_tmux_session(session_name: &str) -> Result<TmuxSuccess> {
    let args = vec!["attach-session".to_string(), "-t".to_string(), session_name.to_string()];
    let config = SpawnConfig {
        command: "tmux".to_string(),
        args,
        cwd: None,
        env: None,
        inherit_stdio: true,
        timeout_ms: None,
    };
    spawn_process(config).await
}

/// List tmux sessions
pub async fn list_tmux_sessions() -> Result<Vec<String>> {
    let args: Vec<&str> = vec!["list-sessions", "-F", "#{session_name}"];
    let output = execute_command(
        "tmux",
        args,
        None,
    )
    .await?;

    let sessions = output
        .lines()
        .map(|s| s.to_string())
        .filter(|s| !s.is_empty())
        .collect();

    Ok(sessions)
}

/// Check if a tmux session exists
pub async fn tmux_session_exists(session_name: &str) -> Result<bool> {
    let config = SpawnConfig {
        command: "tmux".to_string(),
        args: vec!["has-session".to_string(), "-t".to_string(), session_name.to_string()],
        cwd: None,
        env: None,
        inherit_stdio: false,
        timeout_ms: None,
    };
    
    match spawn_process(config).await {
        Ok(_) => Ok(true),
        Err(PhantomError::ProcessExecution(_)) => Ok(false),
        Err(e) => Err(e),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_is_inside_tmux() {
        // This will be false in most test environments
        let inside = is_inside_tmux().await;
        // We can't assert a specific value as it depends on the environment
        assert!(inside == env::var("TMUX").is_ok());
    }

    #[tokio::test]
    async fn test_tmux_options_creation() {
        let options = TmuxOptions {
            direction: TmuxSplitDirection::Vertical,
            command: "echo".to_string(),
            args: Some(vec!["hello".to_string()]),
            cwd: Some("/tmp".to_string()),
            env: Some(HashMap::from([("TEST".to_string(), "value".to_string())])),
            window_name: None,
        };

        assert_eq!(options.direction, TmuxSplitDirection::Vertical);
        assert_eq!(options.command, "echo");
        assert_eq!(options.args.unwrap()[0], "hello");
    }

    #[test]
    fn test_tmux_split_direction_serialization() {
        use serde_json;

        let vertical = TmuxSplitDirection::Vertical;
        let json = serde_json::to_string(&vertical).unwrap();
        assert_eq!(json, "\"vertical\"");

        let deserialized: TmuxSplitDirection = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, TmuxSplitDirection::Vertical);
    }
}