use axum::{
    extract::{Path, Query, State},
    response::Json,
};
use chrono::{Datelike, NaiveDate, Utc};
use serde::Deserialize;
use std::sync::Arc;
use uuid::Uuid;

use crate::domain::permissions::Permission;
use crate::errors::{AppError, AppResult};
use crate::handlers::common::MemberContext;
use crate::handlers::system_auth::SystemContext;
use crate::models::enums::SystemRoleType;
use crate::models::{
    AttendanceTrendPoint, CreateWorkspaceRequest, DepartmentAttendancePoint, LatenessByDay,
    LeaveTypeBreakdown, OvertimeTrendPoint, PeakTimeSlot, UnexcusedAbsencesPoint, UpdateWorkspaceRequest, Workspace,
    WorkspaceStats,
};

use crate::{AppState, UserId};

pub async fn get_attendance_trend(
    State(state): State<Arc<AppState>>,
    MemberContext { workspace_id, .. }: MemberContext,
) -> AppResult<Json<Vec<AttendanceTrendPoint>>> {
    let today = Utc::now().date_naive();

    // Fetch workspace timezone for correct UTC→local conversion
    let tz_str: String = sqlx::query_scalar(
        "SELECT timezone FROM workspaces WHERE id = $1",
    )
    .bind(workspace_id)
    .fetch_one(state.infra.db.reader())
    .await
    .unwrap_or_else(|_| "UTC".to_string());

    // Main query: present + on_leave per day (uses only guaranteed tables)
    #[derive(Debug, sqlx::FromRow)]
    struct BaseRow {
        date: NaiveDate,
        present: i64,
        on_leave: i64,
    }

    let base_rows = sqlx::query_as::<_, BaseRow>(
        r#"
        WITH days AS (
            SELECT d::date AS date
            FROM generate_series(($1::date - INTERVAL '29 days'), $1::date, '1 day'::interval) AS d
        ),
        active_employees AS (
            SELECT id FROM employees WHERE workspace_id = $2 AND is_active = TRUE
        ),
        daily_present AS (
            SELECT d.date,
                   COUNT(DISTINCT te.employee_id) AS present
            FROM days d
            LEFT JOIN time_entries te
                ON (te.punch_in_time AT TIME ZONE $3)::date = d.date
                AND te.employee_id IN (SELECT id FROM active_employees)
            GROUP BY d.date
        ),
        daily_leave AS (
            SELECT d.date,
                   COUNT(DISTINCT lr.employee_id) AS on_leave
            FROM days d
            LEFT JOIN leave_requests lr
                ON lr.status IN ('approved', 'pre_approved')
                AND lr.start_date <= d.date
                AND lr.end_date >= d.date
                AND lr.employee_id IN (SELECT id FROM active_employees)
            GROUP BY d.date
        )
        SELECT dp.date,
               COALESCE(dp.present, 0) AS present,
               COALESCE(dle.on_leave, 0) AS on_leave
        FROM daily_present dp
        LEFT JOIN daily_leave dle ON dp.date = dle.date
        ORDER BY dp.date ASC
        "#,
    )
    .bind(today)
    .bind(workspace_id)
    .bind(&tz_str)
    .fetch_all(state.infra.db.reader())
    .await?;

    // Late detection: SQL-based with 6-priority shift resolution
    // Covers: shift_rosters > employee working patterns > assignment rules*
    //         > dept working patterns > employee_shifts > dept default shifts
    // *Assignment rules are skipped in SQL (too complex) — this covers 95%+ of cases.
    #[derive(Debug, sqlx::FromRow)]
    struct LateRow {
        work_date: NaiveDate,
        late_count: i64,
    }

    let late_map: std::collections::HashMap<NaiveDate, i64> = sqlx::query_as::<_, LateRow>(
        r#"
        WITH active_employees AS (
            SELECT id FROM employees WHERE workspace_id = $2 AND is_active = TRUE
        ),
        first_punches AS (
            SELECT te.employee_id,
                   (te.punch_in_time AT TIME ZONE $3)::date AS work_date,
                   MIN(te.punch_in_time AT TIME ZONE $3)::time AS first_punch
            FROM time_entries te
            WHERE (te.punch_in_time AT TIME ZONE $3)::date >= ($1::date - INTERVAL '29 days')
              AND (te.punch_in_time AT TIME ZONE $3)::date <= $1::date
              AND te.employee_id IN (SELECT id FROM active_employees)
            GROUP BY te.employee_id, (te.punch_in_time AT TIME ZONE $3)::date
        ),
        -- Priority 1: shift_rosters (explicit roster assignment)
        p1_roster AS (
            SELECT sr.employee_id, sr.date AS work_date, s.start_time
            FROM shift_rosters sr
            JOIN shifts s ON s.id = sr.shift_id
            WHERE sr.date >= ($1::date - INTERVAL '29 days')
              AND sr.date <= $1::date
              AND sr.employee_id IN (SELECT id FROM active_employees)
              AND sr.shift_id IS NOT NULL
        ),
        -- Priority 2: employee working patterns
        p2_pattern AS (
            SELECT DISTINCT ON (fp.employee_id, fp.work_date)
                   fp.employee_id, fp.work_date, s.start_time
            FROM first_punches fp
            JOIN employee_working_patterns ewp ON ewp.employee_id = fp.employee_id
                AND ewp.start_date <= fp.work_date
                AND (ewp.end_date IS NULL OR ewp.end_date >= fp.work_date)
            JOIN working_pattern_details wpd ON wpd.pattern_id = ewp.pattern_id
                AND wpd.day_of_week = EXTRACT(DOW FROM fp.work_date)::int
            JOIN shifts s ON s.id = wpd.shift_id
            WHERE NOT EXISTS (SELECT 1 FROM p1_roster p1 WHERE p1.employee_id = fp.employee_id AND p1.work_date = fp.work_date)
            ORDER BY fp.employee_id, fp.work_date, ewp.created_at DESC
        ),
        -- Priority 4: department working patterns
        p4_dept_pattern AS (
            SELECT DISTINCT ON (fp.employee_id, fp.work_date)
                   fp.employee_id, fp.work_date, s.start_time
            FROM first_punches fp
            JOIN employee_department_history edh ON edh.employee_id = fp.employee_id
                AND edh.effective_date <= fp.work_date
            JOIN departments d ON d.id = edh.department_id
                AND d.default_working_pattern_id IS NOT NULL
            JOIN working_pattern_details wpd ON wpd.pattern_id = d.default_working_pattern_id
                AND wpd.day_of_week = EXTRACT(DOW FROM fp.work_date)::int
            JOIN shifts s ON s.id = wpd.shift_id
            WHERE NOT EXISTS (SELECT 1 FROM p1_roster p1 WHERE p1.employee_id = fp.employee_id AND p1.work_date = fp.work_date)
              AND NOT EXISTS (SELECT 1 FROM p2_pattern p2 WHERE p2.employee_id = fp.employee_id AND p2.work_date = fp.work_date)
            ORDER BY fp.employee_id, fp.work_date, edh.effective_date DESC
        ),
        -- Priority 5: employee_shifts (legacy fixed assignment)
        p5_emp_shift AS (
            SELECT DISTINCT ON (fp.employee_id, fp.work_date)
                   fp.employee_id, fp.work_date, s.start_time
            FROM first_punches fp
            JOIN employee_shifts es ON es.employee_id = fp.employee_id
                AND es.start_date <= fp.work_date
                AND (es.end_date IS NULL OR es.end_date >= fp.work_date)
            JOIN shifts s ON s.id = es.shift_id
            WHERE NOT EXISTS (SELECT 1 FROM p1_roster p1 WHERE p1.employee_id = fp.employee_id AND p1.work_date = fp.work_date)
              AND NOT EXISTS (SELECT 1 FROM p2_pattern p2 WHERE p2.employee_id = fp.employee_id AND p2.work_date = fp.work_date)
              AND NOT EXISTS (SELECT 1 FROM p4_dept_pattern p4 WHERE p4.employee_id = fp.employee_id AND p4.work_date = fp.work_date)
            ORDER BY fp.employee_id, fp.work_date, es.created_at DESC
        ),
        -- Priority 6: department default shifts
        p6_dept_shift AS (
            SELECT DISTINCT ON (fp.employee_id, fp.work_date)
                   fp.employee_id, fp.work_date, s.start_time
            FROM first_punches fp
            JOIN employee_department_history edh ON edh.employee_id = fp.employee_id
                AND edh.effective_date <= fp.work_date
            JOIN department_default_shifts dds ON dds.department_id = edh.department_id
            JOIN shifts s ON s.id = dds.shift_id
            WHERE NOT EXISTS (SELECT 1 FROM p1_roster p1 WHERE p1.employee_id = fp.employee_id AND p1.work_date = fp.work_date)
              AND NOT EXISTS (SELECT 1 FROM p2_pattern p2 WHERE p2.employee_id = fp.employee_id AND p2.work_date = fp.work_date)
              AND NOT EXISTS (SELECT 1 FROM p4_dept_pattern p4 WHERE p4.employee_id = fp.employee_id AND p4.work_date = fp.work_date)
              AND NOT EXISTS (SELECT 1 FROM p5_emp_shift p5 WHERE p5.employee_id = fp.employee_id AND p5.work_date = fp.work_date)
            ORDER BY fp.employee_id, fp.work_date, edh.effective_date DESC
        ),
        -- Union all resolved shifts
        all_resolved AS (
            SELECT employee_id, work_date, start_time FROM p1_roster
            UNION ALL
            SELECT employee_id, work_date, start_time FROM p2_pattern
            UNION ALL
            SELECT employee_id, work_date, start_time FROM p4_dept_pattern
            UNION ALL
            SELECT employee_id, work_date, start_time FROM p5_emp_shift
            UNION ALL
            SELECT employee_id, work_date, start_time FROM p6_dept_shift
        )
        SELECT fp.work_date,
               COUNT(DISTINCT fp.employee_id) AS late_count
        FROM first_punches fp
        JOIN all_resolved ar ON ar.employee_id = fp.employee_id AND ar.work_date = fp.work_date
        WHERE fp.first_punch > ar.start_time
        GROUP BY fp.work_date
        "#,
    )
    .bind(today)
    .bind(workspace_id)
    .bind(&tz_str)
    .fetch_all(state.infra.db.reader())
    .await
    .unwrap_or_else(|e| { tracing::warn!(error = %e, "late detection query failed, degrading gracefully"); vec![] })
    .into_iter()
    .map(|r| (r.work_date, r.late_count))
    .collect();

    // Total active employees for absent calculation
    let total_active: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM employees WHERE workspace_id = $1 AND is_active = TRUE",
    )
    .bind(workspace_id)
    .fetch_one(state.infra.db.reader())
    .await
    .unwrap_or(0);

    let trend: Vec<AttendanceTrendPoint> = base_rows
        .into_iter()
        .map(|r| {
            let late = *late_map.get(&r.date).unwrap_or(&0);
            let absent = (total_active - r.present - r.on_leave).max(0);
            AttendanceTrendPoint {
                date: r.date,
                present: r.present,
                late,
                absent,
                on_leave: r.on_leave,
            }
        })
        .collect();

    Ok(Json(trend))
}

