use crate::cli::commands::completion::{CompletionArgs, Shell};
use crate::Result;

const FISH_COMPLETION: &str = r#"# Fish completion for phantom
# Place this in ~/.config/fish/completions/phantom.fish

function __phantom_list_worktrees
    phantom list --names 2>/dev/null
end

function __phantom_using_command
    set -l cmd (commandline -opc)
    set -l cmd_count (count $cmd)
    if test $cmd_count -eq 1
        # No subcommand yet, so any command can be used
        if test (count $argv) -eq 0
            return 0
        else
            return 1
        end
    else if test $cmd_count -ge 2
        # Check if we're in the context of a specific command
        if test (count $argv) -gt 0 -a "$argv[1]" = "$cmd[2]"
            return 0
        end
    end
    return 1
end

# Disable file completion for phantom
complete -c phantom -f

# Main commands
complete -c phantom -n "__phantom_using_command" -a "create" -d "Create a new Git worktree (phantom)"
complete -c phantom -n "__phantom_using_command" -a "attach" -d "Attach to an existing branch by creating a new worktree"
complete -c phantom -n "__phantom_using_command" -a "list" -d "List all Git worktrees (phantoms)"
complete -c phantom -n "__phantom_using_command" -a "where" -d "Output the filesystem path of a specific worktree"
complete -c phantom -n "__phantom_using_command" -a "delete" -d "Delete a Git worktree (phantom)"
complete -c phantom -n "__phantom_using_command" -a "exec" -d "Execute a command in a worktree directory"
complete -c phantom -n "__phantom_using_command" -a "shell" -d "Open an interactive shell in a worktree directory"
complete -c phantom -n "__phantom_using_command" -a "version" -d "Display phantom version information"
complete -c phantom -n "__phantom_using_command" -a "completion" -d "Generate shell completion scripts"

# Global options
complete -c phantom -l help -d "Show help (-h)"
complete -c phantom -l version -d "Show version (-v)"

# create command options
complete -c phantom -n "__phantom_using_command create" -l shell -d "Open an interactive shell in the new worktree after creation (-s)"
complete -c phantom -n "__phantom_using_command create" -l exec -d "Execute a command in the new worktree after creation (-x)" -x
complete -c phantom -n "__phantom_using_command create" -l tmux -d "Open the worktree in a new tmux window (-t)"
complete -c phantom -n "__phantom_using_command create" -l tmux-vertical -d "Open the worktree in a vertical tmux pane"
complete -c phantom -n "__phantom_using_command create" -l tmux-horizontal -d "Open the worktree in a horizontal tmux pane"
complete -c phantom -n "__phantom_using_command create" -l copy-file -d "Copy specified files from the current worktree" -r

# attach command options
complete -c phantom -n "__phantom_using_command attach" -l shell -d "Open an interactive shell in the worktree after attaching (-s)"
complete -c phantom -n "__phantom_using_command attach" -l exec -d "Execute a command in the worktree after attaching (-x)" -x

# list command options
complete -c phantom -n "__phantom_using_command list" -l fzf -d "Use fzf for interactive selection"
complete -c phantom -n "__phantom_using_command list" -l names -d "Output only phantom names (for scripts and completion)"

# where command options
complete -c phantom -n "__phantom_using_command where" -l fzf -d "Use fzf for interactive selection"
complete -c phantom -n "__phantom_using_command where" -a "(__phantom_list_worktrees)"

# delete command options
complete -c phantom -n "__phantom_using_command delete" -l force -d "Force deletion even if worktree has uncommitted changes (-f)"
complete -c phantom -n "__phantom_using_command delete" -l current -d "Delete the current worktree"
complete -c phantom -n "__phantom_using_command delete" -l fzf -d "Use fzf for interactive selection"
complete -c phantom -n "__phantom_using_command delete" -a "(__phantom_list_worktrees)"

# exec command - accept worktree names and then any command
complete -c phantom -n "__phantom_using_command exec" -a "(__phantom_list_worktrees)"

# shell command options
complete -c phantom -n "__phantom_using_command shell" -l fzf -d "Use fzf for interactive selection"
complete -c phantom -n "__phantom_using_command shell" -a "(__phantom_list_worktrees)"

# completion command - shell names
complete -c phantom -n "__phantom_using_command completion" -a "fish zsh" -d "Shell type""#;

const ZSH_COMPLETION: &str = r#"#compdef phantom
# Zsh completion for phantom
# Place this in a directory in your $fpath (e.g., ~/.zsh/completions/)
# Or load dynamically with: eval "$(phantom completion zsh)"

