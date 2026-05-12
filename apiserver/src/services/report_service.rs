use std::sync::Arc;

use crate::errors::AppError;
use chrono::{NaiveDate, TimeZone, Utc};
use chrono_tz::Tz;
use sqlx::PgPool;

use uuid::Uuid;

use crate::domain::policy::resolver::{PolicyResolver, ShiftPolicyOverrides};
use crate::models::{
    AttendanceReportSort, AttendanceSummary, DailyAttendanceReport, PaginatedResponse,
};
use crate::repositories::holiday_repository::HolidayRepository;
use crate::repositories::roster::RosterRepository;
use crate::repositories::workspace_repository::WorkspaceRepository;
use crate::repositories::{employee, DieselWorkspaceRepository, TimeEntryRepository};
use crate::services::AttendanceCalculator;

pub struct AttendanceReportArgs {
    pub workspace_id: Uuid,
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
    pub page: Option<i64>,
    pub limit: Option<i64>,
    pub employee_ids: Option<Vec<Uuid>>,
    pub department_id: Option<Uuid>,
    pub sort: Option<AttendanceReportSort>,
}

pub struct CalculateDayUnitsArgs<'a, 'b> {
    pub day_entries: &'a [&'a crate::models::TimeEntry],
    pub daily: &'b crate::models::EffectiveSchedule,
    pub date: NaiveDate,
    pub tz: Tz,
    pub employee_id: Uuid,
    pub workspace_id: Uuid,
    pub early_arrival_strategy: crate::models::EarlyArrivalStrategy,
    pub work_unit_minutes: i32,
    pub rounding_policy: &'a str,
}

pub struct ReportService {
    time_entry_repo: Arc<dyn TimeEntryRepository>,
    roster_repo: Arc<dyn RosterRepository>,
    pool: PgPool,
    diesel: crate::diesel_pool::DieselPools,
}

impl ReportService {
    pub fn new(
        time_entry_repo: Arc<dyn TimeEntryRepository>,
        roster_repo: Arc<dyn RosterRepository>,
        pool: PgPool,
        diesel: crate::diesel_pool::DieselPools,
    ) -> Self {
        Self {
            time_entry_repo,
            roster_repo,
            pool,
            diesel,
        }
    }

