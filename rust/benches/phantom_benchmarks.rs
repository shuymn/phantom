use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use phantom::core::command_executor::{CommandConfig, CommandExecutor};
use phantom::core::executors::RealCommandExecutor;
use phantom::core::extension_traits::{CommandExecutorExt, StrExt, WorktreeExt};
use phantom::core::types::Worktree;
use phantom::worktree::builder::build_worktree;
use phantom::worktree::validate::validate_worktree_name;
use std::path::PathBuf;
use std::time::Duration;

/// Benchmark command execution patterns
fn bench_command_execution(c: &mut Criterion) {
    let executor = RealCommandExecutor;
    let mut group = c.benchmark_group("command_execution");

    // Benchmark simple command execution
    group.bench_function("echo_simple", |b| {
        b.iter(|| {
            tokio::runtime::Runtime::new().unwrap().block_on(async {
                let _ = executor.run_simple("echo", &["test"]).await;
            });
        });
    });

    // Benchmark command with config
    group.bench_function("echo_with_config", |b| {
        b.iter(|| {
            tokio::runtime::Runtime::new().unwrap().block_on(async {
                let config = CommandConfig::new("echo").with_args(vec!["test".to_string()]);
                let _ = executor.execute(config).await;
            });
        });
    });

    group.finish();
}

/// Benchmark string validation operations
fn bench_string_validation(c: &mut Criterion) {
    let mut group = c.benchmark_group("string_validation");

    // Test various worktree name patterns
    let test_names = vec![
        "simple",
        "feature-branch",
        "feature/complex/branch",
        "very-long-branch-name-with-many-segments",
        "branch_with_underscores",
        "123-numeric-prefix",
    ];

    for name in &test_names {
        group.bench_with_input(
            BenchmarkId::new("validate_worktree_name", name),
            name,
            |b, name| {
                b.iter(|| {
                    let _ = validate_worktree_name(black_box(name));
                });
            },
        );
    }

    // Benchmark string extension trait methods
    group.bench_function("is_branch_like", |b| {
        b.iter(|| {
            black_box("feature/branch-123").is_branch_like();
        });
    });

    group.bench_function("is_commit_like", |b| {
        b.iter(|| {
            black_box("abc123def456").is_commit_like();
        });
    });

    group.bench_function("sanitize_worktree_name", |b| {
        b.iter(|| {
            black_box("feature@branch#123!").sanitize_worktree_name();
        });
    });

    group.finish();
}

/// Benchmark builder pattern performance
fn bench_builder_pattern(c: &mut Criterion) {
    let mut group = c.benchmark_group("builder_pattern");

    // Simple builder
    group.bench_function("build_simple", |b| {
        b.iter(|| {
            let _ = build_worktree().name("feature").build_unchecked();
        });
    });

    // Complex builder with all options
    group.bench_function("build_complex", |b| {
        b.iter(|| {
            let _ = build_worktree()
                .name("feature")
                .branch("feature/new")
                .base("main")
                .copy_file(".env")
                .copy_file("config.json")
                .build_unchecked();
        });
    });

    // Builder with validation
    group.bench_function("build_with_validation", |b| {
        b.iter(|| {
            let _ = build_worktree().name("feature").validate().map(|v| v.build());
        });
    });

    group.finish();
}

/// Benchmark worktree operations
fn bench_worktree_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("worktree_operations");

    // Create test worktree
    let worktree = Worktree {
        name: "feature".to_string(),
        path: PathBuf::from("/repo/.git/phantom/worktrees/feature"),
        branch: Some("feature/new-ui".to_string()),
        commit: "abc123def456".to_string(),
        is_bare: false,
        is_detached: false,
        is_locked: false,
        is_prunable: false,
    };

    // Benchmark extension trait methods
    group.bench_function("is_main", |b| {
        b.iter(|| {
            black_box(&worktree).is_main();
        });
    });

    group.bench_function("display_name", |b| {
        b.iter(|| {
            black_box(&worktree).display_name();
        });
    });

    group.bench_function("relative_path", |b| {
        b.iter(|| {
            black_box(&worktree).relative_path();
        });
    });

    group.finish();
}

/// Benchmark concurrent operations
fn bench_concurrent_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("concurrent_operations");
    group.measurement_time(Duration::from_secs(10));

    // Benchmark different concurrency levels
    for num_tasks in [1, 5, 10, 20].iter() {
        group.bench_with_input(
            BenchmarkId::new("concurrent_futures", num_tasks),
            num_tasks,
            |b, &num_tasks| {
                b.iter(|| {
                    tokio::runtime::Runtime::new().unwrap().block_on(async {
                        use futures::future::join_all;

                        let futures: Vec<_> = (0..num_tasks)
                            .map(|i| async move {
                                // Simulate some async work
                                tokio::time::sleep(Duration::from_micros(10)).await;
                                i
                            })
                            .collect();

                        let _ = join_all(futures).await;
                    });
                });
            },
        );
    }

    group.finish();
}

/// Benchmark memory allocations
fn bench_memory_patterns(c: &mut Criterion) {
    use smallvec::SmallVec;
    let mut group = c.benchmark_group("memory_patterns");

    // Vec vs SmallVec for command arguments
    group.bench_function("vec_args_small", |b| {
        b.iter(|| {
            let args: Vec<String> = vec!["arg1".to_string(), "arg2".to_string()];
            black_box(args);
        });
    });

    group.bench_function("smallvec_args_small", |b| {
        b.iter(|| {
            let mut args: SmallVec<[String; 4]> = SmallVec::new();
            args.push("arg1".to_string());
            args.push("arg2".to_string());
            black_box(args);
        });
    });

    // Larger argument lists
    group.bench_function("vec_args_large", |b| {
        b.iter(|| {
            let args: Vec<String> = (0..10).map(|i| format!("arg{}", i)).collect();
            black_box(args);
        });
    });

    group.bench_function("smallvec_args_large", |b| {
        b.iter(|| {
            let args: SmallVec<[String; 4]> = (0..10).map(|i| format!("arg{}", i)).collect();
            black_box(args);
        });
    });

    group.finish();
}

/// Benchmark startup time (critical for CLI tools)
fn bench_startup_time(c: &mut Criterion) {
    use phantom::cli::context::ProductionContext;

    let mut group = c.benchmark_group("startup");

    // Benchmark context creation
    group.bench_function("context_creation", |b| {
        b.iter(|| {
            let _ = ProductionContext::default();
        });
    });

    // Benchmark CLI argument parsing
    group.bench_function("cli_parsing", |b| {
        use clap::Parser;
        use phantom::cli::Cli;

        b.iter(|| {
            let args = vec!["phantom", "list"];
            let _ = Cli::try_parse_from(args);
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_command_execution,
    bench_string_validation,
    bench_builder_pattern,
    bench_worktree_operations,
    bench_concurrent_operations,
    bench_memory_patterns,
    bench_startup_time
);
criterion_main!(benches);
