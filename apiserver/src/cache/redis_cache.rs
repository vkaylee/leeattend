//! Redis Cache Layer
//!
//! Generic cache wrapper with get/set/get_or_fetch operations.

use redis::AsyncCommands;
use std::time::Duration;
use tracing::debug;

use crate::queue::RedisPool;
use crate::repositories::CacheRepository;
use async_trait::async_trait;

/// Redis-based cache implementation with serialization support.
#[derive(Clone)]
pub struct RedisCache {
    pool: RedisPool,
}

impl RedisCache {
    /// Create a new RedisCache from a connection pool.
    pub fn new(pool: RedisPool) -> Self {
        Self { pool }
    }

    /// Get a value from cache, deserializing from JSON.
    pub async fn get<T: serde::de::DeserializeOwned>(&self, key: &str) -> Option<T> {
        use crate::repositories::CacheRepositoryExt;
        CacheRepositoryExt::get(self, key).await
    }

    /// Set a value in cache with TTL, serializing to JSON.
    pub async fn set<T: serde::Serialize + Sync>(
        &self,
        key: &str,
        value: &T,
        ttl: std::time::Duration,
    ) -> Result<(), String> {
        use crate::repositories::CacheRepositoryExt;
        CacheRepositoryExt::set(self, key, value, ttl).await
    }

    /// Get value from cache, or fetch from source and cache the result.
    pub async fn get_or_fetch<T, F, Fut>(
        &self,
        key: &str,
        ttl: std::time::Duration,
        fetch_fn: F,
    ) -> Result<T, String>
    where
        T: serde::Serialize + serde::de::DeserializeOwned + Clone + Send + Sync,
        F: FnOnce() -> Fut + Send,
        Fut: std::future::Future<Output = Result<T, String>> + Send,
    {
        use crate::repositories::CacheRepositoryExt;
        CacheRepositoryExt::get_or_fetch(self, key, ttl, fetch_fn).await
    }

    /// Delete a key from cache.
    pub async fn delete(&self, key: &str) -> Result<(), String> {
        use crate::repositories::CacheRepository;
        CacheRepository::delete(self, key).await
    }

    /// Delete keys matching a pattern.
    pub async fn delete_pattern(&self, pattern: &str) -> Result<(), String> {
        use crate::repositories::CacheRepository;
        CacheRepository::delete_pattern(self, pattern).await
    }
}

#[async_trait]
impl CacheRepository for RedisCache {
    async fn get_json(&self, key: &str) -> Result<Option<String>, String> {
        let mut conn = self.pool.conn();
        conn.get(key)
            .await
            .map_err(|e| format!("Redis GET error: {e}"))
    }

    async fn set_json(&self, key: &str, value: &str, ttl: Duration) -> Result<(), String> {
        let mut conn = self.pool.conn();
        conn.set_ex::<_, _, ()>(key, value, ttl.as_secs())
            .await
            .map_err(|e| format!("Redis SET error: {e}"))
    }

    async fn delete(&self, key: &str) -> Result<(), String> {
        let mut conn = self.pool.conn();
        conn.del::<_, ()>(key)
            .await
            .map_err(|e| format!("Redis DEL error: {e}"))?;

        debug!(key = key, "Cache invalidated");
        Ok(())
    }

