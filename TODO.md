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
- **Codebase Review**: [rust-codebase-review.md](./rust/docs/rust-codebase-review.md) - Quality improvement opportunities

## 📋 Future Enhancements

### 🔴 High Priority

#### Rust Codebase Quality Improvements (Performance & Safety)
- [x] Replace dynamic dispatch with generics in HandlerContext ✅
  - [x] Convert `Arc<dyn CommandExecutor>` to generic parameter in rust/src/cli/context.rs
  - [x] Update all handler implementations to use static dispatch
  - [x] Maintain testability with direct mock instantiation
  - [x] Document the pattern with examples in handler_with_context.rs
- [x] Implement zero-copy operations for CommandOutput ✅
  - [x] Convert `String` fields to `Cow<'static, str>` in rust/src/core/command_executor.rs
  - [x] Add from_static() and from_owned() constructors
  - [x] Update all usages to avoid unnecessary allocations
  - [x] Examples demonstrate zero-copy patterns
- [x] Add rich error context and source chains ✅
  - [x] Enhanced error types in rust/src/core/error.rs with CommandContext
  - [x] Added ErrorContext and ResultContext extension traits
  - [x] Implemented context() and with_context() methods
  - [x] All errors now include rich debugging information
- [x] Implement type-state pattern for worktrees ✅
  - [x] Created TypedWorktree with phantom type states (Created, Attached, Detached, Locked, Deleted)
  - [x] Enforce compile-time state transitions
  - [x] Prevent invalid operations at compile time (e.g., can't delete attached worktree)
  - [x] Added WorktreeBuilder with type states for safe construction

#### Fix Missing --base Option Implementation
- [x] Implement --base option for create command (regression from TypeScript) ✅
  - [x] Update GitBackend trait to accept commitish parameter
  - [x] Modify add_worktree function to pass commitish to git command
  - [x] Update all GitBackend implementations (CommandBackend)
  - [x] Remove regression test TODO and verify it passes
  - [x] Add unit tests for --base functionality

#### Replace External Dependencies with Native Rust Libraries
- [ ] Replace fzf with skim-rs for fuzzy finding
  - [ ] Integrate skim as a library dependency
  - [ ] Migrate all fzf usage to skim API
  - [ ] Remove requirement for external fzf installation
  - [ ] Maintain identical user experience and features
  - [ ] Update installation docs to remove fzf requirement

#### Native Git Support
- [ ] Integrate libgit2 for native git operations
- [ ] Remove dependency on git CLI commands
- [ ] Improve performance for git operations
- [ ] Better error handling and recovery

#### Performance Improvements  
- [ ] Implement parallel worktree operations
- [ ] Add concurrent file copying with progress
- [ ] Optimize worktree listing for large repositories
- [ ] Reduce startup time with lazy loading

### 🟡 Medium Priority

#### Plugin System
- [ ] Design plugin API for extensibility
- [ ] Support lifecycle hooks (pre/post create, delete, switch)
- [ ] Enable custom commands and integrations
- [ ] Allow UI/UX customization plugins

#### Configuration Profiles
- [ ] Support multiple configuration profiles
- [ ] Per-project configuration overrides  
- [ ] Team-shared configuration templates
- [ ] Environment-specific settings

## ✅ Success Criteria Achieved

- [x] Feature parity with TypeScript version
- [x] Binary size < 5MB (4.5MB achieved)
- [x] Zero runtime dependencies
- [x] Single binary distribution
- [x] Clean test architecture with comprehensive mocking

## 📊 Current Status

**Rust migration is complete!** The codebase has:
- 532 tests total (0 ignored)
- Comprehensive mock-based testing infrastructure
- Full documentation of patterns and practices
- All handlers and operations properly abstracted
- **Advanced Rust patterns implemented:**
  - Zero-cost abstractions with generics (no dynamic dispatch)
  - Zero-copy operations with `Cow<'static, str>`
  - Type-state pattern for compile-time safety
  - Rich error context with extension traits
  - Builder pattern with phantom types

**Known Issue**: One flaky test in `get_current_worktree` that occasionally fails in CI. See [test-race-condition-fix.md](./rust/docs/test-race-condition-fix.md) for details on race condition handling.