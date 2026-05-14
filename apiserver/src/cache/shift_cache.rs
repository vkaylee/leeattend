//! Shift Cache
//!
//! Caches shift data for O(1) lookup during attendance calculations.

use chrono::NaiveTime;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::collections::HashMap;
use uuid::Uuid;

use crate::cache::{RedisCache, DEFAULT_CACHE_TTL};
use crate::repositories::shift_repository::ShiftRepository;

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
    diesel: crate::diesel_pool::DieselPools,
}

impl ShiftCache {
    /// Create a new ShiftCache.
    pub fn new(cache: RedisCache, db: PgPool, diesel: crate::diesel_pool::DieselPools) -> Self {
        Self { cache, db, diesel }
    }

    /// Generate cache key for shifts of a workspace.
    fn key(workspace_id: Uuid) -> String {
        format!("cache:shifts:{workspace_id}")
    }

    /// Get all shifts for workspace as a HashMap for O(1) lookup by shift ID.
    ///
    /// Returns cached data if available, otherwise fetches from DB.
    pub async fn get_map(&self, workspace_id: Uuid) -> Result<HashMap<Uuid, CachedShift>, String> {
        let key = Self::key(workspace_id);
        let _db = self.db.clone();
        let diesel = self.diesel.clone();

        let shifts: Vec<CachedShift> = self
            .cache
            .get_or_fetch(&key, DEFAULT_CACHE_TTL, || async move {
                let repo = crate::repositories::DieselShiftRepository::new(diesel);
                let rows = repo
                    .get_workspace_shifts(workspace_id)
                    .await
                    .map_err(|e| format!("Failed to fetch shifts: {e}"))?;

                let shifts: Vec<CachedShift> = rows
                    .into_iter()
                    .filter(|s| s.is_active)
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
                        early_punch_in_grace_period_minutes: Some(
                            row.early_punch_in_grace_period_minutes,
                        ),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_key_generation() {
        let workspace_id = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap();
        let key = ShiftCache::key(workspace_id);
        assert_eq!(key, "cache:shifts:550e8400-e29b-41d4-a716-446655440000");
    }

    #[test]
    fn test_cached_shift_serialization() {
        let shift = CachedShift {
            id: Uuid::new_v4(),
            name: "Day Shift".to_string(),
            start_time: NaiveTime::from_hms_opt(8, 0, 0).unwrap(),
            end_time: NaiveTime::from_hms_opt(17, 0, 0).unwrap(),
            break_duration_minutes: Some(60),
            break_start: Some(NaiveTime::from_hms_opt(12, 0, 0).unwrap()),
            break_end: Some(NaiveTime::from_hms_opt(13, 0, 0).unwrap()),
            is_break_auto_deduct: Some(true),
            allow_early_ot: Some(false),
            early_punch_in_grace_period_minutes: Some(15),
        };

        let json = serde_json::to_string(&shift).unwrap();
        let decoded: CachedShift = serde_json::from_str(&json).unwrap();

        assert_eq!(shift.id, decoded.id);
        assert_eq!(shift.name, decoded.name);
        assert_eq!(shift.start_time, decoded.start_time);
        assert_eq!(shift.end_time, decoded.end_time);
        assert_eq!(shift.break_duration_minutes, decoded.break_duration_minutes);
    }

    #[test]
    fn test_vec_to_hashmap_conversion() {
        let shift1 = CachedShift {
            id: Uuid::new_v4(),
            name: "Morning".to_string(),
            start_time: NaiveTime::from_hms_opt(6, 0, 0).unwrap(),
            end_time: NaiveTime::from_hms_opt(14, 0, 0).unwrap(),
            break_duration_minutes: Some(30),
            break_start: None,
            break_end: None,
            is_break_auto_deduct: Some(false),
            allow_early_ot: Some(true),
            early_punch_in_grace_period_minutes: Some(10),
        };
        let shift2 = CachedShift {
            id: Uuid::new_v4(),
            name: "Evening".to_string(),
            start_time: NaiveTime::from_hms_opt(14, 0, 0).unwrap(),
            end_time: NaiveTime::from_hms_opt(22, 0, 0).unwrap(),
            break_duration_minutes: Some(30),
            break_start: None,
            break_end: None,
            is_break_auto_deduct: Some(false),
            allow_early_ot: Some(false),
            early_punch_in_grace_period_minutes: Some(5),
        };

        let shifts = vec![shift1.clone(), shift2.clone()];
        let map: HashMap<Uuid, CachedShift> = shifts.into_iter().map(|s| (s.id, s)).collect();

        assert_eq!(map.len(), 2);
        assert!(map.contains_key(&shift1.id));
        assert!(map.contains_key(&shift2.id));
        assert_eq!(map.get(&shift1.id).unwrap().name, "Morning");
        assert_eq!(map.get(&shift2.id).unwrap().name, "Evening");
    }

    #[test]
    fn test_overnight_shift() {
        let shift = CachedShift {
            id: Uuid::new_v4(),
            name: "Night Shift".to_string(),
            start_time: NaiveTime::from_hms_opt(22, 0, 0).unwrap(),
            end_time: NaiveTime::from_hms_opt(6, 0, 0).unwrap(), // Next day
            break_duration_minutes: Some(45),
            break_start: Some(NaiveTime::from_hms_opt(2, 0, 0).unwrap()),
            break_end: Some(NaiveTime::from_hms_opt(2, 45, 0).unwrap()),
            is_break_auto_deduct: Some(true),
            allow_early_ot: Some(true),
            early_punch_in_grace_period_minutes: Some(30),
        };

        // Overnight shift: end_time < start_time
        assert!(shift.end_time < shift.start_time);
        assert_eq!(shift.break_duration_minutes, Some(45));
    }
}
