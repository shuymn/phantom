//! Example demonstrating when to use Arc/Rc vs cloning
//!
//! This example shows best practices for managing shared data in Rust,
//! helping you decide when to use smart pointers vs cloning.

use std::rc::Rc;
use std::sync::Arc;
use std::time::Instant;

/// Large data structure that's expensive to clone
#[derive(Debug, Clone)]
struct LargeData {
    items: Vec<String>,
}

impl LargeData {
    fn new(size: usize) -> Self {
        let items = (0..size).map(|i| format!("Item {}", i)).collect();
        Self { items }
    }
}

/// Lightweight data that's cheap to clone
#[derive(Debug, Clone, Copy)]
struct LightData {
    value: u64,
    flag: bool,
}

/// Configuration that might be shared across handlers
#[derive(Debug, Clone)]
struct Config {
    name: String,
    settings: Vec<String>,
}

/// Example 1: When to use Arc - Sharing large data across threads
async fn share_across_threads() {
    println!("=== Example 1: Sharing Large Data Across Threads ===");

    let large_data = Arc::new(LargeData::new(10000));
    let mut handles = vec![];

    let start = Instant::now();

    // Spawn 5 tasks that all need the same data
    for i in 0..5 {
        let data = Arc::clone(&large_data); // Cheap Arc clone
        let handle = tokio::spawn(async move {
            println!("Task {} processing {} items", i, data.items.len());
            // Simulate work
            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        });
        handles.push(handle);
    }

    // Wait for all tasks
    for handle in handles {
        handle.await.unwrap();
    }

    println!("Time with Arc: {:?}", start.elapsed());

    // Compare with cloning (DON'T DO THIS)
    let large_data = LargeData::new(10000);
    let mut handles = vec![];

    let start = Instant::now();

    for i in 0..5 {
        let data = large_data.clone(); // Expensive clone!
        let handle = tokio::spawn(async move {
            println!("Task {} processing {} items", i, data.items.len());
            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.await.unwrap();
    }

    println!("Time with Clone: {:?}\n", start.elapsed());
}

/// Example 2: When to use Rc - Single-threaded shared ownership
fn share_in_single_thread() {
    println!("=== Example 2: Rc for Single-Threaded Sharing ===");

    let config = Rc::new(Config {
        name: "MyApp".to_string(),
        settings: vec!["opt1".to_string(), "opt2".to_string()],
    });

    // Multiple components in the same thread need the config
    let component1 = Component { config: Rc::clone(&config) };
    let component2 = Component { config: Rc::clone(&config) };

    println!("Component 1 config: {}", component1.config.name);
    println!("Component 2 config: {}", component2.config.name);
    println!("Reference count: {}", Rc::strong_count(&config));
    println!();
}

struct Component {
    config: Rc<Config>,
}

/// Example 3: When cloning is fine - Small, Copy types
fn when_cloning_is_fine() {
    println!("=== Example 3: When Cloning is Fine ===");

    // Small Copy types - just clone/copy them
    let light = LightData { value: 42, flag: true };

    let light2 = light; // Copy, not clone
    let light3 = light; // This is fine!

    println!("All copies are independent: {:?}, {:?}, {:?}", light, light2, light3);

    // Small strings or collections - cloning might be OK
    let small_vec = vec![1, 2, 3, 4, 5];
    let cloned = small_vec.clone(); // This is acceptable for small data

    println!("Small vec clone is OK: {:?}", cloned);
    println!();
}

/// Example 4: Interior mutability with Arc<Mutex<T>>
async fn shared_mutable_state() {
    use std::sync::Mutex;

    println!("=== Example 4: Shared Mutable State ===");

    #[derive(Debug)]
    struct Counter {
        value: u64,
    }

    let counter = Arc::new(Mutex::new(Counter { value: 0 }));
    let mut handles = vec![];

    for i in 0..5 {
        let counter = Arc::clone(&counter);
        let handle = tokio::spawn(async move {
            let mut c = counter.lock().unwrap();
            c.value += 1;
            println!("Task {} incremented counter", i);
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.await.unwrap();
    }

    println!("Final counter value: {:?}\n", counter.lock().unwrap().value);
}

/// Example 5: Avoiding unnecessary Arc in function parameters
fn process_data_good(data: &LargeData) {
    // Good: Accept a reference when you just need to read
    println!("Processing {} items", data.items.len());
}

fn process_data_bad(data: Arc<LargeData>) {
    // Bad: Forces caller to have an Arc even if not needed
    println!("Processing {} items", data.items.len());
}

/// Example 6: Real-world pattern - Command executor sharing
mod command_pattern {
    use std::sync::Arc;

    trait CommandExecutor: Send + Sync {
        fn execute(&self, cmd: &str);
    }

    struct RealExecutor;
    impl CommandExecutor for RealExecutor {
        fn execute(&self, cmd: &str) {
            println!("Executing: {}", cmd);
        }
    }

    // Pattern 1: Arc in the function (flexibility for caller)
    async fn run_with_executor_v1(executor: Arc<dyn CommandExecutor>) {
        executor.execute("command1");
    }

    // Pattern 2: Generic parameter (better performance, no Arc required)
    async fn run_with_executor_v2<E: CommandExecutor>(executor: &E) {
        executor.execute("command2");
    }

    pub async fn demonstrate() {
        println!("=== Example 6: Command Executor Patterns ===");

        let executor = RealExecutor;

        // V1 requires Arc
        run_with_executor_v1(Arc::new(executor)).await;

        // V2 works with reference
        let executor = RealExecutor;
        run_with_executor_v2(&executor).await;

        println!();
    }
}

#[tokio::main]
async fn main() {
    println!("Smart Pointers vs Cloning: Best Practices\n");

    // Run examples
    share_across_threads().await;
    share_in_single_thread();
    when_cloning_is_fine();
    shared_mutable_state().await;

    // Function parameter examples
    println!("=== Example 5: Function Parameters ===");
    let data = LargeData::new(1000);
    process_data_good(&data); // Good: just pass reference
    process_data_bad(Arc::new(data)); // Bad: forces Arc
    println!();

    command_pattern::demonstrate().await;

    // Summary
    println!("=== Summary ===");
    println!("Use Arc when:");
    println!("  - Sharing data across threads");
    println!("  - Multiple owners need the same large data");
    println!("  - Data lifetime is complex");
    println!();
    println!("Use Rc when:");
    println!("  - Single-threaded shared ownership");
    println!("  - Building graphs or trees");
    println!();
    println!("Use Clone when:");
    println!("  - Data is small (Copy types preferred)");
    println!("  - Each owner needs independent data");
    println!("  - Cloning is infrequent");
    println!();
    println!("Avoid:");
    println!("  - Cloning large data structures repeatedly");
    println!("  - Using Arc when a reference would suffice");
    println!("  - Forcing Arc in APIs unnecessarily");
}
