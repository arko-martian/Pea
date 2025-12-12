//! pea.toml configuration parsing and serialization

use std::collections::HashMap;
use std::str::FromStr;
use serde::{Deserialize, Serialize};
use pea_core::types::{Version, VersionReq};
use pea_core::error::PeaError;
use crate::ConfigResult;

/// Complete pea.toml configuration
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PeaToml {
    /// Package metadata section
    pub package: PackageSection,
    
    /// Runtime dependencies
    #[serde(default)]
    pub dependencies: HashMap<String, DependencySpec>,
    
    /// Development dependencies
    #[serde(default, rename = "dev-dependencies")]
    pub dev_dependencies: HashMap<String, DependencySpec>,
    
    /// Peer dependencies
    #[serde(default, rename = "peer-dependencies")]
    pub peer_dependencies: HashMap<String, DependencySpec>,
    
    /// Optional dependencies
    #[serde(default, rename = "optional-dependencies")]
    pub optional_dependencies: HashMap<String, DependencySpec>,
    
    /// Build scripts
    #[serde(default)]
    pub scripts: HashMap<String, String>,
    
    /// Feature flags
    #[serde(default)]
    pub features: HashMap<String, Vec<String>>,
    
    /// Workspace configuration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub workspace: Option<WorkspaceSection>,
    
    /// Build profiles
    #[serde(default)]
    pub profile: HashMap<String, ProfileSection>,
}

/// Package metadata section
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PackageSection {
    /// Package name (required)
    pub name: String,
    
    /// Package version (required)
    #[serde(deserialize_with = "deserialize_version", serialize_with = "serialize_version")]
    pub version: Version,
    
    /// Package description
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    
    /// Main entry point
    #[serde(skip_serializing_if = "Option::is_none")]
    pub main: Option<String>,
    
    /// License identifier
    #[serde(skip_serializing_if = "Option::is_none")]
    pub license: Option<String>,
    
    /// Repository URL
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repository: Option<String>,
    
    /// Keywords for discovery
    #[serde(default)]
    pub keywords: Vec<String>,
    
    /// Authors
    #[serde(default)]
    pub authors: Vec<String>,
    
    /// Homepage URL
    #[serde(skip_serializing_if = "Option::is_none")]
    pub homepage: Option<String>,
}

/// Dependency specification (simple string or detailed object)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum DependencySpec {
    /// Simple version requirement string
    Simple(String),
    
    /// Detailed dependency specification
    Detailed {
        /// Version requirement
        #[serde(skip_serializing_if = "Option::is_none")]
        version: Option<String>,
        
        /// Git repository URL
        #[serde(skip_serializing_if = "Option::is_none")]
        git: Option<String>,
        
        /// Git branch
        #[serde(skip_serializing_if = "Option::is_none")]
        branch: Option<String>,
        
        /// Git tag
        #[serde(skip_serializing_if = "Option::is_none")]
        tag: Option<String>,
        
        /// Git revision
        #[serde(skip_serializing_if = "Option::is_none")]
        rev: Option<String>,
        
        /// Local path
        #[serde(skip_serializing_if = "Option::is_none")]
        path: Option<String>,
        
        /// Workspace reference
        #[serde(skip_serializing_if = "Option::is_none")]
        workspace: Option<bool>,
        
        /// Features to enable
        #[serde(default)]
        features: Vec<String>,
        
        /// Whether dependency is optional
        #[serde(default)]
        optional: bool,
        
        /// Default features flag
        #[serde(default = "default_features_true", rename = "default-features")]
        default_features: bool,
    },
}

/// Workspace configuration
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WorkspaceSection {
    /// Workspace member paths (glob patterns)
    pub members: Vec<String>,
    
    /// Paths to exclude from workspace
    #[serde(default)]
    pub exclude: Vec<String>,
    
    /// Shared workspace dependencies
    #[serde(default)]
    pub dependencies: HashMap<String, DependencySpec>,
}