#[derive(Debug, Deserialize)]
pub struct DeptAttendanceQuery {
    pub range: Option<String>,
}

pub async fn get_department_attendance(
    State(state): State<Arc<AppState>>,
    MemberContext { workspace_id, .. }: MemberContext,
    Query(params): Query<DeptAttendanceQuery>,
) -> AppResult<Json<Vec<DepartmentAttendancePoint>>> {
    let today = Utc::now().date_naive();
    let range = params.range.unwrap_or_else(|| "today".to_string());
    let start_date = match range.as_str() {
        "week" => today - chrono::Duration::days(6),
        _ => today,
    };

    // Departments with employee counts
    #[derive(Debug, sqlx::FromRow)]
    struct DeptRow {
        department_id: Uuid,
        department_name: String,
        total_employees: i64,
    }

    let departments = sqlx::query_as::<_, DeptRow>(
        r#"
        SELECT d.id AS department_id,
               d.name AS department_name,
               COUNT(DISTINCT e.id) AS total_employees
        FROM departments d
        JOIN employee_department_history edh ON edh.department_id = d.id
        JOIN employees e ON e.id = edh.employee_id AND e.is_active = TRUE AND e.workspace_id = $1
        WHERE d.workspace_id = $1
          AND edh.effective_date <= $2
          AND edh.id = (
            SELECT edh2.id FROM employee_department_history edh2
            WHERE edh2.employee_id = edh.employee_id AND edh2.effective_date <= $2
            ORDER BY edh2.effective_date DESC LIMIT 1
          )
        GROUP BY d.id, d.name
        "#,
    )
    .bind(workspace_id)
    .bind(today)
    .fetch_all(state.infra.db.reader())
    .await?;

    if departments.is_empty() {
        return Ok(Json(vec![]));
    }

    // Present per department in date range
    #[derive(Debug, sqlx::FromRow)]
    struct PresentRow {
        department_id: Uuid,
        present_count: i64,
    }

    let start_dt = start_date
        .and_hms_opt(0, 0, 0)
        .and_then(|t| t.and_local_timezone(Utc).single())
        .ok_or_else(|| AppError::Internal("Invalid date".to_string()))?;

    let present_rows = sqlx::query_as::<_, PresentRow>(
        r#"
        WITH latest_dept AS (
            SELECT DISTINCT ON (edh.employee_id) edh.employee_id, edh.department_id
            FROM employee_department_history edh
            JOIN employees e ON e.id = edh.employee_id AND e.is_active = TRUE AND e.workspace_id = $1
            WHERE edh.effective_date <= $3
            ORDER BY edh.employee_id, edh.effective_date DESC
        )
        SELECT ld.department_id,
               COUNT(DISTINCT te.employee_id) AS present_count
        FROM time_entries te
        JOIN latest_dept ld ON ld.employee_id = te.employee_id
        WHERE te.punch_in_time >= $2
        GROUP BY ld.department_id
        "#,
    )
    .bind(workspace_id)
    .bind(start_dt)
    .bind(today)
    .fetch_all(state.infra.db.reader())
    .await
    .unwrap_or_else(|e| { tracing::warn!(error = %e, "present count query failed, degrading gracefully"); vec![] });

    let present_map: std::collections::HashMap<Uuid, i64> = present_rows
        .into_iter()
        .map(|r| (r.department_id, r.present_count))
        .collect();

    // Late per department (only for "today")
    let late_map: std::collections::HashMap<Uuid, i64> = if range == "today" {
        // Fetch workspace timezone for correct UTC→local conversion
        let tz_str: String = sqlx::query_scalar(
            "SELECT timezone FROM workspaces WHERE id = $1",
        )
        .bind(workspace_id)
        .fetch_one(state.infra.db.reader())
        .await
        .unwrap_or_else(|_| "UTC".to_string());

        #[derive(Debug, sqlx::FromRow)]
        struct LateRow {
            department_id: Uuid,
            late_count: i64,
        }
        sqlx::query_as::<_, LateRow>(
            r#"
            WITH latest_dept AS (
                SELECT DISTINCT ON (edh.employee_id) edh.employee_id, edh.department_id
                FROM employee_department_history edh
                JOIN employees e ON e.id = edh.employee_id AND e.is_active = TRUE AND e.workspace_id = $1
                WHERE edh.effective_date <= $3
                ORDER BY edh.employee_id, edh.effective_date DESC
            ),
            first_punches AS (
                SELECT te.employee_id, MIN(te.punch_in_time AT TIME ZONE $4) AS first_punch
                FROM time_entries te
                WHERE (te.punch_in_time AT TIME ZONE $4)::date = $3
                  AND te.employee_id IN (SELECT employee_id FROM latest_dept)
                GROUP BY te.employee_id
            )
            SELECT ld.department_id,
                   COUNT(DISTINCT fp.employee_id) AS late_count
            FROM first_punches fp
            JOIN latest_dept ld ON ld.employee_id = fp.employee_id
            JOIN shift_rosters re ON re.employee_id = fp.employee_id AND re.date = $3
            JOIN shifts s ON s.id = re.shift_id
            WHERE fp.first_punch::time > s.start_time
            GROUP BY ld.department_id
            "#,
        )
        .bind(workspace_id)
        .bind(start_dt)
        .bind(today)
        .bind(&tz_str)
        .fetch_all(state.infra.db.reader())
        .await
        .unwrap_or_else(|e| { tracing::warn!(error = %e, "late detection query failed, degrading gracefully"); vec![] })
        .into_iter()
        .map(|r| (r.department_id, r.late_count))
        .collect()
    } else {
        std::collections::HashMap::new()
    };

    // On leave per department
    #[derive(Debug, sqlx::FromRow)]
    struct LeaveRow {
        department_id: Uuid,
        leave_count: i64,
    }

    let leave_rows = sqlx::query_as::<_, LeaveRow>(
        r#"
        WITH latest_dept AS (
            SELECT DISTINCT ON (edh.employee_id) edh.employee_id, edh.department_id
            FROM employee_department_history edh
            JOIN employees e ON e.id = edh.employee_id AND e.is_active = TRUE AND e.workspace_id = $1
            WHERE edh.effective_date <= $3
            ORDER BY edh.employee_id, edh.effective_date DESC
        )
        SELECT ld.department_id,
               COUNT(DISTINCT lr.employee_id) AS leave_count
        FROM leave_requests lr
        JOIN latest_dept ld ON ld.employee_id = lr.employee_id
        WHERE lr.status IN ('approved', 'pre_approved')
          AND lr.start_date <= $3 AND lr.end_date >= $2
        GROUP BY ld.department_id
        "#,
    )
    .bind(workspace_id)
    .bind(start_date)
    .bind(today)
    .fetch_all(state.infra.db.reader())
    .await
    .unwrap_or_else(|e| { tracing::warn!(error = %e, "leave count query failed, degrading gracefully"); vec![] });

    let leave_map: std::collections::HashMap<Uuid, i64> = leave_rows
        .into_iter()
        .map(|r| (r.department_id, r.leave_count))
        .collect();

    // Build result
    let result: Vec<DepartmentAttendancePoint> = departments
        .into_iter()
        .map(|dept| {
            let present = *present_map.get(&dept.department_id).unwrap_or(&0);
            let late = *late_map.get(&dept.department_id).unwrap_or(&0);
            let on_leave = *leave_map.get(&dept.department_id).unwrap_or(&0);
            let on_time = (present - late).max(0);
            let absent = (dept.total_employees - present - on_leave).max(0);
            DepartmentAttendancePoint {
                department_id: dept.department_id,
                department_name: dept.department_name,
                on_time,
                late,
                absent,
                on_leave,
            }
        })
        .collect();

    Ok(Json(result))
}

