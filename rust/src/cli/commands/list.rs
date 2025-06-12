use clap::Args;
use serde::{Deserialize, Serialize};

#[derive(Args, Debug)]
pub struct ListArgs {
    /// Show paths instead of names
    #[arg(short, long)]
    pub paths: bool,

    /// Output in JSON format
    #[arg(long)]
    pub json: bool,
}

/// Worktree information for list output
#[derive(Serialize, Deserialize)]
pub struct WorktreeInfo {
    pub name: String,
    pub path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub branch: Option<String>,
    pub is_bare: bool,
    pub is_detached: bool,
    pub is_locked: bool,
    pub is_prunable: bool,
}
