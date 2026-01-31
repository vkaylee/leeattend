//! Storage Provider Abstraction
//!
//! Unified interface for S3 and local filesystem storage.
//! Enables fallback to local storage when S3 is not configured.

use async_trait::async_trait;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use tokio::fs;
use tokio::io::AsyncWriteExt;

use crate::services::S3Client;
use crate::StorageBackend;

/// Storage operation error
#[derive(Debug)]
pub struct StorageError(pub String);

impl std::fmt::Display for StorageError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for StorageError {}

/// Unified storage provider trait
#[async_trait]
pub trait StorageProvider: Send + Sync {
    /// Upload file to storage.
    /// Returns storage key/path on success.
    async fn upload(
        &self,
        key: &str,
        data: Vec<u8>,
        content_type: &str,
    ) -> Result<String, StorageError>;

    /// Get download URL for a stored file.
    /// For S3: presigned URL. For local: API endpoint URL.
    async fn get_download_url(&self, key: &str) -> Result<String, StorageError>;

    /// Storage type identifier for logging
    fn storage_type(&self) -> &'static str;
}

pub type DynStorageProvider = Arc<dyn StorageProvider>;

/// S3-backed storage provider
pub struct S3Storage {
    client: S3Client,
}

impl S3Storage {
    pub async fn new() -> Result<Self, StorageError> {
        let client = S3Client::new()
            .await
            .map_err(|e| StorageError(e.to_string()))?;
        Ok(Self { client })
    }
}

#[async_trait]
impl StorageProvider for S3Storage {
    async fn upload(
        &self,
        key: &str,
        data: Vec<u8>,
        content_type: &str,
    ) -> Result<String, StorageError> {
        self.client
            .upload_report(key, data, content_type)
            .await
            .map_err(|e| StorageError(e.to_string()))
    }

    async fn get_download_url(&self, key: &str) -> Result<String, StorageError> {
        const PRESIGNED_URL_EXPIRY: Duration = Duration::from_secs(86400); // 24 hours
        self.client
            .generate_presigned_url(key, PRESIGNED_URL_EXPIRY)
            .await
            .map_err(|e| StorageError(e.to_string()))
    }

    fn storage_type(&self) -> &'static str {
        "s3"
    }
}

/// Local filesystem storage provider
pub struct LocalStorage {
    base_path: PathBuf,
    base_url: String,
}

impl LocalStorage {
    /// Create local storage with base path and API base URL.
    ///
    /// # Arguments
    /// * `base_path` - Directory to store files (e.g., ./storage/reports)
    /// * `base_url` - API base URL for download links (e.g., /api/reports/local)
    pub fn new(base_path: PathBuf, base_url: String) -> Self {
        Self { base_path, base_url }
    }

    /// Ensure directory exists for the given key path
    async fn ensure_dir(&self, key: &str) -> Result<PathBuf, StorageError> {
        let file_path = self.base_path.join(key);

        if let Some(parent) = file_path.parent() {
            fs::create_dir_all(parent)
                .await
                .map_err(|e| StorageError(format!("Failed to create directory: {}", e)))?;
        }

        // Security: verify path is within base_path (prevent path traversal)
        let canonical_base = fs::canonicalize(&self.base_path)
            .await
            .unwrap_or_else(|_| self.base_path.clone());

        // Check after directory creation
        if file_path.starts_with(&canonical_base) {
            Ok(file_path)
        } else {
            // For more robust check, try to canonicalize parent
            if let Some(parent) = file_path.parent() {
                if let Ok(canonical_parent) = fs::canonicalize(parent).await {
                    if canonical_parent.starts_with(&canonical_base) {
                        return Ok(file_path);
                    }
                }
            }
            Err(StorageError("Path traversal detected".to_string()))
        }
    }

    /// Get file path for a key
    pub fn file_path(&self, key: &str) -> PathBuf {
        self.base_path.join(key)
    }
}

