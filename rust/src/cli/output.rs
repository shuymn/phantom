use std::io::{self, Write};

/// Print to stdout
pub fn println(message: &str) {
    println!("{}", message);
}

/// Print to stderr
pub fn eprintln(message: &str) {
    eprintln!("{}", message);
}

/// Print without newline
pub fn print(message: &str) {
    print!("{}", message);
    let _ = io::stdout().flush();
}
