use crate::{PhantomError, Result};
use std::io::{self, Write};
use tracing::debug;

/// Prompt the user for a yes/no confirmation
pub fn confirm(message: &str, default: Option<bool>) -> Result<bool> {
    let suffix = match default {
        Some(true) => " [Y/n] ",
        Some(false) => " [y/N] ",
        None => " [y/n] ",
    };

    print!("{}{}", message, suffix);
    io::stdout().flush().map_err(PhantomError::Io)?;

    let mut input = String::new();
    io::stdin().read_line(&mut input).map_err(PhantomError::Io)?;

    let input = input.trim().to_lowercase();

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
    if let Some(default_value) = default {
        print!("{} [{}] ", message, default_value);
    } else {
        print!("{} ", message);
    }
    io::stdout().flush().map_err(PhantomError::Io)?;

    let mut input = String::new();
    io::stdin().read_line(&mut input).map_err(PhantomError::Io)?;

    let input = input.trim();

    if input.is_empty() {
        if let Some(default_value) = default {
            debug!("Using default value: {}", default_value);
            Ok(default_value.to_string())
        } else {
            Ok(String::new())
        }
    } else {
        Ok(input.to_string())
    }
}

/// Prompt the user to select from a list of options
pub fn select<T: AsRef<str>>(
    message: &str,
    options: &[T],
    default: Option<usize>,
) -> Result<usize> {
    println!("{}", message);

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
        // In test environment, stdin is usually not a TTY
        let should = should_prompt();
        println!("Should prompt in test environment: {}", should);
    }

    // Note: Testing interactive prompts is difficult because they require stdin
    // input. These would be better tested with integration tests that can
    // simulate user input. For unit tests, we can at least verify that
    // the code compiles and basic validation logic works.

    #[test]
    fn test_prompt_validation() {
        // Test that prompt returns expected types
        // Actual behavior would require mocking stdin
        
        // We can at least ensure the functions exist and compile
        let _confirm_fn: fn(&str, Option<bool>) -> Result<bool> = confirm;
        let _prompt_fn: fn(&str, Option<&str>) -> Result<String> = prompt;
        // Note: select is generic over T: AsRef<str>, so we can't assign it to a specific fn pointer
    }

    #[test]
    fn test_select_with_options() {
        // Test that select works with different types that implement AsRef<str>
        let string_options = vec!["Option 1".to_string(), "Option 2".to_string()];
        let str_options = vec!["Option A", "Option B"];
        
        // Verify that the function can be called with both types
        // (actual testing would require stdin mock)
        let _select_string = |msg, opts, def| select::<String>(msg, opts, def);
        let _select_str = |msg, opts, def| select::<&str>(msg, opts, def);
        
        // Ensure these are the right types
        assert_eq!(string_options.len(), 2);
        assert_eq!(str_options.len(), 2);
    }
}
