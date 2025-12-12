# Merge Module Guide

## Purpose
Handle configuration layering, fallback logic, and environment overrides.

## Key Types
- `ConfigLoader` - Main configuration loading interface
- `ConfigLayering` - Layer management and merging
- `ConfigSource` - Source tracking (file, env, CLI)

## Functions (Max 4 Public)
1. `load_project_config()` - Load project configuration with fallbacks
2. `merge_configs()` - Merge multiple configuration layers
3. `apply_overrides()` - Apply environment and CLI overrides
4. `resolve_config_path()` - Find configuration file in project

## Layering Priority (highest to lowest)
1. CLI flags
2. Environment variables
3. Project pea.toml
4. Project package.json (fallback)
5. Global ~/.pea/config.toml