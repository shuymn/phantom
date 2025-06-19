# Phantom Rust Migration TODO

Active tasks for the Phantom Rust implementation.

- Completed tasks: [ARCHIVE.md](./ARCHIVE.md)
- Test strategy details: [TEST_STRATEGY.md](./TEST_STRATEGY.md)

## 🚨 Critical: Mock-Based Testing Strategy Required

**Problem**: Tests are frequently failing in CI due to environment differences,
missing commands, and race conditions. **Solution**: Replace all external command
calls with mocks that verify expected arguments instead of executing commands.

See [MOCK_TESTING_PLAN.md](./MOCK_TESTING_PLAN.md) for detailed implementation plan.

### Progress Update (2025-06-19)
- ✅ Removed 9 redundant CLI test files (1,291 lines)
- ✅ Created SafeGitCommand wrapper for test isolation
- ✅ Fixed CI: added tmux to coverage job, fixed cross-compilation
- ✅ Added clippy lint to prevent std::env::set_var usage
- 📝 Created TEST_RATIONALE.md and TEST_STRATEGY.md documentation

## 📋 Next Steps

### 🚨 Priority 1: Implement Mock Testing Infrastructure (1-2 weeks)

- [ ] Create CommandExecutor trait and implementations
- [ ] Implement MockCommandExecutor with expectation builder
- [ ] Refactor GitExecutor to use CommandExecutor trait
- [ ] Update process operations to use CommandExecutor
- [ ] Migrate tests to use mocks instead of real commands

This is now the highest priority to resolve CI reliability issues.

### Priority 2: Complete Test Migration

- [ ] Move validation logic from integration tests to unit tests
- [ ] Convert all command-executing tests to use mocks
- [ ] Create separate integration test suite with `#[ignore]`
- [ ] Update CI to run appropriate test suites

### Priority 3: Architecture Refactoring (After Mock Implementation)

The dependency injection work naturally aligns with the mock testing infrastructure.

## 🔧 Architecture Refactoring (Depends on Mock Infrastructure)

### Phase 1: Enable Dependency Injection

- [ ] Add `executor: Arc<dyn CommandExecutor>` to all handlers
- [ ] Create `HandlerContext` struct for dependency passing
- [ ] Update CLI main to inject `RealCommandExecutor` by default
- [ ] Ensure backward compatibility

### Phase 2: Implement Test Infrastructure

- [ ] Create `MockGitBackend` with expectation builder
- [ ] Create test examples showing proper usage
- [ ] Document testing patterns in CONTRIBUTING.md
- [ ] Add test helpers for common scenarios

### Phase 3: Migrate Tests

- [ ] Convert handler tests to use mocks
- [ ] Extract business logic to testable functions
- [ ] Reduce E2E tests to essential scenarios only
- [ ] Set up separate CI jobs for different test types

## 📅 Future Work (Low Priority)

### Testing Improvements (Deferred)
- [ ] Remove serial test execution from get_git_root tests
  - Currently using `#[serial]` as a workaround for tests that change working directory
  - Should refactor to avoid changing global state or use a different approach
  - Tests should be able to run in parallel for better performance
- [ ] Implement proper tmux testing approach
  - Extract command building logic from execution
  - Test command construction without actual execution
  - Use dependency injection for tmux operations
  - Mock tmux process execution in tests

### Release & Communication

- [ ] Announce Rust version availability
- [ ] Gather user feedback
- [ ] Plan TypeScript deprecation

### Enhancements

- [ ] Native git support via libgit2
- [ ] Parallel worktree operations
- [ ] Plugin system
- [ ] Configuration profiles

## ✅ Success Criteria

- [x] Feature parity with TypeScript version
- [x] Binary size < 5MB (4.5MB achieved)
- [x] Zero runtime dependencies
- [x] Single binary distribution
- [ ] **Clean test architecture** (in progress)
- [ ] User acceptance testing

### Test Coverage Note

Coverage targets suspended until proper test infrastructure is in place. Current
60% coverage is misleading due to environment-dependent tests.

## 📝 Notes

- **Priority**: Fix architecture before adding features
- **Timeline**: Complete refactoring in 3-4 weeks
- **Principle**: No more git command tests without proper abstractions