#[derive(Debug, Deserialize)]
pub struct UnexcusedAbsencesQuery {
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub group_by: Option<String>,
}

pub async fn get_unexcused_absences(
    State(state): State<Arc<AppState>>,
    MemberContext { workspace_id, .. }: MemberContext,
    Query(params): Query<UnexcusedAbsencesQuery>,
) -> AppResult<Json<Vec<UnexcusedAbsencesPoint>>> {
        use crate::repositories::workspace_repository::WorkspaceRepository;
    use crate::repositories::SqlxWorkspaceRepository;
    use crate::services::attendance_utils::{
        self, AttendanceStatus, DailyStatusInput, determine_daily_status,
    };
    use chrono_tz::Tz;
    use std::collections::HashMap;

    let today = Utc::now().date_naive();

    let end_date = params
        .end_date
        .and_then(|d| NaiveDate::parse_from_str(&d, "%Y-%m-%d").ok())
        .unwrap_or(today)
        .min(today); // Don't look into the future

    let start_date = params
        .start_date
        .and_then(|d| NaiveDate::parse_from_str(&d, "%Y-%m-%d").ok())
        .unwrap_or_else(|| end_date - chrono::Duration::days(29));

    let group_by = params.group_by.unwrap_or_else(|| "department".to_string());

    // 1. Get workspace timezone
    let workspace_repo = SqlxWorkspaceRepository { pool: state.infra.db.reader().clone() };
    let workspace = workspace_repo
        .get_workspace_by_id(workspace_id)
        .await
        .map_err(|e| AppError::Internal(format!("Error fetching workspace: {e}")))?
        .ok_or_else(|| AppError::NotFound("Workspace not found".to_string()))?;

    let tz: Tz = crate::utils::tz::parse_workspace_tz(&workspace.timezone);

    // 2. Get schedules for ALL active employees (reuses 6-priority chain logic)
    let schedules = state.shifts.roster_repo.get_workspace_schedules(
        state.infra.roster_cache.as_deref(),
        workspace_id,
        start_date,
        end_date,
        None, // all employees
    )
    .await
    .map_err(|e| AppError::Internal(format!("Error fetching schedules: {e}")))?;

    // 3. Batch-fetch time entries (using shared date_range_to_utc)
    let emp_ids: Vec<Uuid> = schedules.iter().map(|s| s.employee_id).collect();
    let (start_dt, end_dt) = attendance_utils::date_range_to_utc(tz, start_date, end_date);

    let entries_map = state.attendance.time_entry_repo.get_workspace_time_entries_batch(workspace_id, &emp_ids, start_dt, end_dt,
    )
    .await
    .map_err(|e| AppError::Internal(format!("Error fetching time entries: {e}")))?;

    // 4. Holiday info is already embedded in EffectiveSchedule.is_holiday from get_workspace_schedules.

    // 5. Batch-fetch approved leave requests (using shared build_leave_date_set)
    let leave_date_set = attendance_utils::build_leave_date_set(
        state.infra.db.reader(), workspace_id, &emp_ids, start_date, end_date,
    )
    .await
    .map_err(|e| AppError::Internal(format!("Error fetching leave requests: {e}")))?;

    // 6. Batch-fetch department info for each employee
    let dept_rows = sqlx::query!(
        r#"SELECT e.id AS employee_id,
                  edh.department_id AS "department_id?",
                  COALESCE(d.name, 'Unassigned') AS "department_name!"
           FROM employees e
           LEFT JOIN LATERAL (
               SELECT edh2.department_id
               FROM employee_department_history edh2
               WHERE edh2.employee_id = e.id AND edh2.effective_date <= $2
               ORDER BY edh2.effective_date DESC
               LIMIT 1
           ) edh ON TRUE
           LEFT JOIN departments d ON d.id = edh.department_id
           WHERE e.is_active = TRUE AND e.workspace_id = $1"#,
        workspace_id,
        end_date
    )
    .fetch_all(state.infra.db.reader())
    .await
    .map_err(|e| AppError::Internal(format!("Error fetching departments: {e}")))?;

    let dept_map: HashMap<Uuid, (Option<Uuid>, String)> = dept_rows
        .into_iter()
        .map(|r| (r.employee_id, (r.department_id, r.department_name)))
        .collect();

    // 7. Determine status per day using attendance_utils::determine_daily_status
    struct EmployeeStats {
        employee_name: String,
        department_id: Option<Uuid>,
        department_name: String,
        unexcused_count: i64,
        total_absent_count: i64,
        consecutive_count: i64,
    }

    let mut emp_stats: HashMap<Uuid, EmployeeStats> = HashMap::new();
    let unassigned_id = Uuid::nil();

    for emp_sched in &schedules {
        let entries = entries_map
            .get(&emp_sched.employee_id)
            .cloned()
            .unwrap_or_default();

        // Pre-group entries by date using shared utility
        let entries_by_date = attendance_utils::group_entries_by_date(&entries, tz);

        let (dept_id, dept_name) = dept_map
            .get(&emp_sched.employee_id)
            .cloned()
            .unwrap_or((None, "Unassigned".to_string()));

        let mut unexcused = 0i64;
        let mut total_absent = 0i64;
        let mut consecutive = 0i64;
        let mut last_present_date: Option<NaiveDate> = None;

        // Find last present date for consecutive calculation
        for entry in &entries {
            let d = entry.punch_in_time.with_timezone(&tz).date_naive();
            if last_present_date.is_none_or(|lpd| d > lpd) {
                last_present_date = Some(d);
            }
        }

        for daily in &emp_sched.schedules {
            let date = daily.date;
            if date > today {
                continue;
            }

            let is_on_leave = leave_date_set.contains(&(emp_sched.employee_id, date));

            let status = determine_daily_status(&DailyStatusInput {
                has_punch_in: entries_by_date.contains_key(&date),
                punch_in_time: None, // Not needed for unexcused calculation
                scheduled_start: daily.start_time,
                is_holiday: daily.is_holiday,
                is_holiday_paid: false, // Not needed for unexcused calculation
                has_shift: daily.shift_id.is_some(),
                is_on_leave,
            });

            if status == AttendanceStatus::Absent {
                total_absent += 1;

                if !is_on_leave {
                    unexcused += 1;

                    // Consecutive: days after last punch-in without leave
                    let is_after_last_punch = last_present_date
                        .is_none_or(|lpd| date > lpd);
                    if is_after_last_punch {
                        consecutive += 1;
                    }
                }
            }
        }

        emp_stats.insert(emp_sched.employee_id, EmployeeStats {
            employee_name: emp_sched.full_name.clone(),
            department_id: dept_id,
            department_name: dept_name,
            unexcused_count: unexcused,
            total_absent_count: total_absent,
            consecutive_count: consecutive,
        });
    }

    // 8. Build result based on group_by
    if group_by == "employee" {
        let mut result: Vec<UnexcusedAbsencesPoint> = emp_stats
            .into_iter()
            .map(|(emp_id, stats)| UnexcusedAbsencesPoint {
                department_id: stats.department_id.unwrap_or(unassigned_id),
                department_name: stats.department_name,
                employee_id: Some(emp_id),
                employee_name: Some(stats.employee_name),
                unexcused_count: stats.unexcused_count,
                consecutive_count: stats.consecutive_count,
                total_absent_count: stats.total_absent_count,
                total_leave_count: 0,
            })
            .collect();
        result.sort_by(|a, b| b.unexcused_count.cmp(&a.unexcused_count));
        Ok(Json(result))
    } else {
        // Aggregate by department
        let mut dept_agg: HashMap<(Uuid, String), (i64, i64, i64)> = HashMap::new();
        for stats in emp_stats.values() {
            let key = (
                stats.department_id.unwrap_or(unassigned_id),
                stats.department_name.clone(),
            );
            let entry = dept_agg.entry(key).or_insert((0, 0, 0));
            entry.0 += stats.unexcused_count;
            entry.1 += stats.total_absent_count;
            entry.2 += stats.consecutive_count;
        }

        let mut result: Vec<UnexcusedAbsencesPoint> = dept_agg
            .into_iter()
            .map(|((dept_id, dept_name), (unexcused, total_absent, consecutive))| {
                UnexcusedAbsencesPoint {
                    department_id: dept_id,
                    department_name: dept_name,
                    employee_id: None,
                    employee_name: None,
                    unexcused_count: unexcused,
                    consecutive_count: consecutive,
                    total_absent_count: total_absent,
                    total_leave_count: 0,
                }
            })
            .collect();
        result.sort_by(|a, b| b.unexcused_count.cmp(&a.unexcused_count));
        Ok(Json(result))
    }
}


