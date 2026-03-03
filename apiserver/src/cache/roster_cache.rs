//! Roster Cache Module
//!
//! In-memory LRU cache (moka) backed by Redis version stamps for
//! workspace roster data.  Singleflight loading prevents thundering-herd
//! on cache miss.  Falls back to direct DB load when Redis is unavailable.

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use std::time::Duration;

use chrono::{NaiveDate, NaiveTime, Utc};
use moka::future::Cache;
use redis::AsyncCommands;
use sqlx::PgPool;
use tokio::sync::watch;
use uuid::Uuid;

use crate::queue::RedisPool;

// ---------------------------------------------------------------------------
// Cached data types (Debug + Clone, no Serde — in-memory only)
// ---------------------------------------------------------------------------

/// A single shift's static configuration.
#[derive(Debug, Clone)]
pub struct RosterShift {
    pub name: String,
    pub report_symbol: Option<String>,
    pub start_time: NaiveTime,
    pub end_time: NaiveTime,
    pub break_duration_minutes: i32,
    pub break_start: Option<NaiveTime>,
    pub break_end: Option<NaiveTime>,
    pub is_break_auto_deduct: bool,
    pub early_punch_in_grace_period_minutes: i32,
    pub allow_early_ot: bool,
}

/// A fixed (direct) shift assignment row for one employee.
#[derive(Debug, Clone)]
pub struct RosterFixedAssignment {
    pub shift_id: Uuid,
    pub start_date: NaiveDate,
    pub end_date: Option<NaiveDate>,
}

/// One row from a pattern-based assignment expanded by working_pattern_details.
#[derive(Debug, Clone)]
pub struct RosterPatternEntry {
    pub pattern_id: Uuid,
    pub start_date: NaiveDate,
    pub end_date: Option<NaiveDate>,
    pub day_of_week: i32,
    pub shift_id: Option<Uuid>,
}

/// Department-level default pattern entry for one employee.
#[derive(Debug, Clone)]
pub struct RosterDeptPattern {
    pub default_working_pattern_id: Option<Uuid>,
    pub effective_date: NaiveDate,
    pub day_of_week: i32,
    pub shift_id: Option<Uuid>,
}

/// Department-level fixed shift entry for one employee.
#[derive(Debug, Clone)]
pub struct RosterDeptFixedShift {
    pub shift_id: Uuid,
    pub effective_date: NaiveDate,
}

/// One row of an employee's department-history timeline.
#[derive(Debug, Clone)]
pub struct RosterDeptHistory {
    pub department_id: Uuid,
    pub effective_date: NaiveDate,
}

/// Pre-fetched workspace roster configuration.
///
/// Data is fetched WITHOUT date bounds; the existing resolution loop
/// in `get_workspace_schedules` already filters by date.
#[derive(Debug, Clone)]
pub struct RosterData {
    /// shift_id -> shift details
    pub shifts: HashMap<Uuid, RosterShift>,
    /// employee_id -> fixed assignments (ORDER BY created_at DESC preserved)
    pub fixed_assignments: HashMap<Uuid, Vec<RosterFixedAssignment>>,
    /// employee_id -> pattern entries (ORDER BY created_at DESC preserved)
    pub pattern_assignments: HashMap<Uuid, Vec<RosterPatternEntry>>,
    /// employee_id -> dept pattern entries (ORDER BY effective_date DESC)
    pub dept_patterns: HashMap<Uuid, Vec<RosterDeptPattern>>,
    /// employee_id -> dept fixed shift entries (ORDER BY effective_date DESC)
    pub dept_fixed_shifts: HashMap<Uuid, Vec<RosterDeptFixedShift>>,
    /// Active assignment rules
    pub active_rules: Vec<crate::models::AssignmentRule>,
    /// (pattern_id, day_of_week) -> Option<shift_id>
    pub rule_patterns: HashMap<(Uuid, i32), Option<Uuid>>,
    /// employee_id -> dept history entries (ORDER BY effective_date DESC)
    pub dept_history: HashMap<Uuid, Vec<RosterDeptHistory>>,
}

