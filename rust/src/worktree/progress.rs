use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::mpsc;
use tokio::time::interval;
use tracing::{debug, info};

/// Progress information for file operations
#[derive(Debug, Clone)]
pub struct ProgressInfo {
    /// Total number of files to process
    pub total_files: usize,
    /// Number of files processed so far
    pub processed_files: usize,
    /// Total size in bytes to process
    pub total_bytes: u64,
    /// Number of bytes processed so far
    pub processed_bytes: u64,
    /// Number of errors encountered
    pub errors: usize,
    /// Elapsed time since operation started
    pub elapsed: Duration,
    /// Estimated time remaining
    pub eta: Option<Duration>,
    /// Current operation (e.g., "Copying file.txt")
    pub current_operation: Option<String>,
}

/// Trait for receiving progress updates
pub trait ProgressReporter: Send + Sync {
    /// Called when progress is updated
    fn report(&self, info: &ProgressInfo);

    /// Called when operation is completed
    fn complete(&self, info: &ProgressInfo);
}

/// Simple console progress reporter
pub struct ConsoleProgressReporter {
    /// How often to report progress (in milliseconds)
    pub report_interval_ms: u64,
}

impl Default for ConsoleProgressReporter {
    fn default() -> Self {
        Self {
            report_interval_ms: 500, // Report every 500ms
        }
    }
}

impl ProgressReporter for ConsoleProgressReporter {
    fn report(&self, info: &ProgressInfo) {
        let percent = if info.total_files > 0 {
            (info.processed_files as f64 / info.total_files as f64 * 100.0) as u32
        } else {
            0
        };

        let rate = if info.elapsed.as_secs() > 0 {
            info.processed_bytes / info.elapsed.as_secs()
        } else {
            0
        };

        let eta_str = if let Some(eta) = info.eta {
            format!(" ETA: {}s", eta.as_secs())
        } else {
            String::new()
        };

        info!(
            "Progress: {}/{} files ({}%) | {:.2} MB @ {:.2} MB/s | Errors: {}{}",
            info.processed_files,
            info.total_files,
            percent,
            info.processed_bytes as f64 / 1_048_576.0,
            rate as f64 / 1_048_576.0,
            info.errors,
            eta_str
        );

        if let Some(ref op) = info.current_operation {
            debug!("Current: {}", op);
        }
    }

    fn complete(&self, info: &ProgressInfo) {
        info!(
            "Operation completed: {} files processed ({} errors) in {:.2}s",
            info.processed_files,
            info.errors,
            info.elapsed.as_secs_f64()
        );
    }
}

/// Progress tracker for file operations
pub struct ProgressTracker {
    total_files: AtomicUsize,
    processed_files: AtomicUsize,
    total_bytes: AtomicU64,
    processed_bytes: AtomicU64,
    errors: AtomicUsize,
    start_time: Instant,
    current_operation: Arc<tokio::sync::RwLock<Option<String>>>,
}

impl ProgressTracker {
    /// Create a new progress tracker
    pub fn new() -> Self {
        Self {
            total_files: AtomicUsize::new(0),
            processed_files: AtomicUsize::new(0),
            total_bytes: AtomicU64::new(0),
            processed_bytes: AtomicU64::new(0),
            errors: AtomicUsize::new(0),
            start_time: Instant::now(),
            current_operation: Arc::new(tokio::sync::RwLock::new(None)),
        }
    }
}

impl Default for ProgressTracker {
    fn default() -> Self {
        Self::new()
    }
}

impl ProgressTracker {
    /// Set the total number of files
    pub fn set_total_files(&self, total: usize) {
        self.total_files.store(total, Ordering::Relaxed);
    }

    /// Set the total size in bytes
    pub fn set_total_bytes(&self, total: u64) {
        self.total_bytes.store(total, Ordering::Relaxed);
    }

    /// Increment the processed files counter
    pub fn increment_processed_files(&self) {
        self.processed_files.fetch_add(1, Ordering::Relaxed);
    }

    /// Add to the processed bytes counter
    pub fn add_processed_bytes(&self, bytes: u64) {
        self.processed_bytes.fetch_add(bytes, Ordering::Relaxed);
    }

    /// Increment the error counter
    pub fn increment_errors(&self) {
        self.errors.fetch_add(1, Ordering::Relaxed);
    }

    /// Set the current operation
    pub async fn set_current_operation(&self, operation: Option<String>) {
        let mut current = self.current_operation.write().await;
        *current = operation;
    }

