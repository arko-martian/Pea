//! Unit tests for registry client

use super::*;

use wiremock::{MockServer, Mock, ResponseTemplate};
use wiremock::matchers::{method, path, header};

#[tokio::test]
async fn test_registry_client_creation() {
    let client = RegistryClient::new().unwrap();
    assert_eq!(client.base_url, "https://registry.npmjs.org");
    assert_eq!(client.retry_config.max_retries, 3);
}

#[tokio::test]
async fn test_registry_client_with_auth() {
    let auth = AuthConfig {
        token: Some("test-token".to_string()),
        username: None,
        password: None,
    };
    
    let client = RegistryClient::with_auth(auth).unwrap();
    assert_eq!(client.base_url, "https://registry.npmjs.org");
}

#[tokio::test]
async fn test_encode_package_name() {
    let client = RegistryClient::new().unwrap();
    
    // Regular package
    assert_eq!(client.encode_package_name("lodash"), "lodash");
    
    // Scoped package
    assert_eq!(client.encode_package_name("@types/node"), "@types%2fnode");
}

#[tokio::test]
async fn test_retry_config_default() {
    let config = RetryConfig::default();
    assert_eq!(config.max_retries, 3);
    assert_eq!(config.initial_delay, Duration::from_millis(100));
    assert_eq!(config.max_delay, Duration::from_secs(10));
    assert_eq!(config.multiplier, 2.0);
}

#[tokio::test]
async fn test_fetch_metadata_success() {
    let mock_server = MockServer::start().await;
    
    let mock_response = serde_json::json!({
        "name": "test-package",
        "description": "A test package",
        "dist-tags": {
            "latest": "1.0.0"
        },
        "versions": {
            "1.0.0": {
                "version": "1.0.0",
                "description": "A test package",
                "dist": {
                    "tarball": "https://registry.npmjs.org/test-package/-/test-package-1.0.0.tgz",
                    "shasum": "abc123",
                    "integrity": "sha512-def456"
                }
            }
        },
        "time": {
            "created": "2023-01-01T00:00:00.000Z",
            "1.0.0": "2023-01-01T00:00:00.000Z"
        }
    });

    Mock::given(method("GET"))
        .and(path("/test-package"))
        .and(header("Accept", "application/vnd.npm.install-v1+json"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&mock_response))
        .mount(&mock_server)
        .await;

    let mut client = RegistryClient::new().unwrap();
    client.base_url = mock_server.uri();
    
    let metadata = client.fetch_metadata("test-package").await.unwrap();
    assert_eq!(metadata.name, "test-package");
    assert_eq!(metadata.description, Some("A test package".to_string()));
}

#[tokio::test]
async fn test_fetch_metadata_not_found() {
    let mock_server = MockServer::start().await;
    
    Mock::given(method("GET"))
        .and(path("/nonexistent-package"))
        .respond_with(ResponseTemplate::new(404))
        .mount(&mock_server)
        .await;

    let mut client = RegistryClient::new().unwrap();
    client.base_url = mock_server.uri();
    
    let result = client.fetch_metadata("nonexistent-package").await;
    assert!(result.is_err());
    
    match result.unwrap_err() {
        PeaError::PackageNotFound { name } => {
            assert_eq!(name, "nonexistent-package");
        }
        _ => panic!("Expected PackageNotFound error"),
    }
}

#[tokio::test]
async fn test_scoped_package_url_encoding() {
    let mock_server = MockServer::start().await;
    
    Mock::given(method("GET"))
        .and(path("/@types%2fnode"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "name": "@types/node",
            "dist-tags": { "latest": "1.0.0" },
            "versions": {},
            "time": {}
        })))
        .mount(&mock_server)
        .await;

    let mut client = RegistryClient::new().unwrap();
    client.base_url = mock_server.uri();
    
    let result = client.fetch_metadata("@types/node").await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_integrity_verification_sha512() {
    let client = RegistryClient::new().unwrap();
    let content = b"test content";
    
    // Compute expected blake3 hash
    let hash = blake3::hash(content);
    use base64::{Engine as _, engine::general_purpose};
    let expected_b64 = general_purpose::STANDARD.encode(hash.as_bytes());
    
    let dist_info = crate::api::DistInfo {
        tarball: "https://example.com/test.tgz".to_string(),
        shasum: "wrong-sha1".to_string(),
        integrity: Some(format!("sha512-{}", expected_b64)),
        unpacked_size: None,
        file_count: None,
    };
    
    let result = client.verify_integrity(content, &dist_info);
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_integrity_verification_sha1_fallback() {
    let client = RegistryClient::new().unwrap();
    let content = b"test content";
    
    // Compute expected SHA-1
    use sha1::{Sha1, Digest};
    let mut hasher = Sha1::new();
    hasher.update(content);
    let expected_sha1 = format!("{:x}", hasher.finalize());
    
    let dist_info = crate::api::DistInfo {
        tarball: "https://example.com/test.tgz".to_string(),
        shasum: expected_sha1,
        integrity: None,
        unpacked_size: None,
        file_count: None,
    };
    
    let result = client.verify_integrity(content, &dist_info);
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_integrity_verification_failure() {
    let client = RegistryClient::new().unwrap();
    let content = b"test content";
    
    let dist_info = crate::api::DistInfo {
        tarball: "https://example.com/test.tgz".to_string(),
        shasum: "wrong-hash".to_string(),
        integrity: None,
        unpacked_size: None,
        file_count: None,
    };
    
    let result = client.verify_integrity(content, &dist_info);
    assert!(result.is_err());
    
    match result.unwrap_err() {
        PeaError::IntegrityFailure { .. } => {
            // Expected
        }
        _ => panic!("Expected IntegrityFailure error"),
    }
}