# Rust Migration Plan for Phantom

## Executive Summary

This document outlines a comprehensive plan for migrating the Phantom CLI tool from TypeScript/JavaScript to Rust. The migration targets Unix-based systems (Linux and macOS) only, excluding Windows support to reduce complexity and accelerate development. The migration will be executed in phases to ensure continuous functionality while gradually replacing components.

## Migration Benefits

1. **Performance**: 50%+ performance improvements, especially for file operations and process spawning
2. **Single Binary**: Distribute as a single executable without Node.js dependency
3. **Memory Safety**: Rust's ownership system prevents memory-related bugs
4. **Better Error Handling**: Native Result/Option types align with current architecture
5. **Reduced Memory Usage**: < 50% of TypeScript version's memory footprint
6. **Simplified Maintenance**: Unix-only focus eliminates platform-specific complexity

## Phase 1: Foundation & Core Types (Weeks 1-2)

### Goals
- Set up Rust project structure with async runtime
- Implement comprehensive error handling
- Create core types and abstractions

### Tasks
1. **Project Setup**
   ```toml
   # Cargo.toml
   [package]
   name = "phantom"
   version = "0.1.0"
   edition = "2021"

   [dependencies]
   # Core
   clap = { version = "4", features = ["derive", "cargo"] }
   serde = { version = "1", features = ["derive"] }
   serde_json = "1"
   tokio = { version = "1", features = ["full"] }
   anyhow = "1"
   thiserror = "1"
   
   # Unix-specific
   nix = "0.27"          # Unix system calls
   termion = "2"         # Terminal control
   
   # Utilities
   directories = "5"
   which = "6"
   tracing = "0.1"
   tracing-subscriber = { version = "0.3", features = ["env-filter"] }
   
   # Optional
   git2 = { version = "0.18", optional = true }
   
   [dev-dependencies]
   assert_cmd = "2"
   predicates = "3"
   serial_test = "3"
   proptest = "1"        # Property-based testing
   insta = "1"          # Snapshot testing
   mockall = "0.12"     # Mocking framework
   tempfile = "3"       # Temporary file handling
      ```

2. **Core Module Structure**
   ```
   src/
   ├── main.rs
   ├── cli/
   │   ├── mod.rs
   │   ├── commands/
   │   └── output.rs
   ├── core/
   │   ├── mod.rs
   │   ├── result.rs
   │   ├── error.rs
   │   └── types.rs
   ├── git/
   │   ├── mod.rs
   │   └── executor.rs
   ├── worktree/
   │   ├── mod.rs
   │   └── types.rs
   └── process/
       ├── mod.rs
       └── executor.rs
   ```

3. **Implement Core Types and Error Handling**
   ```rust
   #[derive(Debug, thiserror::Error)]
   pub enum PhantomError {
       #[error("Git operation failed: {message}")]
       Git { message: String, exit_code: i32 },
       
       #[error("IO error: {0}")]
       Io(#[from] std::io::Error),
       
       #[error("Worktree '{name}' already exists")]
       WorktreeExists { name: String },
       
       #[error("Configuration error: {0}")]
       Config(String),
       
       #[error("Terminal multiplexer not found: {0}")]
       MultiplexerNotFound(String),
   }
   ```

## Phase 2: Git Operations - Command-based (Weeks 3-5)

### Goals
- Implement command-based git executor
- Create abstraction layer for future libgit2 integration
- Maintain same interface for higher-level operations

### Tasks
1. **Stage 1: Command-based Git Executor**
   ```rust
   use tokio::process::Command;
   
   pub struct GitExecutor;
   
   impl GitExecutor {
       pub async fn run(&self, args: &[&str]) -> Result<String> {
           let output = Command::new("git")
               .args(args)
               .output()
               .await?;
           
           if output.status.success() {
               Ok(String::from_utf8(output.stdout)?)
           } else {
               Err(PhantomError::Git {
                   message: String::from_utf8_lossy(&output.stderr).to_string(),
                   exit_code: output.status.code().unwrap_or(-1),
               })
           }
       }
   }
   ```

