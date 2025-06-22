use crate::core::command_executor::{CommandArgs, CommandExecutor};
use crate::{PhantomError, Result};
use smallvec::smallvec;
use tracing::{debug, error};

/// Options for FZF selection
#[derive(Debug, Clone, Default)]
pub struct FzfOptions {
    pub prompt: Option<String>,
    pub header: Option<String>,
    pub preview_command: Option<String>,
}

/// Select an item from a list using fzf with CommandExecutor
pub async fn select_with_fzf<E>(
    executor: &E,
    items: Vec<String>,
    options: FzfOptions,
) -> Result<Option<String>>
where
    E: CommandExecutor,
{
    debug!("Starting fzf selection with {} items", items.len());

    if items.is_empty() {
        debug!("No items to select from");
        return Ok(None);
    }

    let mut args: CommandArgs = smallvec![];

    // Add options
    if let Some(prompt) = &options.prompt {
        args.push("--prompt".to_string());
        args.push(prompt.clone());
    }

    if let Some(header) = &options.header {
        args.push("--header".to_string());
        args.push(header.clone());
    }

    if let Some(preview_command) = &options.preview_command {
        args.push("--preview".to_string());
        args.push(preview_command.clone());
    }

    // Join items with newlines for stdin
    let stdin_data = items.join("\n");

    // Execute fzf with stdin data
    let config = crate::core::command_executor::CommandConfig::new("fzf")
        .with_args_smallvec(args)
        .with_stdin_data(stdin_data);

    match executor.execute(config).await {
        Ok(output) => {
            match output.exit_code {
                0 => {
                    // Success - user selected an item
                    let selected = output.stdout.trim().to_string();
                    Ok(if selected.is_empty() { None } else { Some(selected) })
                }
                1 => {
                    // No match found
                    debug!("No match found in fzf");
                    Ok(None)
                }
                2 => {
                    // Error
                    error!("fzf returned an error: {}", output.stderr);
                    Err(PhantomError::ProcessExecutionError {
                        reason: format!("fzf error: {}", output.stderr),
                    })
                }
                130 => {
                    // User cancelled (Ctrl+C)
                    debug!("User cancelled fzf selection");
                    Ok(None)
                }
                _ => {
                    error!("fzf exited with unexpected code: {}", output.exit_code);
                    Err(PhantomError::ProcessFailed {
                        command: "fzf".to_string(),
                        code: output.exit_code,
                    })
                }
            }
        }
        Err(e) => {
            if e.to_string().contains("command not found") || e.to_string().contains("No such file")
            {
                Err(PhantomError::CommandNotFound { command: "fzf".to_string() })
            } else {
                Err(e)
            }
        }
    }
}