    /// Shared calculation for a single day's standard/OT units.
    ///
    /// Consolidates the identical logic previously duplicated in
    /// `get_daily_attendance_report` and `get_workspace_attendance_summary`.
    async fn calculate_day_units(&self, args: CalculateDayUnitsArgs<'_, '_>) -> (i32, i32) {
        // Day-off or no shift assigned → 0 work hours.
        // Without shift boundaries the calculator would treat all punch time
        // as "standard" minutes, inflating the report for roster day-offs.
        if args.daily.shift_id.is_none() {
            return (0, 0);
        }

        let actual_punch_in = args
            .day_entries
            .first()
            .map(|e| e.punch_in_time.with_timezone(&args.tz));
        let actual_punch_out = args
            .day_entries
            .last()
            .and_then(|e| e.punch_out_time.map(|t| t.with_timezone(&args.tz)));

        if let (Some(punch_in), Some(punch_out)) = (actual_punch_in, actual_punch_out) {
            let actual_break_minutes: i32 = args
                .day_entries
                .iter()
                .filter(|e| e.entry_type == "break")
                .map(|e| e.duration_minutes.unwrap_or(0))
                .sum();

            let shift_start_dt = args
                .daily
                .start_time
                .and_then(|t| args.tz.from_local_datetime(&args.date.and_time(t)).single());
            let shift_end_dt = args
                .daily
                .end_time
                .and_then(|t| args.tz.from_local_datetime(&args.date.and_time(t)).single());

            let rules = PolicyResolver::resolve_with_shift_overrides(
                &self.pool,
                args.employee_id,
                args.workspace_id,
                args.date,
                &ShiftPolicyOverrides {
                    grace_period_minutes: args.daily.early_punch_in_grace_period_minutes,
                    is_break_auto_deduct: args.daily.is_break_auto_deduct,
                    allow_early_ot: args.daily.allow_early_ot,
                    early_arrival_strategy: args.early_arrival_strategy,
                    is_break_overtime: false,
                },
            )
            .await;

            // Handle shift boundaries (with overnight shift support)
            let (shift_start, shift_end) =
                if let (Some(start), Some(mut end)) = (shift_start_dt, shift_end_dt) {
                    if args.daily.end_time < args.daily.start_time {
                        end += chrono::Duration::days(1);
                    }
                    (Some(start), Some(end))
                } else {
                    (None, None)
                };

            AttendanceCalculator::calculate_daily_units(
                &crate::services::attendance_calculator::DailyCalcInput {
                    punch_in,
                    punch_out,
                    shift_start,
                    shift_end,
                    scheduled_break_minutes: args.daily.break_duration_minutes.unwrap_or(0),
                    actual_break_minutes,
                    work_unit: args.work_unit_minutes,
                    rounding_policy: args.rounding_policy.to_string(),
                    rules,
                    ot_ceiling: None,
                },
            )
        } else {
            (0, 0)
        }
    }
    #[tracing::instrument(skip_all, err)]
    #[allow(clippy::too_many_lines)]
    pub async fn get_daily_attendance_report(
        &self,
        args: AttendanceReportArgs,
    ) -> Result<PaginatedResponse<DailyAttendanceReport>, AppError> {
        // 0. Get Workspace Settings for Timezone & Rounding Logic
        let workspace_repo = DieselWorkspaceRepository::new(self.pool.clone(), self.diesel.clone());
        let workspace = workspace_repo
            .get_workspace_by_id(args.workspace_id)
            .await
            .map_err(|e| {
                tracing::error!("Error fetching workspace: {}", e);
                AppError::Internal("Internal error".into())
            })?
            .ok_or(AppError::NotFound("Not found".into()))?;

        let tz: Tz = crate::utils::tz::parse_workspace_tz(&workspace.timezone);

        // 1. PAGINATION AT DATABASE LEVEL
        let page = args.page.unwrap_or(1);
        let limit = args.limit.unwrap_or(20);

        // Daily report always uses ASC sort (no sort parameter exposed)
        let (paginated_employees, total_employees) = employee::get_paginated_employee_ids(
            &self.pool,
            args.workspace_id,
            crate::models::EmployeePaginationArgs {
                filter_ids: args.employee_ids.as_ref(),
                department_id: args.department_id,
                page,
                limit,
                sort_desc: false, // ORDER BY full_name ASC
            },
        )
        .await
        .map_err(|e| {
            tracing::error!("Error fetching paginated employees: {}", e);
            AppError::Internal("Internal error".into())
        })?;

        // Early return if no employees
        if paginated_employees.is_empty() {
            return Ok(PaginatedResponse {
                data: Vec::new(),
                total: total_employees as usize,
                page,
                limit,
            });
        }

        // 2. Get employee IDs for filtered queries
        let emp_ids: Vec<Uuid> = paginated_employees.iter().map(|e| e.id).collect();

        // 3. Get schedules ONLY for paginated employees
        let schedules = self
            .roster_repo
            .get_workspace_schedules(
                None,
                args.workspace_id,
                args.start_date,
                args.end_date,
                Some(&emp_ids),
            )
            .await?;

        // 4. Convert NaiveDate to DateTime<Utc> using Workspace Timezone
        let (start_dt, end_dt) = crate::services::attendance_utils::date_range_to_utc_checked(
            tz,
            args.start_date,
            args.end_date,
        )
        .ok_or(AppError::Validation("Invalid request".into()))?;

        // 5. BATCH fetch time entries (eliminates N+1)
        let entries_map = self
            .time_entry_repo
            .get_workspace_time_entries_batch(args.workspace_id, &emp_ids, start_dt, end_dt)
            .await?;

        // 5a. BATCH fetch holidays for the date range
        let holidays_map = HolidayRepository::get_holidays_in_range(
            &self.pool,
            args.workspace_id,
            args.start_date,
            args.end_date,
        )
        .await?;

        // 5b. BATCH fetch OT claim statuses from attendance_daily_summaries
        let ot_status_map = crate::repositories::attendance_daily_summary_repository::get_batch_ot_statuses(
            &self.pool,
            args.workspace_id,
            &emp_ids,
            args.start_date,
            args.end_date,
        )
        .await?;

        // 5c. BATCH fetch approved leave requests for holiday-leave status detection
        let leave_date_set = crate::services::attendance_utils::build_leave_date_set(
            &self.pool,
            args.workspace_id,
            &emp_ids,
            args.start_date,
            args.end_date,
        )
        .await?;

        // 5d. BATCH fetch attendance rules (SCD Type 2)
        let attendance_rules_map = PolicyResolver::resolve_attendance_policy_batch(
            &self.pool,
            &emp_ids,
            args.workspace_id,
            args.start_date, // proxy date for the report period
        )
        .await?;

        // 6. Calculate reports (same logic as before)
        let mut reports = Vec::new();

        for emp_sched in schedules {
            // Get entries from HashMap (O(1) lookup instead of DB query)
            let entries = entries_map
                .get(&emp_sched.employee_id)
                .map(std::vec::Vec::as_slice)
                .unwrap_or(&[]);
            // Pre-group entries by date for O(1) per-date lookup
            let entries_by_date =
                crate::services::attendance_utils::group_entries_by_date(entries, tz);

            for daily in emp_sched.schedules {
                let date = daily.date;

                // O(1) lookup instead of O(entries) filter per day
                let day_entries = entries_by_date
                    .get(&date)
                    .map(std::vec::Vec::as_slice)
                    .unwrap_or(&[]);

                let actual_punch_in = day_entries
                    .first()
                    .map(|e| e.punch_in_time.with_timezone(&tz));
                let actual_punch_out = day_entries
                    .last()
                    .and_then(|e| e.punch_out_time.map(|t| t.with_timezone(&tz)));

                let emp_attendance_rules = attendance_rules_map
                    .get(&emp_sched.employee_id)
                    .cloned()
                    .unwrap_or(crate::models::AttendanceRules {
                        work_unit_minutes: 30,
                        rounding_policy: "floor".to_string(),
                        early_arrival_strategy: crate::models::EarlyArrivalStrategy::Block,
                    });

                let (std_work_units, ot_work_units) = self
                    .calculate_day_units(CalculateDayUnitsArgs {
                        day_entries,
                        daily: &daily,
                        date,
                        tz,
                        employee_id: emp_sched.employee_id,
                        workspace_id: workspace.id,
                        early_arrival_strategy: emp_attendance_rules.early_arrival_strategy,
                        work_unit_minutes: emp_attendance_rules.work_unit_minutes,
                        rounding_policy: &emp_attendance_rules.rounding_policy,
                    })
                    .await;

                // --- STATUS DETERMINATION (using shared attendance_utils) ---
                let holiday_entry = holidays_map.get(&date);
                let is_hol = holiday_entry.is_some();
                let is_on_leave = leave_date_set.contains(&(emp_sched.employee_id, date));

                let status = crate::services::attendance_utils::determine_daily_status(
                    &crate::services::attendance_utils::DailyStatusInput {
                        has_punch_in: actual_punch_in.is_some(),
                        punch_in_time: actual_punch_in.map(|p| p.time()),
                        scheduled_start: daily.start_time,
                        is_holiday: is_hol,
                        is_holiday_paid: holiday_entry.map(|h| h.is_paid).unwrap_or(true),
                        has_shift: daily.shift_id.is_some(),
                        is_on_leave,
                    },
                );

                // Collect metadata from all entries for this day
                let mut metadata_map = serde_json::Map::new();

                let log_metadata: Vec<_> = day_entries
                    .iter()
                    .filter_map(|e| e.generation_metadata.clone())
                    .collect();

                if !log_metadata.is_empty() {
                    metadata_map
                        .insert("time_entries".to_string(), serde_json::json!(log_metadata));
                }

                // Add schedule traces from effective schedule
                if !daily.traces.is_empty() {
                    metadata_map.insert(
                        "schedule_traces".to_string(),
                        serde_json::json!(daily.traces),
                    );
                }

                let combined_metadata = if metadata_map.is_empty() {
                    None
                } else {
                    Some(serde_json::Value::Object(metadata_map))
                };

                reports.push(DailyAttendanceReport {
                    date,
                    employee_id: emp_sched.employee_id,
                    employee_name: emp_sched.full_name.clone(),
                    employee_id_number: emp_sched.employee_id_number.clone(),
                    shift_symbol: daily.shift_symbol.clone(),
                    scheduled_start: daily.start_time,
                    scheduled_end: daily.end_time,
                    break_duration_minutes: daily.break_duration_minutes,
                    break_start: daily.break_start,
                    break_end: daily.break_end,
                    is_break_auto_deduct: daily.is_break_auto_deduct,
                    actual_punch_in: actual_punch_in.map(|t| t.with_timezone(&Utc)),
                    actual_punch_out: actual_punch_out.map(|t| t.with_timezone(&Utc)),
                    std_work_units,
                    ot_work_units,
                    ot_claim_status: ot_status_map.get(&(emp_sched.employee_id, date)).cloned(),
                    unit_type: crate::models::enums::TimekeepingUnit::DAY,
                    unit_coefficient_hours: emp_attendance_rules.work_unit_minutes as f64 / 60.0,
                    computed_std_hours: std_work_units as f64
                        * (emp_attendance_rules.work_unit_minutes as f64 / 60.0),
                    computed_ot_hours: ot_work_units as f64
                        * (emp_attendance_rules.work_unit_minutes as f64 / 60.0),
                    status,
                    is_holiday: is_hol,
                    is_unscheduled: status
                        == crate::services::attendance_utils::AttendanceStatus::Unscheduled,
                    holiday_name: holiday_entry.map(|h| h.name.clone()),
                    generation_metadata: combined_metadata,
                });
            }
        }

        // 7. Return paginated response (NO in-memory skip/take)
        Ok(PaginatedResponse {
            data: reports,
            total: total_employees as usize, // Total from DB pagination
            page,
            limit,
        })
    }

