use async_trait::async_trait;
use std::process::Stdio;
use tokio::process::Command;
use tracing::{debug, error, info};

use crate::core::command_executor::{
    CommandConfig, CommandExecutor, CommandOutput, SpawnConfig, SpawnOutput,
};
use crate::core::error::PhantomError;
use crate::core::result::Result;

pub struct RealCommandExecutor;

impl RealCommandExecutor {
    pub fn new() -> Self {
        Self
    }
}

impl Default for RealCommandExecutor {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl CommandExecutor for RealCommandExecutor {
    async fn execute(&self, config: CommandConfig) -> Result<CommandOutput> {
        info!("Executing command: {} {:?}", config.program, config.args);

        let mut command = Command::new(&config.program);
        command.args(&config.args);

        if let Some(ref cwd) = config.cwd {
            command.current_dir(cwd);
        }

        if let Some(ref env) = config.env {
            command.envs(env);
        }

        let output = if let Some(timeout) = config.timeout {
            match tokio::time::timeout(timeout, command.output()).await {
                Ok(Ok(output)) => output,
                Ok(Err(e)) => {
                    return Err(PhantomError::ProcessExecution(format!(
                        "Failed to execute command '{}': {}",
                        config.program, e
                    )));
                }
                Err(_) => {
                    error!("Command timeout after {:?}, killing process", timeout);
                    return Err(PhantomError::ProcessExecution(format!(
                        "Command '{}' timed out after {:?}",
                        config.program, timeout
                    )));
                }
            }
        } else {
            command.output().await.map_err(|e| {
                PhantomError::ProcessExecution(format!(
                    "Failed to execute command '{}': {}",
                    config.program, e
                ))
            })?
        };

        let exit_code = output.status.code().unwrap_or(-1);
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();

        debug!(
            "Command '{}' exited with code: {}, stdout: {} bytes, stderr: {} bytes",
            config.program,
            exit_code,
            stdout.len(),
            stderr.len()
        );

        Ok(CommandOutput {
            stdout,
            stderr,
            exit_code,
        })
    }

    async fn spawn(&self, config: SpawnConfig) -> Result<SpawnOutput> {
        info!("Spawning process: {} {:?}", config.program, config.args);

        let mut command = Command::new(&config.program);
        command.args(&config.args);

        if let Some(ref cwd) = config.cwd {
            command.current_dir(cwd);
        }

        if let Some(ref env) = config.env {
            command.envs(env);
        }

        command.stdin(if config.stdin {
            Stdio::inherit()
        } else {
            Stdio::null()
        });

        command.stdout(if config.stdout {
            Stdio::inherit()
        } else {
            Stdio::null()
        });

        command.stderr(if config.stderr {
            Stdio::inherit()
        } else {
            Stdio::null()
        });

        let child = command.spawn().map_err(|e| {
            PhantomError::ProcessExecution(format!(
                "Failed to spawn process '{}': {}",
                config.program, e
            ))
        })?;

        let pid = child.id().unwrap_or(0);
        debug!("Process spawned with PID: {}", pid);

        Ok(SpawnOutput { pid })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::time::Duration;

    #[tokio::test]
    async fn test_execute_success() {
        let executor = RealCommandExecutor::new();
        let config = CommandConfig::new("echo")
            .with_args(vec!["hello".to_string(), "world".to_string()]);

        let result = executor.execute(config).await;
        assert!(result.is_ok());

        let output = result.unwrap();
        assert_eq!(output.exit_code, 0);
        assert_eq!(output.stdout.trim(), "hello world");
        assert!(output.stderr.is_empty());
    }

    #[tokio::test]
    async fn test_execute_with_cwd() {
        let executor = RealCommandExecutor::new();
        let temp_dir = tempfile::tempdir().unwrap();
        let cwd = temp_dir.path().to_path_buf();

        std::fs::write(temp_dir.path().join("test.txt"), "test content").unwrap();

        let config = CommandConfig::new("ls").with_cwd(cwd);

        let result = executor.execute(config).await;
        assert!(result.is_ok());

        let output = result.unwrap();
        assert_eq!(output.exit_code, 0);
        assert!(output.stdout.contains("test.txt"));
    }

    #[tokio::test]
    async fn test_execute_with_env() {
        let executor = RealCommandExecutor::new();
        let mut env = HashMap::new();
        env.insert("TEST_VAR".to_string(), "test_value".to_string());

        let config = CommandConfig::new("sh")
            .with_args(vec!["-c".to_string(), "echo $TEST_VAR".to_string()])
            .with_env(env);

        let result = executor.execute(config).await;
        assert!(result.is_ok());

        let output = result.unwrap();
        assert_eq!(output.exit_code, 0);
        assert_eq!(output.stdout.trim(), "test_value");
    }

    #[tokio::test]
    async fn test_execute_nonexistent_command() {
        let executor = RealCommandExecutor::new();
        let config = CommandConfig::new("nonexistent-command-xyz123");

        let result = executor.execute(config).await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Failed to execute command"));
    }

    #[tokio::test]
    async fn test_execute_with_timeout() {
        let executor = RealCommandExecutor::new();
        let config = CommandConfig::new("sleep")
            .with_args(vec!["10".to_string()])
            .with_timeout(Duration::from_millis(100));

        let result = executor.execute(config).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("timed out"));
    }

    #[tokio::test]
    async fn test_execute_non_zero_exit() {
        let executor = RealCommandExecutor::new();
        let config = CommandConfig::new("false");

        let result = executor.execute(config).await;
        assert!(result.is_ok());

        let output = result.unwrap();
        assert_eq!(output.exit_code, 1);
    }

    #[tokio::test]
    async fn test_spawn_success() {
        let executor = RealCommandExecutor::new();
        let config = SpawnConfig::new("sleep").with_args(vec!["0.1".to_string()]);

        let result = executor.spawn(config).await;
        assert!(result.is_ok());

        let output = result.unwrap();
        assert!(output.pid > 0);
    }

    #[tokio::test]
    async fn test_spawn_with_cwd() {
        let executor = RealCommandExecutor::new();
        let temp_dir = tempfile::tempdir().unwrap();
        let cwd = temp_dir.path().to_path_buf();

        let config = SpawnConfig::new("pwd").with_cwd(cwd);

        let result = executor.spawn(config).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_spawn_nonexistent_command() {
        let executor = RealCommandExecutor::new();
        let config = SpawnConfig::new("nonexistent-command-xyz123");

        let result = executor.spawn(config).await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Failed to spawn process"));
    }

    #[tokio::test]
    async fn test_default_impl() {
        let executor1 = RealCommandExecutor::new();
        let executor2 = RealCommandExecutor::default();

        let config = CommandConfig::new("echo").with_args(vec!["test".to_string()]);

        let result1 = executor1.execute(config.clone()).await;
        let result2 = executor2.execute(config).await;

        assert!(result1.is_ok());
        assert!(result2.is_ok());
    }
}