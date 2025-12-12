//! Dependency specification types.
//!
//! Defines structures for package dependencies with features and kinds.

use super::VersionReq;
use rkyv::{Archive, Deserialize, Serialize};

/// Dependency specification
#[derive(Debug, Clone, PartialEq, Eq, Archive, Deserialize, Serialize)]
#[archive(check_bytes)]
pub struct Dependency {
    pub name: String,
    pub version_req: VersionReq,
    pub kind: DependencyKind,
    pub features: Vec<String>,
    pub optional: bool,
}

/// Type of dependency
#[derive(Debug, Clone, Copy, PartialEq, Eq, Archive, Deserialize, Serialize)]
pub enum DependencyKind {
    /// Normal runtime dependency
    Normal,
    /// Development-only dependency
    Dev,
    /// Peer dependency (must be provided by consumer)
    Peer,
    /// Optional dependency (can be missing)
    Optional,
}

impl Dependency {
    /// Create a new normal dependency
    pub fn new(name: String, version_req: VersionReq) -> Self {
        Self {
            name,
            version_req,
            kind: DependencyKind::Normal,
            features: Vec::new(),
            optional: false,
        }
    }

    /// Create a development dependency
    pub fn dev(name: String, version_req: VersionReq) -> Self {
        Self {
            name,
            version_req,
            kind: DependencyKind::Dev,
            features: Vec::new(),
            optional: false,
        }
    }

    /// Add a feature to this dependency
    pub fn with_feature(mut self, feature: String) -> Self {
        self.features.push(feature);
        self
    }

    /// Make this dependency optional
    pub fn optional(mut self) -> Self {
        self.optional = true;
        self
    }
}

impl DependencyKind {
    /// Check if this dependency is needed at runtime
    pub fn is_runtime(&self) -> bool {
        matches!(self, DependencyKind::Normal | DependencyKind::Optional)
    }

    /// Check if this dependency is only for development
    pub fn is_dev_only(&self) -> bool {
        matches!(self, DependencyKind::Dev)
    }

    /// Check if this dependency must be provided by the consumer
    pub fn is_peer(&self) -> bool {
        matches!(self, DependencyKind::Peer)
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dependency_creation() {
        let version_req = VersionReq::parse("^1.0.0").unwrap();
        let dep = Dependency::new("lodash".to_string(), version_req.clone());

        assert_eq!(dep.name, "lodash");
        assert_eq!(dep.version_req, version_req);
        assert_eq!(dep.kind, DependencyKind::Normal);
        assert!(!dep.optional);
        assert!(dep.features.is_empty());
    }

    #[test]
    fn test_dev_dependency() {
        let version_req = VersionReq::parse("^2.0.0").unwrap();
        let dep = Dependency::dev("jest".to_string(), version_req);

        assert_eq!(dep.kind, DependencyKind::Dev);
        assert!(dep.kind.is_dev_only());
        assert!(!dep.kind.is_runtime());
    }

    #[test]
    fn test_dependency_with_features() {
        let version_req = VersionReq::parse("1.0.0").unwrap();
        let dep = Dependency::new("serde".to_string(), version_req)
            .with_feature("derive".to_string())
            .optional();

        assert!(dep.features.contains(&"derive".to_string()));
        assert!(dep.optional);
    }

    #[test]
    fn test_dependency_kinds() {
        assert!(DependencyKind::Normal.is_runtime());
        assert!(!DependencyKind::Normal.is_dev_only());
        assert!(!DependencyKind::Normal.is_peer());

        assert!(!DependencyKind::Dev.is_runtime());
        assert!(DependencyKind::Dev.is_dev_only());

        assert!(!DependencyKind::Peer.is_runtime());
        assert!(DependencyKind::Peer.is_peer());

        assert!(DependencyKind::Optional.is_runtime());
        assert!(!DependencyKind::Optional.is_dev_only());
    }
}