    /// Get current progress information
    pub async fn get_progress_info(&self) -> ProgressInfo {
        let total_files = self.total_files.load(Ordering::Relaxed);
        let processed_files = self.processed_files.load(Ordering::Relaxed);
        let total_bytes = self.total_bytes.load(Ordering::Relaxed);
        let processed_bytes = self.processed_bytes.load(Ordering::Relaxed);
        let errors = self.errors.load(Ordering::Relaxed);
        let elapsed = self.start_time.elapsed();

        // Calculate ETA
        let eta = if processed_files > 0 && total_files > processed_files {
            let remaining_files = total_files - processed_files;
            let rate = processed_files as f64 / elapsed.as_secs_f64();
            if rate > 0.0 {
                Some(Duration::from_secs_f64(remaining_files as f64 / rate))
            } else {
                None
            }
        } else {
            None
        };

        let current_operation = self.current_operation.read().await.clone();

        ProgressInfo {
            total_files,
            processed_files,
            total_bytes,
            processed_bytes,
            errors,
            elapsed,
            eta,
            current_operation,
        }
    }
}

/// Channel-based progress reporter that sends updates to a channel
pub struct ChannelProgressReporter {
    sender: mpsc::UnboundedSender<ProgressInfo>,
}

impl ChannelProgressReporter {
    /// Create a new channel progress reporter
    pub fn new() -> (Self, mpsc::UnboundedReceiver<ProgressInfo>) {
        let (sender, receiver) = mpsc::unbounded_channel();
        (Self { sender }, receiver)
    }
}

impl ProgressReporter for ChannelProgressReporter {
    fn report(&self, info: &ProgressInfo) {
        // Ignore send errors (receiver might have been dropped)
        let _ = self.sender.send(info.clone());
    }

    fn complete(&self, info: &ProgressInfo) {
        let _ = self.sender.send(info.clone());
    }
}

/// Start a progress reporting task
pub fn start_progress_reporter(
    tracker: Arc<ProgressTracker>,
    reporter: Arc<dyn ProgressReporter>,
    interval_ms: u64,
) -> mpsc::Sender<()> {
    let (shutdown_tx, mut shutdown_rx) = mpsc::channel(1);

    tokio::spawn(async move {
        let mut interval = interval(Duration::from_millis(interval_ms));

        loop {
            tokio::select! {
                _ = interval.tick() => {
                    let info = tracker.get_progress_info().await;
                    reporter.report(&info);
                }
                _ = shutdown_rx.recv() => {
                    let info = tracker.get_progress_info().await;
                    reporter.complete(&info);
                    break;
                }
            }
        }
    });

    shutdown_tx
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_progress_tracker() {
        let tracker = ProgressTracker::new();

        tracker.set_total_files(100);
        tracker.set_total_bytes(1_000_000);

        for _ in 0..50 {
            tracker.increment_processed_files();
            tracker.add_processed_bytes(10_000);
        }

        tracker.increment_errors();
        tracker.increment_errors();

        let info = tracker.get_progress_info().await;
        assert_eq!(info.total_files, 100);
        assert_eq!(info.processed_files, 50);
        assert_eq!(info.total_bytes, 1_000_000);
        assert_eq!(info.processed_bytes, 500_000);
        assert_eq!(info.errors, 2);
        assert!(info.elapsed.as_millis() > 0);
    }

    #[tokio::test]
    async fn test_console_progress_reporter() {
        let reporter = ConsoleProgressReporter::default();
        let info = ProgressInfo {
            total_files: 100,
            processed_files: 50,
            total_bytes: 1_000_000,
            processed_bytes: 500_000,
            errors: 2,
            elapsed: Duration::from_secs(5),
            eta: Some(Duration::from_secs(5)),
            current_operation: Some("test.txt".to_string()),
        };

        // Should not panic
        reporter.report(&info);
        reporter.complete(&info);
    }

    #[tokio::test]
    async fn test_channel_progress_reporter() {
        let (reporter, mut receiver) = ChannelProgressReporter::new();

        let info = ProgressInfo {
            total_files: 10,
            processed_files: 5,
            total_bytes: 1000,
            processed_bytes: 500,
            errors: 0,
            elapsed: Duration::from_secs(1),
            eta: Some(Duration::from_secs(1)),
            current_operation: None,
        };

        reporter.report(&info);

        let received = receiver.recv().await.unwrap();
        assert_eq!(received.total_files, 10);
        assert_eq!(received.processed_files, 5);
    }
}