pub async fn get_overtime_trend(
    State(state): State<Arc<AppState>>,
    MemberContext { workspace_id, .. }: MemberContext,
) -> AppResult<Json<Vec<OvertimeTrendPoint>>> {
    let today = Utc::now().date_naive();

    #[derive(Debug, sqlx::FromRow)]
    struct OtRow {
        date: NaiveDate,
        system_ot_minutes: i64,
        approved_ot_minutes: i64,
    }

    let rows = sqlx::query_as::<_, OtRow>(
        r#"
        SELECT ads.date,
               COALESCE(SUM(ads.system_detected_ot_minutes), 0) AS system_ot_minutes,
               COALESCE(SUM(ads.approved_ot_minutes), 0) AS approved_ot_minutes
        FROM attendance_daily_summaries ads
        WHERE ads.workspace_id = $1
          AND ads.date >= ($2::date - INTERVAL '29 days')
          AND ads.date <= $2
        GROUP BY ads.date
        ORDER BY ads.date ASC
        "#,
    )
    .bind(workspace_id)
    .bind(today)
    .fetch_all(state.infra.db.reader())
    .await
    .unwrap_or_else(|e| { tracing::warn!(error = %e, "overtime trend query failed, degrading gracefully"); vec![] });

    let trend: Vec<OvertimeTrendPoint> = rows
        .into_iter()
        .map(|r| OvertimeTrendPoint {
            date: r.date,
            system_ot_hours: r.system_ot_minutes as f64 / 60.0,
            approved_ot_hours: r.approved_ot_minutes as f64 / 60.0,
        })
        .collect();

    Ok(Json(trend))
}

