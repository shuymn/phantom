# Git Operations Migration Guide

## Status: ✅ GIT OPERATIONS MIGRATION COMPLETE!

All git operations have been successfully migrated to use CommandExecutor. The remaining work is migrating process operations (tmux, kitty, fzf, shell).

## Overview

This guide documents the systematic migration of all git operations to use CommandExecutor. This migration is **critical** for enabling mock testing throughout the codebase.

## Why Complete Migration is Required

Our testing revealed a fundamental issue: **partial migration doesn't work**.

```
Handler → list_worktrees() → GitExecutor::new() → Real Git Commands
   ↑                                ↑
   We mock here                     But this bypasses our mocks!
```

When handlers are tested with mocks but call functions that still use the old GitExecutor, those functions execute real git commands, completely bypassing our test infrastructure.

## Migration Pattern

Every git operation must follow this exact pattern (based on `get_git_root` and `add_worktree`):

### 1. Add CommandExecutor Parameter Function

```rust
use crate::core::command_executor::CommandExecutor;
use crate::git::git_executor_adapter::GitExecutor;
use std::sync::Arc;

/// Function with CommandExecutor parameter
pub async fn operation_name_with_executor(
    executor: Arc<dyn CommandExecutor>,
    // ... other parameters ...
) -> Result<ReturnType> {
    let git_executor = GitExecutor::new(executor);
    // If working directory needed:
    // let git_executor = GitExecutor::new(executor).with_cwd(path);
    
    // ... implementation using git_executor ...
}
```

### 2. Add Backward Compatibility Wrapper

```rust
/// Backward compatible function using default executor
pub async fn operation_name(
    // ... same parameters as before ...
) -> Result<ReturnType> {
    use crate::core::executors::RealCommandExecutor;
    operation_name_with_executor(
        Arc::new(RealCommandExecutor),
        // ... forward parameters ...
    ).await
}
```

### 3. Update All Call Sites

Find all places that create `GitExecutor::new()` or `GitExecutor::with_cwd()` and update them to use the new functions.

## Operations Requiring Migration

### Critical Priority (Block Handler Tests)

These operations are directly called by handlers and must be migrated first:

- [x] `list_worktrees` - Used by list handler - **COMPLETED**
- [x] `get_worktree_branch` - Used by list handler - **COMPLETED**
- [x] `get_worktree_status` - Used by list and delete handlers - **COMPLETED**
- [x] `attach_worktree` - Used by attach handler - **COMPLETED**
- [x] `delete_worktree` / `remove_worktree` - Used by delete handler - **COMPLETED**
- [x] `branch_exists` - Used by create and attach handlers - **COMPLETED**
- [x] `get_current_branch` - **COMPLETED** - Migrated with `get_current_branch_with_executor`
- [x] `get_current_worktree` - Used by multiple handlers - **COMPLETED**

### High Priority (Core Operations)

- [x] `get_git_root` - **COMPLETED** (template example)
- [x] `add_worktree` - **COMPLETED** (template example)
- [x] `is_inside_work_tree` - Basic git check - **COMPLETED**
- [x] `current_commit` - Version info - **COMPLETED**

### Medium Priority (GitBackend Implementation)

Operations in `command_backend.rs` that need migration:

- [x] `list_branches` - List all branches - **COMPLETED**
- [x] `remove_worktree` - Remove worktree - **COMPLETED**

Note: The `status` operation is not needed as a separate function. Status checking is already implemented via `get_worktree_status_with_executor` in `worktree/list.rs`.

### Operations that should be REMOVED from GitBackend (not needed by Phantom):

- `init` - Phantom works with existing repos only
- `clone` - Phantom manages worktrees, not cloning
- `add` - Phantom doesn't stage files
- `commit` - Phantom doesn't create commits
- `checkout` - Phantom uses worktrees, not branch switching
- `execute` - Too generic, specific operations should be used instead

### Process Operations (After Git Migration)

- [ ] `tmux.rs` - Terminal multiplexer operations
- [ ] `kitty.rs` - Kitty terminal operations
- [ ] `shell.rs` - Shell command operations
- [ ] `fzf.rs` - Fuzzy finder operations

## Migration Checklist

For each operation:

1. **Identify the function location**
   - Check `src/git/libs/` for dedicated modules
   - Check `src/git/command_backend.rs` for GitBackend methods
   - Check `src/worktree/` for worktree-specific operations

2. **Create the executor version**
   - Add `_with_executor` variant that accepts `Arc<dyn CommandExecutor>`
   - Use `GitExecutor::new(executor)` instead of `GitExecutor::new()`
   - Preserve all existing logic

3. **Add backward compatibility**
   - Keep original function signature
   - Call new function with `RealCommandExecutor`

4. **Update tests**
   - Add mock tests for the new function
   - Keep existing tests working with compatibility wrapper

5. **Document the change**
   - Update this checklist
   - Note any special considerations

## Example: Migrating `list_worktrees`

Before:
```rust
pub async fn list_worktrees(cwd: &Path) -> Result<Vec<Worktree>> {
    let executor = GitExecutor::with_cwd(cwd);
    // ... rest of implementation
}
```

After:
```rust
pub async fn list_worktrees_with_executor(
    executor: Arc<dyn CommandExecutor>,
    cwd: &Path
) -> Result<Vec<Worktree>> {
    let git_executor = GitExecutor::new(executor).with_cwd(cwd);
    // ... rest of implementation
}

pub async fn list_worktrees(cwd: &Path) -> Result<Vec<Worktree>> {
    use crate::core::executors::RealCommandExecutor;
    list_worktrees_with_executor(Arc::new(RealCommandExecutor), cwd).await
}
```

## Testing After Migration

Once an operation is migrated, you can write effective mock tests:

```rust
#[tokio::test]
async fn test_list_worktrees_empty() {
    let mut mock = MockCommandExecutor::new();
    mock.expect_command("git")
        .with_args(&["worktree", "list", "--porcelain"])
        .returns_output("", "", 0);
    
    let result = list_worktrees_with_executor(
        Arc::new(mock), 
        Path::new("/test")
    ).await.unwrap();
    
    assert!(result.is_empty());
}
```

## Success Criteria

- [x] All git operations accept CommandExecutor ✅
- [x] All existing tests still pass ✅
- [x] Handler mock tests work without executing real git commands ✅
- [x] No direct GitExecutor::new() calls remain (except in compatibility wrappers) ✅
- [ ] Process operations migrated similarly (in progress)

## Notes

- This is tedious but necessary work
- Each migration is straightforward - follow the pattern exactly
- The payoff is enormous: millisecond tests, no environment dependencies, testable error scenarios
- Complete git operations first, then process operations
- Update this guide as you discover edge cases or patterns

Remember: **Every unmigrated operation is a blocker for effective testing**.

## Migration Progress Notes

### List Handler Migration (Completed)
- Successfully migrated `list_worktrees` and all functions in `worktree/list.rs`
- Added comprehensive tests for the list handler using MockCommandExecutor
- All 5 handler tests passing, proving that mock testing is now possible
- The migration pattern has been validated and can be applied to other functions

### get_current_branch Migration (Completed)
- Successfully migrated `get_current_branch` with `get_current_branch_with_executor`
- Added mock tests demonstrating usage patterns
- Function is ready for use in handlers that need current branch information
- Note: The completion handler currently uses static shell scripts and doesn't call git functions

### Key Learnings
- The MockCommandExecutor uses `in_dir()` method, not `with_cwd()`
- Handler tests need to mock both the git operations and any status checks
- The _with_executor pattern allows backward compatibility while enabling testing