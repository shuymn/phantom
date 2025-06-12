use crate::Result;
use std::collections::HashMap;
use std::env;
use std::path::Path;
use tracing::{debug, info};

/// Detected shell information
#[derive(Debug, Clone, PartialEq)]
pub struct ShellInfo {
    pub name: String,
    pub path: String,
    pub shell_type: ShellType,
}

/// Types of shells we support
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ShellType {
    Bash,
    Zsh,
    Fish,
    Sh,
    Unknown,
}

impl ShellType {
    /// Get the appropriate RC file for this shell type
    pub fn rc_file(&self) -> Option<&'static str> {
        match self {
            ShellType::Bash => Some(".bashrc"),
            ShellType::Zsh => Some(".zshrc"),
            ShellType::Fish => Some(".config/fish/config.fish"),
            ShellType::Sh => Some(".profile"),
            ShellType::Unknown => None,
        }
    }

    /// Get shell-specific initialization arguments
    pub fn init_args(&self) -> Vec<&'static str> {
        match self {
            ShellType::Bash => vec!["-i"], // Interactive
            ShellType::Zsh => vec!["-i"],  // Interactive
            ShellType::Fish => vec!["-i"], // Interactive
            ShellType::Sh => vec![],       // No special args for sh
            ShellType::Unknown => vec![],
        }
    }
}

/// Detect the current shell
pub fn detect_shell() -> Result<ShellInfo> {
    // First, try the SHELL environment variable
    if let Ok(shell_path) = env::var("SHELL") {
        if let Some(shell_info) = analyze_shell_path(&shell_path) {
            debug!("Detected shell from $SHELL: {:?}", shell_info);
            return Ok(shell_info);
        }
    }

    // Fallback: try to detect from parent process
    if let Some(shell_info) = detect_from_parent_process() {
        debug!("Detected shell from parent process: {:?}", shell_info);
        return Ok(shell_info);
    }

    // Ultimate fallback: use /bin/sh
    info!("Could not detect shell, falling back to /bin/sh");
    Ok(ShellInfo { name: "sh".to_string(), path: "/bin/sh".to_string(), shell_type: ShellType::Sh })
}

/// Analyze a shell path and determine its type
fn analyze_shell_path(path: &str) -> Option<ShellInfo> {
    let path_lower = path.to_lowercase();
    let name = Path::new(path).file_name()?.to_str()?;

    let shell_type = if path_lower.contains("bash") || name == "bash" {
        ShellType::Bash
    } else if path_lower.contains("zsh") || name == "zsh" {
        ShellType::Zsh
    } else if path_lower.contains("fish") || name == "fish" {
        ShellType::Fish
    } else if name == "sh" {
        ShellType::Sh
    } else {
        ShellType::Unknown
    };

    Some(ShellInfo { name: name.to_string(), path: path.to_string(), shell_type })
}

/// Try to detect shell from parent process (Unix-specific)
#[cfg(unix)]
fn detect_from_parent_process() -> Option<ShellInfo> {
    use std::fs;

    let ppid = get_parent_pid()?;
    let cmdline_path = format!("/proc/{}/cmdline", ppid);

    if let Ok(cmdline) = fs::read_to_string(&cmdline_path) {
        let args: Vec<&str> = cmdline.split('\0').collect();
        if let Some(cmd) = args.first() {
            return analyze_shell_path(cmd);
        }
    }

    None
}

#[cfg(not(unix))]
fn detect_from_parent_process() -> Option<ShellInfo> {
    None
}

/// Get parent process ID (Unix-specific)
#[cfg(unix)]
fn get_parent_pid() -> Option<u32> {
    use std::fs;

    let stat_path = format!("/proc/{}/stat", std::process::id());
    if let Ok(stat) = fs::read_to_string(&stat_path) {
        // The parent PID is the 4th field in /proc/PID/stat
        let parts: Vec<&str> = stat.split_whitespace().collect();
        if parts.len() > 3 {
            return parts[3].parse().ok();
        }
    }

    None
}

