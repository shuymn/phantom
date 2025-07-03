use clap::Args;

#[derive(Args, Debug)]
pub struct AttachArgs {
    /// Branch name to attach to
    pub branch: String,

    /// Execute a command after attaching
    #[arg(short = 'e', long = "exec")]
    pub exec: Option<String>,

    /// Open a shell in the attached worktree
    #[arg(short = 's', long)]
    pub shell: bool,

    /// Output in JSON format
    #[arg(long)]
    pub json: bool,
}
