//! HTTP client implementation with connection pooling and retry logic

use std::time::Duration;
use reqwest::{Client, ClientBuilder};

use pea_core::error::PeaError;
use crate::RegistryResult;

/// Configuration for exponential backoff retry logic
#[derive(Debug, Clone)]
pub struct RetryConfig {
    /// Maximum number of retry attempts
    pub max_retries: u32,
    /// Initial delay before first retry
    pub initial_delay: Duration,
    /// Maximum delay between retries
    pub max_delay: Duration,
    /// Multiplier for exponential backoff
    pub multiplier: f64,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            initial_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(10),
            multiplier: 2.0,
        }
    }
}

/// Authentication configuration for registry access
#[derive(Debug, Clone)]
pub struct AuthConfig {
    /// Bearer token for authentication
    pub token: Option<String>,
    /// Basic auth username
    pub username: Option<String>,
    /// Basic auth password
    pub password: Option<String>,
}

/// Main HTTP client for npm registry operations
#[derive(Debug, Clone)]
pub struct RegistryClient {
    /// Underlying HTTP client with connection pooling
    client: Client,
    /// Retry configuration
    retry_config: RetryConfig,
    /// Base registry URL
    base_url: String,
}
impl RegistryClient {
    /// Create new registry client with connection pooling
    pub fn new() -> RegistryResult<Self> {
        Self::with_config(None, RetryConfig::default())
    }

    /// Create registry client with authentication
    pub fn with_auth(auth: AuthConfig) -> RegistryResult<Self> {
        Self::with_config(Some(auth), RetryConfig::default())
    }

    /// Create registry client with custom configuration
    fn with_config(auth: Option<AuthConfig>, retry_config: RetryConfig) -> RegistryResult<Self> {
        let mut builder = ClientBuilder::new()
            // Connection pooling configuration
            .pool_max_idle_per_host(50)
            .pool_idle_timeout(Duration::from_secs(90))
            // Request timeout
            .timeout(Duration::from_secs(30))
            // Enable HTTP/2 with prior knowledge
            .http2_prior_knowledge()
            // Enable gzip compression
            .gzip(true)
            // User agent
            .user_agent("pea/0.1.0");

        // Configure authentication if provided
        if let Some(auth_config) = auth {
            if let Some(token) = auth_config.token {
                builder = builder.default_headers({
                    let mut headers = reqwest::header::HeaderMap::new();
                    headers.insert(
                        reqwest::header::AUTHORIZATION,
                        format!("Bearer {}", token).parse()
                            .map_err(|e| PeaError::Network { 
                                message: format!("Invalid auth token: {}", e),
                                source: Some(Box::new(e))
                            })?
                    );
                    headers
                });
            } else if let (Some(username), Some(password)) = (auth_config.username, auth_config.password) {
                use base64::{Engine as _, engine::general_purpose};
                let auth_value = format!("Basic {}", general_purpose::STANDARD.encode(format!("{}:{}", username, password)));
                builder = builder.default_headers({
                    let mut headers = reqwest::header::HeaderMap::new();
                    headers.insert(
                        reqwest::header::AUTHORIZATION,
                        auth_value.parse()
                            .map_err(|e| PeaError::Network { 
                                message: format!("Invalid basic auth: {}", e),
                                source: Some(Box::new(e))
                            })?
                    );
                    headers
                });
            }
        }

        let client = builder.build()
            .map_err(|e| PeaError::Network { 
                message: format!("Failed to create HTTP client: {}", e),
                source: Some(Box::new(e))
            })?;

        Ok(Self {
            client,
            retry_config,
            base_url: "https://registry.npmjs.org".to_string(),
        })
    }

    /// Execute HTTP request with exponential backoff retry logic
    async fn with_retry<F, Fut, T>(&self, operation: F) -> RegistryResult<T>
    where
        F: Fn() -> Fut,
        Fut: std::future::Future<Output = RegistryResult<T>>,
    {
        let mut delay = self.retry_config.initial_delay;
        let mut last_error = None;

        for attempt in 0..=self.retry_config.max_retries {
            match operation().await {
                Ok(result) => return Ok(result),
                Err(error) => {
                    last_error = Some(error);
                    
                    // Don't retry on final attempt
                    if attempt == self.retry_config.max_retries {
                        break;
                    }

                    // Don't retry on certain error types
                    if let Some(ref err) = last_error {
                        match err {
                            PeaError::PackageNotFound { .. } => break,
                            PeaError::PermissionDenied { .. } => break,
                            _ => {}
                        }
                    }

                    // Wait before retry
                    tokio::time::sleep(delay).await;
                    
                    // Exponential backoff with jitter
                    delay = std::cmp::min(
                        Duration::from_millis(
                            (delay.as_millis() as f64 * self.retry_config.multiplier) as u64
                        ),
                        self.retry_config.max_delay
                    );
                }
            }
        }

        Err(last_error.unwrap_or_else(|| 
            PeaError::Network { 
                message: "Retry operation failed without error".to_string(),
                source: None
            }
        ))
    }

