use clap::Args;
use serde::{Deserialize, Serialize};

#[derive(Args, Debug)]
pub struct CreateArgs {
    /// Name of the worktree to create
    pub name: String,

    /// Branch name (defaults to worktree name)
    #[arg(short, long)]
    pub branch: Option<String>,

    /// Execute a command after creating the worktree
    #[arg(short = 'x', long)]
    pub exec: Option<String>,

    /// Open a shell in the new worktree
    #[arg(short, long)]
    pub shell: bool,

    /// Open in a new tmux window
    #[arg(short, long)]
    pub tmux: bool,

    /// Open in a vertical tmux split
    #[arg(long)]
    pub tmux_vertical: bool,

    /// Alias for --tmux-vertical
    #[arg(long = "tmux-v", hide = true)]
    pub tmux_v: bool,

    /// Open in a horizontal tmux split
    #[arg(long)]
    pub tmux_horizontal: bool,

    /// Alias for --tmux-horizontal
    #[arg(long = "tmux-h", hide = true)]
    pub tmux_h: bool,

    /// Open in a new Kitty tab
    #[arg(long)]
    pub kitty: bool,

    /// Open in a vertical Kitty split
    #[arg(long)]
    pub kitty_vertical: bool,

    /// Alias for --kitty-vertical
    #[arg(long = "kitty-v", hide = true)]
    pub kitty_v: bool,

    /// Open in a horizontal Kitty split
    #[arg(long)]
    pub kitty_horizontal: bool,

    /// Alias for --kitty-horizontal
    #[arg(long = "kitty-h", hide = true)]
    pub kitty_h: bool,

    /// Files to copy from the current worktree
    #[arg(long = "copy", value_delimiter = ',')]
    pub copy_files: Option<Vec<String>>,

    /// Base ref for the new branch (commit/branch/tag)
    #[arg(long)]
    pub base: Option<String>,

    /// Output in JSON format
    #[arg(long)]
    pub json: bool,
}

/// Result of create command for JSON output
#[derive(Serialize, Deserialize)]
pub struct CreateResult {
    pub success: bool,
    pub name: String,
    pub branch: String,
    pub path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub copied_files: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}
