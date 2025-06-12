use clap::Args;
use serde::{Deserialize, Serialize};

#[derive(Args, Debug)]
pub struct AttachArgs {
    /// Branch name to attach to
    pub branch: String,

    /// Name for the new worktree (defaults to branch name)
    #[arg(short, long)]
    pub name: Option<String>,

    /// Execute a command after attaching
    #[arg(short = 'x', long)]
    pub exec: Option<String>,

    /// Open a shell in the attached worktree
    #[arg(short, long)]
    pub shell: bool,

    /// Open in a new tmux window
    #[arg(short, long)]
    pub tmux: bool,

    /// Open in a vertical tmux split
    #[arg(long)]
    pub tmux_vertical: bool,

    /// Open in a horizontal tmux split
    #[arg(long)]
    pub tmux_horizontal: bool,

    /// Open in a new Kitty tab
    #[arg(long)]
    pub kitty: bool,

    /// Open in a vertical Kitty split
    #[arg(long)]
    pub kitty_vertical: bool,

    /// Open in a horizontal Kitty split
    #[arg(long)]
    pub kitty_horizontal: bool,

    /// Output in JSON format
    #[arg(long)]
    pub json: bool,
}

/// Result of attach command for JSON output
#[derive(Serialize, Deserialize)]
pub struct AttachResult {
    pub success: bool,
    pub name: String,
    pub branch: String,
    pub path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}
