//! `pea install` command implementation.
//!
//! Installs dependencies specified in pea.toml by resolving them,
//! downloading packages, storing in CAS, and creating node_modules.

use camino::{Utf8Path, Utf8PathBuf};
use pea_cache::{CasStore, Linker};
use pea_config::{ConfigLoader, PeaToml};
use pea_core::error::{PeaError, PeaResult};
use pea_registry::{RegistryClient, MetadataCache};
use pea_resolver::Resolver;
use std::sync::Arc;
use std::time::Instant;
use tokio::fs;

use super::CommandContext;

/// Execute the `pea install` command
pub async fn execute(frozen: bool, ctx: &CommandContext) -> PeaResult<()> {
    let start_time = Instant::now();
    
    if frozen {
        ctx.output.step("🔒", "Installing dependencies (frozen mode)");
    } else {
        ctx.output.step("📦", "Installing dependencies");
    }
    
    // Parse configuration
    let cwd_utf8 = Utf8PathBuf::from_path_buf(ctx.cwd.clone()).unwrap();
    let config_loader = ConfigLoader::new(cwd_utf8);
    let (config, _source) = config_loader.load_project_config().await?;
    
    // Check for existing lockfile
    let lockfile_path = ctx.cwd.join("pea.lock");
    let has_lockfile = lockfile_path.exists();
    
    if frozen && !has_lockfile {
        return Err(PeaError::ConfigValidation {
            field: "lockfile".to_string(),
            reason: "Frozen install requires existing lockfile".to_string(),
        });
    }
    
    // Initialize components
    let cache_dir = get_cache_dir()?;
    let cas_store = Arc::new(CasStore::new(&cache_dir.join("store"))?);
    let registry_client = Arc::new(RegistryClient::new()?);
    let metadata_cache = Arc::new(MetadataCache::new());
    let resolver = Resolver::new(registry_client.clone(), metadata_cache.clone());
    let linker = Linker::new(cas_store.clone());
    
    if has_lockfile && !frozen {
        // Cached install flow
        ctx.output.step("⚡", "Using cached dependencies");
        cached_install(&config, &linker, ctx).await?;
    } else {
        // Fresh install flow
        ctx.output.step("🔍", "Resolving dependencies");
        fresh_install(&config, &resolver, &cas_store, &linker, ctx).await?;
    }
    
    let duration = start_time.elapsed();
    ctx.output.success(&format!("✅ Dependencies installed in {:.2}s", duration.as_secs_f64()));
    
    Ok(())
}

/// Perform a fresh install by resolving dependencies
async fn fresh_install(
    config: &PeaToml,
    resolver: &Resolver,
    cas_store: &Arc<CasStore>,
    linker: &Linker,
    ctx: &CommandContext,
) -> PeaResult<()> {
    ctx.output.step("🧩", "Resolving dependency tree");
    
    // Check if we have dependencies to install
    let dep_count = config.dependencies.len();
    ctx.output.info(&format!("📊 Found {} dependencies to install", dep_count));
    
    if dep_count == 0 {
        ctx.output.info("No dependencies to install");
        return create_empty_node_modules(ctx).await;
    }
    
    // Convert dependencies to resolver format
    let root_dependencies: Vec<(String, String)> = config.dependencies
        .iter()
        .map(|(name, spec)| {
            let version_req = match spec {
                pea_config::DependencySpec::Simple(version) => version.clone(),
                pea_config::DependencySpec::Detailed { version, .. } => {
                    version.clone().unwrap_or_else(|| "*".to_string())
                },
            };
            (name.clone(), version_req)
        })
        .collect();
    
    // Resolve dependencies
    ctx.output.info(&format!("🔍 Resolving {} root dependencies", root_dependencies.len()));
    let resolution_result = resolver.resolve(root_dependencies).await
        .map_err(|e| PeaError::VersionConflict {
            package: "resolution".to_string(),
            required: "compatible versions".to_string(),
            conflicting: "resolution".to_string(),
            conflict: e.to_string(),
        })?;
    
    ctx.output.info(&format!("✅ Resolved {} packages in {}ms", 
        resolution_result.package_count, 
        resolution_result.resolution_time_ms));
    
    // Download and store packages in CAS
    ctx.output.step("📥", "Downloading packages");
    let packages = download_packages(&resolution_result.graph, cas_store, ctx).await?;
    
    // Create node_modules structure
    ctx.output.step("🔗", "Creating node_modules");
    let node_modules_dir = Utf8PathBuf::from_path_buf(ctx.cwd.join("node_modules")).unwrap();
    
    // Clean existing node_modules if it exists
    if node_modules_dir.exists() {
        linker.cleanup_node_modules(&node_modules_dir)?;
    }
    
    let link_result = linker.create_node_modules(&packages, &node_modules_dir)?;
    
    ctx.output.info(&format!("  📦 Linked {} packages", link_result.packages_linked));
    ctx.output.info(&format!("  🔗 Created {} hardlinks", link_result.hardlinks_created));
    if link_result.files_copied > 0 {
        ctx.output.info(&format!("  📄 Copied {} files (fallback)", link_result.files_copied));
    }
    if link_result.bin_links_created > 0 {
        ctx.output.info(&format!("  🔧 Created {} binary links", link_result.bin_links_created));
    }
    
    // TODO: Generate lockfile
    ctx.output.step("🔒", "Generating lockfile");
    ctx.output.info("  📝 pea.lock (lockfile generation not yet implemented)");
    
    Ok(())
}