#[async_trait]
impl StorageProvider for LocalStorage {
    async fn upload(
        &self,
        key: &str,
        data: Vec<u8>,
        _content_type: &str,
    ) -> Result<String, StorageError> {
        let file_path = self.ensure_dir(key).await?;

        // Write atomically using temp file
        let temp_path = file_path.with_extension("tmp");
        let mut file = fs::File::create(&temp_path)
            .await
            .map_err(|e| StorageError(format!("Failed to create file: {}", e)))?;

        file.write_all(&data)
            .await
            .map_err(|e| StorageError(format!("Failed to write file: {}", e)))?;

        file.sync_all()
            .await
            .map_err(|e| StorageError(format!("Failed to sync file: {}", e)))?;

        fs::rename(&temp_path, &file_path)
            .await
            .map_err(|e| StorageError(format!("Failed to finalize file: {}", e)))?;

        tracing::info!(
            path = %file_path.display(),
            size = data.len(),
            "Report saved to local storage"
        );

        Ok(key.to_string())
    }

    async fn get_download_url(&self, key: &str) -> Result<String, StorageError> {
        // Return API endpoint URL for local files
        // Format: /api/reports/local/{key}
        // Note: key already contains workspace_id/year/month/filename structure
        Ok(format!("{}/{}", self.base_url, key))
    }

    fn storage_type(&self) -> &'static str {
        "local"
    }
}

/// Create storage provider from configuration
pub async fn create_storage_provider(
    config: &StorageBackend,
) -> Result<DynStorageProvider, StorageError> {
    match config {
        StorageBackend::S3 { .. } => {
            tracing::info!("Initializing S3 storage provider");
            let provider = S3Storage::new().await?;
            Ok(Arc::new(provider))
        }
        StorageBackend::Local { base_path } => {
            let reports_path = base_path.join("reports");
            tracing::info!(path = %reports_path.display(), "Initializing local storage provider");

            // Create base directory
            tokio::fs::create_dir_all(&reports_path)
                .await
                .map_err(|e| {
                    StorageError(format!("Failed to create storage directory: {}", e))
                })?;

            let provider = LocalStorage::new(reports_path, "/api/reports/local".to_string());
            Ok(Arc::new(provider))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_storage_backend_defaults_to_local_without_s3() {
        // Save original env vars
        let original_storage = std::env::var("STORAGE_TYPE").ok();
        let original_bucket = std::env::var("S3_BUCKET").ok();

        // Clear env vars
        std::env::remove_var("STORAGE_TYPE");
        std::env::remove_var("S3_BUCKET");

        let backend = StorageBackend::from_env();
        assert!(matches!(backend, StorageBackend::Local { .. }));

        // Restore
        if let Some(v) = original_storage {
            std::env::set_var("STORAGE_TYPE", v);
        }
        if let Some(v) = original_bucket {
            std::env::set_var("S3_BUCKET", v);
        }
    }

    #[test]
    fn test_storage_backend_uses_s3_when_bucket_set() {
        let original_storage = std::env::var("STORAGE_TYPE").ok();
        let original_bucket = std::env::var("S3_BUCKET").ok();

        std::env::remove_var("STORAGE_TYPE");
        std::env::set_var("S3_BUCKET", "test-bucket");

        let backend = StorageBackend::from_env();
        assert!(matches!(backend, StorageBackend::S3 { bucket } if bucket == "test-bucket"));

        // Restore
        if let Some(v) = original_storage {
            std::env::set_var("STORAGE_TYPE", v);
        }
        std::env::remove_var("S3_BUCKET");
        if let Some(v) = original_bucket {
            std::env::set_var("S3_BUCKET", v);
        }
    }

    #[test]
    fn test_storage_backend_explicit_local() {
        let original_storage = std::env::var("STORAGE_TYPE").ok();
        let original_bucket = std::env::var("S3_BUCKET").ok();

        std::env::set_var("STORAGE_TYPE", "local");
        std::env::set_var("S3_BUCKET", "ignored-bucket");

        let backend = StorageBackend::from_env();
        assert!(matches!(backend, StorageBackend::Local { .. }));

        // Restore
        std::env::remove_var("STORAGE_TYPE");
        std::env::remove_var("S3_BUCKET");
        if let Some(v) = original_storage {
            std::env::set_var("STORAGE_TYPE", v);
        }
        if let Some(v) = original_bucket {
            std::env::set_var("S3_BUCKET", v);
        }
    }
}
