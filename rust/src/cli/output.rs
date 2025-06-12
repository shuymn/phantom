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
