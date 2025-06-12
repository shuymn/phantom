# Tests Excluded Due to Interactive Behavior

This file documents tests that have been marked as `#[ignore]` because they require interactive input or external dependencies that cause them to hang during automated test runs.

## Excluded Tests

### process/fzf.rs
1. **test_select_with_fzf_empty_items** - Line 151
   - Already passes (not interactive with empty items)
   
2. **test_select_with_fzf_single_item** - Line 185
   - Reason: Spawns fzf process which waits for interactive input
   - Marked as: `#[ignore = "This test requires fzf and runs interactively"]`

3. **test_select_with_fzf_with_options** - Line 200
   - Reason: Spawns fzf process with custom options, waits for user input
   - Marked as: `#[ignore = "This test requires fzf and runs interactively"]`

4. **test_select_with_fzf_multiple_items** - Line 304
   - Reason: Spawns fzf process with multiple items, requires user selection
   - Marked as: `#[ignore = "This test requires fzf and runs interactively"]`

### worktree/select.rs
1. **test_select_worktree_with_fzf_empty** - Line 491
   - Reason: Calls select_worktree_with_fzf which spawns interactive fzf
   - Marked as: `#[ignore = "This test requires fzf and runs interactively"]`

2. **test_select_worktree_with_custom_options** - Line 513
   - Reason: Calls select_worktree_with_fzf_and_options which spawns interactive fzf
   - Marked as: `#[ignore = "This test requires fzf and runs interactively"]`

### process/tmux.rs
All tmux tests pass without issues as they handle the case where tmux is not installed.

### process/prompt.rs
All prompt tests pass as they only test the logic without actual stdin interaction.

## Running Ignored Tests

To run these ignored tests manually (when you want to test with actual fzf interaction):
```bash
cargo test -- --ignored
```

## Coverage Impact

These ignored tests primarily cover the interactive paths of the fzf integration. The core logic (error handling, options parsing, etc.) is still tested through other non-interactive tests.

## Future Improvements

Consider:
1. Mocking the Command execution to test the full flow without spawning actual processes
2. Using a test double for fzf that immediately returns predetermined output
3. Setting up integration tests that can handle interactive processes