//! AttendanceMatcher — Shift resolution, entry-to-shift matching,
//! daily unit calculation, and holiday credit logic.
//!
//! Extracted from `PayrollService::calculate_monthly_payslip` to satisfy SRP.

use crate::domain::policy::defaults::DEFAULT_OT_MULTIPLIER;
use crate::domain::policy::overtime::OvertimeEngine;
use crate::domain::policy::resolver::PolicyResolver;
use crate::repositories::overtime_policy_repository::OvertimePolicyRepositoryTrait;
use crate::repositories::roster::shift_resolution::resolve_effective_shift_from_data;
use crate::services::AttendanceCalculator;
use chrono::{NaiveDate, TimeZone};
use rust_decimal::prelude::{FromPrimitive, ToPrimitive};
use rust_decimal::Decimal;

use super::financial_calculator::FinancialCalculator;
use super::payroll_service::PayrollContext;

/// Result of processing attendance for a single payroll segment.
pub struct SegmentAttendanceResult {
    pub std_units: i32,
    pub ot_units: i32,
    pub ot_pay: Decimal,
    pub holiday_work_premium: Decimal,
    pub late_early_count: i32,
}

use std::collections::HashMap;
use uuid::Uuid;

pub struct ProcessSegmentAttendanceArgs<'a> {
    pub ctx: &'a PayrollContext,
    pub seg_start: NaiveDate,
    pub seg_end: NaiveDate,
    pub segment_total_base: Decimal,
    pub segment_ot_base: Decimal,
    pub std_config: &'a crate::domain::policy::resolver::StandardMonthConfig,
    pub attendance_rules: &'a crate::models::AttendanceRules,
    pub seg_units_per_day: Decimal,
    pub policy_cache: &'a mut HashMap<Uuid, Vec<crate::models::OvertimeRule>>,
    pub overtime_policy_repo: &'a dyn OvertimePolicyRepositoryTrait,
}

pub struct AttendanceMatcher;