/// Create empty node_modules directory for projects with no dependencies
async fn create_empty_node_modules(ctx: &CommandContext) -> PeaResult<()> {
    ctx.output.step("🔗", "Creating node_modules");
    let node_modules_dir = Utf8PathBuf::from_path_buf(ctx.cwd.join("node_modules")).unwrap();
    fs::create_dir_all(&node_modules_dir).await
        .map_err(|e| PeaError::io("Failed to create node_modules".to_string(), e))?;
    
    ctx.output.info("  📁 Created empty node_modules directory");
    Ok(())
}

/// Download packages and store them in CAS
async fn download_packages(
    graph: &pea_resolver::graph::DependencyGraph,
    cas_store: &Arc<CasStore>,
    ctx: &CommandContext,
) -> PeaResult<Vec<pea_cache::link::PackageInfo>> {
    use pea_cache::link::PackageInfo;
    use pea_registry::RegistryClient;
    
    let mut packages = Vec::new();
    let registry_client = RegistryClient::new()
        .map_err(|e| PeaError::Network { 
            message: format!("Failed to create registry client: {}", e),
            source: Some(Box::new(e))
        })?;
    
    let total_packages = graph.package_count();
    let mut downloaded = 0;
    
    for package in graph.packages() {
        downloaded += 1;
        ctx.output.info(&format!("  📦 [{}/{}] Downloading {}@{}", 
            downloaded, total_packages, package.name, package.version));
        
        // Skip workspace packages (they have file:// URLs)
        if package.resolved_url.starts_with("file://") {
            ctx.output.info(&format!("    📁 Workspace package, skipping download"));
            
            // For workspace packages, use the local path directly
            let workspace_path = package.resolved_url.strip_prefix("file://").unwrap();
            let package_info = PackageInfo::new(
                package.name.clone(),
                package.version.to_string(),
                Utf8PathBuf::from(workspace_path),
            ).as_workspace();
            
            packages.push(package_info);
            continue;
        }
        
        // Download tarball
        let tarball_bytes = registry_client.download_tarball(&pea_registry::api::DistInfo {
            tarball: package.resolved_url.clone(),
            shasum: extract_shasum(&package.integrity),
            integrity: Some(package.integrity.clone()),
            file_count: Some(0), // Not used for download
            unpacked_size: Some(0), // Not used for download
        }).await
        .map_err(|e| PeaError::Network {
            message: format!("Failed to download {}: {}", package.name, e),
            source: Some(Box::new(e))
        })?;
        
        // Store in CAS
        let content_hash = cas_store.store(&tarball_bytes)?;
        ctx.output.info(&format!("    💾 Stored in CAS: {}", content_hash.to_hex()[..12].to_string()));
        
        // Extract to temporary location for linking
        let temp_dir = tempfile::tempdir()
            .map_err(|e| PeaError::io("Failed to create temp directory".to_string(), e))?;
        
        let extract_path = Utf8PathBuf::from_path_buf(temp_dir.path().to_path_buf()).unwrap();
        let tarball_cursor = std::io::Cursor::new(&tarball_bytes);
        pea_cache::tarball::extract_tarball(tarball_cursor, extract_path.as_std_path())?;
        
        // Look for package.json to extract bin entries
        let package_json_path = extract_path.join("package").join("package.json");
        let bin_entries = if package_json_path.exists() {
            extract_bin_entries(&package_json_path)?
        } else {
            std::collections::HashMap::new()
        };
        
        let package_info = PackageInfo::new(
            package.name.clone(),
            package.version.to_string(),
            extract_path.join("package"), // npm tarballs have package/ prefix
        );
        
        let package_info = bin_entries.into_iter().fold(package_info, |pkg, (name, path)| {
            pkg.with_bin(name, path)
        });
        
        packages.push(package_info);
    }
    
    ctx.output.info(&format!("✅ Downloaded and stored {} packages", packages.len()));
    Ok(packages)
}

