use clap::Parser;
use phantom::cli::context::ProductionContext;
use phantom::cli::{self, Commands};
use phantom::{PhantomError, Result};
use std::process;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

#[tokio::main]
async fn main() {
    // Parse CLI arguments
    let cli = cli::Cli::parse();

    // Initialize output handler based on flags
    cli::output::init_output(cli.quiet, cli.verbose, false);

    // Initialize tracing if verbose mode
    if cli.verbose {
        if let Err(e) = init_tracing() {
            eprintln!("Failed to initialize tracing: {}", e);
        }
    }

    // Create handler context
    let context = ProductionContext::default();

    // Handle commands
    let result = match cli.command {
        Commands::Create(args) => cli::handlers::create::handle(args, context.clone()).await,
        Commands::Attach(args) => cli::handlers::attach::handle(args, context.clone()).await,
        Commands::List(args) => cli::handlers::list::handle(args, context.clone()).await,
        Commands::Where(args) => cli::handlers::where_cmd::handle(args, context.clone()).await,
        Commands::Delete(args) => cli::handlers::delete::handle(args, context.clone()).await,
        Commands::Exec(args) => cli::handlers::exec::handle(args, context.clone()).await,
        Commands::Shell(args) => cli::handlers::shell::handle(args, context.clone()).await,
        Commands::Version(args) => {
            cli::handlers::version::handle(args);
            Ok(())
        }
        Commands::Completion(args) => cli::handlers::completion::handle(args),
    };

    // Handle errors
    if let Err(e) = result {
        cli::output::output().error(&e.to_string());
        let exit_code = cli::error::error_to_exit_code(&e);
        process::exit(exit_code);
    }
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