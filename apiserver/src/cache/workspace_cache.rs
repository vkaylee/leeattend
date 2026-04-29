//! Workspace Cache
//!
//! Caches workspace configuration for attendance calculations.

use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

use crate::cache::{RedisCache, DEFAULT_CACHE_TTL};
use crate::models::EarlyArrivalStrategy;

/// Cached workspace configuration.
/// Contains fields needed for attendance calculations.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedWorkspaceConfig {
    pub id: Uuid,
    pub timezone: String,
    pub early_arrival_strategy: EarlyArrivalStrategy,
}

/// Cache for workspace configuration.
pub struct WorkspaceCache {
    cache: RedisCache,
    db: PgPool,
}

impl WorkspaceCache {
    /// Create a new WorkspaceCache.
    pub fn new(cache: RedisCache, db: PgPool) -> Self {
        Self { cache, db }
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
        let db = self.db.clone();

        self.cache
            .get_or_fetch(&key, DEFAULT_CACHE_TTL, || async move {
                let row = sqlx::query!(
                    r#"
                    SELECT
                        id,
                        timezone,
                        early_arrival_strategy as "early_arrival_strategy: EarlyArrivalStrategy"
                    FROM workspaces
                    WHERE id = $1
                    "#,
                    workspace_id
                )
                .fetch_one(&db)
                .await
                .map_err(|e| format!("Failed to fetch workspace config: {e}"))?;

                Ok(CachedWorkspaceConfig {
                    id: row.id,
                    timezone: row.timezone,
                    early_arrival_strategy: row.early_arrival_strategy,
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
            early_arrival_strategy: EarlyArrivalStrategy::Allow,
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
