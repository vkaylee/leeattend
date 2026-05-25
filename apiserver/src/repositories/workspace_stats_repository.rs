use crate::diesel_pool::DieselPools;
use crate::errors::AppError;
use crate::models::{
    AttendanceTrendPoint, DepartmentAttendancePoint, LatenessByDay, LeaveTypeBreakdown,
    OvertimeTrendPoint, PeakTimeSlot, WorkspaceStats,
};
use async_trait::async_trait;
use chrono::{Datelike, NaiveDate, Utc};
use diesel_async::RunQueryDsl;

use crate::repositories::raw_models::workspace_stats::*;
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

        let base_rows: Vec<BaseRow> = diesel::sql_query(include_str!(
            "sql/workspace_stats/attendance_trend_base.sql"
        ))
        .bind::<diesel::sql_types::Date, _>(today)
        .bind::<diesel::sql_types::Uuid, _>(workspace_id)
        .bind::<diesel::sql_types::Text, _>(&tz_str)
        .load(&mut *conn)
        .await?;

        let late_rows: Vec<LateRow> = diesel::sql_query(include_str!(
            "sql/workspace_stats/attendance_trend_late.sql"
        ))
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

        let departments: Vec<DeptRow> = diesel::sql_query(include_str!(
            "sql/workspace_stats/department_attendance_departments.sql"
        ))
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

        let present_rows: Vec<PresentRow> = diesel::sql_query(include_str!(
            "sql/workspace_stats/department_attendance_present.sql"
        ))
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

            let late_rows: Vec<LateDepartmentRow> = diesel::sql_query(include_str!(
                "sql/workspace_stats/department_attendance_late.sql"
            ))
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

        let leave_rows: Vec<LeaveRow> = diesel::sql_query(include_str!(
            "sql/workspace_stats/department_attendance_leave.sql"
        ))
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

        let rows: Vec<OtRow> =
            diesel::sql_query(include_str!("sql/workspace_stats/overtime_trend.sql"))
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

        let payroll: rust_decimal::Decimal = diesel::sql_query(include_str!(
            "sql/workspace_stats/workspace_stats_payroll.sql"
        ))
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

        let late_today: i64 = diesel::sql_query(include_str!(
            "sql/workspace_stats/workspace_stats_late_today.sql"
        ))
        .bind::<diesel::sql_types::Timestamptz, _>(start_of_day)
        .bind::<diesel::sql_types::Uuid, _>(workspace_id)
        .bind::<diesel::sql_types::Date, _>(today)
        .bind::<diesel::sql_types::Text, _>(&tz_str)
        .get_result::<CountRow>(&mut *conn)
        .await
        .map(|r| r.count)
        .unwrap_or(0);

        let early_departure_today: i64 = diesel::sql_query(include_str!(
            "sql/workspace_stats/workspace_stats_early_departure.sql"
        ))
        .bind::<diesel::sql_types::Timestamptz, _>(start_of_day)
        .bind::<diesel::sql_types::Uuid, _>(workspace_id)
        .bind::<diesel::sql_types::Date, _>(today)
        .bind::<diesel::sql_types::Text, _>(&tz_str)
        .get_result::<CountRow>(&mut *conn)
        .await
        .map(|r| r.count)
        .unwrap_or(0);

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

        let rows: Vec<SlotRow> =
            diesel::sql_query(include_str!("sql/workspace_stats/peak_times.sql"))
                .bind::<diesel::sql_types::Timestamptz, _>(start_of_day)
                .bind::<diesel::sql_types::Uuid, _>(workspace_id)
                .load(&mut *conn)
                .await
                .unwrap_or_else(|e| {
                    tracing::warn!(error = %e, "peak times query failed, degrading gracefully");
                    vec![]
                });

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

        let rows: Vec<LtRow> = diesel::sql_query(include_str!(
            "sql/workspace_stats/leave_types_breakdown.sql"
        ))
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

        let rows: Vec<DayRow> = diesel::sql_query(include_str!(
            "sql/workspace_stats/lateness_by_day.sql"
        ))
        .bind::<diesel::sql_types::Date, _>(today)
        .bind::<diesel::sql_types::Uuid, _>(workspace_id)
        .bind::<diesel::sql_types::Text, _>(&tz_str)
        .load(&mut *conn)
        .await
        .unwrap_or_else(|e| {
            tracing::warn!(error = %e, "lateness by day query failed, degrading gracefully");
            vec![]
        });

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
