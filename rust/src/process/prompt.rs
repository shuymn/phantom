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
pub fn select<T: AsRef<str>>(message: &str, options: &[T], default: Option<usize>) -> Result<usize> {
    println!("{}", message);
    
    for (i, option) in options.iter().enumerate() {
        let marker = if Some(i) == default { ">" } else { " " };
        println!("{} {}) {}", marker, i + 1, option.as_ref());
    }
    
    let default_display = default.map(|d| (d + 1).to_string());
    let input = prompt("Select an option:", default_display.as_deref())?;
    
    if input.is_empty() && default.is_some() {
        return Ok(default.unwrap());
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
}