impl AttendanceMatcher {
    /// Process daily attendance for a payroll segment date range.
    ///
    /// Iterates each calendar day in `[seg_start, seg_end]`, resolves shifts,
    /// matches time‑entries to shifts, calculates standard/OT units, holiday
    /// premiums, and OT pay.
    #[tracing::instrument(skip_all)]
    #[allow(clippy::too_many_lines)]
    pub async fn process_segment_attendance(
        args: ProcessSegmentAttendanceArgs<'_>,
    ) -> SegmentAttendanceResult {
        let employee_id = args.ctx.employee_id;

        let mut segment_std_units = 0i32;
        let mut segment_ot_units = 0i32;
        let mut segment_ot_pay = Decimal::ZERO;
        let mut holiday_work_premium = Decimal::ZERO;
        let mut late_early_count = 0i32;

        let mut current_date = args.seg_start;
        while current_date <= args.seg_end {
            // Find active contract on current_date
            let active_contract = args
                .ctx
                .active_contracts
                .iter()
                .filter(|c| {
                    c.status() == crate::models::enums::ContractStatus::Active
                        && c.start_date() <= current_date
                        && c.end_date().is_none_or(|end| end >= current_date)
                })
                .max_by_key(|c| c.start_date());
            let active_employer_id =
                active_contract.map(crate::models::EmployeeContract::employer_id);

            // Find active assignment on current_date
            let active_assignment = args
                .ctx
                .active_assignments
                .iter()
                .filter(|a| {
                    a.status == crate::models::employee_assignment::AssignmentStatusEnum::Active
                        && a.start_date <= current_date
                        && a.end_date.is_none_or(|end| end >= current_date)
                })
                .max_by_key(|a| a.start_date);
            let active_location_id = active_assignment
                .map(|a| a.host_location_id)
                .unwrap_or(args.ctx.primary_location_id);

            // Resolve shift from pre-loaded roster data (in-memory, zero DB queries)
            let (shift_ids, _, _) = resolve_effective_shift_from_data(
                &args.ctx.roster_data,
                &args.ctx.payroll_roster_map,
                employee_id,
                &args.ctx.emp_id_number,
                args.ctx.emp_tags.as_ref(),
                current_date,
                Some(active_location_id),
                active_employer_id,
            );

            // Collect resolved shifts sorted by start_time for OT ceiling
            let mut day_shifts: Vec<_> = shift_ids
                .iter()
                .filter_map(|opt| opt.and_then(|id| args.ctx.roster_data.shifts.get(&id)))
                .collect();
            day_shifts.sort_by_key(|s| s.start_time());

            if let Some(entries) = args.ctx.daily_entries_map.get(&current_date) {
                let work_entries: Vec<_> = entries
                    .iter()
                    .filter(|e| e.entry_type() == "work" || e.entry_type().is_empty())
                    .collect();
                let break_entries: Vec<_> = entries
                    .iter()
                    .filter(|e| e.entry_type() == "break")
                    .collect();
                let actual_break_minutes: i32 = break_entries
                    .iter()
                    .map(|e| e.duration_minutes().unwrap_or(0))
                    .sum();

                // Process each shift with OT ceiling = next shift's start
                for (shift_idx, s) in day_shifts.iter().enumerate() {
                    // OT ceiling: cap late OT at next shift's start time
                    let ot_ceiling = day_shifts.get(shift_idx + 1).and_then(|next_s| {
                        args.ctx
                            .tz
                            .from_local_datetime(&current_date.and_time(next_s.start_time()))
                            .single()
                    });

                    let start_dt = args
                        .ctx
                        .tz
                        .from_local_datetime(&current_date.and_time(s.start_time()))
                        .single();
                    let end_dt = args
                        .ctx
                        .tz
                        .from_local_datetime(&current_date.and_time(s.end_time()))
                        .single();

                    if let (Some(start), Some(mut end)) = (start_dt, end_dt) {
                        if s.end_time() < s.start_time() {
                            end += chrono::Duration::days(1);
                        }

                        // Find the work entry that belongs to this shift
                        let matching_entries: Vec<_> = work_entries
                            .iter()
                            .filter(|entry| {
                                let punch_in = entry.punch_in_time().with_timezone(&args.ctx.tz);
                                let prev_end = if shift_idx > 0 {
                                    day_shifts.get(shift_idx - 1).and_then(|prev_s| {
                                        args.ctx
                                            .tz
                                            .from_local_datetime(
                                                &current_date.and_time(prev_s.end_time()),
                                            )
                                            .single()
                                    })
                                } else {
                                    None
                                };
                                let after_prev = prev_end.is_none_or(|pe| punch_in >= pe);
                                let before_end = punch_in < end;
                                after_prev && before_end
                            })
                            .collect();

                        for entry in matching_entries {
                            let punch_in = entry.punch_in_time().with_timezone(&args.ctx.tz);
                            if let Some(punch_out_raw) = entry.punch_out_time() {
                                let punch_out = punch_out_raw.with_timezone(&args.ctx.tz);

                                let rules = PolicyResolver::resolve_with_shift_overrides_pure(
                                    Some(args.attendance_rules.clone()),
                                    args.ctx.working_conditions.clone(),
                                    current_date,
                                    &crate::domain::policy::resolver::ShiftPolicyOverrides::new(
                                        Some(s.early_punch_in_grace_period_minutes()),
                                        Some(s.is_break_auto_deduct()),
                                        Some(s.allow_early_ot()),
                                        args.attendance_rules.early_arrival_strategy,
                                        entry.is_break_overtime(),
                                    ),
                                );

                                let (std_u, ot_u) = AttendanceCalculator::calculate_daily_units(
                                    &crate::services::attendance_calculator::DailyCalcInput {
                                        punch_in,
                                        punch_out,
                                        shift_start: Some(start),
                                        shift_end: Some(end),
                                        scheduled_break_minutes: s.break_duration_minutes(),
                                        actual_break_minutes,
                                        break_start: s.break_start().and_then(|t| {
                                            args.ctx
                                                .tz
                                                .from_local_datetime(&current_date.and_time(t))
                                                .single()
                                        }),
                                        break_end: s.break_end().and_then(|t| {
                                            let mut dt = args
                                                .ctx
                                                .tz
                                                .from_local_datetime(&current_date.and_time(t))
                                                .single()?;
                                            if s.break_end() < s.break_start() {
                                                dt += chrono::Duration::days(1);
                                            }
                                            Some(dt)
                                        }),
                                        work_unit: args.attendance_rules.work_unit_minutes,
                                        rounding_policy: args
                                            .attendance_rules
                                            .rounding_policy
                                            .clone(),
                                        rules: rules.clone(),
                                        ot_ceiling,
                                        max_early_ot_minutes: s.max_early_ot_minutes(),
                                    },
                                );
                                segment_std_units += std_u;
                                segment_ot_units += ot_u;

                                // --- HOLIDAY WORK COMPENSATION ---
                                if let Some(holiday) = args.ctx.holidays_map.get(&current_date) {
                                    let daily_rate = FinancialCalculator::calculate_daily_rate(
                                        args.segment_total_base,
                                        args.std_config.std_working_days(),
                                    );
                                    if holiday.is_credit_on_work() {
                                        segment_std_units +=
                                            args.seg_units_per_day.to_i32().unwrap_or(0);
                                    }
                                    let multiplier =
                                        holiday.ot_multiplier().unwrap_or_else(|| Decimal::from(3));
                                    if multiplier > Decimal::ONE {
                                        let worked_fraction =
                                            Decimal::from(std_u) / args.seg_units_per_day;
                                        holiday_work_premium += worked_fraction
                                            * daily_rate
                                            * (multiplier - Decimal::ONE);
                                    }
                                }

                                if ot_u > 0 {
                                    if let Some(policy_id) =
                                        args.ctx.overtime_data.resolve_policy_for_employee(
                                            employee_id,
                                            current_date,
                                            &args.ctx.all_dept_history,
                                        )
                                    {
                                        let rules =
                                            if let Some(r) = args.policy_cache.get(&policy_id) {
                                                r.clone()
                                            } else {
                                                let r = args
                                                    .ctx
                                                    .overtime_data
                                                    .policies
                                                    .get(&policy_id)
                                                    .map(|(_, rules)| rules.clone())
                                                    .unwrap_or_default();
                                                args.policy_cache.insert(policy_id, r.clone());
                                                r
                                            };

                                        let hourly_rate =
                                            FinancialCalculator::calculate_hourly_rate(
                                                args.segment_ot_base,
                                                args.std_config.std_working_days(),
                                                args.std_config.hours_per_day(),
                                            );

                                        let mut total_ot_pay = Decimal::ZERO;
                                        if punch_in < start {
                                            let ot_end = std::cmp::min(punch_out, start);
                                            let daily_splits =
                                                OvertimeEngine::split_interval_by_day(
                                                    &punch_in,
                                                    &ot_end,
                                                    &args.ctx.tz,
                                                );
                                            for (date, intervals) in daily_splits {
                                                let day_is_holiday =
                                                    args.ctx.holidays_map.contains_key(&date);
                                                total_ot_pay += OvertimeEngine::calculate_overtime_pay_from_times(&rules, date, day_is_holiday, &intervals, hourly_rate);
                                            }
                                        }
                                        if punch_out > end {
                                            let ot_start = std::cmp::max(punch_in, end);
                                            let ot_end_for_pay = match ot_ceiling {
                                                Some(ceil) => std::cmp::min(punch_out, ceil),
                                                None => punch_out,
                                            };
                                            let daily_splits =
                                                OvertimeEngine::split_interval_by_day(
                                                    &ot_start,
                                                    &ot_end_for_pay,
                                                    &args.ctx.tz,
                                                );
                                            for (date, intervals) in daily_splits {
                                                let day_is_holiday =
                                                    args.ctx.holidays_map.contains_key(&date);
                                                total_ot_pay += OvertimeEngine::calculate_overtime_pay_from_times(&rules, date, day_is_holiday, &intervals, hourly_rate);
                                            }
                                        }
                                        segment_ot_pay += total_ot_pay;
                                    } else {
                                        let hourly_rate =
                                            FinancialCalculator::calculate_hourly_rate(
                                                args.segment_ot_base,
                                                args.std_config.std_working_days(),
                                                args.std_config.hours_per_day(),
                                            );
                                        segment_ot_pay += (Decimal::from(ot_u)
                                            * Decimal::from(
                                                args.attendance_rules.work_unit_minutes,
                                            )
                                            / Decimal::from(60))
                                            * hourly_rate
                                            * Decimal::from_f64(DEFAULT_OT_MULTIPLIER)
                                                .unwrap_or_else(|| Decimal::new(15, 1));
                                    }
                                }

                                // Violation detection for IncentiveContext
                                if !args.ctx.holidays_map.contains_key(&current_date) {
                                    let is_late = punch_in
                                        > start
                                            + chrono::Duration::minutes(i64::from(
                                                rules.grace_period_minutes(),
                                            ));
                                    let is_early = punch_out < end;
                                    if is_late || is_early {
                                        late_early_count += 1;
                                    }
                                }
                            }
                        }
                    }
                }

                // Fallback: entries with no matching shift (day_shifts is empty)
                if day_shifts.is_empty() {
                    for entry in &work_entries {
                        let punch_in = entry.punch_in_time().with_timezone(&args.ctx.tz);
                        if let Some(punch_out_raw) = entry.punch_out_time() {
                            let punch_out = punch_out_raw.with_timezone(&args.ctx.tz);
                            segment_std_units += AttendanceCalculator::calculate_units(
                                (punch_out - punch_in).num_minutes() as i32,
                                args.attendance_rules.work_unit_minutes,
                                &args.attendance_rules.rounding_policy,
                            );
                        }
                    }
                }
            } else if let Some(holiday) = args.ctx.holidays_map.get(&current_date) {
                // Paid holiday with no attendance → auto-credit standard day
                if holiday.is_paid() {
                    let credit_units = if holiday.holiday_type() == "half_day" {
                        (args.seg_units_per_day / Decimal::from(2))
                            .to_i32()
                            .unwrap_or(0)
                    } else {
                        args.seg_units_per_day.to_i32().unwrap_or(0)
                    };
                    segment_std_units += credit_units;
                }
            }
            current_date = match current_date.succ_opt() {
                Some(d) => d,
                None => break,
            };
        }

        SegmentAttendanceResult {
            std_units: segment_std_units,
            ot_units: segment_ot_units,
            ot_pay: segment_ot_pay,
            holiday_work_premium,
            late_early_count,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cache::{RosterData, RosterShift};
    use crate::models::enums::EarlyArrivalStrategy;
    use crate::models::{AttendanceRules, Holiday, TimeEntry, Workspace};
    use crate::repositories::overtime_policy_repository::OvertimePolicyRepositoryTrait;
    use chrono::{NaiveDate, NaiveTime, Utc};
    use std::collections::HashMap;
    use std::sync::Arc;
    use uuid::Uuid;

    struct MockOvertimePolicyRepository;

    #[async_trait::async_trait]
    impl OvertimePolicyRepositoryTrait for MockOvertimePolicyRepository {
        async fn create_policy_definition(
            &self,
            _workspace_id: Uuid,
            _name: &str,
            _description: Option<&str>,
        ) -> Result<crate::models::OvertimePolicy, crate::errors::AppError> {
            unimplemented!()
        }

        async fn get_policy_definitions(
            &self,
            _workspace_id: Uuid,
        ) -> Result<Vec<crate::models::OvertimePolicy>, crate::errors::AppError> {
            unimplemented!()
        }

        async fn get_policy_with_rules(
            &self,
            _workspace_id: Uuid,
            _id: Uuid,
        ) -> Result<
            Option<(
                crate::models::OvertimePolicy,
                Vec<crate::models::OvertimeRule>,
            )>,
            crate::errors::AppError,
        > {
            Ok(None)
        }

        async fn add_rule(
            &self,
            _workspace_id: Uuid,
            _rule: &crate::models::OvertimeRule,
        ) -> Result<crate::models::OvertimeRule, crate::errors::AppError> {
            unimplemented!()
        }

        async fn resolve_policy_for_employee(
            &self,
            _employee_id: Uuid,
            _workspace_id: Uuid,
            _date: NaiveDate,
        ) -> Result<Option<Uuid>, crate::errors::AppError> {
            Ok(None)
        }

        async fn create_policy_assignment(
            &self,
            _workspace_id: Uuid,
            _assignee_type: &str,
            _assignee_id: Option<Uuid>,
            _policy_id: Uuid,
            _effective_from: NaiveDate,
        ) -> Result<(), crate::errors::AppError> {
            unimplemented!()
        }

        async fn get_workspace_overtime_data(
            &self,
            _workspace_id: Uuid,
        ) -> Result<
            crate::repositories::overtime_policy_repository::WorkspaceOvertimeData,
            crate::errors::AppError,
        > {
            Ok(
                crate::repositories::overtime_policy_repository::WorkspaceOvertimeData {
                    policies: std::collections::HashMap::new(),
                    assignments: Vec::new(),
                },
            )
        }
    }

    struct MockOvertimePolicyWithRulesRepository {
        policy_id: Uuid,
        rules: Vec<crate::models::OvertimeRule>,
    }

    #[async_trait::async_trait]
    impl OvertimePolicyRepositoryTrait for MockOvertimePolicyWithRulesRepository {
        async fn create_policy_definition(
            &self,
            _workspace_id: Uuid,
            _name: &str,
            _description: Option<&str>,
        ) -> Result<crate::models::OvertimePolicy, crate::errors::AppError> {
            unimplemented!()
        }

        async fn get_policy_definitions(
            &self,
            _workspace_id: Uuid,
        ) -> Result<Vec<crate::models::OvertimePolicy>, crate::errors::AppError> {
            unimplemented!()
        }

        async fn get_policy_with_rules(
            &self,
            _workspace_id: Uuid,
            id: Uuid,
        ) -> Result<
            Option<(
                crate::models::OvertimePolicy,
                Vec<crate::models::OvertimeRule>,
            )>,
            crate::errors::AppError,
        > {
            if id == self.policy_id {
                let policy = crate::models::OvertimePolicy::new(
                    id,
                    _workspace_id,
                    "Test Policy".to_string(),
                    None,
                    None,
                    None,
                );
                Ok(Some((policy, self.rules.clone())))
            } else {
                Ok(None)
            }
        }

        async fn add_rule(
            &self,
            _workspace_id: Uuid,
            _rule: &crate::models::OvertimeRule,
        ) -> Result<crate::models::OvertimeRule, crate::errors::AppError> {
            unimplemented!()
        }

        async fn resolve_policy_for_employee(
            &self,
            _employee_id: Uuid,
            _workspace_id: Uuid,
            _date: NaiveDate,
        ) -> Result<Option<Uuid>, crate::errors::AppError> {
            Ok(Some(self.policy_id))
        }

        async fn create_policy_assignment(
            &self,
            _workspace_id: Uuid,
            _assignee_type: &str,
            _assignee_id: Option<Uuid>,
            _policy_id: Uuid,
            _effective_from: NaiveDate,
        ) -> Result<(), crate::errors::AppError> {
            unimplemented!()
        }

        async fn get_workspace_overtime_data(
            &self,
            _workspace_id: Uuid,
        ) -> Result<
            crate::repositories::overtime_policy_repository::WorkspaceOvertimeData,
            crate::errors::AppError,
        > {
            Ok(
                crate::repositories::overtime_policy_repository::WorkspaceOvertimeData {
                    policies: std::collections::HashMap::new(),
                    assignments: Vec::new(),
                },
            )
        }
    }

    fn test_workspace(ws_id: Uuid) -> Workspace {
        Workspace::new(
            ws_id,
            "Test Workspace".to_string(),
            "YYYY-MM-DD".to_string(),
            "HH:mm:ss".to_string(),
            "UTC".to_string(),
            "#000".to_string(),
            "en".to_string(),
            None,
            None,
            "SplitAtMidnight".to_string(),
            Decimal::new(30, 1),
            Utc::now(),
            None,
        )
    }

    #[tokio::test]
    async fn test_process_segment_attendance_no_shift_fallback() {
        let employee_id = Uuid::new_v4();
        let ws_id = Uuid::new_v4();

        let workspace = test_workspace(ws_id);

        // Create time entry (punch-in: 08:00, punch-out: 12:00) => 4 hours (8 units of 30 mins)
        let punch_in = Utc::now()
            .date_naive()
            .and_hms_opt(8, 0, 0)
            .unwrap()
            .and_utc();
        let punch_out = Utc::now()
            .date_naive()
            .and_hms_opt(12, 0, 0)
            .unwrap()
            .and_utc();

        let entry = TimeEntry::new(
            Uuid::new_v4(),
            employee_id,
            ws_id,
            None,
            punch_in,
            Some(punch_out),
            Some(240),
            false,
            "work".to_string(),
            None,
            None,
            Utc::now(),
            Utc::now(),
        );

        let mut daily_entries_map = HashMap::new();
        daily_entries_map.insert(Utc::now().date_naive(), vec![entry]);

        let attendance_rules = AttendanceRules {
            work_unit_minutes: 30,
            rounding_policy: "floor".to_string(),
            early_arrival_strategy: EarlyArrivalStrategy::Block,
        };

        let ctx = PayrollContext {
            employee_id,
            workspace,
            tz: chrono_tz::UTC,
            start_date: Utc::now().date_naive(),
            end_date: Utc::now().date_naive(),
            salary_history: vec![],
            all_dept_history: vec![],
            daily_entries_map,
            holidays_map: HashMap::new(),
            roster_data: Arc::new(RosterData::default()),
            emp_id_number: "EMP001".to_string(),
            emp_tags: None,
            payroll_roster_map: HashMap::new(),
            working_conditions: vec![],
            attendance_policy: attendance_rules.clone(),
            emp_payroll_configs: vec![],
            ws_payroll_settings_history: vec![],
            department_payroll_configs: HashMap::new(),
            overtime_data:
                crate::repositories::overtime_policy_repository::WorkspaceOvertimeData::default(),
            active_contracts: vec![],
            active_assignments: vec![],
            primary_location_id: Uuid::new_v4(),
        };

        let std_config = crate::domain::policy::resolver::StandardMonthConfig::new(
            Decimal::new(22, 0),
            Decimal::new(8, 0),
            "workspace".to_string(),
        );

        let mut policy_cache = HashMap::new();
        let overtime_policy_repo = MockOvertimePolicyRepository;

        // resolver doesn't get called because day_shifts is empty
        let args = ProcessSegmentAttendanceArgs {
            ctx: &ctx,
            seg_start: Utc::now().date_naive(),
            seg_end: Utc::now().date_naive(),
            segment_total_base: Decimal::new(1000, 0),
            segment_ot_base: Decimal::new(1000, 0),
            std_config: &std_config,
            attendance_rules: &attendance_rules,
            seg_units_per_day: Decimal::new(16, 0),
            policy_cache: &mut policy_cache,
            overtime_policy_repo: &overtime_policy_repo,
        };

        let result = AttendanceMatcher::process_segment_attendance(args).await;
        // 240 mins / 30 mins = 8 units
        assert_eq!(result.std_units, 8);
        assert_eq!(result.ot_units, 0);
    }

    #[tokio::test]
    async fn test_process_segment_attendance_with_shift_matching() {
        let employee_id = Uuid::new_v4();
        let ws_id = Uuid::new_v4();
        let shift_id = Uuid::new_v4();

        let workspace = test_workspace(ws_id);

        let date = Utc::now().date_naive();
        // Time entry: 09:00 to 18:00 (matching 9-18 shift)
        let punch_in = date.and_hms_opt(9, 0, 0).unwrap().and_utc();
        let punch_out = date.and_hms_opt(18, 0, 0).unwrap().and_utc();

        let entry = TimeEntry::new(
            Uuid::new_v4(),
            employee_id,
            ws_id,
            None,
            punch_in,
            Some(punch_out),
            Some(540),
            false,
            "work".to_string(),
            None,
            None,
            Utc::now(),
            Utc::now(),
        );

        let mut daily_entries_map = HashMap::new();
        daily_entries_map.insert(date, vec![entry]);

        // Define shift: 09:00 to 18:00, 1 hour (60 min) break
        let shift = RosterShift::new(
            "Day Shift".to_string(),
            Some("D".to_string()),
            NaiveTime::from_hms_opt(9, 0, 0).unwrap(),
            NaiveTime::from_hms_opt(18, 0, 0).unwrap(),
            60,
            Some(NaiveTime::from_hms_opt(12, 0, 0).unwrap()),
            Some(NaiveTime::from_hms_opt(13, 0, 0).unwrap()),
            true,
            0,
            false,
            false,
            None,
        );

        let mut shifts = HashMap::new();
        shifts.insert(shift_id, shift);

        let roster_data = RosterData {
            shifts,
            ..Default::default()
        };

        let mut payroll_roster_map = HashMap::new();
        payroll_roster_map.insert((employee_id, date), Some(shift_id));

        let attendance_rules = AttendanceRules {
            work_unit_minutes: 30,
            rounding_policy: "floor".to_string(),
            early_arrival_strategy: EarlyArrivalStrategy::Block,
        };

        let ctx = PayrollContext {
            employee_id,
            workspace,
            tz: chrono_tz::UTC,
            start_date: date,
            end_date: date,
            salary_history: vec![],
            all_dept_history: vec![],
            daily_entries_map,
            holidays_map: HashMap::new(),
            roster_data: Arc::new(roster_data),
            emp_id_number: "EMP001".to_string(),
            emp_tags: None,
            payroll_roster_map,
            working_conditions: vec![],
            attendance_policy: attendance_rules.clone(),
            emp_payroll_configs: vec![],
            ws_payroll_settings_history: vec![],
            department_payroll_configs: HashMap::new(),
            overtime_data:
                crate::repositories::overtime_policy_repository::WorkspaceOvertimeData::default(),
            active_contracts: vec![],
            active_assignments: vec![],
            primary_location_id: Uuid::new_v4(),
        };

        let std_config = crate::domain::policy::resolver::StandardMonthConfig::new(
            Decimal::new(22, 0),
            Decimal::new(8, 0),
            "workspace".to_string(),
        );

        let mut policy_cache = HashMap::new();
        let overtime_policy_repo = MockOvertimePolicyRepository;

        let args = ProcessSegmentAttendanceArgs {
            ctx: &ctx,
            seg_start: date,
            seg_end: date,
            segment_total_base: Decimal::new(1000, 0),
            segment_ot_base: Decimal::new(1000, 0),
            std_config: &std_config,
            attendance_rules: &attendance_rules,
            seg_units_per_day: Decimal::new(16, 0),
            policy_cache: &mut policy_cache,
            overtime_policy_repo: &overtime_policy_repo,
        };

        let result = AttendanceMatcher::process_segment_attendance(args).await;
        // Total work duration = 9 hours (540 mins) minus 60 min break = 8 hours (480 mins) = 16 units of 30 mins.
        assert_eq!(result.std_units, 16);
    }

    #[tokio::test]
    async fn test_process_segment_attendance_paid_holiday_no_work() {
        let employee_id = Uuid::new_v4();
        let ws_id = Uuid::new_v4();

        let workspace = test_workspace(ws_id);

        let date = Utc::now().date_naive();

        // Holiday map with paid holiday
        let mut holidays_map = HashMap::new();
        holidays_map.insert(
            date,
            Holiday::new(
                Uuid::new_v4(),
                ws_id,
                "New Year".to_string(),
                date,
                "full_day".to_string(),
                true,
                None,
                Some(Decimal::new(30, 1)),
                None,
                None,
                false,
                Utc::now(),
                Utc::now(),
                None,
            ),
        );

        let attendance_rules = AttendanceRules {
            work_unit_minutes: 30,
            rounding_policy: "floor".to_string(),
            early_arrival_strategy: EarlyArrivalStrategy::Block,
        };

        let ctx = PayrollContext {
            employee_id,
            workspace,
            tz: chrono_tz::UTC,
            start_date: date,
            end_date: date,
            salary_history: vec![],
            all_dept_history: vec![],
            daily_entries_map: HashMap::new(),
            holidays_map,
            roster_data: Arc::new(RosterData::default()),
            emp_id_number: "EMP001".to_string(),
            emp_tags: None,
            payroll_roster_map: HashMap::new(),
            working_conditions: vec![],
            attendance_policy: attendance_rules.clone(),
            emp_payroll_configs: vec![],
            ws_payroll_settings_history: vec![],
            department_payroll_configs: HashMap::new(),
            overtime_data:
                crate::repositories::overtime_policy_repository::WorkspaceOvertimeData::default(),
            active_contracts: vec![],
            active_assignments: vec![],
            primary_location_id: Uuid::new_v4(),
        };

        let std_config = crate::domain::policy::resolver::StandardMonthConfig::new(
            Decimal::new(22, 0),
            Decimal::new(8, 0),
            "workspace".to_string(),
        );

        let mut policy_cache = HashMap::new();
        let overtime_policy_repo = MockOvertimePolicyRepository;

        let args = ProcessSegmentAttendanceArgs {
            ctx: &ctx,
            seg_start: date,
            seg_end: date,
            segment_total_base: Decimal::new(1000, 0),
            segment_ot_base: Decimal::new(1000, 0),
            std_config: &std_config,
            attendance_rules: &attendance_rules,
            seg_units_per_day: Decimal::new(16, 0), // 8 hours * 2 units/hr = 16 units
            policy_cache: &mut policy_cache,
            overtime_policy_repo: &overtime_policy_repo,
        };

        let result = AttendanceMatcher::process_segment_attendance(args).await;
        // Auto-credit standard day (16 units) because it's a paid holiday with no attendance
        assert_eq!(result.std_units, 16);
    }

    #[tokio::test]
    async fn test_process_segment_attendance_holiday_work_compensation() {
        let employee_id = Uuid::new_v4();
        let ws_id = Uuid::new_v4();
        let shift_id = Uuid::new_v4();

        let workspace = test_workspace(ws_id);

        let date = Utc::now().date_naive();
        // Time entry: 09:00 to 18:00 (matching 9-18 shift)
        let punch_in = date.and_hms_opt(9, 0, 0).unwrap().and_utc();
        let punch_out = date.and_hms_opt(18, 0, 0).unwrap().and_utc();

        let entry = TimeEntry::new(
            Uuid::new_v4(),
            employee_id,
            ws_id,
            None,
            punch_in,
            Some(punch_out),
            Some(540),
            false,
            "work".to_string(),
            None,
            None,
            Utc::now(),
            Utc::now(),
        );

        let mut daily_entries_map = HashMap::new();
        daily_entries_map.insert(date, vec![entry]);

        // Define shift: 09:00 to 18:00, 1 hour break
        let shift = RosterShift::new(
            "Day Shift".to_string(),
            Some("D".to_string()),
            NaiveTime::from_hms_opt(9, 0, 0).unwrap(),
            NaiveTime::from_hms_opt(18, 0, 0).unwrap(),
            60,
            Some(NaiveTime::from_hms_opt(12, 0, 0).unwrap()),
            Some(NaiveTime::from_hms_opt(13, 0, 0).unwrap()),
            true,
            0,
            false,
            false,
            None,
        );

        let mut shifts = HashMap::new();
        shifts.insert(shift_id, shift);

        let roster_data = RosterData {
            shifts,
            ..Default::default()
        };

        let mut payroll_roster_map = HashMap::new();
        payroll_roster_map.insert((employee_id, date), Some(shift_id));

        let mut holidays_map = HashMap::new();
        holidays_map.insert(
            date,
            Holiday::new(
                Uuid::new_v4(),
                ws_id,
                "New Year".to_string(),
                date,
                "full_day".to_string(),
                true,
                None,
                Some(Decimal::new(30, 1)),
                None,
                None,
                true,
                Utc::now(),
                Utc::now(),
                None,
            ),
        );

        let attendance_rules = AttendanceRules {
            work_unit_minutes: 30,
            rounding_policy: "floor".to_string(),
            early_arrival_strategy: EarlyArrivalStrategy::Block,
        };

        let ctx = PayrollContext {
            employee_id,
            workspace,
            tz: chrono_tz::UTC,
            start_date: date,
            end_date: date,
            salary_history: vec![],
            all_dept_history: vec![],
            daily_entries_map,
            holidays_map,
            roster_data: Arc::new(roster_data),
            emp_id_number: "EMP001".to_string(),
            emp_tags: None,
            payroll_roster_map,
            working_conditions: vec![],
            attendance_policy: attendance_rules.clone(),
            emp_payroll_configs: vec![],
            ws_payroll_settings_history: vec![],
            department_payroll_configs: HashMap::new(),
            overtime_data:
                crate::repositories::overtime_policy_repository::WorkspaceOvertimeData::default(),
            active_contracts: vec![],
            active_assignments: vec![],
            primary_location_id: Uuid::new_v4(),
        };

        let std_config = crate::domain::policy::resolver::StandardMonthConfig::new(
            Decimal::new(22, 0),
            Decimal::new(8, 0),
            "workspace".to_string(),
        );

        let mut policy_cache = HashMap::new();
        let overtime_policy_repo = MockOvertimePolicyRepository;

        let args = ProcessSegmentAttendanceArgs {
            ctx: &ctx,
            seg_start: date,
            seg_end: date,
            segment_total_base: Decimal::new(1000, 0),
            segment_ot_base: Decimal::new(1000, 0),
            std_config: &std_config,
            attendance_rules: &attendance_rules,
            seg_units_per_day: Decimal::new(16, 0), // 8 hours = 16 units
            policy_cache: &mut policy_cache,
            overtime_policy_repo: &overtime_policy_repo,
        };

        let result = AttendanceMatcher::process_segment_attendance(args).await;
        // worked units: 16 (8 hours)
        // is_credit_on_work adds seg_units_per_day (16) => 32 std units
        assert_eq!(result.std_units, 32);
        // worked fraction = 16 / 16 = 1.0
        // daily rate = 1000 / 22 = 45.4545...
        // multiplier - 1.0 = 2.0
        // holiday premium = 1.0 * (1000 / 22) * 2.0 = 90.90909090909090909090909091
        let expected_premium = Decimal::new(1000, 0) / Decimal::new(22, 0) * Decimal::new(2, 0);
        assert_eq!(result.holiday_work_premium, expected_premium);
    }

    #[tokio::test]
    async fn test_process_segment_attendance_with_overtime_pay() {
        let employee_id = Uuid::new_v4();
        let ws_id = Uuid::new_v4();
        let shift_id = Uuid::new_v4();
        let policy_id = Uuid::new_v4();

        let workspace = test_workspace(ws_id);

        let date = NaiveDate::from_ymd_opt(2026, 1, 26).unwrap(); // Monday
                                                                  // Time entry: 09:00 to 20:00 (matching 9-18 shift) -> 2 hours late OT
        let punch_in = date.and_hms_opt(9, 0, 0).unwrap().and_utc();
        let punch_out = date.and_hms_opt(20, 0, 0).unwrap().and_utc();

        let entry = TimeEntry::new(
            Uuid::new_v4(),
            employee_id,
            ws_id,
            None,
            punch_in,
            Some(punch_out),
            Some(660),
            false,
            "work".to_string(),
            None,
            None,
            Utc::now(),
            Utc::now(),
        );

        let mut daily_entries_map = HashMap::new();
        daily_entries_map.insert(date, vec![entry]);

        // Define shift: 09:00 to 18:00, 1 hour break
        let shift = RosterShift::new(
            "Day Shift".to_string(),
            Some("D".to_string()),
            NaiveTime::from_hms_opt(9, 0, 0).unwrap(),
            NaiveTime::from_hms_opt(18, 0, 0).unwrap(),
            60,
            Some(NaiveTime::from_hms_opt(12, 0, 0).unwrap()),
            Some(NaiveTime::from_hms_opt(13, 0, 0).unwrap()),
            true,
            0,
            false,
            false,
            None,
        );

        let mut shifts = HashMap::new();
        shifts.insert(shift_id, shift);

        let roster_data = RosterData {
            shifts,
            ..Default::default()
        };

        let mut payroll_roster_map = HashMap::new();
        payroll_roster_map.insert((employee_id, date), Some(shift_id));

        let attendance_rules = AttendanceRules {
            work_unit_minutes: 30,
            rounding_policy: "floor".to_string(),
            early_arrival_strategy: EarlyArrivalStrategy::Block,
        };

        let rules = vec![crate::models::OvertimeRule::new(
            Uuid::new_v4(),
            policy_id,
            1,
            Decimal::new(20, 1), // 2.0x
            Some(NaiveTime::from_hms_opt(18, 0, 0).unwrap()),
            Some(NaiveTime::from_hms_opt(22, 0, 0).unwrap()),
            127,
            false,
            None,
            None,
        )];

        let mut policies = HashMap::new();
        let overtime_policy = crate::models::OvertimePolicy::new(
            policy_id,
            Uuid::new_v4(),
            "Test Policy".to_string(),
            None,
            None,
            None,
        );
        policies.insert(policy_id, (overtime_policy, rules.clone()));

        let assignments = vec![crate::models::OvertimePolicyAssignment::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            "employee".to_string(),
            Some(employee_id),
            policy_id,
            date,
            None,
            Some(chrono::Utc::now()),
        )];

        let overtime_data =
            crate::repositories::overtime_policy_repository::WorkspaceOvertimeData {
                policies,
                assignments,
            };

        let ctx = PayrollContext {
            employee_id,
            workspace,
            tz: chrono_tz::UTC,
            start_date: date,
            end_date: date,
            salary_history: vec![],
            all_dept_history: vec![],
            daily_entries_map,
            holidays_map: HashMap::new(),
            roster_data: Arc::new(roster_data),
            emp_id_number: "EMP001".to_string(),
            emp_tags: None,
            payroll_roster_map,
            working_conditions: vec![],
            attendance_policy: attendance_rules.clone(),
            emp_payroll_configs: vec![],
            ws_payroll_settings_history: vec![],
            department_payroll_configs: HashMap::new(),
            overtime_data,
            active_contracts: vec![],
            active_assignments: vec![],
            primary_location_id: Uuid::new_v4(),
        };

        let std_config = crate::domain::policy::resolver::StandardMonthConfig::new(
            Decimal::new(22, 0),
            Decimal::new(8, 0),
            "workspace".to_string(),
        );

        let overtime_policy_repo = MockOvertimePolicyWithRulesRepository {
            policy_id,
            rules: rules.clone(),
        };

        let mut policy_cache = HashMap::new();
        let args = ProcessSegmentAttendanceArgs {
            ctx: &ctx,
            seg_start: date,
            seg_end: date,
            segment_total_base: Decimal::new(1760, 0),
            segment_ot_base: Decimal::new(1760, 0), // hourly_rate = 1760 / (22*8) = 10
            std_config: &std_config,
            attendance_rules: &attendance_rules,
            seg_units_per_day: Decimal::new(16, 0),
            policy_cache: &mut policy_cache,
            overtime_policy_repo: &overtime_policy_repo,
        };

        let result = AttendanceMatcher::process_segment_attendance(args).await;
        // 8 hours std + 2 hours OT
        assert_eq!(result.std_units, 16);
        assert_eq!(result.ot_units, 4); // 2 hours = 4 units
                                        // 2 hours OT * 10 hourly_rate * 2.0 multiplier = 40.0
        assert_eq!(result.ot_pay, Decimal::new(40, 0));
    }
}
