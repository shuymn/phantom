# Phantom Rust Architecture

This document describes the technical architecture of the Rust implementation of Phantom.

## Overview

The Rust implementation follows a layered architecture with clear separation of concerns:

```
┌─────────────────────────────────────────┐
│           CLI Layer (src/cli/)          │
│  Commands, Handlers, Output Formatting  │
└─────────────────────────┬───────────────┘
                          │
┌─────────────────────────▼───────────────┐
│         Core Layer (src/core/)          │
│   Business Logic (Platform Agnostic)    │
├─────────────────────────────────────────┤
│  • Worktree Management (worktree/)      │
│  • Process Execution (process/)         │
│  • Git Operations (git/)                │
│  • Configuration (config/)              │
└─────────────────────────────────────────┘
```

## Directory Structure

```
src/
├── main.rs              # Entry point
├── lib.rs               # Library root
├── errors.rs            # Error types
├── cli/                 # CLI layer
│   ├── commands/        # Command definitions
│   ├── handlers/        # Command handlers
│   └── output/          # Output formatting
├── core/                # Core business logic
│   ├── git/             # Git operations
│   │   ├── backend.rs   # Git backend trait
│   │   ├── command.rs   # Command-based implementation
│   │   ├── executor.rs  # Git command executor
│   │   ├── factory.rs   # Backend factory
│   │   └── libs/        # Git operation helpers
│   ├── worktree/        # Worktree management
│   │   ├── create.rs    # Create worktrees
│   │   ├── delete.rs    # Delete worktrees
│   │   ├── list.rs      # List worktrees
│   │   ├── attach.rs    # Attach to branches
│   │   └── validate.rs  # Validation logic
│   ├── process/         # Process execution
│   │   ├── spawn.rs     # Process spawning
│   │   ├── exec.rs      # Command execution
│   │   ├── shell.rs     # Shell operations
│   │   ├── tmux.rs      # tmux integration
│   │   └── kitty.rs     # Kitty terminal integration
│   └── config/          # Configuration
│       ├── loader.rs    # Config loading
│       └── types.rs     # Config types
└── test_utils.rs        # Test utilities
```

## Key Design Principles

### 1. Single Responsibility
Each module has one clear responsibility:
- `git/`: All git-related operations
- `worktree/`: Worktree lifecycle management
- `process/`: Process and terminal operations
- `config/`: Configuration management

### 2. Dependency Injection
The `GitBackend` trait allows for different git implementations:
```rust
pub trait GitBackend: Send + Sync {
    async fn add_worktree(&self, path: &Path, branch: Option<&str>, create_branch: bool) -> Result<()>;
    async fn list_worktrees(&self) -> Result<Vec<WorktreeInfo>>;
    // ...
}
```

### 3. Error Handling
Comprehensive error types with context:
```rust
#[derive(Error, Debug)]
pub enum PhantomError {
    #[error("Git error: {message}")]
    Git { message: String, exit_code: Option<i32> },
    
    #[error("Worktree error: {0}")]
    Worktree(String),
    
    // ...
}
```

### 4. Async/Await
All I/O operations are async using Tokio:
```rust
pub async fn create_worktree(
    git_root: &Path,
    name: &str,
    options: CreateWorktreeOptions,
) -> Result<CreateWorktreeSuccess>
```

## Core Components

### Git Operations
- **GitExecutor**: Executes git commands asynchronously
- **CommandBackend**: Default implementation using git CLI
- **GitBackend trait**: Abstraction for future libgit2 support

### Worktree Management
- **create**: Creates new worktrees with optional file copying
- **attach**: Attaches to existing branches
- **list**: Lists all worktrees with detailed information
- **delete**: Removes worktrees safely

### Process Management
- **spawn**: Spawns processes with proper signal handling
- **exec**: Executes commands in worktree directories
- **shell**: Opens interactive shells
- **Terminal multiplexers**: tmux and Kitty support

### Configuration
- Supports both JSON and TOML formats
- Backward compatible with TypeScript config
- Per-worktree configuration options

## Testing Strategy

### Unit Tests
- Located alongside implementation files
- Test individual functions in isolation
- Mock external dependencies

### Integration Tests
- Located in `tests/` directory
- Test full command flows
- Use real git repositories

### Property-Based Tests
- Use `proptest` for validation logic
- Test edge cases automatically

## Performance Considerations

1. **Lazy Loading**: Configuration loaded only when needed
2. **Concurrent Operations**: Git operations can run in parallel
3. **Zero-Copy**: Use references where possible
4. **Static Dispatch**: Prefer generics over trait objects

## Future Enhancements

1. **libgit2 Support**: Native git operations without CLI
2. **Parallel Operations**: Batch worktree operations
3. **Plugin System**: Extensible command framework
4. **Advanced Caching**: Cache git operation results

## Compatibility

The Rust implementation maintains full compatibility with the TypeScript version:
- Same CLI interface
- Same configuration format
- Same worktree structure
- Same command behavior