/// Build profile configuration
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProfileSection {
    /// Optimization level
    #[serde(skip_serializing_if = "Option::is_none")]
    pub opt_level: Option<String>,
    
    /// Debug information
    #[serde(skip_serializing_if = "Option::is_none")]
    pub debug: Option<bool>,
    
    /// Link-time optimization
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lto: Option<bool>,
    
    /// Code generation units
    #[serde(skip_serializing_if = "Option::is_none")]
    pub codegen_units: Option<u32>,
    
    /// Panic strategy
    #[serde(skip_serializing_if = "Option::is_none")]
    pub panic: Option<String>,
}

/// Default value for default-features (true)
fn default_features_true() -> bool {
    true
}

impl DependencySpec {
    /// Get the version requirement string
    pub fn version_req(&self) -> ConfigResult<Option<VersionReq>> {
        match self {
            DependencySpec::Simple(version) => {
                let req = VersionReq::parse(version)
                    .map_err(|e| PeaError::ConfigValidation {
                        field: "version".to_string(),
                        reason: format!("Invalid version requirement '{}': {}", version, e),
                    })?;
                Ok(Some(req))
            }
            DependencySpec::Detailed { version: Some(version), .. } => {
                let req = VersionReq::parse(version)
                    .map_err(|e| PeaError::ConfigValidation {
                        field: "version".to_string(),
                        reason: format!("Invalid version requirement '{}': {}", version, e),
                    })?;
                Ok(Some(req))
            }
            DependencySpec::Detailed { version: None, .. } => Ok(None),
        }
    }
    
    /// Check if this is a workspace dependency
    pub fn is_workspace(&self) -> bool {
        matches!(self, DependencySpec::Detailed { workspace: Some(true), .. })
    }
    
    /// Check if this is a path dependency
    pub fn is_path(&self) -> bool {
        matches!(self, DependencySpec::Detailed { path: Some(_), .. })
    }
    
    /// Check if this is a git dependency
    pub fn is_git(&self) -> bool {
        matches!(self, DependencySpec::Detailed { git: Some(_), .. })
    }
}

/// Parse TOML string to PeaToml configuration
pub fn parse_pea_toml(content: &str) -> ConfigResult<PeaToml> {
    // First try with toml_edit for better error reporting
    let _document = content.parse::<toml_edit::DocumentMut>()
        .map_err(|e| PeaError::TomlParse {
            message: format!("TOML syntax error: {}", e),
            line: 0,
            column: 0,
        })?;
    
    // Then parse with serde for type safety
    let config: PeaToml = toml::from_str(content)
        .map_err(|e| PeaError::TomlParse {
            message: format!("TOML parsing error: {}", e),
            line: 0,
            column: 0,
        })?;
    
    // Validate required fields
    validate_config(&config)?;
    
    Ok(config)
}

/// Serialize PeaToml to TOML string
pub fn serialize_pea_toml(config: &PeaToml) -> ConfigResult<String> {
    toml::to_string_pretty(config)
        .map_err(|e| PeaError::TomlParse {
            message: format!("TOML serialization error: {}", e),
            line: 0,
            column: 0,
        })
}

/// Validate configuration completeness
pub fn validate_config(config: &PeaToml) -> ConfigResult<()> {
    // Validate package name
    if config.package.name.is_empty() {
        return Err(PeaError::ConfigValidation {
            field: "package.name".to_string(),
            reason: "Package name is required in [package] section".to_string(),
        });
    }
    
    // Validate package name format (npm-compatible)
    if !is_valid_package_name(&config.package.name) {
        return Err(PeaError::ConfigValidation {
            field: "package.name".to_string(),
            reason: format!("Invalid package name '{}'. Package names must be lowercase, alphanumeric, and may contain hyphens or underscores", config.package.name),
        });
    }
    
    // Validate dependency specifications
    for (name, spec) in &config.dependencies {
        validate_dependency_spec(name, spec)?;
    }
    
    for (name, spec) in &config.dev_dependencies {
        validate_dependency_spec(name, spec)?;
    }
    
    for (name, spec) in &config.peer_dependencies {
        validate_dependency_spec(name, spec)?;
    }
    
    for (name, spec) in &config.optional_dependencies {
        validate_dependency_spec(name, spec)?;
    }
    
    // Validate workspace configuration
    if let Some(workspace) = &config.workspace {
        if workspace.members.is_empty() {
            return Err(PeaError::ConfigValidation {
                field: "workspace.members".to_string(),
                reason: "Workspace must have at least one member".to_string(),
            });
        }
    }
    
    Ok(())
}

