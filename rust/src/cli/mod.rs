pub mod commands;
pub mod error;
pub mod handlers;
pub mod output;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "phantom",
    version = env!("CARGO_PKG_VERSION"),
    about = "Ephemeral Git worktrees made easy",
    long_about = "Phantom is a CLI tool that makes working with Git worktrees simple and efficient.\n\
                  It helps you manage multiple worktrees, switch between them, and maintain a clean\n\
                  development environment."
)]
#[command(propagate_version = true)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    /// Enable verbose output
    #[arg(short, long, global = true)]
    pub verbose: bool,

    /// Suppress all output except errors
    #[arg(short, long, global = true)]
    pub quiet: bool,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Create a new Git worktree (phantom)
    Create(commands::create::CreateArgs),

    /// Attach to an existing branch by creating a new worktree
    Attach(commands::attach::AttachArgs),

    /// List all Git worktrees (phantoms)
    List(commands::list::ListArgs),

    /// Output the filesystem path of a specific worktree
    Where(commands::where_cmd::WhereArgs),

    /// Delete a Git worktree (phantom)
    Delete(commands::delete::DeleteArgs),

    /// Execute a command in a worktree directory
    Exec(commands::exec::ExecArgs),

    /// Open an interactive shell in a worktree directory
    Shell(commands::shell::ShellArgs),

    /// Display phantom version information
    Version(commands::version::VersionArgs),

    /// Generate shell completion scripts
    Completion(commands::completion::CompletionArgs),
}
