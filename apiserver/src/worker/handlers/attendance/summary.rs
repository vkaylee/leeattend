use super::*;

impl AttendanceHandler {
    #[allow(clippy::too_many_lines)]
    pub(super) async fn calculate_summary(args: CalculateBatchArgs<'_>) -> Result<String, String> {
        let mut summaries: Vec<AttendanceSummary> = Vec::new();
        let emp_ids = args.employee_ids;

        // Batch fetch leave data
        let leave_data = args
            .leave_repo
            .get_batch_leave_days(
                &crate::diesel_pool::TenantAuthToken::system_override(args.workspace_id),
                emp_ids,
                args.start_date,
                args.end_date,
            )
            .await
            .map_err(|e| format!("Failed to fetch leave data: {e}"))?;

        // Batch fetch working conditions for all employees (eliminates N+1)
        let conditions_map = PolicyResolverService::resolve_attendance_rules_batch(
            args.employee_repo,
            emp_ids,
            &crate::diesel_pool::TenantAuthToken::system_override(args.workspace_id),
            args.start_date,
            args.end_date,
        )
        .await
        .map_err(|e| format!("Failed to fetch working conditions: {e}"))?;

        let attendance_rules_map = PolicyResolverService::resolve_attendance_policy_batch(
            args.policy_repo,
            emp_ids,
            &crate::diesel_pool::TenantAuthToken::system_override(args.workspace_id),
            args.start_date, // Using start_date as proxy for the report period
        )
        .await
        .map_err(|e| format!("Failed to fetch attendance rules: {e}"))?;

        for emp_sched in args.schedules {
            let Some(employee_id) = args
                .employment_to_employee
                .get(&emp_sched.employment_id())
                .copied()
            else {
                tracing::warn!(
                    employment_id = %emp_sched.employment_id(),
                    "employment→employee mapping miss — skipping row"
                );
                continue;
            };

            let entries = args
                .entries_map
                .get(&emp_sched.employment_id())
                .map_or_else(Default::default, std::vec::Vec::as_slice);
            let mut total_std_units = 0i32;
            let mut total_ot_units = 0i32;

            // Get this employee's conditions once
            let emp_conditions = conditions_map
                .get(&employee_id)
                .map_or_else(Default::default, std::vec::Vec::as_slice);

            let emp_attendance_rules = attendance_rules_map.get(&employee_id).cloned().unwrap_or(
                crate::models::AttendanceRules {
                    work_unit_minutes: 30,
                    rounding_policy: crate::models::RoundingPolicy::Floor,
                    early_arrival_strategy: crate::models::EarlyArrivalStrategy::Block,
                    grace_period_minutes: None,
                },
            );

            // Pre-group entries by date once — O(entries) instead of O(entries × days)
            let entries_by_date =
                crate::services::attendance_utils::group_entries_by_date(entries, args.tz);

            for daily in emp_sched.schedules() {
                let date = daily.date();

                let day_entries = entries_by_date
                    .get(&date)
                    .map_or_else(Default::default, std::vec::Vec::as_slice);

                if daily.shift_id().is_some() {
                    // Build rules from pre-fetched conditions (no DB call)
                    let mut rules = PolicyResolver::build_rules_from_conditions(
                        emp_conditions,
                        date,
                        emp_attendance_rules.early_arrival_strategy,
                        emp_attendance_rules.grace_period_minutes.unwrap_or(
                            crate::domain::policy::defaults::DEFAULT_GRACE_PERIOD_MINUTES,
                        ),
                        Some(args.condition_type_catalog),
                    );

                    // Override with shift-specific settings
                    if let Some(bad) = daily.is_break_auto_deduct() {
                        rules.set_is_break_auto_deduct(bad);
                    }
                    if let Some(aeo) = daily.allow_early_ot() {
                        rules.set_allow_early_ot(aeo);
                    }

                    // Delegate to shared AttendanceService instead of
                    // building DailyCalcInput manually.
                    let result = crate::services::AttendanceService::compute_daily_attendance(
                        crate::services::ComputeDailyInput {
                            daily,
                            day_entries,
                            date,
                            tz: &args.tz,
                            rules: &rules,
                            work_unit_minutes: emp_attendance_rules.work_unit_minutes,
                            rounding_policy: emp_attendance_rules.rounding_policy,
                            max_early_ot_minutes: daily.max_early_ot_minutes(),
                            ot_ceiling: None,
                            segment_id: None,
                            ot_aggregation_mode:
                                crate::services::attendance_calculator::OtAggregationMode::Aggregate,
                        },
                    );

                    if let Some(r) = result {
                        let (std_units, ot_units) = r.into_units(
                            emp_attendance_rules.work_unit_minutes,
                            emp_attendance_rules.rounding_policy,
                        );
                        total_std_units += std_units;
                        total_ot_units += ot_units;
                    }
                }
            }

            let (employer_id, employer_name) = args
                .employment_to_employer
                .get(&emp_sched.employment_id())
                .cloned()
                .map(|(id, name)| (Some(id), name))
                .unwrap_or((None, None));

            summaries.push(AttendanceSummary {
                employment_id: emp_sched.employment_id(),
                employee_id,
                employee_name: emp_sched.full_name().to_string(),
                employee_id_number: emp_sched.employee_id_number().to_string(),
                total_std_units,
                total_ot_units,
                total_leave_days: leave_data.get(&employee_id).copied().unwrap_or(0),
                unit_type: crate::models::enums::TimekeepingUnit::DAY,
                unit_coefficient_hours: f64::from(emp_attendance_rules.work_unit_minutes) / 60.0,
                computed_std_hours: f64::from(total_std_units)
                    * (f64::from(emp_attendance_rules.work_unit_minutes) / 60.0),
                computed_ot_hours: f64::from(total_ot_units)
                    * (f64::from(emp_attendance_rules.work_unit_minutes) / 60.0),
                employer_id,
                employer_name,
            });
        }

        serde_json::to_string(&summaries).map_err(|e| format!("Failed to serialize: {e}"))
    }
}
