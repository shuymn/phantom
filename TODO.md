# Phantom Rust Migration TODO

This document tracks all tasks for migrating Phantom from TypeScript to Rust.
Tasks are organized by phase according to the migration plan.

## Pre-Migration Setup

- [x] Finalize Unix-only decision with stakeholders
- [ ] Create migration tracking project board on GitHub
- [x] Set up migration branch strategy (e.g., `rust-migration` branch)
- [ ] Document current TypeScript version behavior for reference

## Phase 1: Foundation & Core Types (Weeks 1-2)

### Project Setup

- [x] Initialize Rust project with `cargo init`
- [x] Create `Cargo.toml` with all dependencies
- [x] Set up project directory structure
- [x] Configure `.gitignore` for Rust
- [x] Set up `rust-toolchain.toml` for consistent toolchain version
- [x] Configure rustfmt and clippy settings

### Core Implementation

- [x] Implement `PhantomError` enum with thiserror
- [x] Create core domain types:
  - [x] `Worktree` struct
  - [x] `GitConfig` struct
  - [x] `PhantomConfig` struct
- [x] Set up async runtime with Tokio
- [x] Implement logging with tracing/tracing-subscriber
- [x] Create basic error handling utilities
- [x] Set up module structure (core, git, worktree, process, cli)

### Testing Infrastructure

- [x] Set up test utilities module
- [x] Create test fixtures for git repositories
- [x] Set up property-based testing with proptest

## Phase 2: Git Operations - Command-based (Weeks 3-5)

### Stage 1: Command Executor

- [x] Implement `GitExecutor` struct
- [x] Create async `run` method for git commands
- [x] Add proper error handling for git command failures
- [x] Implement command output parsing utilities
- [x] Add timeout handling for git operations

### Stage 2: Git Operations

- [x] Port `add_worktree` function
- [x] Port `list_worktrees` function
- [x] Port `get_current_branch` function
- [x] Port `get_current_worktree` function
- [x] Port `get_git_root` function
- [x] Port `branch_exists` function
- [x] Port `attach_worktree` function
- [x] Create unit tests for each operation
- [x] Add integration tests with real git repos

### Stage 3: Git Backend Abstraction

- [x] Define `GitBackend` trait
- [x] Implement `CommandBackend` struct
- [x] Add feature flag for future libgit2 support
- [x] Create backend factory function
- [x] Write tests for backend abstraction

## Phase 3: Async File Operations & Worktree Management (Weeks 6-7)

### Worktree Operations

- [x] Implement async `create_worktree` function
- [x] Implement async `delete_worktree` function
- [x] Implement async `list_worktrees` function (completed in Phase 2)
- [x] Implement `attach_to_branch` function (completed as `attach_worktree` in Phase 2)
- [x] Port worktree name validation
- [x] Add worktree selection with fzf integration

### File Operations

- [x] Implement async file copier
- [x] ~~Add `.gitignore` pattern matching~~ (Removed - not in TypeScript version)
- [x] ~~Implement parallel file copying with concurrency limits~~ (Removed - not in TypeScript version)
- [x] ~~Add progress reporting for file operations~~ (Removed - not in TypeScript version)

### Configuration

- [x] Define configuration structs with serde
- [x] Implement JSON config loading (maintain compatibility)
- [x] Add TOML config support (new feature)
- [x] Implement config validation
- [x] Add config migration utility (JSON to TOML)
- [x] Create default config generation

### Additional Phase 3 Tasks (added during implementation)

- [x] Implement property-based testing with proptest for worktree validation
- [x] ~~Add snapshot testing with insta~~ (Removed - difficult to maintain)
- [x] Remove unnecessary complexity not present in TypeScript version:
  - [x] Remove gitignore support from file copier
  - [x] Remove parallel_copier module
  - [x] Remove progress reporting module
  - [x] Simplify create_worktree to match TypeScript implementation

## Phase 4: Process & Terminal Integration (Weeks 8-9)

### Process Management

