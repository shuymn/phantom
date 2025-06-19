# Phantom Test Strategy

## The Fundamental Problem

**Tests that execute git commands will ALWAYS behave differently between local and CI environments.**

This is not a problem we can "fix" with workarounds. It's a fundamental architectural issue:

1. **Git is environment-dependent by design**
   - User configuration (name, email, signing keys)
   - System configuration (core.editor, diff.tool)
   - Global gitignore and attributes
   - SSH keys and credentials
   - Git version differences

2. **Current tests are integration tests disguised as unit tests**
   - They test git CLI behavior, not our code's logic
   - They're testing git itself, not phantom
   - Success depends on external factors we don't control

3. **No amount of cleanup will fix this**
   - We can reduce flakiness but not eliminate it
   - We're fighting against git's design
   - Each "fix" is just another workaround

## The Only Real Solution

**Stop executing git commands in tests.** Use proper abstractions:

1. **Unit tests**: Test business logic with mocked GitBackend
2. **Integration tests**: Use in-process git library (libgit2)
3. **E2E tests**: Accept environment dependencies, run sparingly

## Safe Testing Practices

### ⚠️ SAFETY WARNING
Direct git configuration manipulation can damage your local git settings!

**Safe Testing Principles**:
1. **NEVER modify global git config** in tests
2. **NEVER use `git config --global` or `--system`** in test code
3. **ALWAYS use isolated environments** for git operations
4. **ALWAYS use environment variables** instead of config files

### Rust Limitations

Rust doesn't have Go's `t.Setenv` for automatic cleanup. Use these approaches instead:

```rust
// RECOMMENDED: Pass environment explicitly to commands
Command::new("git")
    .env("GIT_CONFIG_GLOBAL", "/dev/null")
    .env("GIT_CONFIG_SYSTEM", "/dev/null")
    .env("HOME", temp_dir.path())
    .env("GIT_AUTHOR_NAME", "Test Suite")
    .env("GIT_AUTHOR_EMAIL", "test@example.com")
    .args(&["init"])
    .output()?;

// Create a safe wrapper
struct SafeGitCommand {
    temp_home: TempDir,
}

impl SafeGitCommand {
    fn new() -> Self {
        let temp_home = TempDir::new().unwrap();
        Self { temp_home }
    }
    
    fn command(&self, args: &[&str]) -> Command {
        let mut cmd = Command::new("git");
        cmd.env("HOME", self.temp_home.path())
           .env("GIT_CONFIG_GLOBAL", "/dev/null")
           .env("GIT_CONFIG_SYSTEM", "/dev/null")
           .args(args);
        cmd
    }
}
```

## Three-Layer Test Architecture

1. **Layer 1: Pure Unit Tests (Target: 80%)**
   - Test business logic without any I/O operations
   - Use `MockGitBackend` implementing the `GitBackend` trait
   - Fast, reliable, environment-independent
   - Example: worktree validation, path calculation, config parsing

2. **Layer 2: Integration Tests (Target: 15%)**
   - Test git operations with `TestGitBackend` (in-memory or isolated)
   - Controlled environment, no external dependencies
   - Verify correct git command construction and parsing
   - Example: worktree creation logic, branch operations

3. **Layer 3: E2E Tests (Target: 5%)**
   - Test complete workflows with real git
   - Accept environment dependencies as necessary
   - Run in CI with proper environment setup
   - Example: full command execution, shell integration

## Implementation Example

```rust
// Before: Untestable handler
pub async fn handle(args: CreateArgs) -> Result<()> {
    let worktree = create_worktree(&args.name, &args.base).await?;
    // Direct git operations...
}

// After: Testable handler with DI
pub async fn handle(args: CreateArgs, backend: Arc<dyn GitBackend>) -> Result<()> {
    let worktree = create_worktree(&args.name, &args.base, backend).await?;
    // Operations through backend trait...
}

// In tests:
#[test]
async fn test_create_handler() {
    let mut mock = MockGitBackend::new();
    mock.expect_add_worktree()
        .returning(|_, _, _| Ok(()));
    
    let result = handle(args, Arc::new(mock)).await;
    assert!(result.is_ok());
}
```

## Key Insight

The current test problems are symptoms of missing abstraction. By implementing proper dependency injection and the `GitBackend` trait throughout the codebase, we not only fix tests but also improve the overall architecture.