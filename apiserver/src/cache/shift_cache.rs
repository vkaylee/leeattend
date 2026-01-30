//! Shift Cache
//!
//! Caches shift data for O(1) lookup during attendance calculations.

use chrono::NaiveTime;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::collections::HashMap;
use uuid::Uuid;

use crate::cache::{RedisCache, DEFAULT_CACHE_TTL};

/// Cached shift data.
/// Contains all fields needed for attendance calculations.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedShift {
    pub id: Uuid,
    pub name: String,
    pub start_time: NaiveTime,
    pub end_time: NaiveTime,
    pub break_duration_minutes: Option<i32>,
    pub break_start: Option<NaiveTime>,
    pub break_end: Option<NaiveTime>,
    pub is_break_auto_deduct: Option<bool>,
    pub allow_early_ot: Option<bool>,
    pub early_punch_in_grace_period_minutes: Option<i32>,
}

/// Cache for shift data.
pub struct ShiftCache {
    cache: RedisCache,
    db: PgPool,
}

impl ShiftCache {
    /// Create a new ShiftCache.
    pub fn new(cache: RedisCache, db: PgPool) -> Self {
        Self { cache, db }
    }

    /// Generate cache key for shifts of a workspace.
    fn key(workspace_id: Uuid) -> String {
        format!("cache:shifts:{}", workspace_id)
    }

    /// Get all shifts for workspace as a HashMap for O(1) lookup by shift ID.
    ///
    /// Returns cached data if available, otherwise fetches from DB.
    pub async fn get_map(&self, workspace_id: Uuid) -> Result<HashMap<Uuid, CachedShift>, String> {
        let key = Self::key(workspace_id);
        let db = self.db.clone();

        let shifts: Vec<CachedShift> = self
            .cache
            .get_or_fetch(&key, DEFAULT_CACHE_TTL, || async move {
                let rows = sqlx::query!(
                    r#"
                    SELECT
                        id,
                        name,
                        start_time,
                        end_time,
                        break_duration_minutes,
                        break_start,
                        break_end,
                        is_break_auto_deduct,
                        allow_early_ot,
                        early_punch_in_grace_period_minutes
                    FROM shifts
                    WHERE workspace_id = $1 AND is_active = true
                    "#,
                    workspace_id
                )
                .fetch_all(&db)
                .await
                .map_err(|e| format!("Failed to fetch shifts: {}", e))?;

                let shifts: Vec<CachedShift> = rows
                    .into_iter()
                    .map(|row| CachedShift {
                        id: row.id,
                        name: row.name,
                        start_time: row.start_time,
                        end_time: row.end_time,
                        break_duration_minutes: Some(row.break_duration_minutes),
                        break_start: row.break_start,
                        break_end: row.break_end,
                        is_break_auto_deduct: Some(row.is_break_auto_deduct),
                        allow_early_ot: Some(row.allow_early_ot),
                        early_punch_in_grace_period_minutes: Some(row.early_punch_in_grace_period_minutes),
                    })
                    .collect();

                Ok(shifts)
            })
            .await?;

        // Convert Vec to HashMap for O(1) lookup
        Ok(shifts.into_iter().map(|s| (s.id, s)).collect())
    }

    /// Invalidate cached shifts for a workspace.
    /// Call this when shifts are created, updated, or deleted.
    pub async fn invalidate(&self, workspace_id: Uuid) -> Result<(), String> {
        self.cache.delete(&Self::key(workspace_id)).await
    }
}
