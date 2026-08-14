use crate::diesel_pool::DieselPools;
use crate::errors::AppError;
use crate::models::{
    AttendanceTrendPoint, DepartmentAttendancePoint, LatenessByDay, LeaveTypeBreakdown,
    OvertimeTrendPoint, PeakTimeSlot, RateQuantity, WorkspaceStats,
};
use async_trait::async_trait;
use chrono::{Datelike, NaiveDate, Utc};
use chrono_tz::Tz;
use diesel::QueryableByName;
use diesel_async::RunQueryDsl;
use rust_decimal::Decimal;

use crate::repositories::raw_models::workspace_stats::*;
use crate::repositories::roster::RosterRepository;
use crate::utils::tz;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct ExecutiveDashboardRequest {
    pub current_start: NaiveDate,
    pub current_end: NaiveDate,
    pub comparison_start: NaiveDate,
    pub comparison_end: NaiveDate,
    pub department_ids: Option<Vec<Uuid>>,
    pub location_ids: Option<Vec<Uuid>>,
    pub timezone: String,
    /// Whether overtime cost may be resolved (caller has `PayrollSensitiveRead`).
    pub allow_overtime_cost: bool,
}

#[derive(Debug, Clone, Default)]
pub struct ChartAnalyticsQuery {
    /// Inclusive start date in workspace-local time. `None` falls back to a
    /// default window (e.g. last 30 days) resolved from the workspace timezone.
    pub start_date: Option<NaiveDate>,
    /// Inclusive end date in workspace-local time. `None` falls back to today.
    pub end_date: Option<NaiveDate>,
    /// Workspace timezone. `None` resolves the workspace's configured timezone.
    pub timezone: Option<String>,
    /// Scoped-manager department filter. `None` = global scope.
    pub department_ids: Option<Vec<Uuid>>,
    /// Optional Top-N cap applied after rate-sorting.
    pub top_n: Option<i64>,
}

#[derive(Debug, Clone, Default)]
pub struct ExecutiveDashboardSnapshot {
    pub headcount: i64,
    pub eligible_employee_days: i64,
    pub present_employee_days: i64,
    pub absent_employee_days: i64,
    pub leave_employee_days: i64,
    pub termination_count: i64,
    pub headcount_start: i64,
    pub headcount_end: i64,
    pub approved_overtime_minutes: i64,
    pub pending_hr_actions: i64,
    pub currency_code: Option<String>,
    /// Sum of `overtime_pay_amount` for finalized payroll in the period.
    /// `None` when cost is unauthorized or the payroll source is unavailable.
    pub overtime_cost: Option<Decimal>,
}

#[derive(Debug, Clone)]
pub struct ExecutiveDashboardSnapshots {
    pub current: ExecutiveDashboardSnapshot,
    pub comparison: ExecutiveDashboardSnapshot,
}

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait WorkspaceStatsRepository: Send + Sync {
    async fn fetch_attendance_trend(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        query: &ChartAnalyticsQuery,
    ) -> Result<Vec<AttendanceTrendPoint>, AppError>;
    async fn fetch_department_attendance(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        query: &ChartAnalyticsQuery,
    ) -> Result<Vec<DepartmentAttendancePoint>, AppError>;
    async fn fetch_overtime_trend(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
    ) -> Result<Vec<OvertimeTrendPoint>, AppError>;
    async fn fetch_workspace_stats(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        query: &ChartAnalyticsQuery,
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
        query: &ChartAnalyticsQuery,
    ) -> Result<Vec<LatenessByDay>, AppError>;
    async fn fetch_executive_dashboard(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        request: &ExecutiveDashboardRequest,
    ) -> Result<ExecutiveDashboardSnapshots, AppError>;
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

    /// Resolve the effective timezone for a chart query: an explicit client
    /// timezone is validated strictly (no silent fallback); otherwise the
    /// workspace's configured timezone is used.
    async fn resolve_timezone(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        query: &ChartAnalyticsQuery,
    ) -> Result<Tz, AppError> {
        match query.timezone.as_deref() {
            Some(explicit) => tz::validate_tz(explicit),
            None => self.get_timezone(auth_token).await,
        }
    }

    /// Resolve the inclusive `(start, end)` window for a chart query, falling
    /// back to `(today - default_days, today)` when either bound is omitted.
    fn resolve_range(
        query: &ChartAnalyticsQuery,
        today: NaiveDate,
        default_days: i64,
    ) -> (NaiveDate, NaiveDate) {
        let end = query.end_date.unwrap_or(today).min(today);
        let start = query
            .start_date
            .unwrap_or_else(|| end - chrono::Duration::days(default_days));
        (start.min(end), end)
    }
}

