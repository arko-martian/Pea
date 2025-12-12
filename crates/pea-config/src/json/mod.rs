//! package.json configuration parsing and serialization

use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use pea_core::error::PeaError;
use crate::{ConfigResult, toml::{PeaToml, PackageSection, DependencySpec}};

/// Complete package.json configuration
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PackageJson {
    /// Package name (required)
    pub name: String,
    
    /// Package version (required)
    pub version: String,
    
    /// Package description
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    
    /// Main entry point
    #[serde(skip_serializing_if = "Option::is_none")]
    pub main: Option<String>,
    
    /// License identifier
    #[serde(skip_serializing_if = "Option::is_none")]
    pub license: Option<String>,
    
    /// Repository information
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repository: Option<RepositoryInfo>,
    
    /// Keywords for discovery
    #[serde(default)]
    pub keywords: Vec<String>,
    
    /// Author information
    #[serde(skip_serializing_if = "Option::is_none")]
    pub author: Option<String>,
    
    /// Authors array
    #[serde(default)]
    pub authors: Vec<String>,
    
    /// Homepage URL
    #[serde(skip_serializing_if = "Option::is_none")]
    pub homepage: Option<String>,
    
    /// Runtime dependencies
    #[serde(default)]
    pub dependencies: HashMap<String, String>,
    
    /// Development dependencies
    #[serde(default, rename = "devDependencies")]
    pub dev_dependencies: HashMap<String, String>,
    
    /// Peer dependencies
    #[serde(default, rename = "peerDependencies")]
    pub peer_dependencies: HashMap<String, String>,
    
    /// Optional dependencies
    #[serde(default, rename = "optionalDependencies")]
    pub optional_dependencies: HashMap<String, String>,
    
    /// Bundled dependencies
    #[serde(default, rename = "bundledDependencies")]
    pub bundled_dependencies: Vec<String>,
    
    /// npm scripts
    #[serde(default)]
    pub scripts: HashMap<String, String>,
    
    /// Workspace configuration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub workspaces: Option<WorkspacesConfig>,
    
    /// Engine requirements
    #[serde(default)]
    pub engines: HashMap<String, String>,
    
    /// OS requirements
    #[serde(default)]
    pub os: Vec<String>,
    
    /// CPU requirements
    #[serde(default)]
    pub cpu: Vec<String>,
    
    /// Private flag
    #[serde(default)]
    pub private: bool,
    
    /// Files to include in package
    #[serde(default)]
    pub files: Vec<String>,
    
    /// Binary executables
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bin: Option<BinConfig>,
    
    /// Module type (commonjs or module)
    #[serde(skip_serializing_if = "Option::is_none", rename = "type")]
    pub module_type: Option<String>,
    
    /// ES module exports
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exports: Option<serde_json::Value>,
    
    /// TypeScript types entry point
    #[serde(skip_serializing_if = "Option::is_none")]
    pub types: Option<String>,
    
    /// TypeScript typings entry point
    #[serde(skip_serializing_if = "Option::is_none")]
    pub typings: Option<String>,
}

/// Repository information
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum RepositoryInfo {
    /// Simple URL string
    Simple(String),
    /// Detailed repository object
    Detailed {
        #[serde(rename = "type")]
        repo_type: String,
        url: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        directory: Option<String>,
    },
}

/// Workspace configuration
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum WorkspacesConfig {
    /// Simple array of workspace paths
    Simple(Vec<String>),
    /// Detailed workspace configuration
    Detailed {
        packages: Vec<String>,
        #[serde(default)]
        nohoist: Vec<String>,
    },
}

/// Binary configuration
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum BinConfig {
    /// Single binary (package name as binary name)
    Simple(String),
    /// Multiple binaries
    Multiple(HashMap<String, String>),
}

/// Parse JSON string to PackageJson configuration
pub fn parse_package_json(content: &str) -> ConfigResult<PackageJson> {
    serde_json::from_str(content)
        .map_err(|e| PeaError::JsonParse {
            message: format!("JSON parsing error: {}", e),
        })
}

/// Serialize PackageJson to JSON string
pub fn serialize_package_json(config: &PackageJson) -> ConfigResult<String> {
    serde_json::to_string_pretty(config)
        .map_err(|e| PeaError::JsonParse {
            message: format!("JSON serialization error: {}", e),
        })
}

