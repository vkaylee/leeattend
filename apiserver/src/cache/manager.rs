//! Cache Manager
//!
//! Bundles all cache instances and provides convenience methods.

use tracing::{debug, warn};
use uuid::Uuid;

use crate::cache::{ShiftCache, WorkspaceCache};

/// Manager for all cache instances.
/// Provides unified access to workspace and shift caches.
pub struct CacheManager {
    pub workspace: WorkspaceCache,
    pub shifts: ShiftCache,
    pub repo: Arc<dyn crate::repositories::CacheRepository>,
}

use crate::repositories::CacheRepository;
use std::sync::Arc;

impl CacheManager {
    pub fn new(
        cache: Arc<dyn CacheRepository>,
        workspace_repo: Arc<dyn crate::repositories::WorkspaceRepository>,
        shift_repo: Arc<dyn crate::repositories::shift_repository::ShiftRepository>,
    ) -> Self {
        Self {
            workspace: WorkspaceCache::new(Arc::clone(&cache), workspace_repo),
            shifts: ShiftCache::new(Arc::clone(&cache), shift_repo),
            repo: cache,
        }
    }

    /// Warm up caches for a workspace.
    ///
    /// Call at job start to pre-fetch static data.
    /// This reduces latency for first access during processing.
    pub async fn warm_up(&self, workspace_id: Uuid) {
        debug!(workspace_id = %workspace_id, "Warming up cache");

        // Warm up caches sequentially to prevent DB pool exhaustion
        let ws_result = self.workspace.get(workspace_id).await;
        let shifts_result = self.shifts.get_map(workspace_id).await;

        if let Err(e) = ws_result {
            warn!(workspace_id = %workspace_id, error = %e, "Workspace cache warm-up failed");
        }

        if let Err(e) = shifts_result {
            warn!(workspace_id = %workspace_id, error = %e, "Shifts cache warm-up failed");
        }
    }

    /// Invalidate all caches for a workspace.
    ///
    /// Call when workspace settings or shifts are modified.
    pub async fn invalidate_workspace(&self, workspace_id: Uuid) {
        debug!(workspace_id = %workspace_id, "Invalidating workspace caches");

        // Invalidate caches sequentially to prevent DB pool exhaustion
        let ws_result = self.workspace.invalidate(workspace_id).await;
        let shifts_result = self.shifts.invalidate(workspace_id).await;

        if let Err(e) = ws_result {
            warn!(workspace_id = %workspace_id, error = %e, "Workspace cache invalidation failed");
        }

        if let Err(e) = shifts_result {
            warn!(workspace_id = %workspace_id, error = %e, "Shifts cache invalidation failed");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repositories::{
        cache_repository::MockCacheRepository, shift_repository::MockShiftRepository,
        workspace_repository::MockWorkspaceRepository,
    };

    #[tokio::test]
    async fn test_cache_manager_warm_up() {
        let mut mock_cache = MockCacheRepository::new();
        // Setup mock to return ok for getting workspace and shifts
        mock_cache.expect_get_json().returning(|_| Ok(None));
        mock_cache.expect_set_json().returning(|_, _, _| Ok(()));

        let mut mock_workspace = MockWorkspaceRepository::new();
        mock_workspace
            .expect_get_workspace_by_id()
            .returning(|_| Box::pin(std::future::ready(Ok(None))));

        let mut mock_shift = MockShiftRepository::new();
        mock_shift
            .expect_get_workspace_shifts()
            .returning(|_, _, _| Box::pin(std::future::ready(Ok(vec![]))));

        let manager = CacheManager::new(
            Arc::new(mock_cache),
            Arc::new(mock_workspace),
            Arc::new(mock_shift),
        );

        manager.warm_up(Uuid::new_v4()).await;
        // if it doesn't panic, it's good
    }

    #[tokio::test]
    async fn test_cache_manager_invalidate() {
        let mut mock_cache = MockCacheRepository::new();
        mock_cache.expect_delete().returning(|_| Ok(()));
        mock_cache.expect_delete_pattern().returning(|_| Ok(()));

        let mock_workspace = MockWorkspaceRepository::new();
        let mock_shift = MockShiftRepository::new();

        let manager = CacheManager::new(
            Arc::new(mock_cache),
            Arc::new(mock_workspace),
            Arc::new(mock_shift),
        );

        manager.invalidate_workspace(Uuid::new_v4()).await;
    }

    #[tokio::test]
    async fn test_cache_manager_warm_up_errors() {
        let mut mock_cache = MockCacheRepository::new();
        mock_cache.expect_get_json().returning(|_| Ok(None));

        let mut mock_workspace = MockWorkspaceRepository::new();
        mock_workspace.expect_get_workspace_by_id().returning(|_| {
            Box::pin(std::future::ready(Err(crate::errors::AppError::Internal(
                "DB Error".to_string(),
            ))))
        });

        let mut mock_shift = MockShiftRepository::new();
        mock_shift
            .expect_get_workspace_shifts()
            .returning(|_, _, _| {
                Box::pin(std::future::ready(Err(crate::errors::AppError::Internal(
                    "DB Error".to_string(),
                ))))
            });

        let manager = CacheManager::new(
            Arc::new(mock_cache),
            Arc::new(mock_workspace),
            Arc::new(mock_shift),
        );

        manager.warm_up(Uuid::new_v4()).await;
    }

    #[tokio::test]
    async fn test_cache_manager_invalidate_errors() {
        let mut mock_cache = MockCacheRepository::new();
        mock_cache
            .expect_delete()
            .returning(|_| Err("Cache Error".to_string()));
        mock_cache
            .expect_delete_pattern()
            .returning(|_| Err("Cache Error".to_string()));

        let mock_workspace = MockWorkspaceRepository::new();
        let mock_shift = MockShiftRepository::new();

        let manager = CacheManager::new(
            Arc::new(mock_cache),
            Arc::new(mock_workspace),
            Arc::new(mock_shift),
        );

        manager.invalidate_workspace(Uuid::new_v4()).await;
    }
}
