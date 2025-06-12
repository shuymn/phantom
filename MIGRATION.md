# Migration Guide: TypeScript to Rust

This guide helps you migrate from the TypeScript version of Phantom to the new Rust implementation.

## Overview

The Rust version of Phantom maintains full compatibility with the TypeScript version while offering improved performance and a single binary distribution. All commands and features work identically, ensuring a seamless transition.

## Key Benefits of the Rust Version

- **Single Binary**: No runtime dependencies (Node.js/npm not required)
- **Better Performance**: Faster execution and lower memory usage
- **Native Integration**: Better terminal multiplexer support
- **Cross-platform**: Pre-built binaries for Linux and macOS (x86_64 and ARM64)

## Installation

### Option 1: Clone and Build

Clone the repository and build from source:
```bash
git clone https://github.com/aku11i/phantom.git
cd phantom/rust
cargo build --release
sudo cp target/release/phantom /usr/local/bin/
```

### Option 2: Using Cargo Install

If you have Rust installed, you can install directly with cargo:
```bash
cargo install --git https://github.com/aku11i/phantom --path rust
```

## Compatibility

The Rust version maintains 100% compatibility with the TypeScript version:

### Commands
All commands work identically:
- `phantom create <name>` - Create a new worktree
- `phantom attach <branch>` - Attach to an existing branch
- `phantom list` - List all worktrees
- `phantom where <name>` - Get worktree path
- `phantom delete <name>` - Delete a worktree
- `phantom exec <name> -- <command>` - Execute command in worktree
- `phantom shell <name>` - Open shell in worktree
- `phantom version` - Show version
- `phantom completion <shell>` - Generate shell completions

### Configuration
Both `.phantom.json` and `.phantom.toml` configuration files are supported:

#### JSON Format (existing)
```json
{
  "copyFiles": ["package.json", "package-lock.json", ".env"]
}
```

#### TOML Format (new option)
```toml
copyFiles = ["package.json", "package-lock.json", ".env"]
```

### Features
- All command options and flags work identically
- Terminal multiplexer support (tmux, Kitty)
- Shell detection and integration
- Interactive worktree selection with fzf
- JSON output mode (`--json` flag)

## Migration Steps

1. **Backup Current Setup** (optional)
   ```bash
   phantom list > my-worktrees.txt
   ```

2. **Install Rust Version**
   Use one of the installation methods above

3. **Verify Installation**
   ```bash
   phantom version
   # Should show: phantom 0.1.0
   ```

4. **Test Functionality**
   ```bash
   phantom list
   # Should show your existing worktrees
   ```

5. **Remove TypeScript Version** (optional)
   ```bash
   npm uninstall -g @aku11i/phantom
   ```

## Differences and Improvements

### New Features
- **JSON Output**: Most commands now support `--json` flag for structured output
- **TOML Config**: Configuration can now use TOML format
- **Better Error Messages**: More descriptive error handling
- **Verbose Mode**: Use `-v` flag for detailed logging

### Performance Improvements
- Faster worktree operations
- Lower memory footprint
- No startup overhead (no Node.js runtime)

### Platform Support
- Linux x86_64 and ARM64
- macOS x86_64 and ARM64 (Intel and Apple Silicon)
- No Windows support (Unix-only, same as TypeScript version)

## Troubleshooting

### Command Not Found
Ensure the binary is in your PATH:
```bash
echo $PATH
which phantom
```

### Permission Denied
Make the binary executable:
```bash
chmod +x /usr/local/bin/phantom
```

### Git Operations Fail
Ensure git is installed and accessible:
```bash
git --version
```

### FZF Not Found
Install fzf for interactive selection:
```bash
# macOS
brew install fzf

# Linux
git clone --depth 1 https://github.com/junegunn/fzf.git ~/.fzf
~/.fzf/install
```

## Reporting Issues

If you encounter any issues during migration:

1. Check if the issue exists in the TypeScript version
2. Report bugs at: https://github.com/aku11i/phantom/issues
3. Include:
   - Phantom version: `phantom version`
   - Platform: `uname -a`
   - Git version: `git --version`
   - Error messages and steps to reproduce

## Rollback

If you need to rollback to the TypeScript version:

1. Remove Rust version:
   ```bash
   sudo rm /usr/local/bin/phantom
   ```

2. Reinstall TypeScript version:
   ```bash
   npm install -g @aku11i/phantom
   ```

Your worktrees and configurations remain unchanged and will work with either version.