# Phantom Benchmarks

This directory contains performance benchmarks for the Phantom CLI tool.

## Running Benchmarks

### Run all benchmarks:
```bash
cargo bench
```

### Run specific benchmark suite:
```bash
cargo bench --bench phantom_benchmarks
cargo bench --bench optimization_benchmarks
```

### Run specific benchmark function:
```bash
cargo bench -- string_validation
cargo bench -- get_git_root
```

### Generate HTML reports:
```bash
cargo bench -- --save-baseline my_baseline
```

Reports are generated in `target/criterion/`

## Benchmark Suites

### phantom_benchmarks.rs
General performance benchmarks covering:
- Command execution patterns
- String validation operations
- Builder pattern performance
- Worktree operations
- Concurrent operations
- Memory allocation patterns
- Startup time (critical for CLI tools)

### optimization_benchmarks.rs
Specific benchmarks for recent optimizations:
- Generic vs dynamic dispatch for git operations
- Cloning patterns (context vs Arc)
- Cow<str> string optimizations
- SmallVec vs Vec for command arguments

## Performance Goals

Based on our benchmarks, we aim for:
- CLI startup time < 50ms
- Worktree creation < 100ms
- List operations < 200ms for 100 worktrees
- Zero-copy string operations where possible

## Interpreting Results

Criterion generates detailed statistical analysis:
- **Time**: Lower is better
- **Throughput**: Higher is better
- **Outliers**: Watch for high variance
- **Regression**: Compare against baselines

## Continuous Benchmarking

For CI integration, use:
```bash
cargo bench -- --save-baseline main
cargo bench -- --baseline main
```

This helps catch performance regressions in PRs.