# Error Handling Refactor Test Fixes

This document summarizes the fixes made to tests that were failing after the error handling refactor.

## Fixed Tests

### 1. `test_create_worktree_invalid_name`
**Issue**: The `InvalidWorktreeName` error had an empty name field when converted from `WorktreeError`.
**Fix**: Updated the `create_worktree` functions to map the error and populate the name field correctly.

### 2. `test_delete_worktree_with_uncommitted_changes`
**Issue**: Expected `FileOperation` error but now returns specific `WorktreeHasUncommittedChanges`.
**Fix**: 
- Updated the error type returned by `delete_worktree` to use `PhantomError::WorktreeHasUncommittedChanges`
- Fixed the test assertion to check for the correct error type
- Fixed the worktree name in the assertion (was "dirty-worktree", should be "feature")

### 3. `test_execute_nonexistent_command`
**Issue**: Error message assertion was checking for old format.
**Fix**: Updated test to check for `PhantomError::CommandNotFound` with the correct command name.

### 4. `test_attach_worktree_already_checked_out`
**Issue**: Git error messages changed format from string to structured `PhantomError::Git`.
**Fix**: Updated test to match against the structured error and check the `stderr` field.

### 5. `test_exec_in_dir_nonexistent_command`
**Issue**: Expected `ProcessExecutionError` but now returns `CommandNotFound`.
**Fix**: Updated test to check for `PhantomError::CommandNotFound` error type.

### 6. `test_error_handling` (fzf)
**Issue**: Error message format changed for `CommandNotFound`.
**Fix**: Updated assertion from "fzf command not found" to "Command 'fzf' not found".

### 7. `test_spawn_detached_failure`
**Issue**: Command name mismatch - config used "command-that-does-not-exist-xyz" but assertion checked for "nonexistent-command-xyz123".
**Fix**: Updated assertion to use the correct command name.

### 8. `test_run_fzf_error_cases`
**Issue**: Same as #6 - error message format changed.
**Fix**: Updated assertion to match new error format.

## Summary

All tests now properly handle the new structured error types introduced in the error handling refactor. The key changes were:
- Using specific error types instead of generic string errors
- Matching against structured error fields rather than string messages
- Ensuring error conversions preserve all necessary information