- [x] Implement async process spawning
- [x] Add Unix-specific shell detection
- [x] Implement `spawn_shell` function
- [x] Add `exec_in_dir` function
- [x] Implement proper signal handling
- [x] Add process timeout handling

### Terminal Multiplexer Support

- [ ] Implement tmux integration:
  - [ ] Session creation
  - [ ] Window management
  - [ ] Pane splitting
- [ ] Implement Kitty integration:
  - [ ] OSC sequence support
  - [ ] Tab/window creation
- [ ] Add multiplexer detection
- [ ] Create fallback for unsupported terminals

### Interactive Features

- [ ] Implement fzf integration via subprocess
- [ ] Add TTY detection and handling
- [ ] Implement interactive prompts
- [ ] Add color output support
- [ ] Handle NO_COLOR and FORCE_COLOR env vars

## Phase 5: CLI Implementation (Weeks 10-11)

### CLI Structure

- [ ] Define CLI structs with clap derive
- [ ] Implement all subcommands:
  - [ ] `create` command
  - [ ] `attach` command
  - [ ] `list` command
  - [ ] `where` command
  - [ ] `delete` command
  - [ ] `exec` command
  - [ ] `shell` command
  - [ ] `version` command
  - [ ] `completion` command

### Command Handlers

- [ ] Implement async handler for each command
- [ ] Port output formatting (maintain compatibility)
- [ ] Add error handling with proper exit codes
- [ ] Implement help text generation
- [ ] Add shell completion generation:
  - [ ] Fish completion
  - [ ] Zsh completion
  - [ ] Bash completion

### CLI Features

- [ ] Add verbose/quiet flags
- [ ] Implement dry-run mode
- [ ] Add JSON output format option
- [ ] Maintain exact CLI compatibility with TypeScript version

## Phase 6: Testing & Distribution (Weeks 12-13)

### Comprehensive Testing

- [ ] Write unit tests for all modules (target 85% coverage)
- [ ] Create integration tests for all commands
- [ ] Add property-based tests for critical functions
- [ ] Implement snapshot tests for CLI output
- [ ] Create end-to-end test scenarios
- [ ] Add regression tests based on TypeScript behavior


### Distribution Setup

- [ ] Configure GitHub Actions workflow:
  - [ ] Linux x86_64 build
  - [ ] Linux aarch64 build
  - [ ] macOS x86_64 build
  - [ ] macOS aarch64 build
- [ ] Create release automation
- [ ] Set up binary signing (macOS)
- [ ] Create installation scripts
- [ ] Update Homebrew formula
- [ ] Add cargo install support
- [ ] Create .deb and .rpm packages

### Documentation

- [ ] Update README for Rust version
- [ ] Create migration guide for users
- [ ] Generate API documentation with cargo doc
- [ ] Update installation instructions
- [ ] Create troubleshooting guide

## Post-Migration Tasks

- [ ] Announce Rust version availability
- [ ] Gather user feedback
- [ ] Create issues for reported bugs
- [ ] Plan deprecation timeline for TypeScript version
- [ ] Consider additional Rust-specific features:
  - [ ] Parallel worktree operations
  - [ ] Native git protocol support (libgit2)
  - [ ] Plugin system
  - [ ] Configuration profiles

## Continuous Tasks

These tasks should be done throughout the migration:

- [ ] Keep TypeScript version maintained
- [ ] Run parallel CI for both versions
- [ ] Update migration progress weekly
- [ ] Conduct code reviews for each phase
- [ ] Maintain feature parity tests
- [ ] Document any behavioral differences
- [ ] Communicate progress to users

## Success Criteria Checklist

- [ ] All commands work identically to TypeScript version
- [ ] Binary size < 5MB (stripped)
- [ ] Test coverage > 85%
- [ ] Zero runtime dependencies
- [ ] Single binary distribution working
- [ ] All existing tests passing
- [ ] User acceptance testing completed

## Notes

- Priority: Focus on command compatibility first
- Testing: Every feature must have tests before marking complete
- Review: Each phase requires code review before proceeding
- Rollback: Keep ability to rollback to TypeScript version at any point
