use crate::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::kitty::{execute_kitty_command, is_inside_kitty, KittyOptions, KittySplitDirection};
use super::spawn::{spawn_process, SpawnConfig, SpawnSuccess};
use super::tmux::{execute_tmux_command, is_inside_tmux, TmuxOptions, TmuxSplitDirection};

/// Supported terminal multiplexers
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Multiplexer {
    Tmux,
    Kitty,
    None,
}

/// Unified split direction for all multiplexers
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SplitDirection {
    New,
    Vertical,
    Horizontal,
}

/// Options for multiplexer command execution
#[derive(Debug, Clone)]
pub struct MultiplexerOptions {
    pub direction: SplitDirection,
    pub command: String,
    pub args: Option<Vec<String>>,
    pub cwd: Option<String>,
    pub env: Option<HashMap<String, String>>,
    pub window_name: Option<String>,
}

/// Detect the current terminal multiplexer
pub async fn detect_multiplexer() -> Multiplexer {
    if is_inside_tmux().await {
        Multiplexer::Tmux
    } else if is_inside_kitty().await {
        Multiplexer::Kitty
    } else {
        Multiplexer::None
    }
}

/// Execute a command in the detected multiplexer
pub async fn execute_in_multiplexer(options: MultiplexerOptions) -> Result<SpawnSuccess> {
    let multiplexer = detect_multiplexer().await;
    
    match multiplexer {
        Multiplexer::Tmux => {
            let tmux_direction = match options.direction {
                SplitDirection::New => TmuxSplitDirection::New,
                SplitDirection::Vertical => TmuxSplitDirection::Vertical,
                SplitDirection::Horizontal => TmuxSplitDirection::Horizontal,
            };
            
            let tmux_options = TmuxOptions {
                direction: tmux_direction,
                command: options.command,
                args: options.args,
                cwd: options.cwd,
                env: options.env,
                window_name: options.window_name,
            };
            
            execute_tmux_command(tmux_options).await
        }
        Multiplexer::Kitty => {
            let kitty_direction = match options.direction {
                SplitDirection::New => KittySplitDirection::New,
                SplitDirection::Vertical => KittySplitDirection::Vertical,
                SplitDirection::Horizontal => KittySplitDirection::Horizontal,
            };
            
            let kitty_options = KittyOptions {
                direction: kitty_direction,
                command: options.command,
                args: options.args,
                cwd: options.cwd,
                env: options.env,
                window_title: options.window_name,
            };
            
            execute_kitty_command(kitty_options).await
        }
        Multiplexer::None => {
            // Fallback: just spawn the process normally
            execute_fallback(options).await
        }
    }
}

/// Fallback for when no multiplexer is detected
async fn execute_fallback(options: MultiplexerOptions) -> Result<SpawnSuccess> {
    // For fallback, we can only execute the command in the current terminal
    // New window/split functionality won't work
    if options.direction != SplitDirection::New {
        tracing::warn!(
            "No terminal multiplexer detected. Cannot create splits. Running command in current terminal."
        );
    }
    
    let config = SpawnConfig {
        command: options.command,
        args: options.args.unwrap_or_default(),
        cwd: options.cwd,
        env: options.env,
        inherit_stdio: true,
        timeout_ms: None,
    };
    spawn_process(config).await
}

/// Check if any supported multiplexer is available
pub async fn is_multiplexer_available() -> bool {
    detect_multiplexer().await != Multiplexer::None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_detect_multiplexer() {
        // This will depend on the test environment
        let multiplexer = detect_multiplexer().await;
        // We can't assert a specific value, but we can ensure it returns something
        matches!(multiplexer, Multiplexer::Tmux | Multiplexer::Kitty | Multiplexer::None);
    }

    #[tokio::test]
    async fn test_multiplexer_options_creation() {
        let options = MultiplexerOptions {
            direction: SplitDirection::Vertical,
            command: "ls".to_string(),
            args: Some(vec!["-la".to_string()]),
            cwd: Some("/tmp".to_string()),
            env: Some(HashMap::from([("TEST".to_string(), "value".to_string())])),
            window_name: Some("Test".to_string()),
        };

        assert_eq!(options.direction, SplitDirection::Vertical);
        assert_eq!(options.command, "ls");
        assert!(options.args.is_some());
    }

    #[test]
    fn test_split_direction_serialization() {
        use serde_json;

        let new = SplitDirection::New;
        let json = serde_json::to_string(&new).unwrap();
        assert_eq!(json, "\"new\"");

        let deserialized: SplitDirection = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, SplitDirection::New);
    }

    #[test]
    fn test_multiplexer_serialization() {
        use serde_json;

        let tmux = Multiplexer::Tmux;
        let json = serde_json::to_string(&tmux).unwrap();
        assert_eq!(json, "\"tmux\"");

        let deserialized: Multiplexer = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, Multiplexer::Tmux);
    }
}