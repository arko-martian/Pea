//! `pea new` command implementation.
//!
//! Creates a new Pea project with the specified name and optional template.

use pea_core::error::{PeaError, PeaResult};
use std::fs;
use std::path::Path;
use super::CommandContext;

/// Execute the `pea new` command
pub async fn execute(name: String, template: Option<String>, ctx: &CommandContext) -> PeaResult<()> {
    validate_project_name(&name)?;
    
    let project_path = ctx.cwd.join(&name);
    
    // Check if directory already exists
    if project_path.exists() {
        return Err(PeaError::ConfigValidation {
            field: "project_name".to_string(),
            reason: format!("Directory '{}' already exists", name),
        });
    }
    
    ctx.output.step("📁", &format!("Creating project directory: {}", name));
    create_project_structure(&project_path, &name, template.as_deref())?;
    
    ctx.output.success(&format!("Created new project: {}", name));
    ctx.output.info("");
    ctx.output.info("Next steps:");
    ctx.output.info(&format!("  cd {}", name));
    ctx.output.info("  pea install");
    ctx.output.info("  pea run dev");
    
    Ok(())
}

/// Validate project name follows naming conventions
fn validate_project_name(name: &str) -> PeaResult<()> {
    if name.is_empty() {
        return Err(PeaError::ConfigValidation {
            field: "project_name".to_string(),
            reason: "Project name cannot be empty".to_string(),
        });
    }
    
    if !name.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_') {
        return Err(PeaError::ConfigValidation {
            field: "project_name".to_string(),
            reason: "Project name can only contain alphanumeric characters, hyphens, and underscores".to_string(),
        });
    }
    
    if name.starts_with('-') || name.ends_with('-') {
        return Err(PeaError::ConfigValidation {
            field: "project_name".to_string(),
            reason: "Project name cannot start or end with a hyphen".to_string(),
        });
    }
    
    Ok(())
}

/// Create the project directory structure
fn create_project_structure(project_path: &Path, name: &str, template: Option<&str>) -> PeaResult<()> {
    // Create main directory
    fs::create_dir_all(project_path)
        .map_err(|e| PeaError::Io {
            message: format!("Failed to create project directory: {}", project_path.display()),
            source: e,
        })?;
    
    // Create src directory
    let src_dir = project_path.join("src");
    fs::create_dir_all(&src_dir)
        .map_err(|e| PeaError::Io {
            message: format!("Failed to create src directory: {}", src_dir.display()),
            source: e,
        })?;
    
    // Create pea.toml
    create_pea_toml(project_path, name)?;
    
    // Create main entry point
    create_main_file(&src_dir, template)?;
    
    // Create GUIDEMAP.md
    create_guidemap(project_path, name)?;
    
    Ok(())
}

/// Create pea.toml configuration file
fn create_pea_toml(project_path: &Path, name: &str) -> PeaResult<()> {
    let content = format!(
        r#"[package]
name = "{}"
version = "0.1.0"
description = "A new Pea project"
main = "src/index.ts"

[dependencies]

[dev-dependencies]

[scripts]
dev = "pea run src/index.ts"
build = "pea build"
test = "pea test"
"#,
        name
    );
    
    let toml_path = project_path.join("pea.toml");
    fs::write(&toml_path, content)
        .map_err(|e| PeaError::Io {
            message: format!("Failed to create pea.toml: {}", toml_path.display()),
            source: e,
        })?;
    
    Ok(())
}

/// Create main entry file
fn create_main_file(src_dir: &Path, template: Option<&str>) -> PeaResult<()> {
    let (filename, content) = match template {
        Some("library") => ("index.ts", create_library_template()),
        Some("cli") => ("index.ts", create_cli_template()),
        _ => ("index.ts", create_default_template()),
    };
    
    let file_path = src_dir.join(filename);
    fs::write(&file_path, content)
        .map_err(|e| PeaError::Io {
            message: format!("Failed to create {}", file_path.display()),
            source: e,
        })?;
    
    Ok(())
}

fn create_default_template() -> &'static str {
    r#"// Welcome to your new Pea project! 🫛
console.log("Hello from Pea! 🚀");

// This is a simple TypeScript file that demonstrates Pea's capabilities
function greet(name: string): string {
    return `Hello, ${name}! Welcome to the future of JavaScript.`;
}

console.log(greet("World"));
"#
}

fn create_library_template() -> &'static str {
    r#"// Pea Library Template 📚

/**
 * A simple utility library built with Pea
 */
export class PeaLibrary {
    private version = "0.1.0";
    
    /**
     * Get the library version
     */
    getVersion(): string {
        return this.version;
    }
    
    /**
     * A sample utility function
     */
    capitalize(text: string): string {
        return text.charAt(0).toUpperCase() + text.slice(1);
    }
}

export default PeaLibrary;
"#
}

fn create_cli_template() -> &'static str {
    r#"#!/usr/bin/env pea
// Pea CLI Template 🛠️

import { parseArgs } from "util";

const { values, positionals } = parseArgs({
    args: process.argv.slice(2),
    options: {
        help: { type: "boolean", short: "h" },
        version: { type: "boolean", short: "v" },
    },
    allowPositionals: true,
});

if (values.help) {
    console.log("Usage: my-cli [options] <command>");
    console.log("Options:");
    console.log("  -h, --help     Show help");
    console.log("  -v, --version  Show version");
    process.exit(0);
}

if (values.version) {
    console.log("1.0.0");
    process.exit(0);
}

const command = positionals[0];
if (!command) {
    console.error("Error: No command specified");
    process.exit(1);
}

console.log(`Executing command: ${command}`);
"#
}

fn create_guidemap(project_path: &Path, name: &str) -> PeaResult<()> {
    let content = format!(
        r#"# {} Project Guide

## Purpose
A new Pea project created with `pea new`.

## Structure
- `src/` - Source code directory
- `src/index.ts` - Main entry point
- `pea.toml` - Project configuration
- `GUIDEMAP.md` - This file

## Getting Started
1. Install dependencies: `pea install`
2. Run in development: `pea run dev`
3. Build for production: `pea build`
4. Run tests: `pea test`

## Scripts
- `dev` - Run the project in development mode
- `build` - Build for production
- `test` - Run tests

## Next Steps
- Add dependencies with `pea add <package>`
- Explore the Pea documentation
- Start building amazing things! 🚀
"#,
        name
    );
    
    let guidemap_path = project_path.join("GUIDEMAP.md");
    fs::write(&guidemap_path, content)
        .map_err(|e| PeaError::Io {
            message: format!("Failed to create GUIDEMAP.md: {}", guidemap_path.display()),
            source: e,
        })?;
    
    Ok(())
}