//! Roster Cache Module
//!
//! In-memory LRU cache (moka) backed by Redis version stamps for
//! workspace roster data.  Singleflight loading prevents thundering-herd
//! on cache miss.  Falls back to direct DB load when Redis is unavailable.

use parking_lot::Mutex;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use crate::repositories::diesel_department_repository::EmployeeDepartmentHistoryEntity;
use chrono::{NaiveDate, NaiveTime};
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use moka::future::Cache;
use redis::AsyncCommands;
use tokio::sync::watch;
use uuid::Uuid;

use crate::queue::RedisPool;

// ---------------------------------------------------------------------------
// Cached data types (Debug + Clone, no Serde — in-memory only)
// ---------------------------------------------------------------------------

/// A single shift's static configuration — lightweight identity envelope.
/// Time, break, and policy settings live exclusively on `RosterInterval`.
#[derive(Debug, Clone)]
pub struct RosterShift {
    name: String,
    report_symbol: Option<String>,
}

impl RosterShift {
    pub fn new(name: String, report_symbol: Option<String>) -> Self {
        Self {
            name,
            report_symbol,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn report_symbol(&self) -> Option<&str> {
        self.report_symbol.as_deref()
    }
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
    pub schedule_day: i32,
    pub shift_id: Option<Uuid>,
    /// "weekly" or "rotation"
    pub pattern_type: String,
    /// Only set for rotation patterns
    pub cycle_length_days: Option<i32>,
    /// Per-assignment phase offset for rotation patterns
    pub anchor_date: Option<NaiveDate>,
}

/// One row of an employee's department-history timeline.
#[derive(Debug, Clone)]
pub struct RosterDeptHistory {
    pub department_id: Uuid,
    pub effective_date: NaiveDate,
}

/// One row from `shift_default_intervals` — template interval info for a shift.
/// This is the sole source of truth for all time, break, and policy configuration.
#[derive(Debug, Clone)]
#[allow(clippy::struct_excessive_bools)]
pub struct RosterInterval {
    pub interval_id: Uuid,
    pub sort_order: i32,
    pub interval_name: Option<String>,
    pub start_time: NaiveTime,
    pub end_time: NaiveTime,
    pub break_duration_minutes: i32,
    pub break_start: Option<NaiveTime>,
    pub break_end: Option<NaiveTime>,
    pub is_break_auto_deduct: bool,
    pub break_is_overtime: bool,
    pub early_punch_in_grace_period_minutes: i32,
    pub allow_early_ot: bool,
    pub max_early_ot_minutes: Option<i32>,
    pub ot_aggregation_mode: crate::services::attendance_calculator::OtAggregationMode,
    pub is_night_shift: bool,
}

/// Pre-fetched workspace roster configuration.
///
/// Data is fetched WITHOUT date bounds; the existing resolution loop
/// in `get_workspace_schedules` already filters by date.
#[derive(Debug, Clone, Default)]
pub struct RosterData {
    /// shift_id -> shift details
    pub shifts: HashMap<Uuid, RosterShift>,
    /// employee_id -> fixed assignments (ORDER BY created_at DESC preserved)
    pub fixed_assignments: HashMap<Uuid, Vec<RosterFixedAssignment>>,
    /// employee_id -> pattern entries (ORDER BY created_at DESC preserved)
    pub pattern_assignments: HashMap<Uuid, Vec<RosterPatternEntry>>,

    /// Active assignment rules
    pub active_rules: Vec<crate::models::AssignmentRule>,
    /// (pattern_id, schedule_day) -> list of shift_ids (supports split shifts)
    pub rule_patterns: HashMap<(Uuid, i32), Vec<Option<Uuid>>>,
    /// employee_id -> dept history entries (ORDER BY effective_date DESC)
    pub dept_history: HashMap<Uuid, Vec<RosterDeptHistory>>,
    /// shift_id -> interval templates (ORDER BY sort_order ASC)
    pub shift_intervals: HashMap<Uuid, Vec<RosterInterval>>,
}

// ---------------------------------------------------------------------------
// build_roster_data — standalone async loader
// ---------------------------------------------------------------------------

/// Fetch all roster-related data for a workspace in parallel-friendly queries
/// and assemble into an `Arc<RosterData>`.
///
/// Standalone (not a method on `RosterCache`) so it can be called directly
/// as a fallback when Redis is unavailable.
#[allow(clippy::too_many_lines)]
pub async fn build_roster_data(
    diesel_pools: &crate::diesel_pool::DieselPools,
    workspace_id: Uuid,
) -> Result<Arc<RosterData>, String> {
    let mut conn = diesel_pools
        .tenant_transaction_conn(&crate::diesel_pool::TenantAuthToken::system_override(
            workspace_id,
        ))
        .await
        .map_err(|e| format!("db connection: {e}"))?;

    use crate::schema::{
        assignment_rules, employee_shifts, employee_working_patterns, employees, employments,
        shift_default_intervals, shift_templates, shift_versions, working_pattern_details,
        working_patterns,
    };

    // 1. Shift Templates + Current Versions
    let shift_rows: Vec<(Uuid, String, Option<String>)> = shift_templates::table
        .inner_join(
            shift_versions::table
                .on(shift_templates::current_version_id.eq(shift_versions::id.nullable())),
        )
        .filter(shift_templates::workspace_id.eq(workspace_id))
        .select((
            shift_templates::id, // template_id (stable key)
            shift_versions::name,
            shift_versions::report_symbol,
        ))
        .load::<(Uuid, String, Option<String>)>(&mut conn)
        .await
        .map_err(|e| format!("shifts: {e}"))?;

    // 2. Fixed Assignments
    let fixed_rows = employee_shifts::table
        .inner_join(employments::table)
        .filter(employments::workspace_id.eq(workspace_id))
        .select(crate::models::diesel::roster::EmployeeShiftEntity::as_select())
        .order(employee_shifts::created_at.desc())
        .load::<crate::models::diesel::roster::EmployeeShiftEntity>(&mut conn)
        .await
        .map_err(|e| format!("fixed_assignments: {e}"))?;

    // 3. Pattern Assignments — JOIN working_patterns for rotation metadata
    let pattern_rows = employee_working_patterns::table
        .inner_join(
            working_pattern_details::table
                .on(employee_working_patterns::pattern_id.eq(working_pattern_details::pattern_id)),
        )
        .inner_join(
            working_patterns::table
                .on(employee_working_patterns::pattern_id.eq(working_patterns::id)),
        )
        .filter(employee_working_patterns::workspace_id.eq(workspace_id))
        .order(employee_working_patterns::created_at.desc())
        .select((
            employee_working_patterns::employment_id,
            employee_working_patterns::pattern_id,
            employee_working_patterns::start_date,
            employee_working_patterns::end_date,
            working_pattern_details::schedule_day,
            working_pattern_details::template_id,
            working_patterns::pattern_type,
            working_patterns::cycle_length_days,
            employee_working_patterns::anchor_date,
        ))
        .load::<(
            Uuid,
            Uuid,
            NaiveDate,
            Option<NaiveDate>,
            i32,
            Option<Uuid>,
            String,
            Option<i32>,
            Option<NaiveDate>,
        )>(&mut conn)
        .await
        .map_err(|e| format!("pattern_assignments: {e}"))?;

    // 6. Active Rules
    let active_rules_entities = assignment_rules::table
        .filter(assignment_rules::workspace_id.eq(workspace_id))
        .filter(assignment_rules::is_active.eq(true))
        .order((
            assignment_rules::priority.desc(),
            assignment_rules::created_at.asc(),
        ))
        .load::<crate::models::diesel::assignment_rule::AssignmentRuleEntity>(&mut conn)
        .await
        .map_err(|e| format!("active_rules: {e}"))?;

    let active_rules: Vec<crate::models::AssignmentRule> =
        active_rules_entities.into_iter().map(Into::into).collect();

    // 7. Rule Patterns
    let rule_pattern_rows = working_pattern_details::table
        .filter(
            working_pattern_details::pattern_id.eq_any(
                assignment_rules::table
                    .filter(assignment_rules::workspace_id.eq(workspace_id))
                    .filter(assignment_rules::is_active.eq(true))
                    .select(assignment_rules::pattern_id),
            ),
        )
        .select((
            working_pattern_details::pattern_id,
            working_pattern_details::schedule_day,
            working_pattern_details::template_id,
        ))
        .load::<(Uuid, i32, Option<Uuid>)>(&mut conn)
        .await
        .map_err(|e| format!("rule_patterns: {e}"))?;

    // 8. Dept History
    let dept_history_rows = employments::table
        .filter(
            employments::employee_id.eq_any(
                employees::table
                    .filter(employees::workspace_id.eq(workspace_id))
                    .select(employees::id),
            ),
        )
        .filter(employments::deleted_at.is_null())
        .filter(employments::department_id.is_not_null())
        .order(employments::start_date.desc())
        .select((
            employments::id,
            employments::employee_id,
            employments::department_id.assume_not_null(),
            employments::start_date,
            employments::notes,
            employments::created_at,
        ))
        .load::<EmployeeDepartmentHistoryEntity>(&mut conn)
        .await
        .map_err(|e| format!("dept_history: {e}"))?;

    // 9. Shift Default Intervals — template intervals for current versions
    //     Join through shift_versions to get template_id for the HashMap key.
    let interval_rows: Vec<(
        crate::models::diesel::roster::ShiftDefaultIntervalEntity,
        Uuid,
    )> = shift_default_intervals::table
        .inner_join(
            shift_versions::table.on(shift_default_intervals::version_id.eq(shift_versions::id)),
        )
        .inner_join(shift_templates::table.on(shift_versions::template_id.eq(shift_templates::id)))
        .filter(shift_templates::workspace_id.eq(workspace_id))
        .filter(shift_versions::effective_to.is_null())
        .order(shift_default_intervals::sort_order.asc())
        .select((
            crate::models::diesel::roster::ShiftDefaultIntervalEntity::as_select(),
            shift_versions::template_id,
        ))
        .load::<(
            crate::models::diesel::roster::ShiftDefaultIntervalEntity,
            Uuid,
        )>(&mut conn)
        .await
        .map_err(|e| format!("shift_intervals: {e}"))?;

    // Transform rows into lookup structures
    let shifts_map: HashMap<Uuid, RosterShift> = shift_rows
        .into_iter()
        .map(|(template_id, name, report_symbol)| {
            (
                template_id,
                RosterShift {
                    name,
                    report_symbol,
                },
            )
        })
        .collect();

    let mut fixed_assignments: HashMap<Uuid, Vec<RosterFixedAssignment>> = HashMap::new();
    for r in fixed_rows {
        fixed_assignments
            .entry(r.employment_id)
            .or_default()
            .push(RosterFixedAssignment {
                shift_id: r.template_id,
                start_date: r.start_date,
                end_date: r.end_date,
            });
    }

    let mut pattern_assignments: HashMap<Uuid, Vec<RosterPatternEntry>> = HashMap::new();
    for (emp_id, pat_id, s_date, e_date, sday, s_id, p_type, cycle_len, anchor) in pattern_rows {
        pattern_assignments
            .entry(emp_id)
            .or_default()
            .push(RosterPatternEntry {
                pattern_id: pat_id,
                start_date: s_date,
                end_date: e_date,
                schedule_day: sday,
                shift_id: s_id,
                pattern_type: p_type,
                cycle_length_days: cycle_len,
                anchor_date: anchor,
            });
    }

    let mut rule_patterns: HashMap<(Uuid, i32), Vec<Option<Uuid>>> = HashMap::new();
    for (pat_id, dow, s_id) in rule_pattern_rows {
        rule_patterns.entry((pat_id, dow)).or_default().push(s_id);
    }

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

    let mut shift_intervals: HashMap<Uuid, Vec<RosterInterval>> = HashMap::new();
    for (interval_entity, template_id) in interval_rows {
        let is_night_shift =
            interval_entity.is_night_shift || interval_entity.end_time < interval_entity.start_time;
        shift_intervals
            .entry(template_id) // Key by stable template_id
            .or_default()
            .push(RosterInterval {
                interval_id: interval_entity.id,
                sort_order: interval_entity.sort_order,
                interval_name: interval_entity.interval_name,
                start_time: interval_entity.start_time,
                end_time: interval_entity.end_time,
                break_duration_minutes: interval_entity.break_duration_minutes,
                break_start: interval_entity.break_start,
                break_end: interval_entity.break_end,
                is_break_auto_deduct: interval_entity.is_break_auto_deduct,
                break_is_overtime: interval_entity.break_is_overtime,
                early_punch_in_grace_period_minutes: interval_entity
                    .early_punch_in_grace_period_minutes,
                allow_early_ot: interval_entity.allow_early_ot,
                max_early_ot_minutes: interval_entity.max_early_ot_minutes,
                ot_aggregation_mode: interval_entity
                    .ot_aggregation_mode
                    .parse()
                    .unwrap_or_default(),
                is_night_shift,
            });
    }

    Ok(Arc::new(RosterData {
        shifts: shifts_map,
        fixed_assignments,
        pattern_assignments,

        active_rules,
        rule_patterns,
        dept_history,
        shift_intervals,
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
    /// A missing key is atomically initialized to zero so an existing local
    /// entry can never be mistaken for a valid cache hit after Redis loses the
    /// version key. Redis read failures return `None`, forcing callers to bypass
    /// the local cache rather than risk serving stale data.
    async fn fetch_version(&self, workspace_id: Uuid) -> Option<i64> {
        let key = Self::version_key(workspace_id);
        let mut conn = self.redis.conn();
        match conn.get::<_, Option<i64>>(&key).await {
            Ok(Some(v)) => Some(v),
            Ok(None) => match conn.incr::<_, _, i64>(&key, 0).await {
                Ok(v) => Some(v),
                Err(e) => {
                    tracing::warn!(
                        workspace_id = %workspace_id,
                        error = %e,
                        "Redis INCR version initialization failed"
                    );
                    None
                }
            },
            Err(e) => {
                tracing::warn!(workspace_id = %workspace_id, error = %e, "Redis GET version failed");
                None
            }
        }
    }

    // --- public API --------------------------------------------------------

    /// Atomically bump the version stamp in Redis and evict the moka entry.
    ///
    /// Call this whenever roster-related tables are mutated for a workspace.
    /// Redis `INCR` prevents same-second collisions and never moves a version
    /// backwards when an existing stamp is ahead of the wall clock.
    pub async fn invalidate(&self, workspace_id: Uuid) {
        let key = Self::version_key(workspace_id);
        let mut last_error = None;
        for attempt in 0..3 {
            let mut conn = self.redis.conn();
            match conn.incr::<_, _, i64>(&key, 1).await {
                Ok(_) => {
                    last_error = None;
                    break;
                }
                Err(error) => {
                    last_error = Some(error);
                    if attempt < 2 {
                        tokio::time::sleep(Duration::from_millis(25 * (attempt + 1))).await;
                    }
                }
            }
        }
        if let Some(error) = last_error {
            tracing::warn!(
                workspace_id = %workspace_id,
                error = %error,
                "Redis INCR version failed during invalidate after retries"
            );
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
        diesel_pools: &crate::diesel_pool::DieselPools,
    ) -> Result<Arc<RosterData>, String> {
        let Some(remote_version) = self.fetch_version(workspace_id).await else {
            self.cache.invalidate(&workspace_id).await;
            return build_roster_data(diesel_pools, workspace_id).await;
        };

        // Fast path: version match in local cache
        if let Some((data, ver)) = self.cache.get(&workspace_id).await {
            if ver == remote_version {
                return Ok(data);
            }
        }

        // Slow path: singleflight load
        self.singleflight_load(workspace_id, diesel_pools, remote_version)
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
        diesel_pools: &crate::diesel_pool::DieselPools,
        version: i64,
    ) -> Result<Arc<RosterData>, String> {
        // Determine leader vs follower while holding the mutex only briefly.
        let (is_leader, mut rx) = {
            let mut map = self.inflight.lock();
            let res = if let Some(sender) = map.get(&workspace_id) {
                // Follower: subscribe to the leader's result.
                (false, Some(sender.subscribe()))
            } else {
                // Leader: create a new watch channel seeded with None.
                let (tx, rx) = watch::channel(None);
                map.insert(workspace_id, Arc::new(tx));
                (true, Some(rx))
            };
            drop(map);
            res
        };

        if is_leader {
            // --- leader path -----------------------------------------------
            let result = build_roster_data(diesel_pools, workspace_id).await;

            // Remove ourselves from the inflight map and retrieve the sender.
            let sender = self.inflight.lock().remove(&workspace_id);

            // Notify all followers (ignore error if no receivers remain).
            if let Some(sender) = sender {
                let _ = sender.send(Some(result.as_ref().map(Arc::clone).map_err(Clone::clone)));
            }

            // On success, insert into moka.
            if let Ok(ref data) = result {
                self.cache
                    .insert(workspace_id, (Arc::clone(data), version))
                    .await;
            }

            result
        } else {
            // --- follower path ---------------------------------------------
            let rx = rx
                .as_mut()
                .ok_or_else(|| "follower must have receiver".to_string())?;

            // wait_for borrows rx mutably; we only need to know whether it
            // succeeded so we can drop the borrow before calling rx.borrow().
            let leader_finished = tokio::time::timeout(
                Duration::from_secs(30),
                rx.wait_for(std::option::Option::is_some),
            )
            .await
            .is_ok();
            // Mutable borrow from wait_for is now fully dropped.

            if leader_finished {
                // Leader finished — extract the result via shared borrow.
                let value = rx.borrow();
                match value.as_ref() {
                    Some(Ok(data)) => Ok(Arc::clone(data)),
                    Some(Err(e)) => Err(e.clone()),
                    None => unreachable!("wait_for ensures Some"),
                }
            } else {
                // Timeout or channel closed — fall back to direct load.
                build_roster_data(diesel_pools, workspace_id).await
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
        let shift = RosterShift::new("Morning".to_string(), Some("M".to_string()));

        assert_eq!(shift.name(), "Morning");
        assert_eq!(shift.report_symbol(), Some("M"));

        // Clone round-trip
        let clone = shift.clone();
        assert_eq!(clone.name(), shift.name());
        assert_eq!(clone.report_symbol(), shift.report_symbol());
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
            schedule_day: 1, // Monday
            shift_id: Some(Uuid::new_v4()),
            pattern_type: "weekly".to_string(),
            cycle_length_days: None,
            anchor_date: None,
        };
        assert_eq!(entry.schedule_day, 1);
        assert!(entry.shift_id.is_some());
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
        let mut rule_patterns: HashMap<(Uuid, i32), Vec<Option<Uuid>>> = HashMap::new();

        let shift_id = Some(Uuid::new_v4());
        rule_patterns
            .entry((pattern_id, 1))
            .or_default()
            .push(shift_id);
        rule_patterns.entry((pattern_id, 2)).or_default().push(None);

        assert_eq!(rule_patterns[&(pattern_id, 1)], vec![shift_id]);
        assert_eq!(rule_patterns[&(pattern_id, 2)], vec![None]);
        // Different pattern_id should miss
        assert!(!rule_patterns.contains_key(&(Uuid::new_v4(), 1)));
    }

    #[test]
    fn test_roster_data_empty_construction() {
        let data = RosterData {
            shifts: HashMap::new(),
            fixed_assignments: HashMap::new(),
            pattern_assignments: HashMap::new(),

            active_rules: Vec::new(),
            rule_patterns: HashMap::new(),
            dept_history: HashMap::new(),
            shift_intervals: HashMap::new(),
        };

        assert!(data.shifts.is_empty());
        assert!(data.active_rules.is_empty());

        // Clone round-trip
        let cloned = data;
        assert!(cloned.shifts.is_empty());
    }

    // --- helper ----------------------------------------------------------

    fn make_empty_roster_data() -> RosterData {
        RosterData {
            shifts: HashMap::new(),
            fixed_assignments: HashMap::new(),
            pattern_assignments: HashMap::new(),

            active_rules: Vec::new(),
            rule_patterns: HashMap::new(),
            dept_history: HashMap::new(),
            shift_intervals: HashMap::new(),
        }
    }

    // --- moka cache behaviour ---------------------------------------------

    /// Fast path: entry is returned when local version == remote version.
    #[tokio::test]
    async fn test_moka_hit_when_version_matches() {
        let cache: Cache<Uuid, (Arc<RosterData>, i64)> = Cache::builder().max_capacity(100).build();
        let ws = Uuid::new_v4();
        let data = Arc::new(make_empty_roster_data());
        cache.insert(ws, (Arc::clone(&data), 1000)).await;

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
            rx.wait_for(std::option::Option::is_some)
                .await
                .expect("watch channel failed");
            let value = rx.borrow();
            match value.as_ref() {
                Some(Ok(d)) => Ok(Arc::clone(d)),
                Some(Err(e)) => Err(e.clone()),
                None => unreachable!(),
            }
        });

        // Brief delay so follower is blocked, then leader publishes
        tokio::time::sleep(Duration::from_millis(10)).await;
        tx.send(Some(Ok(Arc::clone(&data)))).expect("send failed");

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
            rx.wait_for(std::option::Option::is_some)
                .await
                .expect("watch channel failed");
            let value = rx.borrow();
            match value.as_ref() {
                Some(Ok(d)) => Ok(Arc::clone(d)),
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

        // 2. Simulate Follower entering
        {
            let map = inflight.lock().expect("mutex poisoned");
            let _sender = Arc::clone(map.get(&ws_id).expect("should exist"));
            // Follower subscribes...
        }

        // 3. Simulate Leader exiting (CRITICAL STEP for memory leak prevention)
        {
            let mut map = inflight.lock().expect("mutex poisoned");
            map.remove(&ws_id);
        }

        // 4. Verify Map is Empty
        assert!(
            inflight.lock().expect("mutex poisoned").is_empty(),
            "Inflight map should be empty after leader exit"
        );
    }

    /// Timeout fires cleanly when leader never publishes (simulates the
    /// 30 s fallback path in singleflight_load).
    #[tokio::test]
    async fn test_singleflight_follower_timeout_does_not_panic() {
        let (_tx, mut rx) = watch::channel::<Option<Result<Arc<RosterData>, String>>>(None);

        let timed_out = tokio::time::timeout(
            Duration::from_millis(50),
            rx.wait_for(std::option::Option::is_some),
        )
        .await
        .is_err();

        assert!(
            timed_out,
            "Follower should time out when leader never publishes"
        );
    }
}
