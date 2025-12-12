//! Semantic version types with rkyv support.
//!
//! Provides Version and VersionReq types that follow the semantic versioning
//! specification with zero-copy serialization support.

use rkyv::{Archive, Deserialize, Serialize};
use serde::{Deserialize as SerdeDeserialize, Serialize as SerdeSerialize};
use std::cmp::Ordering;
use std::fmt;
use std::str::FromStr;
use thiserror::Error;

/// Semantic version (major.minor.patch-prerelease+build)
#[derive(
    Debug, Clone, PartialEq, Eq, Archive, Deserialize, Serialize, SerdeDeserialize, SerdeSerialize,
)]
#[archive(check_bytes)]
pub struct Version {
    pub major: u64,
    pub minor: u64,
    pub patch: u64,
    pub prerelease: Option<String>,
    pub build: Option<String>,
}

/// Version requirement (^1.0.0, ~2.3.0, >=1.0.0 <2.0.0)
#[derive(Debug, Clone, PartialEq, Eq, Archive, Deserialize, Serialize)]
#[archive(check_bytes)]
pub struct VersionReq {
    pub comparators: Vec<Comparator>,
}

/// Individual version comparator
#[derive(Debug, Clone, PartialEq, Eq, Archive, Deserialize, Serialize)]
pub struct Comparator {
    pub op: Op,
    pub version: PartialVersion,
}

/// Comparison operator for version requirements
#[derive(Debug, Clone, Copy, PartialEq, Eq, Archive, Deserialize, Serialize)]
pub enum Op {
    Exact,     // =1.0.0
    Greater,   // >1.0.0
    GreaterEq, // >=1.0.0
    Less,      // <1.0.0
    LessEq,    // <=1.0.0
    Tilde,     // ~1.0.0
    Caret,     // ^1.0.0
    Wildcard,  // *
}

/// Partial version for comparisons (may have missing components)
#[derive(Debug, Clone, PartialEq, Eq, Archive, Deserialize, Serialize)]
pub struct PartialVersion {
    pub major: u64,
    pub minor: Option<u64>,
    pub patch: Option<u64>,
    pub prerelease: Option<String>,
}

/// Version parsing and validation errors
#[derive(Error, Debug)]
pub enum VersionError {
    #[error("Invalid version format: {input}")]
    InvalidFormat { input: String },

    #[error("Invalid number in version: {component}")]
    InvalidNumber { component: String },

    #[error("Invalid prerelease identifier: {prerelease}")]
    InvalidPrerelease { prerelease: String },

    #[error("Invalid build metadata: {build}")]
    InvalidBuild { build: String },
}
impl Version {
    /// Create a new version
    pub fn new(major: u64, minor: u64, patch: u64) -> Self {
        Self {
            major,
            minor,
            patch,
            prerelease: None,
            build: None,
        }
    }

    /// Check if this version satisfies a version requirement
    pub fn satisfies(&self, req: &VersionReq) -> bool {
        req.matches(self)
    }

    /// Check if this is a prerelease version
    pub fn is_prerelease(&self) -> bool {
        self.prerelease.is_some()
    }

    /// Get the precedence for comparison (ignores build metadata)
    fn precedence_cmp(&self, other: &Self) -> Ordering {
        match (self.major, self.minor, self.patch).cmp(&(other.major, other.minor, other.patch)) {
            Ordering::Equal => {
                match (&self.prerelease, &other.prerelease) {
                    (None, None) => Ordering::Equal,
                    (Some(_), None) => Ordering::Less, // prerelease < normal
                    (None, Some(_)) => Ordering::Greater, // normal > prerelease
                    (Some(a), Some(b)) => a.cmp(b),    // lexical comparison
                }
            },
            other => other,
        }
    }
}
impl FromStr for Version {
    type Err = VersionError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let input = s.trim();

        // Split on '+' for build metadata
        let (version_part, build) = match input.split_once('+') {
            Some((v, b)) => (v, Some(b.to_string())),
            None => (input, None),
        };

        // Split on '-' for prerelease
        let (core_part, prerelease) = match version_part.split_once('-') {
            Some((c, p)) => (c, Some(p.to_string())),
            None => (version_part, None),
        };

        // Parse major.minor.patch
        let parts: Vec<&str> = core_part.split('.').collect();
        if parts.len() != 3 {
            return Err(VersionError::InvalidFormat {
                input: input.to_string(),
            });
        }

        let major = parts[0].parse().map_err(|_| VersionError::InvalidNumber {
            component: parts[0].to_string(),
        })?;
        let minor = parts[1].parse().map_err(|_| VersionError::InvalidNumber {
            component: parts[1].to_string(),
        })?;
        let patch = parts[2].parse().map_err(|_| VersionError::InvalidNumber {
            component: parts[2].to_string(),
        })?;

        Ok(Version {
            major,
            minor,
            patch,
            prerelease,
            build,
        })
    }
}