/// Get environment variables for a phantom session
pub fn get_phantom_env(worktree_name: &str, worktree_path: &str) -> HashMap<String, String> {
    let mut env = HashMap::new();

    // Set phantom-specific environment variables
    env.insert("PHANTOM_WORKTREE".to_string(), worktree_name.to_string());
    env.insert("PHANTOM_WORKTREE_PATH".to_string(), worktree_path.to_string());
    env.insert("PHANTOM_ACTIVE".to_string(), "1".to_string());

    // Update prompt if PS1 is set
    if let Ok(ps1) = env::var("PS1") {
        let phantom_ps1 = format!("(phantom:{}) {}", worktree_name, ps1);
        env.insert("PS1".to_string(), phantom_ps1);
    }

    env
}

/// Check if we're currently in a phantom session
pub fn is_phantom_session() -> bool {
    env::var("PHANTOM_ACTIVE").is_ok()
}

/// Get the current phantom worktree name if in a session
pub fn current_phantom_worktree() -> Option<String> {
    env::var("PHANTOM_WORKTREE").ok()
}

/// Open an interactive shell in a directory
pub async fn shell_in_dir(dir: &Path) -> Result<()> {
    let shell_info = detect_shell()?;
    let worktree_name = dir.file_name().and_then(|n| n.to_str()).unwrap_or("phantom");

    let env_vars = get_phantom_env(worktree_name, &dir.to_string_lossy());

    info!("Opening shell in: {}", dir.display());

    let config = super::spawn::SpawnConfig {
        command: shell_info.path.clone(),
        args: shell_info.shell_type.init_args().iter().map(|s| s.to_string()).collect(),
        cwd: Some(dir.to_string_lossy().to_string()),
        env: Some(env_vars),
        inherit_stdio: true,
        timeout_ms: None,
    };

    super::spawn::spawn_process(config).await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_analyze_shell_path() {
        assert_eq!(analyze_shell_path("/bin/bash").unwrap().shell_type, ShellType::Bash);
        assert_eq!(analyze_shell_path("/usr/bin/zsh").unwrap().shell_type, ShellType::Zsh);
        assert_eq!(analyze_shell_path("/usr/local/bin/fish").unwrap().shell_type, ShellType::Fish);
        assert_eq!(analyze_shell_path("/bin/sh").unwrap().shell_type, ShellType::Sh);
        assert_eq!(analyze_shell_path("/usr/bin/ksh").unwrap().shell_type, ShellType::Unknown);
    }

    #[test]
    fn test_shell_type_rc_file() {
        assert_eq!(ShellType::Bash.rc_file(), Some(".bashrc"));
        assert_eq!(ShellType::Zsh.rc_file(), Some(".zshrc"));
        assert_eq!(ShellType::Fish.rc_file(), Some(".config/fish/config.fish"));
        assert_eq!(ShellType::Sh.rc_file(), Some(".profile"));
        assert_eq!(ShellType::Unknown.rc_file(), None);
    }

    #[test]
    fn test_shell_type_init_args() {
        assert_eq!(ShellType::Bash.init_args(), vec!["-i"]);
        assert_eq!(ShellType::Zsh.init_args(), vec!["-i"]);
        assert_eq!(ShellType::Fish.init_args(), vec!["-i"]);
        assert_eq!(ShellType::Sh.init_args(), vec![] as Vec<&str>);
    }

    #[test]
    fn test_get_phantom_env() {
        let env = get_phantom_env("feature-branch", "/path/to/worktree");

        assert_eq!(env.get("PHANTOM_WORKTREE").unwrap(), "feature-branch");
        assert_eq!(env.get("PHANTOM_WORKTREE_PATH").unwrap(), "/path/to/worktree");
        assert_eq!(env.get("PHANTOM_ACTIVE").unwrap(), "1");
    }

    #[test]
    fn test_detect_shell() {
        // This test should always pass since we have a fallback
        let result = detect_shell();
        assert!(result.is_ok());

        let shell_info = result.unwrap();
        assert!(!shell_info.path.is_empty());
        assert!(!shell_info.name.is_empty());
    }

    #[test]
    fn test_is_phantom_session() {
        // Should be false in test environment
        assert!(!is_phantom_session());
    }

    #[test]
    fn test_current_phantom_worktree() {
        // Should be None in test environment
        assert!(current_phantom_worktree().is_none());
    }
}
