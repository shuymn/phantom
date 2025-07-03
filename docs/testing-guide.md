# Testing Guide for Phantom

This guide consolidates all testing-related documentation for the Phantom Rust implementation.

## Overview

The Phantom project uses a mock-based testing strategy to ensure reliable, deterministic tests that are independent of the execution environment. This approach was developed to solve persistent issues with tests failing in CI due to environment differences, missing commands, and race conditions.

## Test Organization

Tests in Phantom should be organized as follows:

### Unit Tests
- **Location**: Within the same source file in a `mod tests` block at the bottom
- **Convention**: Do NOT create separate `*_test.rs` files for unit tests
- **Rationale**: Keeps tests close to the code they test, improving maintainability

```rust
// src/worktree/create.rs
pub fn create_worktree() { /* implementation */ }

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_create_worktree() {
        // test implementation
    }
}
```

### Integration Tests
- **Location**: In the `tests/` directory at the project root
- **Purpose**: Test interactions between modules and full workflows
- **Examples**: `e2e_scenarios.rs`, `cli_snapshots.rs`

## The Problem We Solved

Tests were frequently failing in CI due to:
- Different git versions/configurations across environments
- Missing external commands (tmux, fzf, kitty)
- Race conditions in parallel test execution
- Tests modifying global state (working directory)
- Environment-specific behavior differences

## The Solution: Command Execution Abstraction

We implemented a comprehensive abstraction layer that allows all external command execution to be mocked in tests.

### Core Components

#### 1. CommandExecutor Trait (`src/core/command_executor.rs`)

```rust
#[async_trait]
pub trait CommandExecutor: Send + Sync {
    async fn execute(&self, config: CommandConfig) -> Result<CommandOutput>;
    async fn spawn(&self, config: SpawnConfig) -> Result<SpawnOutput>;
}
```

- Defines the interface for executing commands
- Supports both `execute` (wait for completion) and `spawn` (detached) operations
- Configuration includes program, args, cwd, env, and timeout

#### 2. RealCommandExecutor (`src/core/executors/real_executor.rs`)

- Production implementation using tokio::process::Command
- Handles timeouts, environment variables, and working directories
- Returns structured output with stdout, stderr, and exit code

#### 3. MockCommandExecutor (`src/core/executors/mock_executor.rs`)

- Test implementation with expectation builder pattern
- Supports verifying command, arguments, working directory, and environment
- Returns predefined outputs for testing

### Additional Abstractions

#### FileSystem Trait
- Abstracts filesystem operations (read, write, exists, etc.)
- Enables testing without actual filesystem access
- MockFileSystem provides controlled responses

#### ExitHandler Trait
- Abstracts process::exit calls
- Allows testing commands that normally exit the process
- MockExitHandler captures exit codes for verification

## Usage Examples

### Basic Mock Usage

```rust
// In tests
let mut mock = MockCommandExecutor::new();
mock.expect_command("git")
    .with_args(&["status", "--porcelain"])
    .in_dir("/repo/path")
    .returns_output("M file.txt\n", "", 0);

let executor = Arc::new(mock);
// Use executor in code under test...
```

### Handler Testing Pattern

```rust
// Create test context with mocks
let mut mock_executor = MockCommandExecutor::new();
let mock_fs = MockFileSystem::new();
let mock_exit = MockExitHandler::new();

// Set up expectations...

let context = HandlerContext::new(
    Arc::new(mock_executor),
    Arc::new(mock_fs),
    Arc::new(mock_exit),
);

// Test handler
let result = handle(args, context).await;
```

See `examples/mock_testing_example.rs` for a complete working example.

## Key Lessons Learned

### 1. Partial Migration Doesn't Work

We discovered that mock testing requires **complete migration** of all dependencies:

```
Handler → list_worktrees() → GitExecutor → Real Commands
   ↑                             ↑
   We mock here                  But this still executes!
```

When handlers are tested with mocks but call functions that haven't been migrated, those functions execute real commands, bypassing our test infrastructure.

### 2. All External Dependencies Need Abstraction

Not just commands, but also:
- Filesystem operations
- Process exit calls
- Environment variable access
- Time-based operations

### 3. Backward Compatibility Pattern

To migrate incrementally while maintaining compatibility:

```rust
// New function with executor
pub async fn operation_with_executor(
    executor: Arc<dyn CommandExecutor>,
    /* params */
) -> Result<T> {
    // Implementation
}

// Backward compatible wrapper
pub async fn operation(/* params */) -> Result<T> {
    operation_with_executor(
        Arc::new(RealCommandExecutor),
        /* params */
    ).await
}
```

## Migration Status

### Completed ✅
- All git operations (13/13) migrated to CommandExecutor
- All process operations (tmux, fzf, kitty, shell) migrated
- All handlers accept dependencies via HandlerContext
- Comprehensive mock testing infrastructure in place
- 519 tests total, 0 ignored

### Testing Patterns Established
- Mock-based unit tests for handlers
- Integration tests using TestRepo
- E2E tests for complete workflows
- Snapshot tests for CLI output

## Best Practices

1. **Always use abstractions** - Never call external commands directly
2. **Test behavior, not implementation** - Focus on outcomes, not command details
3. **Keep mocks simple** - Only mock what's needed for the specific test
4. **Verify expectations** - Use mock verification to ensure correct calls
5. **Use TestRepo for integration tests** - Provides isolated git repositories

## Future Improvements

While the current testing infrastructure is complete and functional, potential improvements include:
- Performance optimization for test execution
- Better error messages from mock mismatches
- Integration with code coverage tools
- Automated detection of unmocked external calls

## References

- Example implementation: `examples/mock_testing_example.rs`
- Handler tests: See `mod tests` blocks in `src/cli/handlers/*.rs`
- Unit tests: See `mod tests` blocks in source files
- Integration tests: `tests/` directory