/// Load and parse pea.toml from file path
pub async fn load_from_file(path: &camino::Utf8Path) -> ConfigResult<PeaToml> {
    let content = tokio::fs::read_to_string(path).await
        .map_err(|e| PeaError::io(format!("Failed to read {}", path), e))?;
    
    parse_pea_toml(&content)
        .map_err(|e| match e {
            PeaError::TomlParse { message, line, column } => PeaError::TomlParse {
                message: format!("In file {}: {}", path, message),
                line,
                column,
            },
            PeaError::ConfigValidation { field, reason } => PeaError::ConfigValidation {
                field,
                reason: format!("In file {}: {}", path, reason),
            },
            other => other,
        })
}

/// Validate a dependency specification
fn validate_dependency_spec(name: &str, spec: &DependencySpec) -> ConfigResult<()> {
    // Validate dependency name
    if !is_valid_package_name(name) {
        return Err(PeaError::ConfigValidation {
            field: format!("dependencies.{}", name),
            reason: format!("Invalid dependency name '{}'. Package names must be lowercase, alphanumeric, and may contain hyphens or underscores", name),
        });
    }
    
    // Validate version requirement if present
    if let Ok(Some(_)) = spec.version_req() {
        // Version requirement is valid
    } else if let Err(e) = spec.version_req() {
        return Err(e);
    }
    
    // Validate that at least one source is specified
    match spec {
        DependencySpec::Simple(_) => {
            // Simple version spec is always valid
        }
        DependencySpec::Detailed { version, git, path, workspace, .. } => {
            let source_count = [version.is_some(), git.is_some(), path.is_some(), workspace.unwrap_or(false)].iter().filter(|&&x| x).count();
            
            if source_count == 0 {
                return Err(PeaError::ConfigValidation {
                    field: format!("dependencies.{}", name),
                    reason: format!("Dependency '{}' must specify at least one source (version, git, path, or workspace)", name),
                });
            }
            
            if source_count > 1 {
                return Err(PeaError::ConfigValidation {
                    field: format!("dependencies.{}", name),
                    reason: format!("Dependency '{}' can only specify one source (version, git, path, or workspace)", name),
                });
            }
        }
    }
    
    Ok(())
}

/// Check if a package name is valid (npm-compatible)
fn is_valid_package_name(name: &str) -> bool {
    if name.is_empty() || name.len() > 214 {
        return false;
    }
    
    // Must start with alphanumeric or @
    if !name.chars().next().unwrap_or(' ').is_alphanumeric() && !name.starts_with('@') {
        return false;
    }
    
    // Can contain alphanumeric, hyphens, underscores, dots, and forward slashes (for scoped packages)
    name.chars().all(|c| c.is_alphanumeric() || matches!(c, '-' | '_' | '.' | '/' | '@'))
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_parse_minimal_config() {
        let toml = r#"
[package]
name = "test-package"
version = "1.0.0"
"#;
        
        let config = parse_pea_toml(toml).unwrap();
        assert_eq!(config.package.name, "test-package");
        assert_eq!(config.package.version.to_string(), "1.0.0");
        assert!(config.dependencies.is_empty());
    }
    
    #[test]
    fn test_parse_with_dependencies() {
        let toml = r#"
[package]
name = "test-package"
version = "1.0.0"

[dependencies]
lodash = "^4.17.21"
react = { version = "^18.0.0", features = ["jsx"] }
local-pkg = { path = "../local-pkg" }
workspace-pkg = { workspace = true }
"#;
        
        let config = parse_pea_toml(toml).unwrap();
        assert_eq!(config.dependencies.len(), 4);
        
        // Test simple dependency
        assert!(matches!(config.dependencies.get("lodash"), Some(DependencySpec::Simple(_))));
        
        // Test detailed dependency
        if let Some(DependencySpec::Detailed { version, features, .. }) = config.dependencies.get("react") {
            assert_eq!(version.as_ref().unwrap(), "^18.0.0");
            assert_eq!(features, &vec!["jsx".to_string()]);
        } else {
            panic!("Expected detailed dependency for react");
        }
        
        // Test path dependency
        assert!(config.dependencies.get("local-pkg").unwrap().is_path());
        
        // Test workspace dependency
        assert!(config.dependencies.get("workspace-pkg").unwrap().is_workspace());
    }
    
    #[test]
    fn test_invalid_package_name() {
        let toml = r#"
[package]
name = ""
version = "1.0.0"
"#;
        
        assert!(parse_pea_toml(toml).is_err());
    }
    
    #[test]
    fn test_invalid_version() {
        let toml = r#"
[package]
name = "test-package"
version = "invalid"
"#;
        
        assert!(parse_pea_toml(toml).is_err());
    }
    
    #[test]
    fn test_round_trip_serialization() {
        let toml = r#"
[package]
name = "test-package"
version = "1.0.0"
description = "A test package"

[dependencies]
lodash = "^4.17.21"
"#;
        
        let config = parse_pea_toml(toml).unwrap();
        let serialized = serialize_pea_toml(&config).unwrap();
        let reparsed = parse_pea_toml(&serialized).unwrap();
        
        assert_eq!(config, reparsed);
    }
    
    #[test]
    fn test_valid_package_names() {
        assert!(is_valid_package_name("my-package"));
        assert!(is_valid_package_name("my_package"));
        assert!(is_valid_package_name("@org/package"));
        assert!(is_valid_package_name("package123"));
        
        assert!(!is_valid_package_name(""));
        assert!(!is_valid_package_name("-invalid"));
        assert!(!is_valid_package_name("_invalid"));
        assert!(!is_valid_package_name("invalid space"));
    }
}

