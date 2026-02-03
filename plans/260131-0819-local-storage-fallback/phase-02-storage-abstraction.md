# Phase 2: Storage Abstraction Layer

## Context Links
- [s3_client.rs](../../../apiserver/src/services/s3_client.rs) - Current S3 implementation
- [Code Standards](../../../docs/code-standards.md) - Rust patterns

## Overview
- **Priority**: High (core implementation)
- **Current Status**: Pending
- **Description**: Create unified storage trait with S3 and Local implementations

## Key Insights

Current S3Client interface:
```rust
impl S3Client {
    pub async fn new() -> Result<Self, S3Error>;
    pub async fn upload_report(&self, key: &str, data: Vec<u8>, content_type: &str) -> Result<String, S3Error>;
    pub async fn generate_presigned_url(&self, key: &str, expires_in: Duration) -> Result<String, S3Error>;
}
```

Need unified trait that both S3 and Local implementations can satisfy.

## Requirements

### Functional
- `StorageProvider` trait with upload and URL generation
- `S3Storage` impl wrapping existing S3Client
- `LocalStorage` impl using tokio::fs
- Factory function to create provider from `StorageBackend` config

### Non-Functional
- Async trait using `#[async_trait]` (already in deps)
- Error handling consistent with existing patterns
- Thread-safe (Arc-wrapped provider)

## Architecture

```
┌─────────────────────────────────────────────────────────┐
│                  StorageProvider (Trait)                 │
├─────────────────────────────────────────────────────────┤
│  async fn upload(&self, key, data, content_type) -> Result<String>  │
│  async fn get_download_url(&self, key) -> Result<String>            │
│  fn storage_type(&self) -> &'static str                              │
└───────────────────┬─────────────────────┬───────────────┘
                    │                     │
         ┌──────────┴──────────┐ ┌────────┴────────┐
         │    S3Storage        │ │  LocalStorage   │
         ├─────────────────────┤ ├─────────────────┤
         │ - s3_client: S3Client│ │ - base_path     │
         │ - bucket: String    │ │ - base_url: String│
         └─────────────────────┘ └─────────────────┘
```

## Related Code Files

### Files to Create
- `apiserver/src/services/storage.rs` - New storage abstraction

### Files to Modify
- `apiserver/src/services/mod.rs` - Export storage module

## Implementation Steps

1. **Create storage.rs with trait definition**
   ```rust
   //! Storage Provider Abstraction
   //!
   //! Unified interface for S3 and local filesystem storage.
   //! Enables fallback to local storage when S3 is not configured.

   use async_trait::async_trait;
   use std::path::PathBuf;
   use std::sync::Arc;
   use std::time::Duration;

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
   ```

2. **Implement S3Storage**
   ```rust
   use crate::services::S3Client;

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
           const PRESIGNED_URL_EXPIRY: Duration = Duration::from_secs(86400);
           self.client
               .generate_presigned_url(key, PRESIGNED_URL_EXPIRY)
               .await
               .map_err(|e| StorageError(e.to_string()))
       }

       fn storage_type(&self) -> &'static str {
           "s3"
       }
   }
   ```

3. **Implement LocalStorage**
   ```rust
   use tokio::fs;
   use tokio::io::AsyncWriteExt;

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

           // Security: ensure path stays within base_path
           let canonical_base = self.base_path.canonicalize().unwrap_or_else(|_| self.base_path.clone());
           if let Some(parent) = file_path.parent() {
               fs::create_dir_all(parent)
                   .await
                   .map_err(|e| StorageError(format!("Failed to create directory: {}", e)))?;
           }

           // Verify path is within base (prevent traversal)
           let canonical_file = file_path.canonicalize().unwrap_or_else(|_| file_path.clone());
           if !canonical_file.starts_with(&canonical_base) {
               return Err(StorageError("Path traversal detected".to_string()));
           }

           Ok(file_path)
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
           // Format: /api/reports/local/{encoded_key}
           let encoded_key = urlencoding::encode(key);
           Ok(format!("{}/{}", self.base_url, encoded_key))
       }

       fn storage_type(&self) -> &'static str {
           "local"
       }
   }
   ```

4. **Add factory function**
   ```rust
   use crate::StorageBackend;

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
                   .map_err(|e| StorageError(format!("Failed to create storage directory: {}", e)))?;

               let provider = LocalStorage::new(
                   reports_path,
                   "/api/reports/local".to_string(),
               );
               Ok(Arc::new(provider))
           }
       }
   }
   ```

5. **Update services/mod.rs**
   ```rust
   pub mod storage;
   pub use storage::{StorageProvider, DynStorageProvider, StorageError, create_storage_provider};
   ```

## Todo List

- [ ] Create `apiserver/src/services/storage.rs`
- [ ] Define `StorageProvider` trait
- [ ] Implement `S3Storage` wrapping existing client
- [ ] Implement `LocalStorage` with atomic writes
- [ ] Add path traversal protection
- [ ] Implement `create_storage_provider` factory
- [ ] Export from `services/mod.rs`

## Success Criteria

- [ ] Both impls satisfy `StorageProvider` trait
- [ ] Local storage creates directory structure
- [ ] Atomic file writes (temp file + rename)
- [ ] Path traversal prevention works
- [ ] Factory returns correct impl based on config

## Security Considerations

- **Path Traversal**: Validate all keys stay within base_path using canonicalize()
- **Atomic Writes**: Use temp file + rename to prevent partial file reads
- **Permissions**: Files created with default umask (readable by server user)

## Next Steps

Proceed to [Phase 3: Worker Handler Integration](./phase-03-worker-integration.md)
