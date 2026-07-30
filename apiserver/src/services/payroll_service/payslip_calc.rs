use super::*;
use crate::services::attendance_context::AttendanceMatcherContext;
use rust_decimal::prelude::ToPrimitive;

impl PayrollService {
    #[tracing::instrument(skip_all, err)]
    #[allow(clippy::too_many_lines)]
    pub async fn calculate_payslip_with_context(
        &self,
        ctx: PayrollContext,
    ) -> Result<Payslip, AppError> {
        let employee_id = ctx.employee_id;
        let workspace_id = ctx.workspace.id();

        // Build the attendance-only view once; the segment loop borrows it.
        let att_ctx = AttendanceMatcherContext::from_payroll(&ctx);

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
                    ctx: &att_ctx,
                    seg_start,
                    seg_end,
                    segment_total_base,
                    segment_ot_base,
                    std_config: &std_config,
                    attendance_rules: &attendance_policy,
                    seg_units_per_day,
                    policy_cache: &mut policy_cache,
                },
            )
            .await;

            // C. Financials — units from persisted daily summaries (single source of truth)
            let mut seg_std_units = 0i32;
            let mut seg_ot_units = 0i32;
            let mut seg_holiday_premium = Decimal::ZERO;

            // Find the active employment for this segment date range
            let seg_emp_id = ctx
                .active_employments
                .iter()
                .find(|e| {
                    e.start_date() <= seg_start
                        && e.end_date().is_none_or(|end| end >= seg_start)
                })
                .map(|e| e.id());

            let daily_rate = FinancialCalculator::calculate_daily_rate(
                segment_total_base,
                std_config.std_working_days(),
            );

            if let Some(emp_id) = seg_emp_id {
                for ((eid, date), summary) in &ctx.daily_summaries {
                    if *eid == emp_id && *date >= seg_start && *date <= seg_end {
                        let std_u = crate::services::attendance_calculator::AttendanceCalculator::calculate_units(
                            summary.standard_minutes,
                            attendance_policy.work_unit_minutes,
                            attendance_policy.rounding_policy,
                        );
                        seg_std_units += std_u;
                        seg_ot_units += crate::services::attendance_calculator::AttendanceCalculator::calculate_units(
                            summary.early_ot_minutes
                                + summary.late_ot_minutes
                                + summary.break_ot_minutes,
                            attendance_policy.work_unit_minutes,
                            attendance_policy.rounding_policy,
                        );

                        // Holiday work premium from summary data
                        if let Some(holiday) = ctx.holidays_map.get(date) {
                            if holiday.is_credit_on_work() {
                                seg_std_units += seg_units_per_day.to_i32().unwrap_or(0);
                            }
                            let multiplier = holiday.ot_multiplier().unwrap_or_else(|| Decimal::from(3));
                            if multiplier > Decimal::ONE && std_u > 0 {
                                let worked_fraction = Decimal::from(std_u) / seg_units_per_day;
                                seg_holiday_premium +=
                                    worked_fraction * daily_rate * (multiplier - Decimal::ONE);
                            }
                        }
                    }
                }
            }

            // Paid holiday auto-credit: if no entries exist for a paid holiday date
            // in this segment, credit a full standard day.
            if let Some(_emp_id) = seg_emp_id {
                let mut curr = seg_start;
                while curr <= seg_end {
                    let has_entries = ctx.daily_entries_map.get(&curr).is_some_and(|e| !e.is_empty());
                    if !has_entries {
                        if let Some(holiday) = ctx.holidays_map.get(&curr) {
                            if holiday.is_paid() {
                                seg_std_units += seg_units_per_day.to_i32().unwrap_or(0);
                            }
                        }
                    }
                    curr = match curr.succ_opt() {
                        Some(d) => d,
                        None => break,
                    };
                }
            }

            let worked_days_dec = Decimal::from(seg_std_units) / seg_units_per_day;

            let segment_amount = daily_rate * worked_days_dec;

            total_base_pay += segment_amount;
            total_std_units += seg_std_units;
            total_ot_units += seg_ot_units;
            total_overtime_pay += att.ot_pay;
            total_holiday_work_premium += att.holiday_work_premium + seg_holiday_premium;
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
    pub(super) async fn persist_calculated_payslip(
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
}
