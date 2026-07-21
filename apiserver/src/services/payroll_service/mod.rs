use crate::errors::{AppError, AppResult};
use crate::utils::i18n::I18nKey;

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
use crate::repositories::roster::RosterRepository;
use crate::repositories::workspace_repository::WorkspaceRepository;

use super::attendance_matcher::AttendanceMatcher;
use super::financial_calculator::{
    FinancialCalculator, IncentivePolicyWithTiers, RuleAssignmentWithTiers,
};
use super::PolicyResolverService;

pub mod context;
pub mod segments;
use context::{
    build_daily_entries_map, default_attendance_rules, month_boundaries, ChunkSharedData,
};

pub use context::{DirectRosterDataProvider, PayrollContext, RosterDataProvider, SegmentSnapshot};

pub struct PayrollServiceArgs {
    pub payroll_repo: Arc<dyn PayrollRepository>,
    pub department_repo: Arc<dyn DepartmentRepository>,
    pub time_entry_repo: Arc<dyn crate::repositories::time_entry_repository::TimeEntryRepository>,
    pub workspace_repo: Arc<dyn WorkspaceRepository>,
    pub holiday_repo: Arc<dyn HolidayRepository>,
    pub roster_repo: Arc<dyn RosterRepository>,
    pub leave_repo: Arc<dyn LeaveRequestRepository>,
    pub policy_repo: Arc<dyn crate::repositories::AttendancePolicyRepository>,
    pub condition_type_repo: Arc<dyn crate::repositories::ConditionTypeRepository>,
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
    time_entry_repo: Arc<dyn crate::repositories::time_entry_repository::TimeEntryRepository>,
    workspace_repo: Arc<dyn WorkspaceRepository>,
    holiday_repo: Arc<dyn HolidayRepository>,
    roster_repo: Arc<dyn RosterRepository>,
    leave_repo: Arc<dyn LeaveRequestRepository>,
    policy_repo: Arc<dyn crate::repositories::AttendancePolicyRepository>,
    condition_type_repo: Arc<dyn crate::repositories::ConditionTypeRepository>,
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
            condition_type_repo: args.condition_type_repo,
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
            .ok_or_else(|| AppError::not_found(crate::utils::i18n::I18nKey::NotFound))?;

        let tz: Tz = crate::utils::tz::parse_workspace_tz(workspace.timezone());

        let start_date = NaiveDate::from_ymd_opt(year, month, 1)
            .ok_or_else(|| AppError::validation(crate::utils::i18n::I18nKey::Validation))?;
        let next_month_date = if month == 12 {
            NaiveDate::from_ymd_opt(year + 1, 1, 1)
        } else {
            NaiveDate::from_ymd_opt(year, month + 1, 1)
        }
        .ok_or_else(|| AppError::validation(crate::utils::i18n::I18nKey::Validation))?;
        let end_date = next_month_date
            .pred_opt()
            .ok_or_else(|| AppError::Internal("Date calculation error".into()))?;

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
            .ok_or_else(|| AppError::not_found(crate::utils::i18n::I18nKey::EmployeeNotFound))?;

        let active_emp_id = employee
            .active_employment_id()
            .ok_or_else(|| AppError::validation(crate::utils::i18n::I18nKey::NoActiveEmployment))?;

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

