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
- ✅ **NEW**: Implemented CommandExecutor trait with RealCommandExecutor and MockCommandExecutor
- ✅ **NEW**: Created working example demonstrating mock usage patterns
- ✅ **NEW**: Added MOCK_TESTING_MIGRATION.md with comprehensive guide
- ✅ **NEW**: Integrated CommandExecutor into handlers and created GitExecutor adapter
- ✅ **NEW**: Written mock tests for handlers - revealed incomplete migration blocks testing
- 📝 **LEARNING**: Partial migration doesn't work - all git operations must use CommandExecutor
- 📝 **NEW**: Created MOCK_TESTING_SUMMARY.md documenting lessons learned
- 📝 **NEW**: Created GIT_OPERATIONS_MIGRATION_GUIDE.md with complete migration checklist

## 📋 Next Steps

### 🚨 Priority 1: Complete Git Operations Migration

**Critical Insight**: Mock tests cannot work until ALL git operations use CommandExecutor.

- [x] Create CommandExecutor trait and implementations ✅
- [x] Update handlers to accept HandlerContext ✅
- [x] Create GitExecutor adapter ✅
- [x] Document migration pattern in GIT_OPERATIONS_MIGRATION_GUIDE.md ✅
- [ ] **IN PROGRESS**: Migrate all git operations (see [GIT_OPERATIONS_MIGRATION_GUIDE.md](./rust/GIT_OPERATIONS_MIGRATION_GUIDE.md))
  - [x] get_git_root ✅ (template example)
  - [x] add_worktree ✅ (template example)
  - [ ] list_worktrees (blocks list handler tests)
  - [ ] get_worktree_branch, get_worktree_status (blocks list handler)
  - [ ] attach_worktree (blocks attach handler)
  - [ ] delete_worktree (blocks delete handler)
  - [ ] branch_exists, get_current_branch (blocks create handler)
  - [ ] And 15+ more operations...
- [ ] Only then: Write effective mock tests for handlers

Progress: Infrastructure complete, migration pattern documented, 2/20+ operations migrated.

### Priority 2: Complete Test Migration

- [ ] Move validation logic from integration tests to unit tests
- [ ] Convert all command-executing tests to use mocks
- [ ] Create separate integration test suite with `#[ignore]`
- [ ] Update CI to run appropriate test suites

### Priority 3: Architecture Refactoring (After Mock Implementation)

The dependency injection work naturally aligns with the mock testing infrastructure.

## 🔧 Architecture Refactoring (Complete)

The mock infrastructure has been successfully implemented:

- [x] Created CommandExecutor trait with Real and Mock implementations ✅
- [x] Created HandlerContext for dependency injection ✅
- [x] Updated all handlers to accept HandlerContext ✅
- [x] Updated CLI main to inject RealCommandExecutor ✅
- [x] Created working examples showing proper usage ✅
- [x] Documented patterns in multiple guides ✅

Remaining work is completing the migration of existing code to use this infrastructure.

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
