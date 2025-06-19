# Git Operations Migration Plan

## Overview
This document tracks the migration of git operations to use CommandExecutor for enabling mock testing in the Phantom Rust project.

## Migration Status

### ✅ Already Migrated (12/20+)
- `get_git_root` - Migrated with `get_git_root_with_executor`
- `add_worktree` - Migrated with `add_worktree_with_executor`
- `list_worktrees` - Migrated with `list_worktrees_with_executor`
- `get_current_branch` - Migrated with `get_current_branch_with_executor`
- `branch_exists` - Migrated with `branch_exists_with_executor`
- `attach_worktree` - Migrated with `attach_worktree_with_executor`
- `get_current_worktree` - Migrated with `get_current_worktree_with_executor`
- `create_branch` - Migrated with `create_branch_with_executor`
- `is_inside_work_tree` - Migrated with `is_inside_work_tree_with_executor`
- `current_commit` - Migrated with `current_commit_with_executor`
- Functions in `worktree/list.rs`:
  - `get_worktree_branch` - Migrated with `get_worktree_branch_with_executor`
  - `get_worktree_status` - Migrated with `get_worktree_status_with_executor`
  - `get_worktree_info` - Migrated with `get_worktree_info_with_executor`
  - `list_worktrees` - Migrated with `list_worktrees_with_executor`
- Functions in `worktree/delete.rs`:
  - `delete_worktree` - Migrated with `delete_worktree_with_executor`
  - `get_worktree_status` - Migrated with `get_worktree_status_with_executor`

### 🎯 Handler Testing Status
- **List Handler**: ✅ Fully testable - 5 comprehensive mock tests
- **Attach Handler**: ✅ Fully testable - 5 comprehensive mock tests  
- **Create Handler**: ✅ Partially testable - 5 mock tests (filesystem ops limit full mocking)
- **Delete Handler**: ⚠️ Partially testable - filesystem operations limit full mocking
- **Other Handlers**: 🚫 Blocked by remaining migrations

### 🔄 Need Migration (Priority Order)

#### Priority 1: Core Git Operations
1. **`checkout`** - Switch branches
2. **`list_branches`** - List all branches  
3. **`status`** - Get repository status
4. **`fetch`** - Fetch from remote
5. **`pull`** - Pull from remote
6. **`push`** - Push to remote

#### Priority 2: GitBackend Operations
- Operations in `command_backend.rs` that need migration
- These are lower priority as they're not directly used by handlers

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

### Attach Handler Migration (Completed)
- Successfully migrated `attach_worktree` and `branch_exists`
- Added 5 comprehensive mock tests for the attach handler
- All tests passing, handler is fully testable with mocks

### Delete Handler Migration (Partially Complete)
- Migrated `delete_worktree` and related functions to use CommandExecutor
- Discovered limitation: `validate_worktree_exists` uses filesystem operations
- Can only test early failures and document the limitation with ignored tests
- Future work: Abstract filesystem operations for complete testability

### Key Learnings
- The MockCommandExecutor uses `in_dir()` method, not `with_cwd()`
- Handler tests need to mock both the git operations and any status checks
- The _with_executor pattern allows backward compatibility while enabling testing
- **Important**: Filesystem operations (fs::metadata) also need abstraction for complete mock testing
- Dead code cleanup: Removed unused backward compatibility wrappers