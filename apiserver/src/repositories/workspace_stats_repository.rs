use crate::diesel_pool::DieselPools;
use crate::errors::AppError;
use crate::models::{
    AttendanceTrendPoint, DepartmentAttendancePoint, LatenessByDay, LeaveTypeBreakdown,
    OvertimeTrendPoint, PeakTimeSlot, WorkspaceStats,
};
use async_trait::async_trait;
use chrono::{Datelike, NaiveDate, Utc};
use diesel::prelude::*;
use diesel_async::RunQueryDsl;

use uuid::Uuid;

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait WorkspaceStatsRepository: Send + Sync {
    async fn fetch_attendance_trend(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
    ) -> Result<Vec<AttendanceTrendPoint>, AppError>;
    async fn fetch_department_attendance(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        range: &str,
    ) -> Result<Vec<DepartmentAttendancePoint>, AppError>;
    async fn fetch_overtime_trend(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
    ) -> Result<Vec<OvertimeTrendPoint>, AppError>;
    async fn fetch_workspace_stats(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
    ) -> Result<WorkspaceStats, AppError>;
    async fn fetch_peak_times(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
    ) -> Result<Vec<PeakTimeSlot>, AppError>;
    async fn fetch_leave_types_breakdown(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
    ) -> Result<Vec<LeaveTypeBreakdown>, AppError>;
    async fn fetch_lateness_by_day(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
    ) -> Result<Vec<LatenessByDay>, AppError>;
}

pub struct DieselWorkspaceStatsRepository {
    pub diesel_pools: DieselPools,
}

impl DieselWorkspaceStatsRepository {
    pub fn new(diesel_pools: DieselPools) -> Self {
        Self { diesel_pools }
    }

    async fn get_timezone(&self, auth_token: &crate::diesel_pool::TenantAuthToken) -> String {
        let workspace_id = auth_token.workspace_id();
        #[derive(QueryableByName)]
        struct TzRow {
            #[diesel(sql_type = diesel::sql_types::Text)]
            timezone: String,
        }

        let Ok(mut conn) = self.diesel_pools.api.get().await else {
            return "UTC".to_string();
        };

        #[allow(clippy::disallowed_methods)]
        diesel::sql_query("SELECT timezone::text FROM workspaces WHERE id = $1")
            .bind::<diesel::sql_types::Uuid, _>(workspace_id)
            .get_result::<TzRow>(&mut *conn)
            .await
            .map(|r| r.timezone)
            .unwrap_or_else(|_| "UTC".to_string())
    }
}

