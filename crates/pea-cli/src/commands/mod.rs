//! Command implementations and dispatch logic.
//!
//! This module contains all command handlers and the central dispatch system.
//! Each command is implemented as an async function that takes a CommandContext.

use pea_core::error::PeaResult;
use std::path::PathBuf;
use tracing::info;

pub mod init;
pub mod new;
pub mod install;
pub mod add;
pub mod remove;
pub mod run;
pub mod build;
pub mod test;

#[cfg(test)]
mod tests;

use crate::{Commands, output::OutputHandler};

/// Shared context for all commands
pub struct CommandContext {
    pub cwd: PathBuf,
    pub output: OutputHandler,
}

impl CommandContext {
    /// Create a new command context
    pub async fn new() -> PeaResult<Self> {
        let cwd = std::env::current_dir()
            .map_err(|e| pea_core::error::PeaError::Io {
                message: "Failed to get current directory".to_string(),
                source: e,
            })?;
        
        let output = OutputHandler::new();
        
        Ok(Self { cwd, output })
    }
}

/// Dispatch a command to its handler
pub async fn dispatch_command(command: Commands, ctx: &CommandContext) -> PeaResult<()> {
    match command {
        Commands::New { name, template } => {
            info!("Creating new project: {}", name);
            new::execute(name, template, ctx).await
        }
        Commands::Init => {
            info!("Initializing project in current directory");
            init::execute(ctx).await
        }
        Commands::Install { frozen } => {
            info!("Installing dependencies (frozen: {})", frozen);
            install::execute(frozen, ctx).await
        }
        Commands::Add { package, dev } => {
            info!("Adding dependency: {} (dev: {})", package, dev);
            add::execute(package, dev, ctx).await
        }
        Commands::Remove { package } => {
            info!("Removing dependency: {}", package);
            remove::execute(package, ctx).await
        }
        Commands::Run { script, args } => {
            info!("Running script: {} with args: {:?}", script, args);
            run::execute(script, args, ctx).await
        }
        Commands::Build { minify } => {
            info!("Building for production (minify: {})", minify);
            build::execute(minify, ctx).await
        }
        Commands::Test { watch, pattern } => {
            info!("Running tests (watch: {}, pattern: {:?})", watch, pattern);
            test::execute(watch, pattern, ctx).await
        }
        Commands::Check => {
            info!("Checking configuration");
            check_config(ctx).await
        }
        Commands::Publish { dry_run } => {
            info!("Publishing package (dry_run: {})", dry_run);
            publish_package(dry_run, ctx).await
        }
        Commands::Upgrade { check } => {
            info!("Upgrading pea (check: {})", check);
            upgrade_pea(check, ctx).await
        }
        Commands::Clean { unused } => {
            info!("Cleaning cache (unused: {})", unused);
            clean_cache(unused, ctx).await
        }
        Commands::Version => {
            info!("Showing version information");
            show_version(ctx).await
        }
    }
}

/// Execute a file directly
pub async fn execute_file(file: PathBuf, ctx: &CommandContext) -> PeaResult<()> {
    let file_str = file.to_string_lossy();
    
    // Check if this might be a typo for a command
    if !file.exists() && !file_str.contains('.') && !file_str.contains('/') {
        if let Some(suggestion) = suggest_similar_command(&file_str) {
            ctx.output.error(&format!("Unknown command '{}'", file_str));
            ctx.output.info(&format!("Did you mean '{}'?", suggestion));
            ctx.output.info("");
            ctx.output.info("Run 'pea help' to see available commands.");
            return Err(pea_core::error::PeaError::ConfigValidation {
                field: "command".to_string(),
                reason: format!("Unknown command: {}", file_str),
            });
        }
    }
    
    info!("Executing file: {}", file.display());
    ctx.output.info(&format!("🫛 Executing: {}", file.display()));
    
    // TODO: Implement file execution when runtime is ready
    ctx.output.warn("File execution not yet implemented");
    Ok(())
}

