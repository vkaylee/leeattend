use crate::errors::AppError;

use crate::repositories::overtime_policy_repository::OvertimePolicyRepositoryTrait;
use chrono::NaiveDate;
use chrono_tz::Tz;
use futures_util::stream::{self, StreamExt};
use rust_decimal::Decimal;
use std::collections::{BTreeSet, HashMap};
use std::sync::Arc;
use uuid::Uuid;

use crate::domain::policy::resolver::PolicyResolver;
use crate::models::Payslip;
use crate::repositories::department_repository::DepartmentRepository;
use crate::repositories::holiday_repository::HolidayRepository;
use crate::repositories::leave_request_repository::LeaveRequestRepository;
use crate::repositories::payroll_repository::PayrollRepository;
use crate::repositories::workspace_repository::WorkspaceRepository;
use crate::repositories::{roster::RosterRepository, time_entry_repository};

use super::attendance_matcher::AttendanceMatcher;
use super::financial_calculator::FinancialCalculator;
use super::PolicyResolverService;

// ---------------------------------------------------------------------------
// Internal data structs for PayrollService decomposition
// ---------------------------------------------------------------------------

/// All pre-fetched data required for payroll calculation.
/// Constructed once by `fetch_context()` and passed to subsequent phases.
pub struct PayrollContext {
    pub(crate) employee_id: Uuid,
    pub(crate) workspace: crate::models::Workspace,
    pub(crate) tz: Tz,
    pub(crate) start_date: NaiveDate,
    pub(crate) end_date: NaiveDate,
    pub(crate) salary_history: Vec<(
        crate::models::EmployeeSalaryComponent,
        crate::models::SalaryComponentDefinition,
    )>,
    pub(crate) all_dept_history: Vec<crate::models::EmployeeDepartmentHistory>,
    pub(crate) daily_entries_map: HashMap<NaiveDate, Vec<crate::models::TimeEntry>>,
    pub(crate) holidays_map: HashMap<NaiveDate, crate::models::Holiday>,
    pub(crate) roster_data: Arc<crate::cache::RosterData>,
    pub(crate) emp_id_number: String,
    pub(crate) emp_tags: Option<Vec<String>>,
    pub(crate) payroll_roster_map: HashMap<(Uuid, NaiveDate), Option<Uuid>>,
    pub(crate) working_conditions: Vec<crate::models::WorkingCondition>,
    pub(crate) attendance_policy: crate::models::AttendanceRules,
    pub(crate) emp_payroll_configs: Vec<crate::models::EmployeePayrollConfig>,
    pub(crate) ws_payroll_settings_history: Vec<crate::models::PayrollSettings>,
    pub(crate) department_payroll_configs:
        HashMap<Uuid, Vec<crate::models::DepartmentPayrollConfig>>,
    pub(crate) overtime_data:
        crate::repositories::overtime_policy_repository::WorkspaceOvertimeData,
    #[allow(dead_code)]
    pub(crate) active_contracts: Vec<crate::models::contract::EmployeeContract>,
    pub(crate) active_assignments: Vec<crate::models::employee_assignment::EmployeeAssignment>,
    pub(crate) active_employments: Vec<crate::models::employment::Employment>,
    pub(crate) primary_location_id: Uuid,
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

#[async_trait::async_trait]
pub trait RosterDataProvider: Send + Sync {
    async fn get_roster_data(
        &self,
        workspace_id: Uuid,
    ) -> Result<Arc<crate::cache::RosterData>, crate::errors::AppError>;
}

/// Concrete `RosterDataProvider` that calls `RosterRepository::get_raw_roster_data`.
/// Used in production (`app_builder`) and in integration tests.
pub struct DirectRosterDataProvider(pub Arc<dyn RosterRepository>);

#[async_trait::async_trait]
impl RosterDataProvider for DirectRosterDataProvider {
    async fn get_roster_data(
        &self,
        workspace_id: Uuid,
    ) -> Result<Arc<crate::cache::RosterData>, crate::errors::AppError> {
        self.0.get_raw_roster_data(workspace_id).await
    }
}

pub struct PayrollServiceArgs {
    pub payroll_repo: Arc<dyn PayrollRepository>,
    pub department_repo: Arc<dyn DepartmentRepository>,
    pub time_entry_repo: Arc<dyn time_entry_repository::TimeEntryRepository>,
    pub workspace_repo: Arc<dyn WorkspaceRepository>,
    pub holiday_repo: Arc<dyn HolidayRepository>,
    pub roster_repo: Arc<dyn RosterRepository>,
    pub leave_repo: Arc<dyn LeaveRequestRepository>,
    pub policy_repo: Arc<dyn crate::repositories::AttendancePolicyRepository>,
    pub employee_repo: Arc<dyn crate::repositories::EmployeeRepository>,
    pub overtime_policy_repo: Arc<dyn OvertimePolicyRepositoryTrait>,
    pub salary_base_repo:
        Arc<dyn crate::repositories::salary_base_repository::SalaryBaseRepository>,
    pub roster_provider: Arc<dyn RosterDataProvider>,
    pub contract_repo: Arc<dyn crate::repositories::EmployeeContractRepository>,
    pub employment_repo: Arc<dyn crate::repositories::EmploymentRepository>,
}

pub struct PayrollService {
    payroll_repo: Arc<dyn PayrollRepository>,
    department_repo: Arc<dyn DepartmentRepository>,
    time_entry_repo: Arc<dyn time_entry_repository::TimeEntryRepository>,
    workspace_repo: Arc<dyn WorkspaceRepository>,
    holiday_repo: Arc<dyn HolidayRepository>,
    roster_repo: Arc<dyn RosterRepository>,
    leave_repo: Arc<dyn LeaveRequestRepository>,
    policy_repo: Arc<dyn crate::repositories::AttendancePolicyRepository>,
    employee_repo: Arc<dyn crate::repositories::EmployeeRepository>,
    overtime_policy_repo: Arc<dyn OvertimePolicyRepositoryTrait>,
    salary_base_repo: Arc<dyn crate::repositories::salary_base_repository::SalaryBaseRepository>,
    roster_provider: Arc<dyn RosterDataProvider>,
    contract_repo: Arc<dyn crate::repositories::EmployeeContractRepository>,
    employment_repo: Arc<dyn crate::repositories::EmploymentRepository>,
}
impl PayrollService {
    pub fn new(args: PayrollServiceArgs) -> Self {
        Self {
            payroll_repo: args.payroll_repo,
            department_repo: args.department_repo,
            time_entry_repo: args.time_entry_repo,
            workspace_repo: args.workspace_repo,
            holiday_repo: args.holiday_repo,
            roster_repo: args.roster_repo,
            leave_repo: args.leave_repo,
            policy_repo: args.policy_repo,
            employee_repo: args.employee_repo,
            overtime_policy_repo: args.overtime_policy_repo,
            salary_base_repo: args.salary_base_repo,
            roster_provider: args.roster_provider,
            contract_repo: args.contract_repo,
            employment_repo: args.employment_repo,
        }
    }

