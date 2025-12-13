//! Semantic version resolution and constraint satisfaction
//!
//! Provides advanced semver operations for dependency resolution including
//! version selection, constraint satisfaction, and compatibility checking.

use std::collections::BTreeSet;

use pea_core::types::{Version, VersionReq};


/// Version selector for finding best matching versions
#[derive(Debug, Clone)]
pub struct VersionSelector {
    /// Available versions sorted in descending order
    available_versions: BTreeSet<Version>,
}

/// Version constraint satisfaction checker
#[derive(Debug, Clone)]
pub struct ConstraintSolver {
    /// Active version constraints
    constraints: Vec<VersionReq>,
}

impl VersionSelector {
    /// Create new version selector with available versions
    pub fn new(versions: Vec<Version>) -> Self {
        let available_versions = versions.into_iter().collect();
        Self { available_versions }
    }

    /// Select highest version matching all constraints
    pub fn select_best(&self, constraints: &[VersionReq]) -> Option<Version> {
        // Find highest version that satisfies all constraints
        self.available_versions
            .iter()
            .rev() // Start with highest versions
            .find(|version| {
                constraints.iter().all(|req| req.matches(version))
            })
            .cloned()
    }

    /// Select highest stable version (no prerelease) matching constraints
    pub fn select_best_stable(&self, constraints: &[VersionReq]) -> Option<Version> {
        self.available_versions
            .iter()
            .rev()
            .filter(|version| !version.is_prerelease())
            .find(|version| {
                constraints.iter().all(|req| req.matches(version))
            })
            .cloned()
    }

    /// Select version with preference for stability
    pub fn select_preferred(&self, constraints: &[VersionReq], allow_prerelease: bool) -> Option<Version> {
        if allow_prerelease {
            self.select_best(constraints)
        } else {
            // Try stable first, fall back to prerelease if no stable version available
            self.select_best_stable(constraints)
                .or_else(|| self.select_best(constraints))
        }
    }

    /// Find all versions matching constraints
    pub fn find_matching(&self, constraint: &VersionReq) -> Vec<Version> {
        self.available_versions
            .iter()
            .filter(|version| constraint.matches(version))
            .cloned()
            .collect()
    }

    /// Find all stable versions matching constraints
    pub fn find_matching_stable(&self, constraint: &VersionReq) -> Vec<Version> {
        self.available_versions
            .iter()
            .filter(|version| !version.is_prerelease() && constraint.matches(version))
            .cloned()
            .collect()
    }

    /// Get the highest available version
    pub fn highest_version(&self) -> Option<&Version> {
        self.available_versions.iter().rev().next()
    }

    /// Get the lowest available version
    pub fn lowest_version(&self) -> Option<&Version> {
        self.available_versions.iter().next()
    }

    /// Check if any version satisfies the constraints
    pub fn has_matching(&self, constraints: &[VersionReq]) -> bool {
        self.available_versions
            .iter()
            .any(|version| {
                constraints.iter().all(|req| req.matches(version))
            })
    }
}

impl ConstraintSolver {
    /// Create new constraint solver
    pub fn new() -> Self {
        Self {
            constraints: Vec::new(),
        }
    }

    /// Add version constraint
    pub fn add_constraint(&mut self, constraint: VersionReq) {
        self.constraints.push(constraint);
    }

    /// Check if constraints are satisfiable
    pub fn is_satisfiable(&self, available_versions: &[Version]) -> bool {
        available_versions.iter().any(|version| {
            self.constraints.iter().all(|req| req.matches(version))
        })
    }
}

impl Default for ConstraintSolver {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    fn create_versions() -> Vec<Version> {
        vec![
            Version::from_str("1.0.0").unwrap(),
            Version::from_str("1.1.0").unwrap(),
            Version::from_str("1.2.0").unwrap(),
            Version::from_str("2.0.0-alpha.1").unwrap(),
            Version::from_str("2.0.0").unwrap(),
            Version::from_str("2.1.0").unwrap(),
        ]
    }

    #[test]
    fn test_version_selector_creation() {
        let versions = create_versions();
        let selector = VersionSelector::new(versions.clone());
        
        assert_eq!(selector.available_versions.len(), 6);
        assert!(selector.available_versions.contains(&versions[0]));
    }

