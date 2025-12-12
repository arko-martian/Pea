//! `pea add` command implementation.
//!
//! Adds a new dependency to the project.

use pea_core::error::PeaResult;
use super::CommandContext;

/// Execute the `pea add` command
pub async fn execute(package: String, dev: bool, ctx: &CommandContext) -> PeaResult<()> {
    let dep_type = if dev { "dev dependency" } else { "dependency" };
    ctx.output.step("➕", &format!("Adding {} {}", dep_type, package));
    
    // TODO: Implement actual dependency addition
    ctx.output.warn("Dependency addition not yet implemented");
    ctx.output.info("This will be implemented when the config parser is ready");
    
    Ok(())
}