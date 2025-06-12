use phantom::cli::commands::version::VersionArgs;
use phantom::cli::handlers::version;

#[test]
fn test_version_command_basic() {
    let args = VersionArgs { json: false };

    // This just ensures it doesn't panic
    version::handle(args);
}

#[test]
fn test_version_command_json() {
    let args = VersionArgs { json: true };

    // This just ensures it doesn't panic
    version::handle(args);
}
