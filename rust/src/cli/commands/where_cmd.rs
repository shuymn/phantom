use clap::Args;
use serde::{Deserialize, Serialize};

#[derive(Args, Debug)]
pub struct WhereArgs {
    /// Name of the worktree
    pub name: String,

    /// Output in JSON format
    #[arg(long)]
    pub json: bool,
}

/// Result of where command for JSON output
#[derive(Serialize, Deserialize)]
pub struct WhereResult {
    pub success: bool,
    pub name: String,
    pub path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}
