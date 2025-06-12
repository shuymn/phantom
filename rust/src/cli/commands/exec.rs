use clap::Args;

#[derive(Args, Debug)]
pub struct ExecArgs {
    /// Name of the worktree
    pub name: String,

    /// Command and arguments to execute
    #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
    pub command: Vec<String>,
}
