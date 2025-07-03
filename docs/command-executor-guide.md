# CommandExecutor Migration Guide

This guide documents how to migrate operations to use the CommandExecutor pattern, enabling mock testing throughout the Phantom codebase.

## Overview

The CommandExecutor pattern is the foundation of Phantom's testing infrastructure. It abstracts all external command execution, allowing tests to run without executing real commands.

## Migration Pattern

Every operation that executes external commands must follow this pattern:

### 1. Create Function with Executor Parameter

```rust
use crate::core::command_executor::CommandExecutor;
use std::sync::Arc;

/// New function that accepts CommandExecutor
pub async fn operation_name_with_executor(
    executor: Arc<dyn CommandExecutor>,
    // ... other parameters ...
) -> Result<ReturnType> {
    // For git operations, use GitExecutor adapter:
    let git_executor = GitExecutor::new(executor);
    
    // For direct command execution:
    let config = CommandConfig::new("command")
        .with_args(vec!["arg1".to_string()])
        .with_cwd(path);
    let output = executor.execute(config).await?;
    
    // Process output...
}
```

### 2. Create Backward Compatible Wrapper

```rust
/// Original function signature for compatibility
pub async fn operation_name(
    // ... original parameters ...
) -> Result<ReturnType> {
    operation_name_with_executor(
        Arc::new(RealCommandExecutor),
        // ... parameters ...
    ).await
}
```

### 3. Update Call Sites in Handlers

In test mode, handlers should use the executor version:

```rust
let result = if cfg!(test) {
    operation_name_with_executor(context.executor.clone(), params).await?
} else {
    operation_name(params).await?
};
```

## Migration Examples

### Git Operations

All git operations have been migrated following this pattern:

```rust
// Before
pub async fn list_worktrees(git_root: &Path) -> Result<Vec<Worktree>> {
    let executor = GitExecutor::new();
    // ...
}

// After
pub async fn list_worktrees_with_executor(
    executor: Arc<dyn CommandExecutor>,
    git_root: &Path,
) -> Result<Vec<Worktree>> {
    let git_executor = GitExecutor::new(executor);
    // ...
}

pub async fn list_worktrees(git_root: &Path) -> Result<Vec<Worktree>> {
    list_worktrees_with_executor(Arc::new(RealCommandExecutor), git_root).await
}
```

### Process Operations

Process operations (tmux, kitty, fzf, shell) follow the same pattern:

```rust
// FZF example
pub async fn is_fzf_available_with_executor(
    executor: Arc<dyn CommandExecutor>
) -> bool {
    let config = CommandConfig::new("fzf")
        .with_args(vec!["--version".to_string()]);
    
    match executor.execute(config).await {
        Ok(output) => output.exit_code == 0,
        Err(_) => false,
    }
}
```

## Completed Migrations

### Git Operations (13/13) ✅
- `get_git_root` / `get_git_root_with_executor`
- `add_worktree` / `add_worktree_with_executor`
- `list_worktrees` / `list_worktrees_with_executor`
- `get_worktree_branch` / `get_worktree_branch_with_executor`
- `attach_worktree` / `attach_worktree_with_executor`
- `branch_exists` / `branch_exists_with_executor`
- `create_branch` / `create_branch_with_executor`
- `get_current_branch` / `get_current_branch_with_executor`
- `get_current_worktree` / `get_current_worktree_with_executor`
- `is_inside_work_tree` / `is_inside_work_tree_with_executor`
- `current_commit` / `current_commit_with_executor`
- `list_branches` / `list_branches_with_executor`
- `remove_worktree` / `remove_worktree_with_executor`

### Process Operations ✅
- **Shell**: `spawn_shell_in_worktree_with_executor`
- **Exec**: `exec_in_worktree_with_executor`
- **Tmux**: `execute_tmux_command_with_executor`
- **Kitty**: `execute_kitty_command_with_executor`
- **FZF**: `is_fzf_available_with_executor`, `run_fzf_with_input_with_executor`
- **Worktree Selection**: `select_worktree_with_fzf_with_executor`

## Testing with Mocks

### Setting Up Expectations

```rust
let mut mock = MockCommandExecutor::new();

// Expect specific command with arguments
mock.expect_command("git")
    .with_args(&["worktree", "list", "--porcelain"])
    .in_dir("/repo/path")
    .returns_output("worktree /repo\nHEAD abc123\n", "", 0);

// Expect command to be called exactly twice
mock.expect_command("git")
    .with_args(&["status"])
    .times(2)
    .returns_output("", "", 0);

// Expect command with partial environment
mock.expect_command("tmux")
    .with_env("PHANTOM_ACTIVE", "1")
    .returns_output("", "", 0);
```

### Using in Tests

```rust
#[tokio::test]
async fn test_operation() {
    let mut mock = MockCommandExecutor::new();
    // Set up expectations...
    
    let result = operation_with_executor(
        Arc::new(mock),
        // parameters
    ).await;
    
    assert!(result.is_ok());
}
```

## Important Considerations

### 1. Complete Migration Required
Partial migration doesn't work. If a function calls another function that hasn't been migrated, it will bypass mocks and execute real commands.

### 2. Maintain Backward Compatibility
Always provide a wrapper function with the original signature to avoid breaking existing code.

### 3. Test Mode Detection
Handlers should detect test mode using `cfg!(test)` to use executor versions during testing.

### 4. Error Handling
The CommandExecutor returns structured errors. Map these appropriately to your domain errors.

## Benefits

- **Deterministic Tests**: No dependency on external commands or environment
- **Fast Test Execution**: No process spawning overhead
- **Precise Testing**: Verify exact commands and arguments
- **CI Reliability**: Tests pass consistently across different environments
- **Better Coverage**: Can test error conditions and edge cases easily

## Future Work

While all current operations have been migrated, new features should:
1. Always use CommandExecutor from the start
2. Follow the established patterns
3. Include comprehensive mock tests
4. Document any special considerations