2. **Stage 2: Abstract Backend Design**
   ```rust
   #[async_trait]
   trait GitBackend {
       async fn list_worktrees(&self, repo_path: &Path) -> Result<Vec<Worktree>>;
       async fn create_worktree(&self, repo_path: &Path, name: &str) -> Result<()>;
       async fn delete_worktree(&self, repo_path: &Path, name: &str) -> Result<()>;
   }
   
   // Feature flag controlled
   #[cfg(feature = "libgit2")]
   struct LibGit2Backend;
   
   #[cfg(not(feature = "libgit2"))]
   struct CommandBackend;
   ```

3. **Port Git Operations**
   - `add_worktree`
   - `list_worktrees`
   - `get_current_branch`
   - `branch_exists`
   - etc.

## Phase 3: Async File Operations & Worktree Management (Weeks 6-7)

### Goals
- Implement async file operations for better performance
- Port worktree management with parallel operations
- Implement efficient file copying

### Tasks
1. **Async Worktree Operations**
   ```rust
   use tokio::fs;
   use tokio::io::AsyncReadExt;
   
   pub async fn create_worktree(name: &str, branch: Option<&str>) -> Result<()> {
       // Parallel operations for efficiency
       let (git_result, config_result) = tokio::join!(
           git_backend.create_worktree(name, branch),
           load_config()
       );
       
       git_result?;
       let config = config_result?;
       
       // Copy files asynchronously
       if !config.copy_files.is_empty() {
           copy_files_async(&config.copy_files).await?;
       }
       
       Ok(())
   }
   ```

2. **Async File Copier**
   ```rust
   use tokio::fs;
   use futures::stream::{self, StreamExt};
   
   async fn copy_files_async(patterns: &[String]) -> Result<()> {
       let files = collect_files_to_copy(patterns)?;
       
       // Copy files in parallel with concurrency limit
       stream::iter(files)
           .map(|file| async move {
               fs::copy(&file.src, &file.dst).await
           })
           .buffer_unordered(10) // Limit concurrent operations
           .try_collect()
           .await?;
       
       Ok(())
   }
   ```

3. **Configuration Management**
   - Consider TOML format for better Rust integration
   - Async config loading
   - Validation with serde

## Phase 4: Process & Terminal Integration (Weeks 8-9)

### Goals
- Implement Unix-specific process execution
- Add terminal multiplexer support
- Interactive shell support with proper TTY handling

### Tasks
1. **Unix Process Executor**
   ```rust
   use tokio::process::Command;
   use std::os::unix::fs::PermissionsExt;
   
   pub async fn spawn_shell(dir: &Path) -> Result<()> {
       let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/sh".to_string());
       
       Command::new(&shell)
           .current_dir(dir)
           .spawn()?
           .wait()
           .await?;
       
       Ok(())
   }
   
   pub fn make_executable(path: &Path) -> Result<()> {
       let mut perms = fs::metadata(path)?.permissions();
       perms.set_mode(0o755);
       fs::set_permissions(path, perms)?;
       Ok(())
   }
   ```

2. **Terminal Integration**
   ```rust
   // tmux support
   pub async fn open_in_tmux(dir: &Path, session: &str) -> Result<()> {
       Command::new("tmux")
           .args(&["new-session", "-d", "-s", session, "-c", dir.to_str().unwrap()])
           .output()
           .await?;
       Ok(())
   }
   
   // Kitty support via OSC sequences
   pub fn open_in_kitty(dir: &Path) -> Result<()> {
       println!("\x1b]7;file://{}\x1b\\", dir.display());
       Ok(())
   }
   ```

3. **Interactive Features**
   - Direct fzf integration via process spawning
   - TTY handling with `termion` for interactive shells
   - Unix signal handling for graceful shutdown

## Phase 5: CLI Implementation (Weeks 10-11)

### Goals
- Implement CLI using clap with async handlers
- Port all command handlers with exact compatibility
- Implement comprehensive completion support