#[async_trait]
impl WorkspaceStatsRepository for DieselWorkspaceStatsRepository {
    #[allow(clippy::too_many_lines)]
    async fn fetch_attendance_trend(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
    ) -> Result<Vec<AttendanceTrendPoint>, AppError> {
        let workspace_id = auth_token.workspace_id();
        let today = Utc::now().date_naive();
        let tz_str = self.get_timezone(auth_token).await;

        let mut conn = self
            .diesel_pools
            .tenant_transaction_conn(auth_token)
            .await?;

        #[derive(QueryableByName)]
        struct BaseRow {
            #[diesel(sql_type = diesel::sql_types::Date)]
            date: NaiveDate,
            #[diesel(sql_type = diesel::sql_types::BigInt)]
            present: i64,
            #[diesel(sql_type = diesel::sql_types::BigInt)]
            on_leave: i64,
        }

        let base_rows: Vec<BaseRow> = diesel::sql_query(
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
               COALESCE(dp.present, 0)::bigint AS present,
               COALESCE(dle.on_leave, 0)::bigint AS on_leave
        FROM daily_present dp
        LEFT JOIN daily_leave dle ON dp.date = dle.date
        ORDER BY dp.date ASC
        "#,
        )
        .bind::<diesel::sql_types::Date, _>(today)
        .bind::<diesel::sql_types::Uuid, _>(workspace_id)
        .bind::<diesel::sql_types::Text, _>(&tz_str)
        .load(&mut *conn)
        .await?;

        #[derive(QueryableByName)]
        struct LateRow {
            #[diesel(sql_type = diesel::sql_types::Date)]
            work_date: NaiveDate,
            #[diesel(sql_type = diesel::sql_types::BigInt)]
            late_count: i64,
        }

        let late_rows: Vec<LateRow> = diesel::sql_query(
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
               COUNT(DISTINCT fp.employee_id)::bigint AS late_count
        FROM first_punches fp
        JOIN resolve_employee_shift($2, fp.work_date) ar ON ar.employee_id = fp.employee_id
        WHERE fp.first_punch > ar.start_time
        GROUP BY fp.work_date
        "#,
        )
        .bind::<diesel::sql_types::Date, _>(today)
        .bind::<diesel::sql_types::Uuid, _>(workspace_id)
        .bind::<diesel::sql_types::Text, _>(&tz_str)
        .load(&mut *conn)
        .await
        .unwrap_or_else(|e| {
            tracing::warn!(error = %e, "late detection query failed, degrading gracefully");
            vec![]
        });

        let late_map: std::collections::HashMap<NaiveDate, i64> = late_rows
            .into_iter()
            .map(|r| (r.work_date, r.late_count))
            .collect();

        #[derive(QueryableByName)]
        struct CountRow {
            #[diesel(sql_type = diesel::sql_types::BigInt)]
            count: i64,
        }

        let total_active: i64 = diesel::sql_query(
            "SELECT COUNT(*)::bigint as count FROM employees WHERE workspace_id = $1 AND is_active = TRUE",
        )
        .bind::<diesel::sql_types::Uuid, _>(workspace_id)
        .get_result::<CountRow>(&mut *conn)
        .await
        .map(|r| r.count)
        .unwrap_or(0);

        let trend = base_rows
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
        auth_token: &crate::diesel_pool::TenantAuthToken,
        range: &str,
    ) -> Result<Vec<DepartmentAttendancePoint>, AppError> {
        let workspace_id = auth_token.workspace_id();
        let today = Utc::now().date_naive();
        let start_date = match range {
            "week" => today - chrono::Duration::days(6),
            _ => today,
        };

        let mut conn = self
            .diesel_pools
            .tenant_transaction_conn(auth_token)
            .await?;

        #[derive(QueryableByName)]
        struct DeptRow {
            #[diesel(sql_type = diesel::sql_types::Uuid)]
            department_id: Uuid,
            #[diesel(sql_type = diesel::sql_types::Text)]
            department_name: String,
            #[diesel(sql_type = diesel::sql_types::BigInt)]
            total_employees: i64,
        }

        let departments: Vec<DeptRow> = diesel::sql_query(
            r#"
        SELECT d.id AS department_id,
               d.name::text AS department_name,
               COUNT(DISTINCT e.id)::bigint AS total_employees
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
        .bind::<diesel::sql_types::Uuid, _>(workspace_id)
        .bind::<diesel::sql_types::Date, _>(today)
        .load(&mut *conn)
        .await?;

        if departments.is_empty() {
            return Ok(vec![]);
        }

        let start_dt = start_date
            .and_hms_opt(0, 0, 0)
            .and_then(|t| t.and_local_timezone(Utc).single())
            .ok_or_else(|| AppError::Internal("Invalid date".to_string()))?;

        #[derive(QueryableByName)]
        struct PresentRow {
            #[diesel(sql_type = diesel::sql_types::Uuid)]
            department_id: Uuid,
            #[diesel(sql_type = diesel::sql_types::BigInt)]
            present_count: i64,
        }

        let present_rows: Vec<PresentRow> = diesel::sql_query(
            r#"
        WITH latest_dept AS (
            SELECT DISTINCT ON (edh.employee_id) edh.employee_id, edh.department_id
            FROM employee_department_history edh
            JOIN employees e ON e.id = edh.employee_id AND e.is_active = TRUE AND e.workspace_id = $1
            WHERE edh.effective_date <= $3
            ORDER BY edh.employee_id, edh.effective_date DESC
        )
        SELECT ld.department_id,
               COUNT(DISTINCT te.employee_id)::bigint AS present_count
        FROM time_entries te
        JOIN latest_dept ld ON ld.employee_id = te.employee_id
        WHERE te.punch_in_time >= $2
        GROUP BY ld.department_id
        "#,
        )
        .bind::<diesel::sql_types::Uuid, _>(workspace_id)
        .bind::<diesel::sql_types::Timestamptz, _>(start_dt)
        .bind::<diesel::sql_types::Date, _>(today)
        .load(&mut *conn)
        .await
        .unwrap_or_else(|e| {
            tracing::warn!(error = %e, "present count query failed, degrading gracefully");
            vec![]
        });

        let present_map: std::collections::HashMap<Uuid, i64> = present_rows
            .into_iter()
            .map(|r| (r.department_id, r.present_count))
            .collect();

        let late_map: std::collections::HashMap<Uuid, i64> = if range == "today" {
            let tz_str = self.get_timezone(auth_token).await;

            #[derive(QueryableByName)]
            struct LateRow {
                #[diesel(sql_type = diesel::sql_types::Uuid)]
                department_id: Uuid,
                #[diesel(sql_type = diesel::sql_types::BigInt)]
                late_count: i64,
            }
            let late_rows: Vec<LateRow> = diesel::sql_query(
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
                   COUNT(DISTINCT fp.employee_id)::bigint AS late_count
            FROM first_punches fp
            JOIN latest_dept ld ON ld.employee_id = fp.employee_id
            JOIN shift_rosters re ON re.employee_id = fp.employee_id AND re.date = $3
            JOIN shifts s ON s.id = re.shift_id
            WHERE fp.first_punch::time > s.start_time
            GROUP BY ld.department_id
            "#,
            )
            .bind::<diesel::sql_types::Uuid, _>(workspace_id)
            .bind::<diesel::sql_types::Timestamptz, _>(start_dt)
            .bind::<diesel::sql_types::Date, _>(today)
            .bind::<diesel::sql_types::Text, _>(&tz_str)
            .load(&mut *conn)
            .await
            .unwrap_or_else(|e| {
                tracing::warn!(error = %e, "late detection query failed, degrading gracefully");
                vec![]
            });
            late_rows
                .into_iter()
                .map(|r| (r.department_id, r.late_count))
                .collect()
        } else {
            std::collections::HashMap::new()
        };

        #[derive(QueryableByName)]
        struct LeaveRow {
            #[diesel(sql_type = diesel::sql_types::Uuid)]
            department_id: Uuid,
            #[diesel(sql_type = diesel::sql_types::BigInt)]
            leave_count: i64,
        }

        let leave_rows: Vec<LeaveRow> = diesel::sql_query(
            r#"
        WITH latest_dept AS (
            SELECT DISTINCT ON (edh.employee_id) edh.employee_id, edh.department_id
            FROM employee_department_history edh
            JOIN employees e ON e.id = edh.employee_id AND e.is_active = TRUE AND e.workspace_id = $1
            WHERE edh.effective_date <= $3
            ORDER BY edh.employee_id, edh.effective_date DESC
        )
        SELECT ld.department_id,
               COUNT(DISTINCT lr.employee_id)::bigint AS leave_count
        FROM leave_requests lr
        JOIN latest_dept ld ON ld.employee_id = lr.employee_id
        WHERE lr.status IN ('approved', 'pre_approved')
          AND lr.start_date <= $3 AND lr.end_date >= $2
        GROUP BY ld.department_id
        "#,
        )
        .bind::<diesel::sql_types::Uuid, _>(workspace_id)
        .bind::<diesel::sql_types::Date, _>(start_date)
        .bind::<diesel::sql_types::Date, _>(today)
        .load(&mut *conn)
        .await
        .unwrap_or_else(|e| {
            tracing::warn!(error = %e, "leave count query failed, degrading gracefully");
            vec![]
        });

        let leave_map: std::collections::HashMap<Uuid, i64> = leave_rows
            .into_iter()
            .map(|r| (r.department_id, r.leave_count))
            .collect();

        let result = departments
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
        auth_token: &crate::diesel_pool::TenantAuthToken,
    ) -> Result<Vec<OvertimeTrendPoint>, AppError> {
        let workspace_id = auth_token.workspace_id();
        let today = Utc::now().date_naive();
        let mut conn = self
            .diesel_pools
            .tenant_transaction_conn(auth_token)
            .await?;

        #[derive(QueryableByName)]
        struct OtRow {
            #[diesel(sql_type = diesel::sql_types::Date)]
            date: NaiveDate,
            #[diesel(sql_type = diesel::sql_types::BigInt)]
            system_ot_minutes: i64,
            #[diesel(sql_type = diesel::sql_types::BigInt)]
            approved_ot_minutes: i64,
        }

        let rows: Vec<OtRow> = diesel::sql_query(
            r#"
        SELECT ads.date,
               COALESCE(SUM(ads.system_detected_ot_minutes), 0)::bigint AS system_ot_minutes,
               COALESCE(SUM(ads.approved_ot_minutes), 0)::bigint AS approved_ot_minutes
        FROM attendance_daily_summaries ads
        WHERE ads.workspace_id = $1
          AND ads.date >= ($2::date - INTERVAL '29 days')
          AND ads.date <= $2
        GROUP BY ads.date
        ORDER BY ads.date ASC
        "#,
        )
        .bind::<diesel::sql_types::Uuid, _>(workspace_id)
        .bind::<diesel::sql_types::Date, _>(today)
        .load(&mut *conn)
        .await
        .unwrap_or_else(|e| {
            tracing::warn!(error = %e, "overtime trend query failed, degrading gracefully");
            vec![]
        });

        let trend = rows
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
    async fn fetch_workspace_stats(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
    ) -> Result<WorkspaceStats, AppError> {
        let workspace_id = auth_token.workspace_id();
        let mut conn = self
            .diesel_pools
            .tenant_transaction_conn(auth_token)
            .await?;

        #[derive(QueryableByName)]
        struct CountRow {
            #[diesel(sql_type = diesel::sql_types::BigInt)]
            count: i64,
        }

        let total_employees: i64 = diesel::sql_query(
            "SELECT COUNT(*)::bigint as count FROM employees WHERE workspace_id = $1 AND is_active = TRUE",
        )
        .bind::<diesel::sql_types::Uuid, _>(workspace_id)
        .get_result::<CountRow>(&mut *conn)
        .await
        .map(|r| r.count)
        .unwrap_or(0);

        let tz_str = self.get_timezone(auth_token).await;
        let today = Utc::now().date_naive();
        let start_of_day = today
            .and_hms_opt(0, 0, 0)
            .and_then(|t| t.and_local_timezone(Utc).single())
            .ok_or_else(|| AppError::Internal("Invalid date calculation".to_string()))?;

        let present_today: i64 = diesel::sql_query(
            "SELECT COUNT(DISTINCT employee_id)::bigint as count FROM time_entries
         WHERE (punch_in_time AT TIME ZONE $3)::date = $1::date
         AND employee_id IN (SELECT id FROM employees WHERE workspace_id = $2)",
        )
        .bind::<diesel::sql_types::Date, _>(today)
        .bind::<diesel::sql_types::Uuid, _>(workspace_id)
        .bind::<diesel::sql_types::Text, _>(&tz_str)
        .get_result::<CountRow>(&mut *conn)
        .await
        .map(|r| r.count)
        .unwrap_or(0);

        let pending_leave: i64 = diesel::sql_query(
            "SELECT COUNT(*)::bigint as count FROM leave_requests
         WHERE status = 'pending'
         AND employee_id IN (SELECT id FROM employees WHERE workspace_id = $1)",
        )
        .bind::<diesel::sql_types::Uuid, _>(workspace_id)
        .get_result::<CountRow>(&mut *conn)
        .await
        .map(|r| r.count)
        .unwrap_or(0);

        #[derive(QueryableByName)]
        struct DecimalRow {
            #[diesel(sql_type = diesel::sql_types::Numeric)]
            amount: rust_decimal::Decimal,
        }

        let payroll: rust_decimal::Decimal = diesel::sql_query(
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
        SELECT COALESCE(SUM(total_amount), 0)::numeric as amount FROM dynamic_payroll
        "#,
        )
        .bind::<diesel::sql_types::Uuid, _>(workspace_id)
        .get_result::<DecimalRow>(&mut *conn)
        .await
        .map(|r| r.amount)
        .unwrap_or(rust_decimal::Decimal::ZERO);

        let on_leave_today: i64 = diesel::sql_query(
        "SELECT COUNT(DISTINCT lr.employee_id)::bigint as count FROM leave_requests lr
         WHERE lr.status IN ('approved', 'pre_approved')
         AND lr.start_date <= $1 AND lr.end_date >= $1
         AND lr.employee_id IN (SELECT id FROM employees WHERE workspace_id = $2 AND is_active = TRUE)",
    )
    .bind::<diesel::sql_types::Date, _>(today)
    .bind::<diesel::sql_types::Uuid, _>(workspace_id)
    .get_result::<CountRow>(&mut *conn)
    .await
    .map(|r| r.count)
    .unwrap_or(0);

        let absent_today = (total_employees - present_today - on_leave_today).max(0);

        let late_today: i64 = diesel::sql_query(
        r#"
        WITH today_punches AS (
            SELECT te.employee_id,
                   MIN(te.punch_in_time AT TIME ZONE $4)::time AS first_punch
            FROM time_entries te
            WHERE (te.punch_in_time AT TIME ZONE $4)::date = $3
              AND te.employee_id IN (SELECT id FROM employees WHERE workspace_id = $2 AND is_active = TRUE)
            GROUP BY te.employee_id
        )
        SELECT COUNT(*)::bigint as count
        FROM today_punches tp
        JOIN resolve_employee_shift($2, $3) ar ON ar.employee_id = tp.employee_id
        WHERE tp.first_punch > ar.start_time
        "#,
    )
    .bind::<diesel::sql_types::Timestamptz, _>(start_of_day)
    .bind::<diesel::sql_types::Uuid, _>(workspace_id)
    .bind::<diesel::sql_types::Date, _>(today)
    .bind::<diesel::sql_types::Text, _>(&tz_str)
    .get_result::<CountRow>(&mut *conn)
    .await
    .map(|r| r.count)
    .unwrap_or(0);

        let early_departure_today: i64 = diesel::sql_query(
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
        SELECT COUNT(*)::bigint as count
        FROM today_completed tc
        JOIN resolve_employee_shift($2, $3) ar ON ar.employee_id = tc.employee_id
        WHERE tc.last_punch_out < ar.end_time
        "#,
    )
    .bind::<diesel::sql_types::Timestamptz, _>(start_of_day)
    .bind::<diesel::sql_types::Uuid, _>(workspace_id)
    .bind::<diesel::sql_types::Date, _>(today)
    .bind::<diesel::sql_types::Text, _>(&tz_str)
    .get_result::<CountRow>(&mut *conn)
    .await
    .map(|r| r.count)
    .unwrap_or(0);

        #[derive(QueryableByName)]
        struct FloatRow {
            #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Double>)]
            amount: Option<f64>,
        }

        let avg_work_minutes: Option<f64> = diesel::sql_query(
        "SELECT AVG(duration_minutes)::float8 as amount FROM time_entries
         WHERE punch_in_time >= $1
         AND punch_out_time IS NOT NULL
         AND duration_minutes > 0
         AND employee_id IN (SELECT id FROM employees WHERE workspace_id = $2 AND is_active = TRUE)",
    )
    .bind::<diesel::sql_types::Timestamptz, _>(start_of_day)
    .bind::<diesel::sql_types::Uuid, _>(workspace_id)
    .get_result::<FloatRow>(&mut *conn)
    .await
    .ok()
    .and_then(|r| r.amount);

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

    async fn fetch_peak_times(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
    ) -> Result<Vec<PeakTimeSlot>, AppError> {
        let workspace_id = auth_token.workspace_id();
        let today = Utc::now().date_naive();
        let start_of_day = today
            .and_hms_opt(0, 0, 0)
            .and_then(|t| t.and_local_timezone(Utc).single())
            .ok_or_else(|| AppError::Internal("Invalid date".to_string()))?;

        let mut conn = self
            .diesel_pools
            .tenant_transaction_conn(auth_token)
            .await?;

        #[derive(QueryableByName)]
        struct SlotRow {
            #[diesel(sql_type = diesel::sql_types::Text)]
            time_slot: String,
            #[diesel(sql_type = diesel::sql_types::BigInt)]
            check_in: i64,
            #[diesel(sql_type = diesel::sql_types::BigInt)]
            check_out: i64,
        }

        let rows: Vec<SlotRow> = diesel::sql_query(
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
        SELECT sl.time_slot::text,
               COALESCE(SUM(CASE WHEN te.punch_in_time::time >= sl.slot_start
                                  AND te.punch_in_time::time < sl.slot_end
                                 THEN 1 ELSE 0 END), 0)::bigint AS check_in,
               COALESCE(SUM(CASE WHEN te.punch_out_time IS NOT NULL
                                  AND te.punch_out_time::time >= sl.slot_start
                                  AND te.punch_out_time::time < sl.slot_end
                                 THEN 1 ELSE 0 END), 0)::bigint AS check_out
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
    .bind::<diesel::sql_types::Timestamptz, _>(start_of_day)
    .bind::<diesel::sql_types::Uuid, _>(workspace_id)
    .load(&mut *conn)
    .await
    .unwrap_or_else(|e| { tracing::warn!(error = %e, "peak times query failed, degrading gracefully"); vec![] });

        let result = rows
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
        auth_token: &crate::diesel_pool::TenantAuthToken,
    ) -> Result<Vec<LeaveTypeBreakdown>, AppError> {
        let workspace_id = auth_token.workspace_id();
        let today = Utc::now().date_naive();
        let month_start = NaiveDate::from_ymd_opt(today.year(), today.month(), 1)
            .ok_or_else(|| AppError::Internal("Invalid date".to_string()))?;

        let mut conn = self
            .diesel_pools
            .tenant_transaction_conn(auth_token)
            .await?;

        #[derive(QueryableByName)]
        struct LtRow {
            #[diesel(sql_type = diesel::sql_types::Text)]
            leave_type: String,
            #[diesel(sql_type = diesel::sql_types::BigInt)]
            count: i64,
        }

        let rows: Vec<LtRow> = diesel::sql_query(
            r#"
        SELECT COALESCE(lt.name, 'Other')::text AS leave_type,
               COUNT(*)::bigint AS count
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
        .bind::<diesel::sql_types::Date, _>(month_start)
        .bind::<diesel::sql_types::Date, _>(today)
        .bind::<diesel::sql_types::Uuid, _>(workspace_id)
        .load(&mut *conn)
        .await
        .unwrap_or_else(|e| {
            tracing::warn!(error = %e, "leave types query failed, degrading gracefully");
            vec![]
        });

        let result = rows
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
        auth_token: &crate::diesel_pool::TenantAuthToken,
    ) -> Result<Vec<LatenessByDay>, AppError> {
        let workspace_id = auth_token.workspace_id();
        let today = Utc::now().date_naive();
        let _start_date = today - chrono::Duration::days(29);
        let tz_str = self.get_timezone(auth_token).await;

        let mut conn = self
            .diesel_pools
            .tenant_transaction_conn(auth_token)
            .await?;

        #[derive(QueryableByName)]
        struct DayRow {
            #[diesel(sql_type = diesel::sql_types::Integer)]
            dow: i32,
            #[diesel(sql_type = diesel::sql_types::BigInt)]
            late_count: i64,
            #[diesel(sql_type = diesel::sql_types::Double)]
            avg_late_minutes: f64,
        }

        let rows: Vec<DayRow> = diesel::sql_query(
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
               COUNT(*)::bigint AS late_count,
               COALESCE(AVG(late_minutes), 0)::float8 AS avg_late_minutes
        FROM late_entries
        GROUP BY dow
        ORDER BY dow
        "#,
    )
    .bind::<diesel::sql_types::Date, _>(today)
    .bind::<diesel::sql_types::Uuid, _>(workspace_id)
    .bind::<diesel::sql_types::Text, _>(&tz_str)
    .load(&mut *conn)
    .await
    .unwrap_or_else(|e| { tracing::warn!(error = %e, "lateness by day query failed, degrading gracefully"); vec![] });

        let day_names = ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"];

        let result = rows
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
