use serde::{Deserialize, Serialize};

/// Main configuration structure for Phantom
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct PhantomConfig {
    /// Post-create actions to perform after creating a worktree
    #[serde(skip_serializing_if = "Option::is_none")]
    pub post_create: Option<PostCreateConfig>,

    /// Default terminal multiplexer to use
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_multiplexer: Option<Multiplexer>,
}

/// Post-create configuration
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct PostCreateConfig {
    /// Files to copy from the original worktree
    #[serde(skip_serializing_if = "Option::is_none")]
    pub copy_files: Option<Vec<String>>,

    /// Commands to run after creating the worktree
    #[serde(skip_serializing_if = "Option::is_none")]
    pub commands: Option<Vec<String>>,
}

/// Supported terminal multiplexers
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Multiplexer {
    Tmux,
    Kitty,
    #[default]
    None,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serialize_config() {
        let config = PhantomConfig {
            post_create: Some(PostCreateConfig {
                copy_files: Some(vec![".env".to_string(), "config.local.json".to_string()]),
                commands: Some(vec!["npm install".to_string()]),
            }),
            default_multiplexer: Some(Multiplexer::Tmux),
        };

        let json = serde_json::to_string_pretty(&config).unwrap();
        assert!(json.contains("\"postCreate\""));
        assert!(json.contains("\"copyFiles\""));
        assert!(json.contains("\"commands\""));
        assert!(json.contains("\"defaultMultiplexer\""));
        assert!(json.contains("\"tmux\""));
    }

    #[test]
    fn test_deserialize_config() {
        let json = r#"{
            "postCreate": {
                "copyFiles": [".env"],
                "commands": ["npm install"]
            },
            "defaultMultiplexer": "kitty"
        }"#;

        let config: PhantomConfig = serde_json::from_str(json).unwrap();
        assert!(config.post_create.is_some());

        let post_create = config.post_create.unwrap();
        assert_eq!(post_create.copy_files.as_ref().unwrap().len(), 1);
        assert_eq!(post_create.copy_files.unwrap()[0], ".env");
        assert_eq!(post_create.commands.as_ref().unwrap().len(), 1);
        assert_eq!(post_create.commands.unwrap()[0], "npm install");

        assert_eq!(config.default_multiplexer, Some(Multiplexer::Kitty));
    }

    #[test]
    fn test_deserialize_minimal_config() {
        let json = "{}";
        let config: PhantomConfig = serde_json::from_str(json).unwrap();
        assert!(config.post_create.is_none());
        assert!(config.default_multiplexer.is_none());
    }

    #[test]
    fn test_deserialize_multiplexer() {
        assert_eq!(serde_json::from_str::<Multiplexer>("\"tmux\"").unwrap(), Multiplexer::Tmux);
        assert_eq!(serde_json::from_str::<Multiplexer>("\"kitty\"").unwrap(), Multiplexer::Kitty);
        assert_eq!(serde_json::from_str::<Multiplexer>("\"none\"").unwrap(), Multiplexer::None);
    }

    #[test]
    fn test_skip_none_fields() {
        let config = PhantomConfig::default();
        let json = serde_json::to_string(&config).unwrap();
        assert_eq!(json, "{}");
    }
}
