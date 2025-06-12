use crate::cli::output::output;

/// Handle the version command
pub fn handle() {
    let version = env!("CARGO_PKG_VERSION");
    let name = env!("CARGO_PKG_NAME");
    let authors = env!("CARGO_PKG_AUTHORS");
    let description = env!("CARGO_PKG_DESCRIPTION");

    output().log(&format!("{} {}", name, version));
    output().log(description);
    output().log(&format!("by {}", authors));
}