    /// Generates monthly summary attendance report.
    ///
    /// **IMPORTANT**: This method uses the same calculation logic as `get_daily_attendance_report`
    /// to ensure data consistency between Summary and Detail reports.
    ///
    /// Process:
    /// 1. DB-level pagination via get_paginated_employee_ids
    /// 2. Batch fetch schedules and time entries (eliminates N+1)
    /// 3. Apply AttendanceCalculator for each day (same as detail report)
    /// 4. Aggregate daily results per employee
    #[tracing::instrument(skip_all, err)]
    #[allow(clippy::too_many_lines)]
    pub async fn get_workspace_attendance_summary(
        &self,
        args: AttendanceReportArgs,
    ) -> Result<PaginatedResponse<AttendanceSummary>, AppError> {

        use std::collections::HashMap;

        // 0. Get Workspace Settings
        let workspace_repo = DieselWorkspaceRepository::new(self.pool.clone(), self.diesel.clone());
        let workspace = workspace_repo
            .get_workspace_by_id(args.workspace_id)
            .await
            .map_err(|e| {
                tracing::error!("Error fetching workspace: {}", e);
                AppError::Internal("Internal error".into())
            })?
            .ok_or(AppError::NotFound("Not found".into()))?;

        let tz: Tz = crate::utils::tz::parse_workspace_tz(&workspace.timezone);

        // 1. PAGINATION AT DATABASE LEVEL
        let page = args.page.unwrap_or(1);
        let limit = args.limit.unwrap_or(20);

        // Determine sort direction for name sorting at DB level
        // NameDesc -> ORDER BY full_name DESC; all others -> ORDER BY full_name ASC
        let sort_desc = matches!(args.sort, Some(AttendanceReportSort::NameDesc));

        let (paginated_employees, total_employees) = employee::get_paginated_employee_ids(
            &self.pool,
            args.workspace_id,
            crate::models::EmployeePaginationArgs {
                filter_ids: args.employee_ids.as_ref(),
                department_id: args.department_id,
                page,
                limit,
                sort_desc,
            },
        )
        .await
        .map_err(|e| {
            tracing::error!("Error fetching paginated employees: {}", e);
            AppError::Internal("Internal error".into())
        })?;

        // Early return if no employees
        if paginated_employees.is_empty() {
            return Ok(PaginatedResponse {
                data: Vec::new(),
                total: total_employees as usize,
                page,
                limit,
            });
        }

        // 2. Get employee IDs for filtered queries
        let emp_ids: Vec<Uuid> = paginated_employees.iter().map(|e| e.id).collect();

        // 3. Get schedules ONLY for paginated employees
        let schedules = self
            .roster_repo
            .get_workspace_schedules(
                None,
                args.workspace_id,
                args.start_date,
                args.end_date,
                Some(&emp_ids),
            )
            .await?;

        // 4. Convert dates to UTC for time entry query
        let (start_dt, end_dt) = crate::services::attendance_utils::date_range_to_utc_checked(
            tz,
            args.start_date,
            args.end_date,
        )
        .ok_or(AppError::Validation("Invalid request".into()))?;

        // 5. BATCH fetch time entries (eliminates N+1)
        let entries_map = self
            .time_entry_repo
            .get_workspace_time_entries_batch(args.workspace_id, &emp_ids, start_dt, end_dt)
            .await?;

        // 6. Fetch leave days for paginated employees (batch query)
        let leave_data = crate::repositories::leave_request_repository::get_batch_leave_days(
            &self.pool,
            &emp_ids,
            args.start_date,
            args.end_date,
        )
        .await?;

        // 6b. BATCH fetch attendance rules (SCD Type 2)
        let attendance_rules_map = PolicyResolver::resolve_attendance_policy_batch(
            &self.pool,
            &emp_ids,
            workspace.id,
            args.start_date, // proxy date for the report period
        )
        .await?;

        // 7. Calculate work units per employee using the SAME logic as daily report
        let mut employee_totals: HashMap<Uuid, (i32, i32, i32)> = HashMap::new();

        for emp_sched in schedules {
            let entries = entries_map
                .get(&emp_sched.employee_id)
                .map(std::vec::Vec::as_slice)
                .unwrap_or(&[]);

            let emp_attendance_rules = attendance_rules_map
                .get(&emp_sched.employee_id)
                .cloned()
                .unwrap_or(crate::models::AttendanceRules {
                    work_unit_minutes: 30,
                    rounding_policy: "floor".to_string(),
                    early_arrival_strategy: crate::models::EarlyArrivalStrategy::Block,
                });
            let mut employee_total_std = 0i32;
            let mut employee_total_ot = 0i32;

            // Pre-group entries by date for O(1) per-date lookup
            let entries_by_date =
                crate::services::attendance_utils::group_entries_by_date(entries, tz);

            // Calculate for each day (same logic as get_daily_attendance_report)
            for daily in &emp_sched.schedules {
                let date = daily.date;

                // O(1) lookup instead of O(entries) filter per day
                let day_entries = entries_by_date
                    .get(&date)
                    .map(std::vec::Vec::as_slice)
                    .unwrap_or(&[]);

                let (std_units, ot_units) = self
                    .calculate_day_units(CalculateDayUnitsArgs {
                        day_entries,
                        daily,
                        date,
                        tz,
                        employee_id: emp_sched.employee_id,
                        workspace_id: workspace.id,
                        early_arrival_strategy: emp_attendance_rules.early_arrival_strategy,
                        work_unit_minutes: emp_attendance_rules.work_unit_minutes,
                        rounding_policy: &emp_attendance_rules.rounding_policy,
                    })
                    .await;

                employee_total_std += std_units;
                employee_total_ot += ot_units;
            }

            employee_totals.insert(
                emp_sched.employee_id,
                (
                    employee_total_std,
                    employee_total_ot,
                    emp_attendance_rules.work_unit_minutes,
                ),
            );
        }

        // 8. Build results from paginated employees (preserve order)
        let mut results: Vec<AttendanceSummary> = paginated_employees
            .iter()
            .map(|emp| {
                let (std, ot, work_unit_minutes) =
                    employee_totals.get(&emp.id).copied().unwrap_or((0, 0, 30));
                let coeff = work_unit_minutes as f64 / 60.0;
                AttendanceSummary {
                    employee_id: emp.id,
                    employee_name: emp.full_name.clone(),
                    employee_id_number: emp.employee_id_number.clone(),
                    total_std_units: std,
                    total_ot_units: ot,
                    total_leave_days: leave_data.get(&emp.id).copied().unwrap_or(0),
                    unit_type: crate::models::enums::TimekeepingUnit::DAY,
                    unit_coefficient_hours: coeff,
                    computed_std_hours: std as f64 * coeff,
                    computed_ot_hours: ot as f64 * coeff,
                }
            })
            .collect();

        // 9. Sort within batch (for calculated fields only)
        if let Some(s) = &args.sort {
            match s {
                AttendanceReportSort::NameAsc | AttendanceReportSort::NameDesc => {
                    // Already sorted from DB - no action needed
                }
                AttendanceReportSort::WorkHoursAsc => results.sort_by(|a, b| {
                    (a.total_std_units + a.total_ot_units)
                        .cmp(&(b.total_std_units + b.total_ot_units))
                }),
                AttendanceReportSort::WorkHoursDesc => results.sort_by(|a, b| {
                    (b.total_std_units + b.total_ot_units)
                        .cmp(&(a.total_std_units + a.total_ot_units))
                }),
                AttendanceReportSort::LeaveDaysAsc => {
                    results.sort_by(|a, b| a.total_leave_days.cmp(&b.total_leave_days))
                }
                AttendanceReportSort::LeaveDaysDesc => {
                    results.sort_by(|a, b| b.total_leave_days.cmp(&a.total_leave_days))
                }
                AttendanceReportSort::OtHoursAsc => {
                    results.sort_by(|a, b| a.total_ot_units.cmp(&b.total_ot_units))
                }
                AttendanceReportSort::OtHoursDesc => {
                    results.sort_by(|a, b| b.total_ot_units.cmp(&a.total_ot_units))
                }
                AttendanceReportSort::TotalHoursAsc => results.sort_by(|a, b| {
                    (a.total_std_units + a.total_ot_units)
                        .cmp(&(b.total_std_units + b.total_ot_units))
                }),
                AttendanceReportSort::TotalHoursDesc => results.sort_by(|a, b| {
                    (b.total_std_units + b.total_ot_units)
                        .cmp(&(a.total_std_units + a.total_ot_units))
                }),
            }
        }

        // 10. Return response (NO in-memory skip/take - pagination done at DB level)
        Ok(PaginatedResponse {
            data: results,
            total: total_employees as usize,
            page,
            limit,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::DailyAttendanceReport;
    use crate::services::attendance_utils::AttendanceStatus;
    use chrono::{NaiveDate, NaiveTime, TimeZone};

    /// Helper: build a DailyAttendanceReport with the specified punch-in and shift presence.
    fn make_report(has_punch_in: bool, has_shift: bool) -> DailyAttendanceReport {
        let date = NaiveDate::from_ymd_opt(2026, 3, 12).unwrap();
        let punch_in = if has_punch_in {
            Some(chrono::Utc.with_ymd_and_hms(2026, 3, 12, 8, 0, 0).unwrap())
        } else {
            None
        };
        let punch_out = if has_punch_in {
            Some(chrono::Utc.with_ymd_and_hms(2026, 3, 12, 17, 0, 0).unwrap())
        } else {
            None
        };
        let (scheduled_start, scheduled_end) = if has_shift {
            (
                Some(NaiveTime::from_hms_opt(8, 0, 0).unwrap()),
                Some(NaiveTime::from_hms_opt(17, 0, 0).unwrap()),
            )
        } else {
            (None, None)
        };

        // Derive is_unscheduled using the same logic as report_service & worker handler
        let status = crate::services::attendance_utils::determine_daily_status(
            &crate::services::attendance_utils::DailyStatusInput {
                has_punch_in,
                punch_in_time: punch_in.map(|p| p.time()),
                scheduled_start,
                is_holiday: false,
                is_holiday_paid: false,
                has_shift,
                is_on_leave: false,
            },
        );

        DailyAttendanceReport {
            date,
            employee_id: Uuid::new_v4(),
            employee_name: "Test Employee".to_string(),
            employee_id_number: "EMP001".to_string(),
            shift_symbol: None,
            scheduled_start,
            scheduled_end,
            break_duration_minutes: None,
            break_start: None,
            break_end: None,
            is_break_auto_deduct: None,
            actual_punch_in: punch_in,
            actual_punch_out: punch_out,
            std_work_units: 0,
            ot_work_units: 0,
            ot_claim_status: None,
            unit_type: crate::models::enums::TimekeepingUnit::DAY,
            unit_coefficient_hours: 8.0,
            computed_std_hours: 0.0,
            computed_ot_hours: 0.0,
            status,
            is_holiday: false,
            is_unscheduled: status == AttendanceStatus::Unscheduled,
            holiday_name: None,
            generation_metadata: None,
        }
    }

    // ============================================================================
    // is_unscheduled flag derivation — all 4 combinations
    // ============================================================================

    /// Employee has a punch-in but NO shift → is_unscheduled = true
    #[test]
    fn test_is_unscheduled_punch_in_no_shift() {
        let report = make_report(true, false);
        assert!(
            report.is_unscheduled,
            "punch_in + no shift → is_unscheduled must be true"
        );
        assert!(report.actual_punch_in.is_some());
        assert!(report.scheduled_start.is_none());
    }

    /// Employee has a punch-in AND a shift → is_unscheduled = false
    #[test]
    fn test_is_not_unscheduled_punch_in_with_shift() {
        let report = make_report(true, true);
        assert!(
            !report.is_unscheduled,
            "punch_in + has shift → is_unscheduled must be false"
        );
    }

    /// Employee has NO punch-in and NO shift → is_unscheduled = false
    #[test]
    fn test_is_not_unscheduled_no_punch_no_shift() {
        let report = make_report(false, false);
        assert!(
            !report.is_unscheduled,
            "no punch + no shift → is_unscheduled must be false"
        );
    }

    /// Employee has NO punch-in but HAS a shift → is_unscheduled = false
    #[test]
    fn test_is_not_unscheduled_no_punch_with_shift() {
        let report = make_report(false, true);
        assert!(
            !report.is_unscheduled,
            "no punch + has shift → is_unscheduled must be false"
        );
    }

    // ============================================================================
    // Serialization roundtrip
    // ============================================================================

    /// is_unscheduled=true survives JSON serialize → deserialize roundtrip
    #[test]
    fn test_is_unscheduled_serialization_roundtrip_true() {
        let report = make_report(true, false);
        assert!(report.is_unscheduled);

        let json = serde_json::to_string(&report).unwrap();
        let decoded: DailyAttendanceReport = serde_json::from_str(&json).unwrap();

        assert!(
            decoded.is_unscheduled,
            "is_unscheduled=true should survive JSON roundtrip"
        );
    }

    /// is_unscheduled=false survives JSON serialize → deserialize roundtrip
    #[test]
    fn test_is_unscheduled_serialization_roundtrip_false() {
        let report = make_report(true, true);
        assert!(!report.is_unscheduled);

        let json = serde_json::to_string(&report).unwrap();
        let decoded: DailyAttendanceReport = serde_json::from_str(&json).unwrap();

        assert!(
            !decoded.is_unscheduled,
            "is_unscheduled=false should survive JSON roundtrip"
        );
    }

    /// When is_unscheduled is missing from JSON (old data), #[serde(default)] → false
    #[test]
    fn test_is_unscheduled_serde_default_when_missing() {
        // Serialize a report normally, remove the is_unscheduled key, then deserialize
        let report = make_report(true, false);
        let mut json_value: serde_json::Value = serde_json::to_value(&report).unwrap();

        // Remove the key to simulate old JSON without the field
        json_value.as_object_mut().unwrap().remove("is_unscheduled");
        assert!(json_value.get("is_unscheduled").is_none());

        let decoded: DailyAttendanceReport = serde_json::from_value(json_value).unwrap();

        assert!(
            !decoded.is_unscheduled,
            "missing is_unscheduled in JSON should default to false via #[serde(default)]"
        );
    }
}
