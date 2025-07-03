# Test Race Condition Fix

## Problem
When running tests with `make test` (which uses `--all-features`), two tests were failing:
- `git::libs::get_current_worktree::tests::test_get_current_worktree_in_worktree`
- `git::libs::list_worktrees::tests::test_list_worktrees_multiple`

### Error Messages
```
fatal: Unable to create '.git/worktrees/test-worktree-41681-1750310237371/index.lock': File exists.
Another git process seems to be running in this repository
```

## Root Cause
These tests were using real git commands to create worktrees. When tests run in parallel (default behavior), multiple tests can try to manipulate the same git repository simultaneously, causing lock file conflicts.

## Solution
Added `#[serial_test::serial]` annotations to force sequential execution:
- Fixed incorrect `#[serial]` annotation in `get_current_worktree.rs` to `#[serial_test::serial]`
- Added `#[serial_test::serial]` to `test_list_worktrees_multiple` in `list_worktrees.rs`

## Why This Validates the Mock Strategy
This issue perfectly demonstrates why the mock implementation strategy is correct:
1. **Real commands cause race conditions**: Tests using actual git commands can interfere with each other
2. **Mocks eliminate external dependencies**: Mock tests don't touch the filesystem or run external commands
3. **Faster and more reliable**: Mock tests run faster and don't have environment-specific failures

## Next Steps
To fully resolve this class of issues:
1. Write mock tests for the remaining git operations that lack them:
   - `list_worktrees`
   - `get_git_root`
   - `attach_worktree`
   - `add_worktree`
2. Gradually replace integration tests that use real git commands with mock-based unit tests
3. Keep a small set of integration tests for end-to-end validation, but run them serially

## Temporary Workaround
The `#[serial_test::serial]` annotation is a temporary fix. The proper solution is to use the mock infrastructure for these tests, which would:
- Eliminate the race condition entirely
- Make tests faster
- Make tests more predictable
- Allow testing of error conditions that are hard to reproduce with real commands