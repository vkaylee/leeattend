use crate::errors::AppResult;
use crate::repositories::storage_config_repo::StorageConfigRepository;
use crate::services::storage::s3_storage::{S3Config, S3Storage};
use crate::services::storage::ObjectStorage;
use moka::future::Cache;
use std::sync::Arc;
use std::time::Duration;
use uuid::Uuid;

#[derive(Clone)]
pub struct StorageResolver {
    central: Arc<dyn ObjectStorage>,
    config_repo: Arc<StorageConfigRepository>,
    cache: Cache<Uuid, Arc<dyn ObjectStorage>>,
}

impl StorageResolver {
    pub fn new(central: Arc<dyn ObjectStorage>, config_repo: Arc<StorageConfigRepository>) -> Self {
        let cache = Cache::builder()
            .time_to_live(Duration::from_secs(3600)) // 1 hour TTL
            .max_capacity(1000)
            .build();

        Self {
            central,
            config_repo,
            cache,
        }
    }

    /// Get storage for workspace
    /// It queries cache first, then DB config. Falls back to central.
    pub async fn resolve(&self, workspace_id: Uuid) -> AppResult<Arc<dyn ObjectStorage>> {
        if let Some(storage) = self.cache.get(&workspace_id).await {
            return Ok(storage);
        }

        let db_config = self.config_repo.get_active_config(workspace_id).await?;

        if let Some(cfg) = db_config {
            let s3_config = S3Config {
                endpoint_url: cfg.endpoint_url,
                region: cfg.region,
                bucket_name: cfg.bucket_name,
                access_key: cfg.access_key_enc,
                secret_key: cfg.secret_key_enc,
                use_path_style: cfg.use_path_style,
            };

            let s3_storage = S3Storage::new(s3_config).await?;
            let storage: Arc<dyn ObjectStorage> = Arc::new(s3_storage);

            self.cache.insert(workspace_id, Arc::clone(&storage)).await;
            return Ok(storage);
        }

        // Fallback to central storage
        Ok(Arc::clone(&self.central))
    }

    /// Get central storage (for system-level files)
    pub fn central(&self) -> Arc<dyn ObjectStorage> {
        Arc::clone(&self.central)
    }

    /// Invalidate cached workspace storage (on config change)
    pub async fn invalidate(&self, workspace_id: Uuid) {
        self.cache.invalidate(&workspace_id).await;
    }
}
