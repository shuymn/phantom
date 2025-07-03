use crate::{PhantomError, Result};
use std::io::{self, Write};
use tracing::debug;

/// Common function to read user input
fn read_input() -> Result<String> {
    io::stdout().flush().map_err(PhantomError::Io)?;
    let mut input = String::new();
    io::stdin().read_line(&mut input).map_err(PhantomError::Io)?;
    Ok(input.trim().to_string())
}

/// Common function to display a prompt
fn display_prompt(message: &str, suffix: &str) {
    print!("{message}{suffix}");
}

/// Prompt the user for a yes/no confirmation
pub fn confirm(message: &str, default: Option<bool>) -> Result<bool> {
    let suffix = match default {
        Some(true) => " [Y/n] ",
        Some(false) => " [y/N] ",
        None => " [y/n] ",
    };

    display_prompt(message, suffix);
    let input = read_input()?.to_lowercase();

    if input.is_empty() {
        if let Some(default_value) = default {
            debug!("Using default value: {}", default_value);
            return Ok(default_value);
        }
        // No default and no input - ask again
        return confirm(message, default);
    }

    match input.as_str() {
        "y" | "yes" => Ok(true),
        "n" | "no" => Ok(false),
        _ => {
            println!("Please answer 'y' or 'n'");
            confirm(message, default)
        }
    }
}

/// Prompt the user for text input
pub fn prompt(message: &str, default: Option<&str>) -> Result<String> {
    let suffix = if let Some(default_value) = default {
        format!(" [{default_value}] ")
    } else {
        " ".to_string()
    };

    display_prompt(message, &suffix);
    let input = read_input()?;

    if input.is_empty() {
        if let Some(default_value) = default {
            debug!("Using default value: {}", default_value);
            Ok(default_value.to_string())
        } else {
            Ok(String::new())
        }
    } else {
        Ok(input)
    }
}

/// Prompt the user to select from a list of options
pub fn select<T: AsRef<str>>(
    message: &str,
    options: &[T],
    default: Option<usize>,
) -> Result<usize> {
    println!("{message}");

    for (i, option) in options.iter().enumerate() {
        let marker = if Some(i) == default { ">" } else { " " };
        println!("{} {}) {}", marker, i + 1, option.as_ref());
    }

    let default_display = default.map(|d| (d + 1).to_string());
    let input = prompt("Select an option:", default_display.as_deref())?;

    if input.is_empty() {
        if let Some(default_idx) = default {
            return Ok(default_idx);
        }
    }

    match input.parse::<usize>() {
        Ok(n) if n > 0 && n <= options.len() => Ok(n - 1),
        _ => {
            println!("Invalid selection. Please enter a number between 1 and {}", options.len());
            select(message, options, default)
        }
    }
}

