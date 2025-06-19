use async_trait::async_trait;
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
    pub stdout: String,
    pub stderr: String,
    pub exit_code: i32,
}

impl CommandOutput {
    pub fn success(&self) -> bool {
        self.exit_code == 0
    }

    pub fn new(stdout: String, stderr: String, exit_code: i32) -> Self {
        Self { stdout, stderr, exit_code }
    }
}

#[derive(Debug, Clone)]
pub struct SpawnConfig {
    pub program: String,
    pub args: Vec<String>,
    pub cwd: Option<PathBuf>,
    pub env: Option<HashMap<String, String>>,
    pub stdin: bool,
    pub stdout: bool,
    pub stderr: bool,
}

impl SpawnConfig {
    pub fn new(program: impl Into<String>) -> Self {
        Self {
            program: program.into(),
            args: Vec::new(),
            cwd: None,
            env: None,
            stdin: false,
            stdout: true,
            stderr: true,
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

    pub fn with_stdin(mut self, stdin: bool) -> Self {
        self.stdin = stdin;
        self
    }

    pub fn with_stdout(mut self, stdout: bool) -> Self {
        self.stdout = stdout;
        self
    }

    pub fn with_stderr(mut self, stderr: bool) -> Self {
        self.stderr = stderr;
        self
    }
}

#[derive(Debug, Clone)]
pub struct SpawnOutput {
    pub pid: u32,
}

#[async_trait]
pub trait CommandExecutor: Send + Sync {
    async fn execute(&self, config: CommandConfig) -> Result<CommandOutput>;
    async fn spawn(&self, config: SpawnConfig) -> Result<SpawnOutput>;
}