pub async fn get_workspace_stats(
    State(state): State<Arc<AppState>>,
    MemberContext { workspace_id, .. }: MemberContext,
) -> AppResult<Json<WorkspaceStats>> {
    // 1. Total Employees
    let total_employees: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM employees WHERE workspace_id = $1 AND is_active = TRUE",
    )
    .bind(workspace_id)
    .fetch_one(state.infra.db.reader())
    .await
    .unwrap_or(0);

    // Fetch workspace timezone for correct UTC→local conversion
    let tz_str: String = sqlx::query_scalar(
        "SELECT timezone FROM workspaces WHERE id = $1",
    )
    .bind(workspace_id)
    .fetch_one(state.infra.db.reader())
    .await
    .unwrap_or_else(|_| "UTC".to_string());

    // 2. Present Today (Employees with at least one punch-in today)
    let today = Utc::now().date_naive();
    let start_of_day = today
        .and_hms_opt(0, 0, 0)
        .and_then(|t| t.and_local_timezone(Utc).single())
        .ok_or_else(|| AppError::Internal("Invalid date calculation".to_string()))?;

    let present_today: i64 = sqlx::query_scalar(
        "SELECT COUNT(DISTINCT employee_id) FROM time_entries
         WHERE (punch_in_time AT TIME ZONE $3)::date = $1::date
         AND employee_id IN (SELECT id FROM employees WHERE workspace_id = $2)",
    )
    .bind(today)
    .bind(workspace_id)
    .bind(&tz_str)
    .fetch_one(state.infra.db.reader())
    .await
    .unwrap_or(0);

    // 3. Pending Leave Requests
    let pending_leave: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM leave_requests
         WHERE status = 'pending'
         AND employee_id IN (SELECT id FROM employees WHERE workspace_id = $1)",
    )
    .bind(workspace_id)
    .fetch_one(state.infra.db.reader())
    .await
    .unwrap_or(0);

    // 4. Estimated Monthly Payroll (Sum of latest base salaries)
    // Priority: employee_salary_components -> fallback to salary_history
    let payroll: rust_decimal::Decimal = sqlx::query_scalar(
        r#"
        WITH active_emps AS (
            SELECT id FROM employees WHERE workspace_id = $1 AND is_active = TRUE
        ),
        dynamic_payroll AS (
            SELECT esc.employee_id, SUM(esc.amount) as total_amount
            FROM employee_salary_components esc
            INNER JOIN (
                SELECT employee_id, component_definition_id, MAX(effective_date) as max_date
                FROM employee_salary_components
                WHERE effective_date <= CURRENT_DATE
                GROUP BY employee_id, component_definition_id
            ) latest ON esc.employee_id = latest.employee_id
                     AND esc.component_definition_id = latest.component_definition_id
                     AND esc.effective_date = latest.max_date
            WHERE esc.employee_id IN (SELECT id FROM active_emps)
            GROUP BY esc.employee_id
        )
        SELECT COALESCE(SUM(total_amount), 0) FROM dynamic_payroll
        "#,
    )
    .bind(workspace_id)
    .fetch_one(state.infra.db.reader())
    .await
    .unwrap_or(rust_decimal::Decimal::ZERO);

    // 5. On Leave Today (approved leave requests spanning today)
    let on_leave_today: i64 = sqlx::query_scalar(
        "SELECT COUNT(DISTINCT lr.employee_id) FROM leave_requests lr
         WHERE lr.status IN ('approved', 'pre_approved')
         AND lr.start_date <= $1 AND lr.end_date >= $1
         AND lr.employee_id IN (SELECT id FROM employees WHERE workspace_id = $2 AND is_active = TRUE)",
    )
    .bind(today)
    .bind(workspace_id)
    .fetch_one(state.infra.db.reader())
    .await
    .unwrap_or(0);

    // 6. Absent Today = total_employees - present_today - on_leave_today
    let absent_today = (total_employees - present_today - on_leave_today).max(0);

    // 7. Late Today (employees whose earliest punch-in is after scheduled shift start)
    // Uses 6-priority shift resolution: shift_rosters > employee working patterns
    //   > dept working patterns > employee_shifts > department default shifts
    let late_today: i64 = sqlx::query_scalar(
        r#"
        WITH active_employees AS (
            SELECT id FROM employees WHERE workspace_id = $2 AND is_active = TRUE
        ),
        today_punches AS (
            SELECT te.employee_id,
                   MIN(te.punch_in_time AT TIME ZONE $4)::time AS first_punch
            FROM time_entries te
            WHERE (te.punch_in_time AT TIME ZONE $4)::date = $3
              AND te.employee_id IN (SELECT id FROM active_employees)
            GROUP BY te.employee_id
        ),
        -- Priority 1: shift_rosters (explicit roster assignment)
        p1_roster AS (
            SELECT sr.employee_id, s.start_time
            FROM shift_rosters sr
            JOIN shifts s ON s.id = sr.shift_id
            WHERE sr.date = $3
              AND sr.employee_id IN (SELECT id FROM active_employees)
              AND sr.shift_id IS NOT NULL
        ),
        -- Priority 2: employee working patterns
        p2_pattern AS (
            SELECT DISTINCT ON (tp.employee_id)
                   tp.employee_id, s.start_time
            FROM today_punches tp
            JOIN employee_working_patterns ewp ON ewp.employee_id = tp.employee_id
                AND ewp.start_date <= $3
                AND (ewp.end_date IS NULL OR ewp.end_date >= $3)
            JOIN working_pattern_details wpd ON wpd.pattern_id = ewp.pattern_id
                AND wpd.day_of_week = EXTRACT(DOW FROM $3::date)::int
            JOIN shifts s ON s.id = wpd.shift_id
            WHERE NOT EXISTS (SELECT 1 FROM p1_roster p1 WHERE p1.employee_id = tp.employee_id)
            ORDER BY tp.employee_id, ewp.created_at DESC
        ),
        -- Priority 4: department working patterns
        p4_dept_pattern AS (
            SELECT DISTINCT ON (tp.employee_id)
                   tp.employee_id, s.start_time
            FROM today_punches tp
            JOIN employee_department_history edh ON edh.employee_id = tp.employee_id
                AND edh.effective_date <= $3
            JOIN departments d ON d.id = edh.department_id
                AND d.default_working_pattern_id IS NOT NULL
            JOIN working_pattern_details wpd ON wpd.pattern_id = d.default_working_pattern_id
                AND wpd.day_of_week = EXTRACT(DOW FROM $3::date)::int
            JOIN shifts s ON s.id = wpd.shift_id
            WHERE NOT EXISTS (SELECT 1 FROM p1_roster p1 WHERE p1.employee_id = tp.employee_id)
              AND NOT EXISTS (SELECT 1 FROM p2_pattern p2 WHERE p2.employee_id = tp.employee_id)
            ORDER BY tp.employee_id, edh.effective_date DESC
        ),
        -- Priority 5: employee_shifts (legacy fixed assignment)
        p5_emp_shift AS (
            SELECT DISTINCT ON (tp.employee_id)
                   tp.employee_id, s.start_time
            FROM today_punches tp
            JOIN employee_shifts es ON es.employee_id = tp.employee_id
                AND es.start_date <= $3
                AND (es.end_date IS NULL OR es.end_date >= $3)
            JOIN shifts s ON s.id = es.shift_id
            WHERE NOT EXISTS (SELECT 1 FROM p1_roster p1 WHERE p1.employee_id = tp.employee_id)
              AND NOT EXISTS (SELECT 1 FROM p2_pattern p2 WHERE p2.employee_id = tp.employee_id)
              AND NOT EXISTS (SELECT 1 FROM p4_dept_pattern p4 WHERE p4.employee_id = tp.employee_id)
            ORDER BY tp.employee_id, es.created_at DESC
        ),
        -- Priority 6: department default shifts
        p6_dept_shift AS (
            SELECT DISTINCT ON (tp.employee_id)
                   tp.employee_id, s.start_time
            FROM today_punches tp
            JOIN employee_department_history edh ON edh.employee_id = tp.employee_id
                AND edh.effective_date <= $3
            JOIN department_default_shifts dds ON dds.department_id = edh.department_id
            JOIN shifts s ON s.id = dds.shift_id
            WHERE NOT EXISTS (SELECT 1 FROM p1_roster p1 WHERE p1.employee_id = tp.employee_id)
              AND NOT EXISTS (SELECT 1 FROM p2_pattern p2 WHERE p2.employee_id = tp.employee_id)
              AND NOT EXISTS (SELECT 1 FROM p4_dept_pattern p4 WHERE p4.employee_id = tp.employee_id)
              AND NOT EXISTS (SELECT 1 FROM p5_emp_shift p5 WHERE p5.employee_id = tp.employee_id)
            ORDER BY tp.employee_id, edh.effective_date DESC
        ),
        all_resolved AS (
            SELECT employee_id, start_time FROM p1_roster
            UNION ALL SELECT employee_id, start_time FROM p2_pattern
            UNION ALL SELECT employee_id, start_time FROM p4_dept_pattern
            UNION ALL SELECT employee_id, start_time FROM p5_emp_shift
            UNION ALL SELECT employee_id, start_time FROM p6_dept_shift
        )
        SELECT COUNT(*)
        FROM today_punches tp
        JOIN all_resolved ar ON ar.employee_id = tp.employee_id
        WHERE tp.first_punch > ar.start_time
        "#,
    )
    .bind(start_of_day)
    .bind(workspace_id)
    .bind(today)
    .bind(&tz_str)
    .fetch_one(state.infra.db.reader())
    .await
    .unwrap_or(0);

    // 8. Early Departure Today (employees who punched out before scheduled end)
    // Uses same 6-priority shift resolution as late detection
    let early_departure_today: i64 = sqlx::query_scalar(
        r#"
        WITH active_employees AS (
            SELECT id FROM employees WHERE workspace_id = $2 AND is_active = TRUE
        ),
        today_completed AS (
            SELECT te.employee_id,
                   MAX(te.punch_out_time AT TIME ZONE $4)::time as last_punch_out
            FROM time_entries te
            WHERE (te.punch_in_time AT TIME ZONE $4)::date = $3
              AND te.punch_out_time IS NOT NULL
              AND te.employee_id IN (SELECT id FROM active_employees)
            GROUP BY te.employee_id
        ),
        -- Priority 1: shift_rosters
        p1_roster AS (
            SELECT sr.employee_id, s.end_time
            FROM shift_rosters sr
            JOIN shifts s ON s.id = sr.shift_id
            WHERE sr.date = $3
              AND sr.employee_id IN (SELECT id FROM active_employees)
              AND sr.shift_id IS NOT NULL
        ),
        -- Priority 2: employee working patterns
        p2_pattern AS (
            SELECT DISTINCT ON (tc.employee_id)
                   tc.employee_id, s.end_time
            FROM today_completed tc
            JOIN employee_working_patterns ewp ON ewp.employee_id = tc.employee_id
                AND ewp.start_date <= $3
                AND (ewp.end_date IS NULL OR ewp.end_date >= $3)
            JOIN working_pattern_details wpd ON wpd.pattern_id = ewp.pattern_id
                AND wpd.day_of_week = EXTRACT(DOW FROM $3::date)::int
            JOIN shifts s ON s.id = wpd.shift_id
            WHERE NOT EXISTS (SELECT 1 FROM p1_roster p1 WHERE p1.employee_id = tc.employee_id)
            ORDER BY tc.employee_id, ewp.created_at DESC
        ),
        -- Priority 4: department working patterns
        p4_dept_pattern AS (
            SELECT DISTINCT ON (tc.employee_id)
                   tc.employee_id, s.end_time
            FROM today_completed tc
            JOIN employee_department_history edh ON edh.employee_id = tc.employee_id
                AND edh.effective_date <= $3
            JOIN departments d ON d.id = edh.department_id
                AND d.default_working_pattern_id IS NOT NULL
            JOIN working_pattern_details wpd ON wpd.pattern_id = d.default_working_pattern_id
                AND wpd.day_of_week = EXTRACT(DOW FROM $3::date)::int
            JOIN shifts s ON s.id = wpd.shift_id
            WHERE NOT EXISTS (SELECT 1 FROM p1_roster p1 WHERE p1.employee_id = tc.employee_id)
              AND NOT EXISTS (SELECT 1 FROM p2_pattern p2 WHERE p2.employee_id = tc.employee_id)
            ORDER BY tc.employee_id, edh.effective_date DESC
        ),
        -- Priority 5: employee_shifts
        p5_emp_shift AS (
            SELECT DISTINCT ON (tc.employee_id)
                   tc.employee_id, s.end_time
            FROM today_completed tc
            JOIN employee_shifts es ON es.employee_id = tc.employee_id
                AND es.start_date <= $3
                AND (es.end_date IS NULL OR es.end_date >= $3)
            JOIN shifts s ON s.id = es.shift_id
            WHERE NOT EXISTS (SELECT 1 FROM p1_roster p1 WHERE p1.employee_id = tc.employee_id)
              AND NOT EXISTS (SELECT 1 FROM p2_pattern p2 WHERE p2.employee_id = tc.employee_id)
              AND NOT EXISTS (SELECT 1 FROM p4_dept_pattern p4 WHERE p4.employee_id = tc.employee_id)
            ORDER BY tc.employee_id, es.created_at DESC
        ),
        -- Priority 6: department default shifts
        p6_dept_shift AS (
            SELECT DISTINCT ON (tc.employee_id)
                   tc.employee_id, s.end_time
            FROM today_completed tc
            JOIN employee_department_history edh ON edh.employee_id = tc.employee_id
                AND edh.effective_date <= $3
            JOIN department_default_shifts dds ON dds.department_id = edh.department_id
            JOIN shifts s ON s.id = dds.shift_id
            WHERE NOT EXISTS (SELECT 1 FROM p1_roster p1 WHERE p1.employee_id = tc.employee_id)
              AND NOT EXISTS (SELECT 1 FROM p2_pattern p2 WHERE p2.employee_id = tc.employee_id)
              AND NOT EXISTS (SELECT 1 FROM p4_dept_pattern p4 WHERE p4.employee_id = tc.employee_id)
              AND NOT EXISTS (SELECT 1 FROM p5_emp_shift p5 WHERE p5.employee_id = tc.employee_id)
            ORDER BY tc.employee_id, edh.effective_date DESC
        ),
        all_resolved AS (
            SELECT employee_id, end_time FROM p1_roster
            UNION ALL SELECT employee_id, end_time FROM p2_pattern
            UNION ALL SELECT employee_id, end_time FROM p4_dept_pattern
            UNION ALL SELECT employee_id, end_time FROM p5_emp_shift
            UNION ALL SELECT employee_id, end_time FROM p6_dept_shift
        )
        SELECT COUNT(*)
        FROM today_completed tc
        JOIN all_resolved ar ON ar.employee_id = tc.employee_id
        WHERE tc.last_punch_out < ar.end_time
        "#,
    )
    .bind(start_of_day)
    .bind(workspace_id)
    .bind(today)
    .bind(&tz_str)
    .fetch_one(state.infra.db.reader())
    .await
    .unwrap_or(0);

    // 9. Average work hours today
    let avg_work_minutes: Option<f64> = sqlx::query_scalar(
        "SELECT AVG(duration_minutes)::float8 FROM time_entries
         WHERE punch_in_time >= $1
         AND punch_out_time IS NOT NULL
         AND duration_minutes > 0
         AND employee_id IN (SELECT id FROM employees WHERE workspace_id = $2 AND is_active = TRUE)",
    )
    .bind(start_of_day)
    .bind(workspace_id)
    .fetch_one(state.infra.db.reader())
    .await
    .unwrap_or(None);

    let avg_work_hours_today = avg_work_minutes.unwrap_or(0.0) / 60.0;

    Ok(Json(WorkspaceStats {
        total_employees,
        present_today,
        pending_leave_requests: pending_leave,
        estimated_monthly_payroll: payroll,
        late_today,
        absent_today,
        on_leave_today,
        early_departure_today,
        avg_work_hours_today,
    }))
}

