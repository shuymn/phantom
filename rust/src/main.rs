use phantom::{PhantomError, Result};
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    init_tracing()?;

    info!("Starting phantom...");

    // TODO: Implement CLI parsing and command handling
    println!("Phantom - Ephemeral Git worktrees made easy");

    Ok(())
}

/// Initialize the tracing subscriber with environment filter
fn init_tracing() -> Result<()> {
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info"))
        .add_directive("phantom=debug".parse().unwrap());

    let fmt_layer = tracing_subscriber::fmt::layer()
        .with_target(false)
        .with_thread_ids(false)
        .with_thread_names(false);

    tracing_subscriber::registry()
        .with(filter)
        .with(fmt_layer)
        .try_init()
        .map_err(|e| PhantomError::Config(format!("Failed to initialize tracing: {}", e)))?;

    Ok(())
}
