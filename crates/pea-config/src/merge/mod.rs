//! Configuration layering, fallback logic, and environment overrides

use std::collections::HashMap;
use camino::Utf8PathBuf;
use pea_core::error::PeaError;
use crate::{ConfigResult, toml::PeaToml};

/// Main configuration loading interface
pub struct ConfigLoader {
    /// Current working directory
    cwd: Utf8PathBuf,
}

/// Configuration layering and merging
pub struct ConfigLayering {
    /// Global configuration
    global_config: Option<PeaToml>,
    /// Project configuration
    project_config: Option<PeaToml>,
    /// Environment overrides
    env_overrides: HashMap<String, String>,
    /// CLI flag overrides
    cli_overrides: HashMap<String, String>,
}

/// Configuration source tracking
#[derive(Debug, Clone, PartialEq)]
pub enum ConfigSource {
    /// Global config file
    Global(Utf8PathBuf),
    /// Project pea.toml file
    ProjectToml(Utf8PathBuf),
    /// Project package.json file (fallback)
    ProjectJson(Utf8PathBuf),
    /// Environment variable
    Environment(String),
    /// CLI flag
    CommandLine,
}

impl ConfigLoader {
    /// Create a new configuration loader
    pub fn new(cwd: Utf8PathBuf) -> Self {
        Self { cwd }
    }
    
    /// Load project configuration with fallbacks
    pub async fn load_project_config(&self) -> ConfigResult<(PeaToml, ConfigSource)> {
        // First, try to find pea.toml
        let pea_toml_path = self.resolve_config_path("pea.toml")?;
        if pea_toml_path.exists() {
            let config = crate::toml::load_from_file(&pea_toml_path).await?;
            return Ok((config, ConfigSource::ProjectToml(pea_toml_path)));
        }
        
        // Fall back to package.json if no pea.toml
        let package_json_path = self.resolve_config_path("package.json")?;
        if package_json_path.exists() {
            let package_json = crate::json::load_from_file(&package_json_path).await?;
            let config = crate::json::import_to_pea_toml(&package_json)?;
            return Ok((config, ConfigSource::ProjectJson(package_json_path)));
        }
        
        // No configuration found
        Err(PeaError::ConfigValidation {
            field: "config".to_string(),
            reason: "No pea.toml or package.json found in current directory or parent directories".to_string(),
        })
    }
    
    /// Find configuration file in project (walks up directory tree)
    pub fn resolve_config_path(&self, filename: &str) -> ConfigResult<Utf8PathBuf> {
        let mut current = self.cwd.as_path();
        
        loop {
            let config_path = current.join(filename);
            if config_path.exists() {
                return Ok(config_path);
            }
            
            // Move up one directory
            if let Some(parent) = current.parent() {
                current = parent;
            } else {
                // Reached filesystem root
                break;
            }
        }
        
        // Return path in current directory even if it doesn't exist
        Ok(self.cwd.join(filename))
    }
    
    /// Load global configuration
    pub async fn load_global_config(&self) -> ConfigResult<Option<PeaToml>> {
        let home_dir = dirs::home_dir()
            .ok_or_else(|| PeaError::ConfigValidation {
                field: "home_dir".to_string(),
                reason: "Could not determine home directory".to_string(),
            })?;
        
        let global_config_path = Utf8PathBuf::try_from(home_dir)
            .map_err(|e| PeaError::ConfigValidation {
                field: "home_dir".to_string(),
                reason: format!("Invalid home directory path: {}", e),
            })?
            .join(".pea")
            .join("config.toml");
        
        if global_config_path.exists() {
            let config = crate::toml::load_from_file(&global_config_path).await?;
            Ok(Some(config))
        } else {
            Ok(None)
        }
    }
}

impl ConfigLayering {
    /// Create a new configuration layering system
    pub fn new() -> Self {
        Self {
            global_config: None,
            project_config: None,
            env_overrides: HashMap::new(),
            cli_overrides: HashMap::new(),
        }
    }
    