    /// Phase 1: Fetch all data needed for payroll calculation.
    /// Public to allow access from decomposed structs.
    #[allow(clippy::too_many_lines)]
    async fn fetch_context(
        &self,
        employee_id: Uuid,
        workspace_id: Uuid,
        year: i32,
        month: u32,
    ) -> Result<PayrollContext, AppError> {
        let workspace = self
            .workspace_repo
            .get_workspace_by_id(workspace_id)
            .await
            .map_err(|e| {
                tracing::error!("Error fetching workspace: {}", e);
                AppError::Internal(format!("Workspace fetch error: {e}"))
            })?
            .ok_or(AppError::NotFound("Not found".into()))?;

        let tz: Tz = crate::utils::tz::parse_workspace_tz(workspace.timezone());

        let start_date = NaiveDate::from_ymd_opt(year, month, 1)
            .ok_or(AppError::Validation("Invalid request".into()))?;
        let next_month_date = if month == 12 {
            NaiveDate::from_ymd_opt(year + 1, 1, 1)
        } else {
            NaiveDate::from_ymd_opt(year, month + 1, 1)
        }
        .ok_or(AppError::Validation("Invalid request".into()))?;
        let end_date = next_month_date
            .pred_opt()
            .ok_or(AppError::Internal("Date calculation error".into()))?;

        let all_dept_history = self
            .department_repo
            .get_employee_department_history(
                &crate::diesel_pool::TenantAuthToken::system_override(workspace_id),
                employee_id,
            )
            .await?;

        let employee = self
            .employee_repo
            .get_employee_by_id(
                employee_id,
                &crate::diesel_pool::TenantAuthToken::system_override(workspace_id),
            )
            .await?
            .ok_or(AppError::NotFound("Employee not found".into()))?;

        let active_emp_id = employee
            .active_employment_id()
            .ok_or_else(|| AppError::validation_raw("No active employment found"))?;

        let salary_history = self
            .payroll_repo
            .get_salary_components_history(workspace_id, active_emp_id, start_date, end_date)
            .await?;

        let emp_id_number = employee.employee_id_number().to_string();
        let emp_tags = Some(employee.tags().to_vec());

        let start_dt = local_to_utc(&tz, start_date, 0, 0, 0).map_err(AppError::Internal)?;
        let end_dt = local_to_utc(&tz, end_date, 23, 59, 59).map_err(AppError::Internal)?;

        let time_entries = self
            .time_entry_repo
            .get_employment_time_entries(
                &crate::diesel_pool::TenantAuthToken::system_override(workspace_id),
                active_emp_id,
                start_dt,
                end_dt,
            )
            .await?;

        let holidays_map = self
            .holiday_repo
            .get_holidays_in_range(
                &crate::diesel_pool::TenantAuthToken::system_override(workspace_id),
                start_date,
                end_date,
            )
            .await?;

        let roster_data = self.roster_provider.get_roster_data(workspace_id).await?;

        let rosters = self
            .roster_repo
            .get_employment_rosters(
                &crate::diesel_pool::TenantAuthToken::system_override(workspace_id),
                active_emp_id,
                start_date,
                end_date,
            )
            .await?;
        let payroll_roster_map: HashMap<(Uuid, NaiveDate), Option<Uuid>> = rosters
            .into_iter()
            .map(|r| ((r.employment_id, r.date), r.shift_id))
            .collect();

        let mut daily_entries_map: HashMap<NaiveDate, Vec<crate::models::TimeEntry>> =
            HashMap::new();
        for entry in time_entries {
            let local_date = entry.punch_in_time().with_timezone(&tz).date_naive();
            daily_entries_map.entry(local_date).or_default().push(entry);
        }

        let (working_conditions, _) = self
            .employee_repo
            .get_working_conditions(
                &crate::diesel_pool::TenantAuthToken::system_override(workspace_id),
                active_emp_id,
                1,
                3000,
            )
            .await?;

        let effective_time = start_date
            .and_hms_opt(0, 0, 0)
            .unwrap_or_default()
            .and_utc();
        let policy_version = self
            .policy_repo
            .get_employee_effective_policy(
                &crate::diesel_pool::TenantAuthToken::system_override(workspace_id),
                employee_id,
                effective_time,
            )
            .await?;

        let attendance_policy = if let Some(pv) = policy_version {
            pv.rules
        } else {
            crate::models::AttendanceRules {
                work_unit_minutes: 30,
                rounding_policy: "floor".to_string(),
                early_arrival_strategy: crate::models::EarlyArrivalStrategy::Block,
            }
        };

        let emp_payroll_configs = self
            .employee_repo
            .get_employee_payroll_configs(
                &crate::diesel_pool::TenantAuthToken::system_override(workspace_id),
                active_emp_id,
            )
            .await?;

        let ws_payroll_settings_history = self
            .payroll_repo
            .get_payroll_settings_history(workspace_id)
            .await?;

        let department_ids: Vec<Uuid> = all_dept_history
            .iter()
            .map(crate::models::EmployeeDepartmentHistory::department_id)
            .collect::<std::collections::BTreeSet<_>>()
            .into_iter()
            .collect();

        let department_payroll_configs = self
            .department_repo
            .get_department_payroll_configs_batch(
                &crate::diesel_pool::TenantAuthToken::system_override(workspace_id),
                &department_ids,
            )
            .await?;

        let overtime_data = self
            .overtime_policy_repo
            .get_workspace_overtime_data(workspace_id)
            .await?;

        let active_contracts = self
            .contract_repo
            .list_employee_contracts(
                &crate::diesel_pool::TenantAuthToken::system_override(workspace_id),
                employee_id,
            )
            .await?;

        let active_assignments = self
            .employee_repo
            .list_employee_assignments(
                &crate::diesel_pool::TenantAuthToken::system_override(workspace_id),
                active_emp_id,
            )
            .await?;

        let auth_token = crate::diesel_pool::TenantAuthToken::system_override(workspace_id);
        let active_employment = self
            .employment_repo
            .get_active_employment_on_date(&auth_token, employee_id, start_date)
            .await?
            .ok_or_else(|| AppError::NotFound("Active employment not found".into()))?;

        let primary_location_id = active_employment.primary_location_id().ok_or_else(|| {
            AppError::NotFound("Primary location not found for active employment".into())
        })?;

        let active_employments = self
            .employment_repo
            .list_employee_employments(&auth_token, employee_id)
            .await?;

        Ok(PayrollContext {
            employee_id,
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
            working_conditions,
            attendance_policy,
            emp_payroll_configs,
            ws_payroll_settings_history,
            department_payroll_configs,
            overtime_data,
            active_contracts,
            active_assignments,
            active_employments,
            primary_location_id,
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
            if dept.effective_date() > ctx.start_date && dept.effective_date() <= ctx.end_date {
                change_points.insert(dept.effective_date());
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
    #[allow(clippy::too_many_lines)]
    pub async fn calculate_monthly_payslip(
        &self,
        employee_id: Uuid,
        workspace_id: Uuid,
        year: i32,
        month: u32,
    ) -> Result<Payslip, AppError> {
        // Phase 1: Fetch all context data
        let ctx = self
            .fetch_context(employee_id, workspace_id, year, month)
            .await?;

        self.calculate_payslip_with_context(ctx).await
    }

    #[tracing::instrument(skip_all, err)]
    #[allow(clippy::too_many_lines)]
    pub async fn calculate_payslip_with_context(
        &self,
        ctx: PayrollContext,
    ) -> Result<Payslip, AppError> {
        let employee_id = ctx.employee_id;
        let workspace_id = ctx.workspace.id();
        let active_emp = ctx
            .active_employments
            .iter()
            .find(|e| {
                e.status() == "active"
                    && e.start_date() <= ctx.start_date
                    && e.end_date().is_none_or(|end| end >= ctx.start_date)
            })
            .or_else(|| ctx.active_employments.first())
            .ok_or_else(|| AppError::NotFound("No active employment found".into()))?;
        let employment_id = active_emp.id();

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
            let active_dept = ctx
                .all_dept_history
                .iter()
                .filter(|d| d.effective_date() <= seg_start)
                .max_by_key(|d| d.effective_date());
            let dept_id = active_dept.map(crate::models::EmployeeDepartmentHistory::department_id);

            let mut segment_total_base = Decimal::ZERO;
            let mut segment_ot_base = Decimal::ZERO;
            let mut processed_def_ids = BTreeSet::new();
            for (esc, scd) in ctx.salary_history.iter().rev() {
                if !processed_def_ids.contains(&scd.id) && esc.effective_date <= seg_start {
                    segment_total_base += esc.amount;
                    // TODO: In the future, check if scd belongs to "ot_base" group.
                    // For now, we assume all components contribute to OT base.
                    segment_ot_base += esc.amount;
                    processed_def_ids.insert(scd.id);
                }
            }

            let _rules = PolicyResolver::resolve_with_shift_overrides_pure(
                Some(ctx.attendance_policy.clone()),
                ctx.working_conditions.clone(),
                seg_start,
                &crate::domain::policy::resolver::ShiftPolicyOverrides::default(),
            );

            let emp_payroll_config = ctx
                .emp_payroll_configs
                .iter()
                .filter(|c| c.effective_date <= seg_start)
                .max_by_key(|c| c.effective_date)
                .cloned();

            let dept_payroll_config = if let Some(d_id) = dept_id {
                ctx.department_payroll_configs
                    .get(&d_id)
                    .and_then(|configs| {
                        configs
                            .iter()
                            .filter(|c| c.effective_date <= seg_start)
                            .max_by_key(|c| c.effective_date)
                            .cloned()
                    })
            } else {
                None
            };

            let ws_payroll_settings = ctx
                .ws_payroll_settings_history
                .iter()
                .filter(|s| s.effective_date <= seg_start)
                .max_by_key(|s| s.effective_date)
                .cloned();

            let std_config = PolicyResolver::resolve_standard_month_pure(
                emp_payroll_config,
                dept_payroll_config,
                ws_payroll_settings,
            );

            let attendance_policy = ctx.attendance_policy.clone();

            let seg_units_per_day = (std_config.hours_per_day() * Decimal::from(60))
                / Decimal::from(attendance_policy.work_unit_minutes);

            // B. Attendance — delegated to AttendanceMatcher
            let att = AttendanceMatcher::process_segment_attendance(
                crate::services::attendance_matcher::ProcessSegmentAttendanceArgs {
                    ctx: &ctx,
                    seg_start,
                    seg_end,
                    segment_total_base,
                    segment_ot_base,
                    std_config: &std_config,
                    attendance_rules: &attendance_policy,
                    seg_units_per_day,
                    policy_cache: &mut policy_cache,
                    overtime_policy_repo: &*self.overtime_policy_repo,
                },
            )
            .await;

            // C. Financials
            let worked_days_dec = Decimal::from(att.std_units) / seg_units_per_day;
            let daily_rate = FinancialCalculator::calculate_daily_rate(
                segment_total_base,
                std_config.std_working_days(),
            );
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
                std_working_days: std_config.std_working_days(),
                hours_per_day: std_config.hours_per_day(),
                actual_worked_days: worked_days_dec,
                paid_days: worked_days_dec,
                amount: segment_amount,
                source: std_config.source().to_string(),
            });
        }

        // Phase 4: Enforce monthly OT cap
        let payroll_settings = self
            .payroll_repo
            .get_payroll_settings_at(workspace_id, ctx.end_date)
            .await?;
        if let Some(ref ps) = payroll_settings {
            if let Some(max_monthly_hours) = ps.max_ot_hours_per_month {
                let month_end_policy = PolicyResolverService::resolve_attendance_policy(
                    &*self.policy_repo,
                    employee_id,
                    workspace_id,
                    ctx.end_date,
                )
                .await?;

                let (capped_units, capped_pay) = FinancialCalculator::apply_ot_cap(
                    total_ot_units,
                    total_overtime_pay,
                    max_monthly_hours,
                    month_end_policy.work_unit_minutes,
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
            self.payroll_repo.as_ref(),
            self.leave_repo.as_ref(),
            crate::services::financial_calculator::CalculateIncentivesArgs {
                workspace_id,
                employee_id,
                total_std_units,
                total_late_early_count,
                start_date: ctx.start_date,
                end_date: ctx.end_date,
            },
        )
        .await?;

        // Phase 6: Deductions — delegated to FinancialCalculator
        let gross_salary = total_base_pay
            + total_overtime_pay
            + total_holiday_work_premium
            + incentives.total_amount;
        let last_department_id = segment_snapshots.last().and_then(|s| s.department_id);

        let mut base_values = HashMap::new();
        base_values.insert("gross".to_string(), gross_salary);
        base_values.insert("base_pay".to_string(), total_base_pay);

        if let Ok(groups) = self.salary_base_repo.get_groups(workspace_id).await {
            for group in groups {
                let amount = if group.slug == "bhxh_base" {
                    total_base_pay
                } else {
                    gross_salary
                };
                base_values.insert(group.slug, amount);
            }
        }

        let deductions = FinancialCalculator::calculate_deductions(
            self.payroll_repo.as_ref(),
            self.leave_repo.as_ref(),
            crate::services::financial_calculator::CalculateDeductionsArgs {
                workspace_id,
                employee_id,
                department_id: last_department_id,
                start_date: ctx.start_date,
                end_date: ctx.end_date,
                base_values: &base_values,
                segment_snapshots: &segment_snapshots,
            },
        )
        .await?;

        // Phase 7: Assemble Payslip — delegated to FinancialCalculator
        let payslip = FinancialCalculator::assemble_payslip(
            crate::services::financial_calculator::AssemblePayslipArgs {
                workspace_id,
                employment_id,
                start_date: ctx.start_date,
                end_date: ctx.end_date,
                total_base_pay,
                total_overtime_pay,
                total_holiday_work_premium,
                total_incentive_amount: incentives.total_amount,
                deductions_amount: deductions.amount,
                total_std_units,
                total_ot_units,
            },
        );

        // Phase 8: Persist — delegated to PayslipPersister
        self.persist_calculated_payslip(
            payslip,
            segment_snapshots,
            incentives.results,
            deductions.rule_results,
        )
        .await
    }

    /// Helper extracted to move Database operations out of the calculation flow
    async fn persist_calculated_payslip(
        &self,
        payslip: Payslip,
        segment_snapshots: Vec<SegmentSnapshot>,
        incentive_results: Vec<crate::models::IncentiveEvaluationResult>,
        rule_results: Vec<crate::services::payroll_rule_evaluator::RuleEvaluationResult>,
    ) -> Result<Payslip, AppError> {
        let workspace_id = payslip.workspace_id;

        let segments_args: Vec<crate::models::CreatePayslipSegmentArgs> = segment_snapshots
            .into_iter()
            .map(|snap| crate::models::CreatePayslipSegmentArgs {
                start_date: snap.seg_start,
                end_date: snap.seg_end,
                department_id: snap.department_id,
                applied_base_salary: snap.total_base,
                applied_std_days: snap.std_working_days,
                hours_per_day: snap.hours_per_day,
                actual_work_days: snap.actual_worked_days,
                paid_work_days: snap.paid_days,
                amount: snap.amount,
                std_days_source: Some(snap.source),
            })
            .collect();

        let rules_args: Vec<(String, Decimal)> = rule_results
            .into_iter()
            .map(|r| (r.rule_code, r.amount))
            .collect();

        self.payroll_repo
            .persist_calculated_payslip_data(
                workspace_id,
                &payslip,
                &segments_args,
                &incentive_results,
                &rules_args,
            )
            .await
    }

    #[tracing::instrument(skip_all)]
    #[allow(clippy::too_many_lines)]
    pub async fn calculate_batch_payslips(
        &self,
        employment_ids: Vec<Uuid>,
        workspace_id: Uuid,
        year: i32,
        month: u32,
    ) -> crate::models::BatchCalculatePayrollResponse {
        const MAX_CONCURRENT_PAYROLL: usize = 10;

        let Ok(Some(workspace)) = self.workspace_repo.get_workspace_by_id(workspace_id).await
        else {
            return crate::models::BatchCalculatePayrollResponse { results: vec![] };
        };
        let tz: Tz = crate::utils::tz::parse_workspace_tz(workspace.timezone());

        let start_date = NaiveDate::from_ymd_opt(year, month, 1).unwrap_or_default();
        let next_month_date = if month == 12 {
            NaiveDate::from_ymd_opt(year + 1, 1, 1)
        } else {
            NaiveDate::from_ymd_opt(year, month + 1, 1)
        }
        .unwrap_or_default();
        let end_date = next_month_date.pred_opt().unwrap_or_default();

        let start_dt = local_to_utc(&tz, start_date, 0, 0, 0).unwrap_or_default();
        let end_dt = local_to_utc(&tz, end_date, 23, 59, 59).unwrap_or_default();

        let token = &crate::diesel_pool::TenantAuthToken::system_override(workspace_id);

        let overtime_data = match self
            .overtime_policy_repo
            .get_workspace_overtime_data(workspace_id)
            .await
        {
            Ok(data) => data,
            Err(e) => {
                tracing::error!(
                    workspace_id = %workspace_id,
                    error = %e,
                    "Failed to fetch overtime data for batch payroll — proceeding with empty overtime data"
                );
                crate::repositories::overtime_policy_repository::WorkspaceOvertimeData {
                    policies: std::collections::HashMap::new(),
                    assignments: Vec::new(),
                }
            }
        };

        let holidays_map = match self
            .holiday_repo
            .get_holidays_in_range(token, start_date, end_date)
            .await
        {
            Ok(map) => map,
            Err(e) => {
                tracing::error!(
                    workspace_id = %workspace_id,
                    error = %e,
                    "Failed to fetch holidays for batch payroll — proceeding without holidays"
                );
                std::collections::HashMap::new()
            }
        };

        let Ok(roster_data) = self.roster_provider.get_roster_data(workspace_id).await else {
            return crate::models::BatchCalculatePayrollResponse { results: vec![] };
        };

        let Ok(ws_payroll_settings_history) = self
            .payroll_repo
            .get_payroll_settings_history(workspace_id)
            .await
        else {
            return crate::models::BatchCalculatePayrollResponse { results: vec![] };
        };

        let mut results = Vec::new();
        for chunk in employment_ids.chunks(50) {
            let chunk_employment_ids = chunk.to_vec();

            let effective_time = start_date
                .and_hms_opt(0, 0, 0)
                .unwrap_or_default()
                .and_utc();

            // Resolve employments → employee mapping
            let employments_history = self
                .employment_repo
                .get_employments_by_ids(token, &chunk_employment_ids)
                .await
                .unwrap_or_default();

            let mut employment_to_employee: HashMap<Uuid, Uuid> = HashMap::new();
            let mut employee_ids_set = std::collections::BTreeSet::new();
            for emp_ment in &employments_history {
                employment_to_employee.insert(emp_ment.id(), emp_ment.employee_id());
                employee_ids_set.insert(emp_ment.employee_id());
            }
            let chunk_employee_ids: Vec<Uuid> = employee_ids_set.into_iter().collect();

            let salary_histories = self
                .payroll_repo
                .get_salary_components_history_batch(
                    workspace_id,
                    &chunk_employment_ids,
                    start_date,
                    end_date,
                )
                .await;

            let dept_histories = self
                .department_repo
                .get_employee_department_history_batch(token, &chunk_employee_ids)
                .await;

            let time_entries = self
                .time_entry_repo
                .get_workspace_time_entries_batch_by_employments(
                    token,
                    &chunk_employment_ids,
                    start_dt,
                    end_dt,
                )
                .await;

            let employees_res = self
                .employee_repo
                .get_employees_by_ids(&chunk_employee_ids, token)
                .await;

            let rosters = self
                .roster_repo
                .get_employment_rosters_batch(token, &chunk_employment_ids, start_date, end_date)
                .await;

            let conditions_res = self
                .employee_repo
                .get_working_conditions_batch_by_employments(
                    token,
                    &chunk_employment_ids,
                    start_date,
                    end_date,
                )
                .await;

            let policies_res = self
                .policy_repo
                .get_employee_effective_policies_batch(token, &chunk_employee_ids, effective_time)
                .await;

            let contracts_res = self
                .contract_repo
                .list_contracts_batch(token, &chunk_employee_ids)
                .await;

            let assignments_res = self
                .employee_repo
                .list_assignments_batch(token, &chunk_employment_ids)
                .await;

            let mut employees = match employees_res {
                Ok(emps) => emps
                    .into_iter()
                    .map(|e| (e.id(), e))
                    .collect::<HashMap<_, _>>(),
                Err(_) => continue,
            };

            let mut salary_map = salary_histories.unwrap_or_default();
            let mut dept_map = dept_histories.unwrap_or_default();

            let mut time_map = HashMap::new();
            if let Ok(entries) = time_entries {
                for (employment_id, list) in entries {
                    time_map.insert(employment_id, list);
                }
            }

            let mut roster_map = HashMap::new();
            if let Ok(r) = rosters {
                for (employment_id, list) in r {
                    roster_map.insert(employment_id, list);
                }
            }

            let mut conditions_map = HashMap::new();
            if let Ok(c) = conditions_res {
                for (employment_id, list) in c {
                    conditions_map.insert(employment_id, list);
                }
            }

            let mut policies_map = policies_res.unwrap_or_default();
            let mut contracts_map = contracts_res.unwrap_or_default();
            let mut assignments_map = assignments_res.unwrap_or_default();

            let batch_dept_ids: Vec<Uuid> = dept_map
                .values()
                .flatten()
                .map(crate::models::EmployeeDepartmentHistory::department_id)
                .collect::<BTreeSet<_>>()
                .into_iter()
                .collect();

            let batch_dept_payroll_configs = self
                .department_repo
                .get_department_payroll_configs_batch(token, &batch_dept_ids)
                .await
                .unwrap_or_default();

            let mut emp_configs_map: HashMap<Uuid, Vec<crate::models::EmployeePayrollConfig>> =
                self.employee_repo
                    .get_employee_payroll_configs_batch(token, &chunk_employment_ids)
                    .await
                    .unwrap_or_default();

            let mut emp_to_location = HashMap::new();
            let mut emp_to_employments = HashMap::new();
            for emp_ment in &employments_history {
                if emp_ment.status() == "active"
                    && emp_ment.start_date() <= start_date
                    && emp_ment.end_date().is_none_or(|end| end >= start_date)
                {
                    if let Some(loc_id) = emp_ment.primary_location_id() {
                        emp_to_location.insert(emp_ment.employee_id(), loc_id);
                    }
                }
                emp_to_employments
                    .entry(emp_ment.employee_id())
                    .or_insert_with(Vec::new)
                    .push(emp_ment.clone());
            }

            let mut contexts = Vec::new();
            for employment_id in &chunk_employment_ids {
                let emp_id = match employment_to_employee.get(employment_id) {
                    Some(id) => *id,
                    None => continue,
                };
                if let Some(employee) = employees.remove(&emp_id) {
                    let mut daily_entries_map: HashMap<NaiveDate, Vec<crate::models::TimeEntry>> =
                        HashMap::new();
                    if let Some(entries) = time_map.remove(employment_id) {
                        for entry in entries {
                            let local_date = entry.punch_in_time().with_timezone(&tz).date_naive();
                            daily_entries_map.entry(local_date).or_default().push(entry);
                        }
                    }

                    let emp_rosters = roster_map.remove(employment_id).unwrap_or_default();
                    let mut payroll_roster_map = HashMap::new();
                    for r in emp_rosters {
                        payroll_roster_map.insert((r.employment_id, r.date), r.shift_id);
                    }

                    let working_conditions = conditions_map.remove(employment_id).unwrap_or_default();
                    let attendance_policy = match policies_map.remove(&emp_id) {
                        Some(pv) => pv.rules,
                        None => crate::models::AttendanceRules {
                            work_unit_minutes: 30,
                            rounding_policy: "floor".to_string(),
                            early_arrival_strategy: crate::models::EarlyArrivalStrategy::Block,
                        },
                    };

                    let emp_payroll_configs = emp_configs_map.remove(employment_id).unwrap_or_default();
                    let all_dept_history = dept_map.remove(&emp_id).unwrap_or_default();
                    let mut emp_dept_payroll_configs = HashMap::new();
                    for dh in &all_dept_history {
                        if let Some(configs) = batch_dept_payroll_configs.get(&dh.department_id()) {
                            emp_dept_payroll_configs.insert(dh.department_id(), configs.clone());
                        }
                    }

                    let primary_location_id = emp_to_location
                        .get(&emp_id)
                        .copied()
                        .unwrap_or_else(Uuid::new_v4);

                    let salary_history =
                        salary_map.remove(employment_id).unwrap_or_default();

                    contexts.push((*employment_id, PayrollContext {
                        employee_id: emp_id,
                        workspace: workspace.clone(),
                        tz,
                        start_date,
                        end_date,
                        salary_history,
                        all_dept_history,
                        daily_entries_map,
                        holidays_map: holidays_map.clone(),
                        roster_data: Arc::clone(&roster_data),
                        emp_id_number: employee.employee_id_number().to_string(),
                        emp_tags: Some(employee.tags().to_vec()),
                        payroll_roster_map,
                        working_conditions,
                        attendance_policy,
                        emp_payroll_configs,
                        ws_payroll_settings_history: ws_payroll_settings_history.clone(),
                        department_payroll_configs: emp_dept_payroll_configs,
                        overtime_data: overtime_data.clone(),
                        active_contracts: contracts_map.remove(&emp_id).unwrap_or_default(),
                        active_assignments: assignments_map.remove(employment_id).unwrap_or_default(),
                        active_employments: emp_to_employments.remove(&emp_id).unwrap_or_default(),
                        primary_location_id,
                    }));
                }
            }

            let chunk_results: Vec<_> = stream::iter(contexts)
                .map(|(employment_id, ctx)| {
                    let emp_id = ctx.employee_id;
                    let svc = Self {
                        payroll_repo: Arc::clone(&self.payroll_repo),
                        department_repo: Arc::clone(&self.department_repo),
                        time_entry_repo: Arc::clone(&self.time_entry_repo),
                        workspace_repo: Arc::clone(&self.workspace_repo),
                        holiday_repo: Arc::clone(&self.holiday_repo),
                        roster_repo: Arc::clone(&self.roster_repo),
                        leave_repo: Arc::clone(&self.leave_repo),
                        policy_repo: Arc::clone(&self.policy_repo),
                        employee_repo: Arc::clone(&self.employee_repo),
                        overtime_policy_repo: Arc::clone(&self.overtime_policy_repo),
                        salary_base_repo: Arc::clone(&self.salary_base_repo),
                        roster_provider: Arc::clone(&self.roster_provider),
                        contract_repo: Arc::clone(&self.contract_repo),
                        employment_repo: Arc::clone(&self.employment_repo),
                    };
                    async move {
                        match svc.calculate_payslip_with_context(ctx).await {
                            Ok(payslip) => crate::models::BatchCalculateResult {
                                employment_id,
                                employee_id: emp_id,
                                status: "success".to_string(),
                                payslip: Some(payslip),
                                error_message: None,
                            },
                            Err(status) => crate::models::BatchCalculateResult {
                                employment_id,
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

            results.extend(chunk_results);
        }

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

    fn make_salary_entry(
        effective_date: NaiveDate,
    ) -> (
        crate::models::EmployeeSalaryComponent,
        crate::models::SalaryComponentDefinition,
    ) {
        (
            crate::models::EmployeeSalaryComponent {
                id: Uuid::new_v4(),
                employment_id: Uuid::new_v4(),
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
                created_at: Utc::now(),
                updated_at: Utc::now(),
            },
        )
    }

    fn make_dept_entry(effective_date: NaiveDate) -> crate::models::EmployeeDepartmentHistory {
        crate::models::EmployeeDepartmentHistory::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            effective_date,
            None,
            Utc::now(),
        )
    }

    fn make_ctx(
        start_date: NaiveDate,
        end_date: NaiveDate,
        salary_history: Vec<(
            crate::models::EmployeeSalaryComponent,
            crate::models::SalaryComponentDefinition,
        )>,
        all_dept_history: Vec<crate::models::EmployeeDepartmentHistory>,
    ) -> PayrollContext {
        PayrollContext {
            employee_id: Uuid::new_v4(),
            workspace: crate::models::Workspace::new(
                Uuid::new_v4(),
                "Test".into(),
                "YYYY-MM-DD".into(),
                "HH:mm".into(),
                "UTC".into(),
                "#000".into(),
                "en".into(),
                None,
                None,
                "split".into(),
                Decimal::from(2),
                Utc::now(),
                None,
            ),
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
                active_rules: Vec::new(),
                rule_patterns: HashMap::new(),
                dept_history: HashMap::new(),
            }),
            emp_id_number: "EMP001".into(),
            emp_tags: None,
            payroll_roster_map: HashMap::new(),
            working_conditions: Vec::new(),
            attendance_policy: crate::models::AttendanceRules {
                work_unit_minutes: 30,
                rounding_policy: "floor".to_string(),
                early_arrival_strategy: crate::models::EarlyArrivalStrategy::Block,
            },
            emp_payroll_configs: Vec::new(),
            ws_payroll_settings_history: Vec::new(),
            department_payroll_configs: HashMap::new(),
            overtime_data: crate::repositories::overtime_policy_repository::WorkspaceOvertimeData {
                policies: HashMap::new(),
                assignments: Vec::new(),
            },
            active_contracts: Vec::new(),
            active_assignments: Vec::new(),
            active_employments: Vec::new(),
            primary_location_id: Uuid::new_v4(),
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
            d(2026, 3, 1),
            d(2026, 3, 31),
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
            d(2026, 3, 1),
            d(2026, 3, 31),
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
            d(2026, 3, 1),
            d(2026, 3, 31),
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
            d(2026, 3, 1),
            d(2026, 3, 31),
            vec![make_salary_entry(d(2026, 3, 1))],
            vec![],
        );
        let segments = PayrollService::build_segments(&ctx);
        assert_eq!(segments.len(), 1); // change on start_date doesn't create extra segment
    }

    #[test]
    fn test_build_segments_change_after_end_ignored() {
        let ctx = make_ctx(
            d(2026, 3, 1),
            d(2026, 3, 31),
            vec![make_salary_entry(d(2026, 4, 5))],
            vec![],
        );
        let segments = PayrollService::build_segments(&ctx);
        assert_eq!(segments.len(), 1); // change outside range skipped
    }

    #[test]
    fn test_build_segments_same_day_salary_and_dept() {
        let ctx = make_ctx(
            d(2026, 3, 1),
            d(2026, 3, 31),
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
        let rate =
            FinancialCalculator::calculate_daily_rate(Decimal::from(26_000), Decimal::from(26));
        assert_eq!(rate, Decimal::from(1_000));
    }

    #[test]
    fn test_daily_rate_zero_working_days_returns_zero() {
        // Edge case: zero working days should not panic/divide-by-zero
        let rate = FinancialCalculator::calculate_daily_rate(Decimal::from(30_000), Decimal::ZERO);
        assert_eq!(rate, Decimal::ZERO);
    }

    #[test]
    fn test_daily_rate_zero_base_returns_zero() {
        let rate = FinancialCalculator::calculate_daily_rate(Decimal::ZERO, Decimal::from(26));
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
            40, // exactly at limit
            Decimal::from(8_000),
            Decimal::from(20), // 20h * 60 / 30 = 40
            30,
        );
        assert_eq!(units, 40); // no reduction
        assert_eq!(pay, Decimal::from(8_000));
    }

    #[test]
    fn test_ot_cap_zero_units() {
        let (units, pay) =
            FinancialCalculator::apply_ot_cap(0, Decimal::ZERO, Decimal::from(20), 30);
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

        let daily_rate =
            FinancialCalculator::calculate_daily_rate(snap.total_base, snap.std_working_days);
        assert_eq!(daily_rate, Decimal::from(1_000));
    }
}