### Tasks
1. **Async CLI Structure**
   ```rust
   use clap::{Parser, Subcommand};
   
   #[derive(Parser)]
   #[command(name = "phantom")]
   #[command(about = "Ephemeral Git worktrees made easy")]
   #[command(version)]
   struct Cli {
       #[command(subcommand)]
       command: Commands,
   }
   
   #[derive(Subcommand)]
   enum Commands {
       /// Create a new Git worktree (phantom)
       Create { 
           /// Name for the new phantom
           name: Option<String>,
           /// Copy files from current worktree
           #[arg(short, long)]
           copy: bool,
       },
       /// Attach to an existing branch
       Attach { 
           /// Branch name to attach to
           branch: String 
       },
       /// List all phantoms
       List,
       /// Execute a command in a phantom
       Exec {
           /// Phantom name
           phantom: String,
           /// Command to execute
           #[arg(trailing_var_arg = true)]
           command: Vec<String>,
       },
       // etc.
   }
   ```

2. **Async Command Handlers**
   ```rust
   #[tokio::main]
   async fn main() -> Result<()> {
       let cli = Cli::parse();
       
       // Initialize tracing
       tracing_subscriber::fmt()
           .with_env_filter(EnvFilter::from_default_env())
           .init();
       
       match cli.command {
           Commands::Create { name, copy } => {
               create_handler(name, copy).await?
           }
           Commands::List => {
               list_handler().await?
           }
           // etc.
       }
       
       Ok(())
   }
   ```

3. **Output and Compatibility**
   - Maintain exact output format
   - Same exit codes as TypeScript version
   - Support for NO_COLOR and FORCE_COLOR env vars

## Phase 6: Testing & Distribution (Weeks 12-13)

### Goals
- Comprehensive test suite with property-based testing
- Performance benchmarking against TypeScript baseline
- Simplified Unix-only distribution

### Tasks
1. **Comprehensive Testing Strategy**
   ```rust
   // Property-based testing example
   use proptest::prelude::*;
   
   proptest! {
       #[test]
       fn test_worktree_name_validation(name in "[a-zA-Z0-9-_]{1,255}") {
           let result = validate_worktree_name(&name);
           prop_assert!(result.is_ok());
       }
   }
   
   // Snapshot testing for output
   #[test]
   fn test_list_output() {
       let output = list_worktrees().unwrap();
       insta::assert_snapshot!(output);
   }
   ```

2. **Performance Benchmarking**
   ```rust
   use criterion::{black_box, criterion_group, criterion_main, Criterion};
   
   fn bench_list_worktrees(c: &mut Criterion) {
       let repo = setup_test_repo();
       
       c.bench_function("list_worktrees", |b| {
           b.iter(|| list_worktrees(black_box(&repo)))
       });
   }
   
   criterion_group!(benches, bench_list_worktrees);
   criterion_main!(benches);
   ```

3. **Unix-only Distribution**
   ```yaml
   # .github/workflows/release.yml
   name: Release
   on:
     push:
       tags:
         - 'v*'
   
   jobs:
     build:
       strategy:
         matrix:
           include:
             - os: ubuntu-latest
               target: x86_64-unknown-linux-gnu
             - os: macos-latest
               target: x86_64-apple-darwin
             - os: macos-latest
               target: aarch64-apple-darwin
       runs-on: ${{ matrix.os }}
       steps:
         - uses: actions/checkout@v4
         - uses: dtolnay/rust-toolchain@stable
         - run: cargo build --release --target ${{ matrix.target }}
         - run: cargo test --release
   ```

## Migration Execution Strategy

### Gradual Migration Approach
1. **Phase-by-Phase Implementation**
   - Start with command-based git operations (no FFI complexity)
   - Test each phase thoroughly before proceeding
   - Maintain TypeScript version in parallel

2. **Testing Strategy**
   - Feature parity tests between TypeScript and Rust
   - Performance comparison benchmarks
   - User acceptance testing with beta releases

3. **Release Strategy**
   - Alpha releases for early adopters (v2.0.0-alpha.x)
   - Beta releases with feature parity (v2.0.0-beta.x)
   - Stable release when all tests pass (v2.0.0)

### Risk Mitigation
1. **No FFI Complexity**
   - Direct port without Node.js bindings reduces risk
   - Clean break allows for better optimization

