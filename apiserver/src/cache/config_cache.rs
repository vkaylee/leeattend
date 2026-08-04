//! Config Cache
//!
//! Caches workspace configurations and policies using Redis cache-aside.

use chrono::NaiveDate;
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
        let token = *auth_token;

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

    /// Get effective config from memory cache-aside.
    pub async fn get_effective_config(
        &self,
        auth_token: &TenantAuthToken,
        workspace_id: Uuid,
        config_key: &str,
        scope_type: &str,
        scope_id: Option<Uuid>,
        target_date: NaiveDate,
    ) -> Result<Option<WorkspaceConfigEntry>, String> {
        let entries = self.get_entries(auth_token, workspace_id).await?;

        let mut filtered: Vec<WorkspaceConfigEntry> = entries
            .into_iter()
            .filter(|entry| {
                entry.config_key() == config_key
                    && entry.scope_type() == scope_type
                    && entry.effective_from() <= target_date
                    && match (scope_id, entry.scope_id()) {
                        (Some(id1), Some(id2)) => id1 == id2,
                        (None, None) => true,
                        _ => false,
                    }
            })
            .collect();

        filtered.sort_by(|a, b| b.effective_from().cmp(&a.effective_from()));

        if let Some(entry) = filtered.into_iter().next() {
            if let Some(to) = entry.effective_to() {
                if target_date > to {
                    return Ok(None);
                }
            }
            return Ok(Some(entry));
        }

        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::*;
    use crate::repositories::cache_repository::MockCacheRepository;
    use crate::repositories::workspace_config_repository::MockWorkspaceConfigRepository;
    use chrono::Utc;

    #[test]
    fn test_cache_key_generation() {
        let workspace_id = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap();
        let key = ConfigCache::key(workspace_id);
        assert_eq!(
            key,
            "cache:workspace_config:550e8400-e29b-41d4-a716-446655440000"
        );
    }

    #[tokio::test]
    async fn test_get_effective_config_cached() {
        let mut mock_cache = MockCacheRepository::new();
        let workspace_id = Uuid::new_v4();

        let entry = WorkspaceConfigEntry::new(NewWorkspaceConfigEntryParams {
        id: Uuid::new_v4(),
        workspace_id: workspace_id,
        scope_type: "workspace".to_string(),
        scope_id: None,
        config_key: "leave.conflict_policy".to_string(),
        value_int: None,
        value_decimal: None,
        value_text: Some("auto_cancel".to_string()),
        value_bool: None,
        value_json: None,
        effective_from: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
        effective_to: None,
        created_by: None,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    });

        let entries_json = serde_json::to_string(&vec![entry.clone()]).unwrap();
        let entries_json_clone = entries_json.clone();

        mock_cache
            .expect_get_json()
            .withf(move |k| k.contains(&workspace_id.to_string()))
            .returning(move |_| Ok(Some(entries_json_clone.clone())));

        let mock_repo = MockWorkspaceConfigRepository::new();
        let config_cache = ConfigCache::new(Arc::new(mock_cache), Arc::new(mock_repo));

        let auth_token = TenantAuthToken::system_override(workspace_id);
        let result = config_cache
            .get_effective_config(
                &auth_token,
                workspace_id,
                "leave.conflict_policy",
                "workspace",
                None,
                NaiveDate::from_ymd_opt(2024, 6, 1).unwrap(),
            )
            .await;

        assert!(result.is_ok());
        let opt_entry = result.unwrap();
        assert!(opt_entry.is_some());
        assert_eq!(opt_entry.unwrap().value_text(), Some("auto_cancel"));
    }
}
