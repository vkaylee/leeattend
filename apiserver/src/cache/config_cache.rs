//! Config Cache
//!
//! Caches workspace configurations and policies using Redis cache-aside.

use std::sync::Arc;
use uuid::Uuid;

use crate::cache::DEFAULT_CACHE_TTL;
use crate::diesel_pool::TenantAuthToken;
use crate::models::WorkspaceConfigEntry;
use crate::repositories::{CacheRepository, CacheRepositoryExt, WorkspaceConfigRepository};

/// Cache for workspace configurations.
pub struct ConfigCache {
    cache: Arc<dyn CacheRepository>,
    repo: Arc<dyn WorkspaceConfigRepository>,
}

impl ConfigCache {
    /// Create a new ConfigCache.
    pub fn new(cache: Arc<dyn CacheRepository>, repo: Arc<dyn WorkspaceConfigRepository>) -> Self {
        Self { cache, repo }
    }

    /// Generate cache key for a workspace's config entries.
    fn key(workspace_id: Uuid) -> String {
        format!("cache:workspace_config:{workspace_id}")
    }

    /// Get all config entries for a workspace, using cache-aside pattern.
    pub async fn get_entries(
        &self,
        auth_token: &TenantAuthToken,
        workspace_id: Uuid,
    ) -> Result<Vec<WorkspaceConfigEntry>, String> {
        let key = Self::key(workspace_id);
        let repo = Arc::clone(&self.repo);
        let token = auth_token.clone();

        self.cache
            .get_or_fetch(&key, DEFAULT_CACHE_TTL, || async move {
                repo.list_entries(&token, workspace_id, None)
                    .await
                    .map_err(|e| format!("Failed to fetch workspace config entries: {e}"))
            })
            .await
    }

    /// Invalidate cached workspace config entries.
    pub async fn invalidate(&self, workspace_id: Uuid) -> Result<(), String> {
        self.cache.delete(&Self::key(workspace_id)).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_key_generation() {
        let workspace_id = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap();
        let key = ConfigCache::key(workspace_id);
        assert_eq!(key, "cache:workspace_config:550e8400-e29b-41d4-a716-446655440000");
    }
}