2. **Unix-only Focus**
   - Eliminates cross-platform edge cases
   - Simpler testing matrix
   - Better performance optimizations

## Technical Considerations

### Unix-Specific Optimizations
```rust
// Simple path handling without Windows complexity
use std::path::{Path, PathBuf};

fn normalize_path(path: &Path) -> PathBuf {
    path.to_path_buf() // No UNC path handling needed
}

// Unix shell detection
fn get_default_shell() -> String {
    std::env::var("SHELL").unwrap_or_else(|_| {
        if cfg!(target_os = "macos") {
            "/bin/zsh".to_string()
        } else {
            "/bin/bash".to_string()
        }
    })
}
```

### Async-First Architecture
- Use Tokio from the start for all I/O operations
- Leverage async for parallel git operations
- Better resource utilization with async file copying

### Enhanced Error Handling
- Structured errors with `thiserror`
- Context with `anyhow` for internal errors
- User-friendly error messages matching TypeScript version

## Dependencies Mapping

| TypeScript | Rust | Purpose |
|------------|------|---------|
| Built-in fs | tokio::fs | Async file operations |
| Built-in child_process | tokio::process | Async process execution |
| N/A | clap | CLI parsing |
| N/A | serde/serde_json | Config parsing |
| N/A | tokio | Async runtime |
| N/A | nix | Unix system calls |
| N/A | termion | Terminal control |
| External fzf | Direct fzf via subprocess | Fuzzy finding |
| N/A | tracing/tracing-subscriber | Logging/debugging |

## Success Metrics

1. **Performance**: 50%+ improvement in common operations
2. **Binary Size**: < 5MB stripped binary (reduced from 10MB)
3. **Test Coverage**: > 85% code coverage
4. **Compatibility**: 100% command compatibility
5. **Memory Usage**: < 50% of TypeScript version
6. **Installation**: Single binary, no Node.js requirement

## Risks & Mitigation

| Risk | Mitigation |
|------|------------|
| Performance regression | Comprehensive benchmarks from day one |
| Git version differences | Test with multiple git versions (2.20+) |
| Migration period challenges | Feature parity tests, automated compatibility checks |
| Async complexity | Start with async, use well-tested Tokio patterns |
| Terminal handling differences | Use termion for consistent Unix terminal control |

## Revised Timeline (Unix-only)

| Phase | Description | Duration | Cumulative |
|-------|-------------|----------|------------|
| 1 | Foundation & Core Types | 2 weeks | 2 weeks |
| 2 | Git Operations (Command-based) | 3 weeks | 5 weeks |
| 3 | Async File Operations | 2 weeks | 7 weeks |
| 4 | Process & Terminal Integration | 2 weeks | 9 weeks |
| 5 | CLI Implementation | 2 weeks | 11 weeks |
| 6 | Testing & Distribution | 2 weeks | 13 weeks |

**Total estimated time: 13 weeks** (reduced from 18 weeks with Windows support)

## Additional Considerations

### Logging and Debugging
- Use `RUST_LOG` environment variable for debug output
- Structured logging with `tracing` for better observability
- Compatible with existing debug workflows

### Configuration Format Migration
Consider migrating from JSON to TOML for better Rust ecosystem integration:
```toml
# phantom.toml
[copy]
files = [".env", ".env.local", "config/"]

[terminal]
multiplexer = "tmux" # or "kitty"
```

### Documentation Strategy
- Generate API docs with `cargo doc`
- Maintain user-facing documentation compatibility
- Add Rust-specific installation instructions

## Next Steps

1. Finalize Unix-only decision with stakeholders
2. Set up Rust project with async-first architecture
3. Implement comprehensive benchmarking framework
4. Create detailed migration tracking issues
5. Begin Phase 1 implementation with Tokio runtime

## Conclusion

By excluding Windows support and adopting an async-first approach, the Rust migration becomes significantly more straightforward. The reduced complexity allows for:
- Faster development (5 weeks saved)
- Cleaner codebase (no conditional compilation for Windows)
- Better performance (Unix-specific optimizations)
- Simplified maintenance and testing

The command-based Git implementation strategy eliminates FFI complexity while maintaining a clear path to future libgit2 integration if needed.