impl fmt::Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)?;

        if let Some(ref pre) = self.prerelease {
            write!(f, "-{}", pre)?;
        }

        if let Some(ref build) = self.build {
            write!(f, "+{}", build)?;
        }

        Ok(())
    }
}
impl PartialOrd for Version {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Version {
    fn cmp(&self, other: &Self) -> Ordering {
        self.precedence_cmp(other)
    }
}

impl VersionReq {
    /// Parse a version requirement string
    pub fn parse(input: &str) -> Result<Self, VersionError> {
        let input = input.trim();

        if input == "*" {
            return Ok(VersionReq {
                comparators: vec![Comparator {
                    op: Op::Wildcard,
                    version: PartialVersion {
                        major: 0,
                        minor: None,
                        patch: None,
                        prerelease: None,
                    },
                }],
            });
        }

        // Parse operator prefix
        let (op, version_str) = if let Some(stripped) = input.strip_prefix("^") {
            (Op::Caret, stripped)
        } else if let Some(stripped) = input.strip_prefix("~") {
            (Op::Tilde, stripped)
        } else if let Some(stripped) = input.strip_prefix(">=") {
            (Op::GreaterEq, stripped)
        } else if let Some(stripped) = input.strip_prefix("<=") {
            (Op::LessEq, stripped)
        } else if let Some(stripped) = input.strip_prefix(">") {
            (Op::Greater, stripped)
        } else if let Some(stripped) = input.strip_prefix("<") {
            (Op::Less, stripped)
        } else if let Some(stripped) = input.strip_prefix("=") {
            (Op::Exact, stripped)
        } else {
            (Op::Exact, input)
        };

        // Parse the version part
        let version = Version::from_str(version_str)?;
        Ok(VersionReq {
            comparators: vec![Comparator {
                op,
                version: PartialVersion {
                    major: version.major,
                    minor: Some(version.minor),
                    patch: Some(version.patch),
                    prerelease: version.prerelease,
                },
            }],
        })
    }

    /// Check if a version matches this requirement
    pub fn matches(&self, version: &Version) -> bool {
        self.comparators.iter().all(|comp| comp.matches(version))
    }
}
impl Comparator {
    /// Check if a version matches this comparator
    pub fn matches(&self, version: &Version) -> bool {
        match self.op {
            Op::Exact => self.version.matches_exact(version),
            Op::Wildcard => true,
            Op::Greater => version > &self.version.to_version(),
            Op::GreaterEq => version >= &self.version.to_version(),
            Op::Less => version < &self.version.to_version(),
            Op::LessEq => version <= &self.version.to_version(),
            Op::Tilde => self.version.matches_tilde(version),
            Op::Caret => self.version.matches_caret(version),
        }
    }
}

impl PartialVersion {
    /// Convert to a full version (filling missing parts with 0)
    pub fn to_version(&self) -> Version {
        Version {
            major: self.major,
            minor: self.minor.unwrap_or(0),
            patch: self.patch.unwrap_or(0),
            prerelease: self.prerelease.clone(),
            build: None,
        }
    }

    /// Check exact match
    fn matches_exact(&self, version: &Version) -> bool {
        version.major == self.major
            && self.minor.map_or(true, |m| version.minor == m)
            && self.patch.map_or(true, |p| version.patch == p)
            && version.prerelease == self.prerelease
    }

    /// Check tilde match (~1.2.3 allows >=1.2.3 <1.3.0)
    fn matches_tilde(&self, version: &Version) -> bool {
        if version.major != self.major {
            return false;
        }

        match self.minor {
            Some(minor) => version.minor == minor && version.patch >= self.patch.unwrap_or(0),
            None => true,
        }
    }