#[cfg(all(test, feature = "property-tests"))]
mod property_tests {
    use super::*;
    use proptest::prelude::*;
    use pea_core::types::Version;
    
    // Property test generators
    prop_compose! {
        fn arb_version()(major in 0u64..100, minor in 0u64..100, patch in 0u64..100) -> Version {
            Version::new(major, minor, patch)
        }
    }
    
    prop_compose! {
        fn arb_package_section()(
            name in "[a-z][a-z0-9-]{0,20}",
            version in arb_version(),
            description in prop::option::of("[a-zA-Z0-9 ]{0,100}"),
        ) -> PackageSection {
            PackageSection {
                name,
                version,
                description,
                main: None,
                license: None,
                repository: None,
                keywords: Vec::new(),
                authors: Vec::new(),
                homepage: None,
            }
        }
    }
    
    prop_compose! {
        fn arb_dependency_spec()(
            major in 0u64..100,
            minor in 0u64..100,
            patch in 0u64..100,
        ) -> DependencySpec {
            DependencySpec::Simple(format!("^{}.{}.{}", major, minor, patch))
        }
    }
    
    prop_compose! {
        fn arb_pea_toml()(
            package in arb_package_section(),
            deps in prop::collection::hash_map("[a-z][a-z0-9-]{0,10}", arb_dependency_spec(), 0..5),
        ) -> PeaToml {
            PeaToml {
                package,
                dependencies: deps,
                dev_dependencies: HashMap::new(),
                peer_dependencies: HashMap::new(),
                optional_dependencies: HashMap::new(),
                scripts: HashMap::new(),
                features: HashMap::new(),
                workspace: None,
                profile: HashMap::new(),
            }
        }
    }
    
    proptest! {
        /// Property 1: TOML Configuration Round-Trip
        /// Test serialize(parse(serialize(config))) == serialize(config)
        #[test]
        fn toml_round_trip(config in arb_pea_toml()) {
            let serialized1 = serialize_pea_toml(&config).unwrap();
            let parsed = parse_pea_toml(&serialized1).unwrap();
            let serialized2 = serialize_pea_toml(&parsed).unwrap();
            
            // Parse both serialized versions to compare semantically
            let config1 = parse_pea_toml(&serialized1).unwrap();
            let config2 = parse_pea_toml(&serialized2).unwrap();
            
            prop_assert_eq!(config1, config2);
        }
    }
}
/// Custom deserializer for Version from string
fn deserialize_version<'de, D>(deserializer: D) -> Result<Version, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    Version::from_str(&s).map_err(serde::de::Error::custom)
}
/// Custom serializer for Version to string
fn serialize_version<S>(version: &Version, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    serializer.serialize_str(&version.to_string())
}