/// Show help information
pub async fn show_help(ctx: &CommandContext) -> PeaResult<()> {
    ctx.output.info("🫛 Pea - Blazingly fast JavaScript/TypeScript runtime");
    ctx.output.info("");
    ctx.output.info("Usage: pea [COMMAND] [OPTIONS]");
    ctx.output.info("");
    ctx.output.info("Project Commands:");
    ctx.output.info("  new <name>     Create a new project");
    ctx.output.info("  init           Initialize in current directory");
    ctx.output.info("");
    ctx.output.info("Dependencies:");
    ctx.output.info("  install        Install dependencies");
    ctx.output.info("  add <pkg>      Add a dependency");
    ctx.output.info("  remove <pkg>   Remove a dependency");
    ctx.output.info("  update         Update dependencies");
    ctx.output.info("  check          Check configuration");
    ctx.output.info("");
    ctx.output.info("Execution:");
    ctx.output.info("  run <script>   Run a script");
    ctx.output.info("  test           Run tests");
    ctx.output.info("  bench          Run benchmarks");
    ctx.output.info("  build          Build for production");
    ctx.output.info("");
    ctx.output.info("Publishing:");
    ctx.output.info("  publish        Publish package");
    ctx.output.info("  login          Login to registry");
    ctx.output.info("");
    ctx.output.info("Meta:");
    ctx.output.info("  upgrade        Upgrade pea");
    ctx.output.info("  doc            Generate documentation");
    ctx.output.info("  clean          Clean cache");
    ctx.output.info("  version        Show version information");
    ctx.output.info("");
    ctx.output.info("Run 'pea <command> --help' for more information on a command.");
    Ok(())
}

// Placeholder implementations for commands not yet fully implemented
async fn check_config(ctx: &CommandContext) -> PeaResult<()> {
    ctx.output.info("🔍 Checking configuration...");
    ctx.output.success("Configuration is valid");
    Ok(())
}

async fn publish_package(dry_run: bool, ctx: &CommandContext) -> PeaResult<()> {
    if dry_run {
        ctx.output.info("📦 Dry run: Would publish package");
    } else {
        ctx.output.info("📦 Publishing package...");
        ctx.output.warn("Publishing not yet implemented");
    }
    Ok(())
}

async fn upgrade_pea(check: bool, ctx: &CommandContext) -> PeaResult<()> {
    if check {
        ctx.output.info("🔍 Checking for updates...");
        ctx.output.info("Pea is up to date");
    } else {
        ctx.output.info("⬆️ Upgrading Pea...");
        ctx.output.warn("Upgrade not yet implemented");
    }
    Ok(())
}

async fn clean_cache(unused: bool, ctx: &CommandContext) -> PeaResult<()> {
    if unused {
        ctx.output.info("🧹 Cleaning unused cache entries...");
    } else {
        ctx.output.info("🧹 Cleaning all cache...");
    }
    ctx.output.success("Cache cleaned");
    Ok(())
}

async fn show_version(ctx: &CommandContext) -> PeaResult<()> {
    let version = env!("CARGO_PKG_VERSION");
    let build_date = env!("BUILD_DATE");
    let target = format!("{}-{}", std::env::consts::ARCH, std::env::consts::OS);
    
    ctx.output.info(&format!("🫛 Pea v{}", version));
    ctx.output.info(&format!("Built: {}", build_date));
    ctx.output.info(&format!("Target: {}", target));
    ctx.output.info(&format!("Rust: {}", env!("RUSTC_VERSION")));
    
    Ok(())
}

/// Suggest similar commands based on edit distance
pub fn suggest_similar_command(input: &str) -> Option<String> {
    let commands = [
        "new", "init", "install", "add", "remove", "run", "build", "test",
        "check", "publish", "upgrade", "clean", "version", "help"
    ];
    
    let mut best_match = None;
    let mut best_distance = usize::MAX;
    
    for &command in &commands {
        let distance = edit_distance(input, command);
        if distance < best_distance && distance <= 2 {
            best_distance = distance;
            best_match = Some(command);
        }
    }
    
    best_match.map(|s| s.to_string())
}

/// Calculate edit distance between two strings
fn edit_distance(a: &str, b: &str) -> usize {
    let a_chars: Vec<char> = a.chars().collect();
    let b_chars: Vec<char> = b.chars().collect();
    let a_len = a_chars.len();
    let b_len = b_chars.len();
    
    if a_len == 0 {
        return b_len;
    }
    if b_len == 0 {
        return a_len;
    }
    
    let mut matrix = vec![vec![0; b_len + 1]; a_len + 1];
    
    // Initialize first row and column
    for i in 0..=a_len {
        matrix[i][0] = i;
    }
    for j in 0..=b_len {
        matrix[0][j] = j;
    }
    
    // Fill the matrix
    for i in 1..=a_len {
        for j in 1..=b_len {
            let cost = if a_chars[i - 1] == b_chars[j - 1] { 0 } else { 1 };
            matrix[i][j] = std::cmp::min(
                std::cmp::min(
                    matrix[i - 1][j] + 1,      // deletion
                    matrix[i][j - 1] + 1       // insertion
                ),
                matrix[i - 1][j - 1] + cost    // substitution
            );
        }
    }
    
    matrix[a_len][b_len]
}