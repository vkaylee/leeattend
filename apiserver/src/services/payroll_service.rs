use crate::errors::AppError;
use chrono::NaiveDate;
use chrono_tz::Tz;
use futures_util::stream::{self, StreamExt};
use rust_decimal::Decimal;
use sqlx::PgPool;
use std::collections::{BTreeSet, HashMap};
use std::sync::Arc;
use uuid::Uuid;

use crate::domain::policy::resolver::PolicyResolver;
use crate::models::Payslip;
use crate::repositories::department_repository::DepartmentRepository;
use crate::repositories::holiday_repository::HolidayRepositoryTrait;
use crate::repositories::payroll_repository::PayrollRepository;
use crate::repositories::workspace_repository::WorkspaceRepository;
use crate::repositories::{
    roster_repository::RosterRepository, time_entry_repository,
};

use super::attendance_matcher::AttendanceMatcher;
use super::financial_calculator::FinancialCalculator;

pub struct PayrollService {
    payroll_repo: Arc<dyn PayrollRepository>,
    department_repo: Arc<dyn DepartmentRepository>,
    time_entry_repo: Arc<dyn time_entry_repository::TimeEntryRepository>,
    workspace_repo: Arc<dyn WorkspaceRepository>,
    pub holiday_repo: Arc<dyn HolidayRepositoryTrait>,
    pub roster_repo: Arc<dyn RosterRepository>,
    pub pool: PgPool,
}

// ---------------------------------------------------------------------------
// Internal data structs for PayrollService decomposition
// ---------------------------------------------------------------------------

/// All pre-fetched data required for payroll calculation.
/// Constructed once by `fetch_context()` and passed to subsequent phases.
pub struct PayrollContext {
    pub(crate) workspace: crate::models::Workspace,
    pub(crate) tz: Tz,
    pub(crate) start_date: NaiveDate,
    pub(crate) end_date: NaiveDate,
    pub(crate) salary_history: Vec<(crate::models::EmployeeSalaryComponent, crate::models::SalaryComponentDefinition)>,
    pub(crate) all_dept_history: Vec<crate::models::EmployeeDepartmentHistory>,
    pub(crate) daily_entries_map: HashMap<NaiveDate, Vec<crate::models::TimeEntry>>,
    pub(crate) holidays_map: HashMap<NaiveDate, crate::models::Holiday>,
    pub(crate) roster_data: Arc<crate::cache::RosterData>,
    pub(crate) emp_id_number: String,
    pub(crate) emp_tags: Option<Vec<String>>,
    pub(crate) payroll_roster_map: HashMap<(Uuid, NaiveDate), Option<Uuid>>,
}

/// Named record for a payroll segment snapshot.
///
/// Replaces the 10-element tuple that was previously used for
/// `segment_snapshots`, giving each field a meaningful name.
#[derive(Debug, Clone)]
pub struct SegmentSnapshot {
    pub(crate) seg_start: NaiveDate,
    pub(crate) seg_end: NaiveDate,
    pub(crate) department_id: Option<Uuid>,
    pub(crate) total_base: Decimal,
    pub(crate) std_working_days: Decimal,
    pub(crate) hours_per_day: Decimal,
    pub(crate) actual_worked_days: Decimal,
    pub(crate) paid_days: Decimal,
    pub(crate) amount: Decimal,
    pub(crate) source: String,
}

