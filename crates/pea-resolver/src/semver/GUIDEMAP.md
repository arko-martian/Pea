# Semver Module Guide

## Purpose
Semantic version resolution and constraint satisfaction for dependency resolution.

## Key Types
- `VersionSelector` - Finds best matching versions from available set
- `ConstraintSolver` - Checks constraint satisfiability
- Version constraint operations and compatibility checking

## Functions (Max 4 Public)
1. `select_best()` - Choose highest version matching all constraints
2. `find_matching()` - Find all versions satisfying constraint
3. `is_satisfiable()` - Check if constraints can be satisfied
4. `add_constraint()` - Add version requirement to solver

## Version Selection Strategy
- Prefer highest compatible versions (latest-first)
- Satisfy all version constraints simultaneously
- Handle complex constraint combinations (^, ~, >=, etc.)
- Detect unsatisfiable constraint sets

## Constraint Types
- Caret constraints (^1.2.3) - compatible within major version
- Tilde constraints (~1.2.3) - compatible within minor version
- Range constraints (>=1.0.0, <2.0.0) - explicit bounds
- Exact constraints (=1.2.3) - specific version only