use criterion::{black_box, criterion_group, criterion_main, Criterion};
use phantom_rs::core::executors::MockCommandExecutor;
use phantom_rs::git::libs::get_git_root::get_git_root;
use std::sync::Arc;

/// Benchmark get_git_root performance
fn bench_get_git_root_performance(c: &mut Criterion) {
    let mut group = c.benchmark_group("get_git_root_performance");

    // Setup mock executor for consistent results
    let mut mock = MockCommandExecutor::new();
    for _ in 0..100 {
        mock.expect_command("git").with_args(&["rev-parse", "--git-common-dir"]).returns_output(
            "/home/user/project/.git",
            "",
            0,
        );
    }

    // Benchmark get_git_root function
    group.bench_function("get_git_root", |b| {
        let executor = mock.clone();
        b.iter(|| {
            tokio::runtime::Runtime::new().unwrap().block_on(async {
                let _ = get_git_root(executor.clone()).await;
            });
        });
    });

    group.finish();
}

/// Benchmark cloning patterns
fn bench_cloning_patterns(c: &mut Criterion) {
    use phantom_rs::cli::context::ProductionContext;

    let mut group = c.benchmark_group("cloning_patterns");

    // Benchmark context cloning (lightweight types)
    group.bench_function("context_clone", |b| {
        let context = ProductionContext::default();
        b.iter(|| {
            let _ = black_box(context.clone());
        });
    });

    // Benchmark Arc cloning for comparison
    group.bench_function("arc_context_clone", |b| {
        let context = Arc::new(ProductionContext::default());
        b.iter(|| {
            let _ = black_box(Arc::clone(&context));
        });
    });

    // Benchmark large data structure cloning
    #[derive(Clone)]
    struct LargeData {
        #[allow(dead_code)]
        data: Vec<String>,
    }

    let large_data = LargeData { data: (0..1000).map(|i| format!("Item {i}")).collect() };

    group.bench_function("large_data_clone", |b| {
        b.iter(|| {
            let _ = black_box(large_data.clone());
        });
    });

    let arc_large_data = Arc::new(large_data);

    group.bench_function("arc_large_data_clone", |b| {
        b.iter(|| {
            let _ = black_box(Arc::clone(&arc_large_data));
        });
    });

    group.finish();
}

/// Benchmark string operations with Cow
fn bench_cow_strings(c: &mut Criterion) {
    use std::borrow::Cow;

    let mut group = c.benchmark_group("cow_strings");

    // Benchmark owned string creation
    group.bench_function("owned_string", |b| {
        b.iter(|| {
            let output = "Hello, World!".to_string();
            black_box(output);
        });
    });

    // Benchmark Cow borrowed
    group.bench_function("cow_borrowed", |b| {
        b.iter(|| {
            let output: Cow<str> = Cow::Borrowed("Hello, World!");
            black_box(output);
        });
    });

    // Benchmark Cow with conditional modification
    group.bench_function("cow_conditional_modify", |b| {
        let input = "Hello, World!";
        b.iter(|| {
            let output: Cow<str> = if black_box(false) {
                Cow::Owned(input.replace("World", "Rust"))
            } else {
                Cow::Borrowed(input)
            };
            black_box(output);
        });
    });

    group.finish();
}

/// Benchmark SmallVec vs Vec for command arguments
fn bench_smallvec_optimization(c: &mut Criterion) {
    use smallvec::SmallVec;

    let mut group = c.benchmark_group("smallvec_optimization");

    // Small argument list (typical case)
    let small_args = ["status", "--porcelain"];

    group.bench_function("vec_small", |b| {
        b.iter(|| {
            let args: Vec<String> = small_args.iter().map(|s| s.to_string()).collect();
            black_box(args);
        });
    });

    group.bench_function("smallvec_small", |b| {
        b.iter(|| {
            let args: SmallVec<[String; 4]> = small_args.iter().map(|s| s.to_string()).collect();
            black_box(args);
        });
    });

    // Large argument list (edge case)
    let large_args: Vec<&str> = (0..10).map(|_| "arg").collect();

    group.bench_function("vec_large", |b| {
        b.iter(|| {
            let args: Vec<String> = large_args.iter().map(|s| s.to_string()).collect();
            black_box(args);
        });
    });

    group.bench_function("smallvec_large", |b| {
        b.iter(|| {
            let args: SmallVec<[String; 4]> = large_args.iter().map(|s| s.to_string()).collect();
            black_box(args);
        });
    });

    group.finish();
}

criterion_group!(
    optimization_benches,
    bench_get_git_root_performance,
    bench_cloning_patterns,
    bench_cow_strings,
    bench_smallvec_optimization
);
criterion_main!(optimization_benches);
