# 🤝 Contributing to Phantom

Thank you for your interest in contributing to Phantom! This guide will help you get started with development.

## 📋 Table of Contents

- [Development Setup](#development-setup)
- [Development Guidelines](#development-guidelines)
- [Testing](#testing)
- [Code Quality](#code-quality)
- [Performance Guidelines (Rust)](#-performance-guidelines-rust)
- [Pull Request Process](#pull-request-process)
- [Documentation](#documentation)
- [Release Process](#release-process)
- [Additional Resources](#additional-resources)

## 🛠️ Development Setup

### Prerequisites

- Node.js 22+ and pnpm 10+

### Getting Started

```bash
# Clone and setup
git clone https://github.com/aku11i/phantom.git
cd phantom
pnpm install

# run phantom in development mode
pnpm phantom
```

### Development Workflow

```bash
# Run tests
pnpm test

# Type checking
pnpm typecheck

# Linting
pnpm lint
# or
pnpm fix

# Run all checks before committing
pnpm ready
```

## 📝 Development Guidelines

### Language Requirements

- **All files, issues, and pull requests must be written in English**
- This ensures the project is accessible to the global community

### Code Style

- Follow existing code conventions and patterns
- Use TypeScript for all new code
- Follow the Single Responsibility Principle
- Keep modules focused and testable

### Architecture Principles

- **Single Responsibility Principle**: Each module has one clear responsibility
- **Separation of Concerns**: CLI, business logic, and git operations are separated
- **Testability**: Core modules are framework-agnostic and easily testable
- **No Code Duplication**: Common operations are centralized
- **Clear Dependencies**: Dependencies flow from CLI → Core (including Git operations)

## 🧪 Testing

### Running Tests

```bash
# Run all tests
pnpm test

# Run specific test file
pnpm test:file src/core/worktree/create.test.js
```

### Writing Tests

- Add tests for all new features
- Follow existing test patterns
- Use descriptive test names
- Test both success and error cases

## ✨ Code Quality

### Before Committing

Always run the following command before committing:

```bash
pnpm ready
```

This command runs:
- Linting (`pnpm lint`)
- Type checking (`pnpm typecheck`)
- All tests (`pnpm test`)

### Security Best Practices

- Never introduce code that exposes or logs secrets and keys
- Never commit secrets or keys to the repository
- Be careful with user input validation

## 🚀 Performance Guidelines (Rust)

### Performance Standards

- **Startup time**: < 50ms for all commands
- **Memory usage**: Predictable and bounded
- **Zero-cost abstractions**: Prefer compile-time polymorphism over runtime dispatch

### Best Practices

#### 1. Smart Pointer Usage

**Use `Arc<T>` when:**
- Sharing immutable data across threads
- Multiple owners need the same large data structure
- Data lifetime is complex or spans multiple async operations

**Use `Rc<T>` when:**
- Single-threaded shared ownership is needed
- Building graphs or tree structures

**Prefer cloning when:**
- Data is small (`Copy` types preferred)
- Each owner needs independent data
- Cloning is infrequent

**Example:**
```rust
// Good: Generic function, no Arc required
async fn process<E: CommandExecutor>(executor: &E) { ... }

// Avoid: Forces Arc even when not needed
async fn process(executor: Arc<dyn CommandExecutor>) { ... }
```

#### 2. Iterator Optimization

- Use `into_iter()` when consuming collections to avoid cloning
- Chain iterator methods instead of collecting intermediate results
- Use `collect()` with type hints for better performance

```rust
// Good: Consumes vector, no cloning
let results: Vec<_> = items
    .into_iter()
    .filter(|x| x.is_valid())
    .map(|x| process(x))
    .collect();

// Avoid: Unnecessary cloning
let results: Vec<_> = items
    .iter()
    .filter(|x| x.is_valid())
    .cloned()
    .map(|x| process(x))
    .collect();
```

#### 3. String Handling

- Use `Cow<str>` for strings that are sometimes owned, sometimes borrowed
- Use `&str` parameters when possible, `String` only when ownership is needed
- Consider `SmallVec` for collections that are usually small

```rust
// Good: Flexible string handling
use std::borrow::Cow;
fn process(text: Cow<str>) -> Cow<str> { ... }

// Good: Efficient for small collections
use smallvec::SmallVec;
type Args = SmallVec<[String; 4]>;
```

#### 4. Async Best Practices

- Use `FuturesUnordered` for concurrent operations
- Avoid unnecessary `Arc` wrapping in async contexts
- Batch operations when possible

```rust
// Good: Concurrent execution
use futures::stream::FuturesUnordered;
let results: Vec<_> = tasks
    .into_iter()
    .map(|task| async move { process(task).await })
    .collect::<FuturesUnordered<_>>()
    .collect()
    .await;
```

### Benchmarking

Run benchmarks to verify performance improvements:

```bash
# Run all benchmarks
cargo bench

# Run specific benchmark
cargo bench phantom_benchmarks

# Compare with baseline
cargo bench -- --save-baseline before
# make changes...
cargo bench -- --baseline before
```

### Advanced Rust Patterns

#### 1. Type-State Pattern

Use types to enforce state transitions at compile time:

```rust
pub struct WorktreeBuilder<S> {
    name: String,
    _state: PhantomData<S>,
}

pub struct Unnamed;
pub struct Named;

impl WorktreeBuilder<Unnamed> {
    pub fn name(self, name: String) -> WorktreeBuilder<Named> { ... }
}

impl WorktreeBuilder<Named> {
    pub fn build(self) -> Result<Worktree> { ... }
}
```

#### 2. Extension Traits

Add methods to external types safely:

```rust
pub trait PathExt {
    fn to_string_lossy_owned(&self) -> String;
}

impl PathExt for Path {
    fn to_string_lossy_owned(&self) -> String {
        self.to_string_lossy().into_owned()
    }
}
```

#### 3. Sealed Traits

Prevent external implementation of internal traits:

```rust
mod private {
    pub trait Sealed {}
}

pub trait MyTrait: private::Sealed {
    fn method(&self);
}

// Only types in this crate can implement Sealed
impl private::Sealed for MyType {}
impl MyTrait for MyType { ... }
```

#### 4. Zero-Cost Abstractions

Prefer generics over trait objects when possible:

```rust
// Good: Zero-cost abstraction
pub async fn execute<E: CommandExecutor>(executor: E) { ... }

// Avoid when possible: Runtime dispatch
pub async fn execute(executor: Box<dyn CommandExecutor>) { ... }
```

## 🚀 Pull Request Requirements

- Clear description of changes
- Tests for new functionality
- Documentation updates if applicable
- All checks passing (`pnpm ready`)
- Follow existing code style

## 📚 Documentation

When contributing documentation:

- Keep language clear and concise
- Update the table of contents if adding sections
- Check for broken links

## 🚀 Release Process

To release a new version of Phantom:

1. **Ensure you're on main branch and up to date**
   ```bash
   git checkout main
   git pull
   ```

2. **Run all checks**
   ```bash
   pnpm ready
   ```

3. **Bump version**
   ```bash
   # For patch releases (bug fixes)
   npm version patch

   # For minor releases (new features)
   npm version minor

   # For major releases (breaking changes)
   npm version major
   ```

4. **Push the version commit and tag**
   ```bash
   git push && git push --tags
   ```

5. **Publish to npm**
   ```bash
   pnpm publish
   ```

6. **Create GitHub release**
   ```bash
   # Create a release with automatically generated notes
   gh release create v<version> \
     --title "Phantom v<version>" \
     --generate-notes \
     --target main

   # Example for v0.1.3:
   gh release create v0.1.3 \
     --title "Phantom v0.1.3" \
     --generate-notes \
     --target main
   ```

7. **Update release notes for clarity**
   - Review the auto-generated release notes using `gh release view v<version>`
   - Check PR descriptions for important details using `gh pr view <number>`
   - Update the release notes to be more user-friendly:
     - Group changes by category (Features, Bug Fixes, Improvements)
     - Add usage examples for new features
     - Explain the impact of changes in plain language
     - Highlight security fixes and breaking changes
   
   ```bash
   # Edit the release notes
   gh release edit v<version> --notes "$(cat <<'EOF'
   ## 🚀 What's New in v<version>
   
   ### ✨ New Features
   - Feature description with usage example
   
   ### 🐛 Bug Fixes
   - Clear description of what was fixed
   
   ### 🛠️ Improvements
   - Performance, security, or other improvements
   EOF
   )"
   ```

## 🙏 Thank You!

Your contributions make Phantom better for everyone. If you have questions, feel free to:
- Open an issue for bugs or feature requests
- Start a discussion for general questions
- Ask in pull request comments

