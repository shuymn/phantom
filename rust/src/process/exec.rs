use crate::core::filesystem::FileSystem;
use crate::process::shell::{detect_shell, get_phantom_env};
use crate::process::spawn::{spawn_process, SpawnConfig, SpawnSuccess};
use crate::worktree::validate::validate_worktree_exists;
use crate::{PhantomError, Result};
use std::collections::HashMap;
use std::env;
use std::path::Path;
use tracing::{debug, error, info};

/// Execute a command in a specific directory
pub async fn exec_in_dir(dir: &Path, command: &str, args: &[String]) -> Result<SpawnSuccess> {
    info!("Executing '{}' in directory: {}", command, dir.display());

    let config = SpawnConfig {
        command: command.to_string(),
        args: args.to_vec(),
        cwd: Some(dir.to_string_lossy().to_string()),
        inherit_stdio: true,
        ..Default::default()
    };

    spawn_process(config).await
}

/// Execute a command in a worktree
pub async fn exec_in_worktree(
    git_root: &Path,
    worktree_name: &str,
    command: &str,
    args: &[String],
    filesystem: &dyn FileSystem,
) -> Result<SpawnSuccess> {
    // Validate worktree exists
    let validation = validate_worktree_exists(git_root, worktree_name, filesystem).await?;
    let worktree_path = validation.path;

    info!("Executing '{}' in worktree '{}' at {}", command, worktree_name, worktree_path.display());

    // Prepare environment with phantom variables
    let mut env: HashMap<String, String> = env::vars().collect();
    let phantom_env = get_phantom_env(worktree_name, &worktree_path.to_string_lossy());
    env.extend(phantom_env);

    let config = SpawnConfig {
        command: command.to_string(),
        args: args.to_vec(),
        cwd: Some(worktree_path.to_string_lossy().to_string()),
        env: Some(env),
        inherit_stdio: true,
        ..Default::default()
    };

    spawn_process(config).await
}

/// Spawn a shell in a specific directory
pub async fn spawn_shell_in_dir(dir: &Path) -> Result<SpawnSuccess> {
    let shell_info = detect_shell()?;
    info!("Spawning {} shell in directory: {}", shell_info.name, dir.display());

    let config = SpawnConfig {
        command: shell_info.path,
        args: shell_info.shell_type.init_args().iter().map(|s| s.to_string()).collect(),
        cwd: Some(dir.to_string_lossy().to_string()),
        inherit_stdio: true,
        ..Default::default()
    };

    spawn_process(config).await
}

/// Spawn a shell in a worktree
pub async fn spawn_shell_in_worktree(
    git_root: &Path,
    worktree_name: &str,
    filesystem: &dyn FileSystem,
) -> Result<SpawnSuccess> {
    // Validate worktree exists
    let validation = validate_worktree_exists(git_root, worktree_name, filesystem).await?;
    let worktree_path = validation.path;

    let shell_info = detect_shell()?;
    info!(
        "Spawning {} shell in worktree '{}' at {}",
        shell_info.name,
        worktree_name,
        worktree_path.display()
    );

    // Prepare environment with phantom variables
    let mut env: HashMap<String, String> = env::vars().collect();
    let phantom_env = get_phantom_env(worktree_name, &worktree_path.to_string_lossy());
    env.extend(phantom_env);

    // Add a custom prompt or greeting for the shell
    debug!("Shell type: {:?}", shell_info.shell_type);

    let config = SpawnConfig {
        command: shell_info.path,
        args: shell_info.shell_type.init_args().iter().map(|s| s.to_string()).collect(),
        cwd: Some(worktree_path.to_string_lossy().to_string()),
        env: Some(env),
        inherit_stdio: true,
        ..Default::default()
    };

    let result = spawn_process(config).await?;

    // Log exit
    info!("Shell exited with code {} for worktree '{}'", result.exit_code, worktree_name);

    Ok(result)
}

