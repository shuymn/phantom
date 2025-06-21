/// Example demonstrating custom smart pointers
///
/// This example shows how to use SmallBox and SmallVec for optimized
/// memory allocation patterns.
use phantom::core::smart_pointers::{SmallBox, SmallVec};
use std::mem::size_of;

fn main() {
    println!("=== Custom Smart Pointers Example ===\n");

    // Example 1: SmallBox with inline storage
    example_small_box_inline();

    // Example 2: SmallBox with heap allocation
    example_small_box_heap();

    // Example 3: SmallVec usage
    example_small_vec();

    // Example 4: Performance comparison
    example_performance_comparison();

    // Example 5: Real-world use case
    example_real_world_usage();
}

fn example_small_box_inline() {
    println!("1. SmallBox with inline storage:");

    // Small values are stored inline
    let small_value = SmallBox::<u32, 64>::new(42);
    println!("  SmallBox<u32>: value = {}, inline = {}", *small_value, small_value.is_inline());

    // Small strings can also be inline
    let small_string = SmallBox::<String, 64>::new("Hello, world!".to_string());
    println!(
        "  SmallBox<String>: value = '{}', inline = {}",
        *small_string,
        small_string.is_inline()
    );

    // The size of SmallBox is fixed regardless of inline/heap storage
    println!("  Size of SmallBox<String, 64>: {} bytes", size_of::<SmallBox<String, 64>>());
    println!();
}

fn example_small_box_heap() {
    println!("2. SmallBox with heap allocation:");

    // Large values are stored on heap
    let large_array = [0u8; 256];
    let large_value = SmallBox::<[u8; 256], 64>::new(large_array);
    println!(
        "  SmallBox<[u8; 256]>: size = {}, inline = {}",
        large_value.len(),
        large_value.is_inline()
    );

    // But the SmallBox itself is still small
    println!("  Size of SmallBox<[u8; 256], 64>: {} bytes", size_of::<SmallBox<[u8; 256], 64>>());
    println!();
}

fn example_small_vec() {
    println!("3. SmallVec usage:");

    // Create a SmallVec with inline capacity of 4
    let mut commands = SmallVec::<String, 4>::new();

    // Add some items (stays inline)
    commands.push("git".to_string());
    commands.push("status".to_string());
    commands.push("--porcelain".to_string());

    println!("  Commands: {:?}, inline = {}", commands.as_slice(), commands.is_inline());

    // Add more items (might spill to heap)
    commands.push("--branch".to_string());
    commands.push("--ahead-behind".to_string());

    println!(
        "  After adding more: len = {}, capacity = {}, inline = {}",
        commands.len(),
        commands.capacity(),
        commands.is_inline()
    );
    println!();
}

fn example_performance_comparison() {
    println!("4. Performance comparison:");

    use std::time::Instant;

    const ITERATIONS: usize = 100_000;

    // Benchmark Box allocations
    let start = Instant::now();
    for _ in 0..ITERATIONS {
        let _ = Box::new(42u64);
    }
    let box_time = start.elapsed();

    // Benchmark SmallBox allocations (inline)
    let start = Instant::now();
    for _ in 0..ITERATIONS {
        let _ = SmallBox::<u64, 64>::new(42u64);
    }
    let small_box_time = start.elapsed();

    println!("  Box allocations: {:?}", box_time);
    println!("  SmallBox allocations (inline): {:?}", small_box_time);
    println!(
        "  SmallBox is {:.2}x faster",
        box_time.as_nanos() as f64 / small_box_time.as_nanos() as f64
    );
    println!();
}

fn example_real_world_usage() {
    println!("5. Real-world use case - Command arguments:");

    // Common git commands typically have few arguments
    enum GitCommand {
        Status {
            args: SmallVec<String, 4>,
        },
        Commit {
            message: SmallBox<String, 64>,
            args: SmallVec<String, 8>,
        },
        WorktreeAdd {
            path: String,
            branch: Option<SmallBox<String, 32>>,
            options: SmallVec<String, 2>,
        },
    }

    // Most git commands have few arguments - perfect for SmallVec
    let status_cmd = GitCommand::Status {
        args: {
            let mut args = SmallVec::new();
            args.push("--porcelain".to_string());
            args.push("--branch".to_string());
            args
        },
    };

    // Commit messages are usually small - good for SmallBox
    let commit_cmd = GitCommand::Commit {
        message: SmallBox::new("fix: resolve merge conflict".to_string()),
        args: {
            let mut args = SmallVec::new();
            args.push("-m".to_string());
            args.push("fix: resolve merge conflict".to_string());
            args
        },
    };

    match status_cmd {
        GitCommand::Status { args } => {
            println!("  Status command with {} args (inline: {})", args.len(), args.is_inline());
        }
        _ => {}
    }

    match commit_cmd {
        GitCommand::Commit { message, args } => {
            println!(
                "  Commit: message = '{}' (inline: {}), args = {} (inline: {})",
                *message,
                message.is_inline(),
                args.len(),
                args.is_inline()
            );
        }
        _ => {}
    }

    println!("\n  Benefits:");
    println!("  - Reduced heap allocations for common cases");
    println!("  - Better cache locality");
    println!("  - Lower memory fragmentation");
    println!("  - Predictable performance");
}