// ---------------------------------------------------------------------------
// build_roster_data — standalone async loader
// ---------------------------------------------------------------------------

/// Fetch all roster-related data for a workspace in parallel-friendly queries
/// and assemble into an `Arc<RosterData>`.
///
/// Standalone (not a method on `RosterCache`) so it can be called directly
/// as a fallback when Redis is unavailable.
pub async fn build_roster_data(
    pool: &PgPool,
    workspace_id: Uuid,
) -> Result<Arc<RosterData>, String> {
    // Create futures at top level for sqlx::query! type inference to work
    let q1 = sqlx::query!(
        r#"
        SELECT id, name, report_symbol, start_time, end_time, break_duration_minutes,
               break_start, break_end, is_break_auto_deduct,
               early_punch_in_grace_period_minutes, allow_early_ot
        FROM shifts WHERE workspace_id = $1
        "#,
        workspace_id,
    )
    .fetch_all(pool);

    let q2 = sqlx::query!(
        r#"
        SELECT employee_id, shift_id, start_date, end_date
        FROM employee_shifts
        WHERE employee_id IN (SELECT id FROM employees WHERE workspace_id = $1)
        ORDER BY created_at DESC
        "#,
        workspace_id,
    )
    .fetch_all(pool);

    let q3 = sqlx::query!(
        r#"
        SELECT ewp.employee_id, ewp.pattern_id, ewp.start_date, ewp.end_date,
               wpd.day_of_week, wpd.shift_id as pattern_shift_id
        FROM employee_working_patterns ewp
        JOIN working_pattern_details wpd ON ewp.pattern_id = wpd.pattern_id
        WHERE ewp.workspace_id = $1
        ORDER BY ewp.created_at DESC
        "#,
        workspace_id,
    )
    .fetch_all(pool);

    let q4 = sqlx::query!(
        r#"
        SELECT edh.employee_id, d.default_working_pattern_id, edh.effective_date,
               wpd.day_of_week, wpd.shift_id as pattern_shift_id
        FROM employee_department_history edh
        JOIN departments d ON edh.department_id = d.id
        JOIN working_pattern_details wpd ON d.default_working_pattern_id = wpd.pattern_id
        WHERE d.workspace_id = $1
        ORDER BY edh.effective_date DESC
        "#,
        workspace_id,
    )
    .fetch_all(pool);

    let q5 = sqlx::query!(
        r#"
        SELECT edh.employee_id, dds.shift_id, edh.effective_date
        FROM employee_department_history edh
        JOIN department_default_shifts dds ON edh.department_id = dds.department_id
        JOIN departments d ON dds.department_id = d.id
        WHERE d.workspace_id = $1
        ORDER BY edh.effective_date DESC
        "#,
        workspace_id,
    )
    .fetch_all(pool);

    let q6 = crate::repositories::assignment_rule_repository::AssignmentRuleRepository::get_active_rules(
        pool,
        workspace_id,
    );

    let q7 = sqlx::query!(
        r#"
        SELECT pattern_id, day_of_week, shift_id
        FROM working_pattern_details
        WHERE pattern_id IN (
            SELECT pattern_id FROM assignment_rules
            WHERE workspace_id = $1 AND is_active = true
        )
        "#,
        workspace_id,
    )
    .fetch_all(pool);

    let q8 = sqlx::query!(
        r#"
        SELECT edh.employee_id, edh.department_id, edh.effective_date
        FROM employee_department_history edh
        WHERE edh.employee_id IN (SELECT id FROM employees WHERE workspace_id = $1)
        ORDER BY edh.effective_date DESC
        "#,
        workspace_id,
    )
    .fetch_all(pool);

    // Run all 8 independent queries concurrently
    let (
        shift_rows,
        fixed_rows,
        pattern_rows,
        dept_pattern_rows,
        dept_fixed_rows,
        active_rules,
        rule_pattern_rows,
        dept_history_rows,
    ): (_, _, _, _, _, _, _, _) = tokio::try_join!(
        async { q1.await.map_err(|e| format!("shifts: {e}")) },
        async { q2.await.map_err(|e| format!("fixed_assignments: {e}")) },
        async { q3.await.map_err(|e| format!("pattern_assignments: {e}")) },
        async { q4.await.map_err(|e| format!("dept_patterns: {e}")) },
        async { q5.await.map_err(|e| format!("dept_fixed_shifts: {e}")) },
        async { q6.await.map_err(|e| format!("active_rules: {e}")) },
        async { q7.await.map_err(|e| format!("rule_patterns: {e}")) },
        async { q8.await.map_err(|e| format!("dept_history: {e}")) },
    )?;

    // Transform rows into lookup structures
    let shifts: HashMap<Uuid, RosterShift> = shift_rows
        .into_iter()
        .map(|r| {
            (
                r.id,
                RosterShift {
                    name: r.name,
                    report_symbol: r.report_symbol,
                    start_time: r.start_time,
                    end_time: r.end_time,
                    break_duration_minutes: r.break_duration_minutes,
                    break_start: r.break_start,
                    break_end: r.break_end,
                    is_break_auto_deduct: r.is_break_auto_deduct,
                    early_punch_in_grace_period_minutes: r.early_punch_in_grace_period_minutes,
                    allow_early_ot: r.allow_early_ot,
                },
            )
        })
        .collect();

    let mut fixed_assignments: HashMap<Uuid, Vec<RosterFixedAssignment>> = HashMap::new();
    for r in fixed_rows {
        fixed_assignments
            .entry(r.employee_id)
            .or_default()
            .push(RosterFixedAssignment {
                shift_id: r.shift_id,
                start_date: r.start_date,
                end_date: r.end_date,
            });
    }

    let mut pattern_assignments: HashMap<Uuid, Vec<RosterPatternEntry>> = HashMap::new();
    for r in pattern_rows {
        pattern_assignments
            .entry(r.employee_id)
            .or_default()
            .push(RosterPatternEntry {
                pattern_id: r.pattern_id,
                start_date: r.start_date,
                end_date: r.end_date,
                day_of_week: r.day_of_week,
                shift_id: r.pattern_shift_id,
            });
    }

    let mut dept_patterns: HashMap<Uuid, Vec<RosterDeptPattern>> = HashMap::new();
    for r in dept_pattern_rows {
        dept_patterns
            .entry(r.employee_id)
            .or_default()
            .push(RosterDeptPattern {
                default_working_pattern_id: r.default_working_pattern_id,
                effective_date: r.effective_date,
                day_of_week: r.day_of_week,
                shift_id: r.pattern_shift_id,
            });
    }

    let mut dept_fixed_shifts: HashMap<Uuid, Vec<RosterDeptFixedShift>> = HashMap::new();
    for r in dept_fixed_rows {
        dept_fixed_shifts
            .entry(r.employee_id)
            .or_default()
            .push(RosterDeptFixedShift {
                shift_id: r.shift_id,
                effective_date: r.effective_date,
            });
    }

    let rule_patterns: HashMap<(Uuid, i32), Option<Uuid>> = rule_pattern_rows
        .into_iter()
        .map(|r| ((r.pattern_id, r.day_of_week), r.shift_id))
        .collect();

    let mut dept_history: HashMap<Uuid, Vec<RosterDeptHistory>> = HashMap::new();
    for r in dept_history_rows {
        dept_history
            .entry(r.employee_id)
            .or_default()
            .push(RosterDeptHistory {
                department_id: r.department_id,
                effective_date: r.effective_date,
            });
    }

    Ok(Arc::new(RosterData {
        shifts,
        fixed_assignments,
        pattern_assignments,
        dept_patterns,
        dept_fixed_shifts,
        active_rules,
        rule_patterns,
        dept_history,
    }))
}