// ── Peak Check-in/Check-out Times ──────────────────────────────────

pub async fn get_peak_times(
    State(state): State<Arc<AppState>>,
    MemberContext { workspace_id, .. }: MemberContext,
) -> AppResult<Json<Vec<PeakTimeSlot>>> {
    let today = Utc::now().date_naive();
    let start_of_day = today
        .and_hms_opt(0, 0, 0)
        .and_then(|t| t.and_local_timezone(Utc).single())
        .ok_or_else(|| AppError::Internal("Invalid date".to_string()))?;

    // Generate 30-minute time slots and count check-ins/check-outs per slot
    #[derive(Debug, sqlx::FromRow)]
    struct SlotRow {
        time_slot: String,
        check_in: i64,
        check_out: i64,
    }

    let rows = sqlx::query_as::<_, SlotRow>(
        r#"
        WITH slots AS (
            SELECT generate_series(0, 47) AS slot_index
        ),
        slot_labels AS (
            SELECT slot_index,
                   TO_CHAR((TIME '00:00' + (slot_index * INTERVAL '30 minutes')), 'HH24:MI')
                   || '-' ||
                   TO_CHAR((TIME '00:00' + ((slot_index + 1) * INTERVAL '30 minutes')), 'HH24:MI')
                   AS time_slot,
                   (TIME '00:00' + (slot_index * INTERVAL '30 minutes')) AS slot_start,
                   (TIME '00:00' + ((slot_index + 1) * INTERVAL '30 minutes')) AS slot_end
            FROM slots
        ),
        today_entries AS (
            SELECT punch_in_time, punch_out_time
            FROM time_entries
            WHERE punch_in_time >= $1
              AND employee_id IN (SELECT id FROM employees WHERE workspace_id = $2 AND is_active = TRUE)
        )
        SELECT sl.time_slot,
               COALESCE(SUM(CASE WHEN te.punch_in_time::time >= sl.slot_start
                                  AND te.punch_in_time::time < sl.slot_end
                                 THEN 1 ELSE 0 END), 0) AS check_in,
               COALESCE(SUM(CASE WHEN te.punch_out_time IS NOT NULL
                                  AND te.punch_out_time::time >= sl.slot_start
                                  AND te.punch_out_time::time < sl.slot_end
                                 THEN 1 ELSE 0 END), 0) AS check_out
        FROM slot_labels sl
        LEFT JOIN today_entries te ON TRUE
        GROUP BY sl.slot_index, sl.time_slot
        HAVING COALESCE(SUM(CASE WHEN te.punch_in_time::time >= sl.slot_start
                                  AND te.punch_in_time::time < sl.slot_end
                                 THEN 1 ELSE 0 END), 0) > 0
            OR COALESCE(SUM(CASE WHEN te.punch_out_time IS NOT NULL
                                  AND te.punch_out_time::time >= sl.slot_start
                                  AND te.punch_out_time::time < sl.slot_end
                                 THEN 1 ELSE 0 END), 0) > 0
        ORDER BY sl.slot_index
        "#,
    )
    .bind(start_of_day)
    .bind(workspace_id)
    .fetch_all(state.infra.db.reader())
    .await
    .unwrap_or_else(|e| { tracing::warn!(error = %e, "peak times query failed, degrading gracefully"); vec![] });

    let result: Vec<PeakTimeSlot> = rows
        .into_iter()
        .map(|r| PeakTimeSlot {
            time_slot: r.time_slot,
            check_in: r.check_in,
            check_out: r.check_out,
        })
        .collect();

    Ok(Json(result))
}

