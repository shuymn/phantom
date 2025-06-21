# Advanced Rust Features: From Intermediate to Expert

This guide helps intermediate Rust developers master advanced language features to write more sophisticated, performant, and idiomatic code. Each section shows progression from basic to advanced patterns with practical examples.

## Table of Contents
1. [Zero-Cost Abstractions](#zero-cost-abstractions)
2. [Advanced Lifetime Management](#advanced-lifetime-management)
3. [Type System Mastery](#type-system-mastery)
4. [Performance Optimization](#performance-optimization)
5. [Advanced Error Handling](#advanced-error-handling)
6. [Async and Concurrency Patterns](#async-and-concurrency-patterns)
7. [Smart API Design](#smart-api-design)
8. [Memory Management Techniques](#memory-management-techniques)
9. [Compile-Time Programming](#compile-time-programming)
10. [Unsafe Rust: When and How](#unsafe-rust-when-and-how)

## Zero-Cost Abstractions

### Principle
Rust's zero-cost abstractions let you write high-level code that compiles to the same machine code as hand-optimized low-level code.

### From Dynamic to Static Dispatch

**Beginner approach (runtime polymorphism):**
```rust
pub struct Logger {
    writer: Box<dyn Write>,
}

impl Logger {
    pub fn log(&mut self, msg: &str) -> Result<()> {
        self.writer.write_all(msg.as_bytes())?;
        Ok(())
    }
}
```

**Advanced approach (compile-time polymorphism):**
```rust
pub struct Logger<W: Write> {
    writer: W,
}

impl<W: Write> Logger<W> {
    pub fn log(&mut self, msg: &str) -> Result<()> {
        self.writer.write_all(msg.as_bytes())?;
        Ok(())
    }
}

// Zero-cost newtype pattern for additional type safety
pub struct StdoutLogger(Logger<std::io::Stdout>);
pub struct FileLogger(Logger<std::fs::File>);
```

**Benefits:**
- No vtable lookup overhead
- Enables inlining and optimization
- Type safety at compile time
- Stack allocation possible

### Iterator Chains vs Loops

**Beginner approach:**
```rust
fn process_data(numbers: Vec<i32>) -> Vec<i32> {
    let mut result = Vec::new();
    for n in numbers {
        if n > 0 {
            result.push(n * 2);
        }
    }
    result
}
```

**Advanced approach:**
```rust
fn process_data(numbers: impl IntoIterator<Item = i32>) -> impl Iterator<Item = i32> {
    numbers.into_iter()
        .filter(|&n| n > 0)
        .map(|n| n * 2)
}

// Even more advanced: custom iterator
pub struct ProcessedData<I> {
    inner: I,
}

impl<I> Iterator for ProcessedData<I>
where
    I: Iterator<Item = i32>,
{
    type Item = i32;
    
    fn next(&mut self) -> Option<Self::Item> {
        self.inner.find_map(|n| (n > 0).then(|| n * 2))
    }
}
```

## Advanced Lifetime Management

### Beyond Basic Lifetimes

**Beginner approach:**
```rust
struct Parser {
    data: String,
}

impl Parser {
    fn parse(&self) -> Vec<&str> {
        self.data.split_whitespace().collect()
    }
}
```

**Advanced approach with explicit lifetimes:**
```rust
struct Parser<'a> {
    data: &'a str,
}

impl<'a> Parser<'a> {
    fn parse(&self) -> impl Iterator<Item = &'a str> + '_ {
        self.data.split_whitespace()
    }
    
    // Higher-ranked trait bounds (HRTB)
    fn parse_with<F, R>(&self, f: F) -> R
    where
        F: for<'b> FnOnce(&'b str) -> R,
    {
        f(self.data)
    }
}
```

### Lifetime Elision and Subtyping

```rust
// Variance and lifetime subtyping
struct Invariant<'a> {
    data: &'a mut String,  // invariant over 'a
}

struct Covariant<'a> {
    data: &'a str,  // covariant over 'a
}

// Advanced: GATs (Generic Associated Types)
trait Container {
    type Item<'a> where Self: 'a;
    
    fn get<'a>(&'a self) -> Self::Item<'a>;
}

impl Container for String {
    type Item<'a> = &'a str;
    
    fn get<'a>(&'a self) -> Self::Item<'a> {
        self.as_str()
    }
}
```

## Type System Mastery

### Phantom Types for Compile-Time State Machines

```rust
use std::marker::PhantomData;

// Type-level state machine
mod states {
    pub struct Disconnected;
    pub struct Connected;
    pub struct Authenticated;
}

pub struct Connection<State> {
    socket: TcpStream,
    _state: PhantomData<State>,
}

impl Connection<states::Disconnected> {
    pub fn connect(addr: &str) -> Result<Connection<states::Connected>> {
        let socket = TcpStream::connect(addr)?;
        Ok(Connection {
            socket,
            _state: PhantomData,
        })
    }
}

impl Connection<states::Connected> {
    pub fn authenticate(self, credentials: &str) -> Result<Connection<states::Authenticated>> {
        // ... authentication logic ...
        Ok(Connection {
            socket: self.socket,
            _state: PhantomData,
        })
    }
}

impl Connection<states::Authenticated> {
    pub fn send_data(&mut self, data: &[u8]) -> Result<()> {
        self.socket.write_all(data)
    }
}
```

### Advanced Trait Patterns

```rust
// Sealed traits pattern
mod private {
    pub trait Sealed {}
}

pub trait MyTrait: private::Sealed {
    fn method(&self);
}

impl private::Sealed for String {}
impl MyTrait for String {
    fn method(&self) {}
}

// Associated type projections
trait Database {
    type Connection: DatabaseConnection;
    type Error: std::error::Error + Send + Sync + 'static;
    
    fn connect(&self) -> Result<Self::Connection, Self::Error>;
}

// Trait aliases (when stable)
trait AsyncService = Service + Send + Sync + 'static;

// Higher-kinded types simulation
trait Functor {
    type Wrapped<T>;
    
    fn map<A, B, F>(wrapped: Self::Wrapped<A>, f: F) -> Self::Wrapped<B>
    where
        F: FnOnce(A) -> B;
}
```

## Performance Optimization

### Smart Memory Usage

```rust
use std::borrow::Cow;

// Avoid allocations with Cow
fn process_string(input: &str) -> Cow<str> {
    if input.contains("old") {
        Cow::Owned(input.replace("old", "new"))
    } else {
        Cow::Borrowed(input)
    }
}

// Small string optimization
use smallvec::SmallVec;

fn collect_args() -> SmallVec<[String; 4]> {
    // Stack allocation for <= 4 elements
    std::env::args().collect()
}

// Custom allocators (when stable)
#[global_allocator]
static ALLOC: jemallocator::Jemalloc = jemallocator::Jemalloc;
```

### Const Evaluation and Compile-Time Computation

```rust
// Const functions for compile-time computation
const fn fibonacci(n: u32) -> u32 {
    match n {
        0 => 0,
        1 => 1,
        _ => fibonacci(n - 1) + fibonacci(n - 2),
    }
}

const FIB_10: u32 = fibonacci(10);

// Const generics for zero-cost configuration
struct Buffer<const SIZE: usize> {
    data: [u8; SIZE],
}

impl<const SIZE: usize> Buffer<SIZE> {
    const fn new() -> Self {
        Self { data: [0; SIZE] }
    }
}

// Type-level integers (before const generics)
use typenum::{U16, U32};

struct TypedBuffer<N> {
    _marker: PhantomData<N>,
}
```

## Advanced Error Handling

### Error Design Patterns

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Parse error at line {line}: {message}")]
    Parse { line: usize, message: String },
    
    #[error("Configuration error")]
    Config(#[source] Box<dyn std::error::Error + Send + Sync>),
    
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

// Type-safe error handling with phantom types
struct Validated<T> {
    value: T,
}

struct Unvalidated;
struct Valid;

impl Validated<Unvalidated> {
    fn validate(self) -> Result<Validated<Valid>, ValidationError> {
        // validation logic
        Ok(Validated { value: () })
    }
}
```

### Result Combinators and Early Returns

```rust
// Advanced error propagation
fn complex_operation() -> Result<String, AppError> {
    let config = load_config()
        .map_err(|e| AppError::Config(Box::new(e)))?;
    
    let data = fetch_data(&config.url)
        .and_then(parse_response)
        .or_else(|_| fetch_fallback_data())?;
    
    process_data(data)
        .map(|result| result.to_string())
        .inspect_err(|e| eprintln!("Processing failed: {}", e))
}

// Custom Try trait implementation (when stable)
enum Option2<T> {
    Some(T),
    None,
    Unknown,
}

impl<T> std::ops::Try for Option2<T> {
    type Output = T;
    type Residual = Option2<!>;
    
    // Implementation details...
}
```

## Async and Concurrency Patterns

### Advanced Async Patterns

```rust
use futures::{stream, StreamExt};

// Custom Future implementation
pin_project! {
    pub struct Timeout<F> {
        #[pin]
        future: F,
        #[pin]
        delay: tokio::time::Sleep,
    }
}

impl<F> Future for Timeout<F>
where
    F: Future,
{
    type Output = Option<F::Output>;
    
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project();
        
        match this.future.poll(cx) {
            Poll::Ready(output) => Poll::Ready(Some(output)),
            Poll::Pending => match this.delay.poll(cx) {
                Poll::Ready(_) => Poll::Ready(None),
                Poll::Pending => Poll::Pending,
            }
        }
    }
}

// Stream processing with backpressure
async fn process_stream<S>(stream: S) -> Result<Vec<ProcessedItem>>
where
    S: Stream<Item = RawItem> + Unpin,
{
    stream
        .map(|item| async move { process_item(item).await })
        .buffer_unordered(10)  // Process up to 10 items concurrently
        .try_collect()
        .await
}

// Select! for complex control flow
async fn multiplex_operations() {
    let mut interval = tokio::time::interval(Duration::from_secs(1));
    let mut receiver = create_channel();
    
    loop {
        tokio::select! {
            _ = interval.tick() => {
                println!("Periodic task");
            }
            Some(msg) = receiver.recv() => {
                println!("Received: {}", msg);
            }
            _ = tokio::signal::ctrl_c() => {
                println!("Shutting down");
                break;
            }
        }
    }
}
```

### Lock-Free Concurrency

```rust
use std::sync::atomic::{AtomicU64, Ordering};
use crossbeam::epoch;

// Lock-free counter
struct Counter {
    value: AtomicU64,
}

impl Counter {
    fn increment(&self) -> u64 {
        self.value.fetch_add(1, Ordering::Relaxed)
    }
    
    fn get(&self) -> u64 {
        self.value.load(Ordering::Acquire)
    }
}

// Epoch-based memory reclamation
struct Node<T> {
    data: T,
    next: Atomic<Node<T>>,
}

struct Stack<T> {
    head: Atomic<Node<T>>,
}

impl<T> Stack<T> {
    fn push(&self, data: T) {
        let guard = &epoch::pin();
        let mut new_node = Owned::new(Node {
            data,
            next: Atomic::null(),
        });
        
        loop {
            let head = self.head.load(Ordering::Acquire, guard);
            new_node.next.store(head, Ordering::Relaxed);
            
            match self.head.compare_exchange(
                head,
                new_node,
                Ordering::Release,
                Ordering::Acquire,
                guard,
            ) {
                Ok(_) => break,
                Err(e) => new_node = e.new,
            }
        }
    }
}
```

## Smart API Design

### Builder Pattern with Type States

```rust
pub struct ServerBuilder<State> {
    port: u16,
    _state: PhantomData<State>,
}

pub struct NoAuth;
pub struct WithAuth { 
    provider: Box<dyn AuthProvider>,
}

impl ServerBuilder<NoAuth> {
    pub fn new() -> Self {
        ServerBuilder {
            port: 8080,
            _state: PhantomData,
        }
    }
    
    pub fn with_auth(self, provider: impl AuthProvider + 'static) -> ServerBuilder<WithAuth> {
        ServerBuilder {
            port: self.port,
            _state: PhantomData,
        }
    }
}

impl ServerBuilder<WithAuth> {
    pub fn build(self) -> Server {
        // Can only build when auth is configured
        Server::new(self.port)
    }
}
```

### Extension Traits Pattern

```rust
// Core functionality
pub trait Iterator {
    type Item;
    fn next(&mut self) -> Option<Self::Item>;
}

// Extended functionality in separate trait
pub trait IteratorExt: Iterator {
    fn collect_vec(self) -> Vec<Self::Item>
    where
        Self: Sized,
    {
        let mut vec = Vec::new();
        self.for_each(|item| vec.push(item));
        vec
    }
}

// Blanket implementation
impl<T: Iterator> IteratorExt for T {}
```

## Memory Management Techniques

### Custom Smart Pointers

```rust
use std::ops::{Deref, DerefMut};
use std::ptr::NonNull;

// Custom box with small buffer optimization
pub struct SmallBox<T, const N: usize> {
    data: SmallBoxData<T, N>,
}

enum SmallBoxData<T, const N: usize> {
    Stack([MaybeUninit<T>; N]),
    Heap(NonNull<T>),
}

impl<T, const N: usize> Deref for SmallBox<T, N> {
    type Target = T;
    
    fn deref(&self) -> &Self::Target {
        match &self.data {
            SmallBoxData::Stack(arr) => unsafe {
                &*(arr.as_ptr() as *const T)
            },
            SmallBoxData::Heap(ptr) => unsafe {
                ptr.as_ref()
            },
        }
    }
}
```

### Arena Allocation

```rust
use typed_arena::Arena;

struct TreeNode<'a> {
    value: i32,
    children: Vec<&'a TreeNode<'a>>,
}

fn build_tree<'a>(arena: &'a Arena<TreeNode<'a>>) -> &'a TreeNode<'a> {
    let root = arena.alloc(TreeNode {
        value: 1,
        children: vec![],
    });
    
    let child1 = arena.alloc(TreeNode {
        value: 2,
        children: vec![],
    });
    
    // Safe because all nodes have the same lifetime
    root.children.push(child1);
    root
}
```

## Compile-Time Programming

### Type-Level Programming

```rust
// Type-level boolean
struct True;
struct False;

trait Bool {
    type Not: Bool;
}

impl Bool for True {
    type Not = False;
}

impl Bool for False {
    type Not = True;
}

// Type-level lists
struct Nil;
struct Cons<Head, Tail>(PhantomData<(Head, Tail)>);

trait TypeList {
    type Head;
    type Tail: TypeList;
}

// Compile-time assertions
trait AssertEq<T> {}
impl<T> AssertEq<T> for T {}

fn require_same_types<T, U>()
where
    T: AssertEq<U>,
{
}
```

### Procedural Macros

```rust
// derive macro
#[proc_macro_derive(Builder)]
pub fn derive_builder(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    
    // Generate builder pattern implementation
    quote! {
        // Implementation...
    }
}

// Attribute macro for compile-time validation
#[proc_macro_attribute]
pub fn validate(args: TokenStream, input: TokenStream) -> TokenStream {
    // Parse and validate at compile time
}
```

## Unsafe Rust: When and How

### Safe Abstractions Over Unsafe Code

```rust
pub struct RawVec<T> {
    ptr: NonNull<T>,
    cap: usize,
}

impl<T> RawVec<T> {
    pub fn new() -> Self {
        let cap = if mem::size_of::<T>() == 0 { usize::MAX } else { 0 };
        RawVec {
            ptr: NonNull::dangling(),
            cap,
        }
    }
    
    pub fn grow(&mut self) {
        unsafe {
            let (new_cap, new_layout) = if self.cap == 0 {
                (1, Layout::array::<T>(1).unwrap())
            } else {
                let new_cap = 2 * self.cap;
                (new_cap, Layout::array::<T>(new_cap).unwrap())
            };
            
            let new_ptr = if self.cap == 0 {
                alloc::alloc(new_layout)
            } else {
                let old_layout = Layout::array::<T>(self.cap).unwrap();
                alloc::realloc(self.ptr.as_ptr() as *mut u8, old_layout, new_layout.size())
            };
            
            self.ptr = NonNull::new(new_ptr as *mut T)
                .expect("Failed to allocate memory");
            self.cap = new_cap;
        }
    }
}

// SAFETY: RawVec properly manages memory
unsafe impl<T: Send> Send for RawVec<T> {}
unsafe impl<T: Sync> Sync for RawVec<T> {}
```

## Best Practices and Guidelines

### When to Use Each Pattern

1. **Use Zero-Cost Abstractions when:**
   - Performance is critical
   - The types are known at compile time
   - You want to enable compiler optimizations

2. **Use Advanced Lifetimes when:**
   - Working with complex borrowing patterns
   - Building zero-copy APIs
   - Implementing custom collections

3. **Use Type-State Pattern when:**
   - Enforcing API usage at compile time
   - Building state machines
   - Preventing invalid states

4. **Use Unsafe when:**
   - Implementing fundamental abstractions
   - Interfacing with C libraries
   - Optimizing hot paths (with benchmarks)

### Common Pitfalls to Avoid

1. **Over-engineering:** Don't use advanced features just because you can
2. **Premature optimization:** Profile before optimizing
3. **Unsafe without invariants:** Always document safety requirements
4. **Complex lifetimes:** Sometimes cloning is clearer and fast enough
5. **Macro abuse:** Prefer regular functions when possible

### Learning Path

1. **Foundation:** Master ownership, borrowing, and basic traits
2. **Intermediate:** Learn async/await, error handling, and testing
3. **Advanced:** Study type system, performance, and unsafe
4. **Expert:** Contribute to compiler, write procedural macros, design APIs

### Resources for Continued Learning

- The Rustonomicon (Advanced Rust Programming)
- Rust for Rustaceans by Jon Gjengset
- Rust Performance Book
- Rust Async Book
- Rust API Guidelines

Remember: Advanced features should make code better, not just different. Always prioritize clarity and maintainability while leveraging Rust's powerful type system and zero-cost abstractions.