    /// Merge multiple configuration layers
    pub fn merge_configs(
        global_config: Option<PeaToml>,
        project_config: PeaToml,
        env_overrides: HashMap<String, String>,
        cli_overrides: HashMap<String, String>,
    ) -> ConfigResult<PeaToml> {
        let mut merged = project_config;
        
        // Apply global config as base (if present)
        if let Some(global) = global_config {
            // Merge global dependencies that aren't overridden
            for (name, spec) in global.dependencies {
                merged.dependencies.entry(name).or_insert(spec);
            }
            
            // Merge global scripts that aren't overridden
            for (name, script) in global.scripts {
                merged.scripts.entry(name).or_insert(script);
            }
            
            // Merge global profiles that aren't overridden
            for (name, profile) in global.profile {
                merged.profile.entry(name).or_insert(profile);
            }
        }
        
        // Apply environment variable overrides
        Self::apply_env_overrides(&mut merged, &env_overrides)?;
        
        // Apply CLI flag overrides (highest priority)
        Self::apply_cli_overrides(&mut merged, &cli_overrides)?;
        
        Ok(merged)
    }
    
    /// Apply environment variable overrides
    fn apply_env_overrides(config: &mut PeaToml, overrides: &HashMap<String, String>) -> ConfigResult<()> {
        for (key, value) in overrides {
            match key.as_str() {
                "PEA_PACKAGE_NAME" => {
                    config.package.name = value.clone();
                }
                "PEA_PACKAGE_VERSION" => {
                    config.package.version = value.parse()
                        .map_err(|e| PeaError::ConfigValidation {
                            field: "PEA_PACKAGE_VERSION".to_string(),
                            reason: format!("Invalid version in PEA_PACKAGE_VERSION: {}", e),
                        })?;
                }
                "PEA_PACKAGE_DESCRIPTION" => {
                    config.package.description = Some(value.clone());
                }
                key if key.starts_with("PEA_SCRIPT_") => {
                    let script_name = key.strip_prefix("PEA_SCRIPT_").unwrap().to_lowercase();
                    config.scripts.insert(script_name, value.clone());
                }
                _ => {
                    // Unknown environment variable, ignore
                }
            }
        }
        
        Ok(())
    }
    
    /// Apply CLI flag overrides
    fn apply_cli_overrides(config: &mut PeaToml, overrides: &HashMap<String, String>) -> ConfigResult<()> {
        for (key, value) in overrides {
            match key.as_str() {
                "name" => {
                    config.package.name = value.clone();
                }
                "version" => {
                    config.package.version = value.parse()
                        .map_err(|e| PeaError::ConfigValidation {
                            field: "version".to_string(),
                            reason: format!("Invalid version in --version flag: {}", e),
                        })?;
                }
                "description" => {
                    config.package.description = Some(value.clone());
                }
                _ => {
                    // Unknown CLI override, ignore
                }
            }
        }
        
        Ok(())
    }
    
    /// Collect environment variable overrides
    pub fn collect_env_overrides() -> HashMap<String, String> {
        let mut overrides = HashMap::new();
        
        for (key, value) in std::env::vars() {
            if key.starts_with("PEA_") {
                overrides.insert(key, value);
            }
        }
        
        overrides
    }
}

impl Default for ConfigLayering {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::toml::PackageSection;
    use pea_core::types::Version;
    use tempfile::TempDir;

    
    fn create_test_config() -> PeaToml {
        PeaToml {
            package: PackageSection {
                name: "test-package".to_string(),
                version: Version::new(1, 0, 0),
                description: Some("A test package".to_string()),
                main: None,
                license: None,
                repository: None,
                keywords: Vec::new(),
                authors: Vec::new(),
                homepage: None,
            },
            dependencies: HashMap::new(),
            dev_dependencies: HashMap::new(),
            peer_dependencies: HashMap::new(),
            optional_dependencies: HashMap::new(),
            scripts: HashMap::new(),
            features: HashMap::new(),
            workspace: None,
            profile: HashMap::new(),
        }
    }
    
    #[tokio::test]
    async fn test_config_loader_creation() {
        let cwd = Utf8PathBuf::from("/test");
        let loader = ConfigLoader::new(cwd.clone());
        assert_eq!(loader.cwd, cwd);
    }
    
