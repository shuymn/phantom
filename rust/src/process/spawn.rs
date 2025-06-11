use crate::{PhantomError, Result};
use std::collections::HashMap;
use std::ffi::OsStr;
use std::path::Path;
use std::process::Stdio;
use tokio::process::{Child, Command};
use tracing::{debug, error, info};

/// Configuration for spawning a process
#[derive(Debug, Clone)]
pub struct SpawnConfig {
    /// The command to execute
    pub command: String,
    /// Arguments to pass to the command
    pub args: Vec<String>,
    /// Working directory for the process
    pub cwd: Option<String>,
    /// Environment variables
    pub env: Option<HashMap<String, String>>,
    /// Whether to inherit stdio
    pub inherit_stdio: bool,
    /// Timeout in milliseconds (None for no timeout)
    pub timeout_ms: Option<u64>,
}

impl Default for SpawnConfig {
    fn default() -> Self {
        Self {
            command: String::new(),
            args: Vec::new(),
            cwd: None,
            env: None,
            inherit_stdio: true,
            timeout_ms: None,
        }
    }
}

/// Result of a successful process spawn
#[derive(Debug, Clone)]
pub struct SpawnSuccess {
    pub exit_code: i32,
}

/// Spawn a process asynchronously
pub async fn spawn_process(config: SpawnConfig) -> Result<SpawnSuccess> {
    info!("Spawning process: {} {:?}", config.command, config.args);

    let mut command = Command::new(&config.command);
    command.args(&config.args);

    // Set working directory if provided
    if let Some(ref cwd) = config.cwd {
        command.current_dir(cwd);
    }

    // Set environment variables if provided
    if let Some(ref env) = config.env {
        command.envs(env);
    }

    // Configure stdio
    if config.inherit_stdio {
        command.stdin(Stdio::inherit());
        command.stdout(Stdio::inherit());
        command.stderr(Stdio::inherit());
    } else {
        command.stdin(Stdio::null());
        command.stdout(Stdio::null());
        command.stderr(Stdio::null());
    }

    // Spawn the process
    let mut child = command.spawn().map_err(|e| {
        PhantomError::ProcessExecution(format!(
            "Failed to spawn process '{}': {}",
            config.command, e
        ))
    })?;

    debug!("Process spawned with PID: {:?}", child.id());

    // Handle timeout if specified
    let exit_status = if let Some(timeout_ms) = config.timeout_ms {
        match tokio::time::timeout(
            tokio::time::Duration::from_millis(timeout_ms),
            child.wait(),
        )
        .await
        {
            Ok(Ok(status)) => status,
            Ok(Err(e)) => {
                return Err(PhantomError::ProcessExecution(format!(
                    "Failed to wait for process: {}",
                    e
                )));
            }
            Err(_) => {
                // Timeout occurred, kill the process
                error!("Process timeout after {}ms, killing process", timeout_ms);
                child.kill().await.ok();
                return Err(PhantomError::ProcessExecution(format!(
                    "Process '{}' timed out after {}ms",
                    config.command, timeout_ms
                )));
            }
        }
    } else {
        child.wait().await.map_err(|e| {
            PhantomError::ProcessExecution(format!("Failed to wait for process: {}", e))
        })?
    };

    let exit_code = exit_status.code().unwrap_or(-1);
    debug!("Process exited with code: {}", exit_code);

    Ok(SpawnSuccess { exit_code })
}

/// Spawn a process and return immediately without waiting
pub async fn spawn_detached(config: SpawnConfig) -> Result<Child> {
    info!("Spawning detached process: {} {:?}", config.command, config.args);

    let mut command = Command::new(&config.command);
    command.args(&config.args);

    // Set working directory if provided
    if let Some(ref cwd) = config.cwd {
        command.current_dir(cwd);
    }

    // Set environment variables if provided
    if let Some(ref env) = config.env {
        command.envs(env);
    }

    // Configure stdio for detached process
    command.stdin(Stdio::null());
    command.stdout(Stdio::null());
    command.stderr(Stdio::null());

    // Spawn the process
    let child = command.spawn().map_err(|e| {
        PhantomError::ProcessExecution(format!(
            "Failed to spawn detached process '{}': {}",
            config.command, e
        ))
    })?;

    debug!("Detached process spawned with PID: {:?}", child.id());
    Ok(child)
}

