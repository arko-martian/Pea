# TOML Module Guide

## Purpose
Parse and serialize pea.toml configuration files with comprehensive error reporting.

## Key Types
- `PeaToml` - Root configuration structure
- `PackageSection` - Package metadata
- `DependencySpec` - Dependency specifications (simple string or detailed object)
- `WorkspaceSection` - Workspace configuration
- `ProfileSection` - Build profiles

## Functions (Max 4 Public)
1. `parse_pea_toml()` - Parse TOML string to PeaToml
2. `serialize_pea_toml()` - Serialize PeaToml to TOML string
3. `validate_config()` - Validate configuration completeness
4. `load_from_file()` - Load and parse from file path

## Error Handling
- Use `toml_edit` for precise error locations
- Report line/column numbers for syntax errors
- Validate required fields with helpful messages
- Warn about unknown fields