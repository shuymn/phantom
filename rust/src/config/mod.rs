pub mod default;
pub mod errors;
pub mod loader;
pub mod types;
pub mod validate;

// Re-export commonly used types
pub use default::{
    default_config, example_config, minimal_config, write_default_config, write_example_config,
    ConfigFormat,
};
pub use errors::ConfigError;
pub use loader::{
    find_config_file, load_config, load_config_from_file, CONFIG_FILE_NAME, TOML_CONFIG_FILE_NAME,
};
pub use types::{Multiplexer, PhantomConfig, PostCreateConfig};
pub use validate::validate_config;
