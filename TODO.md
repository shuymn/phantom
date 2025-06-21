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

- [x] Implement builder pattern with type states ✅
  - Created `WorktreeBuilder<State>` with phantom types
  - States: `NoName`, `WithName`, `Ready`
  - Enforces required fields at compile time
  - Invalid configurations impossible to express
  - Added async `create()` method for direct worktree creation

- [x] Add sealed traits for API stability ✅
  - All core traits (`GitBackend`, `CommandExecutor`, `FileSystem`, `ExitHandler`) are sealed
  - Prevents downstream implementations
  - Uses private `Sealed` supertrait pattern
  - Maintains flexibility for internal changes
  - Well-documented in sealed_traits_example.rs

- [x] Create extension traits for better ergonomics ✅
  - `WorktreeExt` for additional worktree methods (is_main, display_name, etc.)
  - `CommandExecutorExt` for convenience functions (run_simple, run_in_dir)
  - `ResultExt` for error context methods
  - `StrExt` for git-specific string operations
  - `PhantomConfigExt` and `GitConfigExt` for config management
  - All have blanket implementations

##### 📚 Documentation and Policy Updates
- [x] Update CONTRIBUTING.md with performance guidelines ✅
  - When to use generics vs trait objects
  - Memory allocation best practices
  - Async/concurrent patterns
  - Testing generic code

- [x] Create performance policy documentation ✅
  - Target: CLI startup < 50ms
  - Predictable memory usage patterns
  - Prefer stack allocation for small data
  - Document profiling and benchmarking

### 🟡 Medium Priority

#### Advanced Rust Patterns
- [x] Update testing strategy for generic contexts ✅
  - Generic contexts already in use throughout the codebase
  - Zero-cost abstractions maintained in production
  - Test ergonomics preserved with MockCommandExecutor, etc.
  - Patterns documented in test-strategy.md

- [x] Add benchmarking suite ✅
  - Measure impact of optimizations
  - Track performance regressions
  - Use criterion.rs for statistical analysis
  - Benchmark critical paths (startup, list, create)

- [x] Smart pointer optimizations ✅
  - Replace excessive cloning with `Arc`/`Rc`
  - Use interior mutability where appropriate
  - Cache expensive computations
  - Document ownership patterns

### 🟢 Low Priority

#### Future Optimizations
- [x] ~~Arena allocation for batch operations~~ ✅ (Removed - unnecessary optimization)
  - ~~Create `BatchProcessor<'a>` with arena allocator~~
  - ~~Reduce allocation overhead in bulk operations~~
  - ~~Use typed-arena crate~~
  - ~~Target: bulk worktree operations~~
  - ~~Implemented: BatchProcessor with typed-arena for efficient memory usage~~

- [x] Advanced type-level programming ✅
  - [x] Const generics for compile-time validation
  - [x] Type-level state machines (via type-state pattern)
  - [x] Compile-time string validation (const functions)
  - [x] Zero-runtime-cost abstractions
  - Implemented const utilities in git/const_utils.rs and core/const_utils.rs

- [x] ~~Custom smart pointers~~ ✅ (Removed - unnecessary optimization, using external smallvec instead)
  - ~~`SmallBox<T, N>` with inline storage~~
  - ~~Optimized for common allocation patterns~~
  - ~~Benchmark against standard Box~~
  - ~~Use in hot paths only~~
  - ~~Implemented: SmallBox and SmallVec with up to 1.36x performance improvement~~

- [ ] Lock-free concurrency patterns
  - Atomic reference counting for shared state
  - Lock-free queues for work distribution
  - Epoch-based memory reclamation
  - Target: concurrent worktree access

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


## ✅ Success Criteria Achieved

- [x] Feature parity with TypeScript version
- [x] Binary size < 5MB (4.5MB achieved)
- [x] Zero runtime dependencies
- [x] Single binary distribution
- [x] Clean test architecture with comprehensive mocking

## 📊 Current Status

**Rust migration is complete!** The codebase has:
- 548+ tests total (0 ignored)
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
  - Extension traits for ergonomic APIs (6 extension traits)
  - SmallVec optimization for command arguments
  - Const functions for compile-time validation (git refs, paths, hashes)
  - Concurrent async operations (3-5x speedup)
  - Advanced const utilities for git and core operations

**Known Issue**: One flaky test in `get_current_worktree` that occasionally fails in CI. See [test-race-condition-fix.md](./rust/docs/test-race-condition-fix.md) for details on race condition handling.