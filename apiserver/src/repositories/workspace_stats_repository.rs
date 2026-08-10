use crate::diesel_pool::DieselPools;
use crate::errors::AppError;
use crate::models::{
    AttendanceTrendPoint, DepartmentAttendancePoint, LatenessByDay, LeaveTypeBreakdown,
    OvertimeTrendPoint, PeakTimeSlot, WorkspaceStats,
};
use async_trait::async_trait;
use chrono::{Datelike, NaiveDate, Utc};
use chrono_tz::Tz;
use diesel_async::RunQueryDsl;

use crate::repositories::raw_models::workspace_stats::*;
use crate::repositories::roster::RosterRepository;
use crate::utils::tz;
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
    diesel_pools: DieselPools,
}

impl DieselWorkspaceStatsRepository {
    pub fn new(diesel_pools: DieselPools) -> Self {
        Self { diesel_pools }
    }

    /// Return the current calendar date in the workspace's local timezone.
    ///
    /// Replaces the incorrect `Utc::now().date_naive()` which gives
    /// the UTC calendar date, not the workspace-local "today".
    fn workspace_local_date(tz: Tz) -> NaiveDate {
        Utc::now().with_timezone(&tz).date_naive()
    }

    async fn get_timezone(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
    ) -> Result<Tz, AppError> {
        let workspace_id = auth_token.workspace_id();
        let mut conn = self
            .diesel_pools
            .api
            .get()
            .await
            .map_err(|e| AppError::Internal(format!("dashboard timezone pool: {e}")))?;

        let row = diesel::sql_query("SELECT timezone::text FROM workspaces WHERE id = $1")
            .bind::<diesel::sql_types::Uuid, _>(workspace_id)
            .get_result::<TzRow>(&mut *conn)
            .await
            .map_err(|e| {
                tracing::error!(
                    workspace_id = ?workspace_id,
                    operation = "dashboard_timezone",
                    error = %e,
                    "workspace timezone query failed"
                );
                AppError::from(e)
            })?;

        tz::validate_tz(&row.timezone)
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
        let tz = self.get_timezone(auth_token).await?;
        let tz_name = tz.name();
        let today = Self::workspace_local_date(tz);
        let start_date = today - chrono::Duration::days(29);

        let utc_range_start = tz::start_of_day_utc(&tz, start_date)
            .map_err(|e| AppError::Internal(format!("Invalid start date: {e}")))?;
        let utc_range_end_exclusive = tz::start_of_day_utc(
            &tz,
            today
                .succ_opt()
                .ok_or_else(|| AppError::Internal("Date overflow".to_string()))?,
        )
        .map_err(|e| AppError::Internal(format!("Invalid end date: {e}")))?;

        let mut conn = self
            .diesel_pools
            .tenant_transaction_conn(auth_token)
            .await?;

        let base_rows: Vec<BaseRow> = diesel::sql_query(include_str!(
            "sql/workspace_stats/attendance_trend_base.sql"
        ))
        .bind::<diesel::sql_types::Date, _>(today)
        .bind::<diesel::sql_types::Uuid, _>(workspace_id)
        .bind::<diesel::sql_types::Timestamptz, _>(utc_range_start)
        .bind::<diesel::sql_types::Timestamptz, _>(utc_range_end_exclusive)
        .bind::<diesel::sql_types::Text, _>(tz_name)
        .load(&mut *conn)
        .await?;

        // Release the tenant connection while the roster repository acquires
        // its own connection.
        drop(conn);

        // Resolve workspace schedules for the range in Rust
        let roster_repo =
            crate::repositories::diesel_roster_repository::DieselRosterRepository::new(
                self.diesel_pools.clone(),
            );
        let (schedules, _) = roster_repo
            .get_workspace_schedules(None, auth_token, start_date, today, None, None, None)
            .await
            .map_err(|e| {
                tracing::error!(workspace_id = ?workspace_id, operation = "dashboard_roster", error = %e, "dashboard roster query failed");
                e
            })?;

        let mut conn = self
            .diesel_pools
            .tenant_transaction_conn(auth_token)
            .await?;

        // Fetch first punches in range
        let punch_rows: Vec<EmployeeFirstPunchRow> =
            diesel::sql_query(include_str!("sql/workspace_stats/first_punches_range.sql"))
                .bind::<diesel::sql_types::Timestamptz, _>(utc_range_start)
                .bind::<diesel::sql_types::Timestamptz, _>(utc_range_end_exclusive)
                .bind::<diesel::sql_types::Text, _>(tz_name)
                .bind::<diesel::sql_types::Uuid, _>(workspace_id)
                .load(&mut *conn)
                .await
                .map_err(|e| {
                    tracing::error!(workspace_id = ?workspace_id, operation = "dashboard_punches", error = %e, "dashboard query failed");
                    AppError::from(e)
                })?;

        let mut punch_map = std::collections::HashMap::new();
        for row in punch_rows {
            punch_map.insert((row.employment_id, row.work_date), row.first_punch);
        }

        // Count each late employment once per local work date.
        let mut late_employments_by_date: std::collections::HashMap<
            NaiveDate,
            std::collections::HashSet<Uuid>,
        > = std::collections::HashMap::new();
        for schedule in &schedules {
            for s in schedule.schedules() {
                if let Some(shift_start) = s.start_time() {
                    if let Some(&first_punch) = punch_map.get(&(schedule.employment_id(), s.date()))
                    {
                        if first_punch > shift_start {
                            late_employments_by_date
                                .entry(s.date())
                                .or_default()
                                .insert(schedule.employment_id());
                        }
                    }
                }
            }
        }

        let trend = base_rows
            .into_iter()
            .map(|r| {
                let late = late_employments_by_date
                    .get(&r.date)
                    .map_or(0_i64, |employments| employments.len() as i64);
                let absent = (r.active_employees - r.present - r.on_leave).max(0);
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
        let tz = self.get_timezone(auth_token).await?;
        let tz_name = tz.name();
        let today = Self::workspace_local_date(tz);
        let start_date = match range {
            "today" => today,
            "week" => today - chrono::Duration::days(6),
            "month" => today - chrono::Duration::days(29),
            _ => {
                return Err(AppError::validation(
                    crate::utils::i18n::I18nKey::Validation,
                ))
            }
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

        let utc_range_start = tz::start_of_day_utc(&tz, start_date)
            .map_err(|e| AppError::Internal(format!("Invalid date: {e}")))?;
        let utc_range_end_exclusive = tz::start_of_day_utc(
            &tz,
            today
                .succ_opt()
                .ok_or_else(|| AppError::Internal("Date overflow".to_string()))?,
        )
        .map_err(|e| AppError::Internal(format!("Invalid date: {e}")))?;

        let present_rows: Vec<PresentRow> = diesel::sql_query(include_str!(
            "sql/workspace_stats/department_attendance_present.sql"
        ))
        .bind::<diesel::sql_types::Uuid, _>(workspace_id)
        .bind::<diesel::sql_types::Timestamptz, _>(utc_range_start)
        .bind::<diesel::sql_types::Date, _>(today)
        .bind::<diesel::sql_types::Timestamptz, _>(utc_range_end_exclusive)
        .load(&mut *conn)
        .await?;

        let present_map: std::collections::HashMap<Uuid, i64> = present_rows
            .into_iter()
            .map(|r| (r.department_id, r.present_count))
            .collect();

        // Resolve late punches across the selected range. The previous implementation
        // only populated this map for `today`, which made the week view silently report
        // zero late employees even when late punches existed earlier in the week.
        let punch_rows: Vec<EmployeeFirstPunchRow> =
            diesel::sql_query(include_str!("sql/workspace_stats/first_punches_range.sql"))
                .bind::<diesel::sql_types::Timestamptz, _>(utc_range_start)
                .bind::<diesel::sql_types::Timestamptz, _>(utc_range_end_exclusive)
                .bind::<diesel::sql_types::Text, _>(tz_name)
                .bind::<diesel::sql_types::Uuid, _>(workspace_id)
                .load(&mut *conn)
                .await?;

        let dept_rows: Vec<EmployeeDepartmentRow> = diesel::sql_query(include_str!(
            "sql/workspace_stats/employee_departments_today.sql"
        ))
        .bind::<diesel::sql_types::Uuid, _>(workspace_id)
        .bind::<diesel::sql_types::Date, _>(today)
        .load(&mut *conn)
        .await?;

        let dept_map: std::collections::HashMap<Uuid, Uuid> = dept_rows
            .into_iter()
            .map(|r| (r.employment_id, r.department_id))
            .collect();

        let roster_repo =
            crate::repositories::diesel_roster_repository::DieselRosterRepository::new(
                self.diesel_pools.clone(),
            );
        let (schedules, _) = roster_repo
            .get_workspace_schedules(None, auth_token, start_date, today, None, None, None)
            .await?;

        let mut punch_map = std::collections::HashMap::new();
        for row in punch_rows {
            punch_map.insert((row.employment_id, row.work_date), row.first_punch);
        }

        let mut late_employments_by_department: std::collections::HashMap<
            Uuid,
            std::collections::HashSet<Uuid>,
        > = std::collections::HashMap::new();
        for schedule in schedules {
            let Some(dept_id) = dept_map.get(&schedule.employment_id()) else {
                continue;
            };

            for daily_schedule in schedule.schedules() {
                let Some(shift_start) = daily_schedule.start_time() else {
                    continue;
                };
                let Some(&first_punch) =
                    punch_map.get(&(schedule.employment_id(), daily_schedule.date()))
                else {
                    continue;
                };

                if first_punch > shift_start {
                    late_employments_by_department
                        .entry(*dept_id)
                        .or_default()
                        .insert(schedule.employment_id());
                }
            }
        }

        let late_map: std::collections::HashMap<Uuid, i64> = late_employments_by_department
            .into_iter()
            .map(|(department_id, employments)| (department_id, employments.len() as i64))
            .collect();

        let leave_rows: Vec<LeaveRow> = diesel::sql_query(include_str!(
            "sql/workspace_stats/department_attendance_leave.sql"
        ))
        .bind::<diesel::sql_types::Uuid, _>(workspace_id)
        .bind::<diesel::sql_types::Date, _>(start_date)
        .bind::<diesel::sql_types::Date, _>(today)
        .load(&mut *conn)
        .await?;

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
        let tz = self.get_timezone(auth_token).await?;
        let today = Self::workspace_local_date(tz);
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
                .map_err(|e| {
                    tracing::error!(workspace_id = ?workspace_id, operation = "overtime_trend", error = %e, "dashboard query failed");
                    AppError::from(e)
                })?;

        let trend = rows
            .into_iter()
            .map(|r| {
                #[allow(clippy::cast_precision_loss)]
                let system_ot_hours = r.system_ot_minutes as f64 / 60.0;
                #[allow(clippy::cast_precision_loss)]
                let approved_ot_hours = r.approved_ot_minutes as f64 / 60.0;
                OvertimeTrendPoint {
                    date: r.date,
                    system_ot_hours,
                    approved_ot_hours,
                }
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
        let tz = self.get_timezone(auth_token).await?;
        let tz_name = tz.name();
        let today = Self::workspace_local_date(tz);

        let utc_today_start = tz::start_of_day_utc(&tz, today)
            .map_err(|e| AppError::Internal(format!("Invalid date: {e}")))?;
        let utc_today_end_exclusive = tz::start_of_day_utc(
            &tz,
            today
                .succ_opt()
                .ok_or_else(|| AppError::Internal("Date overflow".to_string()))?,
        )
        .map_err(|e| AppError::Internal(format!("Invalid date: {e}")))?;

        // ── Phase 1: Run 8 independent read queries in parallel ─────
        // Each future acquires its own connection from the tenant pool
        // and handles query errors gracefully (matching original behavior).

        let pools = self.diesel_pools.clone();

        let f_total_employees = {
            let pools = pools.clone();
            async move {
                let mut conn = pools.tenant_transaction_conn(auth_token).await?;
                diesel::sql_query(
                    "SELECT COUNT(DISTINCT employee_id)::bigint as count FROM employments
                     WHERE workspace_id = $1
                     AND status = 'active'
                     AND deleted_at IS NULL
                     AND start_date <= $2
                     AND (end_date IS NULL OR end_date >= $2)",
                )
                .bind::<diesel::sql_types::Uuid, _>(workspace_id)
                .bind::<diesel::sql_types::Date, _>(today)
                .get_result::<CountRow>(&mut *conn)
                .await
                .map(|r| r.count)
                .map_err(|e| {
                    tracing::error!(workspace_id = ?workspace_id, operation = "workspace_stats_count", error = %e, "dashboard query failed");
                    AppError::from(e)
                })
            }
        };

        let f_present_today = {
            let pools = pools.clone();
            async move {
                let mut conn = pools.tenant_transaction_conn(auth_token).await?;
                diesel::sql_query(
                    "SELECT COUNT(DISTINCT emp.employee_id)::bigint as count FROM time_entries te
                     JOIN employments emp ON emp.id = te.employment_id
                     JOIN employees e ON e.id = emp.employee_id AND e.workspace_id = $3 AND e.deleted_at IS NULL
                     WHERE te.punch_in_time >= $1
                     AND te.punch_in_time < $2
                     AND emp.workspace_id = $3
                     AND emp.status = 'active'
                     AND emp.deleted_at IS NULL
                     AND emp.start_date <= $4
                     AND (emp.end_date IS NULL OR emp.end_date >= $4)",
                )
                .bind::<diesel::sql_types::Timestamptz, _>(utc_today_start)
                .bind::<diesel::sql_types::Timestamptz, _>(utc_today_end_exclusive)
                .bind::<diesel::sql_types::Uuid, _>(workspace_id)
                .bind::<diesel::sql_types::Date, _>(today)
                .get_result::<CountRow>(&mut *conn)
                .await
                .map(|r| r.count)
                .map_err(|e| {
                    tracing::error!(workspace_id = ?workspace_id, operation = "workspace_stats_count", error = %e, "dashboard query failed");
                    AppError::from(e)
                })
            }
        };

        let f_pending_leave = {
            let pools = pools.clone();
            async move {
                let mut conn = pools.tenant_transaction_conn(auth_token).await?;
                diesel::sql_query(
                    "SELECT COUNT(*)::bigint as count FROM leave_requests lr
                     JOIN employments emp ON emp.id = lr.employment_id
                     JOIN employees e ON e.id = emp.employee_id AND e.workspace_id = $1 AND e.deleted_at IS NULL
                     WHERE lr.status = 'pending'
                     AND emp.workspace_id = $1
                     AND emp.status = 'active'
                     AND emp.deleted_at IS NULL
                     AND emp.start_date <= $2
                     AND (emp.end_date IS NULL OR emp.end_date >= $2)",
                )
                .bind::<diesel::sql_types::Uuid, _>(workspace_id)
                .bind::<diesel::sql_types::Date, _>(today)
                .get_result::<CountRow>(&mut *conn)
                .await
                .map(|r| r.count)
                .map_err(|e| {
                    tracing::error!(workspace_id = ?workspace_id, operation = "workspace_stats_count", error = %e, "dashboard query failed");
                    AppError::from(e)
                })
            }
        };

        let f_payroll = {
            let pools = pools.clone();
            async move {
                let mut conn = pools.tenant_transaction_conn(auth_token).await?;
                diesel::sql_query(include_str!(
                    "sql/workspace_stats/workspace_stats_payroll.sql"
                ))
                .bind::<diesel::sql_types::Uuid, _>(workspace_id)
                .bind::<diesel::sql_types::Date, _>(today)
                .get_result::<DecimalRow>(&mut *conn)
                .await
                .map(|r| r.amount)
                .map_err(|e| {
                    tracing::error!(workspace_id = ?workspace_id, operation = "workspace_stats_payroll", error = %e, "dashboard query failed");
                    AppError::from(e)
                })
            }
        };

        let f_on_leave_today = {
            let pools = pools.clone();
            async move {
                let mut conn = pools.tenant_transaction_conn(auth_token).await?;
                diesel::sql_query(
                    "SELECT COUNT(DISTINCT emp.employee_id)::bigint as count FROM leave_requests lr
                     JOIN employments emp ON emp.id = lr.employment_id
                     JOIN employees e ON e.id = emp.employee_id AND e.workspace_id = $2 AND e.deleted_at IS NULL
                     WHERE lr.status IN ('approved', 'pre_approved')
                     AND lr.start_date <= $1 AND lr.end_date >= $1
                     AND emp.workspace_id = $2
                     AND emp.status = 'active'
                     AND emp.deleted_at IS NULL
                     AND emp.start_date <= $1
                     AND (emp.end_date IS NULL OR emp.end_date >= $1)",
                )
                .bind::<diesel::sql_types::Date, _>(today)
                .bind::<diesel::sql_types::Uuid, _>(workspace_id)
                .get_result::<CountRow>(&mut *conn)
                .await
                .map(|r| r.count)
                .map_err(|e| {
                    tracing::error!(workspace_id = ?workspace_id, operation = "workspace_stats_count", error = %e, "dashboard query failed");
                    AppError::from(e)
                })
            }
        };

        let f_first_punches = {
            let pools = pools.clone();
            async move {
                let mut conn = pools.tenant_transaction_conn(auth_token).await?;
                diesel::sql_query(include_str!("sql/workspace_stats/first_punches_today.sql"))
                    .bind::<diesel::sql_types::Timestamptz, _>(utc_today_start)
                    .bind::<diesel::sql_types::Timestamptz, _>(utc_today_end_exclusive)
                    .bind::<diesel::sql_types::Uuid, _>(workspace_id)
                    .bind::<diesel::sql_types::Text, _>(tz_name)
                    .bind::<diesel::sql_types::Date, _>(today)
                    .load(&mut *conn)
                    .await
                    .map_err(|e| {
                        tracing::error!(workspace_id = ?workspace_id, operation = "workspace_stats_punches", error = %e, "dashboard query failed");
                        AppError::from(e)
                    })
            }
        };

        let f_last_punches = {
            let pools = pools.clone();
            async move {
                let mut conn = pools.tenant_transaction_conn(auth_token).await?;
                diesel::sql_query(include_str!("sql/workspace_stats/last_punches_today.sql"))
                    .bind::<diesel::sql_types::Timestamptz, _>(utc_today_start)
                    .bind::<diesel::sql_types::Timestamptz, _>(utc_today_end_exclusive)
                    .bind::<diesel::sql_types::Uuid, _>(workspace_id)
                    .bind::<diesel::sql_types::Text, _>(tz_name)
                    .bind::<diesel::sql_types::Date, _>(today)
                    .load(&mut *conn)
                    .await
                    .map_err(|e| {
                        tracing::error!(workspace_id = ?workspace_id, operation = "workspace_stats_punches", error = %e, "dashboard query failed");
                        AppError::from(e)
                    })
            }
        };

        let f_avg_work = {
            let pools = pools.clone();
            async move {
                let mut conn = pools.tenant_transaction_conn(auth_token).await?;
                diesel::sql_query(
                    "SELECT AVG(te.duration_minutes)::float8 as amount FROM time_entries te
                     JOIN employments emp ON emp.id = te.employment_id
                     JOIN employees e ON e.id = emp.employee_id AND e.workspace_id = $3 AND e.deleted_at IS NULL
                     WHERE te.punch_in_time >= $1
                     AND te.punch_in_time < $2
                     AND te.punch_out_time IS NOT NULL
                     AND te.duration_minutes > 0
                     AND emp.workspace_id = $3
                     AND emp.status = 'active'
                     AND emp.deleted_at IS NULL
                     AND emp.start_date <= $4
                     AND (emp.end_date IS NULL OR emp.end_date >= $4)",
                )
                .bind::<diesel::sql_types::Timestamptz, _>(utc_today_start)
                .bind::<diesel::sql_types::Timestamptz, _>(utc_today_end_exclusive)
                .bind::<diesel::sql_types::Uuid, _>(workspace_id)
                .bind::<diesel::sql_types::Date, _>(today)
                .get_result::<FloatRow>(&mut *conn)
                .await
                .map(|r| r.amount)
                .map_err(|e| {
                    tracing::error!(workspace_id = ?workspace_id, operation = "workspace_stats_average_work", error = %e, "dashboard query failed");
                    AppError::from(e)
                })
            }
        };

        // Read sequentially so a tenant pool of five connections is never
        // saturated by this endpoint's eight independent queries.
        let total_employees = f_total_employees.await?;
        let present_today = f_present_today.await?;
        let pending_leave = f_pending_leave.await?;
        let payroll = f_payroll.await?;
        let on_leave_today = f_on_leave_today.await?;
        let first_punch_rows = f_first_punches.await?;
        let last_punch_rows = f_last_punches.await?;
        let avg_work_minutes = f_avg_work.await?;

        let absent_today = (total_employees - present_today - on_leave_today).max(0);

        // ── Phase 2: Fetch schedules (separate to avoid pool pressure) ──

        let first_punch_map: std::collections::HashMap<Uuid, chrono::NaiveTime> = first_punch_rows
            .into_iter()
            .map(|r: EmployeeFirstPunchTodayRow| (r.employment_id, r.first_punch))
            .collect();

        let last_punch_map: std::collections::HashMap<Uuid, chrono::NaiveTime> = last_punch_rows
            .into_iter()
            .map(|r: EmployeeLastPunchRow| (r.employment_id, r.last_punch_out))
            .collect();

        let roster_repo =
            crate::repositories::diesel_roster_repository::DieselRosterRepository::new(
                self.diesel_pools.clone(),
            );
        let (schedules, _) = roster_repo
            .get_workspace_schedules(None, auth_token, today, today, None, None, None)
            .await
            .map_err(|e| {
                tracing::error!(workspace_id = ?workspace_id, operation = "workspace_stats_roster", error = %e, "dashboard roster query failed");
                e
            })?;

        let mut late_today = 0;
        let mut early_departure_today = 0;

        for schedule in schedules {
            if !schedule.schedules().is_empty() {
                let s = &schedule.schedules()[0];
                if let Some(shift_start) = s.start_time() {
                    if let Some(&first_punch) = first_punch_map.get(&schedule.employment_id()) {
                        if first_punch > shift_start {
                            late_today += 1;
                        }
                    }
                }
                if let Some(shift_end) = s.end_time() {
                    if let Some(&last_punch_out) = last_punch_map.get(&schedule.employment_id()) {
                        if last_punch_out < shift_end {
                            early_departure_today += 1;
                        }
                    }
                }
            }
        }

        let avg_work_hours_today = avg_work_minutes.map_or(0.0, |minutes| minutes / 60.0);

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
        let tz = self.get_timezone(auth_token).await?;
        let today = Self::workspace_local_date(tz);

        let utc_start = tz::start_of_day_utc(&tz, today)
            .map_err(|e| AppError::Internal(format!("Invalid date: {e}")))?;
        let utc_end_exclusive = tz::start_of_day_utc(
            &tz,
            today
                .succ_opt()
                .ok_or_else(|| AppError::Internal("Date overflow".to_string()))?,
        )
        .map_err(|e| AppError::Internal(format!("Invalid date: {e}")))?;

        let mut conn = self
            .diesel_pools
            .tenant_transaction_conn(auth_token)
            .await?;

        let rows: Vec<SlotRow> =
            diesel::sql_query(include_str!("sql/workspace_stats/peak_times.sql"))
                .bind::<diesel::sql_types::Timestamptz, _>(utc_start)
                .bind::<diesel::sql_types::Timestamptz, _>(utc_end_exclusive)
                .bind::<diesel::sql_types::Uuid, _>(workspace_id)
                .bind::<diesel::sql_types::Text, _>(tz.name())
                .bind::<diesel::sql_types::Date, _>(today)
                .load(&mut *conn)
                .await?;

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
        let tz = self.get_timezone(auth_token).await?;
        let today = Self::workspace_local_date(tz);
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
        .bind::<diesel::sql_types::Date, _>(today)
        .load(&mut *conn)
        .await
        .map_err(|e| {
            tracing::error!(workspace_id = ?workspace_id, operation = "leave_types_breakdown", error = %e, "dashboard query failed");
            AppError::from(e)
        })?;

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
        let tz = self.get_timezone(auth_token).await?;
        let tz_name = tz.name();
        let today = Self::workspace_local_date(tz);
        let start_date = today - chrono::Duration::days(29);

        let utc_start = tz::start_of_day_utc(&tz, start_date)
            .map_err(|e| AppError::Internal(format!("Invalid start date: {e}")))?;
        let utc_end_exclusive = tz::start_of_day_utc(
            &tz,
            today
                .succ_opt()
                .ok_or_else(|| AppError::Internal("Date overflow".to_string()))?,
        )
        .map_err(|e| AppError::Internal(format!("Invalid end date: {e}")))?;

        let conn = self
            .diesel_pools
            .tenant_transaction_conn(auth_token)
            .await?;

        // Release the tenant connection while the roster repository acquires
        // its own connection.
        drop(conn);

        // Resolve workspace schedules for the range in Rust
        let roster_repo =
            crate::repositories::diesel_roster_repository::DieselRosterRepository::new(
                self.diesel_pools.clone(),
            );
        let (schedules, _) = roster_repo
            .get_workspace_schedules(None, auth_token, start_date, today, None, None, None)
            .await
            .map_err(|e| {
                tracing::error!(workspace_id = ?workspace_id, operation = "dashboard_roster", error = %e, "dashboard roster query failed");
                e
            })?;

        let mut conn = self
            .diesel_pools
            .tenant_transaction_conn(auth_token)
            .await?;

        // Fetch first punches in range
        let punch_rows: Vec<EmployeeFirstPunchRow> =
            diesel::sql_query(include_str!("sql/workspace_stats/first_punches_range.sql"))
                .bind::<diesel::sql_types::Timestamptz, _>(utc_start)
                .bind::<diesel::sql_types::Timestamptz, _>(utc_end_exclusive)
                .bind::<diesel::sql_types::Text, _>(tz_name)
                .bind::<diesel::sql_types::Uuid, _>(workspace_id)
                .load(&mut *conn)
                .await
                .map_err(|e| {
                    tracing::error!(workspace_id = ?workspace_id, operation = "dashboard_punches", error = %e, "dashboard query failed");
                    AppError::from(e)
                })?;

        let mut punch_map = std::collections::HashMap::new();
        for row in punch_rows {
            punch_map.insert((row.employment_id, row.work_date), row.first_punch);
        }

        let mut dow_stats: std::collections::HashMap<i32, (i64, f64)> =
            std::collections::HashMap::new();
        for schedule in &schedules {
            for s in schedule.schedules() {
                if let Some(shift_start) = s.start_time() {
                    if let Some(&first_punch) = punch_map.get(&(schedule.employment_id(), s.date()))
                    {
                        if first_punch > shift_start {
                            #[allow(clippy::cast_precision_loss)]
                            let late_mins = (first_punch - shift_start).num_seconds() as f64 / 60.0;
                            let dow = i32::try_from(s.date().weekday().number_from_monday())
                                .map_err(|_| AppError::Internal("Invalid weekday".to_string()))?;
                            let entry = dow_stats.entry(dow).or_insert((0, 0.0));
                            entry.0 += 1;
                            entry.1 += late_mins;
                        }
                    }
                }
            }
        }

        let day_names = ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"];
        let mut result = Vec::new();
        for dow in 1..=7 {
            if let Some(&(late_count, total_late_minutes)) = dow_stats.get(&dow) {
                if late_count > 0 {
                    #[allow(clippy::cast_precision_loss)]
                    let avg_late_minutes = total_late_minutes / (late_count as f64);
                    let day_idx = usize::try_from(dow - 1)
                        .map_err(|_| AppError::Internal("Invalid weekday index".to_string()))?;
                    result.push(LatenessByDay {
                        day: day_names[day_idx].to_string(),
                        late_count,
                        avg_late_minutes: (avg_late_minutes * 10.0).round() / 10.0,
                    });
                }
            }
        }

        Ok(result)
    }
}