impl PayrollService {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        payroll_repo: Arc<dyn PayrollRepository>,
        department_repo: Arc<dyn DepartmentRepository>,
        time_entry_repo: Arc<dyn time_entry_repository::TimeEntryRepository>,
        workspace_repo: Arc<dyn WorkspaceRepository>,
        holiday_repo: Arc<dyn HolidayRepositoryTrait>,
        roster_repo: Arc<dyn RosterRepository>,
        pool: PgPool,
    ) -> Self {
        Self { payroll_repo, department_repo, time_entry_repo, workspace_repo, holiday_repo, roster_repo, pool }
    }

    /// Phase 1: Fetch all data needed for payroll calculation.
    /// Public to allow access from decomposed structs.
    async fn fetch_context(
        &self,
        employee_id: Uuid,
        workspace_id: Uuid,
        year: i32,
        month: u32,
    ) -> Result<PayrollContext, AppError> {
        let pool = &self.pool;
        let workspace = self.workspace_repo
            .get_workspace_by_id(workspace_id)
            .await
            .map_err(|e| {
                tracing::error!("Error fetching workspace: {}", e);
                AppError::Internal("Internal error".into())
            })?
            .ok_or(AppError::NotFound("Not found".into()))?;

        let tz: Tz = workspace.timezone.parse().unwrap_or(chrono_tz::UTC);

        let start_date = NaiveDate::from_ymd_opt(year, month, 1).ok_or(AppError::Validation("Invalid request".into()))?;
        let next_month_date = if month == 12 {
            NaiveDate::from_ymd_opt(year + 1, 1, 1)
        } else {
            NaiveDate::from_ymd_opt(year, month + 1, 1)
        }
        .ok_or(AppError::Validation("Invalid request".into()))?;
        let end_date = next_month_date
            .pred_opt()
            .ok_or(AppError::Internal("Internal error".into()))?;

        let salary_history = self.payroll_repo.get_salary_components_history(
            workspace_id, employee_id, start_date, end_date,
        )
        .await
        .map_err(AppError::Sqlx)?;

        let all_dept_history =
            self.department_repo.get_employee_department_history(workspace_id, employee_id)
                .await
                .map_err(AppError::Sqlx)?;

        let start_dt = local_to_utc(&tz, start_date, 0, 0, 0)
            .map_err(AppError::Internal)?;
        let end_dt = local_to_utc(&tz, end_date, 23, 59, 59)
            .map_err(AppError::Internal)?;

        let time_entries =
            self.time_entry_repo.get_employee_time_entries(workspace_id, employee_id, start_dt, end_dt)
                .await
                .map_err(AppError::Sqlx)?;

        let holidays_map = self.holiday_repo.get_holidays_in_range(
            workspace_id, start_date, end_date,
        )
        .await
        .unwrap_or_default();

        let roster_data = crate::cache::build_roster_data(pool, workspace_id)
            .await
            .map_err(|e| {
                tracing::error!("Failed to build roster data: {}", e);
                AppError::Internal("Internal error".into())
            })?;

        let emp_row = sqlx::query(
            "SELECT employee_id_number, tags FROM employees WHERE id = $1",
        )
        .bind(employee_id)
        .fetch_optional(pool)
        .await
        .map_err(AppError::Sqlx)?
        .ok_or(AppError::NotFound("Not found".into()))?;

        let emp_id_number: String = {
            use sqlx::Row;
            emp_row.get("employee_id_number")
        };
        let emp_tags: Option<Vec<String>> = {
            use sqlx::Row;
            emp_row.get("tags")
        };

        let rosters = self.roster_repo.get_employee_rosters( employee_id, start_date, end_date)
            .await
            .map_err(AppError::Sqlx)?;
        let payroll_roster_map: HashMap<(Uuid, NaiveDate), Option<Uuid>> = rosters
            .into_iter()
            .map(|r| ((r.employee_id, r.date), r.shift_id))
            .collect();

        let mut daily_entries_map: HashMap<NaiveDate, Vec<crate::models::TimeEntry>> =
            HashMap::new();
        for entry in time_entries {
            let local_date = entry.punch_in_time.with_timezone(&tz).date_naive();
            daily_entries_map.entry(local_date).or_default().push(entry);
        }

        Ok(PayrollContext {
            workspace,
            tz,
            start_date,
            end_date,
            salary_history,
            all_dept_history,
            daily_entries_map,
            holidays_map,
            roster_data,
            emp_id_number,
            emp_tags,
            payroll_roster_map,
        })
    }

    /// Phase 2: Determine salary/dept change segments.
    fn build_segments(ctx: &PayrollContext) -> Vec<(NaiveDate, NaiveDate)> {
        let mut change_points = BTreeSet::new();
        change_points.insert(ctx.start_date);

        for (comp, _) in &ctx.salary_history {
            if comp.effective_date > ctx.start_date && comp.effective_date <= ctx.end_date {
                change_points.insert(comp.effective_date);
            }
        }
        for dept in &ctx.all_dept_history {
            if dept.effective_date > ctx.start_date && dept.effective_date <= ctx.end_date {
                change_points.insert(dept.effective_date);
            }
        }

        let sorted_dates: Vec<NaiveDate> = change_points.into_iter().collect();
        let mut segments = Vec::new();

        for i in 0..sorted_dates.len() {
            let seg_start = sorted_dates[i];
            let seg_end = if i + 1 < sorted_dates.len() {
                sorted_dates[i + 1].pred_opt().unwrap_or(seg_start)
            } else {
                ctx.end_date
            };
            segments.push((seg_start, seg_end));
        }
        segments
    }

    #[tracing::instrument(skip_all, err)]
    pub async fn calculate_monthly_payslip(
        &self,
        employee_id: Uuid,
        workspace_id: Uuid,
        year: i32,
        month: u32,
    ) -> Result<Payslip, AppError> {
        let pool = &self.pool;
        // Phase 1: Fetch all context data
        let ctx = self.fetch_context(employee_id, workspace_id, year, month).await?;

        // Phase 2: Build segments
        let segments = Self::build_segments(&ctx);

        // Phase 3: Calculate each segment
        let mut total_base_pay = Decimal::ZERO;
        let mut total_std_units = 0;
        let mut total_ot_units = 0;
        let mut total_late_early_count = 0;
        let mut segment_snapshots = Vec::new();
        let mut total_overtime_pay = Decimal::ZERO;
        let mut total_holiday_work_premium = Decimal::ZERO;
        let mut policy_cache: HashMap<Uuid, Vec<crate::models::OvertimeRule>> = HashMap::new();

        for (seg_start, seg_end) in segments {
            // A. Context Resolution
            let active_dept = ctx.all_dept_history
                .iter()
                .filter(|d| d.effective_date <= seg_start)
                .max_by_key(|d| d.effective_date);
            let dept_id = active_dept.map(|d| d.department_id);

            let mut segment_total_base = Decimal::ZERO;
            let mut segment_ot_base = Decimal::ZERO;
            let mut processed_def_ids = BTreeSet::new();
            for (esc, scd) in ctx.salary_history.iter().rev() {
                if !processed_def_ids.contains(&scd.id) && esc.effective_date <= seg_start {
                    segment_total_base += esc.amount;
                    if scd.is_base_for_ot {
                        segment_ot_base += esc.amount;
                    }
                    processed_def_ids.insert(scd.id);
                }
            }

            // Resolve Standard Month Config
            let std_config = PolicyResolver::resolve_standard_month(
                pool, employee_id, dept_id, workspace_id, seg_start,
            )
            .await
            .map_err(AppError::Sqlx)?;

            let seg_units_per_day = (std_config.hours_per_day * Decimal::from(60))
                / Decimal::from(ctx.workspace.work_unit_minutes);

            // B. Attendance — delegated to AttendanceMatcher
            let att = AttendanceMatcher::process_segment_attendance(
                pool, &ctx, seg_start, seg_end,
                segment_total_base, segment_ot_base,
                &std_config, seg_units_per_day,
                &mut policy_cache,
            ).await;

            // C. Financials
            let worked_days_dec = Decimal::from(att.std_units) / seg_units_per_day;
            let daily_rate = FinancialCalculator::calculate_daily_rate(segment_total_base, std_config.std_working_days);
            let segment_amount = daily_rate * worked_days_dec;

            total_base_pay += segment_amount;
            total_std_units += att.std_units;
            total_ot_units += att.ot_units;
            total_overtime_pay += att.ot_pay;
            total_holiday_work_premium += att.holiday_work_premium;
            total_late_early_count += att.late_early_count;

            segment_snapshots.push(SegmentSnapshot {
                seg_start,
                seg_end,
                department_id: dept_id,
                total_base: segment_total_base,
                std_working_days: std_config.std_working_days,
                hours_per_day: std_config.hours_per_day,
                actual_worked_days: worked_days_dec,
                paid_days: worked_days_dec,
                amount: segment_amount,
                source: std_config.source,
            });
        }

        // Phase 4: Enforce monthly OT cap
        let payroll_settings = self.payroll_repo.get_payroll_settings_at(
            workspace_id, ctx.end_date,
        ).await.unwrap_or(None);
        if let Some(ref ps) = payroll_settings {
            if let Some(max_monthly_hours) = ps.max_ot_hours_per_month {
                let (capped_units, capped_pay) = FinancialCalculator::apply_ot_cap(
                    total_ot_units, total_overtime_pay,
                    max_monthly_hours, ctx.workspace.work_unit_minutes,
                );
                if capped_units < total_ot_units {
                    tracing::info!(
                        employee_id = %employee_id,
                        "Monthly OT capped at {} units (max {} hours)",
                        capped_units, max_monthly_hours
                    );
                }
                total_ot_units = capped_units;
                total_overtime_pay = capped_pay;
            }
        }

        // Phase 5: Incentives — delegated to FinancialCalculator
        let incentives = FinancialCalculator::calculate_incentives(
            pool, workspace_id, employee_id,
            total_std_units, total_late_early_count,
            ctx.start_date, ctx.end_date,
        ).await?;

        // Phase 6: Deductions — delegated to FinancialCalculator
        let deductions = FinancialCalculator::calculate_deductions(
            pool, workspace_id, employee_id,
            ctx.start_date, ctx.end_date, &segment_snapshots,
        ).await?;

        // Phase 7: Assemble Payslip — delegated to FinancialCalculator
        let payslip = FinancialCalculator::assemble_payslip(
            employee_id, ctx.start_date, ctx.end_date,
            total_base_pay, total_overtime_pay, total_holiday_work_premium,
            incentives.total_amount, deductions.amount,
            total_std_units, total_ot_units,
        );

        // Phase 8: Persist — delegated to PayslipPersister
        self.persist_calculated_payslip(payslip, segment_snapshots, incentives.results).await
    }

    /// Helper extracted to move Database operations out of the calculation flow
    #[allow(clippy::too_many_arguments)]
    async fn persist_calculated_payslip(
        &self,
        payslip: Payslip,
        segment_snapshots: Vec<SegmentSnapshot>,
        incentive_results: Vec<crate::models::IncentiveEvaluationResult>,
    ) -> Result<Payslip, AppError> {
        let created_payslip = self.payroll_repo.create_payslip(&payslip)
            .await
            .map_err(AppError::Sqlx)?;

        for snap in segment_snapshots {
            self.payroll_repo.create_payslip_segment(
                created_payslip.id,
                snap.seg_start,
                snap.seg_end,
                snap.department_id,
                snap.total_base,
                snap.std_working_days,
                snap.hours_per_day,
                snap.actual_worked_days,
                snap.paid_days,
                snap.amount,
                Some(snap.source),
            )
            .await
            .map_err(AppError::Sqlx)?;
        }

        for res in incentive_results {
            let _ = self.payroll_repo.create_payslip_incentive(created_payslip.id, &res).await;
        }

        Ok(created_payslip)
    }

    #[tracing::instrument(skip_all)]
    pub async fn calculate_batch_payslips(
        &self,
        employee_ids: Vec<Uuid>,
        workspace_id: Uuid,
        year: i32,
        month: u32,
    ) -> crate::models::BatchCalculatePayrollResponse {
        const MAX_CONCURRENT_PAYROLL: usize = 10;

        let results: Vec<_> = stream::iter(employee_ids)
            .map(|emp_id| {
                let svc = Self {
                    payroll_repo: self.payroll_repo.clone(),
                    department_repo: self.department_repo.clone(),
                    time_entry_repo: self.time_entry_repo.clone(),
                    workspace_repo: self.workspace_repo.clone(),
                    holiday_repo: self.holiday_repo.clone(),
                    roster_repo: self.roster_repo.clone(),
                    pool: self.pool.clone(),
                };
                async move {
                    match svc.calculate_monthly_payslip(
                        emp_id,
                        workspace_id,
                        year,
                        month,
                    )
                    .await
                    {
                        Ok(payslip) => crate::models::BatchCalculateResult {
                            employee_id: emp_id,
                            status: "success".to_string(),
                            payslip: Some(payslip),
                            error_message: None,
                        },
                        Err(status) => crate::models::BatchCalculateResult {
                            employee_id: emp_id,
                            status: "error".to_string(),
                            payslip: None,
                            error_message: Some(format!("HTTP Error: {status}")),
                        },
                    }
                }
            })
            .buffer_unordered(MAX_CONCURRENT_PAYROLL)
            .collect()
            .await;

        crate::models::BatchCalculatePayrollResponse { results }
    }
}

