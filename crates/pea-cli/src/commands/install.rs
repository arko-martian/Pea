//! `pea install` command implementation.
//!
//! Installs dependencies specified in pea.toml.

use pea_core::error::PeaResult;
use super::CommandContext;

/// Execute the `pea install` command
pub async fn execute(frozen: bool, ctx: &CommandContext) -> PeaResult<()> {
    if frozen {
        ctx.output.step("🔒", "Installing dependencies (frozen mode)");
    } else {
        ctx.output.step("📦", "Installing dependencies");
    }
    
    // TODO: Implement actual dependency installation
    ctx.output.warn("Dependency installation not yet implemented");
    ctx.output.info("This will be implemented when the resolver and registry client are ready");
    
    Ok(())
}