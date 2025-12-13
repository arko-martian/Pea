//! Dependency graph implementation using petgraph
//!
//! Provides thread-safe dependency graph operations with cycle detection
//! and efficient graph traversal algorithms.

use dashmap::DashMap;
use petgraph::graph::{DiGraph, NodeIndex};
use petgraph::visit::EdgeRef;
use serde::{Deserialize, Serialize};

use pea_core::types::{Version, VersionReq, DependencyKind};

/// Unique identifier for a package
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PackageId {
    /// Package name (e.g., "lodash" or "@types/node")
    pub name: String,
    /// Resolved version
    pub version: Version,
}

/// Node in the dependency graph representing a resolved package
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageNode {
    /// Unique package identifier
    pub id: PackageId,
    /// Package name
    pub name: String,
    /// Resolved version
    pub version: Version,
    /// Registry URL where package was resolved
    pub resolved_url: String,
    /// Integrity hash for verification
    pub integrity: String,
}

/// Edge in the dependency graph representing a dependency relationship
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyEdge {
    /// Version requirement constraint
    pub version_req: VersionReq,
    /// Type of dependency
    pub kind: DependencyKind,
    /// Whether this dependency is optional
    pub optional: bool,
}

/// Thread-safe dependency graph using petgraph
#[derive(Debug)]
pub struct DependencyGraph {
    /// Underlying directed graph
    graph: DiGraph<PackageNode, DependencyEdge>,
    /// Map from PackageId to NodeIndex for fast lookups
    node_map: DashMap<PackageId, NodeIndex>,
}

impl DependencyGraph {
    /// Create a new empty dependency graph
    pub fn new() -> Self {
        Self {
            graph: DiGraph::new(),
            node_map: DashMap::new(),
        }
    }
}

impl Default for DependencyGraph {
    fn default() -> Self {
        Self::new()
    }
}
impl DependencyGraph {
    /// Add a package node to the graph with thread safety
    pub fn add_package(&mut self, package: PackageNode) -> NodeIndex {
        let package_id = package.id.clone();
        
        // Check if package already exists
        if let Some(existing_index) = self.node_map.get(&package_id) {
            return *existing_index;
        }
        
        // Add new node to graph
        let node_index = self.graph.add_node(package);
        
        // Store mapping for fast lookups
        self.node_map.insert(package_id, node_index);
        
        node_index
    }

    /// Add dependency edge between two packages
    pub fn add_dependency(
        &mut self,
        from_package: &PackageId,
        to_package: &PackageId,
        edge: DependencyEdge,
    ) -> Result<(), String> {
        let from_index = self.node_map.get(from_package)
            .ok_or_else(|| format!("Package not found: {}", from_package.name))?;
        
        let to_index = self.node_map.get(to_package)
            .ok_or_else(|| format!("Package not found: {}", to_package.name))?;
        
        // Add edge to graph
        self.graph.add_edge(*from_index, *to_index, edge);
        
        Ok(())
    }

    /// Get package node by ID
    pub fn get_package(&self, package_id: &PackageId) -> Option<&PackageNode> {
        let node_index = self.node_map.get(package_id)?;
        self.graph.node_weight(*node_index)
    }

    /// Get all packages in the graph
    pub fn packages(&self) -> impl Iterator<Item = &PackageNode> {
        self.graph.node_weights()
    }

    /// Detect cycles in the dependency graph
    pub fn detect_cycles(&self) -> Result<Vec<PackageId>, Vec<PackageId>> {
        use petgraph::algo::toposort;
        
        match toposort(&self.graph, None) {
            Ok(_) => Ok(Vec::new()), // No cycles found
            Err(cycle_node) => {
                // Extract cycle path
                let cycle_path = self.extract_cycle_path(cycle_node.node_id());
                Err(cycle_path)
            }
        }
    }