/// Extract shasum from integrity string
fn extract_shasum(integrity: &str) -> String {
    // For now, return a placeholder since we're using blake3 for integrity
    // In a real implementation, we'd parse the integrity string properly
    integrity.chars().filter(|c| c.is_alphanumeric()).take(40).collect()
}

/// Extract binary entries from package.json
fn extract_bin_entries(package_json_path: &Utf8Path) -> PeaResult<std::collections::HashMap<String, String>> {
    use std::collections::HashMap;
    
    let content = std::fs::read_to_string(package_json_path)
        .map_err(|e| PeaError::io("Failed to read package.json".to_string(), e))?;
    
    let package_json: serde_json::Value = serde_json::from_str(&content)
        .map_err(|e| PeaError::JsonParse {
            message: format!("Failed to parse {}: {}", package_json_path, e),
        })?;
    
    let mut bin_entries = HashMap::new();
    
    if let Some(bin) = package_json.get("bin") {
        match bin {
            serde_json::Value::String(path) => {
                // Single binary with package name
                if let Some(name) = package_json.get("name").and_then(|n| n.as_str()) {
                    bin_entries.insert(name.to_string(), path.clone());
                }
            }
            serde_json::Value::Object(map) => {
                // Multiple binaries
                for (name, path) in map {
                    if let Some(path_str) = path.as_str() {
                        bin_entries.insert(name.clone(), path_str.to_string());
                    }
                }
            }
            _ => {}
        }
    }
    
    Ok(bin_entries)
}

/// Perform a cached install using existing lockfile
async fn cached_install(
    config: &PeaToml,
    linker: &Linker,
    ctx: &CommandContext,
) -> PeaResult<()> {
    ctx.output.step("📋", "Reading lockfile");
    ctx.output.warn("Lockfile reading not yet implemented, falling back to fresh install");
    
    // For now, fall back to fresh install since lockfile system isn't implemented yet
    // This ensures we still provide working functionality
    let cache_dir = get_cache_dir()?;
    let cas_store = Arc::new(CasStore::new(&cache_dir.join("store"))?);
    let registry_client = Arc::new(RegistryClient::new()?);
    let metadata_cache = Arc::new(MetadataCache::new());
    let resolver = Resolver::new(registry_client.clone(), metadata_cache.clone());
    
    fresh_install(config, &resolver, &cas_store, linker, ctx).await
}



/// Get the cache directory path
fn get_cache_dir() -> PeaResult<Utf8PathBuf> {
    let home_dir = dirs::home_dir()
        .ok_or_else(|| PeaError::ConfigValidation {
            field: "home_directory".to_string(),
            reason: "Could not determine home directory".to_string(),
        })?;
    
    let cache_dir = home_dir.join(".pea");
    Ok(Utf8PathBuf::from_path_buf(cache_dir).unwrap())
}