/// Convert PackageJson to PeaToml configuration
pub fn import_to_pea_toml(package_json: &PackageJson) -> ConfigResult<PeaToml> {
    // Parse version
    let version = package_json.version.parse()
        .map_err(|e| PeaError::ConfigValidation {
            field: "version".to_string(),
            reason: format!("Invalid version '{}': {}", package_json.version, e),
        })?;
    
    // Create package section
    let package = PackageSection {
        name: package_json.name.clone(),
        version,
        description: package_json.description.clone(),
        main: package_json.main.clone(),
        license: package_json.license.clone(),
        repository: extract_repository_url(&package_json.repository),
        keywords: package_json.keywords.clone(),
        authors: extract_authors(package_json),
        homepage: package_json.homepage.clone(),
    };
    
    // Convert dependencies
    let dependencies = convert_dependencies(&package_json.dependencies)?;
    let dev_dependencies = convert_dependencies(&package_json.dev_dependencies)?;
    let peer_dependencies = convert_dependencies(&package_json.peer_dependencies)?;
    let optional_dependencies = convert_dependencies(&package_json.optional_dependencies)?;
    
    // Convert workspace configuration
    let workspace = package_json.workspaces.as_ref().map(|ws| {
        let members = match ws {
            WorkspacesConfig::Simple(packages) => packages.clone(),
            WorkspacesConfig::Detailed { packages, .. } => packages.clone(),
        };
        
        crate::toml::WorkspaceSection {
            members,
            exclude: Vec::new(),
            dependencies: HashMap::new(),
        }
    });
    
    Ok(PeaToml {
        package,
        dependencies,
        dev_dependencies,
        peer_dependencies,
        optional_dependencies,
        scripts: package_json.scripts.clone(),
        features: HashMap::new(),
        workspace,
        profile: HashMap::new(),
    })
}

/// Load and parse package.json from file path
pub async fn load_from_file(path: &camino::Utf8Path) -> ConfigResult<PackageJson> {
    let content = tokio::fs::read_to_string(path).await
        .map_err(|e| PeaError::io(format!("Failed to read {}", path), e))?;
    
    parse_package_json(&content)
        .map_err(|e| match e {
            PeaError::JsonParse { message } => PeaError::JsonParse {
                message: format!("In file {}: {}", path, message),
            },
            other => other,
        })
}

/// Convert npm dependency map to Pea dependency specs
fn convert_dependencies(deps: &HashMap<String, String>) -> ConfigResult<HashMap<String, DependencySpec>> {
    let mut result = HashMap::new();
    
    for (name, version) in deps {
        // Handle special npm version formats
        let spec = if version.starts_with("file:") {
            // Local file dependency
            let path = version.strip_prefix("file:").unwrap().to_string();
            DependencySpec::Detailed {
                version: None,
                git: None,
                branch: None,
                tag: None,
                rev: None,
                path: Some(path),
                workspace: None,
                features: Vec::new(),
                optional: false,
                default_features: true,
            }
        } else if version.starts_with("git+") || version.contains("github.com") {
            // Git dependency
            let git_url = if version.starts_with("git+") {
                version.strip_prefix("git+").unwrap().to_string()
            } else {
                version.clone()
            };
            
            DependencySpec::Detailed {
                version: None,
                git: Some(git_url),
                branch: None,
                tag: None,
                rev: None,
                path: None,
                workspace: None,
                features: Vec::new(),
                optional: false,
                default_features: true,
            }
        } else {
            // Regular version dependency
            DependencySpec::Simple(version.clone())
        };
        
        result.insert(name.clone(), spec);
    }
    
    Ok(result)
}

/// Extract repository URL from repository info
fn extract_repository_url(repo: &Option<RepositoryInfo>) -> Option<String> {
    match repo {
        Some(RepositoryInfo::Simple(url)) => Some(url.clone()),
        Some(RepositoryInfo::Detailed { url, .. }) => Some(url.clone()),
        None => None,
    }
}