    #[test]
    fn test_select_best() {
        let versions = create_versions();
        let selector = VersionSelector::new(versions);
        
        let req = VersionReq::parse("^1.0.0").unwrap();
        let selected = selector.select_best(&[req]).unwrap();
        
        // Should select highest 1.x version
        assert_eq!(selected, Version::from_str("1.2.0").unwrap());
    }

    #[test]
    fn test_select_best_stable() {
        let versions = create_versions();
        let selector = VersionSelector::new(versions);
        
        let req = VersionReq::parse(">=2.0.0").unwrap();
        let selected = selector.select_best_stable(&[req]).unwrap();
        
        // Should select 2.1.0, not 2.0.0-alpha.1
        assert_eq!(selected, Version::from_str("2.1.0").unwrap());
    }

    #[test]
    fn test_select_preferred_stable() {
        let versions = create_versions();
        let selector = VersionSelector::new(versions);
        
        let req = VersionReq::parse(">=2.0.0").unwrap();
        let selected = selector.select_preferred(&[req], false).unwrap();
        
        // Should prefer stable version
        assert_eq!(selected, Version::from_str("2.1.0").unwrap());
    }

    #[test]
    fn test_select_preferred_allow_prerelease() {
        let versions = vec![
            Version::from_str("1.0.0").unwrap(),
            Version::from_str("2.0.0-beta.1").unwrap(),
        ];
        let selector = VersionSelector::new(versions);
        
        let req = VersionReq::parse(">=2.0.0-alpha").unwrap();
        let selected = selector.select_preferred(&[req], true).unwrap();
        
        // Should select prerelease when allowed
        assert_eq!(selected, Version::from_str("2.0.0-beta.1").unwrap());
    }

    #[test]
    fn test_find_matching() {
        let versions = create_versions();
        let selector = VersionSelector::new(versions);
        
        let req = VersionReq::parse("^1.0.0").unwrap();
        let matching = selector.find_matching(&req);
        
        assert_eq!(matching.len(), 3); // 1.0.0, 1.1.0, 1.2.0
        assert!(matching.contains(&Version::from_str("1.0.0").unwrap()));
        assert!(matching.contains(&Version::from_str("1.1.0").unwrap()));
        assert!(matching.contains(&Version::from_str("1.2.0").unwrap()));
    }

    #[test]
    fn test_find_matching_stable() {
        let versions = create_versions();
        let selector = VersionSelector::new(versions);
        
        let req = VersionReq::parse(">=2.0.0").unwrap();
        let matching = selector.find_matching_stable(&req);
        
        assert_eq!(matching.len(), 2); // 2.0.0, 2.1.0 (not 2.0.0-alpha.1)
        assert!(matching.contains(&Version::from_str("2.0.0").unwrap()));
        assert!(matching.contains(&Version::from_str("2.1.0").unwrap()));
        assert!(!matching.contains(&Version::from_str("2.0.0-alpha.1").unwrap()));
    }

    #[test]
    fn test_highest_lowest_version() {
        let versions = create_versions();
        let selector = VersionSelector::new(versions);
        
        assert_eq!(selector.highest_version(), Some(&Version::from_str("2.1.0").unwrap()));
        assert_eq!(selector.lowest_version(), Some(&Version::from_str("1.0.0").unwrap()));
    }

    #[test]
    fn test_has_matching() {
        let versions = create_versions();
        let selector = VersionSelector::new(versions);
        
        let req1 = VersionReq::parse("^1.0.0").unwrap();
        assert!(selector.has_matching(&[req1]));
        
        let req2 = VersionReq::parse("^3.0.0").unwrap();
        assert!(!selector.has_matching(&[req2]));
    }

    #[test]
    fn test_constraint_solver() {
        let mut solver = ConstraintSolver::new();
        
        solver.add_constraint(VersionReq::parse(">=1.0.0").unwrap());
        solver.add_constraint(VersionReq::parse("<2.0.0").unwrap());
        
        let versions = create_versions();
        assert!(solver.is_satisfiable(&versions));
        
        // Add conflicting constraint
        solver.add_constraint(VersionReq::parse(">=3.0.0").unwrap());
        assert!(!solver.is_satisfiable(&versions));
    }
}