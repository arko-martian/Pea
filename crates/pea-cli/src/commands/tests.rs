//! Unit tests for CLI commands.

use super::*;
use std::fs;
use tempfile::TempDir;

/// Create a temporary directory for testing
fn create_temp_dir() -> TempDir {
    tempfile::tempdir().expect("Failed to create temp directory")
}

/// Create a test command context in a temporary directory
async fn create_test_context(temp_dir: &TempDir) -> CommandContext {
    CommandContext {
        cwd: temp_dir.path().to_path_buf(),
        output: crate::output::OutputHandler::new(),
    }
}

#[tokio::test]
async fn test_suggest_similar_command() {
    // Test exact matches
    assert_eq!(suggest_similar_command("install"), Some("install".to_string()));
    
    // Test typos
    assert_eq!(suggest_similar_command("instal"), Some("install".to_string()));
    assert_eq!(suggest_similar_command("buld"), Some("build".to_string()));
    assert_eq!(suggest_similar_command("tst"), Some("test".to_string()));
    assert_eq!(suggest_similar_command("nw"), Some("new".to_string()));
    
    // Test no suggestion for very different strings
    assert_eq!(suggest_similar_command("xyz"), None);
    assert_eq!(suggest_similar_command("completely-different"), None);
}

#[tokio::test]
async fn test_edit_distance() {
    assert_eq!(edit_distance("", ""), 0);
    assert_eq!(edit_distance("", "abc"), 3);
    assert_eq!(edit_distance("abc", ""), 3);
    assert_eq!(edit_distance("abc", "abc"), 0);
    assert_eq!(edit_distance("abc", "ab"), 1);
    assert_eq!(edit_distance("abc", "abcd"), 1);
    assert_eq!(edit_distance("install", "instal"), 1);
    assert_eq!(edit_distance("build", "buld"), 1);
    assert_eq!(edit_distance("test", "tst"), 1);
}

#[tokio::test]
async fn test_new_command_validation() {
    let temp_dir = create_temp_dir();
    let ctx = create_test_context(&temp_dir).await;
    
    // Test valid project name
    let result = new::execute("valid-project".to_string(), None, &ctx).await;
    assert!(result.is_ok());
    
    // Check that project was created
    let project_path = temp_dir.path().join("valid-project");
    assert!(project_path.exists());
    assert!(project_path.join("pea.toml").exists());
    assert!(project_path.join("src").exists());
    assert!(project_path.join("src/index.ts").exists());
    assert!(project_path.join("GUIDEMAP.md").exists());
}

#[tokio::test]
async fn test_new_command_invalid_names() {
    let temp_dir = create_temp_dir();
    let ctx = create_test_context(&temp_dir).await;
    
    // Test invalid project names
    let invalid_names = vec![
        "",
        "invalid@name",
        "-invalid",
        "invalid-",
        "invalid space",
    ];
    
    for name in invalid_names {
        let result = new::execute(name.to_string(), None, &ctx).await;
        assert!(result.is_err(), "Expected error for invalid name: {}", name);
    }
}

#[tokio::test]
async fn test_new_command_existing_directory() {
    let temp_dir = create_temp_dir();
    let ctx = create_test_context(&temp_dir).await;
    
    // Create a directory first
    let project_path = temp_dir.path().join("existing");
    fs::create_dir(&project_path).unwrap();
    
    // Try to create project with same name
    let result = new::execute("existing".to_string(), None, &ctx).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_init_command_empty_directory() {
    let temp_dir = create_temp_dir();
    let ctx = create_test_context(&temp_dir).await;
    
    // Test init in empty directory
    let result = init::execute(&ctx).await;
    assert!(result.is_ok());
    
    // Check that files were created
    assert!(temp_dir.path().join("pea.toml").exists());
    assert!(temp_dir.path().join("src").exists());
    assert!(temp_dir.path().join("src/index.ts").exists());
    assert!(temp_dir.path().join("GUIDEMAP.md").exists());
}

#[tokio::test]
async fn test_init_command_existing_pea_toml() {
    let temp_dir = create_temp_dir();
    let ctx = create_test_context(&temp_dir).await;
    
    // Create existing pea.toml
    fs::write(temp_dir.path().join("pea.toml"), "existing content").unwrap();
    
    // Test init - should skip
    let result = init::execute(&ctx).await;
    assert!(result.is_ok());
    
    // Check that existing file wasn't overwritten
    let content = fs::read_to_string(temp_dir.path().join("pea.toml")).unwrap();
    assert_eq!(content, "existing content");
}

#[tokio::test]
async fn test_init_command_import_package_json() {
    let temp_dir = create_temp_dir();
    let ctx = create_test_context(&temp_dir).await;
    
    // Create package.json
    let package_json = r#"{
        "name": "test-import",
        "version": "1.0.0",
        "description": "Test package",
        "main": "dist/index.js",
        "dependencies": {
            "lodash": "^4.17.21"
        },
        "devDependencies": {
            "typescript": "^5.0.0"
        },
        "scripts": {
            "build": "tsc",
            "start": "node dist/index.js"
        }
    }"#;
    
    fs::write(temp_dir.path().join("package.json"), package_json).unwrap();
    
    // Test init - should import
    let result = init::execute(&ctx).await;
    assert!(result.is_ok());
    
    // Check that pea.toml was created with imported content
    let pea_toml = fs::read_to_string(temp_dir.path().join("pea.toml")).unwrap();
    assert!(pea_toml.contains("name = \"test-import\""));
    assert!(pea_toml.contains("version = \"1.0.0\""));
    assert!(pea_toml.contains("lodash = \"^4.17.21\""));
    assert!(pea_toml.contains("typescript = \"^5.0.0\""));
    assert!(pea_toml.contains("build = \"tsc\""));
}

#[tokio::test]
async fn test_show_version() {
    let temp_dir = create_temp_dir();
    let ctx = create_test_context(&temp_dir).await;
    
    // Test version command
    let result = show_version(&ctx).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_show_help() {
    let temp_dir = create_temp_dir();
    let ctx = create_test_context(&temp_dir).await;
    
    // Test help command
    let result = show_help(&ctx).await;
    assert!(result.is_ok());
}