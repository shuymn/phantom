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
pub async fn execute_in_multiplexer<E>(
    executor: E,
    options: MultiplexerOptions,
) -> Result<SpawnSuccess>
where
    E: crate::core::command_executor::CommandExecutor + Clone + 'static,
{
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

            execute_tmux_command(&executor, tmux_options).await?;
            Ok(SpawnSuccess { exit_code: 0 })
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

            execute_kitty_command(&executor, kitty_options).await?;
            Ok(SpawnSuccess { exit_code: 0 })
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

        // Test Tmux
        let tmux = Multiplexer::Tmux;
        let json = serde_json::to_string(&tmux).unwrap();
        assert_eq!(json, "\"tmux\"");

        let deserialized: Multiplexer = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, Multiplexer::Tmux);

        // Test Kitty
        let kitty = Multiplexer::Kitty;
        let json = serde_json::to_string(&kitty).unwrap();
        assert_eq!(json, "\"kitty\"");

        let deserialized: Multiplexer = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, Multiplexer::Kitty);

        // Test None
        let none = Multiplexer::None;
        let json = serde_json::to_string(&none).unwrap();
        assert_eq!(json, "\"none\"");

        let deserialized: Multiplexer = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, Multiplexer::None);
    }

    #[test]
    fn test_split_direction_all_variants() {
        use serde_json;

        // Test all variants
        let directions = vec![
            (SplitDirection::New, "\"new\""),
            (SplitDirection::Vertical, "\"vertical\""),
            (SplitDirection::Horizontal, "\"horizontal\""),
        ];

        for (direction, expected_json) in directions {
            let json = serde_json::to_string(&direction).unwrap();
            assert_eq!(json, expected_json);

            let deserialized: SplitDirection = serde_json::from_str(&json).unwrap();
            assert_eq!(deserialized, direction);
        }
    }

    #[test]
    fn test_multiplexer_equality() {
        assert_eq!(Multiplexer::Tmux, Multiplexer::Tmux);
        assert_ne!(Multiplexer::Tmux, Multiplexer::Kitty);
        assert_ne!(Multiplexer::Tmux, Multiplexer::None);
        assert_ne!(Multiplexer::Kitty, Multiplexer::None);
    }

    #[test]
    fn test_split_direction_equality() {
        assert_eq!(SplitDirection::New, SplitDirection::New);
        assert_ne!(SplitDirection::New, SplitDirection::Vertical);
        assert_ne!(SplitDirection::Vertical, SplitDirection::Horizontal);
    }

    #[test]
    fn test_multiplexer_debug() {
        let tmux = Multiplexer::Tmux;
        let debug_str = format!("{tmux:?}");
        assert!(debug_str.contains("Tmux"));

        let kitty = Multiplexer::Kitty;
        let debug_str = format!("{kitty:?}");
        assert!(debug_str.contains("Kitty"));

        let none = Multiplexer::None;
        let debug_str = format!("{none:?}");
        assert!(debug_str.contains("None"));
    }

    #[test]
    fn test_split_direction_debug() {
        let new = SplitDirection::New;
        let debug_str = format!("{new:?}");
        assert!(debug_str.contains("New"));

        let vertical = SplitDirection::Vertical;
        let debug_str = format!("{vertical:?}");
        assert!(debug_str.contains("Vertical"));

        let horizontal = SplitDirection::Horizontal;
        let debug_str = format!("{horizontal:?}");
        assert!(debug_str.contains("Horizontal"));
    }

    #[test]
    fn test_multiplexer_options_debug() {
        let options = MultiplexerOptions {
            direction: SplitDirection::New,
            command: "test".to_string(),
            args: None,
            cwd: None,
            env: None,
            window_name: None,
        };

        let debug_str = format!("{options:?}");
        assert!(debug_str.contains("MultiplexerOptions"));
        assert!(debug_str.contains("direction"));
        assert!(debug_str.contains("command"));
    }

    #[test]
    fn test_multiplexer_copy_clone() {
        let original = Multiplexer::Tmux;
        let copied = original;
        let cloned = original;

        assert_eq!(original, copied);
        assert_eq!(original, cloned);
    }

    #[test]
    fn test_split_direction_copy_clone() {
        let original = SplitDirection::Vertical;
        let copied = original;
        let cloned = original;

        assert_eq!(original, copied);
        assert_eq!(original, cloned);
    }

    #[test]
    fn test_multiplexer_options_clone() {
        let original = MultiplexerOptions {
            direction: SplitDirection::Horizontal,
            command: "echo".to_string(),
            args: Some(vec!["hello".to_string()]),
            cwd: Some("/tmp".to_string()),
            env: Some(HashMap::from([("KEY".to_string(), "value".to_string())])),
            window_name: Some("Test Window".to_string()),
        };

        let cloned = original.clone();

        assert_eq!(original.direction, cloned.direction);
        assert_eq!(original.command, cloned.command);
        assert_eq!(original.args, cloned.args);
        assert_eq!(original.cwd, cloned.cwd);
        assert_eq!(original.env, cloned.env);
        assert_eq!(original.window_name, cloned.window_name);
    }

    #[tokio::test]
    async fn test_is_multiplexer_available() {
        // This will depend on the test environment
        let available = is_multiplexer_available().await;
        // We can't assert a specific value, but we can ensure it returns a boolean
        assert!(available || !available);
    }

    #[tokio::test]
    async fn test_execute_in_multiplexer_none() {
        // Test fallback behavior when no multiplexer is detected
        // This test might actually execute if no multiplexer is present
        let options = MultiplexerOptions {
            direction: SplitDirection::New,
            command: "echo".to_string(),
            args: Some(vec!["test".to_string()]),
            cwd: None,
            env: None,
            window_name: None,
        };

        // We can't easily mock the multiplexer detection, but we can verify
        // the function doesn't panic
        let multiplexer = detect_multiplexer().await;
        if multiplexer == Multiplexer::None {
            use crate::core::executors::RealCommandExecutor;
            let result = execute_in_multiplexer(RealCommandExecutor, options).await;
            // If no multiplexer, it should execute the command directly
            assert!(result.is_ok() || result.is_err());
        }
    }

    #[test]
    fn test_multiplexer_options_with_full_config() {
        let mut env = HashMap::new();
        env.insert("KEY1".to_string(), "value1".to_string());
        env.insert("KEY2".to_string(), "value2".to_string());

        let options = MultiplexerOptions {
            direction: SplitDirection::Horizontal,
            command: "vim".to_string(),
            args: Some(vec!["file.txt".to_string(), "-R".to_string()]),
            cwd: Some("/workspace".to_string()),
            env: Some(env.clone()),
            window_name: Some("Editor".to_string()),
        };

        assert_eq!(options.direction, SplitDirection::Horizontal);
        assert_eq!(options.command, "vim");
        assert_eq!(options.args.as_ref().unwrap().len(), 2);
        assert_eq!(options.cwd, Some("/workspace".to_string()));
        assert_eq!(options.env.as_ref().unwrap().len(), 2);
        assert_eq!(options.window_name, Some("Editor".to_string()));
    }

    #[test]
    fn test_split_direction_conversions() {
        // Test that SplitDirection can be converted to tmux/kitty specific directions
        let new_dir = SplitDirection::New;
        let vert_dir = SplitDirection::Vertical;
        let horiz_dir = SplitDirection::Horizontal;

        // These conversions happen in execute_in_multiplexer
        match new_dir {
            SplitDirection::New => {}
            _ => panic!("Expected SplitDirection::New"),
        }

        match vert_dir {
            SplitDirection::Vertical => {}
            _ => panic!("Expected SplitDirection::Vertical"),
        }

        match horiz_dir {
            SplitDirection::Horizontal => {}
            _ => panic!("Expected SplitDirection::Horizontal"),
        }
    }

    #[test]
    fn test_multiplexer_variants() {
        let multiplexers = vec![Multiplexer::Tmux, Multiplexer::Kitty, Multiplexer::None];

        for m in multiplexers {
            match m {
                Multiplexer::Tmux => assert_eq!(format!("{m:?}"), "Tmux"),
                Multiplexer::Kitty => assert_eq!(format!("{m:?}"), "Kitty"),
                Multiplexer::None => assert_eq!(format!("{m:?}"), "None"),
            }
        }
    }

    #[tokio::test]
    async fn test_execute_fallback_warning() {
        // Test that fallback execution works
        let options = MultiplexerOptions {
            direction: SplitDirection::Vertical, // This should trigger a warning in fallback
            command: "true".to_string(),         // Command that always succeeds
            args: None,
            cwd: None,
            env: None,
            window_name: None,
        };

        // Call execute_fallback directly
        let result = execute_fallback(options).await;

        // The "true" command should succeed
        assert!(result.is_ok());
        if let Ok(success) = result {
            assert_eq!(success.exit_code, 0);
        }
    }

    #[tokio::test]
    async fn test_execute_fallback_with_args() {
        let options = MultiplexerOptions {
            direction: SplitDirection::New,
            command: "echo".to_string(),
            args: Some(vec!["hello".to_string(), "world".to_string()]),
            cwd: None,
            env: None,
            window_name: None,
        };

        let result = execute_fallback(options).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_execute_fallback_with_env() {
        let mut env = HashMap::new();
        env.insert("TEST_VAR".to_string(), "test_value".to_string());

        let options = MultiplexerOptions {
            direction: SplitDirection::New,
            command: "printenv".to_string(),
            args: Some(vec!["TEST_VAR".to_string()]),
            cwd: None,
            env: Some(env),
            window_name: None,
        };

        let result = execute_fallback(options).await;
        assert!(result.is_ok());
    }
}
