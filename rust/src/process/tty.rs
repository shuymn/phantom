use std::env;
use std::io::{self, IsTerminal};

/// Check if stdin is a TTY
pub fn is_stdin_tty() -> bool {
    io::stdin().is_terminal()
}

/// Check if stdout is a TTY
pub fn is_stdout_tty() -> bool {
    io::stdout().is_terminal()
}

/// Check if stderr is a TTY
pub fn is_stderr_tty() -> bool {
    io::stderr().is_terminal()
}

/// Check if we're in an interactive terminal session
pub fn is_interactive() -> bool {
    is_stdin_tty() && is_stdout_tty()
}

/// Check if color output should be enabled
pub fn should_use_color() -> bool {
    // Check NO_COLOR first (it has the highest precedence)
    if env::var("NO_COLOR").is_ok() {
        return false;
    }

    // Check FORCE_COLOR
    if env::var("FORCE_COLOR").is_ok() {
        return true;
    }

    // Check TERM
    if let Ok(term) = env::var("TERM") {
        if term == "dumb" {
            return false;
        }
    }

    // Default to using color if stdout is a TTY
    is_stdout_tty()
}

/// Get terminal width
pub fn terminal_width() -> Option<usize> {
    terminal_size::terminal_size().map(|(terminal_size::Width(w), _)| w as usize)
}

/// Get terminal height
pub fn terminal_height() -> Option<usize> {
    terminal_size::terminal_size().map(|(_, terminal_size::Height(h))| h as usize)
}

/// Get terminal size (width, height)
pub fn terminal_size() -> Option<(usize, usize)> {
    terminal_size::terminal_size()
        .map(|(terminal_size::Width(w), terminal_size::Height(h))| (w as usize, h as usize))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tty_detection() {
        // These tests will have different results depending on the test environment
        let stdin_tty = is_stdin_tty();
        let stdout_tty = is_stdout_tty();
        let stderr_tty = is_stderr_tty();

        // In most CI environments, these will be false
        println!("stdin is tty: {}", stdin_tty);
        println!("stdout is tty: {}", stdout_tty);
        println!("stderr is tty: {}", stderr_tty);
    }

    #[test]
    fn test_is_interactive() {
        let interactive = is_interactive();
        // Should be false in test environment
        println!("Is interactive: {}", interactive);
    }

    #[test]
    fn test_should_use_color() {
        // Save original env vars
        let force_color = env::var("FORCE_COLOR").ok();
        let no_color = env::var("NO_COLOR").ok();

        // Test FORCE_COLOR
        env::set_var("FORCE_COLOR", "1");
        assert!(should_use_color());

        // Test NO_COLOR (should override FORCE_COLOR)
        env::set_var("NO_COLOR", "1");
        assert!(!should_use_color());

        // Restore original env vars
        if let Some(val) = force_color {
            env::set_var("FORCE_COLOR", val);
        } else {
            env::remove_var("FORCE_COLOR");
        }

        if let Some(val) = no_color {
            env::set_var("NO_COLOR", val);
        } else {
            env::remove_var("NO_COLOR");
        }
    }

    #[test]
    fn test_terminal_size() {
        // This will depend on the terminal
        if let Some((width, height)) = terminal_size() {
            println!("Terminal size: {}x{}", width, height);
            assert!(width > 0);
            assert!(height > 0);
        } else {
            println!("Unable to get terminal size");
        }
    }

    #[test]
    fn test_terminal_width_height() {
        // Test individual width/height functions
        let width = terminal_width();
        let height = terminal_height();
        
        if width.is_some() && height.is_some() {
            assert!(width.unwrap() > 0);
            assert!(height.unwrap() > 0);
        }
        
        // If we can get full size, individual functions should work too
        if let Some((w, h)) = terminal_size() {
            assert_eq!(width, Some(w));
            assert_eq!(height, Some(h));
        }
    }

    #[test]
    fn test_should_use_color_with_term() {
        // Save original env vars
        let orig_term = env::var("TERM").ok();
        let orig_no_color = env::var("NO_COLOR").ok();
        let orig_force_color = env::var("FORCE_COLOR").ok();
        
        // Clean environment
        env::remove_var("NO_COLOR");
        env::remove_var("FORCE_COLOR");
        
        // Test with dumb terminal
        env::set_var("TERM", "dumb");
        assert!(!should_use_color());
        
        // Test with normal terminal
        env::set_var("TERM", "xterm-256color");
        // Result depends on if stdout is a TTY
        let _ = should_use_color();
        
        // Restore original env vars
        match orig_term {
            Some(val) => env::set_var("TERM", val),
            None => env::remove_var("TERM"),
        }
        match orig_no_color {
            Some(val) => env::set_var("NO_COLOR", val),
            None => env::remove_var("NO_COLOR"),
        }
        match orig_force_color {
            Some(val) => env::set_var("FORCE_COLOR", val),
            None => env::remove_var("FORCE_COLOR"),
        }
    }

    #[test]
    fn test_all_tty_functions() {
        // Just ensure all functions can be called without panic
        let _ = is_stdin_tty();
        let _ = is_stdout_tty();
        let _ = is_stderr_tty();
        let _ = is_interactive();
        let _ = should_use_color();
        let _ = terminal_width();
        let _ = terminal_height();
        let _ = terminal_size();
    }
}
