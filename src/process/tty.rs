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
    use crate::test_utils::EnvGuard;

    #[test]
    fn test_tty_detection() {
        // These tests will have different results depending on the test environment
        let stdin_tty = is_stdin_tty();
        let stdout_tty = is_stdout_tty();
        let stderr_tty = is_stderr_tty();

        // In most CI environments, these will be false
        println!("stdin is tty: {stdin_tty}");
        println!("stdout is tty: {stdout_tty}");
        println!("stderr is tty: {stderr_tty}");
    }

    #[test]
    fn test_is_interactive() {
        let interactive = is_interactive();
        // Should be false in test environment
        println!("Is interactive: {interactive}");
    }

    #[test]
    fn test_should_use_color() {
        // Test FORCE_COLOR
        let _guard1 = EnvGuard::set("FORCE_COLOR", "1");
        assert!(should_use_color());

        // Test NO_COLOR (should override FORCE_COLOR)
        let _guard2 = EnvGuard::set("NO_COLOR", "1");
        assert!(!should_use_color());

        // Guards will automatically restore env vars when dropped
    }

    #[test]
    fn test_terminal_size() {
        // This will depend on the terminal
        if let Some((width, height)) = terminal_size() {
            println!("Terminal size: {width}x{height}");
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
        // Clean environment
        let _guard1 = EnvGuard::remove("NO_COLOR");
        let _guard2 = EnvGuard::remove("FORCE_COLOR");

        // Test with dumb terminal
        let _guard3 = EnvGuard::set("TERM", "dumb");
        assert!(!should_use_color());
        drop(_guard3);

        // Test with normal terminal
        let _guard4 = EnvGuard::set("TERM", "xterm-256color");
        // Result depends on if stdout is a TTY
        let _ = should_use_color();

        // Guards will automatically restore env vars when dropped
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
