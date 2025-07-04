use async_trait::async_trait;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use crate::core::command_executor::{CommandConfig, CommandExecutor, CommandOutput};
use crate::core::error::PhantomError;
use crate::core::result::Result;
use crate::core::sealed::Sealed;

#[derive(Debug, Clone)]
pub struct CommandExpectation {
    pub program: String,
    pub args: Option<Vec<String>>,
    pub cwd: Option<PathBuf>,
    pub env: Option<HashMap<String, String>>,
    pub stdin_data: Option<String>,
    pub times: Option<usize>,
    pub returns: CommandOutput,
}

#[derive(Debug, Clone)]
pub struct CommandCall {
    pub program: String,
    pub args: Vec<String>,
    pub cwd: Option<PathBuf>,
    pub env: Option<HashMap<String, String>>,
    pub stdin_data: Option<String>,
}

#[derive(Debug, Clone)]
pub struct MockCommandExecutor {
    expectations: Arc<Mutex<Vec<CommandExpectation>>>,
    calls: Arc<Mutex<Vec<CommandCall>>>,
}

impl MockCommandExecutor {
    pub fn new() -> Self {
        Self {
            expectations: Arc::new(Mutex::new(Vec::new())),
            calls: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn expect_command(&mut self, program: &str) -> CommandExpectationBuilder {
        CommandExpectationBuilder::new(self.expectations.clone(), program)
    }

    pub fn verify(&self) -> Result<()> {
        let expectations = self.expectations.lock().unwrap();
        let calls = self.calls.lock().unwrap();

        for expectation in expectations.iter() {
            if let Some(expected_times) = expectation.times {
                let actual_calls =
                    calls.iter().filter(|call| self.matches_expectation(call, expectation)).count();

                if actual_calls != expected_times {
                    return Err(PhantomError::ProcessExecutionError {
                        reason: format!(
                            "Expected command '{}' to be called {} times, but was called {} times",
                            expectation.program, expected_times, actual_calls
                        ),
                    });
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

        if let Some(ref expected_stdin) = expectation.stdin_data {
            if call.stdin_data.as_ref() != Some(expected_stdin) {
                return false;
            }
        }

        true
    }

    pub fn calls(&self) -> Vec<CommandCall> {
        self.calls.lock().unwrap().clone()
    }
}

impl Default for MockCommandExecutor {
    fn default() -> Self {
        Self::new()
    }
}

// Implement the sealed trait
impl Sealed for MockCommandExecutor {}

// Implement Sealed for &MockCommandExecutor
impl Sealed for &MockCommandExecutor {}

#[async_trait]
impl CommandExecutor for MockCommandExecutor {
    async fn execute(&self, config: CommandConfig) -> Result<CommandOutput> {
        let call = CommandCall {
            program: config.program.clone(),
            args: config.args.to_vec(),
            cwd: config.cwd.clone(),
            env: config.env.clone(),
            stdin_data: config.stdin_data.clone(),
        };

        self.calls.lock().unwrap().push(call.clone());

        let expectations = self.expectations.lock().unwrap();
        for expectation in expectations.iter() {
            if self.matches_expectation(&call, expectation) {
                return Ok(expectation.returns.clone());
            }
        }

        Err(PhantomError::ProcessExecutionError {
            reason: format!("Unexpected command execution: {} {:?}", config.program, config.args),
        })
    }
}

// Implement CommandExecutor for &MockCommandExecutor
#[async_trait]
impl CommandExecutor for &MockCommandExecutor {
    async fn execute(&self, config: CommandConfig) -> Result<CommandOutput> {
        (*self).execute(config).await
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
                stdin_data: None,
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

    pub fn with_stdin_data(mut self, stdin_data: &str) -> Self {
        self.expectation.stdin_data = Some(stdin_data.to_string());
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
