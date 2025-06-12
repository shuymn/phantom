# Phantom Rust Implementation

This is the Rust implementation of phantom, a CLI tool for managing Git worktrees.

## Prerequisites

- Rust 1.75.0 or later
- Cargo (comes with Rust)
- Git 2.5 or later

## Installation

### Clone and Build
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

## Available Commands

### Core Commands
- `create <name>` - Create a new worktree
- `delete <name>` - Delete a worktree
- `list` - List all worktrees
- `where <name>` - Get the path of a worktree
- `exec <name> -- <command>` - Execute a command in a worktree
- `shell <name>` - Open a shell in a worktree

### Utility Commands
- `completion <shell>` - Generate shell completions
- `version` - Show version information

### Command Options
Most commands support:
- `--fzf` - Use fzf for interactive selection
- `--json` - Output in JSON format
- `-h, --help` - Show help for a command

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

## Integration with Shell

### Bash/Zsh
```bash
# Generate completions
./target/release/phantom completion bash > phantom.bash
source phantom.bash
```

### Fish
```fish
./target/release/phantom completion fish > ~/.config/fish/completions/phantom.fish
```

## Contributing

When making changes:
1. Run `cargo fmt` to format code
2. Run `cargo clippy` to check for issues
3. Run `cargo test` to ensure tests pass
4. Update tests for new functionality