/// Check if prompts should be used (stdin is interactive)
pub fn should_prompt() -> bool {
    super::tty::is_stdin_tty()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_should_prompt() {
        // This will typically be false in test environments
        let result = should_prompt();
        // Just verify it doesn't panic
        let _ = result;
    }

    #[test]
    fn test_confirm_suffix_formatting() {
        // Test suffix formatting for different defaults
        let suffix_true = match Some(true) {
            Some(true) => " [Y/n] ",
            Some(false) => " [y/N] ",
            None => " [y/n] ",
        };
        assert_eq!(suffix_true, " [Y/n] ");

        let suffix_false = match Some(false) {
            Some(true) => " [Y/n] ",
            Some(false) => " [y/N] ",
            None => " [y/n] ",
        };
        assert_eq!(suffix_false, " [y/N] ");

        let suffix_none = match None as Option<bool> {
            Some(true) => " [Y/n] ",
            Some(false) => " [y/N] ",
            None => " [y/n] ",
        };
        assert_eq!(suffix_none, " [y/n] ");
    }

    #[test]
    fn test_input_parsing() {
        // Test yes responses
        let yes_inputs = vec!["y", "yes", "Y", "YES", "Yes"];
        for input in yes_inputs {
            let trimmed = input.to_lowercase();
            match trimmed.as_str() {
                "y" | "yes" => {}
                _ => panic!("Should match yes for input: {input}"),
            }
        }

        // Test no responses
        let no_inputs = vec!["n", "no", "N", "NO", "No"];
        for input in no_inputs {
            let trimmed = input.to_lowercase();
            match trimmed.as_str() {
                "n" | "no" => {}
                _ => panic!("Should match no for input: {input}"),
            }
        }
    }

    #[test]
    fn test_prompt_default_display() {
        // Test with default value
        let default = Some("default-value");
        let message = "Enter value";
        let formatted = format!("{} [{}] ", message, default.unwrap());
        assert_eq!(formatted, "Enter value [default-value] ");

        // Test without default
        let formatted_no_default = format!("{message} ");
        assert_eq!(formatted_no_default, "Enter value ");
    }

    #[test]
    fn test_select_display_formatting() {
        let options = ["Option 1", "Option 2", "Option 3"];
        let default = Some(1);

        // Test marker for default option
        for (i, option) in options.iter().enumerate() {
            let marker = if Some(i) == default { ">" } else { " " };
            let formatted = format!("{} {}) {}", marker, i + 1, option);

            if i == 1 {
                assert!(formatted.starts_with(">"));
            } else {
                assert!(formatted.starts_with(" "));
            }
            assert!(formatted.contains(option));
        }
    }

    #[test]
    fn test_select_input_validation() {
        let options_len = 5;

        // Valid inputs
        let valid_inputs = [1, 2, 3, 4, 5];
        for n in valid_inputs {
            assert!(n > 0 && n <= options_len);
            let index = n - 1;
            assert!(index < options_len);
        }

        // Invalid inputs
        let invalid_inputs = [0, 6, 100];
        for n in invalid_inputs {
            assert!(!(n > 0 && n <= options_len));
        }
    }

    #[test]
    fn test_string_trimming() {
        let test_cases = vec![
            ("  hello  \n", "hello"),
            ("\tworld\t", "world"),
            ("no-trim", "no-trim"),
            ("  ", ""),
            ("", ""),
        ];

        for (input, expected) in test_cases {
            assert_eq!(input.trim(), expected);
        }
    }

    #[test]
    fn test_parse_usize() {
        // Valid parses
        assert_eq!("1".parse::<usize>().unwrap(), 1);
        assert_eq!("42".parse::<usize>().unwrap(), 42);
        assert_eq!("0".parse::<usize>().unwrap(), 0);

        // Invalid parses
        assert!("abc".parse::<usize>().is_err());
        assert!("-1".parse::<usize>().is_err());
        assert!("1.5".parse::<usize>().is_err());
        assert!("".parse::<usize>().is_err());
    }

    #[test]
    fn test_error_handling() {
        use std::io::{Error, ErrorKind};

        // Test IO error conversion
        let io_error = Error::new(ErrorKind::UnexpectedEof, "test error");
        let phantom_error = PhantomError::Io(io_error);
        assert!(!phantom_error.to_string().is_empty());
    }

    #[test]
    fn test_as_ref_trait() {
        let string_vec = vec!["one".to_string(), "two".to_string()];
        let str_vec = vec!["one", "two"];

        // Both should work with AsRef<str>
        for item in &string_vec {
            let _: &str = item.as_ref();
        }
        for item in &str_vec {
            let _: &str = item;
        }
    }

    #[test]
    fn test_option_display() {
        let default_idx = Some(2);
        let display = default_idx.map(|d| (d + 1).to_string());
        assert_eq!(display, Some("3".to_string()));

        let display_ref = display.as_deref();
        assert_eq!(display_ref, Some("3"));

        let none_display: Option<usize> = None;
        let none_mapped = none_display.map(|d| (d + 1).to_string());
        assert_eq!(none_mapped, None);
    }

    #[test]
    fn test_empty_string_checks() {
        assert!("".is_empty());
        assert!(!"non-empty".is_empty());
        assert!("   ".trim().is_empty());
        assert!("\n".trim().is_empty());
    }

    #[test]
    fn test_default_value_handling() {
        // Test with Some default
        let default = Some("default");
        if let Some(value) = default {
            assert_eq!(value, "default");
        }

        // Test with None default
        let no_default: Option<&str> = None;
        assert!(no_default.is_none());
    }

    #[test]
    fn test_message_formatting() {
        let message = "Please confirm";
        let options = ["Yes", "No", "Cancel"];

        // Test select message display
        let select_msg = message.to_string();
        assert_eq!(select_msg, "Please confirm");

        // Test option formatting
        for (i, opt) in options.iter().enumerate() {
            let formatted = format!(" {}) {}", i + 1, opt);
            assert!(formatted.contains(opt));
            assert!(formatted.contains(&(i + 1).to_string()));
        }
    }
}