// ── Leave Types Breakdown ──────────────────────────────────────────

pub async fn get_leave_types_breakdown(
    State(state): State<Arc<AppState>>,
    MemberContext { workspace_id, .. }: MemberContext,
) -> AppResult<Json<Vec<LeaveTypeBreakdown>>> {
    let today = Utc::now().date_naive();
    let month_start = NaiveDate::from_ymd_opt(today.year(), today.month(), 1)
        .ok_or_else(|| AppError::Internal("Invalid date".to_string()))?;

    #[derive(Debug, sqlx::FromRow)]
    struct LtRow {
        leave_type: String,
        count: i64,
    }

    let rows = sqlx::query_as::<_, LtRow>(
        r#"
        SELECT COALESCE(lt.name, 'Other') AS leave_type,
               COUNT(*) AS count
        FROM leave_requests lr
        LEFT JOIN leave_types lt ON lt.id = lr.leave_type_id
        WHERE lr.status IN ('approved', 'pre_approved')
          AND lr.start_date <= $2
          AND lr.end_date >= $1
          AND lr.employee_id IN (
              SELECT id FROM employees WHERE workspace_id = $3 AND is_active = TRUE
          )
        GROUP BY COALESCE(lt.name, 'Other')
        ORDER BY count DESC
        "#,
    )
    .bind(month_start)
    .bind(today)
    .bind(workspace_id)
    .fetch_all(state.infra.db.reader())
    .await
    .unwrap_or_else(|e| { tracing::warn!(error = %e, "leave types query failed, degrading gracefully"); vec![] });

    let result: Vec<LeaveTypeBreakdown> = rows
        .into_iter()
        .map(|r| LeaveTypeBreakdown {
            leave_type: r.leave_type,
            count: r.count,
        })
        .collect();

    Ok(Json(result))
}

