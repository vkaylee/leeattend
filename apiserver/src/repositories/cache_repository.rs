use async_trait::async_trait;
use std::time::Duration;

/// Abstract interface for Cache operations.
///
/// This allows us to use Dependency Injection to decouple services
/// from concrete Redis connections, enabling fast, isolated unit tests.
/// We deal with raw JSON strings to avoid complex mockall issues with generics.
#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait CacheRepository: Send + Sync {
    /// Get a value from cache as a raw JSON string
    async fn get_json(&self, key: &str) -> Result<Option<String>, String>;

    /// Set a value in cache as a raw JSON string
    async fn set_json(&self, key: &str, value: &str, ttl: Duration) -> Result<(), String>;

    /// Delete a key from cache
    async fn delete(&self, key: &str) -> Result<(), String>;

    /// Delete keys matching a pattern
    async fn delete_pattern(&self, pattern: &str) -> Result<(), String>;

    /// Increment a numeric value in cache (returns the new value)
    async fn increment(&self, key: &str) -> Result<i64, String>;

    /// Get all fields and values of a hash stored at key
    async fn hgetall(&self, key: &str)
        -> Result<std::collections::HashMap<String, String>, String>;
}

use serde::{de::DeserializeOwned, Serialize};
use std::future::Future;
use tracing::{debug, warn};

/// Extension trait providing generic serialization/deserialization wrappers
/// on top of `CacheRepository`.
///
/// We separate this into an extension trait to avoid confusing `mockall`
/// with generic methods in the mock generation.
#[async_trait]
pub trait CacheRepositoryExt {
    async fn get<T: DeserializeOwned>(&self, key: &str) -> Option<T>;

    async fn set<T: Serialize + Sync>(
        &self,
        key: &str,
        value: &T,
        ttl: Duration,
    ) -> Result<(), String>;

    async fn get_or_fetch<T, F, Fut>(
        &self,
        key: &str,
        ttl: Duration,
        fetch_fn: F,
    ) -> Result<T, String>
    where
        T: Serialize + DeserializeOwned + Clone + Send + Sync,
        F: FnOnce() -> Fut + Send,
        Fut: Future<Output = Result<T, String>> + Send;
}

#[async_trait]
impl<C: CacheRepository + ?Sized> CacheRepositoryExt for C {
    async fn get<T: DeserializeOwned>(&self, key: &str) -> Option<T> {
        let json = match self.get_json(key).await {
            Ok(Some(s)) => s,
            Ok(None) => return None,
            Err(e) => {
                warn!(key = key, error = %e, "Cache GET failed");
                return None;
            }
        };

        match serde_json::from_str(&json) {
            Ok(val) => {
                debug!(key = key, "Cache hit");
                Some(val)
            }
            Err(e) => {
                warn!(key = key, error = %e, "Cache deserialization failed");
                None
            }
        }
    }

    async fn set<T: Serialize + Sync>(
        &self,
        key: &str,
        value: &T,
        ttl: Duration,
    ) -> Result<(), String> {
        let json = serde_json::to_string(value).map_err(|e| format!("Serialize error: {e}"))?;
        self.set_json(key, &json, ttl).await?;
        debug!(key = key, ttl_secs = ttl.as_secs(), "Cache set");
        Ok(())
    }

    async fn get_or_fetch<T, F, Fut>(
        &self,
        key: &str,
        ttl: Duration,
        fetch_fn: F,
    ) -> Result<T, String>
    where
        T: Serialize + DeserializeOwned + Clone + Send + Sync,
        F: FnOnce() -> Fut + Send,
        Fut: Future<Output = Result<T, String>> + Send,
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
