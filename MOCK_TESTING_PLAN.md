# Mock-Based Testing Strategy for Phantom

## Problem Statement
Tests are frequently failing in CI due to:
- Different git versions/configurations across environments
- Missing external commands (tmux, fzf, kitty)
- Race conditions in parallel test execution
- Tests modifying global state (working directory)
- Environment-specific behavior differences

## Solution: Command Execution Abstraction Layer

### Phase 1: Create Command Execution Trait (1 week)

#### 1.1 Define Core Traits
```rust
// src/core/command_executor.rs
#[async_trait]
pub trait CommandExecutor: Send + Sync {
    async fn execute(&self, config: CommandConfig) -> Result<CommandOutput>;
    async fn spawn(&self, config: SpawnConfig) -> Result<SpawnOutput>;
}

pub struct CommandConfig {
    pub program: String,
    pub args: Vec<String>,
    pub cwd: Option<PathBuf>,
    pub env: Option<HashMap<String, String>>,
    pub timeout: Option<Duration>,
}

pub struct CommandOutput {
    pub stdout: String,
    pub stderr: String,
    pub exit_code: i32,
}
```

#### 1.2 Implement Real Executor
```rust
// src/core/executors/real_executor.rs
pub struct RealCommandExecutor;

#[async_trait]
impl CommandExecutor for RealCommandExecutor {
    async fn execute(&self, config: CommandConfig) -> Result<CommandOutput> {
        // Current implementation from spawn.rs
    }
}
```

#### 1.3 Implement Mock Executor
```rust
// src/core/executors/mock_executor.rs
pub struct MockCommandExecutor {
    expectations: Arc<Mutex<Vec<CommandExpectation>>>,
    calls: Arc<Mutex<Vec<CommandCall>>>,
}

impl MockCommandExecutor {
    pub fn expect_command(&mut self, program: &str) -> CommandExpectationBuilder {
        // Builder pattern for setting up expectations
    }
    
    pub fn verify(&self) {
        // Verify all expectations were met
    }
}
```

### Phase 2: Refactor External Command Usage (2 weeks)

#### 2.1 Git Operations
- Replace `GitExecutor` with `GitBackend` trait that uses `CommandExecutor`
- Create `MockGitBackend` that records git operations without execution
- Test example:
```rust
#[test]
async fn test_add_worktree() {
    let mut mock = MockCommandExecutor::new();
    mock.expect_command("git")
        .with_args(&["worktree", "add", "-b", "feature", "/path/to/worktree"])
        .returns_output("", "", 0);
    
    let git = GitBackend::new(Box::new(mock));
    let result = git.add_worktree("feature", "/path/to/worktree").await;
    
    assert!(result.is_ok());
    mock.verify();
}
```

#### 2.2 Process Operations
- Refactor `spawn_process` and `execute_command` to use `CommandExecutor`
- Update all process operations (tmux, kitty, fzf, shell) to use the trait

#### 2.3 Handler Refactoring
- Add `executor: Arc<dyn CommandExecutor>` to handler context
- Pass executor through all handler functions
- Default to `RealCommandExecutor` in production

### Phase 3: Test Migration (1 week)

#### 3.1 Unit Tests
- Replace all command execution with mocks
- Test only the command construction and arguments
- Example patterns:
```rust
// Before
#[tokio::test]
async fn test_create_worktree() {
    let repo = TestRepo::new().await.unwrap();
    // ... actual git operations
}

// After
#[tokio::test]
async fn test_create_worktree() {
    let mut mock = MockCommandExecutor::new();
    mock.expect_command("git")
        .with_args(&["worktree", "add", "-b", "feature", "phantoms/feature"])
        .times(1)
        .returns_success();
    
    let handler = CreateHandler::new(Box::new(mock));
    let result = handler.create("feature", CreateOptions::default()).await;
    
    assert!(result.is_ok());
    mock.verify();
}
```

#### 3.2 Integration Tests
- Keep minimal E2E tests with real commands
- Run in isolated environment with `SafeGitCommand`
- Mark as `#[ignore]` by default, run with `--ignored` flag

#### 3.3 Test Categories
1. **Unit Tests** (default): Use mocks, test logic and command construction
2. **Integration Tests** (`--ignored`): Use real commands in isolated environment
3. **E2E Tests** (`--ignored`): Full workflow with real git repos

### Phase 4: CI Configuration (1 day)

#### 4.1 Test Jobs
```yaml
# .github/workflows/rust-ci.yml
test-unit:
  name: Unit Tests (Mocked)
  run: cargo test --lib --bins

test-integration:
  name: Integration Tests (Real Commands)
  run: cargo test --ignored -- --test-threads=1
  if: github.event_name == 'push' && github.ref == 'refs/heads/main'
```

#### 4.2 Fast Feedback Loop
- PR checks run only unit tests (fast, reliable)
- Main branch runs full test suite
- Manual trigger for integration tests

### Implementation Order

1. **Week 1**: Create command executor trait and implementations
2. **Week 2**: Refactor GitExecutor to use trait
3. **Week 3**: Refactor process operations (tmux, kitty, fzf, shell)
4. **Week 4**: Migrate tests to use mocks
5. **Week 5**: Update CI configuration and documentation

### Benefits

1. **Reliability**: Tests pass consistently across all environments
2. **Speed**: No external process spawning, tests run in milliseconds
3. **Clarity**: Tests explicitly show expected command interactions
4. **Debugging**: Easy to see what commands would be executed
5. **Coverage**: Can test error scenarios without complex setup

### Example Mock Test

```rust
#[tokio::test]
async fn test_delete_worktree_with_uncommitted_changes() {
    let mut mock = MockCommandExecutor::new();
    
    // First, check for uncommitted changes
    mock.expect_command("git")
        .with_args(&["status", "--porcelain"])
        .in_dir("/path/to/worktree")
        .returns_output("M  file.txt\n", "", 0);
    
    // Force delete should proceed
    mock.expect_command("git")
        .with_args(&["worktree", "remove", "--force", "/path/to/worktree"])
        .returns_success();
    
    let git = GitBackend::new(Box::new(mock));
    let result = delete_worktree(&git, "feature", true).await;
    
    assert!(result.is_ok());
    mock.verify();
}
```

### Migration Checklist

- [ ] Create CommandExecutor trait
- [ ] Implement RealCommandExecutor
- [ ] Implement MockCommandExecutor with builder pattern
- [ ] Create GitBackend trait using CommandExecutor
- [ ] Refactor GitExecutor to implement GitBackend
- [ ] Create MockGitBackend for testing
- [ ] Update all handlers to accept CommandExecutor
- [ ] Migrate git operation tests to use mocks
- [ ] Migrate process operation tests to use mocks
- [ ] Create integration test suite with real commands
- [ ] Update CI to run appropriate test suites
- [ ] Document testing strategy in CONTRIBUTING.md

## Conclusion

This approach will make tests deterministic, fast, and reliable while maintaining the ability to verify real command execution in controlled environments. The investment in this infrastructure will pay dividends in development velocity and confidence.