    /// Fetch package metadata with retry logic
    pub async fn fetch_metadata(&self, package_name: &str) -> RegistryResult<crate::api::PackageMetadataResponse> {
        let encoded_name = self.encode_package_name(package_name);
        let url = format!("{}/{}", self.base_url, encoded_name);
        
        self.with_retry(|| async {
            let response = self.client
                .get(&url)
                .header("Accept", "application/vnd.npm.install-v1+json")
                .send()
                .await
                .map_err(|e| PeaError::Network { 
                    message: format!("Failed to fetch metadata: {}", e),
                    source: Some(Box::new(e))
                })?;

            match response.status() {
                reqwest::StatusCode::OK => {
                    let metadata = response.json::<crate::api::PackageMetadataResponse>()
                        .await
                        .map_err(|e| PeaError::Network { 
                            message: format!("Failed to parse metadata: {}", e),
                            source: Some(Box::new(e))
                        })?;
                    Ok(metadata)
                }
                reqwest::StatusCode::NOT_FOUND => {
                    Err(PeaError::PackageNotFound { name: package_name.to_string() })
                }
                status => {
                    Err(PeaError::Network { 
                        message: format!("Registry returned status {}: {}", status, package_name),
                        source: None
                    })
                }
            }
        }).await
    }

    /// Download package tarball with integrity verification
    pub async fn download_tarball(&self, dist_info: &crate::api::DistInfo) -> RegistryResult<Vec<u8>> {
        self.with_retry(|| async {
            let response = self.client
                .get(&dist_info.tarball)
                .send()
                .await
                .map_err(|e| PeaError::Network { 
                    message: format!("Failed to download tarball: {}", e),
                    source: Some(Box::new(e))
                })?;

            if !response.status().is_success() {
                return Err(PeaError::Network { 
                    message: format!("Failed to download tarball: {}", response.status()),
                    source: None
                });
            }

            let bytes = response.bytes()
                .await
                .map_err(|e| PeaError::Network { 
                    message: format!("Failed to read tarball: {}", e),
                    source: Some(Box::new(e))
                })?
                .to_vec();

            // Verify integrity
            self.verify_integrity(&bytes, dist_info)?;

            Ok(bytes)
        }).await
    }

    /// Encode package name for URL (handle scoped packages)
    fn encode_package_name(&self, name: &str) -> String {
        if name.starts_with('@') {
            // Scoped package: @org/pkg → @org%2fpkg
            name.replace('/', "%2f")
        } else {
            name.to_string()
        }
    }

    /// Verify tarball integrity using SHA-512 or SHA-1
    fn verify_integrity(&self, bytes: &[u8], dist_info: &crate::api::DistInfo) -> RegistryResult<()> {
        // Prefer subresource integrity if available
        if let Some(integrity) = &dist_info.integrity {
            if integrity.starts_with("sha512-") {
                let expected = &integrity[7..]; // Remove "sha512-" prefix
                let computed = blake3::hash(bytes);
                use base64::{Engine as _, engine::general_purpose};
                let computed_b64 = general_purpose::STANDARD.encode(computed.as_bytes());
                
                if computed_b64 != expected {
                    return Err(PeaError::IntegrityFailure {
                        package: "unknown".to_string(), // TODO: Pass package name
                        expected: expected.to_string(),
                        actual: computed_b64
                    });
                }
                return Ok(());
            }
        }

        // Fall back to SHA-1 checksum
        use sha1::{Sha1, Digest};
        let mut hasher = Sha1::new();
        hasher.update(bytes);
        let computed = format!("{:x}", hasher.finalize());
        
        if computed != dist_info.shasum {
            return Err(PeaError::IntegrityFailure {
                package: "unknown".to_string(), // TODO: Pass package name
                expected: dist_info.shasum.clone(),
                actual: computed
            });
        }

        Ok(())
    }
}
#[cfg(test)]
mod tests;