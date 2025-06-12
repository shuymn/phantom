use clap::Args;
use serde::{Deserialize, Serialize};

#[derive(Args, Debug)]
pub struct DeleteArgs {
    /// Name of the worktree to delete (optional if using --current or --fzf)
    pub name: Option<String>,

    /// Force deletion even if there are uncommitted changes
    #[arg(short, long)]
    pub force: bool,

    /// Delete the current worktree
    #[arg(long)]
    pub current: bool,

    /// Select worktree interactively with fzf
    #[arg(long)]
    pub fzf: bool,

    /// Output in JSON format
    #[arg(long)]
    pub json: bool,
}

/// Result of delete command for JSON output
#[derive(Serialize, Deserialize)]
pub struct DeleteResult {
    pub success: bool,
    pub name: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}
