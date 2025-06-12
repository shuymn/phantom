use clap::Args;

#[derive(Args, Debug)]
pub struct ShellArgs {
    /// Name of the worktree (optional if using --fzf)
    pub name: Option<String>,

    /// Select worktree interactively with fzf
    #[arg(long)]
    pub fzf: bool,
}