// ---------------------------------------------------------------------------
// RosterCache — the cache layer with singleflight
// ---------------------------------------------------------------------------

/// Singleflight map: workspace_id → watch sender (leader holds the sender).
type SingleflightMap =
    Mutex<HashMap<Uuid, Arc<watch::Sender<Option<Result<Arc<RosterData>, String>>>>>>;

/// In-memory LRU cache for workspace roster data, version-stamped via Redis.
///
/// * moka LRU holds up to 100 workspaces.
/// * A Redis key (`roster:version:{workspace_id}`) tracks the current version
///   timestamp.  When the local cached version matches the remote one the
///   moka entry is returned directly.
/// * Concurrent misses for the same workspace are coalesced via a
///   `tokio::sync::watch` channel (singleflight pattern).
pub struct RosterCache {
    /// moka LRU: workspace_id -> (data, version_timestamp)
    cache: Cache<Uuid, (Arc<RosterData>, i64)>,
    /// Redis pool for version key reads / writes
    redis: RedisPool,
    /// Singleflight map: workspace_id -> watch sender (leader holds this)
    inflight: SingleflightMap,
}

impl RosterCache {
    /// Construct a new `RosterCache`.
    pub fn new(redis: RedisPool) -> Self {
        Self {
            cache: Cache::builder().max_capacity(100).build(),
            redis,
            inflight: Mutex::new(HashMap::new()),
        }
    }

