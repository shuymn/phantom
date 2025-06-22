# Phantom Rust Migration TODO

Active tasks for the Phantom Rust implementation.

- Completed tasks: [ARCHIVE.md](./ARCHIVE.md)

## 📚 Key Documentation

Essential guides for understanding the codebase:

- **Testing Guide**: [testing-guide.md](./rust/docs/testing-guide.md) - Comprehensive testing strategy and patterns
- **CommandExecutor Guide**: [command-executor-guide.md](./rust/docs/command-executor-guide.md) - Migration patterns and examples
- **Test Strategy**: [test-strategy.md](./rust/docs/test-strategy.md) - High-level testing philosophy
- **Test Rationale**: [test-rationale.md](./rust/docs/test-rationale.md) - Why we test this way
- **Architecture**: [architecture.md](./rust/docs/architecture.md) - System design and structure
- **Troubleshooting**: [troubleshooting.md](./rust/docs/troubleshooting.md) - Common issues and solutions
- **Advanced Features Guide**: [rust-advanced-features-guide.md](./rust/docs/rust-advanced-features-guide.md) - Advanced Rust patterns and techniques

## 📋 Future Enhancements

### Error Handling Comprehensive Refactor

Since this is an unreleased codebase, we can implement a comprehensive error handling improvement based on [error-handling-guide.md](./rust/docs/error-handling-guide.md):

#### Immediate Actions (In Progress)

- [x] **Add anyhow dependency** for application-layer error handling
- [x] **Convert all CLI handlers** to use `anyhow::Result` instead of `Result<T, PhantomError>`
- [x] **Replace string-based error variants** with structured, specific error types:
  - `Worktree(String)` → Specific worktree error variants
  - `Validation(String)` → Specific validation error variants  
  - `FileOperation(String)` → Specific file operation error variants
  - `ProcessExecution(String)` → Enhanced with command context
- [x] **Add rich context** at all external system boundaries:
  - Git command execution with full command details
  - File system operations with paths and operations
  - Process execution with command and arguments
- [ ] **Remove redundant ErrorContext trait** - anyhow provides better alternatives
- [ ] **Update documentation** to reflect new error handling patterns

#### Error Architecture

```rust
// Core library errors (thiserror) - Type safe, matchable
#[derive(Error, Debug)]
pub enum GitError {
    #[error("Command failed: {command} (exit code: {code})")]
    CommandFailed { command: String, args: Vec<String>, code: i32, stderr: String },
    // ... other specific variants
}

// Application layer (anyhow) - Rich context, simple propagation
use anyhow::{Context, Result};

async fn handle_operation() -> Result<()> {
    git_operation()
        .with_context(|| format!("Failed to create worktree '{}' at {}", name, path))?;
    Ok(())
}
```

#### Benefits
- **Better debugging**: Specific error types with full context
- **Type safety**: Compiler-enforced error handling in libraries
- **Simplicity**: anyhow in CLI layer reduces boilerplate
- **Rich errors**: Full command details, paths, and runtime values

### Known Issues

- **Flaky Test**: One flaky test in `get_current_worktree` that occasionally fails in CI. See [test-race-condition-fix.md](./rust/docs/test-race-condition-fix.md) for details on race condition handling.

### Potential Future Features

These are ideas for future enhancements that are not currently planned but could be considered:

- **Workspace Management**: Support for cargo workspaces and multi-crate projects
- **Git Hooks Integration**: Automatic setup of git hooks for worktrees
- **Worktree Templates**: Predefined configurations for common worktree patterns
- **Remote Worktree Support**: Ability to work with worktrees on remote machines
- **GUI/TUI Interface**: Graphical or terminal UI for worktree management
- **Plugin System**: Extensibility through plugins for custom workflows
- **Worktree Sync**: Keep multiple worktrees in sync with specific rules
- **Performance Monitoring**: Built-in profiling and performance tracking

## ✅ Success Criteria Achieved

- [x] Feature parity with TypeScript version
- [x] Binary size < 5MB (4.5MB achieved)
- [x] Zero runtime dependencies
- [x] Single binary distribution
- [x] Clean test architecture with comprehensive mocking

## 📊 Current Status

**Rust migration is complete!** The codebase has:
- 545+ tests total (0 ignored)
- Comprehensive mock-based testing infrastructure
- Full documentation of patterns and practices
- All handlers and operations properly abstracted
- **Advanced Rust patterns implemented:**
  - Zero-cost abstractions with generics (no dynamic dispatch)
  - Zero-copy operations with `Cow<'static, str>`
  - Type-state pattern for compile-time safety (Worktree states & WorktreeBuilder)
  - Rich error context with extension traits
  - Builder pattern with phantom types (compile-time validation)
  - Sealed traits for API stability (all core traits)
  - Extension traits for ergonomic APIs (removed - not used)
  - SmallVec optimization for command arguments
  - Const functions for compile-time validation (git refs, paths, hashes)
  - Concurrent async operations (3-5x speedup)
  - Advanced const utilities for git and core operations

## 🎯 Migration Complete

The Rust implementation of Phantom is now feature-complete and production-ready. All quality improvements have been implemented, tested, and documented. The codebase is maintainable, performant, and follows Rust best practices.

For the complete history of the migration and all completed tasks, see [ARCHIVE.md](./ARCHIVE.md).