        let (rosters, _) = self
            .roster_repo
            .get_employment_rosters(
                &crate::diesel_pool::TenantAuthToken::system_override(workspace_id),
                active_emp_id,
                start_date,
                end_date,
                None,
                None,
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
        let condition_type_catalog = PolicyResolverService::load_condition_type_catalog(
            self.condition_type_repo.as_ref(),
            workspace_id,
        )
        .await?;

        let effective_time = start_date
            .and_hms_opt(0, 0, 0)
            .unwrap_or_default()
            .and_utc();
        let policy_version = self
            .policy_repo
            .get_employment_effective_policy(
                &crate::diesel_pool::TenantAuthToken::system_override(workspace_id),
                active_emp_id,
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
                grace_period_minutes: None,
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
            .ok_or_else(|| AppError::not_found(crate::utils::i18n::I18nKey::NoActiveEmployment))?;

        let primary_location_id = active_employment.primary_location_id().ok_or_else(|| {
            AppError::not_found_args(
                crate::utils::i18n::I18nKey::NotFound,
                vec![("details", "primary_location_not_found".to_string())],
            )
        })?;

        let active_employments = self
            .employment_repo
            .list_employee_employments(&auth_token, employee_id)
            .await?;

        let groups = self
            .salary_base_repo
            .get_groups(workspace_id)
            .await
            .unwrap_or_else(|e| {
                tracing::error!(
                    workspace_id = %workspace_id,
                    error = %e,
                    "fetch_context: get_groups failed (ot_base lookup) — returning empty"
                );
                Vec::new()
            });
        let ot_group_id = groups.iter().find(|g| g.slug == "ot_base").map(|g| g.id);

        let mut ot_base_component_ids = std::collections::HashSet::new();
        if let Some(group_id) = ot_group_id {
            if let Ok(members) = self.salary_base_repo.get_group_members(workspace_id).await {
                for member in members {
                    if member.base_group_id == group_id {
                        ot_base_component_ids.insert(member.component_definition_id);
                    }
                }
            }
        } else {
            // Fallback for tests or unseeded workspaces: treat all components as part of OT base
            for (_, scd) in &salary_history {
                ot_base_component_ids.insert(scd.id);
            }
        }

        // Fetch month end payroll settings
        let month_end_payroll_settings = self
            .payroll_repo
            .get_payroll_settings_at(workspace_id, end_date)
            .await?;

        // Resolve month end attendance policy
        let month_end_attendance_policy = PolicyResolverService::resolve_attendance_policy(
            &*self.policy_repo,
            employee_id,
            workspace_id,
            end_date,
        )
        .await?;

        // Fetch leave breakdown
        let leave_breakdown = self
            .leave_repo
            .get_approved_leave_breakdown(
                employee_id,
                start_date,
                end_date,
                &crate::diesel_pool::TenantAuthToken::system_override(workspace_id),
            )
            .await
            .map_err(|e| {
                tracing::error!(
                    "Failed to fetch leave breakdown for employee {}: {}",
                    employee_id,
                    e
                );
                AppError::Internal(format!("Leave breakdown error (incentives): {e}"))
            })?;

        // Fetch active incentive policies with tiers
        let active_policies = self
            .payroll_repo
            .get_active_incentive_policies_for_period(workspace_id, start_date, end_date)
            .await?;

        let mut active_policies_with_tiers = Vec::new();
        for policy in active_policies {
            let tiers = self
                .payroll_repo
                .get_policy_tiers(workspace_id, policy.id())
                .await?;
            active_policies_with_tiers.push(IncentivePolicyWithTiers { policy, tiers });
        }

        // Fetch salary base groups
        let salary_base_groups = self
            .salary_base_repo
            .get_groups(workspace_id)
            .await
            .unwrap_or_else(|e| {
                tracing::error!(
                    workspace_id = %workspace_id,
                    employee_id = %employee_id,
                    error = %e,
                    "fetch_context: get_groups failed (salary base) — returning empty"
                );
                Vec::new()
            });

        // Fetch unpaid holiday days
        let unpaid_holiday_days: i64 = self
            .leave_repo
            .get_unpaid_holiday_days_without_leave(
                &crate::diesel_pool::TenantAuthToken::system_override(workspace_id),
                employee_id,
                start_date,
                end_date,
            )
            .await
            .unwrap_or_else(|e| {
                tracing::error!(
                    workspace_id = %workspace_id,
                    employee_id = %employee_id,
                    error = %e,
                    "fetch_context: get_unpaid_holiday_days_without_leave failed — defaulting to 0"
                );
                0
            });

        // Fetch last department id
        let last_department_id = all_dept_history
            .iter()
            .filter(|d| d.effective_date() <= end_date)
            .max_by_key(|d| d.effective_date())
            .map(crate::models::EmployeeDepartmentHistory::department_id);

        // Fetch rule assignments with tiers
        let rule_assignments = self
            .payroll_repo
            .get_resolved_rule_assignments(
                workspace_id,
                employee_id,
                last_department_id,
                start_date,
                end_date,
            )
            .await?;

        let mut rule_assignments_with_tiers = Vec::new();
        for assignment in rule_assignments {
            let tiers = self
                .payroll_repo
                .get_payroll_rule_tiers(workspace_id, assignment.rule_definition_id)
                .await?;
            rule_assignments_with_tiers.push(RuleAssignmentWithTiers { assignment, tiers });
        }

        Ok(PayrollContext {
            employee_id,
            workspace,
            tz,
            start_date,
            end_date,
            salary_history,
            all_dept_history,
            daily_entries_map,
            holidays_map: Arc::new(holidays_map),
            roster_data,
            emp_id_number,
            emp_tags,
            payroll_roster_map,
            working_conditions,
            condition_type_catalog: Arc::new(condition_type_catalog),
            attendance_policy,
            emp_payroll_configs,
            ws_payroll_settings_history: Arc::new(ws_payroll_settings_history),
            department_payroll_configs,
            overtime_data: Arc::new(overtime_data),
            ot_base_component_ids,
            active_contracts,
            active_assignments,
            active_employments,
            primary_location_id,
            month_end_payroll_settings: Arc::new(month_end_payroll_settings),
            month_end_attendance_policy,
            leave_breakdown,
            active_policies_with_tiers: Arc::new(active_policies_with_tiers),
            salary_base_groups: Arc::new(salary_base_groups),
            unpaid_holiday_days,
            rule_assignments_with_tiers,
        })
    }

    /// Phase 2: Determine salary/dept change segments.
    /// Delegates to `segments::build_segments`.
    fn build_segments(ctx: &PayrollContext) -> Vec<(NaiveDate, NaiveDate)> {
        segments::build_segments(ctx)
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
            .ok_or_else(|| AppError::not_found(crate::utils::i18n::I18nKey::NoActiveEmployment))?;
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
                    if ctx.ot_base_component_ids.contains(&scd.id) {
                        segment_ot_base += esc.amount;
                    }
                    processed_def_ids.insert(scd.id);
                }
            }

