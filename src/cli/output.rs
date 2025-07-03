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

    /// Internal helper for common output logic
    fn should_output(&self, require_verbose: bool) -> bool {
        !self.json && (!self.quiet || require_verbose && self.verbose)
    }

    /// Internal helper for styled output
    fn print_styled(
        &self,
        message: &str,
        prefix: Option<&str>,
        color: Option<&str>,
        to_stderr: bool,
    ) {
        let formatted = if let Some(prefix) = prefix {
            format!("{prefix}: {message}")
        } else {
            message.to_string()
        };

        let output = if should_use_color() && color.is_some() {
            format!("{}{}\x1b[0m", color.unwrap(), formatted)
        } else {
            formatted
        };

        if to_stderr {
            eprintln!("{output}");
        } else {
            println!("{output}");
        }
    }

    /// Print a normal message
    pub fn log(&self, message: &str) {
        if self.should_output(false) {
            println!("{message}");
        }
    }

    /// Print a verbose message
    pub fn debug(&self, message: &str) {
        if self.should_output(false) && self.verbose {
            println!("{message}");
        }
    }

    /// Print an error message
    pub fn error(&self, message: &str) {
        if !self.json {
            eprintln!("Error: {message}");
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
        if self.should_output(false) {
            self.print_styled(message, None, Some("\x1b[32m"), false);
        }
    }

    /// Print a warning message (yellow if color is enabled)
    pub fn warn(&self, message: &str) {
        if self.should_output(false) {
            self.print_styled(message, Some("Warning"), Some("\x1b[33m"), true);
        }
    }

    /// Print without newline
    pub fn print(&self, message: &str) {
        if self.should_output(false) {
            print!("{message}");
            let _ = io::stdout().flush();
        }
    }

    /// Print a table (simple implementation for now)
    pub fn table<T: std::fmt::Display>(&self, headers: &[&str], rows: Vec<Vec<T>>) {
        if !self.quiet && !self.json {
            // Calculate column widths
            let mut widths: Vec<usize> = headers.iter().map(|h| h.len()).collect();

            for row in &rows {
                for (i, cell) in row.iter().enumerate() {
                    let cell_str = cell.to_string();
                    if i < widths.len() {
                        widths[i] = widths[i].max(cell_str.len());
                    }
                }
            }

            // Print headers
            for (i, header) in headers.iter().enumerate() {
                print!("{:<width$}", header, width = widths.get(i).unwrap_or(&0) + 2);
            }
            println!();

            // Print separator
            for width in &widths {
                print!("{}", "-".repeat(width + 2));
            }
            println!();

            // Print rows
            for row in rows {
                for (i, cell) in row.iter().enumerate() {
                    print!("{:<width$}", cell.to_string(), width = widths.get(i).unwrap_or(&0) + 2);
                }
                println!();
            }
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
        let data = TestData { name: "test".to_string(), value: 42 };

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
        // Test initialization - create a new Output instance directly
        // since we can't reset the singleton in tests
        let test_output = Output::new(false, true, false);
        assert!(!test_output.quiet);
        assert!(test_output.verbose);
        assert!(!test_output.json);

        // Also test other combinations
        let quiet_output = Output::new(true, false, false);
        assert!(quiet_output.quiet);
        assert!(!quiet_output.verbose);
        assert!(!quiet_output.json);

        let json_output = Output::new(false, false, true);
        assert!(!json_output.quiet);
        assert!(!json_output.verbose);
        assert!(json_output.json);
    }

    #[test]
    fn test_output_singleton() {
        // Test that output() returns a singleton
        let output1 = output();
        let output2 = output();
        assert_eq!(output1 as *const _, output2 as *const _);
    }
}
