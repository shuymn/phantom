use async_trait::async_trait;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use crate::core::command_executor::{
    CommandConfig, CommandExecutor, CommandOutput, SpawnConfig, SpawnOutput,
};
use crate::core::error::PhantomError;
use crate::core::result::Result;

#[derive(Debug, Clone)]
pub struct CommandExpectation {
    pub program: String,
    pub args: Option<Vec<String>>,
    pub cwd: Option<PathBuf>,
    pub env: Option<HashMap<String, String>>,
    pub times: Option<usize>,
    pub returns: CommandOutput,
}

#[derive(Debug, Clone)]
pub struct CommandCall {
    pub program: String,
    pub args: Vec<String>,
    pub cwd: Option<PathBuf>,
    pub env: Option<HashMap<String, String>>,
}

#[derive(Debug, Clone)]
pub struct MockCommandExecutor {
    expectations: Arc<Mutex<Vec<CommandExpectation>>>,
    calls: Arc<Mutex<Vec<CommandCall>>>,
    spawn_expectations: Arc<Mutex<Vec<SpawnExpectation>>>,
    spawn_calls: Arc<Mutex<Vec<SpawnCall>>>,
}

#[derive(Debug, Clone)]
pub struct SpawnExpectation {
    pub program: String,
    pub args: Option<Vec<String>>,
    pub cwd: Option<PathBuf>,
    pub env: Option<HashMap<String, String>>,
    pub times: Option<usize>,
    pub returns: SpawnOutput,
}

#[derive(Debug, Clone)]
pub struct SpawnCall {
    pub program: String,
    pub args: Vec<String>,
    pub cwd: Option<PathBuf>,
    pub env: Option<HashMap<String, String>>,
}