    // --- helpers -----------------------------------------------------------

    /// Redis key for the version stamp of a workspace's roster data.
    fn version_key(workspace_id: Uuid) -> String {
        format!("roster:version:{workspace_id}")
    }

    /// Read the current version stamp from Redis.
    ///
    /// Returns `Utc::now().timestamp()` on miss or error so that the caller
    /// always triggers a fresh load in those cases.
    async fn fetch_version(&self, workspace_id: Uuid) -> i64 {
        let key = Self::version_key(workspace_id);
        let mut conn = self.redis.conn();
        match conn.get::<_, Option<i64>>(&key).await {
            Ok(Some(v)) => v,
            Ok(None) => Utc::now().timestamp(),
            Err(e) => {
                tracing::warn!(workspace_id = %workspace_id, error = %e, "Redis GET version failed");
                Utc::now().timestamp()
            }
        }
    }

    // --- public API --------------------------------------------------------

    /// Bump the version stamp in Redis and evict the moka entry.
    ///
    /// Call this whenever roster-related tables are mutated for a workspace.
    pub async fn invalidate(&self, workspace_id: Uuid) {
        let key = Self::version_key(workspace_id);
        let mut conn = self.redis.conn();
        if let Err(e) = conn.set::<_, _, ()>(&key, Utc::now().timestamp()).await {
            tracing::warn!(workspace_id = %workspace_id, error = %e, "Redis SET version failed during invalidate");
        }
        self.cache.invalidate(&workspace_id).await;
    }

    /// Retrieve cached roster data, loading from DB on miss.
    ///
    /// 1. Fetch remote version from Redis.
    /// 2. If the moka entry exists **and** its version matches -> return it.
    /// 3. Otherwise enter the singleflight path.
    pub async fn get_roster_data(
        &self,
        workspace_id: Uuid,
        pool: &PgPool,
    ) -> Result<Arc<RosterData>, String> {
        let remote_version = self.fetch_version(workspace_id).await;

        // Fast path: version match in local cache
        if let Some((data, ver)) = self.cache.get(&workspace_id).await {
            if ver == remote_version {
                return Ok(data);
            }
        }

        // Slow path: singleflight load
        self.singleflight_load(workspace_id, pool, remote_version)
            .await
    }

    // --- singleflight ------------------------------------------------------

