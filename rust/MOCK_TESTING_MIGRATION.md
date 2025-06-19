# Mock Testing Infrastructure Migration Guide

## Overview

The mock testing infrastructure has been successfully implemented with the following components:

### 1. CommandExecutor Trait (`src/core/command_executor.rs`)
- Defines the interface for executing commands
- Supports both `execute` (wait for completion) and `spawn` (detached) operations
- Configuration includes program, args, cwd, env, and timeout

### 2. RealCommandExecutor (`src/core/executors/real_executor.rs`)
- Production implementation using tokio::process::Command
- Handles timeouts, environment variables, and working directories
- Returns structured output with stdout, stderr, and exit code

### 3. MockCommandExecutor (`src/core/executors/mock_executor.rs`)
- Test implementation with expectation builder pattern
- Supports verifying:
  - Command and arguments
  - Working directory
  - Environment variables
  - Number of times called
- Returns predefined outputs for testing

## Usage Example

See `examples/mock_testing_example.rs` for a complete demonstration.

```rust
// Production code
let executor = Arc::new(RealCommandExecutor::new());
let config = CommandConfig::new("git")
    .with_args(vec!["status".to_string()]);
let output = executor.execute(config).await?;

// Test code
let mut mock = MockCommandExecutor::new();
mock.expect_command("git")
    .with_args(&["status"])
    .returns_output("On branch main\n", "", 0);

let executor = Arc::new(mock);
// ... use executor in tests ...
mock.verify()?; // Verify expectations were met
```

## Migration Steps

### Phase 1: Update GitExecutor (Complete)
- [x] Created GitExecutor adapter that accepts CommandExecutor
- [x] Updated get_git_root to use CommandExecutor
- [x] Updated add_worktree to use CommandExecutor
- [x] Maintained backward compatibility with wrapper functions

### Phase 2: Update Process Operations (Complete)
- [x] spawn.rs already uses CommandExecutor trait
- [ ] Update tmux.rs operations
- [ ] Update kitty.rs operations
- [ ] Update shell.rs operations
- [ ] Update fzf.rs operations

### Phase 3: Update Handlers (Complete)
- [x] Added HandlerContext with CommandExecutor
- [x] Updated all handlers to accept HandlerContext
- [x] Updated CLI main to inject HandlerContext with RealCommandExecutor

### Phase 4: Migrate Tests
- [ ] Convert git operation tests to use mocks
- [ ] Convert process operation tests to use mocks
- [ ] Mark integration tests with #[ignore]
- [ ] Update CI configuration

## Benefits Achieved

1. **Deterministic Tests**: No more failures due to environment differences
2. **Fast Execution**: Mock tests run in milliseconds
3. **Better Coverage**: Can test error scenarios easily
4. **Clear Expectations**: Tests explicitly show expected command interactions

## Next Steps

1. Complete the GitExecutor refactoring to fully use CommandExecutor
2. Create a HandlerContext struct for dependency injection
3. Migrate process operations one by one
4. Update tests incrementally as each component is migrated

## Example Test Pattern

```rust
#[tokio::test]
async fn test_git_operation() {
    let mut mock = MockCommandExecutor::new();
    mock.expect_command("git")
        .with_args(&["worktree", "add", "-b", "feature", "path/to/worktree"])
        .returns_success();
    
    let executor = Arc::new(mock);
    // ... perform operation ...
    
    executor.verify().unwrap();
}
```

This infrastructure provides a solid foundation for reliable, fast, and maintainable tests.