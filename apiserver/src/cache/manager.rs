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
    // Future extensions: patterns, policies, employees, etc.
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
        }
    }

    /// Warm up caches for a workspace.
    ///
    /// Call at job start to pre-fetch static data.
    /// This reduces latency for first access during processing.
    pub async fn warm_up(&self, workspace_id: Uuid) {
        debug!(workspace_id = %workspace_id, "Warming up cache");

        // Fire-and-forget cache warming
        // Use join! to run in parallel
        let (ws_result, shifts_result) = tokio::join!(
            self.workspace.get(workspace_id),
            self.shifts.get_map(workspace_id),
        );

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

        let (ws_result, shifts_result) = tokio::join!(
            self.workspace.invalidate(workspace_id),
            self.shifts.invalidate(workspace_id),
        );

        if let Err(e) = ws_result {
            warn!(workspace_id = %workspace_id, error = %e, "Workspace cache invalidation failed");
        }

        if let Err(e) = shifts_result {
            warn!(workspace_id = %workspace_id, error = %e, "Shifts cache invalidation failed");
        }
    }
}
