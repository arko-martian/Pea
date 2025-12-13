# Graph Module Guide

## Purpose
Dependency graph implementation using petgraph with cycle detection and thread-safe operations.

## Key Types
- `DependencyGraph` - Thread-safe directed graph using petgraph
- `PackageNode` - Resolved package with metadata
- `DependencyEdge` - Dependency relationship with constraints
- `PackageId` - Unique package identifier (name + version)

## Functions (Max 4 Public)
1. `new()` - Create empty dependency graph
2. `add_package()` - Add package node with thread safety
3. `add_dependency()` - Add dependency edge between packages
4. `detect_cycles()` - Find circular dependencies using toposort

## Performance Features
- Thread-safe operations using DashMap
- Efficient graph traversal with petgraph algorithms
- Cycle detection using topological sort
- Fast package lookups with hash map indexing

## Graph Algorithms
- Topological sorting for cycle detection
- Breadth-first search for dependency traversal
- Strongly connected components for conflict analysis
- Path finding for dependency chains