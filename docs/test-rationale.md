# Test File Rationale

This document explains which test files we keep and why, as part of the test cleanup effort.

## Tests Removed

### CLI Handler Tests (Redundant with E2E)
- **cli_attach.rs** - Covered by e2e_scenarios.rs lines 139-176
- **cli_create.rs** - Basic functionality covered by E2E, post-create commands are environment-dependent
- **cli_delete.rs** - Covered by e2e_scenarios.rs lines 116-122, 251-259
- **cli_exec.rs** - No actual testing, just compilation check
- **cli_list.rs** - Covered by e2e_scenarios.rs lines 59-66, 76-83, 102-115
- **cli_shell.rs** - No actual testing, interactive command
- **cli_version.rs** - No actual testing, just ensures no panic
- **cli_where.rs** - Covered by e2e_scenarios.rs lines 85-92, 219-234
- **cli_completion.rs** - Redundant with cli_snapshots.rs which has better tests

## Tests Kept

### E2E Tests
- **e2e_scenarios.rs** - Comprehensive end-to-end workflow testing
  - Tests complete user workflows
  - Verifies integration between components
  - Essential for ensuring real-world functionality

### Snapshot Tests
- **cli_snapshots.rs** - CLI output consistency
  - Tests help text formatting
  - Tests error message formatting
  - Tests completion script generation
  - Tests version output format
  
- **cli_output_snapshots.rs** - Command output formatting
  - Ensures consistent JSON output
  - Verifies list output formatting
  - Tests where command output

### Regression Tests
- **typescript_regression.rs** - TypeScript compatibility
  - Ensures behavior matches TypeScript version
  - Critical for migration success

### Common Module
- **common/mod.rs** - Test utilities
  - Provides safe git repository initialization
  - Used by other test files

## Test Strategy Going Forward

1. **Unit Tests** (80% target)
   - Test business logic with mocked GitBackend
   - No external command execution
   - Fast and reliable

2. **Integration Tests** (15% target)
   - Test with in-process git operations
   - Use TestGitBackend implementation
   - Controlled environment

3. **E2E Tests** (5% target)
   - Keep current e2e_scenarios.rs
   - Accept environment dependencies
   - Run in CI with proper setup

## Why This Approach?

The removed tests were:
1. Testing git CLI behavior, not phantom logic
2. Duplicating E2E test coverage
3. Environment-dependent without proper isolation
4. Providing minimal value (just checking for panics)

By focusing on proper abstractions and dependency injection, we can write better tests that:
- Are deterministic and reliable
- Test our code, not external tools
- Run quickly and consistently
- Provide meaningful coverage