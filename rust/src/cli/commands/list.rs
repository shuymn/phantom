use clap::Args;

#[derive(Args, Debug)]
pub struct ListArgs {
    /// Use fzf for interactive selection
    #[arg(long)]
    pub fzf: bool,

    /// Show only names (no formatting)
    #[arg(long)]
    pub names: bool,
}
