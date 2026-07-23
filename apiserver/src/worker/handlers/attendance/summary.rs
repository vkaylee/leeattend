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
            args.workspace_id,
            args.start_date,
            args.end_date,
        )
        .await
        .map_err(|e| format!("Failed to fetch working conditions: {e}"))?;

        let attendance_rules_map = PolicyResolverService::resolve_attendance_policy_batch(
            args.policy_repo,
            emp_ids,
            args.workspace_id,
            args.start_date, // Using start_date as proxy for the report period
        )
        .await
        .map_err(|e| format!("Failed to fetch attendance rules: {e}"))?;

        for emp_sched in args.schedules {
            let employee_id = args
                .employment_to_employee
                .get(&emp_sched.employment_id())
                .copied()
                .unwrap_or_else(Uuid::nil);

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

            let emp_attendance_rules = attendance_rules_map
                .get(&employee_id)
                .cloned()
                .unwrap_or_else(|| crate::models::AttendanceRules {
                    work_unit_minutes: 30,
                    rounding_policy: "floor".to_string(),
                    early_arrival_strategy: crate::models::EarlyArrivalStrategy::Block,
                    grace_period_minutes: None,
                });

            // Pre-group entries by date once — O(entries) instead of O(entries × days)
            let entries_by_date =
                crate::services::attendance_utils::group_entries_by_date(entries, args.tz);

            for daily in emp_sched.schedules() {
                let date = daily.date();

                let day_entries = entries_by_date
                    .get(&date)
                    .map_or_else(Default::default, std::vec::Vec::as_slice);

                let actual_punch_in = day_entries
                    .first()
                    .map(|e| e.punch_in_time().with_timezone(&args.tz));
                let actual_punch_out = day_entries
                    .last()
                    .and_then(|e| e.punch_out_time().map(|t| t.with_timezone(&args.tz)));

                if let (Some(punch_in), Some(punch_out)) = (actual_punch_in, actual_punch_out) {
                    let actual_break_minutes: i32 = day_entries
                        .iter()
                        .filter(|e| e.entry_type() == "break")
                        .map(|e| e.duration_minutes().unwrap_or(0))
                        .sum();

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

                    // Handle shift times
                    let shift_start_dt = daily
                        .start_time()
                        .and_then(|t| args.tz.from_local_datetime(&date.and_time(t)).single());
                    let shift_end_dt = daily
                        .end_time()
                        .and_then(|t| args.tz.from_local_datetime(&date.and_time(t)).single());

                    let (shift_start, shift_end) =
                        if let (Some(start), Some(mut end)) = (shift_start_dt, shift_end_dt) {
                            if daily.end_time() < daily.start_time() {
                                end += Duration::days(1);
                            }
                            (Some(start), Some(end))
                        } else {
                            (None, None)
                        };

                    // Build break frame datetimes for Frame Intersection
                    let break_start_dt = daily
                        .break_start()
                        .and_then(|t| args.tz.from_local_datetime(&date.and_time(t)).single());
                    let break_end_dt = daily.break_end().and_then(|t| {
                        let mut dt = args.tz.from_local_datetime(&date.and_time(t)).single()?;
                        if daily.break_end() < daily.break_start() {
                            dt += Duration::days(1);
                        }
                        Some(dt)
                    });

                    let (std_units, ot_units) = AttendanceCalculator::calculate_daily_units(
                        &crate::services::attendance_calculator::DailyCalcInput {
                            punch_in,
                            punch_out,
                            shift_start,
                            shift_end,
                            scheduled_break_minutes: daily.break_duration_minutes().unwrap_or(0),
                            actual_break_minutes,
                            break_start: break_start_dt,
                            break_end: break_end_dt,
                            work_unit: emp_attendance_rules.work_unit_minutes,
                            rounding_policy: emp_attendance_rules.rounding_policy.clone(),
                            rules: rules.clone(),
                            ot_ceiling: None,
                            max_early_ot_minutes: daily.max_early_ot_minutes(),
                            segment_id: None,
                            ot_aggregation_mode:
                                crate::services::attendance_calculator::OtAggregationMode::Aggregate,
                        },
                    );

                    total_std_units += std_units;
                    total_ot_units += ot_units;
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
