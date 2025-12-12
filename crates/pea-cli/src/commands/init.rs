//! `pea init` command implementation.
//!
//! Initializes a Pea project in the current directory, optionally importing
//! from an existing package.json file.

use pea_core::error::{PeaError, PeaResult};
use std::fs;
use std::path::Path;
use super::CommandContext;

/// Execute the `pea init` command
pub async fn execute(ctx: &CommandContext) -> PeaResult<()> {
    let pea_toml_path = ctx.cwd.join("pea.toml");
    let package_json_path = ctx.cwd.join("package.json");
    
    // Check if pea.toml already exists
    if pea_toml_path.exists() {
        ctx.output.info("pea.toml already exists, skipping initialization");
        return Ok(());
    }
    
    ctx.output.step("🫛", "Initializing Pea project in current directory");
    
    // Try to import from package.json if it exists
    if package_json_path.exists() {
        ctx.output.info("Found package.json, importing configuration...");
        import_from_package_json(&package_json_path, &pea_toml_path, ctx).await?;
    } else {
        create_default_pea_toml(&pea_toml_path, ctx).await?;
    }
    
    // Create src directory if it doesn't exist
    let src_dir = ctx.cwd.join("src");
    if !src_dir.exists() {
        ctx.output.step("📁", "Creating src directory");
        fs::create_dir_all(&src_dir)
            .map_err(|e| PeaError::Io {
                message: format!("Failed to create src directory: {}", src_dir.display()),
                source: e,
            })?;
        
        // Create a basic index.ts if it doesn't exist
        let index_path = src_dir.join("index.ts");
        if !index_path.exists() {
            create_basic_index(&index_path)?;
        }
    }
    
    // Create GUIDEMAP.md if it doesn't exist
    let guidemap_path = ctx.cwd.join("GUIDEMAP.md");
    if !guidemap_path.exists() {
        create_project_guidemap(&guidemap_path)?;
    }
    
    ctx.output.success("Initialized Pea project");
    ctx.output.info("");
    ctx.output.info("Next steps:");
    ctx.output.info("  pea install");
    ctx.output.info("  pea run dev");
    
    Ok(())
}

/// Import configuration from package.json
async fn import_from_package_json(
    package_json_path: &Path,
    pea_toml_path: &Path,
    ctx: &CommandContext,
) -> PeaResult<()> {
    let package_json_content = fs::read_to_string(package_json_path)
        .map_err(|e| PeaError::Io {
            message: format!("Failed to read package.json: {}", package_json_path.display()),
            source: e,
        })?;
    
    let package_json: serde_json::Value = serde_json::from_str(&package_json_content)
        .map_err(|e| PeaError::JsonParse {
            message: format!("Invalid package.json: {}", e),
        })?;
    
    let name = package_json["name"]
        .as_str()
        .unwrap_or("my-pea-project");
    
    let version = package_json["version"]
        .as_str()
        .unwrap_or("0.1.0");
    
    let description = package_json["description"]
        .as_str()
        .map(|s| format!("description = \"{}\"", s))
        .unwrap_or_default();
    
    let main = package_json["main"]
        .as_str()
        .unwrap_or("src/index.ts");
    
    // Extract dependencies
    let mut dependencies = Vec::new();
    if let Some(deps) = package_json["dependencies"].as_object() {
        for (name, version) in deps {
            if let Some(version_str) = version.as_str() {
                dependencies.push(format!("{} = \"{}\"", name, version_str));
            }
        }
    }
    
    let mut dev_dependencies = Vec::new();
    if let Some(dev_deps) = package_json["devDependencies"].as_object() {
        for (name, version) in dev_deps {
            if let Some(version_str) = version.as_str() {
                dev_dependencies.push(format!("{} = \"{}\"", name, version_str));
            }
        }
    }
    
    // Extract scripts
    let mut scripts = Vec::new();
    if let Some(script_obj) = package_json["scripts"].as_object() {
        for (name, command) in script_obj {
            if let Some(command_str) = command.as_str() {
                scripts.push(format!("{} = \"{}\"", name, command_str));
            }
        }
    }
    
    let pea_toml_content = create_pea_toml_content(
        name,
        version,
        &description,
        main,
        &dependencies,
        &dev_dependencies,
        &scripts,
    );
    
    fs::write(pea_toml_path, pea_toml_content)
        .map_err(|e| PeaError::Io {
            message: format!("Failed to create pea.toml: {}", pea_toml_path.display()),
            source: e,
        })?;
    
    ctx.output.success("Imported configuration from package.json");
    Ok(())
}

/// Create a default pea.toml file
async fn create_default_pea_toml(pea_toml_path: &Path, ctx: &CommandContext) -> PeaResult<()> {
    let dir_name = ctx.cwd
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("my-pea-project");
    
    let pea_toml_content = create_pea_toml_content(
        dir_name,
        "0.1.0",
        "",
        "src/index.ts",
        &[],
        &[],
        &[
            "dev = \"pea run src/index.ts\"".to_string(),
            "build = \"pea build\"".to_string(),
            "test = \"pea test\"".to_string(),
        ],
    );
    
    fs::write(pea_toml_path, pea_toml_content)
        .map_err(|e| PeaError::Io {
            message: format!("Failed to create pea.toml: {}", pea_toml_path.display()),
            source: e,
        })?;
    
    ctx.output.success("Created pea.toml");
    Ok(())
}

fn create_pea_toml_content(
    name: &str,
    version: &str,
    description: &str,
    main: &str,
    dependencies: &[String],
    dev_dependencies: &[String],
    scripts: &[String],
) -> String {
    let mut content = format!(
        r#"[package]
name = "{}"
version = "{}"
{}"#,
        name,
        version,
        if description.is_empty() { "" } else { description }
    );
    
    if main != "src/index.ts" {
        content.push_str(&format!("\nmain = \"{}\"", main));
    }
    
    content.push_str("\n\n[dependencies]");
    for dep in dependencies {
        content.push_str(&format!("\n{}", dep));
    }
    
    content.push_str("\n\n[dev-dependencies]");
    for dep in dev_dependencies {
        content.push_str(&format!("\n{}", dep));
    }
    
    content.push_str("\n\n[scripts]");
    for script in scripts {
        content.push_str(&format!("\n{}", script));
    }
    
    content.push('\n');
    content
}

fn create_basic_index(index_path: &Path) -> PeaResult<()> {
    let content = r#"// Welcome to Pea! 🫛
console.log("Hello from Pea!");

// Add your code here
export {};
"#;
    
    fs::write(index_path, content)
        .map_err(|e| PeaError::Io {
            message: format!("Failed to create index.ts: {}", index_path.display()),
            source: e,
        })?;
    
    Ok(())
}

fn create_project_guidemap(guidemap_path: &Path) -> PeaResult<()> {
    let content = r#"# Project Guide

## Purpose
A Pea project initialized with `pea init`.

## Structure
- `src/` - Source code directory
- `pea.toml` - Project configuration
- `GUIDEMAP.md` - This file

## Getting Started
1. Install dependencies: `pea install`
2. Run in development: `pea run dev`
3. Build for production: `pea build`
4. Run tests: `pea test`

## Development
- Add dependencies with `pea add <package>`
- Remove dependencies with `pea remove <package>`
- Run scripts with `pea run <script>`

## Resources
- Pea Documentation: https://pea-lang.org/docs
- Community: https://github.com/pea-lang/pea/discussions
"#;
    
    fs::write(guidemap_path, content)
        .map_err(|e| PeaError::Io {
            message: format!("Failed to create GUIDEMAP.md: {}", guidemap_path.display()),
            source: e,
        })?;
    
    Ok(())
}