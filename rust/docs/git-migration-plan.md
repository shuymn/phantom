# Git Operations Migration Plan

## Overview
This document tracks the migration of git operations to use CommandExecutor for enabling mock testing in the Phantom Rust project.

## Migration Status

### ✅ Already Migrated
- `get_git_root` - Migrated with `get_git_root_with_executor`
- `add_worktree` - Migrated with `add_worktree_with_executor`
- `list_worktrees` - Migrated with `list_worktrees_with_executor`
- `get_current_branch` - Migrated with `get_current_branch_with_executor`
- `branch_exists` - Migrated with `branch_exists_with_executor`
- `attach_worktree` - Migrated with `attach_worktree_with_executor`
- `get_current_worktree` - Migrated with `get_current_worktree_with_executor`
- Functions in `worktree/list.rs`:
  - `get_worktree_branch` - Migrated with `get_worktree_branch_with_executor`
  - `get_worktree_status` - Migrated with `get_worktree_status_with_executor`
  - `get_worktree_info` - Migrated with `get_worktree_info_with_executor`
  - `list_worktrees` - Migrated with `list_worktrees_with_executor`
- Functions in `worktree/delete.rs`:
  - `delete_worktree` - Migrated with `delete_worktree_with_executor`
  - `get_worktree_status` - Migrated with `get_worktree_status_with_executor`

### 🔄 Need Migration (Priority Order)

#### Priority 1: Blocking Other Handlers
1. **`create_branch`** (src/git/libs/create_branch.rs)
   - Used by: create handler for branch creation
   - Critical for enabling create handler mock tests

## Migration Pattern

Each migration should follow this pattern:

```rust
// 1. Add new function with executor parameter
pub async fn function_name_with_executor(
    executor: Arc<dyn CommandExecutor>,
    // ... other parameters
) -> Result<ReturnType> {
    let git_executor = GitExecutor::new(executor);
    // ... implementation using git_executor
}

// 2. Keep original function as wrapper
pub async fn function_name(/* ... parameters */) -> Result<ReturnType> {
    use crate::core::executors::RealCommandExecutor;
    function_name_with_executor(Arc::new(RealCommandExecutor), /* ... */).await
}
```

## Implementation Order

1. Start with `list_worktrees` as it blocks the most important handler
2. Migrate `get_worktree_branch` and `get_worktree_status` in `worktree/list.rs`
3. Continue with other functions in priority order
4. Update all callers to use the `_with_executor` variants where appropriate
5. Add comprehensive tests for each migrated function

## Testing Strategy

After each migration:
1. Ensure existing tests still pass
2. Add new tests using MockCommandExecutor
3. Verify handler tests can now mock the git operations
4. Document any edge cases or special considerations

## Migration Progress Notes

### List Handler Migration (Completed)
- Successfully migrated `list_worktrees` and all functions in `worktree/list.rs`
- Added comprehensive tests for the list handler using MockCommandExecutor
- All 5 handler tests passing, proving that mock testing is now possible
- The migration pattern has been validated and can be applied to other functions

### Key Learnings
- The MockCommandExecutor uses `in_dir()` method, not `with_cwd()`
- Handler tests need to mock both the git operations and any status checks
- The _with_executor pattern allows backward compatibility while enabling testing