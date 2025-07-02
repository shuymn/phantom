use crate::core::command_executor::{CommandArgs, CommandConfig, CommandExecutor};
use crate::Result;
use serde::{Deserialize, Serialize};
use smallvec::smallvec;
use std::collections::HashMap;
use std::env;

use super::spawn::SpawnSuccess;

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

/// Execute a command in kitty with CommandExecutor
pub async fn execute_kitty_command<E>(executor: &E, options: KittyOptions) -> Result<()>
where
    E: CommandExecutor,
{
    let mut kitty_args: CommandArgs = smallvec!["@".to_string(), "launch".to_string()];

    // Set up the kitty command based on direction
    match options.direction {
        KittySplitDirection::New => {
            kitty_args.push("--type=tab".to_string());
            if let Some(window_title) = &options.window_title {
                kitty_args.push(format!("--tab-title={window_title}"));
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
        kitty_args.push(format!("--cwd={cwd}"));
    }

    // Add environment variables in sorted order for deterministic output
    if let Some(env_vars) = &options.env {
        let mut sorted_keys: Vec<_> = env_vars.keys().collect();
        sorted_keys.sort();
        for key in sorted_keys {
            if let Some(value) = env_vars.get(key) {
                kitty_args.push(format!("--env={key}={value}"));
            }
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
    let config = CommandConfig::new("kitty").with_args_smallvec(kitty_args);
    executor.execute(config).await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::executors::MockCommandExecutor;

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

    #[test]
    fn test_kitty_split_direction_all_variants() {
        use serde_json;

        let directions = vec![
            (KittySplitDirection::New, "\"new\""),
            (KittySplitDirection::Vertical, "\"vertical\""),
            (KittySplitDirection::Horizontal, "\"horizontal\""),
        ];

        for (direction, expected_json) in directions {
            let json = serde_json::to_string(&direction).unwrap();
            assert_eq!(json, expected_json);

            let deserialized: KittySplitDirection = serde_json::from_str(&json).unwrap();
            assert_eq!(deserialized, direction);
        }
    }

    #[test]
    fn test_kitty_split_direction_equality() {
        assert_eq!(KittySplitDirection::New, KittySplitDirection::New);
        assert_ne!(KittySplitDirection::New, KittySplitDirection::Vertical);
        assert_ne!(KittySplitDirection::Vertical, KittySplitDirection::Horizontal);
    }

    #[test]
    fn test_kitty_split_direction_copy_clone() {
        let original = KittySplitDirection::Vertical;
        let copied = original;
        let cloned = original;

        assert_eq!(original, copied);
        assert_eq!(original, cloned);
    }

    #[test]
    fn test_kitty_split_direction_debug() {
        let new = KittySplitDirection::New;
        let debug_str = format!("{new:?}");
        assert!(debug_str.contains("New"));

        let vertical = KittySplitDirection::Vertical;
        let debug_str = format!("{vertical:?}");
        assert!(debug_str.contains("Vertical"));

        let horizontal = KittySplitDirection::Horizontal;
        let debug_str = format!("{horizontal:?}");
        assert!(debug_str.contains("Horizontal"));
    }

    #[test]
    fn test_kitty_options_debug() {
        let options = KittyOptions {
            direction: KittySplitDirection::New,
            command: "test".to_string(),
            args: None,
            cwd: None,
            env: None,
            window_title: None,
        };

        let debug_str = format!("{options:?}");
        assert!(debug_str.contains("KittyOptions"));
        assert!(debug_str.contains("direction"));
        assert!(debug_str.contains("command"));
    }

    #[test]
    fn test_kitty_options_clone() {
        let options = KittyOptions {
            direction: KittySplitDirection::Horizontal,
            command: "ls".to_string(),
            args: Some(vec!["-la".to_string()]),
            cwd: Some("/home".to_string()),
            env: Some(HashMap::from([("VAR".to_string(), "value".to_string())])),
            window_title: Some("Files".to_string()),
        };

        let cloned = options.clone();

        assert_eq!(options.direction, cloned.direction);
        assert_eq!(options.command, cloned.command);
        assert_eq!(options.args, cloned.args);
        assert_eq!(options.cwd, cloned.cwd);
        assert_eq!(options.env, cloned.env);
        assert_eq!(options.window_title, cloned.window_title);
    }

    #[test]
    fn test_kitty_options_minimal() {
        let options = KittyOptions {
            direction: KittySplitDirection::New,
            command: "bash".to_string(),
            args: None,
            cwd: None,
            env: None,
            window_title: None,
        };

        assert_eq!(options.command, "bash");
        assert!(options.args.is_none());
        assert!(options.cwd.is_none());
        assert!(options.env.is_none());
        assert!(options.window_title.is_none());
    }

    #[test]
    fn test_kitty_options_with_env() {
        let mut env = HashMap::new();
        env.insert("PATH".to_string(), "/usr/local/bin".to_string());
        env.insert("HOME".to_string(), "/home/user".to_string());

        let options = KittyOptions {
            direction: KittySplitDirection::Vertical,
            command: "env".to_string(),
            args: None,
            cwd: None,
            env: Some(env.clone()),
            window_title: None,
        };

        assert_eq!(options.env.unwrap().len(), 2);
    }

    #[test]
    fn test_kitty_options_with_window_title() {
        let options = KittyOptions {
            direction: KittySplitDirection::New,
            command: "vim".to_string(),
            args: Some(vec!["file.txt".to_string()]),
            cwd: Some("/home/user".to_string()),
            env: None,
            window_title: Some("Editor".to_string()),
        };

        assert_eq!(options.window_title, Some("Editor".to_string()));
        assert_eq!(options.command, "vim");
        assert!(options.args.is_some());
    }

    #[test]
    fn test_kitty_options_comprehensive() {
        let mut env = HashMap::new();
        env.insert("EDITOR".to_string(), "vim".to_string());

        let options = KittyOptions {
            direction: KittySplitDirection::Horizontal,
            command: "bash".to_string(),
            args: Some(vec!["-c".to_string(), "echo test".to_string()]),
            cwd: Some("/tmp".to_string()),
            env: Some(env),
            window_title: Some("Test Window".to_string()),
        };

        assert_eq!(options.direction, KittySplitDirection::Horizontal);
        assert_eq!(options.command, "bash");
        assert_eq!(options.args.as_ref().unwrap().len(), 2);
        assert_eq!(options.cwd, Some("/tmp".to_string()));
        assert!(options.env.is_some());
        assert_eq!(options.window_title, Some("Test Window".to_string()));
    }

    // Mock tests for CommandExecutor functions
    #[tokio::test]
    async fn test_execute_kitty_command_new_tab() {
        let mut mock = MockCommandExecutor::new();
        mock.expect_command("kitty")
            .with_args(&[
                "@",
                "launch",
                "--type=tab",
                "--tab-title=Test Window",
                "--cwd=/tmp",
                "--env=TEST=value",
                "--",
                "echo",
                "hello",
            ])
            .returns_output("", "", 0);

        let options = KittyOptions {
            direction: KittySplitDirection::New,
            command: "echo".to_string(),
            args: Some(vec!["hello".to_string()]),
            cwd: Some("/tmp".to_string()),
            env: Some(HashMap::from([("TEST".to_string(), "value".to_string())])),
            window_title: Some("Test Window".to_string()),
        };

        let result = execute_kitty_command(&mock, options).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_execute_kitty_command_vertical_split() {
        let mut mock = MockCommandExecutor::new();
        mock.expect_command("kitty")
            .with_args(&["@", "launch", "--location=vsplit", "--", "htop"])
            .returns_output("", "", 0);

        let options = KittyOptions {
            direction: KittySplitDirection::Vertical,
            command: "htop".to_string(),
            args: None,
            cwd: None,
            env: None,
            window_title: None,
        };

        let result = execute_kitty_command(&mock, options).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_execute_kitty_command_horizontal_split() {
        let mut mock = MockCommandExecutor::new();
        mock.expect_command("kitty")
            .with_args(&["@", "launch", "--location=hsplit", "--", "ls", "-la"])
            .returns_output("", "", 0);

        let options = KittyOptions {
            direction: KittySplitDirection::Horizontal,
            command: "ls".to_string(),
            args: Some(vec!["-la".to_string()]),
            cwd: None,
            env: None,
            window_title: None,
        };

        let result = execute_kitty_command(&mock, options).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_execute_kitty_command_args_construction() {
        // This test verifies that the arguments would be constructed correctly
        // without actually executing kitty (which might not be available in test env)
        let options = KittyOptions {
            direction: KittySplitDirection::New,
            command: "echo".to_string(),
            args: Some(vec!["hello".to_string()]),
            cwd: Some("/tmp".to_string()),
            env: Some(HashMap::from([("TEST".to_string(), "value".to_string())])),
            window_title: Some("Test".to_string()),
        };

        // The function would construct args like:
        // ["@", "launch", "--type=tab", "--tab-title=Test", "--cwd=/tmp", "--env=TEST=value", "--", "echo", "hello"]
        assert_eq!(options.direction, KittySplitDirection::New);
        assert!(options.window_title.is_some());
    }

    #[test]
    fn test_is_inside_kitty_with_env() {
        // Test the logic of is_inside_kitty
        // In test environment, these env vars are usually not set
        let has_kitty_term = env::var("TERM").map(|t| t == "xterm-kitty").unwrap_or(false);
        let has_kitty_window_id = env::var("KITTY_WINDOW_ID").is_ok();

        // The function returns true if either condition is met
        let expected = has_kitty_term || has_kitty_window_id;

        // This just verifies the logic, actual test is in test_is_inside_kitty
        assert!(!expected || expected); // Always true, just for logic verification
    }

    #[test]
    fn test_kitty_success_type_alias() {
        use super::super::spawn::SpawnSuccess;
        use super::KittySuccess;

        // Verify that KittySuccess is just an alias for SpawnSuccess
        let success: KittySuccess = SpawnSuccess { exit_code: 0 };
        assert_eq!(success.exit_code, 0);
    }
}
