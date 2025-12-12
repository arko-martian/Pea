# JSON Module Guide

## Purpose
Parse and serialize package.json files for Node.js compatibility.

## Key Types
- `PackageJson` - Complete package.json structure
- `ScriptsSection` - npm scripts
- `DependenciesSection` - Various dependency types

## Functions (Max 4 Public)
1. `parse_package_json()` - Parse JSON string to PackageJson
2. `serialize_package_json()` - Serialize PackageJson to JSON string
3. `import_to_pea_toml()` - Convert PackageJson to PeaToml
4. `load_from_file()` - Load and parse from file path

## Error Handling
- Use `serde_json` for JSON parsing
- Report syntax errors with locations
- Handle missing optional fields gracefully
- Validate npm-specific constraints