    /// Check caret match (^1.2.3 allows >=1.2.3 <2.0.0)
    fn matches_caret(&self, version: &Version) -> bool {
        if version.major != self.major {
            return false;
        }

        let base_version = self.to_version();
        version >= &base_version
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_parsing() {
        let v = Version::from_str("1.2.3").unwrap();
        assert_eq!(v.major, 1);
        assert_eq!(v.minor, 2);
        assert_eq!(v.patch, 3);
        assert_eq!(v.prerelease, None);
        assert_eq!(v.build, None);
    }

    #[test]
    fn test_version_with_prerelease() {
        let v = Version::from_str("1.2.3-alpha.1").unwrap();
        assert_eq!(v.major, 1);
        assert_eq!(v.minor, 2);
        assert_eq!(v.patch, 3);
        assert_eq!(v.prerelease, Some("alpha.1".to_string()));
        assert_eq!(v.build, None);
    }

    #[test]
    fn test_version_with_build() {
        let v = Version::from_str("1.2.3+build.1").unwrap();
        assert_eq!(v.major, 1);
        assert_eq!(v.minor, 2);
        assert_eq!(v.patch, 3);
        assert_eq!(v.prerelease, None);
        assert_eq!(v.build, Some("build.1".to_string()));
    }

    #[test]
    fn test_version_display() {
        let v = Version::new(1, 2, 3);
        assert_eq!(v.to_string(), "1.2.3");

        let v = Version {
            major: 1,
            minor: 2,
            patch: 3,
            prerelease: Some("alpha".to_string()),
            build: Some("build".to_string()),
        };
        assert_eq!(v.to_string(), "1.2.3-alpha+build");
    }

    #[test]
    fn test_version_comparison() {
        let v1 = Version::new(1, 0, 0);
        let v2 = Version::new(2, 0, 0);
        let v3 = Version::new(1, 1, 0);

        assert!(v1 < v2);
        assert!(v1 < v3);
        assert!(v3 < v2);
    }

    #[test]
    fn test_version_req_exact() {
        let req = VersionReq::parse("1.2.3").unwrap();
        let v1 = Version::new(1, 2, 3);
        let v2 = Version::new(1, 2, 4);

        assert!(req.matches(&v1));
        assert!(!req.matches(&v2));
    }

    #[test]
    fn test_version_req_wildcard() {
        let req = VersionReq::parse("*").unwrap();
        let v1 = Version::new(1, 2, 3);
        let v2 = Version::new(999, 999, 999);

        assert!(req.matches(&v1));
        assert!(req.matches(&v2));
    }
}
#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    // Property 4: Semver Version Round-Trip
    proptest! {
        #[test]
        fn version_round_trip(
            major in 0u64..1000,
            minor in 0u64..1000,
            patch in 0u64..1000,
            prerelease in prop::option::of("[a-zA-Z0-9.-]+"),
            build in prop::option::of("[a-zA-Z0-9.-]+")
        ) {
            let original = Version {
                major,
                minor,
                patch,
                prerelease: prerelease.clone(),
                build: build.clone(),
            };

            let serialized = original.to_string();
            let parsed = Version::from_str(&serialized).unwrap();

            // Build metadata should be preserved in parsing
            prop_assert_eq!(parsed.major, original.major);
            prop_assert_eq!(parsed.minor, original.minor);
            prop_assert_eq!(parsed.patch, original.patch);
            prop_assert_eq!(parsed.prerelease, original.prerelease);
            prop_assert_eq!(parsed.build, original.build);
        }
    }

    // Property 6: Semver Comparison Transitivity
    proptest! {
        #[test]
        fn version_comparison_transitivity(
            a_major in 0u64..100,
            a_minor in 0u64..100,
            a_patch in 0u64..100,
            b_major in 0u64..100,
            b_minor in 0u64..100,
            b_patch in 0u64..100,
            c_major in 0u64..100,
            c_minor in 0u64..100,
            c_patch in 0u64..100,
        ) {
            let a = Version::new(a_major, a_minor, a_patch);
            let b = Version::new(b_major, b_minor, b_patch);
            let c = Version::new(c_major, c_minor, c_patch);

            // If a < b and b < c, then a < c
            if a < b && b < c {
                prop_assert!(a < c, "Transitivity violated: {} < {} < {} but {} >= {}", a, b, c, a, c);
            }

            // If a > b and b > c, then a > c
            if a > b && b > c {
                prop_assert!(a > c, "Transitivity violated: {} > {} > {} but {} <= {}", a, b, c, a, c);
            }
        }
    }
}
#[test]
fn test_rkyv_serialization() {
    use rkyv::Deserialize;

    let version = Version::new(1, 2, 3);

    // Serialize
    let bytes = rkyv::to_bytes::<_, 256>(&version).unwrap();

    // Deserialize
    let archived = rkyv::check_archived_root::<Version>(&bytes[..]).unwrap();
    let deserialized: Version = archived.deserialize(&mut rkyv::Infallible).unwrap();

    assert_eq!(version, deserialized);
}
    #[test]
    fn test_version_req_caret() {
        let req = VersionReq::parse("^1.2.3").unwrap();
        
        // Should match compatible versions
        assert!(req.matches(&Version::new(1, 2, 3)));
        assert!(req.matches(&Version::new(1, 2, 4)));
        assert!(req.matches(&Version::new(1, 3, 0)));
        
        // Should not match incompatible versions
        assert!(!req.matches(&Version::new(2, 0, 0)));
        assert!(!req.matches(&Version::new(0, 9, 9)));
    }

    #[test]
    fn test_version_req_operators() {
        let v1_2_3 = Version::new(1, 2, 3);
        let v1_2_4 = Version::new(1, 2, 4);
        let v1_3_0 = Version::new(1, 3, 0);
        
        // Greater than
        let req = VersionReq::parse(">1.2.3").unwrap();
        assert!(!req.matches(&v1_2_3));
        assert!(req.matches(&v1_2_4));
        assert!(req.matches(&v1_3_0));
        
        // Greater than or equal
        let req = VersionReq::parse(">=1.2.3").unwrap();
        assert!(req.matches(&v1_2_3));
        assert!(req.matches(&v1_2_4));
        assert!(req.matches(&v1_3_0));
        
        // Less than
        let req = VersionReq::parse("<1.2.4").unwrap();
        assert!(req.matches(&v1_2_3));
        assert!(!req.matches(&v1_2_4));
        assert!(!req.matches(&v1_3_0));
    }