use clap::{Args, ValueEnum};

#[derive(Args, Debug)]
pub struct CompletionArgs {
    /// Shell to generate completions for
    pub shell: Shell,
}

#[derive(Debug, Clone, ValueEnum)]
pub enum Shell {
    /// Bash shell
    Bash,
    /// Fish shell
    Fish,
    /// Zsh shell
    Zsh,
    /// PowerShell
    PowerShell,
    /// Elvish shell
    Elvish,
}
