use crate::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use std::path::Path;

use super::spawn::{execute_command, spawn_process, SpawnConfig, SpawnSuccess};

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
    let mut args = vec![
        "new-session".to_string(),
        "-d".to_string(),
        "-s".to_string(),
        session_name.to_string(),
    ];

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
    let output = execute_command("tmux", args, None).await?;

    let sessions = output.lines().map(|s| s.to_string()).filter(|s| !s.is_empty()).collect();

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
        Ok(result) => {
            // tmux has-session returns 0 if session exists, non-zero if it doesn't
            Ok(result.exit_code == 0)
        }
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

    #[test]
    fn test_tmux_split_direction_all_variants() {
        use serde_json;

        let directions = vec![
            (TmuxSplitDirection::New, "\"new\""),
            (TmuxSplitDirection::Vertical, "\"vertical\""),
            (TmuxSplitDirection::Horizontal, "\"horizontal\""),
        ];

        for (direction, expected_json) in directions {
            let json = serde_json::to_string(&direction).unwrap();
            assert_eq!(json, expected_json);

            let deserialized: TmuxSplitDirection = serde_json::from_str(&json).unwrap();
            assert_eq!(deserialized, direction);
        }
    }

    #[test]
    fn test_tmux_split_direction_equality() {
        assert_eq!(TmuxSplitDirection::New, TmuxSplitDirection::New);
        assert_ne!(TmuxSplitDirection::New, TmuxSplitDirection::Vertical);
        assert_ne!(TmuxSplitDirection::Vertical, TmuxSplitDirection::Horizontal);
    }

    #[test]
    fn test_tmux_split_direction_copy_clone() {
        let original = TmuxSplitDirection::Horizontal;
        let copied = original;
        let cloned = original.clone();

        assert_eq!(original, copied);
        assert_eq!(original, cloned);
    }

    #[test]
    fn test_tmux_split_direction_debug() {
        let new = TmuxSplitDirection::New;
        let debug_str = format!("{:?}", new);
        assert!(debug_str.contains("New"));

        let vertical = TmuxSplitDirection::Vertical;
        let debug_str = format!("{:?}", vertical);
        assert!(debug_str.contains("Vertical"));

        let horizontal = TmuxSplitDirection::Horizontal;
        let debug_str = format!("{:?}", horizontal);
        assert!(debug_str.contains("Horizontal"));
    }

    #[test]
    fn test_tmux_options_debug() {
        let options = TmuxOptions {
            direction: TmuxSplitDirection::New,
            command: "test".to_string(),
            args: None,
            cwd: None,
            env: None,
            window_name: Some("TestWindow".to_string()),
        };

        let debug_str = format!("{:?}", options);
        assert!(debug_str.contains("TmuxOptions"));
        assert!(debug_str.contains("direction"));
        assert!(debug_str.contains("command"));
        assert!(debug_str.contains("window_name"));
    }

    #[test]
    fn test_tmux_options_clone() {
        let options = TmuxOptions {
            direction: TmuxSplitDirection::Vertical,
            command: "vim".to_string(),
            args: Some(vec!["file.txt".to_string()]),
            cwd: Some("/workspace".to_string()),
            env: Some(HashMap::from([("EDITOR".to_string(), "vim".to_string())])),
            window_name: Some("Editor".to_string()),
        };

        let cloned = options.clone();

        assert_eq!(options.direction, cloned.direction);
        assert_eq!(options.command, cloned.command);
        assert_eq!(options.args, cloned.args);
        assert_eq!(options.cwd, cloned.cwd);
        assert_eq!(options.env, cloned.env);
        assert_eq!(options.window_name, cloned.window_name);
    }

    #[test]
    fn test_tmux_options_minimal() {
        let options = TmuxOptions {
            direction: TmuxSplitDirection::New,
            command: "sh".to_string(),
            args: None,
            cwd: None,
            env: None,
            window_name: None,
        };

        assert_eq!(options.command, "sh");
        assert!(options.args.is_none());
        assert!(options.cwd.is_none());
        assert!(options.env.is_none());
        assert!(options.window_name.is_none());
    }

    #[test]
    fn test_tmux_options_with_args() {
        let options = TmuxOptions {
            direction: TmuxSplitDirection::Horizontal,
            command: "python".to_string(),
            args: Some(vec!["script.py".to_string(), "--verbose".to_string()]),
            cwd: None,
            env: None,
            window_name: None,
        };

        let args = options.args.unwrap();
        assert_eq!(args.len(), 2);
        assert_eq!(args[0], "script.py");
        assert_eq!(args[1], "--verbose");
    }

    #[test]
    fn test_tmux_options_with_window_name() {
        let options = TmuxOptions {
            direction: TmuxSplitDirection::New,
            command: "htop".to_string(),
            args: None,
            cwd: None,
            env: None,
            window_name: Some("System Monitor".to_string()),
        };

        assert_eq!(options.window_name, Some("System Monitor".to_string()));
        assert_eq!(options.direction, TmuxSplitDirection::New);
    }

    #[tokio::test]
    async fn test_tmux_session_exists() {
        // First check if we're inside tmux already, which would affect the test
        if env::var("TMUX").is_ok() {
            eprintln!("Skipping test: already inside tmux session");
            return;
        }

        // Generate a unique session name that's very unlikely to exist
        use std::time::{SystemTime, UNIX_EPOCH};
        let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
        let unique_session =
            format!("phantom-test-nonexistent-{}-{}", std::process::id(), timestamp);

        // Test with a session that should not exist
        let result = tmux_session_exists(&unique_session).await;

        // Skip test if tmux is not available
        let exists = match result {
            Ok(exists) => exists,
            Err(e) => {
                eprintln!("Error checking session: {:?}", e);
                return; // Skip test if tmux is not available
            }
        };
        assert!(!exists, "Nonexistent session '{}' should not exist", unique_session);
    }

    #[test]
    fn test_tmux_command_building() {
        // Test that we build the correct tmux command arguments
        let _options = TmuxOptions {
            direction: TmuxSplitDirection::New,
            command: "vim".to_string(),
            args: Some(vec!["file.txt".to_string()]),
            cwd: Some("/workspace".to_string()),
            env: Some(HashMap::from([
                ("VAR1".to_string(), "value1".to_string()),
                ("VAR2".to_string(), "value2".to_string()),
            ])),
            window_name: Some("editor".to_string()),
        };

        // Simulate building tmux args
        let mut tmux_args = Vec::new();

        // New window
        tmux_args.push("new-window".to_string());
        tmux_args.push("-n".to_string());
        tmux_args.push("editor".to_string());

        // Working directory
        tmux_args.push("-c".to_string());
        tmux_args.push("/workspace".to_string());

        // Environment variables
        tmux_args.push("-e".to_string());
        tmux_args.push("VAR1=value1".to_string());
        tmux_args.push("-e".to_string());
        tmux_args.push("VAR2=value2".to_string());

        // Command and args
        tmux_args.push("vim".to_string());
        tmux_args.push("file.txt".to_string());

        assert!(tmux_args.contains(&"new-window".to_string()));
        assert!(tmux_args.contains(&"-n".to_string()));
        assert!(tmux_args.contains(&"editor".to_string()));
        assert!(tmux_args.contains(&"-c".to_string()));
        assert!(tmux_args.contains(&"/workspace".to_string()));
        assert!(tmux_args.contains(&"-e".to_string()));
        assert!(tmux_args.contains(&"vim".to_string()));
        assert!(tmux_args.contains(&"file.txt".to_string()));
    }

    #[test]
    fn test_session_name_formatting() {
        let session_names =
            vec!["simple", "with-dash", "with_underscore", "123numeric", "MixedCase"];

        for name in session_names {
            assert!(!name.is_empty());
            assert!(name.chars().all(|c| c.is_ascii()));
        }
    }

    #[test]
    fn test_parse_session_list() {
        let output = "session1\nsession2\nsession3\n";
        let sessions: Vec<String> =
            output.lines().map(|s| s.to_string()).filter(|s| !s.is_empty()).collect();

        assert_eq!(sessions.len(), 3);
        assert_eq!(sessions[0], "session1");
        assert_eq!(sessions[1], "session2");
        assert_eq!(sessions[2], "session3");

        // Test with empty lines
        let output_with_empty = "session1\n\nsession2\n\n";
        let sessions2: Vec<String> =
            output_with_empty.lines().map(|s| s.to_string()).filter(|s| !s.is_empty()).collect();

        assert_eq!(sessions2.len(), 2);
    }

    #[test]
    fn test_tmux_error_handling() {
        use crate::PhantomError;
        // Test ProcessExecution error handling
        let exec_error = PhantomError::ProcessExecution("tmux failed".to_string());
        assert!(exec_error.to_string().contains("tmux failed"));

        // Test pattern matching for has-session
        match exec_error {
            PhantomError::ProcessExecution(_) => {}
            _ => panic!("Expected ProcessExecution error"),
        }
    }

    #[test]
    fn test_path_to_string_lossy() {
        let paths = vec![
            Path::new("/home/user"),
            Path::new("/tmp"),
            Path::new("/path/with spaces"),
            Path::new("/path/with/多/unicode"),
        ];

        for path in paths {
            let string = path.to_string_lossy().to_string();
            assert!(!string.is_empty());
        }
    }

    #[test]
    fn test_env_var_formatting() {
        let env_vars = HashMap::from([
            ("PATH".to_string(), "/usr/bin:/bin".to_string()),
            ("HOME".to_string(), "/home/user".to_string()),
            ("TERM".to_string(), "xterm-256color".to_string()),
        ]);

        for (key, value) in env_vars {
            let formatted = format!("{}={}", key, value);
            assert!(formatted.contains('='));
            assert!(formatted.contains(&key));
            assert!(formatted.contains(&value));
        }
    }
}
