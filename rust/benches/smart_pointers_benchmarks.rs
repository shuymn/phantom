/// Benchmarks for custom smart pointers
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use phantom::core::smart_pointers::{SmallBox, SmallVec};

/// Benchmark SmallBox vs Box for small values
fn bench_small_box_vs_box_small(c: &mut Criterion) {
    let mut group = c.benchmark_group("small_box_vs_box_small");

    // Benchmark Box<u64>
    group.bench_function("box_u64", |b| {
        b.iter(|| {
            let boxed = Box::new(black_box(42u64));
            black_box(boxed);
        });
    });

    // Benchmark SmallBox<u64> (inline)
    group.bench_function("smallbox_u64_inline", |b| {
        b.iter(|| {
            let boxed = SmallBox::<u64, 64>::new(black_box(42u64));
            black_box(boxed);
        });
    });

    // Benchmark Box<String> with small string
    group.bench_function("box_small_string", |b| {
        b.iter(|| {
            let boxed = Box::new(black_box("Hello".to_string()));
            black_box(boxed);
        });
    });

    // Benchmark SmallBox<String> with small string (inline)
    group.bench_function("smallbox_small_string_inline", |b| {
        b.iter(|| {
            let boxed = SmallBox::<String, 64>::new(black_box("Hello".to_string()));
            black_box(boxed);
        });
    });

    group.finish();
}

/// Benchmark SmallBox vs Box for large values
fn bench_small_box_vs_box_large(c: &mut Criterion) {
    let mut group = c.benchmark_group("small_box_vs_box_large");

    // Large array that won't fit inline
    let large_data = [0u8; 1024];

    // Benchmark Box<[u8; 1024]>
    group.bench_function("box_large_array", |b| {
        b.iter(|| {
            let boxed = Box::new(black_box(large_data));
            black_box(boxed);
        });
    });

    // Benchmark SmallBox<[u8; 1024]> (heap)
    group.bench_function("smallbox_large_array_heap", |b| {
        b.iter(|| {
            let boxed = SmallBox::<[u8; 1024], 64>::new(black_box(large_data));
            black_box(boxed);
        });
    });

    group.finish();
}

/// Benchmark SmallVec vs Vec
fn bench_small_vec_vs_vec(c: &mut Criterion) {
    let mut group = c.benchmark_group("small_vec_vs_vec");

    for size in [2, 4, 8, 16].iter() {
        // Benchmark Vec
        group.bench_with_input(BenchmarkId::new("vec", size), size, |b, &size| {
            b.iter(|| {
                let mut vec = Vec::new();
                for i in 0..size {
                    vec.push(black_box(i));
                }
                black_box(vec);
            });
        });

        // Benchmark SmallVec
        group.bench_with_input(BenchmarkId::new("smallvec", size), size, |b, &size| {
            b.iter(|| {
                let mut vec = SmallVec::<usize, 8>::new();
                for i in 0..size {
                    vec.push(black_box(i));
                }
                black_box(vec);
            });
        });
    }

    group.finish();
}

/// Benchmark real-world usage patterns
fn bench_real_world_patterns(c: &mut Criterion) {
    let mut group = c.benchmark_group("real_world_patterns");

    // Simulate git command arguments
    group.bench_function("vec_git_args", |b| {
        b.iter(|| {
            let mut args = Vec::new();
            args.push("status".to_string());
            args.push("--porcelain".to_string());
            args.push("--branch".to_string());
            black_box(args);
        });
    });

    group.bench_function("smallvec_git_args", |b| {
        b.iter(|| {
            let mut args = SmallVec::<String, 4>::new();
            args.push("status".to_string());
            args.push("--porcelain".to_string());
            args.push("--branch".to_string());
            black_box(args);
        });
    });

    // Simulate small error messages
    group.bench_function("box_error_message", |b| {
        b.iter(|| {
            let error = Box::new("File not found".to_string());
            black_box(error);
        });
    });

    group.bench_function("smallbox_error_message", |b| {
        b.iter(|| {
            let error = SmallBox::<String, 64>::new("File not found".to_string());
            black_box(error);
        });
    });

    group.finish();
}

/// Benchmark allocation patterns in loops
fn bench_allocation_patterns(c: &mut Criterion) {
    let mut group = c.benchmark_group("allocation_patterns");

    const ITERATIONS: usize = 1000;

    // Many small allocations with Box
    group.bench_function("box_many_small", |b| {
        b.iter(|| {
            let mut boxes = Vec::with_capacity(ITERATIONS);
            for i in 0..ITERATIONS {
                boxes.push(Box::new(i as u32));
            }
            black_box(boxes);
        });
    });

    // Many small allocations with SmallBox
    group.bench_function("smallbox_many_small", |b| {
        b.iter(|| {
            let mut boxes = Vec::with_capacity(ITERATIONS);
            for i in 0..ITERATIONS {
                boxes.push(SmallBox::<u32, 8>::new(i as u32));
            }
            black_box(boxes);
        });
    });

    // Mixed size allocations - use enum to handle different sizes
    #[allow(dead_code)]
    enum MixedData {
        Small([u8; 16]),
        Large([u8; 256]),
    }

    group.bench_function("mixed_allocations_box", |b| {
        b.iter(|| {
            let mut results = Vec::new();
            for i in 0..100 {
                if i % 10 == 0 {
                    // Large allocation
                    results.push(Box::new(MixedData::Large([0u8; 256])));
                } else {
                    // Small allocation
                    results.push(Box::new(MixedData::Small([0u8; 16])));
                }
            }
            black_box(results);
        });
    });

    group.bench_function("mixed_allocations_smallbox", |b| {
        b.iter(|| {
            let mut results = Vec::new();
            for i in 0..100 {
                if i % 10 == 0 {
                    // Large allocation (heap)
                    results.push(SmallBox::<MixedData, 64>::new(MixedData::Large([0u8; 256])));
                } else {
                    // Small allocation (inline)
                    results.push(SmallBox::<MixedData, 64>::new(MixedData::Small([0u8; 16])));
                }
            }
            black_box(results);
        });
    });

    group.finish();
}

criterion_group!(
    smart_pointer_benches,
    bench_small_box_vs_box_small,
    bench_small_box_vs_box_large,
    bench_small_vec_vs_vec,
    bench_real_world_patterns,
    bench_allocation_patterns
);
criterion_main!(smart_pointer_benches);
