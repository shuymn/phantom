# Phantom Troubleshooting Guide

This guide helps you resolve common issues with Phantom.

## Installation Issues

### "command not found" after installation

**Problem**: After installing Phantom, the `phantom` command is not recognized.

**Solutions**:
1. Check if the binary is in your PATH:
   ```bash
   echo $PATH
   ls -la /usr/local/bin/phantom
   ```

2. Add the installation directory to your PATH:
   ```bash
   # For bash
   echo 'export PATH="/usr/local/bin:$PATH"' >> ~/.bashrc
   source ~/.bashrc
   
   # For zsh
   echo 'export PATH="/usr/local/bin:$PATH"' >> ~/.zshrc
   source ~/.zshrc
   ```

3. Try using the full path:
   ```bash
   /usr/local/bin/phantom --help
   ```

### Permission denied when running phantom

**Problem**: Getting "Permission denied" error when trying to run phantom.

**Solution**:
```bash
chmod +x /usr/local/bin/phantom
```

### Cargo install fails

**Problem**: `cargo install` fails to build or install phantom.

**Solutions**:
1. Ensure you have Rust 1.75.0 or later:
   ```bash
   rustc --version
   ```

2. Update Rust if needed:
   ```bash
   rustup update stable
   ```

3. Try cloning and building manually:
   ```bash
   git clone https://github.com/shuymn/phantom-rs.git
   cd phantom-rs
   cargo build --release
   sudo cp target/release/phantom /usr/local/bin/
   ```

4. Check for build errors:
   ```bash
   cargo check
   cargo test
   ```

## Git-related Issues

### "not a git repository" error

**Problem**: Phantom commands fail with "not a git repository" error.

**Solution**: Ensure you're running phantom from within a Git repository:
```bash
git status  # Should show repository status
git init    # If you need to initialize a new repository
```

### "Git command failed" errors

**Problem**: Git operations fail when creating or managing worktrees.

**Solutions**:
1. Check Git version (requires Git 2.5+):
   ```bash
   git --version
   ```

2. Ensure you have commits in your repository:
   ```bash
   git log --oneline
   ```

3. Check for Git configuration issues:
   ```bash
   git config user.name
   git config user.email
   ```

### Branch already exists

**Problem**: Cannot create a worktree because the branch already exists.

**Solutions**:
1. Use a different name:
   ```bash
   phantom create feature-new
   ```

2. Attach to the existing branch:
   ```bash
   phantom attach existing-branch
   ```

3. Delete the existing branch first:
   ```bash
   git branch -d existing-branch
   phantom create existing-branch
   ```

## Worktree Issues

### Cannot delete worktree - "uncommitted changes"

**Problem**: Phantom refuses to delete a worktree due to uncommitted changes.

**Solutions**:
1. Commit or stash changes first:
   ```bash
   cd $(phantom where worktree-name)
   git add .
   git commit -m "Save work"
   # or
   git stash
   ```

2. Force delete (loses uncommitted changes):
   ```bash
   phantom delete worktree-name --force
   ```

### Worktree not found

**Problem**: Phantom can't find a worktree that should exist.

**Solutions**:
1. List all worktrees to check exact names:
   ```bash
   phantom list
   ```

2. Check if Git knows about the worktree:
   ```bash
   git worktree list
   ```

3. Try to repair Git worktree references:
   ```bash
   git worktree prune
   phantom list
   ```

### Disk space issues

**Problem**: Cannot create new worktrees due to insufficient disk space.

**Solution**: Worktrees share the Git object database, but still need space for working files:
```bash
# Check available space
df -h .

# Clean up Git objects
git gc --aggressive

# Remove unused worktrees
phantom list
phantom delete unused-worktree
```

## Terminal Integration Issues

### FZF not found

**Problem**: Interactive selection fails with "fzf command not found".

**Solution**: Install fzf:
```bash
# macOS
brew install fzf

# Ubuntu/Debian
sudo apt-get install fzf

# From source
git clone --depth 1 https://github.com/junegunn/fzf.git ~/.fzf
~/.fzf/install
```

### Shell detection fails

**Problem**: `phantom shell` opens wrong shell or fails.

**Solutions**:
1. Set your preferred shell explicitly:
   ```bash
   export SHELL=/bin/zsh
   ```

2. Check shell configuration:
   ```bash
   echo $SHELL
   which $SHELL
   ```

### Tmux integration not working

**Problem**: Tmux commands fail or don't create new windows/panes.

**Solutions**:
1. Ensure tmux is installed:
   ```bash
   tmux -V
   ```

2. Check if you're inside a tmux session:
   ```bash
   echo $TMUX
   ```

3. Try without tmux:
   ```bash
   phantom shell worktree-name
   ```

## Configuration Issues

### Config file not recognized

**Problem**: Phantom doesn't copy files specified in configuration.

**Solutions**:
1. Check config file location (must be in repository root):
   ```bash
   ls -la .phantom.json
   ls -la .phantom.toml
   ```

2. Validate JSON syntax:
   ```bash
   cat .phantom.json | jq .
   ```

3. Check file paths are relative to repository root:
   ```json
   {
     "copyFiles": ["package.json", "src/config.js"]
   }
   ```

### Files not being copied

**Problem**: Files specified in config are not copied to new worktrees.

**Solutions**:
1. Ensure files exist in the repository:
   ```bash
   ls -la package.json
   ```

2. Check for typos in file paths
3. Use forward slashes for paths, even on Windows:
   ```json
   {
     "copyFiles": ["src/config/app.json"]
   }
   ```

## Performance Issues

### Slow worktree creation

**Problem**: Creating worktrees takes a long time.

**Solutions**:
1. Check repository size:
   ```bash
   du -sh .git
   ```

2. Optimize Git:
   ```bash
   git gc
   git repack -Ad
   ```

3. Consider using shallow clones for large repositories

### High memory usage

**Problem**: Phantom uses too much memory.

**Solution**: This is unusual for the Rust version. Report the issue with:
- Repository size
- Number of worktrees
- System specifications

## Debug Information

To help diagnose issues, run phantom with verbose output:
```bash
phantom -v list
phantom -v create test-worktree
```

Collect system information:
```bash
# Phantom version
phantom version

# System info
uname -a

# Git version
git --version

# Disk space
df -h .

# Git repository info
git rev-parse --show-toplevel
git worktree list
```

## Getting Help

If these solutions don't resolve your issue:

1. Check existing issues: https://github.com/shuymn/phantom-rs/issues
2. Create a new issue with:
   - Problem description
   - Steps to reproduce
   - Error messages
   - Debug information (see above)
   - Phantom version and platform

## Common Error Codes

- **1**: General error
- **2**: Validation error (invalid input)
- **3**: Git operation failed
- **4**: Worktree operation failed
- **5**: Configuration error
- **6**: Branch not found
- **7**: Process execution failed