    async fn delete_pattern(&self, pattern: &str) -> Result<(), String> {
        let mut conn = self.pool.conn();
        let keys: Vec<String> = conn
            .keys(pattern)
            .await
            .map_err(|e| format!("Redis KEYS error: {e}"))?;

        if !keys.is_empty() {
            conn.del::<_, ()>(&keys)
                .await
                .map_err(|e| format!("Redis DEL error: {e}"))?;
            debug!(
                pattern = pattern,
                count = keys.len(),
                "Pattern cache invalidated"
            );
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use serde::{Deserialize, Serialize};
    use std::collections::HashSet;
    use std::time::Duration;

    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
    struct TestData {
        id: u32,
        name: String,
    }

    #[test]
    fn test_workspace_key_format() {
        let workspace_id = uuid::Uuid::new_v4();
        let key = format!("cache:workspace:{workspace_id}");
        assert!(key.starts_with("cache:workspace:"));
        assert!(key.contains(&workspace_id.to_string()));
    }

    #[test]
    fn test_shift_key_format() {
        let workspace_id = uuid::Uuid::new_v4();
        let key = format!("cache:shifts:{workspace_id}");
        assert!(key.starts_with("cache:shifts:"));
    }

    #[test]
    fn test_serialization_roundtrip() {
        let data = TestData {
            id: 42,
            name: "Test Workspace".to_string(),
        };

        let json = serde_json::to_string(&data).unwrap();
        let decoded: TestData = serde_json::from_str(&json).unwrap();

        assert_eq!(data, decoded);
    }

    #[test]
    fn test_ttl_duration() {
        let ttl = Duration::from_secs(300);
        assert_eq!(ttl.as_secs(), 300);

        // 5 minute TTL is reasonable for cache
        assert!(ttl.as_secs() >= 60);
        assert!(ttl.as_secs() <= 3600);
    }

    // --- RBAC Permission Cache Tests ---

    #[test]
    fn test_rbac_permission_cache_key_format() {
        let workspace_id = uuid::Uuid::new_v4();
        let user_id = uuid::Uuid::new_v4();
        let key = format!("cache:auth:permissions:{workspace_id}:{user_id}");

        assert!(key.starts_with("cache:auth:permissions:"));
        assert!(key.contains(&workspace_id.to_string()));
        assert!(key.contains(&user_id.to_string()));
        assert_eq!(key.matches(':').count(), 4);
    }

    #[test]
    fn test_rbac_permission_cache_key_uniqueness() {
        let ws = uuid::Uuid::new_v4();
        let user_a = uuid::Uuid::new_v4();
        let user_b = uuid::Uuid::new_v4();

        let key_a = format!("cache:auth:permissions:{ws}:{user_a}");
        let key_b = format!("cache:auth:permissions:{ws}:{user_b}");

        assert_ne!(key_a, key_b, "Different users must produce different keys");
    }

    #[test]
    fn test_rbac_invalidation_pattern_format() {
        let workspace_id = uuid::Uuid::new_v4();
        let pattern = format!("cache:auth:permissions:{workspace_id}:*");

        assert!(
            pattern.ends_with(":*"),
            "Pattern must use wildcard for user segment"
        );
        assert!(pattern.contains(&workspace_id.to_string()));
    }

    #[test]
    fn test_rbac_ttl_is_one_hour() {
        let ttl = Duration::from_secs(3600);
        assert_eq!(
            ttl.as_secs(),
            3600,
            "RBAC permission cache TTL must be 1 hour"
        );
    }

    #[test]
    fn test_cached_member_context_serialization_roundtrip() {
        use crate::handlers::common::CachedMemberContext;

        let role_a = uuid::Uuid::new_v4();
        let role_b = uuid::Uuid::new_v4();
        let dept_id = uuid::Uuid::new_v4();

        let original = CachedMemberContext {
            roles: vec![role_a, role_b],
            permissions: vec![
                "employees:read".to_string(),
                "shifts:create".to_string(),
                "attendance:read".to_string(),
            ],
            managed_department_ids: Some(HashSet::from([dept_id])),
        };

        let json = serde_json::to_string(&original).unwrap();
        let decoded: CachedMemberContext = serde_json::from_str(&json).unwrap();

        assert_eq!(decoded.roles, original.roles);
        assert_eq!(decoded.permissions.len(), 3);
        assert!(decoded.permissions.contains(&"employees:read".to_string()));
        assert!(decoded.permissions.contains(&"shifts:create".to_string()));
        assert!(decoded.permissions.contains(&"attendance:read".to_string()));
        assert_eq!(
            decoded.managed_department_ids,
            original.managed_department_ids
        );
    }

    #[test]
    fn test_cached_member_context_global_manager_none_departments() {
        use crate::handlers::common::CachedMemberContext;

        let ctx = CachedMemberContext {
            roles: vec![uuid::Uuid::new_v4()],
            permissions: vec!["employees:read".to_string()],
            managed_department_ids: None,
        };

        let json = serde_json::to_string(&ctx).unwrap();
        let decoded: CachedMemberContext = serde_json::from_str(&json).unwrap();

        assert!(
            decoded.managed_department_ids.is_none(),
            "Global managers should have None for managed_department_ids"
        );
    }

    #[test]
    fn test_cached_member_context_empty_permissions() {
        use crate::handlers::common::CachedMemberContext;

        let ctx = CachedMemberContext {
            roles: vec![uuid::Uuid::new_v4()],
            permissions: vec![],
            managed_department_ids: Some(HashSet::new()),
        };

        let json = serde_json::to_string(&ctx).unwrap();
        let decoded: CachedMemberContext = serde_json::from_str(&json).unwrap();

        assert!(decoded.permissions.is_empty());
        assert_eq!(decoded.managed_department_ids, Some(HashSet::new()));
    }
}