#[async_trait]
impl WorkspaceStatsRepository for DieselWorkspaceStatsRepository {
    #[allow(clippy::too_many_lines)]
    async fn fetch_attendance_trend(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        query: &ChartAnalyticsQuery,
    ) -> Result<Vec<AttendanceTrendPoint>, AppError> {
        let workspace_id = auth_token.workspace_id();
        let tz = self.resolve_timezone(auth_token, query).await?;
        let tz_name = tz.name();
        let today = Self::workspace_local_date(tz);
        let (start_date, end_date) = Self::resolve_range(query, today, 29);
        let department_ids = query.department_ids.as_deref();

        let utc_range_start = tz::start_of_day_utc(&tz, start_date)
            .map_err(|e| AppError::Internal(format!("Invalid start date: {e}")))?;
        let utc_range_end_exclusive = tz::start_of_day_utc(
            &tz,
            end_date
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
        .bind::<diesel::sql_types::Date, _>(start_date)
        .bind::<diesel::sql_types::Date, _>(end_date)
        .bind::<diesel::sql_types::Uuid, _>(workspace_id)
        .bind::<diesel::sql_types::Timestamptz, _>(utc_range_start)
        .bind::<diesel::sql_types::Timestamptz, _>(utc_range_end_exclusive)
        .bind::<diesel::sql_types::Text, _>(tz_name)
        .bind::<diesel::sql_types::Nullable<diesel::sql_types::Array<diesel::sql_types::Uuid>>, _>(
            department_ids,
        )
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
            .get_workspace_schedules(None, auth_token, start_date, end_date, None, None, None)
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
                .bind::<diesel::sql_types::Nullable<diesel::sql_types::Array<diesel::sql_types::Uuid>>, _>(department_ids)
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
                    .and_then(|employments| i64::try_from(employments.len()).ok())
                    .unwrap_or(0);
                let absent = (r.active_employees - r.present - r.on_leave).max(0);
                let eligible = r.active_employees;
                AttendanceTrendPoint {
                    date: r.date,
                    present: r.present,
                    late,
                    absent,
                    on_leave: r.on_leave,
                    eligible,
                    attendance_rate: RateQuantity::percent(r.present, eligible, "employee_days"),
                    absence_rate: RateQuantity::percent(absent, eligible, "employee_days"),
                    late_rate: RateQuantity::percent(late, eligible, "employee_days"),
                }
            })
            .collect();

