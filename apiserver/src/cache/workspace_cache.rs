//! Workspace Cache
//!
//! Caches workspace configuration for attendance calculations.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::cache::DEFAULT_CACHE_TTL;
use crate::repositories::{CacheRepository, CacheRepositoryExt, WorkspaceRepository};
use std::sync::Arc;

/// Cached workspace configuration.
/// Contains fields needed for attendance calculations.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedWorkspaceConfig {
    pub id: Uuid,
    pub timezone: String,
}

/// Cache for workspace configuration.
pub struct WorkspaceCache {
    cache: Arc<dyn CacheRepository>,
    repo: Arc<dyn WorkspaceRepository>,
}

impl WorkspaceCache {
    /// Create a new WorkspaceCache.
    pub fn new(cache: Arc<dyn CacheRepository>, repo: Arc<dyn WorkspaceRepository>) -> Self {
        Self { cache, repo }
    }

    /// Generate cache key for a workspace.
    fn key(workspace_id: Uuid) -> String {
        format!("cache:workspace:{workspace_id}")
    }

    /// Get workspace config, using cache-aside pattern.
    ///
    /// 1. Check Redis cache
    /// 2. On miss, fetch from PostgreSQL
    /// 3. Store in cache for future requests
    pub async fn get(&self, workspace_id: Uuid) -> Result<CachedWorkspaceConfig, String> {
        let key = Self::key(workspace_id);
        let repo = self.repo.clone();

        self.cache
            .get_or_fetch(&key, DEFAULT_CACHE_TTL, || async move {
                let ws = repo
                    .get_workspace_by_id(workspace_id)
                    .await
                    .map_err(|e| format!("Failed to fetch workspace config: {e}"))?
                    .ok_or_else(|| "Workspace not found".to_string())?;

                Ok(CachedWorkspaceConfig {
                    id: ws.id,
                    timezone: ws.timezone,
                })
            })
            .await
    }

    /// Invalidate cached workspace config.
    /// Call this when workspace settings are updated.
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
        let key = WorkspaceCache::key(workspace_id);
        assert_eq!(key, "cache:workspace:550e8400-e29b-41d4-a716-446655440000");
    }

    #[test]
    fn test_cached_config_serialization() {
        let config = CachedWorkspaceConfig {
            id: Uuid::new_v4(),
            timezone: "Asia/Ho_Chi_Minh".to_string(),
        };

        let json = serde_json::to_string(&config).unwrap();
        let decoded: CachedWorkspaceConfig = serde_json::from_str(&json).unwrap();

        assert_eq!(config.id, decoded.id);
        assert_eq!(config.timezone, decoded.timezone);
    }

    #[test]
    fn test_default_ttl() {
        // 5 minutes is reasonable for workspace config
        assert_eq!(DEFAULT_CACHE_TTL.as_secs(), 300);
    }
}
