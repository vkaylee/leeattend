use crate::errors::AppError;
use crate::models::{
    AttendanceTrendPoint, DepartmentAttendancePoint, LatenessByDay, LeaveTypeBreakdown,
    OvertimeTrendPoint, PeakTimeSlot, WorkspaceStats,
};
use async_trait::async_trait;
use chrono::{Datelike, NaiveDate, Utc};
use sqlx::PgPool;
use uuid::Uuid;

#[async_trait]
pub trait WorkspaceStatsRepository: super::PoolBacked + Send + Sync {
    async fn fetch_attendance_trend(
        &self,
        workspace_id: Uuid,
    ) -> Result<Vec<AttendanceTrendPoint>, AppError>;
    async fn fetch_department_attendance(
        &self,
        workspace_id: Uuid,
        range: &str,
    ) -> Result<Vec<DepartmentAttendancePoint>, AppError>;
    async fn fetch_overtime_trend(
        &self,
        workspace_id: Uuid,
    ) -> Result<Vec<OvertimeTrendPoint>, AppError>;
    async fn fetch_workspace_stats(&self, workspace_id: Uuid) -> Result<WorkspaceStats, AppError>;
    async fn fetch_peak_times(&self, workspace_id: Uuid) -> Result<Vec<PeakTimeSlot>, AppError>;
    async fn fetch_leave_types_breakdown(
        &self,
        workspace_id: Uuid,
    ) -> Result<Vec<LeaveTypeBreakdown>, AppError>;
    async fn fetch_lateness_by_day(
        &self,
        workspace_id: Uuid,
    ) -> Result<Vec<LatenessByDay>, AppError>;
}

pub struct SqlxWorkspaceStatsRepository {
    pub pool: PgPool,
}

impl SqlxWorkspaceStatsRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    async fn get_timezone(&self, workspace_id: Uuid) -> String {
        sqlx::query_scalar("SELECT timezone FROM workspaces WHERE id = $1")
            .bind(workspace_id)
            .fetch_one(&self.pool)
            .await
            .unwrap_or_else(|_| "UTC".to_string())
    }
}

#[async_trait]
impl WorkspaceStatsRepository for SqlxWorkspaceStatsRepository {
    #[allow(clippy::too_many_lines)]
    async fn fetch_attendance_trend(
        &self,
        workspace_id: Uuid,
    ) -> Result<Vec<AttendanceTrendPoint>, AppError> {
        let today = Utc::now().date_naive();

        // Fetch workspace timezone for correct UTC→local conversion
        let tz_str = self.get_timezone(workspace_id).await;

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
        .fetch_all(&self.pool)
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
        WITH first_punches AS (
            SELECT te.employee_id,
                   (te.punch_in_time AT TIME ZONE $3)::date AS work_date,
                   MIN(te.punch_in_time AT TIME ZONE $3)::time AS first_punch
            FROM time_entries te
            WHERE (te.punch_in_time AT TIME ZONE $3)::date >= ($1::date - INTERVAL '29 days')
              AND (te.punch_in_time AT TIME ZONE $3)::date <= $1::date
              AND te.employee_id IN (SELECT id FROM employees WHERE workspace_id = $2 AND is_active = TRUE)
            GROUP BY te.employee_id, (te.punch_in_time AT TIME ZONE $3)::date
        )
        SELECT fp.work_date,
               COUNT(DISTINCT fp.employee_id) AS late_count
        FROM first_punches fp
        JOIN resolve_employee_shift($2, fp.work_date) ar ON ar.employee_id = fp.employee_id
        WHERE fp.first_punch > ar.start_time
        GROUP BY fp.work_date
        "#,
    )
    .bind(today)
    .bind(workspace_id)
    .bind(&tz_str)
    .fetch_all(&self.pool)
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
        .fetch_one(&self.pool)
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

