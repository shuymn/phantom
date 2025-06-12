use clap::Args;

#[derive(Args, Debug)]
pub struct ExecArgs {
    /// Name of the worktree (can be omitted with --fzf)
    pub name: Option<String>,

    /// Command and arguments to execute
    #[arg(trailing_var_arg = true, allow_hyphen_values = true, required = true)]
    pub command: Vec<String>,

    /// Select worktree interactively with fzf
    #[arg(long)]
    pub fzf: bool,

    /// Execute in a new tmux window
    #[arg(short = 't', long)]
    pub tmux: bool,

    /// Execute in a vertical tmux pane
    #[arg(long = "tmux-vertical", conflicts_with = "tmux")]
    pub tmux_vertical: bool,

    /// Execute in a vertical tmux pane (shorthand)
    #[arg(long = "tmux-v", conflicts_with_all = &["tmux", "tmux_vertical"])]
    pub tmux_v: bool,

    /// Execute in a horizontal tmux pane
    #[arg(long = "tmux-horizontal", conflicts_with_all = &["tmux", "tmux_vertical", "tmux_v"])]
    pub tmux_horizontal: bool,

    /// Execute in a horizontal tmux pane (shorthand)
    #[arg(long = "tmux-h", conflicts_with_all = &["tmux", "tmux_vertical", "tmux_v", "tmux_horizontal"])]
    pub tmux_h: bool,

    /// Execute in a new kitty tab
    #[arg(short = 'k', long, conflicts_with_all = &["tmux", "tmux_vertical", "tmux_v", "tmux_horizontal", "tmux_h"])]
    pub kitty: bool,

    /// Execute in a vertical kitty split
    #[arg(long = "kitty-vertical", conflicts_with_all = &["tmux", "tmux_vertical", "tmux_v", "tmux_horizontal", "tmux_h", "kitty"])]
    pub kitty_vertical: bool,

    /// Execute in a vertical kitty split (shorthand)
    #[arg(long = "kitty-v", conflicts_with_all = &["tmux", "tmux_vertical", "tmux_v", "tmux_horizontal", "tmux_h", "kitty", "kitty_vertical"])]
    pub kitty_v: bool,

    /// Execute in a horizontal kitty split
    #[arg(long = "kitty-horizontal", conflicts_with_all = &["tmux", "tmux_vertical", "tmux_v", "tmux_horizontal", "tmux_h", "kitty", "kitty_vertical", "kitty_v"])]
    pub kitty_horizontal: bool,

    /// Execute in a horizontal kitty split (shorthand)
    #[arg(long = "kitty-h", conflicts_with_all = &["tmux", "tmux_vertical", "tmux_v", "tmux_horizontal", "tmux_h", "kitty", "kitty_vertical", "kitty_v", "kitty_horizontal"])]
    pub kitty_h: bool,
}
