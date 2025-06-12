use phantom::cli::commands::completion::{CompletionArgs, Shell};
use phantom::cli::handlers::completion;

#[test]
fn test_completion_command_fish() {
    let args = CompletionArgs { shell: Shell::Fish };
    let result = completion::handle(args);
    assert!(result.is_ok(), "Fish completion generation failed");
}

#[test]
fn test_completion_command_zsh() {
    let args = CompletionArgs { shell: Shell::Zsh };
    let result = completion::handle(args);
    assert!(result.is_ok(), "Zsh completion generation failed");
}

#[test]
fn test_completion_command_bash() {
    let args = CompletionArgs { shell: Shell::Bash };
    let result = completion::handle(args);
    assert!(result.is_ok(), "Bash completion generation failed");
}