# Only define the function, don't execute it
_phantom() {
    local -a commands
    commands=(
        'create:Create a new Git worktree (phantom)'
        'attach:Attach to an existing branch by creating a new worktree'
        'list:List all Git worktrees (phantoms)'
        'where:Output the filesystem path of a specific worktree'
        'delete:Delete a Git worktree (phantom)'
        'exec:Execute a command in a worktree directory'
        'shell:Open an interactive shell in a worktree directory'
        'version:Display phantom version information'
        'completion:Generate shell completion scripts'
    )

    _arguments -C \
        '--help[Show help (-h)]' \
        '--version[Show version (-v)]' \
        '1:command:->command' \
        '*::arg:->args'

    case ${state} in
        command)
            _describe 'phantom command' commands
            ;;
        args)
            case ${line[1]} in
                create)
                    _arguments \
                        '--shell[Open an interactive shell in the new worktree after creation (-s)]' \
                        '--exec[Execute a command in the new worktree after creation (-x)]:command:' \
                        '--tmux[Open the worktree in a new tmux window (-t)]' \
                        '--tmux-vertical[Open the worktree in a vertical tmux pane]' \
                        '--tmux-horizontal[Open the worktree in a horizontal tmux pane]' \
                        '*--copy-file[Copy specified files from the current worktree]:file:_files' \
                        '1:name:'
                    ;;
                attach)
                    _arguments \
                        '--shell[Open an interactive shell in the worktree after attaching (-s)]' \
                        '--exec[Execute a command in the worktree after attaching (-x)]:command:' \
                        '1:worktree-name:' \
                        '2:branch-name:'
                    ;;
                list)
                    _arguments \
                        '--fzf[Use fzf for interactive selection]' \
                        '--names[Output only phantom names (for scripts and completion)]'
                    ;;
                where|delete|shell)
                    local worktrees
                    worktrees=(${(f)"$(phantom list --names 2>/dev/null)"})
                    if [[ ${line[1]} == "where" || ${line[1]} == "shell" ]]; then
                        _arguments \
                            '--fzf[Use fzf for interactive selection]' \
                            '1:worktree:(${(q)worktrees[@]})'
                    elif [[ ${line[1]} == "delete" ]]; then
                        _arguments \
                            '--force[Force deletion even if worktree has uncommitted changes (-f)]' \
                            '--current[Delete the current worktree]' \
                            '--fzf[Use fzf for interactive selection]' \
                            '1:worktree:(${(q)worktrees[@]})'
                    fi
                    ;;
                exec)
                    local worktrees
                    worktrees=(${(f)"$(phantom list --names 2>/dev/null)"})
                    _arguments \
                        '1:worktree:(${(q)worktrees[@]})' \
                        '*:command:_command_names'
                    ;;
                completion)
                    _arguments \
                        '1:shell:(fish zsh)'
                    ;;
            esac
            ;;
    esac
}

# Register the completion function if loading dynamically
if [[ -n ${ZSH_VERSION} ]]; then
    autoload -Uz compinit && compinit -C
    compdef _phantom phantom
fi"#;

const BASH_COMPLETION: &str = r#"# Bash completion for phantom
# Place this in /etc/bash_completion.d/phantom or ~/.bash_completion.d/phantom
# Or source it directly: source <(phantom completion bash)

_phantom_completions() {
    local cur prev words cword
    _init_completion || return

    local commands="create attach list where delete exec shell version completion"

    # Handle main command completion
    if [[ $cword -eq 1 ]]; then
        COMPREPLY=($(compgen -W "$commands" -- "$cur"))
        return
    fi

    # Get the command
    local cmd="${words[1]}"

    case "$cmd" in
        create)
            case "$prev" in
                --exec|-x)
                    # No completion for exec command
                    return
                    ;;
                --copy-file)
                    _filedir
                    return
                    ;;
                *)
                    local opts="--shell -s --exec -x --tmux -t --tmux-vertical --tmux-horizontal --copy-file"
                    if [[ "$cur" == -* ]]; then
                        COMPREPLY=($(compgen -W "$opts" -- "$cur"))
                    fi
                    ;;
            esac
            ;;
        attach)
            case "$prev" in
                --exec|-x)
                    # No completion for exec command
                    return
                    ;;
                *)
                    local opts="--shell -s --exec -x"
                    if [[ "$cur" == -* ]]; then
                        COMPREPLY=($(compgen -W "$opts" -- "$cur"))
                    fi
                    ;;
            esac
            ;;
        list)
            local opts="--fzf --names"
            if [[ "$cur" == -* ]]; then
                COMPREPLY=($(compgen -W "$opts" -- "$cur"))
            fi
            ;;
        where|shell)
            local opts="--fzf"
            if [[ "$cur" == -* ]]; then
                COMPREPLY=($(compgen -W "$opts" -- "$cur"))
            else
                # Complete with worktree names
                local worktrees=$(phantom list --names 2>/dev/null)
                COMPREPLY=($(compgen -W "$worktrees" -- "$cur"))
            fi
            ;;
        delete)
            local opts="--force -f --current --fzf"
            if [[ "$cur" == -* ]]; then
                COMPREPLY=($(compgen -W "$opts" -- "$cur"))
            else
                # Complete with worktree names
                local worktrees=$(phantom list --names 2>/dev/null)
                COMPREPLY=($(compgen -W "$worktrees" -- "$cur"))
            fi
            ;;
        exec)
            if [[ $cword -eq 2 ]]; then
                # Complete with worktree names
                local worktrees=$(phantom list --names 2>/dev/null)
                COMPREPLY=($(compgen -W "$worktrees" -- "$cur"))
            else
                # Complete with commands
                COMPREPLY=($(compgen -c -- "$cur"))
            fi
            ;;
        completion)
            COMPREPLY=($(compgen -W "bash fish zsh" -- "$cur"))
            ;;
        *)
            ;;
    esac
}

complete -F _phantom_completions phantom"#;

/// Handle the completion command
pub fn handle(args: CompletionArgs) -> Result<()> {
    let script = match args.shell {
        Shell::Fish => FISH_COMPLETION,
        Shell::Zsh => ZSH_COMPLETION,
        Shell::Bash => BASH_COMPLETION,
    };

    println!("{}", script);
    Ok(())
}