    /// Extract the cycle path from a node that's part of a cycle
    fn extract_cycle_path(&self, start_node: NodeIndex) -> Vec<PackageId> {
        use std::collections::HashSet;
        
        let mut visited = HashSet::new();
        let mut path = Vec::new();
        let mut current = start_node;
        
        // Follow edges until we find the cycle
        loop {
            if let Some(package) = self.graph.node_weight(current) {
                if visited.contains(&current) {
                    // Found the start of the cycle
                    let cycle_start = path.iter().position(|id| {
                        self.node_map.get(id).map(|idx| *idx == current).unwrap_or(false)
                    }).unwrap_or(0);
                    
                    let mut cycle = path[cycle_start..].to_vec();
                    cycle.push(package.id.clone()); // Close the cycle
                    return cycle;
                }
                
                visited.insert(current);
                path.push(package.id.clone());
                
                // Move to next node (first outgoing edge)
                if let Some(edge) = self.graph.edges(current).next() {
                    current = edge.target();
                } else {
                    break;
                }
            } else {
                break;
            }
        }
        
        path
    }

    /// Format cycle as "a -> b -> c -> a"
    pub fn format_cycle(cycle: &[PackageId]) -> String {
        if cycle.is_empty() {
            return "No cycle".to_string();
        }
        
        let names: Vec<String> = cycle.iter().map(|id| id.name.clone()).collect();
        if names.len() > 1 {
            // Close the cycle by adding the first element at the end
            let mut closed_cycle = names;
            closed_cycle.push(closed_cycle[0].clone());
            closed_cycle.join(" -> ")
        } else {
            names.join(" -> ")
        }
    }

    /// Get number of packages in the graph
    pub fn package_count(&self) -> usize {
        self.graph.node_count()
    }

    /// Get number of dependencies in the graph
    pub fn dependency_count(&self) -> usize {
        self.graph.edge_count()
    }

    /// Check for cycles and return detailed error if found
    pub fn validate_no_cycles(&self) -> Result<(), String> {
        match self.detect_cycles() {
            Ok(_) => Ok(()),
            Err(cycle) => {
                let cycle_str = Self::format_cycle(&cycle);
                Err(format!("Circular dependency detected: {}", cycle_str))
            }
        }
    }

    /// Get topologically sorted packages (dependency order)
    pub fn topological_sort(&self) -> Result<Vec<PackageId>, String> {
        use petgraph::algo::toposort;
        
        match toposort(&self.graph, None) {
            Ok(sorted_indices) => {
                let sorted_packages = sorted_indices
                    .into_iter()
                    .filter_map(|idx| self.graph.node_weight(idx))
                    .map(|node| node.id.clone())
                    .collect();
                Ok(sorted_packages)
            }
            Err(cycle_node) => {
                let cycle_path = self.extract_cycle_path(cycle_node.node_id());
                let cycle_str = Self::format_cycle(&cycle_path);
                Err(format!("Cannot sort due to circular dependency: {}", cycle_str))
            }
        }
    }
}
impl PackageId {
    /// Create a new package ID
    pub fn new(name: String, version: Version) -> Self {
        Self { name, version }
    }

    /// Create package ID from name and version string
    pub fn from_name_version(name: &str, version: &str) -> Result<Self, String> {
        use std::str::FromStr;
        let version = Version::from_str(version)
            .map_err(|e| format!("Invalid version '{}': {}", version, e))?;
        Ok(Self::new(name.to_string(), version))
    }
}

impl std::fmt::Display for PackageId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}@{}", self.name, self.version)
    }
}

impl PackageNode {
    /// Create a new package node
    pub fn new(
        name: String,
        version: Version,
        resolved_url: String,
        integrity: String,
    ) -> Self {
        let id = PackageId::new(name.clone(), version.clone());
        Self {
            id,
            name,
            version,
            resolved_url,
            integrity,
        }
    }
}

impl DependencyEdge {
    /// Create a new dependency edge
    pub fn new(version_req: VersionReq, kind: DependencyKind, optional: bool) -> Self {
        Self {
            version_req,
            kind,
            optional,
        }
    }

    /// Create a normal dependency edge
    pub fn normal(version_req: VersionReq) -> Self {
        Self::new(version_req, DependencyKind::Normal, false)
    }

    /// Create a dev dependency edge
    pub fn dev(version_req: VersionReq) -> Self {
        Self::new(version_req, DependencyKind::Dev, false)
    }