use crate::utils::chrono_helpers::local_to_utc;

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    fn d(y: i32, m: u32, day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, day).expect("valid date")
    }

    fn make_salary_entry(effective_date: NaiveDate) -> (crate::models::EmployeeSalaryComponent, crate::models::SalaryComponentDefinition) {
        (
            crate::models::EmployeeSalaryComponent {
                id: Uuid::new_v4(),
                employee_id: Uuid::new_v4(),
                component_definition_id: Uuid::new_v4(),
                amount: Decimal::from(1000),
                effective_date,
                created_at: Utc::now(),
                updated_at: Utc::now(),
            },
            crate::models::SalaryComponentDefinition {
                id: Uuid::new_v4(),
                workspace_id: Uuid::new_v4(),
                name: "Base".into(),
                code: "BASE".into(),
                is_base_for_ot: true,
                is_taxable: true,
                created_at: Utc::now(),
                updated_at: Utc::now(),
            },
        )
    }

    fn make_dept_entry(effective_date: NaiveDate) -> crate::models::EmployeeDepartmentHistory {
        crate::models::EmployeeDepartmentHistory {
            id: Uuid::new_v4(),
            employee_id: Uuid::new_v4(),
            department_id: Uuid::new_v4(),
            effective_date,
            notes: None,
            created_at: Utc::now(),
        }
    }

    fn make_ctx(
        start_date: NaiveDate,
        end_date: NaiveDate,
        salary_history: Vec<(crate::models::EmployeeSalaryComponent, crate::models::SalaryComponentDefinition)>,
        all_dept_history: Vec<crate::models::EmployeeDepartmentHistory>,
    ) -> PayrollContext {
        PayrollContext {
            workspace: crate::models::Workspace {
                id: Uuid::new_v4(),
                name: "Test".into(),
                work_unit_minutes: 30,
                rounding_policy: "floor".into(),
                date_format: "YYYY-MM-DD".into(),
                time_format: "HH:mm".into(),
                timezone: "UTC".into(),
                theme_primary_color: "#000".into(),
                default_language: "en".into(),
                early_arrival_strategy: crate::models::EarlyArrivalStrategy::Allow,
                holiday_overnight_strategy: "split".into(),
                default_holiday_ot_multiplier: Decimal::from(2),
                created_at: Utc::now(),
            },
            tz: chrono_tz::UTC,
            start_date,
            end_date,
            salary_history,
            all_dept_history,
            daily_entries_map: HashMap::new(),
            holidays_map: HashMap::new(),
            roster_data: Arc::new(crate::cache::RosterData {
                shifts: HashMap::new(),
                fixed_assignments: HashMap::new(),
                pattern_assignments: HashMap::new(),
                dept_patterns: HashMap::new(),
                dept_fixed_shifts: HashMap::new(),
                active_rules: Vec::new(),
                rule_patterns: HashMap::new(),
                dept_history: HashMap::new(),
            }),
            emp_id_number: "EMP001".into(),
            emp_tags: None,
            payroll_roster_map: HashMap::new(),
        }
    }

    #[test]
    fn test_build_segments_no_changes() {
        let ctx = make_ctx(d(2026, 3, 1), d(2026, 3, 31), vec![], vec![]);
        let segments = PayrollService::build_segments(&ctx);
        assert_eq!(segments.len(), 1);
        assert_eq!(segments[0], (d(2026, 3, 1), d(2026, 3, 31)));
    }

    #[test]
    fn test_build_segments_salary_change_mid_month() {
        let ctx = make_ctx(
            d(2026, 3, 1), d(2026, 3, 31),
            vec![make_salary_entry(d(2026, 3, 16))],
            vec![],
        );
        let segments = PayrollService::build_segments(&ctx);
        assert_eq!(segments.len(), 2);
        assert_eq!(segments[0], (d(2026, 3, 1), d(2026, 3, 15)));
        assert_eq!(segments[1], (d(2026, 3, 16), d(2026, 3, 31)));
    }

    #[test]
    fn test_build_segments_dept_change_mid_month() {
        let ctx = make_ctx(
            d(2026, 3, 1), d(2026, 3, 31),
            vec![],
            vec![make_dept_entry(d(2026, 3, 10))],
        );
        let segments = PayrollService::build_segments(&ctx);
        assert_eq!(segments.len(), 2);
        assert_eq!(segments[0], (d(2026, 3, 1), d(2026, 3, 9)));
        assert_eq!(segments[1], (d(2026, 3, 10), d(2026, 3, 31)));
    }

    #[test]
    fn test_build_segments_multiple_changes() {
        let ctx = make_ctx(
            d(2026, 3, 1), d(2026, 3, 31),
            vec![make_salary_entry(d(2026, 3, 10))],
            vec![make_dept_entry(d(2026, 3, 20))],
        );
        let segments = PayrollService::build_segments(&ctx);
        assert_eq!(segments.len(), 3);
        assert_eq!(segments[0], (d(2026, 3, 1), d(2026, 3, 9)));
        assert_eq!(segments[1], (d(2026, 3, 10), d(2026, 3, 19)));
        assert_eq!(segments[2], (d(2026, 3, 20), d(2026, 3, 31)));
    }

    #[test]
    fn test_build_segments_change_on_first_day_no_extra_segment() {
        let ctx = make_ctx(
            d(2026, 3, 1), d(2026, 3, 31),
            vec![make_salary_entry(d(2026, 3, 1))],
            vec![],
        );
        let segments = PayrollService::build_segments(&ctx);
        assert_eq!(segments.len(), 1); // change on start_date doesn't create extra segment
    }

    #[test]
    fn test_build_segments_change_after_end_ignored() {
        let ctx = make_ctx(
            d(2026, 3, 1), d(2026, 3, 31),
            vec![make_salary_entry(d(2026, 4, 5))],
            vec![],
        );
        let segments = PayrollService::build_segments(&ctx);
        assert_eq!(segments.len(), 1); // change outside range skipped
    }

    #[test]
    fn test_build_segments_same_day_salary_and_dept() {
        let ctx = make_ctx(
            d(2026, 3, 1), d(2026, 3, 31),
            vec![make_salary_entry(d(2026, 3, 15))],
            vec![make_dept_entry(d(2026, 3, 15))],
        );
        let segments = PayrollService::build_segments(&ctx);
        // Same date → deduplicated by BTreeSet
        assert_eq!(segments.len(), 2);
        assert_eq!(segments[0], (d(2026, 3, 1), d(2026, 3, 14)));
        assert_eq!(segments[1], (d(2026, 3, 15), d(2026, 3, 31)));
    }

    // --- Tests for calculate_daily_rate ---

    #[test]
    fn test_daily_rate_normal() {
        // Base salary 26_000 / 26 working days = 1_000 per day
        let rate = FinancialCalculator::calculate_daily_rate(
            Decimal::from(26_000),
            Decimal::from(26),
        );
        assert_eq!(rate, Decimal::from(1_000));
    }

    #[test]
    fn test_daily_rate_zero_working_days_returns_zero() {
        // Edge case: zero working days should not panic/divide-by-zero
        let rate = FinancialCalculator::calculate_daily_rate(
            Decimal::from(30_000),
            Decimal::ZERO,
        );
        assert_eq!(rate, Decimal::ZERO);
    }

    #[test]
    fn test_daily_rate_zero_base_returns_zero() {
        let rate = FinancialCalculator::calculate_daily_rate(
            Decimal::ZERO,
            Decimal::from(26),
        );
        assert_eq!(rate, Decimal::ZERO);
    }

    // --- Tests for apply_ot_cap ---

    #[test]
    fn test_ot_cap_no_cap_needed() {
        // 10 OT units, cap allows 20 hours (= 40 units at 30-min work_unit)
        let (units, pay) = FinancialCalculator::apply_ot_cap(
            10,                   // total_ot_units
            Decimal::from(5_000), // total_overtime_pay
            Decimal::from(20),    // max_monthly_hours = 20h
            30,                   // work_unit_minutes = 30 min
        );
        // 20h * 60 = 1200 min / 30 = 40 max units; 10 < 40 → no cap
        assert_eq!(units, 10);
        assert_eq!(pay, Decimal::from(5_000));
    }

    #[test]
    fn test_ot_cap_applied() {
        // 50 OT units exceed cap of 40 → should be reduced
        let (units, pay) = FinancialCalculator::apply_ot_cap(
            50,                    // total_ot_units
            Decimal::from(10_000), // total_overtime_pay
            Decimal::from(20),     // max_monthly_hours = 20h
            30,                    // work_unit_minutes = 30 min
        );
        // Max units = 20*60/30 = 40
        assert_eq!(units, 40);
        // Pay proportional: 10_000 * (40/50) = 8_000
        assert_eq!(pay, Decimal::from(8_000));
    }

    #[test]
    fn test_ot_cap_exact_limit() {
        let (units, pay) = FinancialCalculator::apply_ot_cap(
            40,                   // exactly at limit
            Decimal::from(8_000),
            Decimal::from(20),    // 20h * 60 / 30 = 40
            30,
        );
        assert_eq!(units, 40); // no reduction
        assert_eq!(pay, Decimal::from(8_000));
    }

    #[test]
    fn test_ot_cap_zero_units() {
        let (units, pay) = FinancialCalculator::apply_ot_cap(
            0,
            Decimal::ZERO,
            Decimal::from(20),
            30,
        );
        assert_eq!(units, 0);
        assert_eq!(pay, Decimal::ZERO);
    }

    // --- Tests for SegmentSnapshot struct ---

    #[test]
    fn test_segment_snapshot_construction() {
        let snap = SegmentSnapshot {
            seg_start: d(2026, 3, 1),
            seg_end: d(2026, 3, 15),
            department_id: Some(Uuid::new_v4()),
            total_base: Decimal::from(13_000),
            std_working_days: Decimal::from(26),
            hours_per_day: Decimal::from(8),
            actual_worked_days: Decimal::from(11),
            paid_days: Decimal::from(11),
            amount: Decimal::from(5_500),
            source: "workspace".to_string(),
        };

        assert_eq!(snap.seg_start, d(2026, 3, 1));
        assert_eq!(snap.seg_end, d(2026, 3, 15));
        assert!(snap.department_id.is_some());
        assert_eq!(snap.source, "workspace");
    }

    #[test]
    fn test_segment_snapshot_daily_rate_via_helper() {
        let snap = SegmentSnapshot {
            seg_start: d(2026, 3, 1),
            seg_end: d(2026, 3, 31),
            department_id: None,
            total_base: Decimal::from(26_000),
            std_working_days: Decimal::from(26),
            hours_per_day: Decimal::from(8),
            actual_worked_days: Decimal::from(22),
            paid_days: Decimal::from(22),
            amount: Decimal::from(22_000),
            source: "system".to_string(),
        };

        let daily_rate = FinancialCalculator::calculate_daily_rate(
            snap.total_base,
            snap.std_working_days,
        );
        assert_eq!(daily_rate, Decimal::from(1_000));
    }
}