        Ok(trend)
    }

    #[allow(clippy::too_many_lines)]
    async fn fetch_department_attendance(
        &self,
        workspace_id: Uuid,
        range: &str,
    ) -> Result<Vec<DepartmentAttendancePoint>, AppError> {
        let today = Utc::now().date_naive();
        let start_date = match range {
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
        .fetch_all(&self.pool)
        .await?;

        if departments.is_empty() {
            return Ok(vec![]);
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
    .fetch_all(&self.pool)
    .await
    .unwrap_or_else(|e| { tracing::warn!(error = %e, "present count query failed, degrading gracefully"); vec![] });

        let present_map: std::collections::HashMap<Uuid, i64> = present_rows
            .into_iter()
            .map(|r| (r.department_id, r.present_count))
            .collect();

        // Late per department (only for "today")
        let late_map: std::collections::HashMap<Uuid, i64> = if range == "today" {
            // Fetch workspace timezone for correct UTC→local conversion
            let tz_str = self.get_timezone(workspace_id).await;

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
        .fetch_all(&self.pool)
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
    .fetch_all(&self.pool)
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

        Ok(result)
    }

    async fn fetch_overtime_trend(
        &self,
        workspace_id: Uuid,
    ) -> Result<Vec<OvertimeTrendPoint>, AppError> {
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
        .fetch_all(&self.pool)
        .await
        .unwrap_or_else(|e| {
            tracing::warn!(error = %e, "overtime trend query failed, degrading gracefully");
            vec![]
        });

        let trend: Vec<OvertimeTrendPoint> = rows
            .into_iter()
            .map(|r| OvertimeTrendPoint {
                date: r.date,
                system_ot_hours: r.system_ot_minutes as f64 / 60.0,
                approved_ot_hours: r.approved_ot_minutes as f64 / 60.0,
            })
            .collect();

        Ok(trend)
    }

    #[allow(clippy::too_many_lines)]
    async fn fetch_workspace_stats(&self, workspace_id: Uuid) -> Result<WorkspaceStats, AppError> {
        // 1. Total Employees
        let total_employees: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM employees WHERE workspace_id = $1 AND is_active = TRUE",
        )
        .bind(workspace_id)
        .fetch_one(&self.pool)
        .await
        .unwrap_or(0);

        // Fetch workspace timezone for correct UTC→local conversion
        let tz_str = self.get_timezone(workspace_id).await;

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
        .fetch_one(&self.pool)
        .await
        .unwrap_or(0);

        // 3. Pending Leave Requests
        let pending_leave: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM leave_requests
         WHERE status = 'pending'
         AND employee_id IN (SELECT id FROM employees WHERE workspace_id = $1)",
        )
        .bind(workspace_id)
        .fetch_one(&self.pool)
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
        .fetch_one(&self.pool)
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
    .fetch_one(&self.pool)
    .await
    .unwrap_or(0);

        // 6. Absent Today = total_employees - present_today - on_leave_today
        let absent_today = (total_employees - present_today - on_leave_today).max(0);

        // 7. Late Today (employees whose earliest punch-in is after scheduled shift start)
        // Uses 6-priority shift resolution: shift_rosters > employee working patterns
        //   > dept working patterns > employee_shifts > department default shifts
        let late_today: i64 = sqlx::query_scalar(
        r#"
        WITH today_punches AS (
            SELECT te.employee_id,
                   MIN(te.punch_in_time AT TIME ZONE $4)::time AS first_punch
            FROM time_entries te
            WHERE (te.punch_in_time AT TIME ZONE $4)::date = $3
              AND te.employee_id IN (SELECT id FROM employees WHERE workspace_id = $2 AND is_active = TRUE)
            GROUP BY te.employee_id
        )
        SELECT COUNT(*)
        FROM today_punches tp
        JOIN resolve_employee_shift($2, $3) ar ON ar.employee_id = tp.employee_id
        WHERE tp.first_punch > ar.start_time
        "#,
    )
    .bind(start_of_day)
    .bind(workspace_id)
    .bind(today)
    .bind(&tz_str)
    .fetch_one(&self.pool)
    .await
    .unwrap_or(0);

        // 8. Early Departure Today (employees who punched out before scheduled end)
        // Uses same 6-priority shift resolution as late detection
        let early_departure_today: i64 = sqlx::query_scalar(
        r#"
        WITH today_completed AS (
            SELECT te.employee_id,
                   MAX(te.punch_out_time AT TIME ZONE $4)::time as last_punch_out
            FROM time_entries te
            WHERE (te.punch_in_time AT TIME ZONE $4)::date = $3
              AND te.punch_out_time IS NOT NULL
              AND te.employee_id IN (SELECT id FROM employees WHERE workspace_id = $2 AND is_active = TRUE)
            GROUP BY te.employee_id
        )
        SELECT COUNT(*)
        FROM today_completed tc
        JOIN resolve_employee_shift($2, $3) ar ON ar.employee_id = tc.employee_id
        WHERE tc.last_punch_out < ar.end_time
        "#,
    )
    .bind(start_of_day)
    .bind(workspace_id)
    .bind(today)
    .bind(&tz_str)
    .fetch_one(&self.pool)
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
    .fetch_one(&self.pool)
    .await
    .unwrap_or(None);

        let avg_work_hours_today = avg_work_minutes.unwrap_or(0.0) / 60.0;

        Ok(WorkspaceStats {
            total_employees,
            present_today,
            pending_leave_requests: pending_leave,
            estimated_monthly_payroll: payroll,
            late_today,
            absent_today,
            on_leave_today,
            early_departure_today,
            avg_work_hours_today,
        })
    }

    async fn fetch_peak_times(&self, workspace_id: Uuid) -> Result<Vec<PeakTimeSlot>, AppError> {
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
    .fetch_all(&self.pool)
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

        Ok(result)
    }

    async fn fetch_leave_types_breakdown(
        &self,
        workspace_id: Uuid,
    ) -> Result<Vec<LeaveTypeBreakdown>, AppError> {
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
        .fetch_all(&self.pool)
        .await
        .unwrap_or_else(|e| {
            tracing::warn!(error = %e, "leave types query failed, degrading gracefully");
            vec![]
        });

        let result: Vec<LeaveTypeBreakdown> = rows
            .into_iter()
            .map(|r| LeaveTypeBreakdown {
                leave_type: r.leave_type,
                count: r.count,
            })
            .collect();

        Ok(result)
    }

    #[allow(clippy::too_many_lines)]
    async fn fetch_lateness_by_day(
        &self,
        workspace_id: Uuid,
    ) -> Result<Vec<LatenessByDay>, AppError> {
        let today = Utc::now().date_naive();

        // Fetch workspace timezone for correct UTC→local conversion
        let tz_str = self.get_timezone(workspace_id).await;

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
        late_entries AS (
            SELECT fp.work_date,
                   EXTRACT(ISODOW FROM fp.work_date)::int AS dow,
                   EXTRACT(EPOCH FROM (fp.first_punch - rs.start_time)) / 60.0 AS late_minutes
            FROM first_punches fp
            JOIN resolve_employee_shift($2, fp.work_date) rs ON rs.employee_id = fp.employee_id
            WHERE rs.start_time IS NOT NULL
              AND fp.first_punch > rs.start_time
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
    .fetch_all(&self.pool)
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

        Ok(result)
    }
}

impl super::PoolBacked for SqlxWorkspaceStatsRepository {
    fn from_pool(pool: PgPool) -> Self {
        Self::new(pool)
    }
    fn pool(&self) -> &PgPool {
        &self.pool
    }
}
