# Serial Tests Investigation

## Issue
Some tests in the phantom codebase use `#[serial]` annotation from the `serial_test` crate, which forces them to run sequentially rather than in parallel. This impacts test performance.

## Root Cause
The tests that require serial execution are those that:
1. Change the current working directory using `std::env::set_current_dir()`
2. Test functions that rely on the current working directory (e.g., `get_git_root()`)

## Files Affected
- `src/git/libs/get_git_root.rs` - Tests change directory to test from within worktrees
- `src/git/libs/get_current_worktree.rs` - Tests change directory to detect current worktree
- `src/git/libs/list_worktrees.rs` - Tests change directory for worktree operations
- `src/process/shell.rs` - Tests may change directory for shell operations

## Why This Can't Be Easily Fixed

### 1. Git Commands Depend on Current Directory
Many git commands behave differently based on where they're executed:
- `git rev-parse --git-common-dir` returns different results from main repo vs worktree
- `git worktree list` shows different current markers based on location
- Some commands only work from within a git repository

### 2. Production Code Uses Current Directory
The production implementations often need to use the actual current directory:
- `get_git_root()` uses `std::env::current_dir()` to resolve relative paths
- This matches real-world usage where commands are run from various locations

### 3. CommandExecutor Helps But Doesn't Solve Everything
While CommandExecutor allows setting `cwd` for commands, the issue is that:
- Some functions need to resolve relative paths against the current directory
- The behavior of git commands changes based on the execution location
- Testing these behaviors requires actually being in those directories

## Alternatives Considered

### 1. Refactor to Accept Path Parameters
**Pros**: Would allow parallel testing
**Cons**: Would change the API and make the functions less convenient to use

### 2. Use Mock-Only Testing
**Pros**: No directory changes needed
**Cons**: Wouldn't test the actual git command behavior in different contexts

### 3. Create Separate Test Binaries
**Pros**: Each binary could run in parallel
**Cons**: More complex test setup and longer compile times

## Recommendation

Keep the serial tests as they are because:

1. **Correctness over Speed**: The tests accurately verify behavior in different directory contexts
2. **Limited Impact**: Only affects a small number of tests (< 10)
3. **Fast Enough**: The serial tests still run quickly (< 1 second total)
4. **Matches Reality**: Tests mirror how the code is actually used in practice

## Future Considerations

If test performance becomes a problem:
1. Group all serial tests into a single test module to minimize switching
2. Consider running serial tests in a separate CI job
3. Investigate using separate processes for tests that need isolation

## Conclusion

The serial tests are a necessary trade-off to ensure correct behavior when working with git commands that depend on the current directory. The performance impact is minimal and the tests accurately reflect real-world usage patterns.