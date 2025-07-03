# 👻 Phantom

<div align="center">

**A powerful CLI tool for seamless parallel development with Git worktrees**

[![npm version](https://img.shields.io/npm/v/@aku11i/phantom.svg)](https://www.npmjs.com/package/@aku11i/phantom)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Node.js Version](https://img.shields.io/node/v/@aku11i/phantom.svg)](https://nodejs.org)
[![Ask DeepWiki](https://deepwiki.com/badge.svg)](https://deepwiki.com/aku11i/phantom)

[日本語](./README.ja.md) • [Installation](#-installation) • [Why Phantom?](#-why-phantom) • [Basic Usage](#-basic-usage) • [Documentation](#-documentation)

![Phantom demo](./docs/assets/phantom.gif)

</div>

> **🦀 Rust Version Available**: A Rust implementation of Phantom is now available in the [`rust/`](./rust/) directory. See [Migration Guide](./MIGRATION.md) for details.

## ✨ What is Phantom?

Phantom is a powerful CLI tool that dramatically boosts your development productivity by making Git worktrees simple and intuitive. Run multiple tasks in isolated environments simultaneously and achieve true multitask development. Built for the next generation of parallel development workflows, including AI-powered coding with multiple agents.

### Key Features

- 🚀 **Simple worktree management** - Create and manage Git worktrees with intuitive commands
- 🔄 **True multitasking** - Create separate working directories per branch and run multiple tasks simultaneously
- 🎯 **Execute commands from anywhere** - Run commands in any worktree with `phantom exec <worktree> <command>`
- 🪟 **Terminal multiplexer integration** - Built-in support for tmux and kitty terminal
- 🔍 **Interactive selection with fzf** - Use built-in fzf option for worktree selection
- 🎮 **Shell completion** - Full autocomplete support for Fish and Zsh
- ⚡ **Zero dependencies** - Fast and lightweight

## 🚀 Installation

### Using Homebrew (recommended)

```bash
brew install aku11i/tap/phantom
```

#### Using npm

```bash
npm install -g @aku11i/phantom
```

## 🤔 Why Phantom?

Git worktrees are powerful but require manual management of paths and branches. Also, navigating between multiple worktrees is cumbersome. Phantom eliminates these problems:

```bash
# Without Phantom
git worktree add -b feature-awesome ../project-feature-awesome origin/main
cd ../project-feature-awesome

# With Phantom
phantom create feature-awesome --shell
```

### How Phantom Works

When you run `phantom create feature-awesome`, a new Git worktree named `feature-awesome` is created in `.git/phantom/worktrees/`.
All worktrees created with phantom are centrally managed in this location.

```
your-project/    # Git repository
├── .git/
│   └── phantom/
│       └── worktrees/        # Phantom-managed directory
│           ├── feature-awesome/  # branch name = worktree name
│           ├── bugfix-login/     # another worktree
│           └── hotfix-critical/  # yet another worktree
└── ...
```

This convention means you never need to remember worktree paths - just use the branch name for easy worktree operations.

### ✈️ Features for a Comfortable Development Experience

Phantom provides perfect functionality as a command-line tool. Developers feel the trust and comfort of flying first class.

#### Shell Completion

Phantom supports full shell completion for fish and zsh. Use tab key to complete commands and worktree names.

#### Terminal Multiplexer Integration

Phantom supports both tmux and kitty terminal for advanced window management. This allows you to manage multiple work environments simultaneously.

**tmux Integration:**
```bash
# Create and open worktree in new window
phantom create feature-x --tmux
# Create with split panes
phantom create feature-y --tmux-vertical
phantom create feature-z --tmux-horizontal

# Open existing worktrees in tmux
phantom shell feature-x --tmux
phantom shell feature-y --tmux-v
```

![Phantom tmux integration](./docs/assets/phantom-tmux.gif)

**Kitty Integration:**
```bash
# Open in new tab
phantom shell feature-xyz --kitty

# Split vertically
phantom shell feature-xyz --kitty-vertical

# Execute command in horizontal split
phantom exec feature-xyz --kitty-horizontal npm run dev
```

#### Editor Integration

Phantom works seamlessly with editors like VS Code and Cursor. You can specify an editor to open worktrees.

```bash
# Open with VS Code
phantom create feature --exec "code ."

# Or open existing worktree
phantom exec feature code .

# Open with Cursor
phantom create feature --exec "cursor ."
phantom exec feature cursor .
```

![Phantom VS Code integration](./docs/assets/phantom-vscode.gif)

#### fzf Integration

Interactive search with fzf allows quick worktree selection.

```bash
# Open shell with fzf selection
phantom shell --fzf

# Delete with fzf selection
phantom delete --fzf
```

## 🔍 Basic Usage

### Create a new worktree

```bash
phantom create feature-awesome

phantom list
```

### Start a new shell in the worktree

```bash
phantom shell feature-awesome

# Start development work

# Exit the shell when done
exit
```

### Run commands in any worktree

```bash
phantom exec feature-awesome {command to run}
# Example: phantom exec feature-awesome npm run build
```

### Clean up when done

```bash
phantom delete feature-awesome
```


## 📚 Documentation

- **[Getting Started](./docs/getting-started.md)** - Common workflows and tips
- **[Commands Reference](./docs/commands.md)** - All commands and options
- **[Configuration](./docs/configuration.md)** - Set up automatic file copying and post-create commands


## 🤝 Contributing

Contributions are welcome! See our [Contributing Guide](./CONTRIBUTING.md) for:
- Development setup
- Code style guidelines  
- Testing requirements
- Pull request process

## ⚠️ Disclaimer

phantom-rs is an **unofficial** Rust port created as a personal learning project. 
While it aims to provide similar functionality to the original phantom:

- **No guarantee of feature parity** with the original TypeScript version
- **No promise of identical behavior** for equivalent features
- **Breaking changes may occur** as the project evolves
- **Use at your own risk** in production environments

This project serves as both a functional tool and a Rust learning exercise.

## 📄 License

MIT License - see [LICENSE](LICENSE)

## 🙏 Acknowledgments

phantom-rs is a Rust port of the original [phantom](https://github.com/aku11i/phantom) by @aku11i.
The demonstration GIFs and core functionality remain faithful to the original implementation.

- Original TypeScript implementation: [@aku11i](https://github.com/aku11i)
- Rust port and enhancements: [@shuymn](https://github.com/shuymn)
