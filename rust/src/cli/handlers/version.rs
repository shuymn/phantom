use crate::cli::commands::version::VersionArgs;
use crate::cli::output::output;
use serde::Serialize;

#[derive(Serialize)]
struct VersionJsonOutput {
    name: String,
    version: String,
    description: String,
    authors: String,
}

/// Handle the version command
pub fn handle(args: VersionArgs) {
    let version = env!("CARGO_PKG_VERSION");
    let name = env!("CARGO_PKG_NAME");
    let authors = env!("CARGO_PKG_AUTHORS");
    let description = env!("CARGO_PKG_DESCRIPTION");

    if args.json {
        let json_output = VersionJsonOutput {
            name: name.to_string(),
            version: version.to_string(),
            description: description.to_string(),
            authors: authors.to_string(),
        };

        match serde_json::to_string_pretty(&json_output) {
            Ok(json) => output().log(&json),
            Err(e) => output().error(&format!("Failed to serialize JSON: {e}")),
        }
    } else {
        output().log(&format!("{name} {version}"));
        output().log(description);
        output().log(&format!("by {authors}"));
    }
}
