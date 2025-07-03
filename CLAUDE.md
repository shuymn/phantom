# Claude Context for phantom-rs

## Project Overview
Phantom is a powerful CLI tool written in Rust for managing Git worktrees (called "phantoms") with enhanced functionality. This is a Rust port of the original TypeScript implementation, providing memory safety, performance, and a single static binary. For detailed project information, features, and usage, see [](./README.md).

## Development Guidelines
- All files, issues, and pull requests in this repository must be written in English
- Follow existing Rust code conventions and patterns when making changes
- Test all changes before committing
- Always run `make check && make lint && make check-format && cargo test` before committing
- Other rules are written in the [](./CONTRIBUTING.md).

### Testing

- Use `cargo test` to run all tests
- Use `cargo test test_name` to run a specific test
- Use `cargo test module::tests` to run tests for a specific module
- Do not create and run temporary files for testing. Instead, use the mock-based testing strategy
- To verify that implemented features work correctly, write tests using the CommandExecutor abstraction instead of directly executing commands

## Project Structure
- `README.md` - Main project documentation
- `CONTRIBUTING.md` - Contribution guidelines  
- `Cargo.toml` - Rust package manifest and dependencies
- `docs/` - Additional documentation files
  - `architecture.md` - System design and architecture
  - `testing-guide.md` - Comprehensive testing documentation
  - `error-handling-guide.md` - Error handling patterns and best practices
  - `command-executor-guide.md` - Guide to the command execution abstraction
- `src/` - Rust source code following Single Responsibility Principle
  - `main.rs` - Application entry point
  - `lib.rs` - Library root
  - `cli/` - CLI-specific layer (handles user interaction)
    - `commands/` - Command definitions using Clap
    - `handlers/` - Command handlers (orchestration only)
    - `output.rs` - Output formatting and display
    - `error.rs` - CLI error handling and exit codes
  - `core/` - Business logic layer (framework-agnostic)
    - `command_executor.rs` - Command execution abstraction trait
    - `executors/` - Real and mock implementations
    - `filesystem.rs` - Filesystem abstraction trait
    - `filesystems/` - Real and mock implementations
    - `exit_handler.rs` - Process exit abstraction
  - `worktree/` - Worktree management operations
    - `create.rs`, `delete.rs`, `list.rs`, `attach.rs` - Core operations
    - `validate.rs` - Validation logic with property-based tests
    - `file_copier.rs` - Concurrent file copying
  - `process/` - Process and terminal operations
    - `spawn.rs`, `exec.rs`, `shell.rs` - Process management
    - `tmux.rs`, `kitty.rs` - Terminal multiplexer integrations
    - `fzf.rs` - Interactive selection support
  - `git/` - Git operations
    - `backend.rs` - GitBackend trait definition
    - `command_backend.rs` - Command-line git implementation
    - `libs/` - Individual git operation implementations
  - `config/` - Configuration management
    - `loader.rs` - Config loading with JSON/TOML support
    - `migrate.rs` - TypeScript config migration
- `tests/` - Integration tests
- `benches/` - Performance benchmarks

## Architecture Principles
- **Single Responsibility Principle**: Each module has one clear responsibility
- **Separation of Concerns**: CLI, business logic, and git operations are separated
- **Testability**: Mock-based testing with CommandExecutor abstraction
- **No Code Duplication**: Common operations are centralized
- **Clear Dependencies**: Dependencies flow from CLI → Core (including Git operations)
- **Performance First**: Zero-cost abstractions, smart pointer usage, iterator optimization
- **Error Handling**: Structured approach with thiserror for libraries, anyhow for CLI

## Important Notes
- Use English for all communications and documentation
- Maintain consistency with existing Rust code style
- Use the mock-based testing strategy for reliable, deterministic tests
- Core modules should not have CLI-specific dependencies
- All external command execution should use the CommandExecutor trait
- Prefer generics over trait objects for zero-cost abstractions
- Run benchmarks when making performance-related changes
- Be sure to read the `README.md` and `CONTRIBUTING.md` files for detailed project information and contribution guidelines
