//! `pea remove` command implementation.
//!
//! Removes a dependency from the project.

use pea_core::error::PeaResult;
use super::CommandContext;

/// Execute the `pea remove` command
pub async fn execute(package: String, ctx: &CommandContext) -> PeaResult<()> {
    ctx.output.step("➖", &format!("Removing dependency {}", package));
    
    // TODO: Implement actual dependency removal
    ctx.output.warn("Dependency removal not yet implemented");
    ctx.output.info("This will be implemented when the config parser is ready");
    
    Ok(())
}