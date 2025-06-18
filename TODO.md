# Phantom Rust Migration TODO

Active tasks for the Phantom Rust implementation.
- Completed tasks: [ARCHIVE.md](./ARCHIVE.md)
- Test strategy details: [TEST_STRATEGY.md](./TEST_STRATEGY.md)

## 🚨 Critical: Architecture Fix Required

**Problem**: Tests that execute git commands are inherently unreliable across environments.
**Solution**: Implement proper abstractions (GitBackend trait) throughout all handlers.

## 📋 Immediate Tasks (This Week)

### 1. Test Cleanup
- [ ] Remove `cli_exec.rs`, `cli_shell.rs`, `cli_version.rs` - no actual testing
- [ ] Remove duplicate tests covered by E2E (most of `cli_*.rs`)
- [ ] Move validation logic from integration tests to unit tests
- [ ] Document which tests we're keeping and why

### 2. Safety Implementation
- [ ] Create `SafeGitCommand` wrapper to prevent global state changes
- [ ] Replace all `std::env::set_var()` with `Command::env()`
- [ ] Add clippy lint to ban `std::env::set_var` in tests
- [ ] Verify no test modifies global git config

## 🔧 Architecture Refactoring (Next 2-3 Weeks)

### Phase 1: Enable Dependency Injection
- [ ] Add `backend: Arc<dyn GitBackend>` parameter to all handlers
- [ ] Create `HandlerContext` struct for dependency passing
- [ ] Update CLI main to inject `CommandBackend` by default
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
Coverage targets suspended until proper test infrastructure is in place. Current 60% coverage is misleading due to environment-dependent tests.

## 📝 Notes

- **Priority**: Fix architecture before adding features
- **Timeline**: Complete refactoring in 3-4 weeks
- **Principle**: No more git command tests without proper abstractions