impl MockCommandExecutor {
    pub fn new() -> Self {
        Self {
            expectations: Arc::new(Mutex::new(Vec::new())),
            calls: Arc::new(Mutex::new(Vec::new())),
            spawn_expectations: Arc::new(Mutex::new(Vec::new())),
            spawn_calls: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn expect_command(&mut self, program: &str) -> CommandExpectationBuilder {
        CommandExpectationBuilder::new(self.expectations.clone(), program)
    }

    pub fn expect_spawn(&mut self, program: &str) -> SpawnExpectationBuilder {
        SpawnExpectationBuilder::new(self.spawn_expectations.clone(), program)
    }

    pub fn verify(&self) -> Result<()> {
        let expectations = self.expectations.lock().unwrap();
        let calls = self.calls.lock().unwrap();

        for expectation in expectations.iter() {
            if let Some(expected_times) = expectation.times {
                let actual_calls =
                    calls.iter().filter(|call| self.matches_expectation(call, expectation)).count();

                if actual_calls != expected_times {
                    return Err(PhantomError::ProcessExecution(format!(
                        "Expected command '{}' to be called {} times, but was called {} times",
                        expectation.program, expected_times, actual_calls
                    )));
                }
            }
        }

        let spawn_expectations = self.spawn_expectations.lock().unwrap();
        let spawn_calls = self.spawn_calls.lock().unwrap();

        for expectation in spawn_expectations.iter() {
            if let Some(expected_times) = expectation.times {
                let actual_calls = spawn_calls
                    .iter()
                    .filter(|call| self.matches_spawn_expectation(call, expectation))
                    .count();

                if actual_calls != expected_times {
                    return Err(PhantomError::ProcessExecution(format!(
                        "Expected spawn '{}' to be called {} times, but was called {} times",
                        expectation.program, expected_times, actual_calls
                    )));
                }
            }
        }

        Ok(())
    }

    fn matches_expectation(&self, call: &CommandCall, expectation: &CommandExpectation) -> bool {
        if call.program != expectation.program {
            return false;
        }

        if let Some(ref expected_args) = expectation.args {
            if call.args != *expected_args {
                return false;
            }
        }

        if let Some(ref expected_cwd) = expectation.cwd {
            if call.cwd.as_ref() != Some(expected_cwd) {
                return false;
            }
        }

        if let Some(ref expected_env) = expectation.env {
            if call.env.as_ref() != Some(expected_env) {
                return false;
            }
        }

        true
    }

    fn matches_spawn_expectation(&self, call: &SpawnCall, expectation: &SpawnExpectation) -> bool {
        if call.program != expectation.program {
            return false;
        }

        if let Some(ref expected_args) = expectation.args {
            if call.args != *expected_args {
                return false;
            }
        }

        if let Some(ref expected_cwd) = expectation.cwd {
            if call.cwd.as_ref() != Some(expected_cwd) {
                return false;
            }
        }

        if let Some(ref expected_env) = expectation.env {
            if call.env.as_ref() != Some(expected_env) {
                return false;
            }
        }

        true
    }

    pub fn calls(&self) -> Vec<CommandCall> {
        self.calls.lock().unwrap().clone()
    }

    pub fn spawn_calls(&self) -> Vec<SpawnCall> {
        self.spawn_calls.lock().unwrap().clone()
    }
}

impl Default for MockCommandExecutor {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl CommandExecutor for MockCommandExecutor {
    async fn execute(&self, config: CommandConfig) -> Result<CommandOutput> {
        let call = CommandCall {
            program: config.program.clone(),
            args: config.args.clone(),
            cwd: config.cwd.clone(),
            env: config.env.clone(),
        };

        self.calls.lock().unwrap().push(call.clone());

        let expectations = self.expectations.lock().unwrap();
        for expectation in expectations.iter() {
            if self.matches_expectation(&call, expectation) {
                return Ok(expectation.returns.clone());
            }
        }

        Err(PhantomError::ProcessExecution(format!(
            "Unexpected command execution: {} {:?}",
            config.program, config.args
        )))
    }

    async fn spawn(&self, config: SpawnConfig) -> Result<SpawnOutput> {
        let call = SpawnCall {
            program: config.program.clone(),
            args: config.args.clone(),
            cwd: config.cwd.clone(),
            env: config.env.clone(),
        };

        self.spawn_calls.lock().unwrap().push(call.clone());

        let expectations = self.spawn_expectations.lock().unwrap();
        for expectation in expectations.iter() {
            if self.matches_spawn_expectation(&call, expectation) {
                return Ok(expectation.returns.clone());
            }
        }

        Err(PhantomError::ProcessExecution(format!(
            "Unexpected spawn: {} {:?}",
            config.program, config.args
        )))
    }
}

pub struct CommandExpectationBuilder {
    expectations: Arc<Mutex<Vec<CommandExpectation>>>,
    expectation: CommandExpectation,
}

impl CommandExpectationBuilder {
    fn new(expectations: Arc<Mutex<Vec<CommandExpectation>>>, program: &str) -> Self {
        Self {
            expectations,
            expectation: CommandExpectation {
                program: program.to_string(),
                args: None,
                cwd: None,
                env: None,
                times: None,
                returns: CommandOutput::new(String::new(), String::new(), 0),
            },
        }
    }

    pub fn with_args(mut self, args: &[&str]) -> Self {
        self.expectation.args = Some(args.iter().map(|s| s.to_string()).collect());
        self
    }

    pub fn in_dir(mut self, dir: impl Into<PathBuf>) -> Self {
        self.expectation.cwd = Some(dir.into());
        self
    }

    pub fn with_env(mut self, env: HashMap<String, String>) -> Self {
        self.expectation.env = Some(env);
        self
    }

    pub fn times(mut self, times: usize) -> Self {
        self.expectation.times = Some(times);
        self
    }

    pub fn returns_output(mut self, stdout: &str, stderr: &str, exit_code: i32) {
        self.expectation.returns =
            CommandOutput::new(stdout.to_string(), stderr.to_string(), exit_code);
        self.expectations.lock().unwrap().push(self.expectation);
    }

    pub fn returns_success(self) {
        self.returns_output("", "", 0)
    }

    pub fn returns_error(self, stderr: &str) {
        self.returns_output("", stderr, 1)
    }
}

pub struct SpawnExpectationBuilder {
    expectations: Arc<Mutex<Vec<SpawnExpectation>>>,
    expectation: SpawnExpectation,
}

impl SpawnExpectationBuilder {
    fn new(expectations: Arc<Mutex<Vec<SpawnExpectation>>>, program: &str) -> Self {
        Self {
            expectations,
            expectation: SpawnExpectation {
                program: program.to_string(),
                args: None,
                cwd: None,
                env: None,
                times: None,
                returns: SpawnOutput { pid: 12345 },
            },
        }
    }

    pub fn with_args(mut self, args: &[&str]) -> Self {
        self.expectation.args = Some(args.iter().map(|s| s.to_string()).collect());
        self
    }

    pub fn in_dir(mut self, dir: impl Into<PathBuf>) -> Self {
        self.expectation.cwd = Some(dir.into());
        self
    }

    pub fn with_env(mut self, env: HashMap<String, String>) -> Self {
        self.expectation.env = Some(env);
        self
    }

    pub fn times(mut self, times: usize) -> Self {
        self.expectation.times = Some(times);
        self
    }

    pub fn returns_pid(mut self, pid: u32) {
        self.expectation.returns = SpawnOutput { pid };
        self.expectations.lock().unwrap().push(self.expectation);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[tokio::test]
    async fn test_mock_command_success() {
        let mut mock = MockCommandExecutor::new();
        mock.expect_command("echo").with_args(&["hello", "world"]).times(1).returns_output(
            "hello world\n",
            "",
            0,
        );

        let config =
            CommandConfig::new("echo").with_args(vec!["hello".to_string(), "world".to_string()]);

        let result = mock.execute(config).await;
        assert!(result.is_ok());

        let output = result.unwrap();
        assert_eq!(output.stdout, "hello world\n");
        assert_eq!(output.stderr, "");
        assert_eq!(output.exit_code, 0);

        assert!(mock.verify().is_ok());
    }

    #[tokio::test]
    async fn test_mock_command_with_cwd() {
        let mut mock = MockCommandExecutor::new();
        let cwd = PathBuf::from("/tmp");

        mock.expect_command("ls").in_dir(&cwd).returns_output("file1\nfile2\n", "", 0);

        let config = CommandConfig::new("ls").with_cwd(cwd);

        let result = mock.execute(config).await;
        assert!(result.is_ok());

        let output = result.unwrap();
        assert_eq!(output.stdout, "file1\nfile2\n");
    }

    #[tokio::test]
    async fn test_mock_command_unexpected() {
        let mock = MockCommandExecutor::new();
        let config = CommandConfig::new("unexpected-command");

        let result = mock.execute(config).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Unexpected command execution"));
    }

    #[tokio::test]
    async fn test_mock_verify_times() {
        let mut mock = MockCommandExecutor::new();
        mock.expect_command("git").with_args(&["status"]).times(2).returns_success();

        let config = CommandConfig::new("git").with_args(vec!["status".to_string()]);

        mock.execute(config.clone()).await.unwrap();

        let verify_result = mock.verify();
        assert!(verify_result.is_err());
        assert!(verify_result
            .unwrap_err()
            .to_string()
            .contains("Expected command 'git' to be called 2 times, but was called 1 times"));

        mock.execute(config).await.unwrap();
        assert!(mock.verify().is_ok());
    }

    #[tokio::test]
    async fn test_mock_spawn_success() {
        let mut mock = MockCommandExecutor::new();
        mock.expect_spawn("sleep").with_args(&["1"]).returns_pid(9999);

        let config = SpawnConfig::new("sleep").with_args(vec!["1".to_string()]);

        let result = mock.spawn(config).await;
        assert!(result.is_ok());

        let output = result.unwrap();
        assert_eq!(output.pid, 9999);
    }

    #[tokio::test]
    async fn test_mock_with_env() {
        let mut mock = MockCommandExecutor::new();
        let mut env = HashMap::new();
        env.insert("TEST_VAR".to_string(), "test_value".to_string());

        mock.expect_command("sh")
            .with_args(&["-c", "echo $TEST_VAR"])
            .with_env(env.clone())
            .returns_output("test_value\n", "", 0);

        let config = CommandConfig::new("sh")
            .with_args(vec!["-c".to_string(), "echo $TEST_VAR".to_string()])
            .with_env(env);

        let result = mock.execute(config).await;
        assert!(result.is_ok());

        let output = result.unwrap();
        assert_eq!(output.stdout, "test_value\n");
    }

    #[tokio::test]
    async fn test_mock_returns_error() {
        let mut mock = MockCommandExecutor::new();
        mock.expect_command("false").returns_error("Command failed");

        let config = CommandConfig::new("false");

        let result = mock.execute(config).await;
        assert!(result.is_ok());

        let output = result.unwrap();
        assert_eq!(output.exit_code, 1);
        assert_eq!(output.stderr, "Command failed");
    }

    #[tokio::test]
    async fn test_mock_calls_tracking() {
        let mut mock = MockCommandExecutor::new();
        mock.expect_command("git").returns_success();

        let config1 = CommandConfig::new("git").with_args(vec!["status".to_string()]);
        let config2 = CommandConfig::new("git").with_args(vec!["log".to_string()]);

        mock.execute(config1).await.unwrap();
        mock.execute(config2).await.unwrap();

        let calls = mock.calls();
        assert_eq!(calls.len(), 2);
        assert_eq!(calls[0].args, vec!["status"]);
        assert_eq!(calls[1].args, vec!["log"]);
    }

    #[tokio::test]
    async fn test_default_impl() {
        let mock1 = MockCommandExecutor::new();
        let mock2 = MockCommandExecutor::default();

        assert_eq!(mock1.calls().len(), 0);
        assert_eq!(mock2.calls().len(), 0);
    }
}
