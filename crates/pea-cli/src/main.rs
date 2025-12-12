//! # pea-cli
//!
//! Blazingly fast JavaScript/TypeScript runtime and package manager CLI.
//! 
//! This is the main entry point for the Pea CLI tool. It handles command parsing,
//! sets up logging and error handling, and dispatches to the appropriate command handlers.

use clap::{Parser, Subcommand};
use pea_core::error::PeaResult;
use std::path::PathBuf;
use tracing::{info, error};

mod commands;
mod output;

use commands::CommandContext;

/// Blazingly fast JavaScript/TypeScript runtime and package manager
#[derive(Parser)]
#[command(name = "pea", version, about = "Blazingly fast JS/TS runtime")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
    
    /// File to execute directly
    #[arg(value_name = "FILE")]
    pub file: Option<PathBuf>,
    
    /// Enable verbose output
    #[arg(short, long, global = true)]
    pub verbose: bool,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Create a new project
    New { 
        name: String, 
        #[arg(long)] 
        template: Option<String> 
    },
    /// Initialize in current directory
    Init,
    /// Install dependencies
    Install { 
        #[arg(long)] 
        frozen: bool 
    },
    /// Add a dependency
    Add { 
        package: String, 
        #[arg(short = 'D')] 
        dev: bool 
    },
    /// Remove a dependency
    Remove { 
        package: String 
    },
    /// Run a script
    Run { 
        script: String, 
        #[arg(last = true)] 
        args: Vec<String> 
    },
    /// Build for production
    Build { 
        #[arg(long)] 
        minify: bool 
    },
    /// Run tests
    Test { 
        #[arg(long)] 
        watch: bool, 
        pattern: Option<String> 
    },
    /// Check configuration
    Check,
    /// Publish package
    Publish { 
        #[arg(long)] 
        dry_run: bool 
    },
    /// Upgrade pea
    Upgrade { 
        #[arg(long)] 
        check: bool 
    },
    /// Clean cache
    Clean { 
        #[arg(long)] 
        unused: bool 
    },
    /// Show version information
    Version,
}

fn main() -> PeaResult<()> {
    let cli = Cli::parse();
    
    setup_logging(cli.verbose);
    setup_panic_handler();
    
    info!("Starting Pea CLI v{}", env!("CARGO_PKG_VERSION"));
    
    run_cli(cli)
}

fn run_cli(cli: Cli) -> PeaResult<()> {
    // Create Tokio runtime for async operations
    let rt = tokio::runtime::Runtime::new()
        .map_err(|e| pea_core::error::PeaError::Io { 
            message: "Failed to create async runtime".to_string(), 
            source: e 
        })?;
    
    rt.block_on(async {
        let ctx = CommandContext::new().await?;
        
        match cli.command {
            Some(command) => {
                commands::dispatch_command(command, &ctx).await
            }
            None => {
                if let Some(file) = cli.file {
                    commands::execute_file(file, &ctx).await
                } else {
                    commands::show_help(&ctx).await
                }
            }
        }
    })
}

fn setup_logging(verbose: bool) {
    let level = if verbose { "debug" } else { "info" };
    
    tracing_subscriber::fmt()
        .with_env_filter(format!("pea={},pea_core={}", level, level))
        .with_target(false)
        .init();
}

fn setup_panic_handler() {
    std::panic::set_hook(Box::new(|panic_info| {
        error!("Pea encountered an unexpected error: {}", panic_info);
        eprintln!("🫛 Pea crashed! This is a bug.");
        eprintln!("Please report this at: https://github.com/pea-lang/pea/issues");
        eprintln!("Error: {}", panic_info);
    }));
}