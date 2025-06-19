# Process Operations Migration Guide

## Overview

With all git operations successfully migrated to CommandExecutor, the remaining work is migrating process operations. This will complete the mock testing infrastructure and allow all handlers to be fully testable.

## Operations Requiring Migration

### 1. tmux.rs (High Priority)
- **Current state**: Uses `execute_command` and `spawn_process` from `spawn.rs`
- **Functions to migrate**:
  - `execute_tmux_command`
  - `tmux_session_exists`
  - Any other public functions that execute commands
- **Used by**: Multiple handlers for terminal multiplexer operations

### 2. fzf.rs (High Priority)
- **Current state**: Uses `Command::new("fzf")` directly
- **Functions to migrate**:
  - `select_item`
  - `select_multiple_items`
- **Used by**: Select operations for interactive choices

### 3. kitty.rs (Medium Priority)
- **Current state**: Uses `spawn_process` from `spawn.rs`
- **Functions to migrate**:
  - `execute_kitty_command`
- **Used by**: Kitty terminal-specific operations

### 4. shell.rs (Medium Priority)
- **Current state**: Uses `spawn_process` from `spawn.rs`
- **Functions to migrate**:
  - `execute_shell_command`
- **Used by**: Generic shell command execution

## Migration Pattern

Follow the same pattern as git operations:

```rust
// 1. Add _with_executor variant
pub async fn function_name_with_executor(
    executor: Arc<dyn CommandExecutor>,
    // ... other parameters ...
) -> Result<ReturnType> {
    // Use executor instead of direct command execution
}

// 2. Add backward compatibility wrapper
pub async fn function_name(
    // ... same parameters ...
) -> Result<ReturnType> {
    use crate::core::executors::RealCommandExecutor;
    function_name_with_executor(
        Arc::new(RealCommandExecutor),
        // ... forward parameters ...
    ).await
}
```

## Key Considerations

1. **spawn.rs functions**: The `execute_command` and `spawn_process` functions in `spawn.rs` should also be migrated or adapted to use CommandExecutor

2. **Command building**: Process operations often build complex command arguments. Ensure the command building logic is preserved when migrating

3. **Interactive commands**: Some operations (like fzf) are interactive. Consider how to mock these effectively

4. **Environment variables**: Many process operations set environment variables. Ensure these are properly handled in the migration

## Success Criteria

- [ ] All process operations accept CommandExecutor
- [ ] No direct `Command::new()` calls remain (except in executors)
- [ ] No direct `spawn_process` or `execute_command` calls (except through executors)
- [ ] Mock tests can be written for all process operations
- [ ] All handlers can be fully tested with mocks

## Next Steps

1. Start with tmux.rs as it's the most complex and widely used
2. Create mock tests for each migrated function to validate the pattern
3. Update handlers to use the new _with_executor variants
4. Document any special considerations or patterns discovered during migration