            let _rules = PolicyResolver::resolve_with_shift_overrides_pure(
                Some(ctx.attendance_policy.clone()),
                ctx.working_conditions.clone(),
                seg_start,
                Some(&ctx.condition_type_catalog),
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
        if let Some(ref ps) = *ctx.month_end_payroll_settings {
            if let Some(max_monthly_hours) = ps.max_ot_hours_per_month {
                let month_end_policy = ctx.month_end_attendance_policy.clone();

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

        let leave_breakdown = ctx.leave_breakdown;
        let active_policies_with_tiers = Arc::clone(&ctx.active_policies_with_tiers);

        // Phase 5: Incentives — delegated to FinancialCalculator
        let incentives = FinancialCalculator::calculate_incentives(
            &active_policies_with_tiers,
            &leave_breakdown,
            crate::services::financial_calculator::CalculateIncentivesArgs {
                workspace_id,
                employee_id,
                total_std_units,
                total_late_early_count,
                start_date: ctx.start_date,
                end_date: ctx.end_date,
            },
        )?;

        // Phase 6: Deductions — delegated to FinancialCalculator
        let gross_salary = total_base_pay
            + total_overtime_pay
            + total_holiday_work_premium
            + incentives.total_amount;
        let last_department_id = segment_snapshots.last().and_then(|s| s.department_id);

        let mut base_values = HashMap::new();
        base_values.insert("gross".to_string(), gross_salary);
        base_values.insert("base_pay".to_string(), total_base_pay);

        for group in ctx.salary_base_groups.iter() {
            let amount = if group.slug == "bhxh_base" {
                total_base_pay
            } else {
                gross_salary
            };
            base_values.insert(group.slug.clone(), amount);
        }

        let unpaid_holiday_days = ctx.unpaid_holiday_days;
        let rule_assignments_with_tiers = ctx.rule_assignments_with_tiers.clone();

        let deductions = FinancialCalculator::calculate_deductions(
            &leave_breakdown,
            unpaid_holiday_days,
            &rule_assignments_with_tiers,
            crate::services::financial_calculator::CalculateDeductionsArgs {
                workspace_id,
                employee_id,
                department_id: last_department_id,
                start_date: ctx.start_date,
                end_date: ctx.end_date,
                base_values: &base_values,
                segment_snapshots: &segment_snapshots,
            },
        )?;

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
        let workspace_id = payslip.workspace_id();

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
    ) -> AppResult<crate::models::BatchCalculatePayrollResponse> {
        // ── 0. Input validation ──────────────────────────────────────
        if employment_ids.is_empty() {
            return Err(AppError::validation(I18nKey::Validation));
        }
        const MAX_BATCH_EMPLOYEES: usize = 500;
        if employment_ids.len() > MAX_BATCH_EMPLOYEES {
            return Err(AppError::validation_args(
                I18nKey::Validation,
                vec![(
                    "details",
                    format!(
                        "Batch size {} exceeds maximum {MAX_BATCH_EMPLOYEES}",
                        employment_ids.len()
                    ),
                )],
            ));
        }

        // Deduplicate but preserve order for deterministic results
        let mut seen = std::collections::HashSet::new();
        let mut deduped_ids = Vec::with_capacity(employment_ids.len());
        for id in employment_ids {
            if seen.insert(id) {
                deduped_ids.push(id);
            }
        }
        let employment_ids = deduped_ids;

        // ── Phase 1: Workspace-level pre-fetch ──────────────────────
        let workspace = self
            .workspace_repo
            .get_workspace_by_id(workspace_id)
            .await?
            .ok_or_else(|| AppError::not_found(I18nKey::WorkspaceNotFound))?;
        let tz: Tz = crate::utils::tz::parse_workspace_tz(workspace.timezone());

        let (start_date, end_date) = month_boundaries(year, month)?;
        let start_dt = local_to_utc(&tz, start_date, 0, 0, 0)
            .map_err(|e| AppError::Internal(format!("Failed to convert start date to UTC: {e}")))?;
        let end_dt = local_to_utc(&tz, end_date, 23, 59, 59)
            .map_err(|e| AppError::Internal(format!("Failed to convert end date to UTC: {e}")))?;
        let token = &crate::diesel_pool::TenantAuthToken::system_override(workspace_id);

        let condition_type_catalog = PolicyResolverService::load_condition_type_catalog(
            self.condition_type_repo.as_ref(),
            workspace_id,
        )
        .await
        .map_err(|e| AppError::Internal(format!("Failed to fetch condition type catalog: {e}")))?;

        let overtime_data = self.fetch_overtime_data(workspace_id).await;
        let holidays_map = self
            .fetch_holidays(token, start_date, end_date, workspace_id)
            .await;

        let roster_data = self
            .roster_provider
            .get_roster_data(workspace_id)
            .await
            .map_err(|e| AppError::Internal(format!("Failed to fetch roster data: {e}")))?;

        let ws_payroll_settings_history = self
            .payroll_repo
            .get_payroll_settings_history(workspace_id)
            .await
            .map_err(|e| {
                AppError::Internal(format!("Failed to fetch payroll settings history: {e}"))
            })?;

        // month-end payroll settings are optional — absence just means no OT cap
        let month_end_payroll_settings = self
            .payroll_repo
            .get_payroll_settings_at(workspace_id, end_date)
            .await
            .unwrap_or_else(|e| {
                tracing::warn!(workspace_id = %workspace_id, error = %e, "Failed to fetch month end payroll settings, continuing without OT cap");
                None
            });

        let active_policies_with_tiers = self
            .fetch_incentive_policies_with_tiers(workspace_id, start_date, end_date)
            .await;

        let salary_base_groups = self
            .salary_base_repo
            .get_groups(workspace_id)
            .await
            .unwrap_or_else(|e| {
                tracing::warn!(
                    workspace_id = %workspace_id,
                    year = year,
                    month = month,
                    error = %e,
                    "calculate_batch_payslips: get_groups failed — continuing without salary base groups"
                );
                Vec::new()
            });

        // ── Phase 2: Process chunks ─────────────────────────────────
        let shared_data = Arc::new(ChunkSharedData {
            workspace: workspace.clone(),
            tz,
            start_date,
            end_date,
            start_dt,
            end_dt,
            token: *token,
            condition_type_catalog: Arc::new(condition_type_catalog),
            overtime_data: Arc::new(overtime_data),
            holidays_map: Arc::new(holidays_map),
            roster_data: Arc::clone(&roster_data),
            ws_payroll_settings_history: Arc::new(ws_payroll_settings_history),
            month_end_payroll_settings: Arc::new(month_end_payroll_settings),
            active_policies_with_tiers: Arc::new(active_policies_with_tiers),
            salary_base_groups: Arc::new(salary_base_groups),
        });

        let mut results = Vec::new();
        for chunk in employment_ids.chunks(50) {
            let chunk_results = self
                .process_payroll_chunk(chunk.to_vec(), &shared_data, workspace_id)
                .await;
            results.extend(chunk_results);
        }

        Ok(crate::models::BatchCalculatePayrollResponse { results })
    }

    /// Process a single chunk of employment IDs for batch payroll calculation.
    #[allow(clippy::too_many_lines)]
    async fn process_payroll_chunk(
        &self,
        chunk_employment_ids: Vec<Uuid>,
        d: &Arc<ChunkSharedData>,
        workspace_id: Uuid,
    ) -> Vec<crate::models::BatchCalculateResult> {
        const MAX_CONCURRENT_PAYROLL: usize = 10;

        let effective_time = d
            .start_date
            .and_hms_opt(0, 0, 0)
            .unwrap_or_default()
            .and_utc();

        // Resolve employments → employee mapping
        let employments_history = self
            .employment_repo
            .get_employments_by_ids(&d.token, &chunk_employment_ids)
            .await
            .unwrap_or_else(|e| {
                tracing::error!(
                    workspace_id = %workspace_id,
                    chunk_size = chunk_employment_ids.len(),
                    error = %e,
                    "Batch payroll: get_employments_by_ids failed — chunk will have per-item errors"
                );
                Vec::new()
            });

        let mut employment_to_employee: HashMap<Uuid, Uuid> = HashMap::new();
        let mut employee_ids_set = std::collections::BTreeSet::new();
        for emp_ment in &employments_history {
            employment_to_employee.insert(emp_ment.id(), emp_ment.employee_id());
            employee_ids_set.insert(emp_ment.employee_id());
        }
        let chunk_employee_ids: Vec<Uuid> = employee_ids_set.into_iter().collect();

        // Batch fetch all chunk-level data
        let salary_histories = self
            .payroll_repo
            .get_salary_components_history_batch(
                workspace_id,
                &chunk_employment_ids,
                d.start_date,
                d.end_date,
            )
            .await;

        let dept_histories = self
            .department_repo
            .get_employee_department_history_batch(&d.token, &chunk_employee_ids)
            .await;

        let time_entries = self
            .time_entry_repo
            .get_workspace_time_entries_batch_by_employments(
                &d.token,
                &chunk_employment_ids,
                d.start_dt,
                d.end_dt,
            )
            .await;

        let employees_res = self
            .employee_repo
            .get_employees_by_ids(&chunk_employee_ids, &d.token)
            .await;

        let rosters = self
            .roster_repo
            .get_employment_rosters_batch(&d.token, &chunk_employment_ids, d.start_date, d.end_date)
            .await;

        let conditions_res = self
            .employee_repo
            .get_working_conditions_batch_by_employments(
                &d.token,
                &chunk_employment_ids,
                d.start_date,
                d.end_date,
            )
            .await;

        let policies_res = self
            .policy_repo
            .get_employment_effective_policies_batch(
                &d.token,
                &chunk_employment_ids,
                effective_time,
            )
            .await;

        let contracts_res = self
            .contract_repo
            .list_contracts_batch(&d.token, &chunk_employee_ids)
            .await;

        let assignments_res = self
            .employee_repo
            .list_assignments_batch(&d.token, &chunk_employment_ids)
            .await;

        let end_effective_time = d
            .end_date
            .and_hms_opt(23, 59, 59)
            .unwrap_or_default()
            .and_utc();
        let month_end_policies_res = self
            .policy_repo
            .get_employment_effective_policies_batch(
                &d.token,
                &chunk_employment_ids,
                end_effective_time,
            )
            .await;

        let mut employees = match employees_res {
            Ok(emps) => emps
                .into_iter()
                .map(|e| (e.id(), e))
                .collect::<HashMap<_, _>>(),
            Err(e) => {
                tracing::error!(
                    workspace_id = %workspace_id,
                    chunk_size = chunk_employment_ids.len(),
                    error = %e,
                    "Batch payroll: get_employees_by_ids failed — marking all chunk employments as errors"
                );
                let mut error_results = Vec::with_capacity(chunk_employment_ids.len());
                for &employment_id in &chunk_employment_ids {
                    let emp_id = employment_to_employee
                        .get(&employment_id)
                        .copied()
                        .unwrap_or(Uuid::nil());
                    error_results.push(crate::models::BatchCalculateResult {
                        employment_id,
                        employee_id: emp_id,
                        status: "error".to_string(),
                        payslip: None,
                        error_message: Some(format!("Failed to fetch employee data: {e}")),
                    });
                }
                return error_results;
            }
        };

        // Ensure every employment_id in the chunk has a mapping.
        // If employment lookup failed for some IDs, create error results.
        let mut pre_errors = Vec::new();
        for &employment_id in &chunk_employment_ids {
            if !employment_to_employee.contains_key(&employment_id) {
                pre_errors.push(crate::models::BatchCalculateResult {
                    employment_id,
                    employee_id: Uuid::nil(),
                    status: "error".to_string(),
                    payslip: None,
                    error_message: Some("Employment not found in workspace".to_string()),
                });
            }
        }
        // Remove employment_ids with errors from further processing
        let chunk_employment_ids: Vec<Uuid> = chunk_employment_ids
            .into_iter()
            .filter(|id| employment_to_employee.contains_key(id))
            .collect();

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
        let mut month_end_policies_map = month_end_policies_res.unwrap_or_default();
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
            .get_department_payroll_configs_batch(&d.token, &batch_dept_ids)
            .await
            .unwrap_or_else(|e| {
                tracing::warn!(
                    workspace_id = %workspace_id,
                    dept_count = batch_dept_ids.len(),
                    error = %e,
                    "Batch payroll: get_department_payroll_configs_batch failed — continuing without department configs"
                );
                HashMap::new()
            });

        let mut emp_configs_map: HashMap<Uuid, Vec<crate::models::EmployeePayrollConfig>> = self
            .employee_repo
            .get_employee_payroll_configs_batch(&d.token, &chunk_employment_ids)
            .await
            .unwrap_or_else(|e| {
                tracing::warn!(
                    workspace_id = %workspace_id,
                    chunk_size = chunk_employment_ids.len(),
                    error = %e,
                    "Batch payroll: get_employee_payroll_configs_batch failed — continuing without employee configs"
                );
                HashMap::new()
            });

        let mut emp_to_location = HashMap::new();
        let mut emp_to_employments = HashMap::new();
        for emp_ment in &employments_history {
            if emp_ment.status() == "active"
                && emp_ment.start_date() <= d.start_date
                && emp_ment.end_date().is_none_or(|end| end >= d.start_date)
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

        let ot_base_component_ids = self
            .resolve_ot_base_component_ids(workspace_id, &d.salary_base_groups)
            .await;

        let mut leave_breakdowns_map = self
            .leave_repo
            .get_approved_leave_breakdowns_batch(
                &chunk_employee_ids,
                d.start_date,
                d.end_date,
                &d.token,
            )
            .await
            .unwrap_or_else(|e| {
                tracing::warn!(
                    workspace_id = %workspace_id,
                    employee_count = chunk_employee_ids.len(),
                    error = %e,
                    "Batch payroll: get_approved_leave_breakdowns_batch failed — continuing without leave data"
                );
                HashMap::new()
            });

        let mut unpaid_holidays_map = self
            .leave_repo
            .get_unpaid_holiday_days_without_leave_batch(
                &d.token,
                &chunk_employee_ids,
                d.start_date,
                d.end_date,
            )
            .await
            .unwrap_or_else(|e| {
                tracing::warn!(
                    workspace_id = %workspace_id,
                    employee_count = chunk_employee_ids.len(),
                    error = %e,
                    "Batch payroll: get_unpaid_holiday_days_without_leave_batch failed — continuing without unpaid holiday data"
                );
                HashMap::new()
            });

        // Map employee_id -> last_department_id
        let mut employee_departments = HashMap::new();
        for employment_id in &chunk_employment_ids {
            if let Some(emp_id) = employment_to_employee.get(employment_id) {
                let all_dept_history = dept_map.get(emp_id).cloned().unwrap_or_default();
                let last_department_id = all_dept_history
                    .iter()
                    .filter(|dh| dh.effective_date() <= d.end_date)
                    .max_by_key(|dh| dh.effective_date())
                    .map(crate::models::EmployeeDepartmentHistory::department_id);
                employee_departments.insert(*emp_id, last_department_id);
            }
        }

        let rule_assignments_map = self
            .build_rule_assignments_map(
                workspace_id,
                &chunk_employment_ids,
                &employment_to_employee,
                &employee_departments,
                d.start_date,
                d.end_date,
            )
            .await;

        // Build PayrollContexts — split into successes and pre-built error results
        let context_results = build_payroll_contexts(
            &chunk_employment_ids,
            &employment_to_employee,
            &mut employees,
            &mut time_map,
            &mut roster_map,
            &mut conditions_map,
            &mut policies_map,
            &mut emp_configs_map,
            &mut dept_map,
            &batch_dept_payroll_configs,
            &emp_to_location,
            &emp_to_employments,
            &mut salary_map,
            &mut leave_breakdowns_map,
            &mut unpaid_holidays_map,
            &rule_assignments_map,
            &mut month_end_policies_map,
            &ot_base_component_ids,
            &mut contracts_map,
            &mut assignments_map,
            d,
        );

        let mut results: Vec<crate::models::BatchCalculateResult> =
            Vec::with_capacity(chunk_employment_ids.len() + pre_errors.len());
        results.extend(pre_errors);
        let mut ok_contexts: Vec<(Uuid, PayrollContext)> = Vec::new();

        for (employment_id, result) in context_results {
            match result {
                Ok(ctx) => ok_contexts.push((employment_id, ctx)),
                Err(error_result) => results.push(error_result),
            }
        }

        // Run concurrent calculations for successfully built contexts
        let calculated: Vec<crate::models::BatchCalculateResult> = stream::iter(ok_contexts)
            .map(|(employment_id, ctx)| {
                let emp_id = ctx.employee_id;
                let svc = self.clone_service();
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
                            error_message: Some(format!("Calculation error: {status}")),
                        },
                    }
                }
            })
            .buffer_unordered(MAX_CONCURRENT_PAYROLL)
            .collect()
            .await;

        results.extend(calculated);
        results
    }

