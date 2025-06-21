/// Example showing how SmallBox can optimize CommandOutput
///
/// This demonstrates a potential optimization for command outputs
/// where most outputs are small strings.
use phantom::core::command_executor::CommandOutput;
use phantom::core::smart_pointers::SmallBox;
use std::borrow::Cow;

/// Alternative CommandOutput using SmallBox for common cases
#[derive(Debug, Clone)]
pub struct OptimizedCommandOutput {
    pub stdout: SmallBox<String, 128>, // Most command outputs fit in 128 bytes
    pub stderr: SmallBox<String, 64>,  // Error messages are usually smaller
    pub exit_code: i32,
}

impl OptimizedCommandOutput {
    /// Create a new output with owned strings
    pub fn new(stdout: String, stderr: String, exit_code: i32) -> Self {
        Self { stdout: SmallBox::new(stdout), stderr: SmallBox::new(stderr), exit_code }
    }

    /// Create a successful output with stdout only
    pub fn success(stdout: String) -> Self {
        Self::new(stdout, String::new(), 0)
    }

    /// Create a failed output with stderr
    pub fn failure(stderr: String, exit_code: i32) -> Self {
        Self::new(String::new(), stderr, exit_code)
    }
}

fn main() {
    println!("=== SmallBox Optimization Example ===\n");

    // Compare memory layout
    compare_memory_layout();

    // Benchmark common patterns
    benchmark_common_patterns();

    // Show real-world git command outputs
    real_world_examples();
}

fn compare_memory_layout() {
    use std::mem::size_of;

    println!("Memory Layout Comparison:");
    println!("  Original CommandOutput size: {} bytes", size_of::<CommandOutput>());
    println!("  Optimized CommandOutput size: {} bytes", size_of::<OptimizedCommandOutput>());
    println!("  String size: {} bytes", size_of::<String>());
    println!("  SmallBox<String, 128> size: {} bytes", size_of::<SmallBox<String, 128>>());
    println!();
}

fn benchmark_common_patterns() {
    use std::time::Instant;

    println!("Benchmark: Creating 10,000 command outputs");

    const ITERATIONS: usize = 10_000;

    // Benchmark original approach
    let start = Instant::now();
    for _ in 0..ITERATIONS {
        let _output = CommandOutput {
            stdout: Cow::Owned("HEAD abc123".to_string()),
            stderr: Cow::Owned(String::new()),
            exit_code: 0,
        };
    }
    let original_time = start.elapsed();

    // Benchmark optimized approach
    let start = Instant::now();
    for _ in 0..ITERATIONS {
        let _output = OptimizedCommandOutput::success("HEAD abc123".to_string());
    }
    let optimized_time = start.elapsed();

    println!("  Original approach: {:?}", original_time);
    println!("  Optimized approach: {:?}", optimized_time);
    println!(
        "  Performance ratio: {:.2}x",
        original_time.as_nanos() as f64 / optimized_time.as_nanos() as f64
    );
    println!();
}

fn real_world_examples() {
    println!("Real-world Git Command Examples:");

    // Common git commands and their typical outputs
    let examples = vec![
        ("git rev-parse HEAD", "abc123def456", "", 0),
        ("git branch --show-current", "main", "", 0),
        ("git status --porcelain", "", "", 0), // Clean repo
        ("git diff --name-only", "src/main.rs\nsrc/lib.rs", "", 0),
        ("git worktree list", "/repo/.git/worktrees/feature", "", 0),
        ("git invalid", "", "git: 'invalid' is not a git command", 1),
    ];

    let mut inline_count = 0;
    let mut heap_count = 0;

    for (cmd, stdout, stderr, code) in examples {
        let output = OptimizedCommandOutput::new(stdout.to_string(), stderr.to_string(), code);

        let stdout_inline = output.stdout.is_inline();
        let stderr_inline = output.stderr.is_inline();

        if stdout_inline {
            inline_count += 1;
        } else {
            heap_count += 1;
        }
        if stderr_inline {
            inline_count += 1;
        } else {
            heap_count += 1;
        }

        println!("  {}: stdout inline={}, stderr inline={}", cmd, stdout_inline, stderr_inline);
    }

    println!("\nSummary:");
    println!(
        "  Inline allocations: {} ({:.1}%)",
        inline_count,
        inline_count as f64 / (inline_count + heap_count) as f64 * 100.0
    );
    println!(
        "  Heap allocations: {} ({:.1}%)",
        heap_count,
        heap_count as f64 / (inline_count + heap_count) as f64 * 100.0
    );
    println!("\nBenefits:");
    println!("  - Most git command outputs fit inline");
    println!("  - Reduced heap fragmentation");
    println!("  - Better cache locality");
    println!("  - Predictable performance for common cases");
}