/// Execute multiple commands in sequence
pub async fn exec_commands_in_dir(dir: &Path, commands: &[String]) -> Result<Vec<SpawnSuccess>> {
    let mut results = Vec::new();

    for command in commands {
        info!("Executing command: {}", command);

        // Split command into program and arguments
        let parts: Vec<&str> = command.split_whitespace().collect();
        if parts.is_empty() {
            continue;
        }

        let program = parts[0];
        let args: Vec<String> = parts[1..].iter().map(|s| s.to_string()).collect();

        match exec_in_dir(dir, program, &args).await {
            Ok(result) => {
                if result.exit_code != 0 {
                    error!("Command '{}' failed with exit code {}", command, result.exit_code);
                    return Err(PhantomError::ProcessExecution(format!(
                        "Command '{}' failed with exit code {}",
                        command, result.exit_code
                    )));
                }
                results.push(result);
            }
            Err(e) => {
                error!("Failed to execute command '{}': {}", command, e);
                return Err(e);
            }
        }
    }

    Ok(results)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::filesystems::RealFileSystem;
    use crate::test_utils::TestRepo;
    use crate::worktree::create::create_worktree;
    use crate::worktree::types::CreateWorktreeOptions;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_exec_in_dir() {
        let temp_dir = TempDir::new().unwrap();
        let result = exec_in_dir(temp_dir.path(), "echo", &["hello".to_string()]).await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap().exit_code, 0);
    }

    #[tokio::test]
    async fn test_exec_in_dir_with_args() {
        let temp_dir = TempDir::new().unwrap();
        let result = exec_in_dir(temp_dir.path(), "ls", &["-la".to_string()]).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_exec_in_dir_failure() {
        let temp_dir = TempDir::new().unwrap();
        let result = exec_in_dir(temp_dir.path(), "false", &[]).await;

        assert!(result.is_ok());
        assert_ne!(result.unwrap().exit_code, 0);
    }

    #[tokio::test]
    async fn test_exec_commands_in_dir() {
        let temp_dir = TempDir::new().unwrap();
        let commands = vec!["echo hello".to_string(), "echo world".to_string()];

        let results = exec_commands_in_dir(temp_dir.path(), &commands).await;
        assert!(results.is_ok());

        let results = results.unwrap();
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].exit_code, 0);
        assert_eq!(results[1].exit_code, 0);
    }

    #[tokio::test]
    async fn test_exec_commands_in_dir_stops_on_failure() {
        let temp_dir = TempDir::new().unwrap();
        let commands = vec![
            "echo hello".to_string(),
            "false".to_string(),
            "echo world".to_string(), // This should not execute
        ];

        let results = exec_commands_in_dir(temp_dir.path(), &commands).await;
        assert!(results.is_err());
    }

    #[tokio::test]
    async fn test_exec_in_worktree() {
        let repo = TestRepo::new().await.unwrap();
        repo.create_file_and_commit("test.txt", "content", "Initial commit").await.unwrap();

        // Create a worktree
        let options = CreateWorktreeOptions::default();
        create_worktree(repo.path(), "test-worktree", options).await.unwrap();

        // Execute command in worktree
        let filesystem = RealFileSystem::new();
        let result = exec_in_worktree(
            repo.path(),
            "test-worktree",
            "echo",
            &["hello".to_string()],
            &filesystem,
        )
        .await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap().exit_code, 0);
    }

    #[tokio::test]
    async fn test_exec_in_nonexistent_worktree() {
        let repo = TestRepo::new().await.unwrap();

        let filesystem = RealFileSystem::new();
        let result = exec_in_worktree(
            repo.path(),
            "nonexistent",
            "echo",
            &["hello".to_string()],
            &filesystem,
        )
        .await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_spawn_shell_in_dir() {
        let temp_dir = TempDir::new().unwrap();
        // We can't easily test shell spawning without actually spawning a shell
        // which would hang waiting for user input. Just verify the function exists
        // and compiles correctly.
        assert!(temp_dir.path().exists());
        // Function signature verification
        let _ = spawn_shell_in_dir; // Just ensure it exists
    }

    #[tokio::test]
    async fn test_spawn_shell_in_worktree() {
        let repo = TestRepo::new().await.unwrap();
        repo.create_file_and_commit("test.txt", "content", "Initial commit").await.unwrap();

        // Create a worktree
        let options = CreateWorktreeOptions::default();
        create_worktree(repo.path(), "test-shell", options).await.unwrap();

        // We can't easily test shell spawning, but we can verify the function compiles
        let _ = spawn_shell_in_worktree; // Just ensure it exists
    }

    #[tokio::test]
    async fn test_exec_in_dir_with_cwd() {
        let temp_dir = TempDir::new().unwrap();
        let sub_dir = temp_dir.path().join("subdir");
        std::fs::create_dir(&sub_dir).unwrap();

        // Create a file in the subdirectory
        std::fs::write(sub_dir.join("test.txt"), "content").unwrap();

        // Execute command that should see the file
        let result = exec_in_dir(&sub_dir, "ls", &["test.txt".to_string()]).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().exit_code, 0);
    }

    #[tokio::test]
    async fn test_exec_in_worktree_with_env() {
        let repo = TestRepo::new().await.unwrap();
        repo.create_file_and_commit("test.txt", "content", "Initial commit").await.unwrap();

        // Create a worktree
        let options = CreateWorktreeOptions::default();
        create_worktree(repo.path(), "test-env", options).await.unwrap();

        // Execute a safe command that verifies env vars are set without exposing them
        // Use printenv to check specific PHANTOM vars only
        let filesystem = RealFileSystem::new();
        let result = exec_in_worktree(
            repo.path(),
            "test-env",
            "printenv",
            &["PHANTOM_WORKTREE".to_string()],
            &filesystem,
        )
        .await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().exit_code, 0);
    }

    #[tokio::test]
    async fn test_exec_commands_in_dir_empty_command() {
        let temp_dir = TempDir::new().unwrap();
        let commands = vec![
            "echo hello".to_string(),
            "".to_string(), // Empty command should be skipped
            "echo world".to_string(),
        ];

        let results = exec_commands_in_dir(temp_dir.path(), &commands).await;
        assert!(results.is_ok());
        let results = results.unwrap();
        assert_eq!(results.len(), 2); // Only 2 commands executed
    }

    #[tokio::test]
    async fn test_exec_commands_in_dir_complex_args() {
        let temp_dir = TempDir::new().unwrap();
        let commands = vec!["echo hello world".to_string(), "echo -n test".to_string()];

        let results = exec_commands_in_dir(temp_dir.path(), &commands).await;
        assert!(results.is_ok());
        let results = results.unwrap();
        assert_eq!(results.len(), 2);
        assert!(results.iter().all(|r| r.exit_code == 0));
    }

    #[tokio::test]
    async fn test_exec_in_dir_nonexistent_command() {
        let temp_dir = TempDir::new().unwrap();
        let result =
            exec_in_dir(temp_dir.path(), "nonexistent-command-xyz123", &["arg".to_string()]).await;

        assert!(result.is_err());
        match result.unwrap_err() {
            PhantomError::ProcessExecution(msg) => {
                assert!(msg.contains("Failed to spawn process"));
            }
            _ => panic!("Expected ProcessExecution error"),
        }
    }

    #[tokio::test]
    async fn test_exec_commands_in_dir_single_command() {
        let temp_dir = TempDir::new().unwrap();
        let commands = vec!["pwd".to_string()];

        let results = exec_commands_in_dir(temp_dir.path(), &commands).await;
        assert!(results.is_ok());
        let results = results.unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].exit_code, 0);
    }

    #[tokio::test]
    async fn test_exec_in_worktree_validates_worktree() {
        let repo = TestRepo::new().await.unwrap();
        repo.create_file_and_commit("test.txt", "content", "Initial commit").await.unwrap();

        // Try to execute in a worktree that doesn't exist
        let filesystem = RealFileSystem::new();
        let result = exec_in_worktree(
            repo.path(),
            "does-not-exist",
            "echo",
            &["test".to_string()],
            &filesystem,
        )
        .await;

        assert!(result.is_err());
        match result.unwrap_err() {
            PhantomError::WorktreeNotFound { name } => {
                assert_eq!(name, "does-not-exist");
            }
            PhantomError::Worktree(msg) => {
                // Also accept general worktree error
                assert!(msg.contains("does-not-exist") || msg.contains("not found"));
            }
            e => panic!("Expected WorktreeNotFound or Worktree error, got: {:?}", e),
        }
    }
}