    async fn fetch_overtime_data(
        &self,
        workspace_id: Uuid,
    ) -> crate::repositories::overtime_policy_repository::WorkspaceOvertimeData {
        match self
            .overtime_policy_repo
            .get_workspace_overtime_data(workspace_id)
            .await
        {
            Ok(data) => data,
            Err(e) => {
                tracing::error!(workspace_id = %workspace_id, error = %e, "Failed to fetch overtime data — proceeding empty");
                crate::repositories::overtime_policy_repository::WorkspaceOvertimeData {
                    policies: std::collections::HashMap::new(),
                    assignments: Vec::new(),
                }
            }
        }
    }

    async fn fetch_holidays(
        &self,
        token: &crate::diesel_pool::TenantAuthToken,
        start_date: NaiveDate,
        end_date: NaiveDate,
        workspace_id: Uuid,
    ) -> HashMap<NaiveDate, crate::models::Holiday> {
        match self
            .holiday_repo
            .get_holidays_in_range(token, start_date, end_date)
            .await
        {
            Ok(map) => map,
            Err(e) => {
                tracing::error!(workspace_id = %workspace_id, error = %e, "Failed to fetch holidays — proceeding empty");
                std::collections::HashMap::new()
            }
        }
    }

    async fn fetch_incentive_policies_with_tiers(
        &self,
        workspace_id: Uuid,
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> Vec<IncentivePolicyWithTiers> {
        let active_policies = match self
            .payroll_repo
            .get_active_incentive_policies_for_period(workspace_id, start_date, end_date)
            .await
        {
            Ok(policies) => policies,
            Err(e) => {
                tracing::error!(workspace_id = %workspace_id, error = %e, "Failed to fetch incentive policies");
                return Vec::new();
            }
        };
        let mut result = Vec::new();
        for policy in active_policies {
            if let Ok(tiers) = self
                .payroll_repo
                .get_policy_tiers(workspace_id, policy.id())
                .await
            {
                result.push(IncentivePolicyWithTiers { policy, tiers });
            }
        }
        result
    }

    async fn resolve_ot_base_component_ids(
        &self,
        workspace_id: Uuid,
        salary_base_groups: &[crate::models::SalaryBaseGroup],
    ) -> std::collections::HashSet<Uuid> {
        let ot_group_id = salary_base_groups
            .iter()
            .find(|g| g.slug == "ot_base")
            .map(|g| g.id);
        let mut ids = std::collections::HashSet::new();
        if let Some(group_id) = ot_group_id {
            if let Ok(members) = self.salary_base_repo.get_group_members(workspace_id).await {
                for member in members {
                    if member.base_group_id == group_id {
                        ids.insert(member.component_definition_id);
                    }
                }
            }
        }
        ids
    }

    #[allow(clippy::too_many_arguments)]
    async fn build_rule_assignments_map(
        &self,
        workspace_id: Uuid,
        chunk_employment_ids: &[Uuid],
        employment_to_employee: &HashMap<Uuid, Uuid>,
        employee_departments: &HashMap<Uuid, Option<Uuid>>,
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> HashMap<Uuid, Vec<RuleAssignmentWithTiers>> {
        let resolved_assignments_map = self
            .payroll_repo
            .get_resolved_rule_assignments_batch(
                workspace_id,
                &employee_departments.keys().copied().collect::<Vec<_>>(),
                employee_departments,
                start_date,
                end_date,
            )
            .await
            .unwrap_or_else(|e| {
                tracing::error!(
                    workspace_id = %workspace_id,
                    employee_count = employee_departments.len(),
                    error = %e,
                    "build_rule_assignments_map: get_resolved_rule_assignments_batch failed — returning empty"
                );
                HashMap::new()
            });

        let mut rule_def_ids: Vec<Uuid> = resolved_assignments_map
            .values()
            .flat_map(|assignments| assignments.iter().map(|a| a.rule_definition_id))
            .collect();
        rule_def_ids.sort();
        rule_def_ids.dedup();

        let rule_tiers_map = self
            .payroll_repo
            .get_payroll_rule_tiers_batch(workspace_id, &rule_def_ids)
            .await
            .unwrap_or_else(|e| {
                tracing::error!(
                    workspace_id = %workspace_id,
                    rule_def_count = rule_def_ids.len(),
                    error = %e,
                    "build_rule_assignments_map: get_payroll_rule_tiers_batch failed — returning empty"
                );
                HashMap::new()
            });

        let mut rule_assignments_map = HashMap::new();
        for employment_id in chunk_employment_ids {
            if let Some(emp_id) = employment_to_employee.get(employment_id) {
                if let Some(assignments) = resolved_assignments_map.get(emp_id) {
                    for assignment in assignments {
                        let tiers = rule_tiers_map
                            .get(&assignment.rule_definition_id)
                            .cloned()
                            .unwrap_or_default();
                        rule_assignments_map
                            .entry(*employment_id)
                            .or_insert_with(Vec::new)
                            .push(RuleAssignmentWithTiers {
                                assignment: assignment.clone(),
                                tiers,
                            });
                    }
                }
            }
        }
        rule_assignments_map
    }

    fn clone_service(&self) -> Self {
        Self {
            payroll_repo: Arc::clone(&self.payroll_repo),
            department_repo: Arc::clone(&self.department_repo),
            time_entry_repo: Arc::clone(&self.time_entry_repo),
            workspace_repo: Arc::clone(&self.workspace_repo),
            holiday_repo: Arc::clone(&self.holiday_repo),
            roster_repo: Arc::clone(&self.roster_repo),
            leave_repo: Arc::clone(&self.leave_repo),
            policy_repo: Arc::clone(&self.policy_repo),
            condition_type_repo: Arc::clone(&self.condition_type_repo),
            employee_repo: Arc::clone(&self.employee_repo),
            overtime_policy_repo: Arc::clone(&self.overtime_policy_repo),
            salary_base_repo: Arc::clone(&self.salary_base_repo),
            roster_provider: Arc::clone(&self.roster_provider),
            contract_repo: Arc::clone(&self.contract_repo),
            employment_repo: Arc::clone(&self.employment_repo),
        }
    }
}

/// Build PayrollContext values for a chunk of employments.
/// Returns a tuple per employment: `(employment_id, Result<context, error_result>)`.
/// Failures are pre-built `BatchCalculateResult` with `status: "error"`.
#[allow(clippy::too_many_arguments, clippy::too_many_lines)]
pub(crate) fn build_payroll_contexts(
    chunk_employment_ids: &[Uuid],
    employment_to_employee: &HashMap<Uuid, Uuid>,
    employees: &mut HashMap<Uuid, crate::models::employee::Employee>,
    time_map: &mut HashMap<Uuid, Vec<crate::models::TimeEntry>>,
    roster_map: &mut HashMap<Uuid, Vec<crate::models::shift::ShiftRoster>>,
    conditions_map: &mut HashMap<Uuid, Vec<crate::models::WorkingCondition>>,
    policies_map: &mut HashMap<Uuid, crate::models::AttendancePolicyVersion>,
    emp_configs_map: &mut HashMap<Uuid, Vec<crate::models::EmployeePayrollConfig>>,
    dept_map: &mut HashMap<Uuid, Vec<crate::models::EmployeeDepartmentHistory>>,
    batch_dept_payroll_configs: &HashMap<Uuid, Vec<crate::models::DepartmentPayrollConfig>>,
    emp_to_location: &HashMap<Uuid, Uuid>,
    emp_to_employments: &HashMap<Uuid, Vec<crate::models::employment::Employment>>,
    salary_map: &mut HashMap<
        Uuid,
        Vec<(
            crate::models::EmployeeSalaryComponent,
            crate::models::SalaryComponentDefinition,
        )>,
    >,
    leave_breakdowns_map: &mut HashMap<
        Uuid,
        crate::repositories::leave_request_repository::LeaveBreakdown,
    >,
    unpaid_holidays_map: &mut HashMap<Uuid, i64>,
    rule_assignments_map: &HashMap<Uuid, Vec<RuleAssignmentWithTiers>>,
    month_end_policies_map: &mut HashMap<Uuid, crate::models::AttendancePolicyVersion>,
    ot_base_component_ids: &std::collections::HashSet<Uuid>,
    contracts_map: &mut HashMap<Uuid, Vec<crate::models::contract::EmployeeContract>>,
    assignments_map: &mut HashMap<
        Uuid,
        Vec<crate::models::employee_assignment::EmployeeAssignment>,
    >,
    d: &Arc<ChunkSharedData>,
) -> Vec<(
    Uuid,
    Result<PayrollContext, crate::models::BatchCalculateResult>,
)> {
    let mut results: Vec<(
        Uuid,
        Result<PayrollContext, crate::models::BatchCalculateResult>,
    )> = Vec::new();
    for employment_id in chunk_employment_ids {
        let emp_id = if let Some(id) = employment_to_employee.get(employment_id) {
            *id
        } else {
            results.push((
                *employment_id,
                Err(crate::models::BatchCalculateResult {
                    employment_id: *employment_id,
                    employee_id: Uuid::nil(),
                    status: "error".to_string(),
                    payslip: None,
                    error_message: Some("Employment has no associated employee record".into()),
                }),
            ));
            continue;
        };
        let Some(employee) = employees.remove(&emp_id) else {
            results.push((
                *employment_id,
                Err(crate::models::BatchCalculateResult {
                    employment_id: *employment_id,
                    employee_id: emp_id,
                    status: "error".to_string(),
                    payslip: None,
                    error_message: Some("Employee data not found in batch fetch results".into()),
                }),
            ));
            continue;
        };
        let daily_entries_map =
            build_daily_entries_map(time_map.remove(employment_id).unwrap_or_default(), &d.tz);

        let payroll_roster_map = roster_map
            .remove(employment_id)
            .unwrap_or_default()
            .into_iter()
            .map(|r| ((r.employment_id, r.date), r.shift_id))
            .collect();

        let working_conditions = conditions_map.remove(employment_id).unwrap_or_default();

        let attendance_policy = policies_map
            .remove(employment_id)
            .map_or_else(default_attendance_rules, |pv| pv.rules);

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

        let salary_history = salary_map.remove(employment_id).unwrap_or_default();
        let ot_base_comp_ids = if ot_base_component_ids.is_empty() {
            salary_history.iter().map(|(_, scd)| scd.id).collect()
        } else {
            ot_base_component_ids.clone()
        };

        let month_end_attendance_policy = month_end_policies_map
            .remove(employment_id)
            .map_or_else(default_attendance_rules, |pv| pv.rules);

        let leave_breakdown = leave_breakdowns_map.remove(&emp_id).unwrap_or(
            crate::repositories::leave_request_repository::LeaveBreakdown {
                total_days: 0.0,
                unpaid_days: 0.0,
                sick_days: 0.0,
            },
        );

        let unpaid_holiday_days = unpaid_holidays_map.remove(&emp_id).unwrap_or(0);
        let rule_assignments_with_tiers = rule_assignments_map
            .get(employment_id)
            .cloned()
            .unwrap_or_default();

        results.push((
            *employment_id,
            Ok(PayrollContext {
                employee_id: emp_id,
                workspace: d.workspace.clone(),
                tz: d.tz,
                start_date: d.start_date,
                end_date: d.end_date,
                salary_history,
                all_dept_history,
                daily_entries_map,
                holidays_map: Arc::clone(&d.holidays_map),
                roster_data: Arc::clone(&d.roster_data),
                emp_id_number: employee.employee_id_number().to_string(),
                emp_tags: Some(employee.tags().to_vec()),
                payroll_roster_map,
                working_conditions,
                condition_type_catalog: Arc::clone(&d.condition_type_catalog),
                attendance_policy,
                emp_payroll_configs,
                ws_payroll_settings_history: Arc::clone(&d.ws_payroll_settings_history),
                department_payroll_configs: emp_dept_payroll_configs,
                overtime_data: Arc::clone(&d.overtime_data),
                ot_base_component_ids: ot_base_comp_ids,
                active_contracts: contracts_map.remove(&emp_id).unwrap_or_default(),
                active_assignments: assignments_map.remove(employment_id).unwrap_or_default(),
                active_employments: emp_to_employments.get(&emp_id).cloned().unwrap_or_default(),
                primary_location_id,
                month_end_payroll_settings: Arc::clone(&d.month_end_payroll_settings),
                month_end_attendance_policy,
                leave_breakdown,
                active_policies_with_tiers: Arc::clone(&d.active_policies_with_tiers),
                salary_base_groups: Arc::clone(&d.salary_base_groups),
                unpaid_holiday_days,
                rule_assignments_with_tiers,
            }),
        ));
    }
    results
}

use crate::utils::chrono_helpers::local_to_utc;

#[cfg(test)]
mod tests {
    use super::*;

    fn d(y: i32, m: u32, day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, day).expect("valid date")
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

    // --- Tests for build_payroll_contexts error paths ---

    fn make_chunk_shared_data() -> Arc<ChunkSharedData> {
        Arc::new(ChunkSharedData {
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
                chrono::Utc::now(),
                None,
            ),
            tz: chrono_tz::UTC,
            start_date: d(2026, 3, 1),
            end_date: d(2026, 3, 31),
            start_dt: chrono::Utc::now(),
            end_dt: chrono::Utc::now(),
            token: crate::diesel_pool::TenantAuthToken::system_override(Uuid::new_v4()),
            condition_type_catalog: Arc::new(HashMap::new()),
            overtime_data: Arc::new(
                crate::repositories::overtime_policy_repository::WorkspaceOvertimeData {
                    policies: HashMap::new(),
                    assignments: Vec::new(),
                },
            ),
            holidays_map: Arc::new(HashMap::new()),
            roster_data: Arc::new(crate::cache::RosterData {
                shifts: HashMap::new(),
                fixed_assignments: HashMap::new(),
                pattern_assignments: HashMap::new(),
                active_rules: Vec::new(),
                rule_patterns: HashMap::new(),
                dept_history: HashMap::new(),
                shift_intervals: HashMap::new(),
            }),
            ws_payroll_settings_history: Arc::new(Vec::new()),
            month_end_payroll_settings: Arc::new(None),
            active_policies_with_tiers: Arc::new(Vec::new()),
            salary_base_groups: Arc::new(Vec::new()),
        })
    }

    #[test]
    fn test_build_contexts_employment_without_employee() {
        // When an employment_id has no mapping in employment_to_employee,
        // it should return Err with a descriptive error message.
        let employment_id = Uuid::new_v4();
        let d = make_chunk_shared_data();

        let results = build_payroll_contexts(
            &[employment_id],
            &HashMap::new(), // empty: no employment→employee mapping
            &mut HashMap::new(),
            &mut HashMap::new(),
            &mut HashMap::new(),
            &mut HashMap::new(),
            &mut HashMap::new(),
            &mut HashMap::new(),
            &mut HashMap::new(),
            &HashMap::new(),
            &HashMap::new(),
            &HashMap::new(),
            &mut HashMap::new(),
            &mut HashMap::new(),
            &mut HashMap::new(),
            &HashMap::new(),
            &mut HashMap::new(),
            &std::collections::HashSet::new(),
            &mut HashMap::new(),
            &mut HashMap::new(),
            &d,
        );

        assert_eq!(results.len(), 1);
        let (eid, result) = &results[0];
        assert_eq!(*eid, employment_id);
        match result {
            Err(error_result) => {
                assert_eq!(error_result.status, "error");
                assert!(error_result
                    .error_message
                    .as_ref()
                    .unwrap()
                    .contains("no associated employee"));
                assert_eq!(error_result.employment_id, employment_id);
                assert!(error_result.payslip.is_none());
            }
            Ok(_) => panic!("Expected Err, got Ok"),
        }
    }

    #[test]
    fn test_build_contexts_employee_data_not_found() {
        // When employee_id exists in mapping but employee data
        // was not fetched (not in employees map), return Err.
        let employment_id = Uuid::new_v4();
        let employee_id = Uuid::new_v4();
        let d = make_chunk_shared_data();

        let mut emp_to_employee = HashMap::new();
        emp_to_employee.insert(employment_id, employee_id);

        let results = build_payroll_contexts(
            &[employment_id],
            &emp_to_employee,    // mapping exists...
            &mut HashMap::new(), // ...but employee data doesn't
            &mut HashMap::new(),
            &mut HashMap::new(),
            &mut HashMap::new(),
            &mut HashMap::new(),
            &mut HashMap::new(),
            &mut HashMap::new(),
            &HashMap::new(),
            &HashMap::new(),
            &HashMap::new(),
            &mut HashMap::new(),
            &mut HashMap::new(),
            &mut HashMap::new(),
            &HashMap::new(),
            &mut HashMap::new(),
            &std::collections::HashSet::new(),
            &mut HashMap::new(),
            &mut HashMap::new(),
            &d,
        );

        assert_eq!(results.len(), 1);
        let (eid, result) = &results[0];
        assert_eq!(*eid, employment_id);
        match result {
            Err(error_result) => {
                assert_eq!(error_result.status, "error");
                assert!(
                    error_result
                        .error_message
                        .as_ref()
                        .unwrap()
                        .contains("not found in batch fetch"),
                    "Expected 'not found in batch fetch', got: {:?}",
                    error_result.error_message
                );
                assert_eq!(error_result.employment_id, employment_id);
                assert_eq!(error_result.employee_id, employee_id);
                assert!(error_result.payslip.is_none());
            }
            Ok(_) => panic!("Expected Err, got Ok"),
        }
    }
}