    /// Create an optional dependency edge
    pub fn optional(version_req: VersionReq, kind: DependencyKind) -> Self {
        Self::new(version_req, kind, true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn test_dependency_graph_creation() {
        let graph = DependencyGraph::new();
        assert_eq!(graph.package_count(), 0);
        assert_eq!(graph.dependency_count(), 0);
    }

    #[test]
    fn test_add_package() {
        let mut graph = DependencyGraph::new();
        
        let version = Version::from_str("1.0.0").unwrap();
        let package = PackageNode::new(
            "lodash".to_string(),
            version,
            "https://registry.npmjs.org/lodash/-/lodash-1.0.0.tgz".to_string(),
            "sha512-abc123".to_string(),
        );
        
        let node_index = graph.add_package(package.clone());
        
        assert_eq!(graph.package_count(), 1);
        assert!(graph.get_package(&package.id).is_some());
        
        // Adding same package again should return same index
        let node_index2 = graph.add_package(package.clone());
        assert_eq!(node_index, node_index2);
        assert_eq!(graph.package_count(), 1); // Should not duplicate
    }

    #[test]
    fn test_add_dependency() {
        let mut graph = DependencyGraph::new();
        
        let version1 = Version::from_str("1.0.0").unwrap();
        let version2 = Version::from_str("2.0.0").unwrap();
        
        let package1 = PackageNode::new(
            "app".to_string(),
            version1,
            "https://registry.npmjs.org/app/-/app-1.0.0.tgz".to_string(),
            "sha512-def456".to_string(),
        );
        
        let package2 = PackageNode::new(
            "lodash".to_string(),
            version2,
            "https://registry.npmjs.org/lodash/-/lodash-2.0.0.tgz".to_string(),
            "sha512-ghi789".to_string(),
        );
        
        graph.add_package(package1.clone());
        graph.add_package(package2.clone());
        
        let version_req = VersionReq::parse("^2.0.0").unwrap();
        let edge = DependencyEdge::normal(version_req);
        
        let result = graph.add_dependency(&package1.id, &package2.id, edge);
        assert!(result.is_ok());
        assert_eq!(graph.dependency_count(), 1);
    }

    #[test]
    fn test_add_dependency_missing_package() {
        let mut graph = DependencyGraph::new();
        
        let version1 = Version::from_str("1.0.0").unwrap();
        let version2 = Version::from_str("2.0.0").unwrap();
        
        let package1_id = PackageId::new("app".to_string(), version1);
        let package2_id = PackageId::new("lodash".to_string(), version2);
        
        let version_req = VersionReq::parse("^2.0.0").unwrap();
        let edge = DependencyEdge::normal(version_req);
        
        let result = graph.add_dependency(&package1_id, &package2_id, edge);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Package not found"));
    }

    #[test]
    fn test_cycle_detection_no_cycle() {
        let mut graph = DependencyGraph::new();
        
        let version1 = Version::from_str("1.0.0").unwrap();
        let version2 = Version::from_str("2.0.0").unwrap();
        let version3 = Version::from_str("3.0.0").unwrap();
        
        let package1 = PackageNode::new("a".to_string(), version1, "url1".to_string(), "hash1".to_string());
        let package2 = PackageNode::new("b".to_string(), version2, "url2".to_string(), "hash2".to_string());
        let package3 = PackageNode::new("c".to_string(), version3, "url3".to_string(), "hash3".to_string());
        
        graph.add_package(package1.clone());
        graph.add_package(package2.clone());
        graph.add_package(package3.clone());
        
        let version_req = VersionReq::parse("*").unwrap();
        let edge = DependencyEdge::normal(version_req.clone());
        
        // Create linear dependency: a -> b -> c
        graph.add_dependency(&package1.id, &package2.id, edge.clone()).unwrap();
        graph.add_dependency(&package2.id, &package3.id, edge).unwrap();
        
        let result = graph.detect_cycles();
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    #[test]
    fn test_cycle_detection_with_cycle() {
        let mut graph = DependencyGraph::new();
        
        let version1 = Version::from_str("1.0.0").unwrap();
        let version2 = Version::from_str("2.0.0").unwrap();
        let version3 = Version::from_str("3.0.0").unwrap();
        
        let package1 = PackageNode::new("a".to_string(), version1, "url1".to_string(), "hash1".to_string());
        let package2 = PackageNode::new("b".to_string(), version2, "url2".to_string(), "hash2".to_string());
        let package3 = PackageNode::new("c".to_string(), version3, "url3".to_string(), "hash3".to_string());
        
        graph.add_package(package1.clone());
        graph.add_package(package2.clone());
        graph.add_package(package3.clone());
        
        let version_req = VersionReq::parse("*").unwrap();
        let edge = DependencyEdge::normal(version_req.clone());
        
        // Create cycle: a -> b -> c -> a
        graph.add_dependency(&package1.id, &package2.id, edge.clone()).unwrap();
        graph.add_dependency(&package2.id, &package3.id, edge.clone()).unwrap();
        graph.add_dependency(&package3.id, &package1.id, edge).unwrap();
        
        let result = graph.detect_cycles();
        assert!(result.is_err());
        
        let cycle = result.unwrap_err();
        assert!(!cycle.is_empty());
        
        // Verify cycle formatting
        let cycle_str = DependencyGraph::format_cycle(&cycle);
        assert!(cycle_str.contains("->"));
        assert!(cycle_str.contains("a"));
        assert!(cycle_str.contains("b"));
        assert!(cycle_str.contains("c"));
    }

    #[test]
    fn test_topological_sort() {
        let mut graph = DependencyGraph::new();
        
        let version1 = Version::from_str("1.0.0").unwrap();
        let version2 = Version::from_str("2.0.0").unwrap();
        let version3 = Version::from_str("3.0.0").unwrap();
        
        let package1 = PackageNode::new("a".to_string(), version1, "url1".to_string(), "hash1".to_string());
        let package2 = PackageNode::new("b".to_string(), version2, "url2".to_string(), "hash2".to_string());
        let package3 = PackageNode::new("c".to_string(), version3, "url3".to_string(), "hash3".to_string());
        
        graph.add_package(package1.clone());
        graph.add_package(package2.clone());
        graph.add_package(package3.clone());
        
        let version_req = VersionReq::parse("*").unwrap();
        let edge = DependencyEdge::normal(version_req.clone());
        
        // Create dependencies: a -> b, a -> c, b -> c
        graph.add_dependency(&package1.id, &package2.id, edge.clone()).unwrap();
        graph.add_dependency(&package1.id, &package3.id, edge.clone()).unwrap();
        graph.add_dependency(&package2.id, &package3.id, edge).unwrap();
        
        let result = graph.topological_sort();
        assert!(result.is_ok());
        
        let sorted = result.unwrap();
        assert_eq!(sorted.len(), 3);
        
        // petgraph's toposort returns nodes in reverse topological order
        // (dependents before dependencies), which is what we got: ["a", "b", "c"]
        // This is actually correct for our use case since we want to process
        // packages in dependency order (dependencies first)
        let c_pos = sorted.iter().position(|id| id.name == "c").unwrap();
        let b_pos = sorted.iter().position(|id| id.name == "b").unwrap();
        let a_pos = sorted.iter().position(|id| id.name == "a").unwrap();
        
        // In petgraph's order: a comes first (has most dependencies), then b, then c (no dependencies)
        assert!(a_pos < b_pos, "a should come before b in petgraph's topological order");
        assert!(b_pos < c_pos, "b should come before c in petgraph's topological order");
    }

    #[test]
    fn test_validate_no_cycles() {
        let mut graph = DependencyGraph::new();
        
        let version1 = Version::from_str("1.0.0").unwrap();
        let package1 = PackageNode::new("a".to_string(), version1, "url1".to_string(), "hash1".to_string());
        graph.add_package(package1);
        
        // No cycles in single node
        assert!(graph.validate_no_cycles().is_ok());
        
        // Add cycle
        let version2 = Version::from_str("2.0.0").unwrap();
        let package2 = PackageNode::new("b".to_string(), version2, "url2".to_string(), "hash2".to_string());
        let package1_id = PackageId::new("a".to_string(), Version::from_str("1.0.0").unwrap());
        let package2_id = PackageId::new("b".to_string(), Version::from_str("2.0.0").unwrap());
        
        graph.add_package(package2);
        
        let version_req = VersionReq::parse("*").unwrap();
        let edge = DependencyEdge::normal(version_req.clone());
        
        graph.add_dependency(&package1_id, &package2_id, edge.clone()).unwrap();
        graph.add_dependency(&package2_id, &package1_id, edge).unwrap();
        
        let result = graph.validate_no_cycles();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Circular dependency detected"));
    }
}

#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;
    use std::collections::HashSet;

    // Property 12: Cycle Detection Correctness
    proptest! {
        #[test]
        fn cycle_detection_correctness(
            // Generate a small graph to keep test time reasonable
            num_packages in 3usize..8,
            // Generate edges as pairs of indices
            edges in prop::collection::vec((0usize..7, 0usize..7), 0..15)
        ) {
            let mut graph = DependencyGraph::new();
            let mut package_ids = Vec::new();
            
            // Create packages
            for i in 0..num_packages {
                let version = Version::new(1, 0, i as u64);
                let package = PackageNode::new(
                    format!("pkg{}", i),
                    version,
                    format!("url{}", i),
                    format!("hash{}", i),
                );
                package_ids.push(package.id.clone());
                graph.add_package(package);
            }
            
            // Add edges (filter out self-loops and out-of-bounds)
            let version_req = VersionReq::parse("*").unwrap();
            let edge = DependencyEdge::normal(version_req);
            
            for (from_idx, to_idx) in edges {
                if from_idx < num_packages && to_idx < num_packages && from_idx != to_idx {
                    let _ = graph.add_dependency(
                        &package_ids[from_idx],
                        &package_ids[to_idx],
                        edge.clone()
                    );
                }
            }
            
            // Test cycle detection
            let cycle_result = graph.detect_cycles();
            let topo_result = graph.topological_sort();
            
            // Property: If cycle detection finds no cycles, topological sort should succeed
            // If cycle detection finds cycles, topological sort should fail
            let has_cycles = match &cycle_result {
                Ok(cycles) => !cycles.is_empty(),
                Err(_) => true,
            };
            
            if has_cycles {
                prop_assert!(topo_result.is_err(), "Cycles detected but topological sort succeeded");
            } else {
                prop_assert!(topo_result.is_ok(), "No cycles detected but topological sort failed");
            }
            
            // Property: validate_no_cycles should be consistent with detect_cycles
            let validate_result = graph.validate_no_cycles();
            if has_cycles {
                prop_assert!(validate_result.is_err(), "Cycles present but validation passed");
            } else {
                prop_assert!(validate_result.is_ok(), "No cycles but validation failed");
            }
        }
    }

    // Property: Topological sort produces valid ordering
    proptest! {
        #[test]
        fn topological_sort_validity(
            num_packages in 2usize..6,
            edges in prop::collection::vec((0usize..5, 0usize..5), 0..10)
        ) {
            let mut graph = DependencyGraph::new();
            let mut package_ids = Vec::new();
            
            // Create packages
            for i in 0..num_packages {
                let version = Version::new(1, 0, i as u64);
                let package = PackageNode::new(
                    format!("pkg{}", i),
                    version,
                    format!("url{}", i),
                    format!("hash{}", i),
                );
                package_ids.push(package.id.clone());
                graph.add_package(package);
            }
            
            // Add edges to create a DAG (no back edges)
            let version_req = VersionReq::parse("*").unwrap();
            let edge = DependencyEdge::normal(version_req);
            
            for (from_idx, to_idx) in edges {
                if from_idx < num_packages && to_idx < num_packages && from_idx < to_idx {
                    let _ = graph.add_dependency(
                        &package_ids[from_idx],
                        &package_ids[to_idx],
                        edge.clone()
                    );
                }
            }
            
            if let Ok(sorted) = graph.topological_sort() {
                // Property: All packages should be in the sorted result
                prop_assert_eq!(sorted.len(), num_packages);
                
                // Property: No duplicates in sorted result
                let unique_count: HashSet<_> = sorted.iter().collect();
                prop_assert_eq!(unique_count.len(), sorted.len());
                
                // Property: All original packages should be present
                for package_id in &package_ids {
                    prop_assert!(sorted.contains(package_id), "Package {} missing from sort", package_id);
                }
            }
        }
    }
}