    #[tokio::test]
    async fn test_resolve_config_path() {
        let temp_dir = TempDir::new().unwrap();
        let temp_path = Utf8PathBuf::try_from(temp_dir.path().to_path_buf()).unwrap();
        
        // Create a pea.toml file
        let pea_toml_path = temp_path.join("pea.toml");
        tokio::fs::write(&pea_toml_path, "[package]\nname = \"test\"\nversion = \"1.0.0\"").await.unwrap();
        
        let loader = ConfigLoader::new(temp_path.clone());
        let resolved = loader.resolve_config_path("pea.toml").unwrap();
        
        assert_eq!(resolved, pea_toml_path);
    }
    
    #[tokio::test]
    async fn test_load_project_config_pea_toml() {
        let temp_dir = TempDir::new().unwrap();
        let temp_path = Utf8PathBuf::try_from(temp_dir.path().to_path_buf()).unwrap();
        
        // Create a pea.toml file
        let pea_toml_content = r#"
[package]
name = "test-package"
version = "1.0.0"
description = "A test package"
"#;
        tokio::fs::write(temp_path.join("pea.toml"), pea_toml_content).await.unwrap();
        
        let loader = ConfigLoader::new(temp_path);
        let (config, source) = loader.load_project_config().await.unwrap();
        
        assert_eq!(config.package.name, "test-package");
        assert!(matches!(source, ConfigSource::ProjectToml(_)));
    }
    
    #[tokio::test]
    async fn test_load_project_config_package_json_fallback() {
        let temp_dir = TempDir::new().unwrap();
        let temp_path = Utf8PathBuf::try_from(temp_dir.path().to_path_buf()).unwrap();
        
        // Create only a package.json file (no pea.toml)
        let package_json_content = r#"
{
  "name": "test-package",
  "version": "1.0.0",
  "description": "A test package"
}
"#;
        tokio::fs::write(temp_path.join("package.json"), package_json_content).await.unwrap();
        
        let loader = ConfigLoader::new(temp_path);
        let (config, source) = loader.load_project_config().await.unwrap();
        
        assert_eq!(config.package.name, "test-package");
        assert!(matches!(source, ConfigSource::ProjectJson(_)));
    }
    
    #[test]
    fn test_merge_configs() {
        let mut global_config = create_test_config();
        global_config.package.name = "global-package".to_string();
        global_config.scripts.insert("global-script".to_string(), "echo global".to_string());
        
        let mut project_config = create_test_config();
        project_config.package.name = "project-package".to_string();
        project_config.scripts.insert("build".to_string(), "tsc".to_string());
        
        let env_overrides = HashMap::from([
            ("PEA_PACKAGE_DESCRIPTION".to_string(), "Overridden description".to_string()),
        ]);
        
        let cli_overrides = HashMap::from([
            ("version".to_string(), "2.0.0".to_string()),
        ]);
        
        let merged = ConfigLayering::merge_configs(
            Some(global_config),
            project_config,
            env_overrides,
            cli_overrides,
        ).unwrap();
        
        // Project config should take precedence over global
        assert_eq!(merged.package.name, "project-package");
        
        // Global script should be merged
        assert_eq!(merged.scripts.get("global-script").unwrap(), "echo global");
        
        // Project script should be preserved
        assert_eq!(merged.scripts.get("build").unwrap(), "tsc");
        
        // Environment override should be applied
        assert_eq!(merged.package.description, Some("Overridden description".to_string()));
        
        // CLI override should be applied (highest priority)
        assert_eq!(merged.package.version.to_string(), "2.0.0");
    }
    
    #[test]
    fn test_collect_env_overrides() {
        // Set some test environment variables
        std::env::set_var("PEA_PACKAGE_NAME", "env-package");
        std::env::set_var("PEA_SCRIPT_BUILD", "env-build");
        std::env::set_var("NOT_PEA_VAR", "ignored");
        
        let overrides = ConfigLayering::collect_env_overrides();
        
        assert!(overrides.contains_key("PEA_PACKAGE_NAME"));
        assert!(overrides.contains_key("PEA_SCRIPT_BUILD"));
        assert!(!overrides.contains_key("NOT_PEA_VAR"));
        
        // Clean up
        std::env::remove_var("PEA_PACKAGE_NAME");
        std::env::remove_var("PEA_SCRIPT_BUILD");
        std::env::remove_var("NOT_PEA_VAR");
    }
}