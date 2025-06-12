use crate::process::tty::should_use_color;
use serde::Serialize;
use std::io::{self, Write};

/// Output handler for the CLI
pub struct Output {
    pub quiet: bool,
    pub verbose: bool,
    pub json: bool,
}

impl Output {
    /// Create a new output handler
    pub fn new(quiet: bool, verbose: bool, json: bool) -> Self {
        Self { quiet, verbose, json }
    }

    /// Print a normal message
    pub fn log(&self, message: &str) {
        if !self.quiet && !self.json {
            println!("{}", message);
        }
    }

    /// Print a verbose message
    pub fn debug(&self, message: &str) {
        if self.verbose && !self.quiet && !self.json {
            println!("{}", message);
        }
    }

    /// Print an error message
    pub fn error(&self, message: &str) {
        if !self.json {
            eprintln!("Error: {}", message);
        }
    }

    /// Print JSON output
    pub fn json<T: Serialize>(&self, data: &T) -> Result<(), serde_json::Error> {
        if self.json {
            println!("{}", serde_json::to_string_pretty(data)?);
        }
        Ok(())
    }

    /// Print a success message (green if color is enabled)
    pub fn success(&self, message: &str) {
        if !self.quiet && !self.json {
            if should_use_color() {
                println!("\x1b[32m{}\x1b[0m", message);
            } else {
                println!("{}", message);
            }
        }
    }

    /// Print a warning message (yellow if color is enabled)
    pub fn warn(&self, message: &str) {
        if !self.quiet && !self.json {
            if should_use_color() {
                eprintln!("\x1b[33mWarning: {}\x1b[0m", message);
            } else {
                eprintln!("Warning: {}", message);
            }
        }
    }

    /// Print without newline
    pub fn print(&self, message: &str) {
        if !self.quiet && !self.json {
            print!("{}", message);
            let _ = io::stdout().flush();
        }
    }
}

use std::sync::OnceLock;

/// Global output instance (to be set based on CLI flags)
static OUTPUT: OnceLock<Output> = OnceLock::new();

/// Initialize the global output handler
pub fn init_output(quiet: bool, verbose: bool, json: bool) {
    let _ = OUTPUT.set(Output::new(quiet, verbose, json));
}

/// Get the global output handler
pub fn output() -> &'static Output {
    OUTPUT.get().unwrap_or_else(|| {
        // Initialize with defaults if not set
        let _ = OUTPUT.set(Output::new(false, false, false));
        OUTPUT.get().unwrap()
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Serialize;

    #[derive(Serialize)]
    struct TestData {
        name: String,
        value: i32,
    }

    #[test]
    fn test_output_creation() {
        let output = Output::new(false, false, false);
        assert!(!output.quiet);
        assert!(!output.verbose);
        assert!(!output.json);

        let output = Output::new(true, true, true);
        assert!(output.quiet);
        assert!(output.verbose);
        assert!(output.json);
    }

    #[test]
    fn test_output_log() {
        // Normal output
        let output = Output::new(false, false, false);
        output.log("test message"); // Should print

        // Quiet mode
        let output = Output::new(true, false, false);
        output.log("test message"); // Should not print

        // JSON mode
        let output = Output::new(false, false, true);
        output.log("test message"); // Should not print
    }

    #[test]
    fn test_output_debug() {
        // Normal output
        let output = Output::new(false, false, false);
        output.debug("debug message"); // Should not print

        // Verbose mode
        let output = Output::new(false, true, false);
        output.debug("debug message"); // Should print

        // Quiet mode overrides verbose
        let output = Output::new(true, true, false);
        output.debug("debug message"); // Should not print

        // JSON mode
        let output = Output::new(false, true, true);
        output.debug("debug message"); // Should not print
    }

    #[test]
    fn test_output_error() {
        // Normal output
        let output = Output::new(false, false, false);
        output.error("error message"); // Should print to stderr

        // JSON mode
        let output = Output::new(false, false, true);
        output.error("error message"); // Should not print
    }

    #[test]
    fn test_output_json() {
        let data = TestData {
            name: "test".to_string(),
            value: 42,
        };

        // JSON mode
        let output = Output::new(false, false, true);
        let result = output.json(&data);
        assert!(result.is_ok());

        // Non-JSON mode
        let output = Output::new(false, false, false);
        let result = output.json(&data);
        assert!(result.is_ok()); // Should succeed but not print
    }

    #[test]
    fn test_output_success() {
        // Normal output
        let output = Output::new(false, false, false);
        output.success("success message"); // Should print

        // Quiet mode
        let output = Output::new(true, false, false);
        output.success("success message"); // Should not print

        // JSON mode
        let output = Output::new(false, false, true);
        output.success("success message"); // Should not print
    }

    #[test]
    fn test_output_warn() {
        // Normal output
        let output = Output::new(false, false, false);
        output.warn("warning message"); // Should print to stderr

        // Quiet mode
        let output = Output::new(true, false, false);
        output.warn("warning message"); // Should not print

        // JSON mode
        let output = Output::new(false, false, true);
        output.warn("warning message"); // Should not print
    }

    #[test]
    fn test_output_print() {
        // Normal output
        let output = Output::new(false, false, false);
        output.print("test"); // Should print without newline

        // Quiet mode
        let output = Output::new(true, false, false);
        output.print("test"); // Should not print

        // JSON mode
        let output = Output::new(false, false, true);
        output.print("test"); // Should not print
    }

    #[test]
    fn test_init_output() {
        // Test initialization
        init_output(false, true, false);
        let output = output();
        assert!(!output.quiet);
        assert!(output.verbose);
        assert!(!output.json);
    }

    #[test]
    fn test_output_singleton() {
        // Test that output() returns a singleton
        let output1 = output();
        let output2 = output();
        assert_eq!(output1 as *const _, output2 as *const _);
    }
}