/// Extract authors from package.json
fn extract_authors(package_json: &PackageJson) -> Vec<String> {
    let mut authors = package_json.authors.clone();
    
    if let Some(author) = &package_json.author {
        if !authors.contains(author) {
            authors.push(author.clone());
        }
    }
    
    authors
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_parse_minimal_package_json() {
        let json = r#"
{
  "name": "test-package",
  "version": "1.0.0"
}
"#;
        
        let config = parse_package_json(json).unwrap();
        assert_eq!(config.name, "test-package");
        assert_eq!(config.version, "1.0.0");
        assert!(config.dependencies.is_empty());
    }
    
    #[test]
    fn test_parse_with_dependencies() {
        let json = r#"
{
  "name": "test-package",
  "version": "1.0.0",
  "dependencies": {
    "lodash": "^4.17.21",
    "react": "^18.0.0"
  },
  "devDependencies": {
    "typescript": "^4.9.0"
  }
}
"#;
        
        let config = parse_package_json(json).unwrap();
        assert_eq!(config.dependencies.len(), 2);
        assert_eq!(config.dev_dependencies.len(), 1);
        assert_eq!(config.dependencies.get("lodash").unwrap(), "^4.17.21");
    }
    
    #[test]
    fn test_import_to_pea_toml() {
        let json = r#"
{
  "name": "test-package",
  "version": "1.0.0",
  "description": "A test package",
  "dependencies": {
    "lodash": "^4.17.21"
  },
  "scripts": {
    "build": "tsc",
    "test": "jest"
  }
}
"#;
        
        let package_json = parse_package_json(json).unwrap();
        let pea_toml = import_to_pea_toml(&package_json).unwrap();
        
        assert_eq!(pea_toml.package.name, "test-package");
        assert_eq!(pea_toml.package.version.to_string(), "1.0.0");
        assert_eq!(pea_toml.package.description, Some("A test package".to_string()));
        assert_eq!(pea_toml.dependencies.len(), 1);
        assert_eq!(pea_toml.scripts.len(), 2);
    }
    
    #[test]
    fn test_round_trip_serialization() {
        let json = r#"
{
  "name": "test-package",
  "version": "1.0.0",
  "dependencies": {
    "lodash": "^4.17.21"
  }
}
"#;
        
        let config = parse_package_json(json).unwrap();
        let serialized = serialize_package_json(&config).unwrap();
        let reparsed = parse_package_json(&serialized).unwrap();
        
        assert_eq!(config.name, reparsed.name);
        assert_eq!(config.version, reparsed.version);
        assert_eq!(config.dependencies, reparsed.dependencies);
    }
    
    #[test]
    fn test_convert_file_dependency() {
        let deps = HashMap::from([
            ("local-pkg".to_string(), "file:../local-pkg".to_string()),
        ]);
        
        let converted = convert_dependencies(&deps).unwrap();
        let spec = converted.get("local-pkg").unwrap();
        
        assert!(spec.is_path());
    }
    
    #[test]
    fn test_convert_git_dependency() {
        let deps = HashMap::from([
            ("git-pkg".to_string(), "git+https://github.com/user/repo.git".to_string()),
        ]);
        
        let converted = convert_dependencies(&deps).unwrap();
        let spec = converted.get("git-pkg").unwrap();
        
        assert!(spec.is_git());
    }
}
#[cfg(all(test, feature = "property-tests"))]
mod property_tests {
    use super::*;
    use proptest::prelude::*;
    
    // Property test generators
    prop_compose! {
        fn arb_package_json()(
            name in "[a-z][a-z0-9-]{0,20}",
            version in "[0-9]+\\.[0-9]+\\.[0-9]+",
            description in prop::option::of("[a-zA-Z0-9 ]{0,100}"),
            deps in prop::collection::hash_map("[a-z][a-z0-9-]{0,10}", "\\^[0-9]+\\.[0-9]+\\.[0-9]+", 0..5),
        ) -> PackageJson {
            PackageJson {
                name,
                version,
                description,
                main: None,
                license: None,
                repository: None,
                keywords: Vec::new(),
                author: None,
                authors: Vec::new(),
                homepage: None,
                dependencies: deps,
                dev_dependencies: HashMap::new(),
                peer_dependencies: HashMap::new(),
                optional_dependencies: HashMap::new(),
                bundled_dependencies: Vec::new(),
                scripts: HashMap::new(),
                workspaces: None,
                engines: HashMap::new(),
                os: Vec::new(),
                cpu: Vec::new(),
                private: false,
                files: Vec::new(),
                bin: None,
                module_type: None,
                exports: None,
                types: None,
                typings: None,
            }
        }
    }
    
    proptest! {
        /// Property 2: JSON Package Configuration Round-Trip
        /// Test parse(serialize(pkg)) == pkg
        #[test]
        fn json_round_trip(pkg in arb_package_json()) {
            let serialized = serialize_package_json(&pkg).unwrap();
            let parsed = parse_package_json(&serialized).unwrap();
            
            prop_assert_eq!(pkg.name, parsed.name);
            prop_assert_eq!(pkg.version, parsed.version);
            prop_assert_eq!(pkg.description, parsed.description);
            prop_assert_eq!(pkg.dependencies, parsed.dependencies);
        }
    }
}