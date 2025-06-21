use async_trait::async_trait;
use std::borrow::Cow;
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Duration;

use crate::core::result::Result;

#[derive(Debug, Clone)]
pub struct CommandConfig {
    pub program: String,
    pub args: Vec<String>,
    pub cwd: Option<PathBuf>,
    pub env: Option<HashMap<String, String>>,
    pub timeout: Option<Duration>,
    pub stdin_data: Option<String>,
}

impl CommandConfig {
    pub fn new(program: impl Into<String>) -> Self {
        Self {
            program: program.into(),
            args: Vec::new(),
            cwd: None,
            env: None,
            timeout: None,
            stdin_data: None,
        }
    }

    pub fn with_args(mut self, args: Vec<String>) -> Self {
        self.args = args;
        self
    }

    pub fn with_cwd(mut self, cwd: PathBuf) -> Self {
        self.cwd = Some(cwd);
        self
    }

    pub fn with_env(mut self, env: HashMap<String, String>) -> Self {
        self.env = Some(env);
        self
    }

    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }

    pub fn with_stdin_data(mut self, stdin_data: String) -> Self {
        self.stdin_data = Some(stdin_data);
        self
    }
}

#[derive(Debug, Clone)]
pub struct CommandOutput {
    pub stdout: Cow<'static, str>,
    pub stderr: Cow<'static, str>,
    pub exit_code: i32,
}

impl CommandOutput {
    pub fn success(&self) -> bool {
        self.exit_code == 0
    }

    /// Create a new CommandOutput with owned strings (default)
    pub fn new(stdout: String, stderr: String, exit_code: i32) -> Self {
        Self { stdout: Cow::Owned(stdout), stderr: Cow::Owned(stderr), exit_code }
    }

    /// Create from static string references (zero-copy)
    pub fn from_static(stdout: &'static str, stderr: &'static str, exit_code: i32) -> Self {
        Self { stdout: Cow::Borrowed(stdout), stderr: Cow::Borrowed(stderr), exit_code }
    }

    /// Get stdout as &str without allocation
    pub fn stdout_str(&self) -> &str {
        &self.stdout
    }

    /// Get stderr as &str without allocation
    pub fn stderr_str(&self) -> &str {
        &self.stderr
    }

    /// Convert to owned strings if needed
    pub fn into_owned(self) -> (String, String, i32) {
        (self.stdout.into_owned(), self.stderr.into_owned(), self.exit_code)
    }
}

#[async_trait]
pub trait CommandExecutor: Send + Sync {
    async fn execute(&self, config: CommandConfig) -> Result<CommandOutput>;
}