/// Check if fzf is available in the system with CommandExecutor
pub async fn is_fzf_available<E>(executor: &E) -> bool
where
    E: CommandExecutor,
{
    use crate::core::command_executor::CommandConfig;

    let config = CommandConfig::new("fzf").with_args_smallvec(smallvec!["--version".to_string()]);

    match executor.execute(config).await {
        Ok(output) => output.exit_code == 0,
        Err(_) => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::executors::MockCommandExecutor;

    #[tokio::test]
    async fn test_is_fzf_available_with_executor_true() {
        let mut mock = MockCommandExecutor::new();
        mock.expect_command("fzf").with_args(&["--version"]).returns_output(
            "0.42.0 (d471067)",
            "",
            0,
        );

        let result = is_fzf_available(&mock).await;
        assert!(result);
    }

    #[tokio::test]
    async fn test_is_fzf_available_with_executor_false() {
        let mut mock = MockCommandExecutor::new();
        mock.expect_command("fzf").with_args(&["--version"]).returns_error("command not found");

        let result = is_fzf_available(&mock).await;
        assert!(!result);
    }

    #[test]
    fn test_fzf_options_default() {
        let options = FzfOptions::default();
        assert!(options.prompt.is_none());
        assert!(options.header.is_none());
        assert!(options.preview_command.is_none());
    }

    #[test]
    fn test_fzf_options_with_values() {
        let options = FzfOptions {
            prompt: Some("Select an item: ".to_string()),
            header: Some("Available items".to_string()),
            preview_command: Some("echo {}".to_string()),
        };

        assert_eq!(options.prompt.unwrap(), "Select an item: ");
        assert_eq!(options.header.unwrap(), "Available items");
        assert_eq!(options.preview_command.unwrap(), "echo {}");
    }

    #[tokio::test]
    async fn test_select_with_fzf_with_executor_empty_items() {
        let mock = MockCommandExecutor::new();
        let result = select_with_fzf(&mock, vec![], FzfOptions::default()).await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[tokio::test]
    async fn test_select_with_fzf_with_executor_returns_first() {
        let mut mock = MockCommandExecutor::new();
        let items = vec!["item1".to_string(), "item2".to_string()];

        // Set up expectation for fzf command
        mock.expect_command("fzf").with_stdin_data("item1\nitem2").returns_output("item1\n", "", 0);

        let result = select_with_fzf(&mock, items, FzfOptions::default()).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Some("item1".to_string()));
    }

    #[test]
    fn test_fzf_options_debug() {
        let options =
            FzfOptions { prompt: Some("test".to_string()), header: None, preview_command: None };

        let debug_str = format!("{:?}", options);
        assert!(debug_str.contains("FzfOptions"));
        assert!(debug_str.contains("prompt: Some"));
    }

    #[test]
    fn test_fzf_options_clone() {
        let options = FzfOptions {
            prompt: Some("test".to_string()),
            header: Some("header".to_string()),
            preview_command: Some("preview".to_string()),
        };

        let cloned = options.clone();
        assert_eq!(options.prompt, cloned.prompt);
        assert_eq!(options.header, cloned.header);
        assert_eq!(options.preview_command, cloned.preview_command);
    }

    #[tokio::test]
    async fn test_select_with_fzf_single_item() {
        let mut mock = MockCommandExecutor::new();
        let items = vec!["single-item".to_string()];

        // Expect fzf to be called with the single item as stdin
        mock.expect_command("fzf").with_stdin_data("single-item").returns_output(
            "single-item\n",
            "",
            0,
        );

        let result = select_with_fzf(&mock, items, FzfOptions::default()).await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Some("single-item".to_string()));
    }

    #[tokio::test]
    async fn test_select_with_fzf_with_options() {
        let mut mock = MockCommandExecutor::new();
        let items = vec!["item1".to_string(), "item2".to_string()];
        let options = FzfOptions {
            prompt: Some("Choose: ".to_string()),
            header: Some("Items".to_string()),
            preview_command: Some("echo preview: {}".to_string()),
        };

        // Expect fzf to be called with correct options and stdin
        mock.expect_command("fzf")
            .with_args(&[
                "--prompt",
                "Choose: ",
                "--header",
                "Items",
                "--preview",
                "echo preview: {}",
            ])
            .with_stdin_data("item1\nitem2")
            .returns_output("item2\n", "", 0);

        let result = select_with_fzf(&mock, items, options).await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Some("item2".to_string()));
    }

    #[tokio::test]
    async fn test_select_with_fzf_user_cancelled() {
        let mut mock = MockCommandExecutor::new();
        let items = vec!["item1".to_string(), "item2".to_string()];

        // Simulate user pressing Ctrl+C (exit code 130)
        mock.expect_command("fzf").with_stdin_data("item1\nitem2").returns_output("", "", 130);

        let result = select_with_fzf(&mock, items, FzfOptions::default()).await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), None);
    }

    #[tokio::test]
    async fn test_select_with_fzf_no_match() {
        let mut mock = MockCommandExecutor::new();
        let items = vec!["item1".to_string(), "item2".to_string()];

        // Simulate no match found (exit code 1)
        mock.expect_command("fzf").with_stdin_data("item1\nitem2").returns_output("", "", 1);

        let result = select_with_fzf(&mock, items, FzfOptions::default()).await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), None);
    }

    #[test]
    fn test_error_handling() {
        // Test error creation and formatting
        let io_error = std::io::Error::new(std::io::ErrorKind::NotFound, "test error");
        let phantom_error = PhantomError::Io(io_error);
        assert!(!phantom_error.to_string().is_empty());

        let exec_error = PhantomError::CommandNotFound { command: "fzf".to_string() };
        assert!(exec_error.to_string().contains("Command 'fzf' not found"));

        let exit_error = PhantomError::ProcessFailed { command: "fzf".to_string(), code: 2 };
        assert!(exit_error.to_string().contains("exited with code"));

        let signal_error =
            PhantomError::ProcessExecutionError { reason: "fzf terminated by signal".to_string() };
        assert!(signal_error.to_string().contains("Process execution failed"));
    }

    #[test]
    fn test_string_formatting() {
        let items = vec!["first".to_string(), "second".to_string(), "third".to_string()];
        let joined = items.join("\n");
        assert_eq!(joined, "first\nsecond\nthird");
        assert_eq!(joined.as_bytes().len(), 18); // 5 + 1 + 6 + 1 + 5

        // Test trimming
        let with_whitespace = "  selected item  \n";
        assert_eq!(with_whitespace.trim(), "selected item");

        // Test empty string
        let empty = "";
        assert!(empty.is_empty());
        assert_eq!(empty.trim(), "");
    }

    #[test]
    fn test_utf8_handling() {
        let valid_utf8 = b"valid UTF-8 string";
        let result = String::from_utf8_lossy(valid_utf8);
        assert_eq!(result, "valid UTF-8 string");

        // Test with non-ASCII UTF-8
        let unicode = "hello 世界".as_bytes();
        let result = String::from_utf8_lossy(unicode);
        assert_eq!(result, "hello 世界");
    }

    #[test]
    fn test_exit_codes() {
        // Test all expected exit codes
        let exit_codes = vec![
            (0, "Success"),
            (1, "No match"),
            (130, "User cancelled"),
            (2, "Error"),
            (255, "Other error"),
        ];

        for (code, description) in exit_codes {
            match code {
                0 => assert_eq!(description, "Success"),
                1 => assert_eq!(description, "No match"),
                130 => assert_eq!(description, "User cancelled"),
                _ => assert!(description.contains("error") || description.contains("Error")),
            }
        }
    }

    #[test]
    fn test_io_error_kinds() {
        use std::io::ErrorKind;

        let not_found = std::io::Error::new(ErrorKind::NotFound, "not found");
        assert_eq!(not_found.kind(), ErrorKind::NotFound);

        let permission = std::io::Error::new(ErrorKind::PermissionDenied, "denied");
        assert_eq!(permission.kind(), ErrorKind::PermissionDenied);
    }

    #[tokio::test]
    async fn test_select_with_fzf_multiple_items() {
        let mut mock = MockCommandExecutor::new();
        let items =
            vec!["option-one".to_string(), "option-two".to_string(), "option-three".to_string()];

        // Simulate user selecting the second option
        mock.expect_command("fzf")
            .with_stdin_data("option-one\noption-two\noption-three")
            .returns_output("option-two\n", "", 0);

        let result = select_with_fzf(&mock, items.clone(), FzfOptions::default()).await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Some("option-two".to_string()));

        // Verify the input would be formatted correctly
        let expected_input = items.join("\n");
        assert_eq!(expected_input, "option-one\noption-two\noption-three");
    }

    #[test]
    fn test_fzf_command_args() {
        use std::process::Command;
        let mut cmd = Command::new("fzf");

        // Test adding prompt
        cmd.arg("--prompt").arg("Select: ");

        // Test adding header
        cmd.arg("--header").arg("Choose an option");

        // Test adding preview
        cmd.arg("--preview").arg("cat {}");

        // Just verify we can build the command
        assert_eq!(cmd.get_program(), "fzf");
    }
}