        Ok(trend)
    }

    #[allow(clippy::too_many_lines)]
    async fn fetch_department_attendance(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        query: &ChartAnalyticsQuery,
    ) -> Result<Vec<DepartmentAttendancePoint>, AppError> {
        let workspace_id = auth_token.workspace_id();
        let tz = self.resolve_timezone(auth_token, query).await?;
        let tz_name = tz.name();
        let today = Self::workspace_local_date(tz);
        let (start_date, end_date) = Self::resolve_range(query, today, 29);
        let department_ids = query.department_ids.as_deref();

        let mut conn = self
            .diesel_pools
            .tenant_transaction_conn(auth_token)
            .await?;

        let departments: Vec<DeptRow> = diesel::sql_query(include_str!(
            "sql/workspace_stats/department_attendance_departments.sql"
        ))
        .bind::<diesel::sql_types::Uuid, _>(workspace_id)
        .bind::<diesel::sql_types::Date, _>(end_date)
        .bind::<diesel::sql_types::Nullable<diesel::sql_types::Array<diesel::sql_types::Uuid>>, _>(
            department_ids,
        )
        .load(&mut *conn)
        .await?;

        if departments.is_empty() {
            return Ok(vec![]);
        }

        let utc_range_start = tz::start_of_day_utc(&tz, start_date)
            .map_err(|e| AppError::Internal(format!("Invalid date: {e}")))?;
        let utc_range_end_exclusive = tz::start_of_day_utc(
            &tz,
            end_date
                .succ_opt()
                .ok_or_else(|| AppError::Internal("Date overflow".to_string()))?,
        )
        .map_err(|e| AppError::Internal(format!("Invalid date: {e}")))?;

        let present_rows: Vec<PresentRow> = diesel::sql_query(include_str!(
            "sql/workspace_stats/department_attendance_present.sql"
        ))
        .bind::<diesel::sql_types::Uuid, _>(workspace_id)
        .bind::<diesel::sql_types::Timestamptz, _>(utc_range_start)
        .bind::<diesel::sql_types::Date, _>(end_date)
        .bind::<diesel::sql_types::Timestamptz, _>(utc_range_end_exclusive)
        .bind::<diesel::sql_types::Nullable<diesel::sql_types::Array<diesel::sql_types::Uuid>>, _>(
            department_ids,
        )
        .load(&mut *conn)
        .await?;

        let present_map: std::collections::HashMap<Uuid, i64> = present_rows
            .into_iter()
            .map(|r| (r.department_id, r.present_count))
            .collect();

        // Resolve late punches across the selected range. The previous implementation
        // only populated this map for `today`, which made the week view silently report
        // zero late employees even when late punches existed earlier in the week.
        let punch_rows: Vec<EmployeeFirstPunchRow> = diesel::sql_query(include_str!(
            "sql/workspace_stats/first_punches_range.sql"
        ))
        .bind::<diesel::sql_types::Timestamptz, _>(utc_range_start)
        .bind::<diesel::sql_types::Timestamptz, _>(utc_range_end_exclusive)
        .bind::<diesel::sql_types::Text, _>(tz_name)
        .bind::<diesel::sql_types::Uuid, _>(workspace_id)
        .bind::<diesel::sql_types::Nullable<diesel::sql_types::Array<diesel::sql_types::Uuid>>, _>(
            department_ids,
        )
        .load(&mut *conn)
        .await?;

        let dept_rows: Vec<EmployeeDepartmentRow> = diesel::sql_query(include_str!(
            "sql/workspace_stats/employee_departments_today.sql"
        ))
        .bind::<diesel::sql_types::Uuid, _>(workspace_id)
        .bind::<diesel::sql_types::Date, _>(end_date)
        .bind::<diesel::sql_types::Nullable<diesel::sql_types::Array<diesel::sql_types::Uuid>>, _>(
            department_ids,
        )
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
            .get_workspace_schedules(None, auth_token, start_date, end_date, None, None, None)
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
            .map(|(department_id, employments)| {
                (department_id, i64::try_from(employments.len()).unwrap_or(0))
            })
            .collect();

        let leave_rows: Vec<LeaveRow> = diesel::sql_query(include_str!(
            "sql/workspace_stats/department_attendance_leave.sql"
        ))
        .bind::<diesel::sql_types::Uuid, _>(workspace_id)
        .bind::<diesel::sql_types::Date, _>(start_date)
        .bind::<diesel::sql_types::Date, _>(end_date)
        .bind::<diesel::sql_types::Nullable<diesel::sql_types::Array<diesel::sql_types::Uuid>>, _>(
            department_ids,
        )
        .load(&mut *conn)
        .await?;

        let leave_map: std::collections::HashMap<Uuid, i64> = leave_rows
            .into_iter()
            .map(|r| (r.department_id, r.leave_count))
            .collect();

        let mut result: Vec<DepartmentAttendancePoint> = departments
            .into_iter()
            .map(|dept| {
                let present = *present_map.get(&dept.department_id).unwrap_or(&0);
                let late = *late_map.get(&dept.department_id).unwrap_or(&0);
                let on_leave = *leave_map.get(&dept.department_id).unwrap_or(&0);
                let on_time = (present - late).max(0);
                let absent = (dept.total_employees - present - on_leave).max(0);
                let total_employees = dept.total_employees;
                DepartmentAttendancePoint {
                    department_id: dept.department_id,
                    department_name: dept.department_name,
                    on_time,
                    late,
                    absent,
                    on_leave,
                    total_employees,
                    attendance_rate: RateQuantity::percent(on_time, total_employees, "employees"),
                    absence_rate: RateQuantity::percent(absent, total_employees, "employees"),
                    late_rate: RateQuantity::percent(late, total_employees, "employees"),
                }
            })
            .collect();

        // Sort descending by attendance rate (None sorts last), then apply the
        // optional Top-N cap. Raw counts are preserved for the tooltip/table.
        result.sort_by(|a, b| {
            b.attendance_rate
                .value
                .unwrap_or_else(|| Decimal::from(-1))
                .cmp(&a.attendance_rate.value.unwrap_or_else(|| Decimal::from(-1)))
        });
        if let Some(top_n) = query.top_n {
            let limit = usize::try_from(top_n.clamp(0, 100)).unwrap_or_default();
            result.truncate(limit);
        }

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
        query: &ChartAnalyticsQuery,
    ) -> Result<WorkspaceStats, AppError> {
        let workspace_id = auth_token.workspace_id();
        let tz = self.resolve_timezone(auth_token, query).await?;
        let tz_name = tz.name();
        let today = Self::workspace_local_date(tz);
        let department_ids = query.department_ids.as_deref();

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
                     AND (end_date IS NULL OR end_date >= $2)
                     AND ($3::uuid[] IS NULL OR department_id = ANY($3))",
                )
                .bind::<diesel::sql_types::Uuid, _>(workspace_id)
                .bind::<diesel::sql_types::Date, _>(today)
                .bind::<diesel::sql_types::Nullable<diesel::sql_types::Array<diesel::sql_types::Uuid>>, _>(department_ids)
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
                     AND (emp.end_date IS NULL OR emp.end_date >= $4)
                     AND ($5::uuid[] IS NULL OR emp.department_id = ANY($5))",
                )
                .bind::<diesel::sql_types::Timestamptz, _>(utc_today_start)
                .bind::<diesel::sql_types::Timestamptz, _>(utc_today_end_exclusive)
                .bind::<diesel::sql_types::Uuid, _>(workspace_id)
                .bind::<diesel::sql_types::Date, _>(today)
                .bind::<diesel::sql_types::Nullable<diesel::sql_types::Array<diesel::sql_types::Uuid>>, _>(department_ids)
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
                     AND (emp.end_date IS NULL OR emp.end_date >= $2)
                     AND ($3::uuid[] IS NULL OR emp.department_id = ANY($3))",
                )
                .bind::<diesel::sql_types::Uuid, _>(workspace_id)
                .bind::<diesel::sql_types::Date, _>(today)
                .bind::<diesel::sql_types::Nullable<diesel::sql_types::Array<diesel::sql_types::Uuid>>, _>(department_ids)
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
                     AND (emp.end_date IS NULL OR emp.end_date >= $1)
                     AND ($3::uuid[] IS NULL OR emp.department_id = ANY($3))",
                )
                .bind::<diesel::sql_types::Date, _>(today)
                .bind::<diesel::sql_types::Uuid, _>(workspace_id)
                .bind::<diesel::sql_types::Nullable<diesel::sql_types::Array<diesel::sql_types::Uuid>>, _>(department_ids)
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
                     AND (emp.end_date IS NULL OR emp.end_date >= $4)
                     AND ($5::uuid[] IS NULL OR emp.department_id = ANY($5))",
                )
                .bind::<diesel::sql_types::Timestamptz, _>(utc_today_start)
                .bind::<diesel::sql_types::Timestamptz, _>(utc_today_end_exclusive)
                .bind::<diesel::sql_types::Uuid, _>(workspace_id)
                .bind::<diesel::sql_types::Date, _>(today)
                .bind::<diesel::sql_types::Nullable<diesel::sql_types::Array<diesel::sql_types::Uuid>>, _>(department_ids)
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
            attendance_rate: RateQuantity::percent(present_today, total_employees, "employees"),
            absence_rate: RateQuantity::percent(absent_today, total_employees, "employees"),
            late_rate: RateQuantity::percent(late_today, total_employees, "employees"),
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
        query: &ChartAnalyticsQuery,
    ) -> Result<Vec<LatenessByDay>, AppError> {
        let workspace_id = auth_token.workspace_id();
        let tz = self.resolve_timezone(auth_token, query).await?;
        let tz_name = tz.name();
        let today = Self::workspace_local_date(tz);
        let (start_date, end_date) = Self::resolve_range(query, today, 29);
        let department_ids = query.department_ids.as_deref();

        let utc_start = tz::start_of_day_utc(&tz, start_date)
            .map_err(|e| AppError::Internal(format!("Invalid start date: {e}")))?;
        let utc_end_exclusive = tz::start_of_day_utc(
            &tz,
            end_date
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
            .get_workspace_schedules(None, auth_token, start_date, end_date, None, None, None)
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
                .bind::<diesel::sql_types::Nullable<diesel::sql_types::Array<diesel::sql_types::Uuid>>, _>(department_ids)
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
                                .map_err(|error| {
                                    AppError::Internal(format!("Invalid weekday: {error}"))
                                })?;
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
                    let day_idx = usize::try_from(dow - 1).map_err(|error| {
                        AppError::Internal(format!("Invalid weekday index: {error}"))
                    })?;
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

    #[allow(clippy::too_many_lines)]
    async fn fetch_executive_dashboard(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        request: &ExecutiveDashboardRequest,
    ) -> Result<ExecutiveDashboardSnapshots, AppError> {
        let workspace_id = auth_token.workspace_id();
        let department_ids = request.department_ids.as_deref();
        let location_ids = request.location_ids.as_deref();
        let mut conn = self
            .diesel_pools
            .tenant_transaction_conn(auth_token)
            .await?;

        #[derive(Debug, QueryableByName)]
        struct SnapshotRow {
            #[diesel(sql_type = diesel::sql_types::BigInt)]
            headcount: i64,
            #[diesel(sql_type = diesel::sql_types::BigInt)]
            eligible_employee_days: i64,
            #[diesel(sql_type = diesel::sql_types::BigInt)]
            present_employee_days: i64,
            #[diesel(sql_type = diesel::sql_types::BigInt)]
            absent_employee_days: i64,
            #[diesel(sql_type = diesel::sql_types::BigInt)]
            leave_employee_days: i64,
            #[diesel(sql_type = diesel::sql_types::BigInt)]
            termination_count: i64,
            #[diesel(sql_type = diesel::sql_types::BigInt)]
            headcount_start: i64,
            #[diesel(sql_type = diesel::sql_types::BigInt)]
            headcount_end: i64,
            #[diesel(sql_type = diesel::sql_types::BigInt)]
            approved_overtime_minutes: i64,
            #[diesel(sql_type = diesel::sql_types::BigInt)]
            pending_hr_actions: i64,
            #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
            currency_code: Option<String>,
            #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Numeric>)]
            overtime_cost: Option<Decimal>,
        }

        let sql = r"
WITH periods AS (
    SELECT $2::date AS start_date, $3::date AS end_date, 'current' AS period
    UNION ALL
    SELECT $4::date, $5::date, 'comparison'
),
active_days AS (
    SELECT p.period, COUNT(*)::bigint AS eligible_employee_days
    FROM periods p
    JOIN employments emp
      ON emp.workspace_id = $1
     AND emp.deleted_at IS NULL
     AND emp.status = 'active'
     AND emp.start_date <= p.end_date
     AND (emp.end_date IS NULL OR emp.end_date >= p.start_date)
     AND ($6::uuid[] IS NULL OR emp.department_id = ANY($6))
     AND ($7::uuid[] IS NULL OR emp.primary_location_id = ANY($7))
    CROSS JOIN LATERAL generate_series(
        GREATEST(emp.start_date, p.start_date),
        LEAST(COALESCE(emp.end_date, p.end_date), p.end_date),
        interval '1 day'
    ) day_value
    GROUP BY p.period
),
headcounts AS (
    SELECT p.period,
           COUNT(DISTINCT emp.employee_id) FILTER (WHERE emp.start_date <= p.end_date AND (emp.end_date IS NULL OR emp.end_date >= p.end_date))::bigint AS headcount_end,
           COUNT(DISTINCT emp.employee_id) FILTER (WHERE emp.start_date <= p.start_date AND (emp.end_date IS NULL OR emp.end_date >= p.start_date))::bigint AS headcount_start
    FROM periods p
    LEFT JOIN employments emp
      ON emp.workspace_id = $1
     AND emp.deleted_at IS NULL
     AND ($6::uuid[] IS NULL OR emp.department_id = ANY($6))
     AND ($7::uuid[] IS NULL OR emp.primary_location_id = ANY($7))
    GROUP BY p.period
),
attendance_days AS (
    SELECT p.period,
           COUNT(DISTINCT (ads.employment_id, ads.date)) FILTER (WHERE ads.actual_work_minutes > 0)::bigint AS present_employee_days,
           COUNT(DISTINCT (ads.employment_id, ads.date)) FILTER (WHERE ads.actual_work_minutes = 0 AND NOT ads.is_holiday)::bigint AS absent_employee_days,
           COUNT(DISTINCT (lr.employment_id, day_value::date))::bigint AS leave_employee_days
    FROM periods p
    LEFT JOIN attendance_daily_summaries ads
      ON ads.workspace_id = $1 AND ads.date BETWEEN p.start_date AND p.end_date
    LEFT JOIN employments emp ON emp.id = ads.employment_id
      AND emp.workspace_id = $1 AND emp.deleted_at IS NULL
      AND ($6::uuid[] IS NULL OR emp.department_id = ANY($6))
      AND ($7::uuid[] IS NULL OR emp.primary_location_id = ANY($7))
    LEFT JOIN leave_requests lr
      ON lr.status IN ('approved', 'pre_approved')
     AND lr.start_date <= p.end_date AND lr.end_date >= p.start_date
     AND lr.employment_id = emp.id
    LEFT JOIN LATERAL generate_series(
      GREATEST(lr.start_date, p.start_date), LEAST(lr.end_date, p.end_date), interval '1 day'
    ) day_value ON lr.id IS NOT NULL
    GROUP BY p.period
),
overtime AS (
    SELECT p.period, COALESCE(SUM(ads.approved_ot_minutes), 0)::bigint AS approved_overtime_minutes
    FROM periods p
    LEFT JOIN attendance_daily_summaries ads
      ON ads.workspace_id = $1 AND ads.date BETWEEN p.start_date AND p.end_date
    LEFT JOIN employments emp ON emp.id = ads.employment_id
      AND emp.workspace_id = $1 AND emp.deleted_at IS NULL
      AND ($6::uuid[] IS NULL OR emp.department_id = ANY($6))
      AND ($7::uuid[] IS NULL OR emp.primary_location_id = ANY($7))
    GROUP BY p.period
),
terminations AS (
    SELECT p.period, COUNT(DISTINCT e.id)::bigint AS termination_count
    FROM periods p
    LEFT JOIN employees e ON e.workspace_id = $1 AND e.deleted_at IS NULL
      AND e.termination_date BETWEEN p.start_date AND p.end_date
    GROUP BY p.period
),
actions AS (
    SELECT p.period,
      (SELECT COUNT(*) FROM leave_requests lr JOIN employments emp ON emp.id = lr.employment_id
       WHERE lr.status = 'pending' AND emp.workspace_id = $1 AND emp.deleted_at IS NULL
         AND lr.created_at::date BETWEEN p.start_date AND p.end_date
         AND ($6::uuid[] IS NULL OR emp.department_id = ANY($6)))
      + (SELECT COUNT(*) FROM overtime_requests ot
         WHERE ot.workspace_id = $1 AND ot.status = 'pending_approval'
           AND ot.planned_date BETWEEN p.start_date AND p.end_date
           AND ($6::uuid[] IS NULL OR ot.department_id = ANY($6))) AS pending_hr_actions
    FROM periods p
),
currency AS (
    SELECT COALESCE((SELECT currency_code FROM employers WHERE workspace_id = $1 AND deleted_at IS NULL AND currency_code IS NOT NULL ORDER BY created_at LIMIT 1), 'USD') AS currency_code
),
overtime_cost AS (
    SELECT p.period,
           CASE WHEN $9::boolean THEN COALESCE(SUM(ps.overtime_pay_amount), 0) END AS overtime_cost
    FROM periods p
    LEFT JOIN payslips ps
      ON ps.workspace_id = $1
     AND ps.status IN ('paid', 'finalized')
     AND ps.period_start <= p.end_date AND ps.period_end >= p.start_date
    LEFT JOIN employments emp
      ON emp.id = ps.employment_id
     AND emp.workspace_id = $1
     AND emp.deleted_at IS NULL
     AND ($6::uuid[] IS NULL OR emp.department_id = ANY($6))
     AND ($7::uuid[] IS NULL OR emp.primary_location_id = ANY($7))
    WHERE ps.id IS NOT NULL OR $9::boolean IS FALSE
    GROUP BY p.period
)
SELECT h.headcount_end AS headcount,
       COALESCE(a.eligible_employee_days, 0) AS eligible_employee_days,
       COALESCE(ad.present_employee_days, 0) AS present_employee_days,
       COALESCE(ad.absent_employee_days, 0) AS absent_employee_days,
       COALESCE(ad.leave_employee_days, 0) AS leave_employee_days,
       COALESCE(t.termination_count, 0) AS termination_count,
       h.headcount_start,
       h.headcount_end,
       COALESCE(o.approved_overtime_minutes, 0) AS approved_overtime_minutes,
       COALESCE(ac.pending_hr_actions, 0) AS pending_hr_actions,
       c.currency_code,
       oc.overtime_cost
FROM headcounts h
LEFT JOIN active_days a USING (period)
LEFT JOIN attendance_days ad USING (period)
LEFT JOIN overtime o USING (period)
LEFT JOIN terminations t USING (period)
LEFT JOIN actions ac USING (period)
LEFT JOIN overtime_cost oc USING (period)
CROSS JOIN currency c
WHERE h.period = $8
";

        let current_row = diesel::sql_query(sql)
            .bind::<diesel::sql_types::Uuid, _>(workspace_id)
            .bind::<diesel::sql_types::Date, _>(request.current_start)
            .bind::<diesel::sql_types::Date, _>(request.current_end)
            .bind::<diesel::sql_types::Date, _>(request.comparison_start)
            .bind::<diesel::sql_types::Date, _>(request.comparison_end)
            .bind::<diesel::sql_types::Nullable<diesel::sql_types::Array<diesel::sql_types::Uuid>>, _>(department_ids)
            .bind::<diesel::sql_types::Nullable<diesel::sql_types::Array<diesel::sql_types::Uuid>>, _>(location_ids)
            .bind::<diesel::sql_types::Text, _>("current")
            .bind::<diesel::sql_types::Bool, _>(request.allow_overtime_cost)
            .get_result::<SnapshotRow>(&mut *conn)
            .await
            .map_err(AppError::from)?;
        let comparison_row = diesel::sql_query(sql)
            .bind::<diesel::sql_types::Uuid, _>(workspace_id)
            .bind::<diesel::sql_types::Date, _>(request.current_start)
            .bind::<diesel::sql_types::Date, _>(request.current_end)
            .bind::<diesel::sql_types::Date, _>(request.comparison_start)
            .bind::<diesel::sql_types::Date, _>(request.comparison_end)
            .bind::<diesel::sql_types::Nullable<diesel::sql_types::Array<diesel::sql_types::Uuid>>, _>(department_ids)
            .bind::<diesel::sql_types::Nullable<diesel::sql_types::Array<diesel::sql_types::Uuid>>, _>(location_ids)
            .bind::<diesel::sql_types::Text, _>("comparison")
            .bind::<diesel::sql_types::Bool, _>(request.allow_overtime_cost)
            .get_result::<SnapshotRow>(&mut *conn)
            .await
            .map_err(AppError::from)?;
        let to_snapshot = |row: SnapshotRow| ExecutiveDashboardSnapshot {
            headcount: row.headcount,
            eligible_employee_days: row.eligible_employee_days,
            present_employee_days: row.present_employee_days,
            absent_employee_days: row.absent_employee_days,
            leave_employee_days: row.leave_employee_days,
            termination_count: row.termination_count,
            headcount_start: row.headcount_start,
            headcount_end: row.headcount_end,
            approved_overtime_minutes: row.approved_overtime_minutes,
            pending_hr_actions: row.pending_hr_actions,
            currency_code: row.currency_code,
            overtime_cost: row.overtime_cost,
        };
        Ok(ExecutiveDashboardSnapshots {
            current: to_snapshot(current_row),
            comparison: to_snapshot(comparison_row),
        })
    }
}
