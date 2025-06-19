# Phantom Rust Migration TODO

Active tasks for the Phantom Rust implementation.

- Completed tasks: [ARCHIVE.md](./ARCHIVE.md)
- Test strategy details: [TEST_STRATEGY.md](./TEST_STRATEGY.md)

## 📚 Migration Documentation Guide

### When to Reference Each Document

#### Starting Migration Work
1. **Read FIRST**: [MOCK_TESTING_PLAN.md](./MOCK_TESTING_PLAN.md) - Understand the overall strategy
2. **Then**: [MOCK_TESTING_MIGRATION.md](./rust/MOCK_TESTING_MIGRATION.md) - See implementation details and examples
3. **For context**: [MOCK_TESTING_SUMMARY.md](./rust/MOCK_TESTING_SUMMARY.md) - Learn from past mistakes

#### During Active Development
- **For git operations**: [GIT_OPERATIONS_MIGRATION_GUIDE.md](./rust/GIT_OPERATIONS_MIGRATION_GUIDE.md) - ✅ COMPLETE
- **For process operations**: [PROCESS_OPERATIONS_MIGRATION.md](./rust/PROCESS_OPERATIONS_MIGRATION.md) - Step-by-step guide
- **For examples**: Check the `examples/mock_testing_example.rs` file
- **For progress**: Check the "Migration Progress" section below in this file

#### When Writing Tests
- **Test patterns**: [TEST_STRATEGY.md](./TEST_STRATEGY.md) - Overall testing approach
- **Mock examples**: [MOCK_TESTING_MIGRATION.md](./rust/MOCK_TESTING_MIGRATION.md) - Mock usage patterns
- **Why mocks**: [TEST_RATIONALE.md](./TEST_RATIONALE.md) - Understand the reasoning

#### For Review/Cleanup
- **Recent changes**: [CLEANUP_SUMMARY.md](./rust/CLEANUP_SUMMARY.md) - What was cleaned up and why
- **Completed work**: [ARCHIVE.md](./ARCHIVE.md) - What's been done

#### Quick Reference Hierarchy
```
MOCK_TESTING_PLAN.md (Strategy)
└── MOCK_TESTING_MIGRATION.md (Implementation)
    └── GIT_OPERATIONS_MIGRATION_GUIDE.md (Step-by-step)
        └── examples/mock_testing_example.rs (Code examples)
```

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
- ✅ Implemented CommandExecutor trait with RealCommandExecutor and MockCommandExecutor
- ✅ Created working example demonstrating mock usage patterns
- ✅ Added MOCK_TESTING_MIGRATION.md with comprehensive guide
- ✅ Integrated CommandExecutor into handlers and created GitExecutor adapter
- ✅ Written mock tests for handlers - revealed incomplete migration blocks testing
- 📝 **LEARNING**: Partial migration doesn't work - all git operations must use CommandExecutor
- 📝 Created MOCK_TESTING_SUMMARY.md documenting lessons learned
- 📝 Created GIT_OPERATIONS_MIGRATION_GUIDE.md with complete migration checklist
- ✅ **NEW**: Migrated 9 critical git operations to CommandExecutor (45% complete)
- ✅ **NEW**: List and attach handlers now fully testable with mocks
- 📝 **NEW**: Discovered filesystem operation limitations for complete mock testing

## 📋 Next Steps

### 🚨 Priority 1: Complete Git Operations Migration

**Critical Insight**: Mock tests cannot work until ALL git operations use CommandExecutor.

- [x] Create CommandExecutor trait and implementations ✅
- [x] Update handlers to accept HandlerContext ✅
- [x] Create GitExecutor adapter ✅
- [x] Document migration pattern in GIT_OPERATIONS_MIGRATION_GUIDE.md ✅
- [x] **COMPLETED**: Migrate all git operations ✅ (see [GIT_OPERATIONS_MIGRATION_GUIDE.md](./rust/GIT_OPERATIONS_MIGRATION_GUIDE.md))
  - [x] get_git_root ✅ (template example)
  - [x] add_worktree ✅ (template example)
  - [x] list_worktrees ✅ (list handler now testable)
  - [x] get_worktree_branch, get_worktree_status ✅ (list handler fully testable)
  - [x] attach_worktree ✅ (attach handler now testable)
  - [x] delete_worktree ✅ (delete handler partially testable - filesystem ops limit)
  - [x] branch_exists, get_current_branch, get_current_worktree ✅
  - [x] create_branch ✅ (create handler now partially testable)
  - [x] is_inside_work_tree ✅
  - [x] current_commit ✅
  - [x] list_branches ✅
  - [x] remove_worktree ✅
- [x] Write mock tests for list and attach handlers ✅
- [x] Write mock tests for create handler ✅ (partial - filesystem ops limit)
- [ ] Write mock tests for remaining handlers after migration

Progress: Infrastructure complete, all 13 git operations migrated (100%), 3 handlers with mock tests. Process operations remain.

### Priority 2: Continue Handler Testing

- [x] List handler - 5 comprehensive mock tests ✅
- [x] Attach handler - 5 comprehensive mock tests ✅
- [x] Create handler - 5 mock tests ✅ (partial - filesystem ops limit)
- [ ] Delete handler - tests written but limited by filesystem operations
- [ ] Other handlers - blocked by remaining migrations

### Priority 3: Address Testing Limitations

- [ ] Abstract filesystem operations (fs::metadata, etc.) for complete testability
- [ ] Consider creating FileSystem trait similar to CommandExecutor
- [ ] Update validate_worktree_exists to use abstractions

### Priority 4: Complete Process Operations Migration

**See [PROCESS_OPERATIONS_MIGRATION.md](./rust/PROCESS_OPERATIONS_MIGRATION.md) for detailed migration guide**

- [ ] Migrate tmux operations to use CommandExecutor (high priority)
- [ ] Migrate fzf operations to use CommandExecutor (high priority)
- [ ] Migrate kitty operations to use CommandExecutor (medium priority)
- [ ] Migrate shell operations to use CommandExecutor (medium priority)

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

- **Priority**: Complete git operations migration before adding features
- **Timeline**: Complete git migration in 1-2 weeks, process operations in 1 week
- **Principle**: No more git command tests without proper abstractions
- **Learning**: Filesystem operations also need abstraction for complete testability
