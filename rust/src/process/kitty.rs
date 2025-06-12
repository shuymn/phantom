use crate::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;

use super::spawn::{spawn_process, SpawnConfig, SpawnSuccess};

/// Direction for kitty split operations
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum KittySplitDirection {
    New,
    Vertical,
    Horizontal,
}

/// Options for kitty command execution
#[derive(Debug, Clone)]
pub struct KittyOptions {
    pub direction: KittySplitDirection,
    pub command: String,
    pub args: Option<Vec<String>>,
    pub cwd: Option<String>,
    pub env: Option<HashMap<String, String>>,
    pub window_title: Option<String>,
}

/// Success result for kitty operations
pub type KittySuccess = SpawnSuccess;

/// Check if we're running inside Kitty terminal
pub async fn is_inside_kitty() -> bool {
    env::var("TERM").map(|t| t == "xterm-kitty").unwrap_or(false)
        || env::var("KITTY_WINDOW_ID").is_ok()
}

/// Execute a command in kitty
pub async fn execute_kitty_command(options: KittyOptions) -> Result<KittySuccess> {
    let mut kitty_args = vec!["@".to_string(), "launch".to_string()];

    // Set up the kitty command based on direction
    match options.direction {
        KittySplitDirection::New => {
            kitty_args.push("--type=tab".to_string());
            if let Some(window_title) = &options.window_title {
                kitty_args.push(format!("--tab-title={}", window_title));
            }
        }
        KittySplitDirection::Vertical => {
            kitty_args.push("--location=vsplit".to_string());
        }
        KittySplitDirection::Horizontal => {
            kitty_args.push("--location=hsplit".to_string());
        }
    }

    // Add working directory if specified
    if let Some(cwd) = &options.cwd {
        kitty_args.push(format!("--cwd={}", cwd));
    }

    // Add environment variables
    if let Some(env_vars) = &options.env {
        for (key, value) in env_vars {
            kitty_args.push(format!("--env={}={}", key, value));
        }
    }

    // Add separator before command
    kitty_args.push("--".to_string());

    // Add the command
    kitty_args.push(options.command.clone());

    // Add command arguments
    if let Some(args) = &options.args {
        kitty_args.extend(args.clone());
    }

    // Execute the kitty command
    let config = SpawnConfig {
        command: "kitty".to_string(),
        args: kitty_args,
        cwd: None,
        env: None,
        inherit_stdio: true,
        timeout_ms: None,
    };
    spawn_process(config).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_is_inside_kitty() {
        // This will be false in most test environments
        let inside = is_inside_kitty().await;
        // We can't assert a specific value as it depends on the environment
        let expected = env::var("TERM").map(|t| t == "xterm-kitty").unwrap_or(false)
            || env::var("KITTY_WINDOW_ID").is_ok();
        assert_eq!(inside, expected);
    }

    #[tokio::test]
    async fn test_kitty_options_creation() {
        let options = KittyOptions {
            direction: KittySplitDirection::New,
            command: "echo".to_string(),
            args: Some(vec!["hello".to_string()]),
            cwd: Some("/tmp".to_string()),
            env: Some(HashMap::from([("TEST".to_string(), "value".to_string())])),
            window_title: Some("Test Window".to_string()),
        };

        assert_eq!(options.direction, KittySplitDirection::New);
        assert_eq!(options.command, "echo");
        assert_eq!(options.args.unwrap()[0], "hello");
        assert_eq!(options.window_title.unwrap(), "Test Window");
    }

    #[test]
    fn test_kitty_split_direction_serialization() {
        use serde_json;

        let horizontal = KittySplitDirection::Horizontal;
        let json = serde_json::to_string(&horizontal).unwrap();
        assert_eq!(json, "\"horizontal\"");

        let deserialized: KittySplitDirection = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, KittySplitDirection::Horizontal);
    }
}
