use clap::Args;

#[derive(Args, Debug)]
pub struct VersionArgs {
    /// Output in JSON format
    #[arg(long)]
    pub json: bool,
}