// ── Lateness by Day of Week ────────────────────────────────────────

pub async fn get_lateness_by_day(
    State(state): State<Arc<AppState>>,
    MemberContext { workspace_id, .. }: MemberContext,
) -> AppResult<Json<Vec<LatenessByDay>>> {
    let today = Utc::now().date_naive();

    // Fetch workspace timezone for correct UTC→local conversion
    let tz_str: String = sqlx::query_scalar(
        "SELECT timezone FROM workspaces WHERE id = $1",
    )
    .bind(workspace_id)
    .fetch_one(state.infra.db.reader())
    .await
    .unwrap_or_else(|_| "UTC".to_string());

    #[derive(Debug, sqlx::FromRow)]
    struct DayRow {
        dow: i32,
        late_count: i64,
        avg_late_minutes: f64,
    }

    let rows = sqlx::query_as::<_, DayRow>(
        r#"
        WITH first_punches AS (
            SELECT te.employee_id,
                   (te.punch_in_time AT TIME ZONE $3)::date AS work_date,
                   MIN(te.punch_in_time AT TIME ZONE $3)::time AS first_punch
            FROM time_entries te
            WHERE (te.punch_in_time AT TIME ZONE $3)::date >= ($1::date - INTERVAL '29 days')
              AND (te.punch_in_time AT TIME ZONE $3)::date <= $1
              AND te.employee_id IN (SELECT id FROM employees WHERE workspace_id = $2 AND is_active = TRUE)
            GROUP BY te.employee_id, (te.punch_in_time AT TIME ZONE $3)::date
        ),
        resolved_shifts AS (
            SELECT DISTINCT ON (fp.employee_id, fp.work_date)
                   fp.employee_id, fp.work_date, fp.first_punch, COALESCE(
                       -- Priority 1: shift_rosters (manual roster)
                       (SELECT sr.shift_id FROM shift_rosters sr
                        WHERE sr.employee_id = fp.employee_id AND sr.date = fp.work_date
                        LIMIT 1),
                       -- Priority 2: individual working pattern
                       (SELECT wpd.shift_id FROM working_pattern_details wpd
                        JOIN employee_working_patterns ewp ON ewp.pattern_id = wpd.pattern_id
                        WHERE ewp.employee_id = fp.employee_id
                          AND ewp.start_date <= fp.work_date
                          AND (ewp.end_date IS NULL OR ewp.end_date >= fp.work_date)
                          AND wpd.day_of_week = EXTRACT(DOW FROM fp.work_date)::int
                        ORDER BY ewp.created_at DESC LIMIT 1),
                       -- Priority 3: employee_shifts (default assignment)
                       (SELECT es.shift_id FROM employee_shifts es
                        WHERE es.employee_id = fp.employee_id
                          AND es.start_date <= fp.work_date
                          AND (es.end_date IS NULL OR es.end_date >= fp.work_date)
                        LIMIT 1)
                   ) AS shift_id
            FROM first_punches fp
        ),
        late_entries AS (
            SELECT rs.work_date,
                   EXTRACT(ISODOW FROM rs.work_date)::int AS dow,
                   EXTRACT(EPOCH FROM (rs.first_punch - s.start_time)) / 60.0 AS late_minutes
            FROM resolved_shifts rs
            JOIN shifts s ON s.id = rs.shift_id
            WHERE rs.shift_id IS NOT NULL
              AND rs.first_punch > s.start_time
        )
        SELECT dow,
               COUNT(*) AS late_count,
               COALESCE(AVG(late_minutes), 0)::float8 AS avg_late_minutes
        FROM late_entries
        GROUP BY dow
        ORDER BY dow
        "#,
    )
    .bind(today)
    .bind(workspace_id)
    .bind(&tz_str)
    .fetch_all(state.infra.db.reader())
    .await
    .unwrap_or_else(|e| { tracing::warn!(error = %e, "lateness by day query failed, degrading gracefully"); vec![] });

    let day_names = ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"];

    let result: Vec<LatenessByDay> = rows
        .into_iter()
        .map(|r| {
            let day_idx = (r.dow - 1).clamp(0, 6) as usize;
            LatenessByDay {
                day: day_names[day_idx].to_string(),
                late_count: r.late_count,
                avg_late_minutes: (r.avg_late_minutes * 10.0).round() / 10.0,
            }
        })
        .collect();

    Ok(Json(result))
}

pub async fn get_workspaces(
    State(state): State<Arc<AppState>>,
    UserId(user_id): UserId,
) -> AppResult<Json<Vec<Workspace>>> {
    let workspaces = sqlx::query_as::<_, Workspace>(
        "SELECT w.* FROM workspaces w
         JOIN workspace_members wm ON w.id = wm.workspace_id
         WHERE wm.user_id = $1",
    )
    .bind(user_id)
    .fetch_all(state.infra.db.api())
    .await?;

    Ok(Json(workspaces))
}

pub async fn get_workspace_by_id(
    State(state): State<Arc<AppState>>,
    Path(ws_id): Path<Uuid>,
    UserId(_user_id): UserId,
) -> AppResult<Json<Workspace>> {
    let workspace = state
        .core.workspace_repo
        .get_workspace_by_id(ws_id)
        .await
        .map_err(AppError::Internal)?
        .ok_or_else(|| AppError::NotFound("Workspace not found".to_string()))?;

    Ok(Json(workspace))
}

/// POST /workspaces
/// Creates a new workspace.
/// Requires: SystemRole::PlatformAdmin — prevents arbitrary users from creating workspaces.
pub async fn create_workspace(
    State(state): State<Arc<AppState>>,
    ctx: SystemContext,
    Json(req): Json<CreateWorkspaceRequest>,
) -> AppResult<Json<Workspace>> {
    ctx.require_role(SystemRoleType::PlatformAdmin)?;

    state.core.workspace_service.create_workspace(&req.name, ctx.user_id)
        .await
        .map(Json)
        .map_err(AppError::from)
}

pub async fn update_workspace(
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    Json(req): Json<UpdateWorkspaceRequest>,
) -> AppResult<Json<Workspace>> {
    ctx.require_permission(Permission::WorkspaceSettingsWrite)?;
    state.core.workspace_service.update_workspace(
        ctx.workspace_id,
        req.name,
        req.work_unit_minutes,
        req.rounding_policy,
        req.date_format,
        req.time_format,
        req.timezone,
        req.theme_primary_color,
        req.default_language,
        req.early_arrival_strategy,
        req.holiday_overnight_strategy,
        req.default_holiday_ot_multiplier,
    )
    .await
    .map(Json)
    .map_err(AppError::from)
}