/// Execute a command and capture output
pub async fn execute_command<S, I>(command: S, args: I, cwd: Option<&Path>) -> Result<String>
where
    S: AsRef<OsStr>,
    I: IntoIterator<Item = S>,
{
    let mut cmd = Command::new(command);
    cmd.args(args);

    if let Some(cwd) = cwd {
        cmd.current_dir(cwd);
    }

    let output = cmd.output().await.map_err(|e| {
        PhantomError::ProcessExecution(format!("Failed to execute command: {}", e))
    })?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        Err(PhantomError::ProcessExecution(format!(
            "Command failed with exit code: {}",
            output.status.code().unwrap_or(-1)
        )))
    }
}

/// Handle process signals (Unix-specific)
#[cfg(unix)]
pub async fn setup_signal_handlers() -> Result<()> {
    use tokio::signal::unix::{signal, SignalKind};

    let mut sigint = signal(SignalKind::interrupt())
        .map_err(|e| PhantomError::ProcessExecution(format!("Failed to setup SIGINT handler: {}", e)))?;
    
    let mut sigterm = signal(SignalKind::terminate())
        .map_err(|e| PhantomError::ProcessExecution(format!("Failed to setup SIGTERM handler: {}", e)))?;

    tokio::spawn(async move {
        tokio::select! {
            _ = sigint.recv() => {
                info!("Received SIGINT, shutting down gracefully");
            }
            _ = sigterm.recv() => {
                info!("Received SIGTERM, shutting down gracefully");
            }
        }
    });

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_spawn_process_success() {
        let config = SpawnConfig {
            command: "echo".to_string(),
            args: vec!["hello".to_string()],
            inherit_stdio: false,
            ..Default::default()
        };

        let result = spawn_process(config).await;
        assert!(result.is_ok());
        
        let success = result.unwrap();
        assert_eq!(success.exit_code, 0);
    }

    #[tokio::test]
    async fn test_spawn_process_with_args() {
        let config = SpawnConfig {
            command: "ls".to_string(),
            args: vec!["-la".to_string()],
            inherit_stdio: false,
            ..Default::default()
        };

        let result = spawn_process(config).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_spawn_process_failure() {
        let config = SpawnConfig {
            command: "nonexistent-command-that-should-not-exist".to_string(),
            args: vec![],
            inherit_stdio: false,
            ..Default::default()
        };

        let result = spawn_process(config).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_spawn_process_with_cwd() {
        let temp_dir = tempfile::tempdir().unwrap();
        let cwd = temp_dir.path().to_string_lossy().to_string();

        let config = SpawnConfig {
            command: "pwd".to_string(),
            args: vec![],
            cwd: Some(cwd.clone()),
            inherit_stdio: false,
            ..Default::default()
        };

        let result = spawn_process(config).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_spawn_process_with_env() {
        let mut env = HashMap::new();
        env.insert("TEST_VAR".to_string(), "test_value".to_string());

        let config = SpawnConfig {
            command: "env".to_string(),
            args: vec![],
            env: Some(env),
            inherit_stdio: false,
            ..Default::default()
        };

        let result = spawn_process(config).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_spawn_process_with_timeout() {
        let config = SpawnConfig {
            command: "sleep".to_string(),
            args: vec!["10".to_string()],
            inherit_stdio: false,
            timeout_ms: Some(100), // 100ms timeout
            ..Default::default()
        };

        let result = spawn_process(config).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("timed out"));
    }

    #[tokio::test]
    async fn test_execute_command() {
        let result = execute_command("echo", ["hello", "world"], None).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().trim(), "hello world");
    }

    #[tokio::test]
    async fn test_spawn_detached() {
        let config = SpawnConfig {
            command: "sleep".to_string(),
            args: vec!["1".to_string()],
            ..Default::default()
        };

        let result = spawn_detached(config).await;
        assert!(result.is_ok());
        
        let mut child = result.unwrap();
        assert!(child.id().is_some());
        
        // Clean up
        child.kill().await.ok();
    }
}