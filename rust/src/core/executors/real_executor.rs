use async_trait::async_trait;
use std::process::Stdio;
use tokio::process::Command;
use tracing::{debug, error, info};

use crate::core::command_executor::{CommandConfig, CommandExecutor, CommandOutput};
use crate::core::error::PhantomError;
use crate::core::result::Result;
use crate::core::sealed::Sealed;

#[derive(Clone)]
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

// Implement the sealed trait
impl Sealed for RealCommandExecutor {}

// Implement Sealed for &RealCommandExecutor
impl Sealed for &RealCommandExecutor {}

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

        // Configure stdin based on whether stdin_data is provided
        if config.stdin_data.is_some() {
            command.stdin(Stdio::piped());
        } else {
            command.stdin(Stdio::null());
        }

        // Always capture stdout and stderr for CommandOutput
        command.stdout(Stdio::piped());
        command.stderr(Stdio::piped());

        // Handle execution with or without stdin_data
        let output = if let Some(stdin_data) = config.stdin_data {
            debug!("Executing command with stdin data: {} bytes", stdin_data.len());

            // Spawn the process to get access to stdin
            let mut child = command.spawn().map_err(|e| {
                if e.kind() == std::io::ErrorKind::NotFound {
                    PhantomError::CommandNotFound { command: config.program.clone() }
                } else {
                    PhantomError::ProcessExecutionError {
                        reason: format!("Failed to spawn command '{}': {}", config.program, e),
                    }
                }
            })?;

            // Write stdin data
            if let Some(mut stdin) = child.stdin.take() {
                use tokio::io::AsyncWriteExt;
                stdin.write_all(stdin_data.as_bytes()).await.map_err(|e| {
                    PhantomError::ProcessExecutionError {
                        reason: format!("Failed to write stdin to '{}': {}", config.program, e),
                    }
                })?;
                stdin.shutdown().await.map_err(|e| PhantomError::ProcessExecutionError {
                    reason: format!("Failed to close stdin for '{}': {}", config.program, e),
                })?;
            }

            // Wait for completion with optional timeout
            if let Some(timeout) = config.timeout {
                match tokio::time::timeout(timeout, child.wait_with_output()).await {
                    Ok(Ok(output)) => output,
                    Ok(Err(e)) => {
                        return Err(PhantomError::ProcessExecutionError {
                            reason: format!(
                                "Failed to wait for command '{}': {}",
                                config.program, e
                            ),
                        });
                    }
                    Err(_) => {
                        error!("Command timeout after {:?}", timeout);
                        // The child process is consumed by wait_with_output, so we can't kill it
                        // The timeout itself should cause the process to be terminated
                        return Err(PhantomError::ProcessExecutionError {
                            reason: format!(
                                "Command '{}' timed out after {:?}",
                                config.program, timeout
                            ),
                        });
                    }
                }
            } else {
                child.wait_with_output().await.map_err(|e| PhantomError::ProcessExecutionError {
                    reason: format!("Failed to wait for command '{}': {}", config.program, e),
                })?
            }
        } else {
            // No stdin data, use simpler output() method
            if let Some(timeout) = config.timeout {
                match tokio::time::timeout(timeout, command.output()).await {
                    Ok(Ok(output)) => output,
                    Ok(Err(e)) => {
                        return Err(if e.kind() == std::io::ErrorKind::NotFound {
                            PhantomError::CommandNotFound { command: config.program.clone() }
                        } else {
                            PhantomError::ProcessExecutionError {
                                reason: format!(
                                    "Failed to execute command '{}': {}",
                                    config.program, e
                                ),
                            }
                        });
                    }
                    Err(_) => {
                        error!("Command timeout after {:?}, killing process", timeout);
                        return Err(PhantomError::ProcessExecutionError {
                            reason: format!(
                                "Command '{}' timed out after {:?}",
                                config.program, timeout
                            ),
                        });
                    }
                }
            } else {
                command.output().await.map_err(|e| {
                    if e.kind() == std::io::ErrorKind::NotFound {
                        PhantomError::CommandNotFound { command: config.program.clone() }
                    } else {
                        PhantomError::ProcessExecutionError {
                            reason: format!(
                                "Failed to execute command '{}': {}",
                                config.program, e
                            ),
                        }
                    }
                })?
            }
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

        Ok(CommandOutput::new(stdout, stderr, exit_code))
    }
}

// Implement CommandExecutor for &RealCommandExecutor
#[async_trait]
impl CommandExecutor for &RealCommandExecutor {
    async fn execute(&self, config: CommandConfig) -> Result<CommandOutput> {
        (*self).execute(config).await
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
        let config =
            CommandConfig::new("echo").with_args(vec!["hello".to_string(), "world".to_string()]);

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
        assert!(result.unwrap_err().to_string().contains("Failed to execute command"));
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
    async fn test_default_impl() {
        let executor1 = RealCommandExecutor::new();
        let executor2 = RealCommandExecutor::default();

        let config = CommandConfig::new("echo").with_args(vec!["test".to_string()]);

        let result1 = executor1.execute(config.clone()).await;
        let result2 = executor2.execute(config).await;

        assert!(result1.is_ok());
        assert!(result2.is_ok());
    }

    #[tokio::test]
    async fn test_execute_with_stdin_data() {
        let executor = RealCommandExecutor::new();
        let config = CommandConfig::new("cat").with_stdin_data("hello from stdin".to_string());

        let result = executor.execute(config).await;
        assert!(result.is_ok());

        let output = result.unwrap();
        assert_eq!(output.exit_code, 0);
        assert_eq!(output.stdout.trim(), "hello from stdin");
        assert!(output.stderr.is_empty());
    }

    #[tokio::test]
    async fn test_execute_with_stdin_data_and_args() {
        let executor = RealCommandExecutor::new();
        let config = CommandConfig::new("grep")
            .with_args(vec!["hello".to_string()])
            .with_stdin_data("hello world\ngoodbye world\nhello again".to_string());

        let result = executor.execute(config).await;
        assert!(result.is_ok());

        let output = result.unwrap();
        assert_eq!(output.exit_code, 0);
        assert!(output.stdout.contains("hello world"));
        assert!(output.stdout.contains("hello again"));
        assert!(!output.stdout.contains("goodbye"));
    }

    #[tokio::test]
    async fn test_execute_with_stdin_data_timeout() {
        let executor = RealCommandExecutor::new();
        // Use a command that will block waiting for more input
        let config = CommandConfig::new("sh")
            .with_args(vec!["-c".to_string(), "cat && sleep 10".to_string()])
            .with_stdin_data("test".to_string())
            .with_timeout(Duration::from_millis(100));

        let result = executor.execute(config).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("timed out"));
    }
}
