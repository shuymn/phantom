# 👻 Phantom (Rust)

<div align="center">

**A powerful CLI tool for seamless parallel development with Git worktrees**

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust Version](https://img.shields.io/badge/rust-1.75.0+-orange.svg)](https://www.rust-lang.org)

[Installation](#-installation) • [Why Phantom?](#-why-phantom) • [Basic Usage](#-basic-usage) • [Documentation](#-documentation)

</div>

> **Note**: This is the Rust implementation of Phantom. For the original TypeScript version, see the [parent directory](../README.md).

## ✨ What is Phantom?

Phantom is a powerful CLI tool that dramatically boosts your development productivity by making Git worktrees simple and intuitive. Run multiple tasks in isolated environments simultaneously and achieve true multitask development.

### Key Features

- 🚀 **Simple worktree management** - Create and manage Git worktrees with intuitive commands
- 🔄 **True multitasking** - Create separate working directories per branch and run multiple tasks simultaneously
- 🎯 **Execute commands from anywhere** - Run commands in any worktree with `phantom exec <worktree> <command>`
- 🪟 **Terminal multiplexer integration** - Built-in support for tmux and kitty terminal
- 🔍 **Interactive selection with fzf** - Use built-in fzf option for worktree selection
- 🎮 **Shell completion** - Full autocomplete support for Fish, Zsh, and Bash
- ⚡ **Zero runtime dependencies** - Single static binary
- 🦀 **Memory safe** - Written in Rust for reliability and performance

## 🚀 Installation

### From Source (Clone and Build)

```bash
git clone https://github.com/aku11i/phantom.git
cd phantom/rust
cargo build --release
sudo cp target/release/phantom /usr/local/bin/
```

### Using Cargo Install

```bash
cargo install --git https://github.com/aku11i/phantom --path rust
```

## Building

```bash
# Build in debug mode
cargo build

# Build in release mode (optimized)
cargo build --release
```

## Running

```bash
# Run directly with cargo
cargo run -- <command>

# Or build and run the binary
./target/debug/phantom <command>
./target/release/phantom <command>  # for release build
```

## 🤔 Why Phantom?

Git worktrees are powerful but require manual management of paths and branches. Phantom eliminates these problems:

```bash
# Without Phantom
git worktree add -b feature-awesome ../project-feature-awesome origin/main
cd ../project-feature-awesome

# With Phantom
phantom create feature-awesome --shell
```

## 🔍 Basic Usage

### Create a new worktree

```bash
phantom create feature-awesome

# Create and open a shell
phantom create feature-awesome --shell

# Create from specific branch/commit
phantom create hotfix --from main
```

### List worktrees

```bash
phantom list

# JSON output for scripting
phantom list --json
```

### Open a shell in a worktree

```bash
phantom shell feature-awesome

# With tmux integration
phantom shell feature-awesome --tmux
```

### Execute commands in a worktree

```bash
phantom exec feature-awesome cargo build
phantom exec feature-awesome --json cargo test
```

### Delete a worktree

```bash
phantom delete feature-awesome

# Force delete (removes uncommitted changes)
phantom delete feature-awesome --force

# Interactive selection with fzf
phantom delete --fzf
```

### Find worktree path

```bash
phantom where feature-awesome
# Output: /path/to/repo/.git/phantom/worktrees/feature-awesome
```

## 📦 Shell Completion

Generate shell completions for your shell:

```bash
# Fish
phantom completion fish > ~/.config/fish/completions/phantom.fish

# Zsh
phantom completion zsh > ~/.zfunc/_phantom
echo "fpath=(~/.zfunc $fpath)" >> ~/.zshrc

# Bash
phantom completion bash > /etc/bash_completion.d/phantom
```

## Testing

```bash
# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run a specific test
cargo test test_name

# Run tests in a specific module
cargo test module_name::

# Run with coverage (requires cargo-tarpaulin)
cargo install cargo-tarpaulin
cargo tarpaulin --out Html
```

## Development

### Code Structure
- `src/bin/` - Binary entry point
- `src/cli/` - CLI layer (commands, handlers, output)
- `src/core/` - Core business logic
  - `git/` - Git operations
  - `worktree/` - Worktree management
  - `process/` - Process execution
  - `config/` - Configuration

### Running Tests During Development
```bash
# Watch for changes and run tests
cargo install cargo-watch
cargo watch -x test

# Check code without building
cargo check

# Format code
cargo fmt

# Run linter
cargo clippy
```

## Verifying the Implementation

### Basic Workflow Test
```bash
# Initialize a git repository
git init test-repo
cd test-repo
git commit --allow-empty -m "Initial commit"

# Create a worktree
cargo run -- create feature-branch

# List worktrees
cargo run -- list

# Get worktree path
cargo run -- where feature-branch

# Execute command in worktree
cargo run -- exec feature-branch -- pwd

# Open shell in worktree
cargo run -- shell feature-branch

# Delete worktree
cargo run -- delete feature-branch
```

### Testing with FZF
```bash
# Requires fzf to be installed
cargo run -- list --fzf
cargo run -- delete --fzf
cargo run -- shell --fzf
```

### Testing JSON Output
```bash
cargo run -- list --json
cargo run -- where feature-branch --json
```

## Debugging

### Enable Debug Logs
```bash
# Set log level
RUST_LOG=debug cargo run -- <command>
RUST_LOG=phantom=debug cargo run -- <command>

# Trace level for maximum verbosity
RUST_LOG=trace cargo run -- <command>
```

### Common Issues

1. **Permission Denied**
   - Ensure the binary has execute permissions: `chmod +x target/debug/phantom`

2. **Git Not Found**
   - Ensure git is in your PATH: `which git`

3. **Worktree Already Exists**
   - Check existing worktrees: `git worktree list`

4. **Tests Failing on macOS**
   - Some tests assume Linux `/proc` filesystem
   - Platform-specific tests handle this appropriately

## Performance Testing

```bash
# Build with optimizations
cargo build --release

# Time command execution
time ./target/release/phantom list

# Profile with flamegraph (Linux)
cargo install flamegraph
cargo flamegraph -- list
```


## 🔄 Migration from TypeScript Version

If you're migrating from the TypeScript version of Phantom:

1. The command-line interface remains the same
2. Configuration files (`.phantom.json`) are compatible
3. All existing worktrees continue to work
4. Shell completions need to be regenerated

See [MIGRATION.md](../MIGRATION.md) for detailed migration instructions.

## 📚 Documentation

- **[Commands Reference](../docs/commands.md)** - All commands and options
- **[Configuration](../docs/configuration.md)** - Set up automatic file copying and post-create commands
- **[Architecture](ARCHITECTURE.md)** - Technical details of the Rust implementation

## 🤝 Contributing

Contributions are welcome! See our [Contributing Guide](../CONTRIBUTING.md) for:
- Development setup
- Code style guidelines  
- Testing requirements
- Pull request process

When making changes:
1. Run `cargo fmt` to format code
2. Run `cargo clippy` to check for issues
3. Run `cargo test` to ensure tests pass
4. Update tests for new functionality

## 📄 License

MIT License - see [LICENSE](../LICENSE)

## 🙏 Acknowledgments

Built with 👻 by [@aku11i](https://github.com/aku11i) and [Claude](https://claude.ai)