    /// Coalesce concurrent loads for the same workspace.
    ///
    /// The first caller becomes the **leader** and performs the actual DB fetch.
    /// Subsequent callers for the same workspace become **followers** that wait
    /// on the leader's result via a `watch` channel.  If a follower times out
    /// (30 s) it falls back to its own direct load.
    async fn singleflight_load(
        &self,
        workspace_id: Uuid,
        pool: &PgPool,
        version: i64,
    ) -> Result<Arc<RosterData>, String> {
        // Determine leader vs follower while holding the mutex only briefly.
        let (is_leader, mut rx) = {
            let mut map = self.inflight.lock().await;
            if let Some(sender) = map.get(&workspace_id) {
                // Follower: subscribe to the leader's result.
                (false, Some(sender.subscribe()))
            } else {
                // Leader: create a new watch channel seeded with None.
                let (tx, rx) = watch::channel(None);
                map.insert(workspace_id, Arc::new(tx));
                (true, Some(rx))
            }
        };

        if is_leader {
            // --- leader path -----------------------------------------------
            let result = build_roster_data(pool, workspace_id).await;

            // Remove ourselves from the inflight map and retrieve the sender.
            let sender = self
                .inflight
                .lock()
                .await
                .remove(&workspace_id);

            // Notify all followers (ignore error if no receivers remain).
            if let Some(sender) = sender {
                let _ = sender.send(Some(result.clone()));
            }

            // On success, insert into moka.
            if let Ok(ref data) = result {
                self.cache
                    .insert(workspace_id, (data.clone(), version))
                    .await;
            }

            result
        } else {
            // --- follower path ---------------------------------------------
            let rx = rx.as_mut().expect("follower must have receiver");

            // wait_for borrows rx mutably; we only need to know whether it
            // succeeded so we can drop the borrow before calling rx.borrow().
            let leader_finished =
                tokio::time::timeout(Duration::from_secs(30), rx.wait_for(|v| v.is_some()))
                    .await
                    .is_ok();
            // Mutable borrow from wait_for is now fully dropped.

            if leader_finished {
                // Leader finished — extract the result via shared borrow.
                let value = rx.borrow();
                match value.as_ref() {
                    Some(Ok(data)) => Ok(data.clone()),
                    Some(Err(e)) => Err(e.clone()),
                    None => unreachable!("wait_for ensures Some"),
                }
            } else {
                // Timeout or channel closed — fall back to direct load.
                tracing::warn!(
                    workspace_id = %workspace_id,
                    "Singleflight follower timed out; falling back to direct DB load"
                );
                build_roster_data(pool, workspace_id).await
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Unit tests (offline, no Redis / DB)
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_key_format() {
        let ws_id =
            Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").expect("valid UUID constant");
        let key = RosterCache::version_key(ws_id);
        assert_eq!(key, "roster:version:550e8400-e29b-41d4-a716-446655440000");
    }

    #[test]
    fn test_version_key_uniqueness() {
        let a = Uuid::new_v4();
        let b = Uuid::new_v4();
        assert_ne!(RosterCache::version_key(a), RosterCache::version_key(b));
    }

    #[test]
    fn test_roster_shift_construction() {
        let shift = RosterShift {
            name: "Morning".to_string(),
            report_symbol: None,
            start_time: chrono::NaiveTime::from_hms_opt(8, 0, 0).expect("valid time"),
            end_time: chrono::NaiveTime::from_hms_opt(17, 0, 0).expect("valid time"),
            break_duration_minutes: 60,
            break_start: Some(chrono::NaiveTime::from_hms_opt(12, 0, 0).expect("valid time")),
            break_end: Some(chrono::NaiveTime::from_hms_opt(13, 0, 0).expect("valid time")),
            is_break_auto_deduct: true,
            early_punch_in_grace_period_minutes: 15,
            allow_early_ot: false,
        };

        assert_eq!(shift.name, "Morning");
        assert_eq!(shift.break_duration_minutes, 60);
        assert!(shift.is_break_auto_deduct);

        // Clone round-trip
        let clone = shift.clone();
        assert_eq!(clone.name, shift.name);
        assert_eq!(clone.start_time, shift.start_time);
    }

    #[test]
    fn test_roster_fixed_assignment_construction() {
        let assignment = RosterFixedAssignment {
            shift_id: Uuid::new_v4(),
            start_date: chrono::NaiveDate::from_ymd_opt(2026, 1, 1).expect("valid date"),
            end_date: Some(chrono::NaiveDate::from_ymd_opt(2026, 12, 31).expect("valid date")),
        };
        assert!(assignment.end_date.is_some());
    }

    #[test]
    fn test_roster_pattern_entry_construction() {
        let entry = RosterPatternEntry {
            pattern_id: Uuid::new_v4(),
            start_date: chrono::NaiveDate::from_ymd_opt(2026, 2, 1).expect("valid date"),
            end_date: None,
            day_of_week: 1, // Monday
            shift_id: Some(Uuid::new_v4()),
        };
        assert_eq!(entry.day_of_week, 1);
        assert!(entry.shift_id.is_some());
    }

    #[test]
    fn test_roster_dept_pattern_construction() {
        let entry = RosterDeptPattern {
            default_working_pattern_id: Some(Uuid::new_v4()),
            effective_date: chrono::NaiveDate::from_ymd_opt(2026, 1, 15).expect("valid date"),
            day_of_week: 3,
            shift_id: None,
        };
        assert_eq!(entry.day_of_week, 3);
        assert!(entry.shift_id.is_none());
    }

    #[test]
    fn test_roster_dept_fixed_shift_construction() {
        let shift_id = Uuid::new_v4();
        let entry = RosterDeptFixedShift {
            shift_id,
            effective_date: chrono::NaiveDate::from_ymd_opt(2026, 3, 1).expect("valid date"),
        };
        assert_eq!(entry.shift_id, shift_id);
    }

    #[test]
    fn test_roster_dept_history_construction() {
        let dept_id = Uuid::new_v4();
        let entry = RosterDeptHistory {
            department_id: dept_id,
            effective_date: chrono::NaiveDate::from_ymd_opt(2026, 1, 1).expect("valid date"),
        };
        assert_eq!(entry.department_id, dept_id);
    }

    #[test]
    fn test_hashmap_grouping_by_employee() {
        let emp_a = Uuid::new_v4();
        let emp_b = Uuid::new_v4();

        let mut fixed: HashMap<Uuid, Vec<RosterFixedAssignment>> = HashMap::new();

        // Insert two entries for emp_a, one for emp_b
        for i in 0..2 {
            fixed.entry(emp_a).or_default().push(RosterFixedAssignment {
                shift_id: Uuid::new_v4(),
                start_date: chrono::NaiveDate::from_ymd_opt(2026, 1, 1 + i).expect("valid date"),
                end_date: None,
            });
        }
        fixed.entry(emp_b).or_default().push(RosterFixedAssignment {
            shift_id: Uuid::new_v4(),
            start_date: chrono::NaiveDate::from_ymd_opt(2026, 2, 1).expect("valid date"),
            end_date: None,
        });

        assert_eq!(fixed.len(), 2);
        assert_eq!(fixed[&emp_a].len(), 2);
        assert_eq!(fixed[&emp_b].len(), 1);
    }

    #[test]
    fn test_rule_pattern_key_correctness() {
        let pattern_id = Uuid::new_v4();
        let mut rule_patterns: HashMap<(Uuid, i32), Option<Uuid>> = HashMap::new();

        let shift_id = Some(Uuid::new_v4());
        rule_patterns.insert((pattern_id, 1), shift_id);
        rule_patterns.insert((pattern_id, 2), None);

        assert_eq!(rule_patterns[&(pattern_id, 1)], shift_id);
        assert!(rule_patterns[&(pattern_id, 2)].is_none());
        // Different pattern_id should miss
        assert!(!rule_patterns.contains_key(&(Uuid::new_v4(), 1)));
    }

    #[test]
    fn test_roster_data_empty_construction() {
        let data = RosterData {
            shifts: HashMap::new(),
            fixed_assignments: HashMap::new(),
            pattern_assignments: HashMap::new(),
            dept_patterns: HashMap::new(),
            dept_fixed_shifts: HashMap::new(),
            active_rules: Vec::new(),
            rule_patterns: HashMap::new(),
            dept_history: HashMap::new(),
        };

        assert!(data.shifts.is_empty());
        assert!(data.active_rules.is_empty());

        // Clone round-trip
        let cloned = data.clone();
        assert!(cloned.shifts.is_empty());
    }

    // --- helper ----------------------------------------------------------

    fn make_empty_roster_data() -> RosterData {
        RosterData {
            shifts: HashMap::new(),
            fixed_assignments: HashMap::new(),
            pattern_assignments: HashMap::new(),
            dept_patterns: HashMap::new(),
            dept_fixed_shifts: HashMap::new(),
            active_rules: Vec::new(),
            rule_patterns: HashMap::new(),
            dept_history: HashMap::new(),
        }
    }

    // --- moka cache behaviour ---------------------------------------------

    /// Fast path: entry is returned when local version == remote version.
    #[tokio::test]
    async fn test_moka_hit_when_version_matches() {
        let cache: Cache<Uuid, (Arc<RosterData>, i64)> = Cache::builder().max_capacity(100).build();
        let ws = Uuid::new_v4();
        let data = Arc::new(make_empty_roster_data());
        cache.insert(ws, (data.clone(), 1000)).await;

        // Mirror the fast-path check from get_roster_data
        let remote_version: i64 = 1000;
        let hit = cache.get(&ws).await.and_then(
            |(d, v)| {
                if v == remote_version {
                    Some(d)
                } else {
                    None
                }
            },
        );

        assert!(
            hit.is_some(),
            "Should hit when stored version == remote version"
        );
        assert!(Arc::ptr_eq(&hit.expect("should hit"), &data));
    }

    /// Slow path: entry is skipped when local version != remote version.
    #[tokio::test]
    async fn test_moka_miss_when_version_differs() {
        let cache: Cache<Uuid, (Arc<RosterData>, i64)> = Cache::builder().max_capacity(100).build();
        let ws = Uuid::new_v4();
        cache
            .insert(ws, (Arc::new(make_empty_roster_data()), 100))
            .await;

        let remote_version: i64 = 200; // differs from stored 100
        let hit = cache.get(&ws).await.and_then(
            |(d, v)| {
                if v == remote_version {
                    Some(d)
                } else {
                    None
                }
            },
        );

        assert!(
            hit.is_none(),
            "Should miss when stored version != remote version"
        );
    }

    /// invalidate() removes the entry from the LRU.
    #[tokio::test]
    async fn test_moka_invalidate_evicts_entry() {
        let cache: Cache<Uuid, (Arc<RosterData>, i64)> = Cache::builder().max_capacity(100).build();
        let ws = Uuid::new_v4();
        cache
            .insert(ws, (Arc::new(make_empty_roster_data()), 42))
            .await;
        assert!(cache.get(&ws).await.is_some());

        cache.invalidate(&ws).await;
        cache.run_pending_tasks().await;
        assert!(
            cache.get(&ws).await.is_none(),
            "Entry must be gone after invalidate"
        );
    }

    /// LRU eviction fires when capacity is exceeded.
    #[tokio::test]
    async fn test_moka_lru_evicts_on_capacity_overflow() {
        let cache: Cache<Uuid, (Arc<RosterData>, i64)> = Cache::builder().max_capacity(2).build();
        let ids: Vec<Uuid> = (0..3).map(|_| Uuid::new_v4()).collect();
        for (i, &id) in ids.iter().enumerate() {
            cache
                .insert(id, (Arc::new(make_empty_roster_data()), i as i64))
                .await;
        }
        cache.run_pending_tasks().await;

        assert!(
            cache.entry_count() <= 2,
            "LRU cache must not exceed max_capacity"
        );
    }

    // --- singleflight (watch-channel) pattern -------------------------------------

    /// Follower receives the Ok result published by the leader.
    #[tokio::test]
    async fn test_singleflight_follower_receives_leader_ok() {
        let (tx, mut rx) = watch::channel::<Option<Result<Arc<RosterData>, String>>>(None);
        let data = Arc::new(make_empty_roster_data());

        let follower = tokio::spawn(async move {
            rx.wait_for(|v| v.is_some())
                .await
                .expect("watch channel failed");
            let value = rx.borrow();
            match value.as_ref() {
                Some(Ok(d)) => Ok(d.clone()),
                Some(Err(e)) => Err(e.clone()),
                None => unreachable!(),
            }
        });

        // Brief delay so follower is blocked, then leader publishes
        tokio::time::sleep(Duration::from_millis(10)).await;
        tx.send(Some(Ok(data.clone()))).expect("send failed");

        let result = follower
            .await
            .expect("follower panicked")
            .expect("result error");
        assert!(Arc::ptr_eq(&result, &data));
    }

    /// Follower surfaces the Err published by the leader.
    #[tokio::test]
    async fn test_singleflight_follower_receives_leader_error() {
        let (tx, mut rx) = watch::channel::<Option<Result<Arc<RosterData>, String>>>(None);

        let follower = tokio::spawn(async move {
            rx.wait_for(|v| v.is_some())
                .await
                .expect("watch channel failed");
            let value = rx.borrow();
            match value.as_ref() {
                Some(Ok(d)) => Ok(d.clone()),
                Some(Err(e)) => Err(e.clone()),
                None => unreachable!(),
            }
        });

        tokio::time::sleep(Duration::from_millis(10)).await;
        tx.send(Some(Err("DB Error".to_string())))
            .expect("send failed");

        let result = follower.await.expect("follower panicked");
        assert!(result.is_err());
        assert_eq!(result.expect_err("should be error"), "DB Error");
    }

    /// Prove that the singleflight map pattern does not leak entries.
    #[test]
    fn test_singleflight_map_lifecycle_no_leak() {
        let inflight = std::sync::Mutex::new(HashMap::new());
        let ws_id = Uuid::new_v4();

        // 1. Simulate Leader entering
        {
            let mut map = inflight.lock().expect("mutex poisoned");
            let (tx, _) = watch::channel::<Option<Result<Arc<RosterData>, String>>>(None);
            map.insert(ws_id, Arc::new(tx));
        }

        // 2. Simulate Follower finding entry
        {
            let map = inflight.lock().expect("mutex poisoned");
            assert!(map.contains_key(&ws_id));
            let _sender = map.get(&ws_id).expect("should exist").clone();
            // Follower subscribes...
        }

        // 3. Simulate Leader exiting (CRITICAL STEP for memory leak prevention)
        {
            let mut map = inflight.lock().expect("mutex poisoned");
            map.remove(&ws_id);
        }

        // 4. Verify Map is Empty
        {
            let map = inflight.lock().expect("mutex poisoned");
            assert!(
                map.is_empty(),
                "Inflight map should be empty after leader exit"
            );
        }
    }

    /// Timeout fires cleanly when leader never publishes (simulates the
    /// 30 s fallback path in singleflight_load).
    #[tokio::test]
    async fn test_singleflight_follower_timeout_does_not_panic() {
        let (_tx, mut rx) = watch::channel::<Option<Result<Arc<RosterData>, String>>>(None);

        let timed_out =
            tokio::time::timeout(Duration::from_millis(50), rx.wait_for(|v| v.is_some()))
                .await
                .is_err();

        assert!(
            timed_out,
            "Follower should time out when leader never publishes"
        );
    }
}
