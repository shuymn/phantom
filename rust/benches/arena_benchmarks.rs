/// Benchmarks for arena allocation vs standard allocation
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use phantom::core::executors::MockCommandExecutor;
use phantom::worktree::batch_processor::BatchProcessor;
use phantom::worktree::list::WorktreeInfo;
use std::path::PathBuf;
use std::sync::Arc;
use typed_arena::Arena;

/// Benchmark arena-based batch processing
fn bench_arena_batch_processing(c: &mut Criterion) {
    let mut group = c.benchmark_group("batch_processing");

    for size in [10, 50, 100, 500].iter() {
        // Benchmark arena-based approach
        group.bench_with_input(BenchmarkId::new("arena", size), size, |b, &worktree_count| {
            b.iter(|| {
                tokio::runtime::Runtime::new().unwrap().block_on(async {
                    let info_arena = Arena::new();
                    let error_arena = Arena::new();
                    let string_arena = Arena::new();

                    let processor = BatchProcessor::new(&info_arena, &error_arena, &string_arena);

                    let mock = create_mock_with_worktrees(worktree_count);
                    let _ = processor
                        .process_worktrees_with_status(Arc::new(mock), &PathBuf::from("/repo"))
                        .await;
                });
            });
        });

        // Benchmark Vec-based approach (for comparison)
        group.bench_with_input(BenchmarkId::new("vec", size), size, |b, &worktree_count| {
            b.iter(|| {
                tokio::runtime::Runtime::new().unwrap().block_on(async {
                    let mock = create_mock_with_worktrees(worktree_count);
                    let _ = process_with_vec(Arc::new(mock), worktree_count).await;
                });
            });
        });
    }

    group.finish();
}

/// Benchmark memory allocation patterns
fn bench_allocation_patterns(c: &mut Criterion) {
    let mut group = c.benchmark_group("allocation_patterns");

    // Benchmark arena allocation
    group.bench_function("arena_allocation", |b| {
        b.iter(|| {
            let info_arena = Arena::<WorktreeInfo>::new();
            let string_arena = Arena::<String>::new();

            // Allocate 1000 items
            for i in 0..1000 {
                let name = string_arena.alloc(format!("worktree-{}", i));
                let path = string_arena.alloc(format!("/repo/worktree-{}", i));
                info_arena.alloc(WorktreeInfo {
                    name: name.clone(),
                    path: path.clone(),
                    branch: Some(format!("branch-{}", i)),
                    is_clean: i % 2 == 0,
                });
            }

            black_box(&info_arena);
            black_box(&string_arena);
        });
    });

    // Benchmark Vec allocation
    group.bench_function("vec_allocation", |b| {
        b.iter(|| {
            let mut infos = Vec::with_capacity(1000);

            // Allocate 1000 items
            for i in 0..1000 {
                infos.push(WorktreeInfo {
                    name: format!("worktree-{}", i),
                    path: format!("/repo/worktree-{}", i),
                    branch: Some(format!("branch-{}", i)),
                    is_clean: i % 2 == 0,
                });
            }

            black_box(infos);
        });
    });

    group.finish();
}

// Helper functions

fn create_mock_with_worktrees(count: usize) -> MockCommandExecutor {
    let mut mock = MockCommandExecutor::new();

    // Create worktree list output
    let mut output = String::from("worktree /repo\nHEAD abc123\nbranch refs/heads/main\n");
    for i in 0..count {
        output.push_str(&format!(
            "\nworktree /repo/.git/phantom/worktrees/feature-{}\nHEAD def{:03}\nbranch refs/heads/feature-{}\n",
            i, i, i
        ));
    }

    mock.expect_command("git")
        .with_args(&["worktree", "list", "--porcelain"])
        .returns_output(&output, "", 0);

    // Mock status checks
    for i in 0..count {
        mock.expect_command("git")
            .with_args(&["status", "--porcelain"])
            .in_dir(&format!("/repo/.git/phantom/worktrees/feature-{}", i))
            .returns_output(if i % 3 == 0 { "M file.txt\n" } else { "" }, "", 0);
    }

    mock
}

/// Vec-based processing for comparison
async fn process_with_vec(executor: Arc<MockCommandExecutor>, count: usize) -> Vec<WorktreeInfo> {
    use phantom::git::libs::list_worktrees::list_worktrees_with_executor;
    use phantom::worktree::list::get_worktree_status_with_executor;
    use phantom::worktree::paths::get_phantom_directory;

    let git_root = PathBuf::from("/repo");
    let git_worktrees = list_worktrees_with_executor(executor.clone(), &git_root).await.unwrap();
    let phantom_dir = get_phantom_directory(&git_root);

    let mut results = Vec::with_capacity(count);

    for worktree in git_worktrees {
        if worktree.path.starts_with(&phantom_dir) {
            let is_clean = get_worktree_status_with_executor(executor.clone(), &worktree.path)
                .await
                .unwrap_or(true);

            results.push(WorktreeInfo {
                name: worktree.name.clone(),
                path: worktree.path.to_string_lossy().to_string(),
                branch: worktree.branch,
                is_clean,
            });
        }
    }

    results
}

criterion_group!(arena_benches, bench_arena_batch_processing, bench_allocation_patterns);
criterion_main!(arena_benches);
