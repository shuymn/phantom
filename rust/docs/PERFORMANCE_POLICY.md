# Performance Policy

This document outlines Phantom's performance standards and policies to ensure a fast, responsive CLI experience.

## Performance Standards

### 1. Startup Time

**Target**: < 50ms for all commands

**Measurement**:
- Time from process start to first user-visible output
- Measured on standard development hardware (4-core CPU, 8GB RAM)
- Includes argument parsing and initial setup

**Implementation**:
```rust
// Lazy initialization for heavy resources
use once_cell::sync::Lazy;
static CONFIG: Lazy<Config> = Lazy::new(|| load_config());

// Avoid blocking operations during startup
async fn main() {
    // Parse args first (fast)
    let args = parse_args();
    
    // Initialize only what's needed
    match args.command {
        Command::List => handle_list().await,
        Command::Create => handle_create().await,
        // ...
    }
}
```

### 2. Memory Usage

**Target**: Predictable and bounded memory usage

**Guidelines**:
- No unbounded growth in long-running operations
- Memory usage should scale linearly with input size
- Clean up resources promptly

**Implementation**:
```rust
// Use streaming for large data
use futures::stream::{self, StreamExt};

async fn process_worktrees(paths: Vec<PathBuf>) -> Result<()> {
    // Process in chunks to bound memory
    stream::iter(paths)
        .chunks(100)
        .for_each(|chunk| async {
            process_chunk(chunk).await;
        })
        .await;
}

// Use SmallVec for collections usually small
use smallvec::SmallVec;
type Args = SmallVec<[String; 4]>; // Stack-allocated for ≤4 items
```

### 3. Responsiveness

**Target**: Immediate feedback for all operations

**Guidelines**:
- Show progress for operations > 100ms
- Use concurrent operations where beneficial
- Never block the main thread

**Implementation**:
```rust
// Concurrent operations
use futures::stream::FuturesUnordered;

let results: Vec<_> = worktrees
    .into_iter()
    .map(|w| check_status(w))
    .collect::<FuturesUnordered<_>>()
    .collect()
    .await;

// Progress indication
if items.len() > 10 {
    eprintln!("Processing {} items...", items.len());
}
```

## Performance Optimization Strategies

### 1. Zero-Cost Abstractions

Prefer compile-time polymorphism over runtime dispatch:

```rust
// ✅ Good: Generic function (monomorphized at compile time)
pub async fn execute<E: CommandExecutor>(executor: E, cmd: &str) -> Result<String> {
    executor.run(cmd).await
}

// ❌ Avoid: Dynamic dispatch (runtime overhead)
pub async fn execute(executor: Box<dyn CommandExecutor>, cmd: &str) -> Result<String> {
    executor.run(cmd).await
}
```

### 2. Smart Pointer Usage

Choose the right tool for ownership:

```rust
// Decision tree for pointer types:
//
// Need shared ownership?
//   ├─ Yes
//   │   ├─ Across threads? → Arc<T>
//   │   └─ Single thread?  → Rc<T>
//   └─ No
//       ├─ Small data?     → Clone/Copy
//       └─ Large data?     → &T or Box<T>
```

### 3. String Optimization

Minimize allocations with smart string types:

```rust
use std::borrow::Cow;

// Flexible: borrows when possible, clones when necessary
fn process_path(path: Cow<str>) -> Cow<str> {
    if path.contains(' ') {
        // Only allocate if modification needed
        Cow::Owned(path.replace(' ', "_"))
    } else {
        path // No allocation
    }
}
```

### 4. Collection Optimization

Use appropriate collection types:

```rust
// For small, fixed-size collections
use smallvec::SmallVec;
let args: SmallVec<[String; 4]> = SmallVec::new();

// For string interning
use string_cache::DefaultAtom;
let interned = DefaultAtom::from("frequently_used_string");

// For concurrent access
use dashmap::DashMap;
let cache: DashMap<String, Status> = DashMap::new();
```

## Benchmarking and Monitoring

### Running Benchmarks

```bash
# Run all benchmarks
cargo bench

# Run specific benchmark suite
cargo bench phantom_benchmarks

# Generate flame graph
cargo bench --bench optimization_benchmarks -- --profile-time=10
```

### Continuous Performance Monitoring

1. **CI Integration**: Run benchmarks on every PR
2. **Regression Detection**: Compare against baseline
3. **Performance Budget**: Fail builds that exceed limits

```yaml
# Example CI check
- name: Performance Check
  run: |
    cargo bench -- --save-baseline pr
    cargo bench -- --baseline main
    # Script to check for regressions
    ./scripts/check-performance.sh
```

### Performance Profiling

Tools for investigating performance issues:

```bash
# CPU profiling
cargo build --release
perf record --call-graph=dwarf target/release/phantom list
perf report

# Memory profiling
valgrind --tool=massif target/release/phantom create feature
ms_print massif.out.*

# Async runtime analysis
TOKIO_CONSOLE=1 cargo run --features tokio-console
```

## Performance Checklist

Before merging any PR:

- [ ] No new cloning of large data structures in hot paths
- [ ] Async operations use concurrent execution where beneficial
- [ ] String allocations minimized with Cow<str> or &str
- [ ] Collections use appropriate types (SmallVec, etc.)
- [ ] No blocking operations in async contexts
- [ ] Benchmarks show no significant regression
- [ ] Memory usage remains bounded

## Common Performance Pitfalls

### 1. Excessive Cloning
```rust
// ❌ Bad: Clones entire vector
let items = all_items.clone();

// ✅ Good: Takes ownership or borrows
let items = all_items; // or &all_items
```

### 2. Blocking in Async
```rust
// ❌ Bad: Blocks the executor
let data = std::fs::read_to_string("file.txt")?;

// ✅ Good: Async I/O
let data = tokio::fs::read_to_string("file.txt").await?;
```

### 3. Unbounded Concurrency
```rust
// ❌ Bad: Could spawn thousands of tasks
let handles: Vec<_> = items
    .iter()
    .map(|item| tokio::spawn(process(item)))
    .collect();

// ✅ Good: Bounded concurrency
use futures::stream::{self, StreamExt};
let results = stream::iter(items)
    .map(|item| process(item))
    .buffer_unordered(10) // Max 10 concurrent
    .collect()
    .await;
```

### 4. Inefficient String Building
```rust
// ❌ Bad: Many allocations
let mut s = String::new();
for item in items {
    s = s + &item.to_string() + ", ";
}

// ✅ Good: Single allocation
let s = items
    .iter()
    .map(|i| i.to_string())
    .collect::<Vec<_>>()
    .join(", ");
```

## Performance Goals by Command

| Command | Target Time | Notes |
|---------|-------------|-------|
| `phantom list` | < 20ms | Should be near-instant |
| `phantom create` | < 50ms | Excluding git operations |
| `phantom attach` | < 30ms | Fast switching |
| `phantom where` | < 10ms | Simple path lookup |
| `phantom delete` | < 50ms | Excluding git operations |

## Future Optimizations

Potential areas for future performance work:

1. **Compile-time optimization**:
   - Profile-guided optimization (PGO)
   - Link-time optimization (LTO)

2. **Runtime optimization**:
   - Command prediction and preloading
   - Persistent daemon mode for instant response

3. **Memory optimization**:
   - Arena allocation for temporary objects
   - Custom allocators for specific patterns

## References

- [The Rust Performance Book](https://nnethercote.github.io/perf-book/)
- [Tokio Performance Tuning](https://tokio.rs/tokio/topics/tracing)
- [Criterion.rs Documentation](https://bheisler.github.io/criterion.rs/book/)