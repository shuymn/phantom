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

#### Rust Advanced Quality Improvements
Based on [rust-codebase-review.md](./rust/docs/rust-codebase-review.md) and [rust-advanced-features-guide.md](./rust/docs/rust-advanced-features-guide.md).

##### 🚀 Performance Optimizations (Immediate Impact)
- [x] Replace Vec<String> with SmallVec for command arguments ✅
  - Location: `rust/src/core/command_executor.rs:11`
  - Use `SmallVec<[String; 4]>` for stack allocation of ≤4 args
  - Reduces heap allocations for common commands
  - Consider `Cow<'_, str>` for further optimization

##### 🏗️ Architecture Improvements (1 week)
- [x] Add const functions for compile-time validation ✅
  - Convert `default_phantom_dir()` to `const fn`
  - Implement `const fn validate_worktree_name(name: &str) -> bool`
  - Enable compile-time constants and validation
  - Location: Various utility functions

- [x] Implement concurrent async operations ✅
  - Convert sequential worktree operations to use `FuturesUnordered`
  - Add `buffer_unordered(5)` for rate limiting
  - Target: `list_all_worktree_info` and similar batch operations
  - Expected speedup: 3-5x for multi-worktree operations
  - Implemented: list, select, file copy operations now concurrent

- [ ] Implement builder pattern with type states
  - Create `WorktreeBuilder<State>` with phantom types
  - States: `NoName`, `WithName`, `Ready`
  - Enforce required fields at compile time
  - Make invalid configurations impossible to express

- [ ] Add sealed traits for API stability
  - Seal `GitBackend`, `CommandExecutor`, `FileSystem` traits
  - Prevent downstream implementations
  - Use private `Sealed` supertrait pattern
  - Maintain flexibility for internal changes

- [ ] Create extension traits for better ergonomics
  - `WorktreeExt` for additional worktree methods
  - `CommandExecutorExt` for convenience functions
  - `ResultExt` for error context methods
  - Blanket implementations for all types

##### 📚 Documentation and Policy Updates
- [ ] Update CONTRIBUTING.md with performance guidelines
  - When to use generics vs trait objects
  - Memory allocation best practices
  - Async/concurrent patterns
  - Testing generic code

- [ ] Create performance policy documentation
  - Target: CLI startup < 50ms
  - Predictable memory usage patterns
  - Prefer stack allocation for small data
  - Document profiling and benchmarking

### 🟡 Medium Priority

#### Advanced Rust Patterns
- [ ] Update testing strategy for generic contexts
  - Use conditional compilation: `#[cfg(test)]`
  - Maintain zero-cost abstractions in production
  - Keep test ergonomics with type aliases
  - Document patterns in test-strategy.md

- [ ] Add benchmarking suite
  - Measure impact of optimizations
  - Track performance regressions
  - Use criterion.rs for statistical analysis
  - Benchmark critical paths (startup, list, create)

- [ ] Smart pointer optimizations
  - Replace excessive cloning with `Arc`/`Rc`
  - Use interior mutability where appropriate
  - Cache expensive computations
  - Document ownership patterns

### 🟢 Low Priority

#### Future Optimizations
- [ ] Arena allocation for batch operations
  - Create `BatchProcessor<'a>` with arena allocator
  - Reduce allocation overhead in bulk operations
  - Use typed-arena crate
  - Target: bulk worktree operations

- [ ] Derive macros for CLI commands
  - Create `#[derive(Command)]` macro
  - Generate clap definitions automatically
  - Reduce boilerplate in command definitions
  - Include validation in macro

- [ ] Advanced type-level programming
  - Const generics for compile-time validation
  - Type-level state machines
  - Compile-time string validation
  - Zero-runtime-cost abstractions

- [ ] Streaming support for large outputs
  - Add `StreamingOutput` type
  - Support `AsyncRead` for stdout/stderr
  - Reduce memory usage for large git outputs
  - Progressive output handling

- [ ] Custom smart pointers
  - `SmallBox<T, N>` with inline storage
  - Optimized for common allocation patterns
  - Benchmark against standard Box
  - Use in hot paths only

- [ ] Lock-free concurrency patterns
  - Atomic reference counting for shared state
  - Lock-free queues for work distribution
  - Epoch-based memory reclamation
  - Target: concurrent worktree access

- [ ] Procedural macros for validation
  - `#[validate(worktree_name)]` attribute
  - Compile-time and runtime validation
  - Generate error messages automatically
  - Reduce validation boilerplate

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