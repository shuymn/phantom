use clap::Args;

#[derive(Args, Debug)]
pub struct ShellArgs {
    /// Name of the worktree (optional if using --fzf)
    pub name: Option<String>,

    /// Select worktree interactively with fzf
    #[arg(long)]
    pub fzf: bool,

    /// Open in a new tmux window
    #[arg(short = 't', long)]
    pub tmux: bool,

    /// Open in a vertical tmux pane
    #[arg(long = "tmux-vertical", conflicts_with = "tmux")]
    pub tmux_vertical: bool,

    /// Open in a vertical tmux pane (shorthand)
    #[arg(long = "tmux-v", conflicts_with_all = &["tmux", "tmux_vertical"])]
    pub tmux_v: bool,

    /// Open in a horizontal tmux pane
    #[arg(long = "tmux-horizontal", conflicts_with_all = &["tmux", "tmux_vertical", "tmux_v"])]
    pub tmux_horizontal: bool,

    /// Open in a horizontal tmux pane (shorthand)
    #[arg(long = "tmux-h", conflicts_with_all = &["tmux", "tmux_vertical", "tmux_v", "tmux_horizontal"])]
    pub tmux_h: bool,

    /// Open in a new kitty tab
    #[arg(short = 'k', long, conflicts_with_all = &["tmux", "tmux_vertical", "tmux_v", "tmux_horizontal", "tmux_h"])]
    pub kitty: bool,

    /// Open in a vertical kitty split
    #[arg(long = "kitty-vertical", conflicts_with_all = &["tmux", "tmux_vertical", "tmux_v", "tmux_horizontal", "tmux_h", "kitty"])]
    pub kitty_vertical: bool,

    /// Open in a vertical kitty split (shorthand)
    #[arg(long = "kitty-v", conflicts_with_all = &["tmux", "tmux_vertical", "tmux_v", "tmux_horizontal", "tmux_h", "kitty", "kitty_vertical"])]
    pub kitty_v: bool,

    /// Open in a horizontal kitty split
    #[arg(long = "kitty-horizontal", conflicts_with_all = &["tmux", "tmux_vertical", "tmux_v", "tmux_horizontal", "tmux_h", "kitty", "kitty_vertical", "kitty_v"])]
    pub kitty_horizontal: bool,

    /// Open in a horizontal kitty split (shorthand)
    #[arg(long = "kitty-h", conflicts_with_all = &["tmux", "tmux_vertical", "tmux_v", "tmux_horizontal", "tmux_h", "kitty", "kitty_vertical", "kitty_v", "kitty_horizontal"])]
    pub kitty_h: bool,
}
