//! Redis Cache Layer
//!
//! Generic cache wrapper with get/set/get_or_fetch operations.

use redis::AsyncCommands;
use serde::{de::DeserializeOwned, Serialize};
use std::future::Future;
use std::time::Duration;
use tracing::{debug, warn};

use crate::queue::RedisPool;

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
    ///
    /// Returns None if key doesn't exist or deserialization fails.
    pub async fn get<T: DeserializeOwned>(&self, key: &str) -> Option<T> {
        let mut conn = self.pool.conn();
        let json: Option<String> = match conn.get(key).await {
            Ok(val) => val,
            Err(e) => {
                warn!(key = key, error = %e, "Redis GET failed");
                return None;
            }
        };

        match json {
            Some(s) => match serde_json::from_str(&s) {
                Ok(val) => {
                    debug!(key = key, "Cache hit");
                    Some(val)
                }
                Err(e) => {
                    warn!(key = key, error = %e, "Cache deserialization failed");
                    None
                }
            },
            None => {
                debug!(key = key, "Cache miss");
                None
            }
        }
    }

    /// Set a value in cache with TTL, serializing to JSON.
    pub async fn set<T: Serialize>(&self, key: &str, value: &T, ttl: Duration) -> Result<(), String> {
        let json = serde_json::to_string(value)
            .map_err(|e| format!("Serialize error: {}", e))?;

        let mut conn = self.pool.conn();
        conn.set_ex::<_, _, ()>(key, json, ttl.as_secs())
            .await
            .map_err(|e| format!("Redis SET error: {}", e))?;

        debug!(key = key, ttl_secs = ttl.as_secs(), "Cache set");
        Ok(())
    }

    /// Delete a key from cache.
    pub async fn delete(&self, key: &str) -> Result<(), String> {
        let mut conn = self.pool.conn();
        conn.del::<_, ()>(key)
            .await
            .map_err(|e| format!("Redis DEL error: {}", e))?;

        debug!(key = key, "Cache invalidated");
        Ok(())
    }

    /// Get value from cache, or fetch from source and cache the result.
    ///
    /// Implements cache-aside pattern:
    /// 1. Try to get from cache
    /// 2. On miss, call fetch_fn to get value
    /// 3. Store in cache (fire-and-forget, don't fail if cache write fails)
    /// 4. Return value
    pub async fn get_or_fetch<T, F, Fut>(
        &self,
        key: &str,
        ttl: Duration,
        fetch_fn: F,
    ) -> Result<T, String>
    where
        T: Serialize + DeserializeOwned + Clone,
        F: FnOnce() -> Fut,
        Fut: Future<Output = Result<T, String>>,
    {
        // Try cache first
        if let Some(cached) = self.get::<T>(key).await {
            return Ok(cached);
        }

        // Cache miss - fetch from source
        let value = fetch_fn().await?;

        // Store in cache (fire and forget, log errors but don't fail)
        if let Err(e) = self.set(key, &value, ttl).await {
            warn!(key = key, error = %e, "Failed to cache value");
        }

        Ok(value)
    }
}

#[cfg(test)]
mod tests {
    use serde::{Deserialize, Serialize};
    use std::time::Duration;

    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
    struct TestData {
        id: u32,
        name: String,
    }

    #[test]
    fn test_workspace_key_format() {
        let workspace_id = uuid::Uuid::new_v4();
        let key = format!("cache:workspace:{}", workspace_id);
        assert!(key.starts_with("cache:workspace:"));
        assert!(key.contains(&workspace_id.to_string()));
    }

    #[test]
    fn test_shift_key_format() {
        let workspace_id = uuid::Uuid::new_v4();
        let key = format!("cache:shifts:{}", workspace_id);
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
}

