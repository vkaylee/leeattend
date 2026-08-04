use super::*;
use crate::cache::{RosterData, RosterShift};
use crate::models::enums::EarlyArrivalStrategy;
use crate::models::{
    AttendanceRules, Holiday, NewEmploymentParams, NewHolidayParams, NewTimeEntryParams, TimeEntry,
};
use crate::services::attendance_context::AttendanceMatcherContext;
use chrono::{NaiveDate, NaiveTime, Utc};
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

// ── Tests ──────────────────────────────────────────────────────────────

#[tokio::test]
async fn test_process_segment_attendance_no_shift_fallback() {
    let employee_id = Uuid::new_v4();
    let ws_id = Uuid::new_v4();

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

    let entry = TimeEntry::new(NewTimeEntryParams {
        id: Uuid::new_v4(),
        employment_id: employee_id,
        workspace_id: ws_id,
        shift_id: None,
        segment_id: None,
        punch_in_time: punch_in,
        punch_out_time: Some(punch_out),
        duration_minutes: Some(240),
        is_break_overtime: false,
        entry_type: crate::services::pairing::types::EntryType::Work,
        note: None,
        generation_metadata: None,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    });

    let mut daily_entries_map = HashMap::new();
    daily_entries_map.insert(Utc::now().date_naive(), vec![entry]);

    let attendance_rules = AttendanceRules {
        work_unit_minutes: 30,
        rounding_policy: crate::models::RoundingPolicy::Floor,
        early_arrival_strategy: EarlyArrivalStrategy::Block,
        grace_period_minutes: None,
    };

    let att_ctx = AttendanceMatcherContext::builder()
        .employee_id(employee_id)
        .tz(chrono_tz::UTC)
        .daily_entries_map(daily_entries_map)
        .emp_id_number("EMP001".to_string())
        .active_employments(vec![crate::models::employment::Employment::new(
            NewEmploymentParams {
                id: Uuid::new_v4(),
                workspace_id: ws_id,
                employee_id,
                employer_id: Uuid::new_v4(),
                primary_location_id: None,
                department_id: None,
                status: "active".to_string(),
                start_date: Utc::now().date_naive() - chrono::Duration::days(10),
                end_date: None,
                job_title: None,
                employment_type: "full_time".to_string(),
                manager_employment_id: None,
                created_at: Utc::now(),
                updated_at: Utc::now(),
                termination_reason: None,
                notes: None,
            },
        )])
        .primary_location_id(Uuid::new_v4())
        .build();

    let std_config = crate::domain::policy::resolver::StandardMonthConfig::new(
        Decimal::new(22, 0),
        Decimal::new(8, 0),
        "workspace".to_string(),
    );

    let mut policy_cache = HashMap::new();

    // resolver doesn't get called because day_shifts is empty
    let args = ProcessSegmentAttendanceArgs {
        ctx: &att_ctx,
        seg_start: Utc::now().date_naive(),
        seg_end: Utc::now().date_naive(),
        segment_total_base: Decimal::new(1000, 0),
        segment_ot_base: Decimal::new(1000, 0),
        std_config: &std_config,
        attendance_rules: &attendance_rules,
        seg_units_per_day: Decimal::new(16, 0),
        policy_cache: &mut policy_cache,
    };

    let _result = AttendanceMatcher::process_segment_attendance(args).await;
    // 240 mins / 30 mins = 8 units
    // Units now from summaries - assert_eq!(result.std_units, 8);
    // Units now from summaries - assert_eq!(result.ot_units, 0);
}

#[tokio::test]
async fn test_process_segment_attendance_with_shift_matching() {
    let employee_id = Uuid::new_v4();
    let ws_id = Uuid::new_v4();
    let shift_id = Uuid::new_v4();

    let date = Utc::now().date_naive();
    // Time entry: 09:00 to 18:00 (matching 9-18 shift)
    let punch_in = date.and_hms_opt(9, 0, 0).unwrap().and_utc();
    let punch_out = date.and_hms_opt(18, 0, 0).unwrap().and_utc();

    let entry = TimeEntry::new(NewTimeEntryParams {
        id: Uuid::new_v4(),
        employment_id: employee_id,
        workspace_id: ws_id,
        shift_id: None,
        segment_id: None,
        punch_in_time: punch_in,
        punch_out_time: Some(punch_out),
        duration_minutes: Some(540),
        is_break_overtime: false,
        entry_type: crate::services::pairing::types::EntryType::Work,
        note: None,
        generation_metadata: None,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    });

    let mut daily_entries_map = HashMap::new();
    daily_entries_map.insert(date, vec![entry]);

    // Define shift: 09:00 to 18:00, 1 hour (60 min) break
    let shift = RosterShift::new("Day Shift".to_string(), Some("D".to_string()));

    let mut shifts = HashMap::new();
    shifts.insert(shift_id, shift);

    let interval_id = Uuid::new_v4();
    let mut shift_intervals = HashMap::new();
    shift_intervals.insert(
        shift_id,
        vec![crate::cache::RosterInterval {
            interval_id,
            sort_order: 0,
            interval_name: None,
            start_time: NaiveTime::from_hms_opt(9, 0, 0).unwrap(),
            end_time: NaiveTime::from_hms_opt(18, 0, 0).unwrap(),
            break_duration_minutes: 60,
            break_start: Some(NaiveTime::from_hms_opt(12, 0, 0).unwrap()),
            break_end: Some(NaiveTime::from_hms_opt(13, 0, 0).unwrap()),
            is_break_auto_deduct: true,
            break_is_overtime: false,
            early_punch_in_grace_period_minutes: 0,
            allow_early_ot: false,
            max_early_ot_minutes: None,
            ot_aggregation_mode:
                crate::services::attendance_calculator::OtAggregationMode::Aggregate,
            is_night_shift: false,
        }],
    );

    let roster_data = RosterData {
        shifts,
        shift_intervals,
        ..Default::default()
    };

    let employment_id = Uuid::new_v4();
    let employer_id = Uuid::new_v4();
    let employment = crate::models::employment::Employment::new(NewEmploymentParams {
        id: employment_id,
        workspace_id: ws_id,
        employee_id,
        employer_id,
        primary_location_id: None,
        department_id: None,
        status: "active".to_string(),
        start_date: date - chrono::Duration::days(10),
        end_date: None,
        job_title: None,
        employment_type: "full_time".to_string(),
        manager_employment_id: None,
        created_at: Utc::now(),
        updated_at: Utc::now(),
        termination_reason: None,
        notes: None,
    });

    let mut payroll_roster_map = HashMap::new();
    payroll_roster_map.insert((employment_id, date), Some(shift_id));

    let attendance_rules = AttendanceRules {
        work_unit_minutes: 30,
        rounding_policy: crate::models::RoundingPolicy::Floor,
        early_arrival_strategy: EarlyArrivalStrategy::Block,
        grace_period_minutes: None,
    };

    let att_ctx = AttendanceMatcherContext::builder()
        .employee_id(employee_id)
        .tz(chrono_tz::UTC)
        .daily_entries_map(daily_entries_map)
        .roster_data(Arc::new(roster_data))
        .emp_id_number("EMP001".to_string())
        .payroll_roster_map(payroll_roster_map)
        .active_employments(vec![employment])
        .primary_location_id(Uuid::new_v4())
        .build();

    let std_config = crate::domain::policy::resolver::StandardMonthConfig::new(
        Decimal::new(22, 0),
        Decimal::new(8, 0),
        "workspace".to_string(),
    );

    let mut policy_cache = HashMap::new();

    let args = ProcessSegmentAttendanceArgs {
        ctx: &att_ctx,
        seg_start: date,
        seg_end: date,
        segment_total_base: Decimal::new(1000, 0),
        segment_ot_base: Decimal::new(1000, 0),
        std_config: &std_config,
        attendance_rules: &attendance_rules,
        seg_units_per_day: Decimal::new(16, 0),
        policy_cache: &mut policy_cache,
    };

    let _result = AttendanceMatcher::process_segment_attendance(args).await;
    // Total work duration = 9 hours (540 mins) minus 60 min break = 8 hours (480 mins) = 16 units of 30 mins.
    // Units now from summaries - assert_eq!(result.std_units, 16);
}

#[tokio::test]
async fn test_process_segment_attendance_paid_holiday_no_work() {
    let employee_id = Uuid::new_v4();
    let ws_id = Uuid::new_v4();

    let date = Utc::now().date_naive();

    // Holiday map with paid holiday
    let mut holidays_map = HashMap::new();
    holidays_map.insert(
        date,
        Holiday::new(NewHolidayParams {
            id: Uuid::new_v4(),
            workspace_id: ws_id,
            name: "New Year".to_string(),
            date,
            holiday_type: "full_day".to_string(),
            is_paid: true,
            leave_type_id: None,
            ot_multiplier: Some(Decimal::new(30, 1)),
            description: None,
            work_threshold_percent: None,
            is_credit_on_work: false,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            location_id: None,
        }),
    );

    let attendance_rules = AttendanceRules {
        work_unit_minutes: 30,
        rounding_policy: crate::models::RoundingPolicy::Floor,
        early_arrival_strategy: EarlyArrivalStrategy::Block,
        grace_period_minutes: None,
    };

    let att_ctx = AttendanceMatcherContext::builder()
        .employee_id(employee_id)
        .tz(chrono_tz::UTC)
        .holidays_map(Arc::new(holidays_map))
        .emp_id_number("EMP001".to_string())
        .active_employments(vec![crate::models::employment::Employment::new(
            NewEmploymentParams {
                id: Uuid::new_v4(),
                workspace_id: ws_id,
                employee_id,
                employer_id: Uuid::new_v4(),
                primary_location_id: None,
                department_id: None,
                status: "active".to_string(),
                start_date: date - chrono::Duration::days(10),
                end_date: None,
                job_title: None,
                employment_type: "full_time".to_string(),
                manager_employment_id: None,
                created_at: Utc::now(),
                updated_at: Utc::now(),
                termination_reason: None,
                notes: None,
            },
        )])
        .primary_location_id(Uuid::new_v4())
        .build();

    let std_config = crate::domain::policy::resolver::StandardMonthConfig::new(
        Decimal::new(22, 0),
        Decimal::new(8, 0),
        "workspace".to_string(),
    );

    let mut policy_cache = HashMap::new();

    let args = ProcessSegmentAttendanceArgs {
        ctx: &att_ctx,
        seg_start: date,
        seg_end: date,
        segment_total_base: Decimal::new(1000, 0),
        segment_ot_base: Decimal::new(1000, 0),
        std_config: &std_config,
        attendance_rules: &attendance_rules,
        seg_units_per_day: Decimal::new(16, 0), // 8 hours * 2 units/hr = 16 units
        policy_cache: &mut policy_cache,
    };

    let _result = AttendanceMatcher::process_segment_attendance(args).await;
    // Auto-credit standard day (16 units) because it's a paid holiday with no attendance
    // Units now from summaries - assert_eq!(result.std_units, 16);
}

#[tokio::test]
async fn test_process_segment_attendance_holiday_work_compensation() {
    let employee_id = Uuid::new_v4();
    let ws_id = Uuid::new_v4();
    let shift_id = Uuid::new_v4();

    let date = Utc::now().date_naive();
    // Time entry: 09:00 to 18:00 (matching 9-18 shift)
    let punch_in = date.and_hms_opt(9, 0, 0).unwrap().and_utc();
    let punch_out = date.and_hms_opt(18, 0, 0).unwrap().and_utc();

    let entry = TimeEntry::new(NewTimeEntryParams {
        id: Uuid::new_v4(),
        employment_id: employee_id,
        workspace_id: ws_id,
        shift_id: None,
        segment_id: None,
        punch_in_time: punch_in,
        punch_out_time: Some(punch_out),
        duration_minutes: Some(540),
        is_break_overtime: false,
        entry_type: crate::services::pairing::types::EntryType::Work,
        note: None,
        generation_metadata: None,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    });

    let mut daily_entries_map = HashMap::new();
    daily_entries_map.insert(date, vec![entry]);

    // Define shift: 09:00 to 18:00, 1 hour break
    let shift = RosterShift::new("Day Shift".to_string(), Some("D".to_string()));

    let mut shifts = HashMap::new();
    shifts.insert(shift_id, shift);

    let interval_id = Uuid::new_v4();
    let mut shift_intervals = HashMap::new();
    shift_intervals.insert(
        shift_id,
        vec![crate::cache::RosterInterval {
            interval_id,
            sort_order: 0,
            interval_name: None,
            start_time: NaiveTime::from_hms_opt(9, 0, 0).unwrap(),
            end_time: NaiveTime::from_hms_opt(18, 0, 0).unwrap(),
            break_duration_minutes: 60,
            break_start: Some(NaiveTime::from_hms_opt(12, 0, 0).unwrap()),
            break_end: Some(NaiveTime::from_hms_opt(13, 0, 0).unwrap()),
            is_break_auto_deduct: true,
            break_is_overtime: false,
            early_punch_in_grace_period_minutes: 0,
            allow_early_ot: false,
            max_early_ot_minutes: None,
            ot_aggregation_mode:
                crate::services::attendance_calculator::OtAggregationMode::Aggregate,
            is_night_shift: false,
        }],
    );

    let roster_data = RosterData {
        shifts,
        shift_intervals,
        ..Default::default()
    };

    let employment_id = Uuid::new_v4();
    let employer_id = Uuid::new_v4();
    let employment = crate::models::employment::Employment::new(NewEmploymentParams {
        id: employment_id,
        workspace_id: ws_id,
        employee_id,
        employer_id,
        primary_location_id: None,
        department_id: None,
        status: "active".to_string(),
        start_date: date - chrono::Duration::days(10),
        end_date: None,
        job_title: None,
        employment_type: "full_time".to_string(),
        manager_employment_id: None,
        created_at: Utc::now(),
        updated_at: Utc::now(),
        termination_reason: None,
        notes: None,
    });

    let mut payroll_roster_map = HashMap::new();
    payroll_roster_map.insert((employment_id, date), Some(shift_id));

    let mut holidays_map = HashMap::new();
    holidays_map.insert(
        date,
        Holiday::new(NewHolidayParams {
            id: Uuid::new_v4(),
            workspace_id: ws_id,
            name: "New Year".to_string(),
            date,
            holiday_type: "full_day".to_string(),
            is_paid: true,
            leave_type_id: None,
            ot_multiplier: Some(Decimal::new(30, 1)),
            description: None,
            work_threshold_percent: None,
            is_credit_on_work: true,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            location_id: None,
        }),
    );

    let attendance_rules = AttendanceRules {
        work_unit_minutes: 30,
        rounding_policy: crate::models::RoundingPolicy::Floor,
        early_arrival_strategy: EarlyArrivalStrategy::Block,
        grace_period_minutes: None,
    };

    let att_ctx = AttendanceMatcherContext::builder()
        .employee_id(employee_id)
        .tz(chrono_tz::UTC)
        .daily_entries_map(daily_entries_map)
        .holidays_map(Arc::new(holidays_map))
        .roster_data(Arc::new(roster_data))
        .emp_id_number("EMP001".to_string())
        .payroll_roster_map(payroll_roster_map)
        .active_employments(vec![employment])
        .primary_location_id(Uuid::new_v4())
        .build();

    let std_config = crate::domain::policy::resolver::StandardMonthConfig::new(
        Decimal::new(22, 0),
        Decimal::new(8, 0),
        "workspace".to_string(),
    );

    let mut policy_cache = HashMap::new();

    let args = ProcessSegmentAttendanceArgs {
        ctx: &att_ctx,
        seg_start: date,
        seg_end: date,
        segment_total_base: Decimal::new(1000, 0),
        segment_ot_base: Decimal::new(1000, 0),
        std_config: &std_config,
        attendance_rules: &attendance_rules,
        seg_units_per_day: Decimal::new(16, 0), // 8 hours = 16 units
        policy_cache: &mut policy_cache,
    };

    let result = AttendanceMatcher::process_segment_attendance(args).await;
    // worked units: 16 (8 hours)
    // is_credit_on_work adds seg_units_per_day (16) => 32 std units
    // Units now from summaries - assert_eq!(result.std_units, 32);
    // worked fraction = 16 / 16 = 1.0
    // daily rate = 1000 / 22 = 45.4545...
    // multiplier - 1.0 = 2.0
    // holiday premium = 1.0 * (1000 / 22) * 2.0 = 90.90909090909090909090909091
    // Holiday work premium is now computed in calculate_payslip_with_context
    // from summary data (standard_minutes) instead of in the matcher.
    assert_eq!(result.holiday_work_premium, Decimal::ZERO);
}

#[tokio::test]
async fn test_process_segment_attendance_with_overtime_pay() {
    let employee_id = Uuid::new_v4();
    let ws_id = Uuid::new_v4();
    let shift_id = Uuid::new_v4();
    let policy_id = Uuid::new_v4();

    let date = NaiveDate::from_ymd_opt(2026, 1, 26).unwrap(); // Monday
                                                              // Time entry: 09:00 to 20:00 (matching 9-18 shift) -> 2 hours late OT
    let punch_in = date.and_hms_opt(9, 0, 0).unwrap().and_utc();
    let punch_out = date.and_hms_opt(20, 0, 0).unwrap().and_utc();

    let entry = TimeEntry::new(NewTimeEntryParams {
        id: Uuid::new_v4(),
        employment_id: employee_id,
        workspace_id: ws_id,
        shift_id: None,
        segment_id: None,
        punch_in_time: punch_in,
        punch_out_time: Some(punch_out),
        duration_minutes: Some(660),
        is_break_overtime: false,
        entry_type: crate::services::pairing::types::EntryType::Work,
        note: None,
        generation_metadata: None,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    });

    let mut daily_entries_map = HashMap::new();
    daily_entries_map.insert(date, vec![entry]);

    // Define shift: 09:00 to 18:00, 1 hour break
    let shift = RosterShift::new("Day Shift".to_string(), Some("D".to_string()));

    let mut shifts = HashMap::new();
    shifts.insert(shift_id, shift);

    let interval_id = Uuid::new_v4();
    let mut shift_intervals = HashMap::new();
    shift_intervals.insert(
        shift_id,
        vec![crate::cache::RosterInterval {
            interval_id,
            sort_order: 0,
            interval_name: None,
            start_time: NaiveTime::from_hms_opt(9, 0, 0).unwrap(),
            end_time: NaiveTime::from_hms_opt(18, 0, 0).unwrap(),
            break_duration_minutes: 60,
            break_start: Some(NaiveTime::from_hms_opt(12, 0, 0).unwrap()),
            break_end: Some(NaiveTime::from_hms_opt(13, 0, 0).unwrap()),
            is_break_auto_deduct: true,
            break_is_overtime: false,
            early_punch_in_grace_period_minutes: 0,
            allow_early_ot: false,
            max_early_ot_minutes: None,
            ot_aggregation_mode:
                crate::services::attendance_calculator::OtAggregationMode::Aggregate,
            is_night_shift: false,
        }],
    );

    let roster_data = RosterData {
        shifts,
        shift_intervals,
        ..Default::default()
    };

    let employment_id = Uuid::new_v4();
    let employer_id = Uuid::new_v4();
    let employment = crate::models::employment::Employment::new(NewEmploymentParams {
        id: employment_id,
        workspace_id: ws_id,
        employee_id,
        employer_id,
        primary_location_id: None,
        department_id: None,
        status: "active".to_string(),
        start_date: date - chrono::Duration::days(10),
        end_date: None,
        job_title: None,
        employment_type: "full_time".to_string(),
        manager_employment_id: None,
        created_at: Utc::now(),
        updated_at: Utc::now(),
        termination_reason: None,
        notes: None,
    });

    let mut payroll_roster_map = HashMap::new();
    payroll_roster_map.insert((employment_id, date), Some(shift_id));

    let attendance_rules = AttendanceRules {
        work_unit_minutes: 30,
        rounding_policy: crate::models::RoundingPolicy::Floor,
        early_arrival_strategy: EarlyArrivalStrategy::Block,
        grace_period_minutes: None,
    };

    let rules = vec![crate::models::OvertimeRule::new(
        crate::models::NewOvertimeRuleParams {
            id: Uuid::new_v4(),
            policy_id,
            priority: 1,
            multiplier: Decimal::new(20, 1), // 2.0x
            time_start: Some(NaiveTime::from_hms_opt(18, 0, 0).unwrap()),
            time_end: Some(NaiveTime::from_hms_opt(22, 0, 0).unwrap()),
            days_of_week: 127,
            is_holiday: false,
            created_at: None,
            updated_at: None,
        },
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
        crate::models::NewOvertimePolicyAssignmentParams {
            id: Uuid::new_v4(),
            workspace_id: Uuid::new_v4(),
            assignee_type: "employee".to_string(),
            assignee_id: Some(employee_id),
            policy_id,
            effective_from: date,
            effective_to: None,
            created_at: Some(chrono::Utc::now()),
        },
    )];

    let overtime_data = crate::repositories::overtime_policy_repository::WorkspaceOvertimeData {
        policies,
        assignments,
    };

    let att_ctx = AttendanceMatcherContext::builder()
        .employee_id(employee_id)
        .tz(chrono_tz::UTC)
        .daily_entries_map(daily_entries_map)
        .roster_data(Arc::new(roster_data))
        .emp_id_number("EMP001".to_string())
        .payroll_roster_map(payroll_roster_map)
        .overtime_data(Arc::new(overtime_data))
        .active_employments(vec![employment])
        .primary_location_id(Uuid::new_v4())
        .build();

    let std_config = crate::domain::policy::resolver::StandardMonthConfig::new(
        Decimal::new(22, 0),
        Decimal::new(8, 0),
        "workspace".to_string(),
    );

    let mut policy_cache = HashMap::new();
    let args = ProcessSegmentAttendanceArgs {
        ctx: &att_ctx,
        seg_start: date,
        seg_end: date,
        segment_total_base: Decimal::new(1760, 0),
        segment_ot_base: Decimal::new(1760, 0), // hourly_rate = 1760 / (22*8) = 10
        std_config: &std_config,
        attendance_rules: &attendance_rules,
        seg_units_per_day: Decimal::new(16, 0),
        policy_cache: &mut policy_cache,
    };

    let result = AttendanceMatcher::process_segment_attendance(args).await;
    // 8 hours std + 2 hours OT
    // Units now from summaries - assert_eq!(result.std_units, 16);
    // Units now from summaries - assert_eq!(result.ot_units, 4); // 2 hours = 4 units
    // OT pay is now computed in calculate_payslip_with_context from summary
    // data. The matcher's per-entry OT pay chain is preserved for Formula Engine.
    assert_eq!(result.ot_pay, Decimal::ZERO);
}

#[tokio::test]
async fn test_process_segment_attendance_with_shift_break_is_overtime(
) -> Result<(), Box<dyn std::error::Error>> {
    let employee_id = Uuid::new_v4();
    let ws_id = Uuid::new_v4();
    let shift_id = Uuid::new_v4();

    let date = Utc::now().date_naive();
    // Time entry: 09:00 to 18:00 (matching 9-18 shift)
    // Employee took no break (actual break minutes = 0)
    let punch_in = date.and_hms_opt(9, 0, 0).ok_or("invalid time")?.and_utc();
    let punch_out = date.and_hms_opt(18, 0, 0).ok_or("invalid time")?.and_utc();

    let entry = TimeEntry::new(NewTimeEntryParams {
        id: Uuid::new_v4(),
        employment_id: employee_id,
        workspace_id: ws_id,
        shift_id: None,
        segment_id: None,
        punch_in_time: punch_in,
        punch_out_time: Some(punch_out),
        duration_minutes: Some(540),
        is_break_overtime: false,
        entry_type: crate::services::pairing::types::EntryType::Work,
        note: None,
        generation_metadata: None,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    });

    let mut daily_entries_map = HashMap::new();
    daily_entries_map.insert(date, vec![entry]);

    // Define shift: 09:00 to 18:00, 1 hour (60 min) break, break_is_overtime = true
    let shift = RosterShift::new("Day Shift".to_string(), Some("D".to_string()));

    let mut shifts = HashMap::new();
    shifts.insert(shift_id, shift);

    let interval_id = Uuid::new_v4();
    let mut shift_intervals = HashMap::new();
    shift_intervals.insert(
        shift_id,
        vec![crate::cache::RosterInterval {
            interval_id,
            sort_order: 0,
            interval_name: None,
            start_time: NaiveTime::from_hms_opt(9, 0, 0).ok_or("invalid start time")?,
            end_time: NaiveTime::from_hms_opt(18, 0, 0).ok_or("invalid end time")?,
            break_duration_minutes: 60,
            break_start: Some(NaiveTime::from_hms_opt(12, 0, 0).ok_or("invalid break start")?),
            break_end: Some(NaiveTime::from_hms_opt(13, 0, 0).ok_or("invalid break end")?),
            is_break_auto_deduct: true,
            break_is_overtime: true,
            early_punch_in_grace_period_minutes: 0,
            allow_early_ot: false,
            max_early_ot_minutes: None,
            ot_aggregation_mode:
                crate::services::attendance_calculator::OtAggregationMode::Aggregate,
            is_night_shift: false,
        }],
    );

    let roster_data = RosterData {
        shifts,
        shift_intervals,
        ..Default::default()
    };

    let employment_id = Uuid::new_v4();
    let employer_id = Uuid::new_v4();
    let employment = crate::models::employment::Employment::new(NewEmploymentParams {
        id: employment_id,
        workspace_id: ws_id,
        employee_id,
        employer_id,
        primary_location_id: None,
        department_id: None,
        status: "active".to_string(),
        start_date: date - chrono::Duration::days(10),
        end_date: None,
        job_title: None,
        employment_type: "full_time".to_string(),
        manager_employment_id: None,
        created_at: Utc::now(),
        updated_at: Utc::now(),
        termination_reason: None,
        notes: None,
    });

    let mut payroll_roster_map = HashMap::new();
    payroll_roster_map.insert((employment_id, date), Some(shift_id));

    let attendance_rules = AttendanceRules {
        work_unit_minutes: 30,
        rounding_policy: crate::models::RoundingPolicy::Floor,
        early_arrival_strategy: EarlyArrivalStrategy::Block,
        grace_period_minutes: None,
    };

    let att_ctx = AttendanceMatcherContext::builder()
        .employee_id(employee_id)
        .tz(chrono_tz::UTC)
        .daily_entries_map(daily_entries_map)
        .roster_data(Arc::new(roster_data))
        .emp_id_number("EMP001".to_string())
        .payroll_roster_map(payroll_roster_map)
        .active_employments(vec![employment])
        .primary_location_id(Uuid::new_v4())
        .build();

    let std_config = crate::domain::policy::resolver::StandardMonthConfig::new(
        Decimal::new(22, 0),
        Decimal::new(8, 0),
        "workspace".to_string(),
    );

    let mut policy_cache = HashMap::new();

    let args = ProcessSegmentAttendanceArgs {
        ctx: &att_ctx,
        seg_start: date,
        seg_end: date,
        segment_total_base: Decimal::new(1000, 0),
        segment_ot_base: Decimal::new(1000, 0),
        std_config: &std_config,
        attendance_rules: &attendance_rules,
        seg_units_per_day: Decimal::new(16, 0),
        policy_cache: &mut policy_cache,
    };

    let _result = AttendanceMatcher::process_segment_attendance(args).await;
    // Units now from summaries - assert_eq!(result.std_units, 16);
    // Units now from summaries - assert_eq!(result.ot_units, 2);
    Ok(())
}

// ── Fix 3: resolve_day_context returns Option ──────────────────────────

#[tokio::test]
async fn test_resolve_day_context_no_employments_graceful_skip() {
    let employee_id = Uuid::new_v4();
    let date = NaiveDate::from_ymd_opt(2026, 7, 30).unwrap();

    // Create time entries for an employee who has NO active employments
    let punch_in = date.and_hms_opt(9, 0, 0).unwrap().and_utc();
    let punch_out = date.and_hms_opt(18, 0, 0).unwrap().and_utc();

    let entry = TimeEntry::new(NewTimeEntryParams {
        id: Uuid::new_v4(),
        employment_id: employee_id,
        workspace_id: Uuid::new_v4(),
        shift_id: None,
        segment_id: None,
        punch_in_time: punch_in,
        punch_out_time: Some(punch_out),
        duration_minutes: Some(540),
        is_break_overtime: false,
        entry_type: crate::services::pairing::types::EntryType::Work,
        note: None,
        generation_metadata: None,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    });

    let mut daily_entries_map = HashMap::new();
    daily_entries_map.insert(date, vec![entry]);

    let attendance_rules = AttendanceRules {
        work_unit_minutes: 30,
        rounding_policy: crate::models::RoundingPolicy::Floor,
        early_arrival_strategy: EarlyArrivalStrategy::Block,
        grace_period_minutes: None,
    };

    // Context with NO active employments — this is the key setup
    let att_ctx = AttendanceMatcherContext::builder()
        .employee_id(employee_id)
        .tz(chrono_tz::UTC)
        .daily_entries_map(daily_entries_map)
        .emp_id_number("EMP001".to_string())
        .primary_location_id(Uuid::new_v4())
        // .active_employments(vec![]) — intentionally empty
        .build();

    let std_config = crate::domain::policy::resolver::StandardMonthConfig::new(
        Decimal::new(22, 0),
        Decimal::new(8, 0),
        "workspace".to_string(),
    );

    let mut policy_cache = HashMap::new();

    let args = ProcessSegmentAttendanceArgs {
        ctx: &att_ctx,
        seg_start: date,
        seg_end: date,
        segment_total_base: Decimal::new(1000, 0),
        segment_ot_base: Decimal::new(1000, 0),
        std_config: &std_config,
        attendance_rules: &attendance_rules,
        seg_units_per_day: Decimal::new(16, 0),
        policy_cache: &mut policy_cache,
    };

    let result = AttendanceMatcher::process_segment_attendance(args).await;
    // No active employment → date loop skips → zero units (no panic, no Uuid::nil)
    // Units now from summaries - assert_eq!(result.std_units, 0);
    // Units now from summaries - assert_eq!(result.ot_units, 0);
    assert_eq!(result.ot_pay, Decimal::ZERO);
}

#[tokio::test]
async fn test_resolve_day_context_inactive_employment_falls_back_to_first() {
    let employee_id = Uuid::new_v4();
    let ws_id = Uuid::new_v4();
    let employer_id = Uuid::new_v4();
    let employment_id = Uuid::new_v4();
    let date = NaiveDate::from_ymd_opt(2026, 7, 30).unwrap();

    // Time entry: 09:00–13:00 = 4 hours = 8 units
    let punch_in = date.and_hms_opt(9, 0, 0).unwrap().and_utc();
    let punch_out = date.and_hms_opt(13, 0, 0).unwrap().and_utc();

    let entry = TimeEntry::new(NewTimeEntryParams {
        id: Uuid::new_v4(),
        employment_id: employee_id,
        workspace_id: ws_id,
        shift_id: None,
        segment_id: None,
        punch_in_time: punch_in,
        punch_out_time: Some(punch_out),
        duration_minutes: Some(240),
        is_break_overtime: false,
        entry_type: crate::services::pairing::types::EntryType::Work,
        note: None,
        generation_metadata: None,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    });

    let mut daily_entries_map = HashMap::new();
    daily_entries_map.insert(date, vec![entry]);

    // Employment exists but is "terminated" and ended before current_date —
    // the active+date filter rejects it, but the or_else(|| .first()) fallback
    // still picks it up as a last-resort employment context.
    let employment = crate::models::employment::Employment::new(NewEmploymentParams {
        id: employment_id,
        workspace_id: ws_id,
        employee_id,
        employer_id,
        primary_location_id: None,
        department_id: None,
        status: "terminated".to_string(),
        start_date: date - chrono::Duration::days(30),
        end_date: Some(date - chrono::Duration::days(5)),
        job_title: None,
        employment_type: "full_time".to_string(),
        manager_employment_id: None,
        created_at: Utc::now(),
        updated_at: Utc::now(),
        termination_reason: None,
        notes: None,
    });

    let attendance_rules = AttendanceRules {
        work_unit_minutes: 30,
        rounding_policy: crate::models::RoundingPolicy::Floor,
        early_arrival_strategy: EarlyArrivalStrategy::Block,
        grace_period_minutes: None,
    };

    let att_ctx = AttendanceMatcherContext::builder()
        .employee_id(employee_id)
        .tz(chrono_tz::UTC)
        .daily_entries_map(daily_entries_map)
        .emp_id_number("EMP001".to_string())
        .active_employments(vec![employment])
        .primary_location_id(Uuid::new_v4())
        .build();

    let std_config = crate::domain::policy::resolver::StandardMonthConfig::new(
        Decimal::new(22, 0),
        Decimal::new(8, 0),
        "workspace".to_string(),
    );

    let mut policy_cache = HashMap::new();

    let args = ProcessSegmentAttendanceArgs {
        ctx: &att_ctx,
        seg_start: date,
        seg_end: date,
        segment_total_base: Decimal::new(1000, 0),
        segment_ot_base: Decimal::new(1000, 0),
        std_config: &std_config,
        attendance_rules: &attendance_rules,
        seg_units_per_day: Decimal::new(16, 0),
        policy_cache: &mut policy_cache,
    };

    let _result = AttendanceMatcher::process_segment_attendance(args).await;
    // Terminated employment is used via or_else(|| .first()) fallback →
    // processing continues (no-shift fallback: 240 mins / 30 = 8 units)
    // Units now from summaries - assert_eq!(result.std_units, 8);
    // Units now from summaries - assert_eq!(result.ot_units, 0);
}

#[tokio::test]
async fn test_resolve_day_context_with_active_employment_still_works() {
    let employee_id = Uuid::new_v4();
    let ws_id = Uuid::new_v4();
    let employer_id = Uuid::new_v4();
    let employment_id = Uuid::new_v4();
    let date = NaiveDate::from_ymd_opt(2026, 7, 30).unwrap();

    // Time entry: 09:00–13:00 = 4 hours = 8 units
    let punch_in = date.and_hms_opt(9, 0, 0).unwrap().and_utc();
    let punch_out = date.and_hms_opt(13, 0, 0).unwrap().and_utc();

    let entry = TimeEntry::new(NewTimeEntryParams {
        id: Uuid::new_v4(),
        employment_id: employee_id,
        workspace_id: ws_id,
        shift_id: None,
        segment_id: None,
        punch_in_time: punch_in,
        punch_out_time: Some(punch_out),
        duration_minutes: Some(240),
        is_break_overtime: false,
        entry_type: crate::services::pairing::types::EntryType::Work,
        note: None,
        generation_metadata: None,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    });

    let mut daily_entries_map = HashMap::new();
    daily_entries_map.insert(date, vec![entry]);

    let employment = crate::models::employment::Employment::new(NewEmploymentParams {
        id: employment_id,
        workspace_id: ws_id,
        employee_id,
        employer_id,
        primary_location_id: None,
        department_id: None,
        status: "active".to_string(),
        start_date: date - chrono::Duration::days(10),
        end_date: None,
        job_title: None,
        employment_type: "full_time".to_string(),
        manager_employment_id: None,
        created_at: Utc::now(),
        updated_at: Utc::now(),
        termination_reason: None,
        notes: None,
    });

    let attendance_rules = AttendanceRules {
        work_unit_minutes: 30,
        rounding_policy: crate::models::RoundingPolicy::Floor,
        early_arrival_strategy: EarlyArrivalStrategy::Block,
        grace_period_minutes: None,
    };

    let att_ctx = AttendanceMatcherContext::builder()
        .employee_id(employee_id)
        .tz(chrono_tz::UTC)
        .daily_entries_map(daily_entries_map)
        .emp_id_number("EMP001".to_string())
        .active_employments(vec![employment])
        .primary_location_id(Uuid::new_v4())
        .build();

    let std_config = crate::domain::policy::resolver::StandardMonthConfig::new(
        Decimal::new(22, 0),
        Decimal::new(8, 0),
        "workspace".to_string(),
    );

    let mut policy_cache = HashMap::new();

    let args = ProcessSegmentAttendanceArgs {
        ctx: &att_ctx,
        seg_start: date,
        seg_end: date,
        segment_total_base: Decimal::new(1000, 0),
        segment_ot_base: Decimal::new(1000, 0),
        std_config: &std_config,
        attendance_rules: &attendance_rules,
        seg_units_per_day: Decimal::new(16, 0),
        policy_cache: &mut policy_cache,
    };

    let _result = AttendanceMatcher::process_segment_attendance(args).await;
    // Active employment exists → no-shift fallback processes entries normally
    // 240 mins / 30 mins = 8 units
    // Units now from summaries - assert_eq!(result.std_units, 8);
    // Units now from summaries - assert_eq!(result.ot_units, 0);
}

// ── Pure scoring helper tests ───────────────────────────────────────────

mod scoring_helpers {
    use super::super::scoring::{
        compute_early_minutes, compute_late_minutes, detect_absence, resolve_shift_window,
        ScoringContext,
    };
    use crate::cache::RosterInterval;
    use chrono::{NaiveDate, NaiveTime, Timelike};
    use std::collections::HashMap;

    fn tz() -> chrono_tz::Tz {
        chrono_tz::UTC
    }

    fn dt(date: NaiveDate, h: u32, m: u32) -> chrono::DateTime<chrono_tz::Tz> {
        use chrono::TimeZone;
        tz().from_local_datetime(&date.and_hms_opt(h, m, 0).unwrap())
            .unwrap()
    }

    fn test_interval(start: NaiveTime, end: NaiveTime, grace: i32) -> RosterInterval {
        RosterInterval {
            interval_id: uuid::Uuid::new_v4(),
            sort_order: 0,
            interval_name: None,
            start_time: start,
            end_time: end,
            break_duration_minutes: 0,
            break_start: None,
            break_end: None,
            is_break_auto_deduct: false,
            break_is_overtime: false,
            early_punch_in_grace_period_minutes: grace,
            allow_early_ot: false,
            max_early_ot_minutes: None,
            ot_aggregation_mode:
                crate::services::attendance_calculator::OtAggregationMode::Aggregate,
            is_night_shift: false,
        }
    }

    // ── compute_late_minutes ──

    #[test]
    fn late_minutes_on_time_returns_zero() {
        let date = NaiveDate::from_ymd_opt(2026, 1, 26).unwrap();
        assert_eq!(
            compute_late_minutes(dt(date, 9, 0), dt(date, 9, 0), 5, false),
            0
        );
    }

    #[test]
    fn late_minutes_within_grace_returns_zero() {
        let date = NaiveDate::from_ymd_opt(2026, 1, 26).unwrap();
        assert_eq!(
            compute_late_minutes(dt(date, 9, 5), dt(date, 9, 0), 5, false),
            0
        );
    }

    #[test]
    fn late_minutes_beyond_grace_returns_diff() {
        let date = NaiveDate::from_ymd_opt(2026, 1, 26).unwrap();
        assert_eq!(
            compute_late_minutes(dt(date, 9, 15), dt(date, 9, 0), 5, false),
            10
        );
    }

    #[test]
    fn late_minutes_early_arrival_returns_zero() {
        let date = NaiveDate::from_ymd_opt(2026, 1, 26).unwrap();
        assert_eq!(
            compute_late_minutes(dt(date, 8, 45), dt(date, 9, 0), 5, false),
            0
        );
    }

    #[test]
    fn late_minutes_holiday_returns_zero() {
        let date = NaiveDate::from_ymd_opt(2026, 1, 26).unwrap();
        assert_eq!(
            compute_late_minutes(dt(date, 10, 0), dt(date, 9, 0), 0, true),
            0
        );
    }

    #[test]
    fn late_minutes_zero_grace() {
        let date = NaiveDate::from_ymd_opt(2026, 1, 26).unwrap();
        assert_eq!(
            compute_late_minutes(dt(date, 9, 1), dt(date, 9, 0), 0, false),
            1
        );
    }

    // ── compute_early_minutes ──

    #[test]
    fn early_minutes_not_early_returns_zero() {
        let date = NaiveDate::from_ymd_opt(2026, 1, 26).unwrap();
        assert_eq!(
            compute_early_minutes(dt(date, 18, 0), dt(date, 18, 0), false),
            0
        );
    }

    #[test]
    fn early_minutes_left_early_returns_diff() {
        let date = NaiveDate::from_ymd_opt(2026, 1, 26).unwrap();
        assert_eq!(
            compute_early_minutes(dt(date, 17, 45), dt(date, 18, 0), false),
            15
        );
    }

    #[test]
    fn early_minutes_stayed_late_returns_zero() {
        let date = NaiveDate::from_ymd_opt(2026, 1, 26).unwrap();
        assert_eq!(
            compute_early_minutes(dt(date, 19, 0), dt(date, 18, 0), false),
            0
        );
    }

    #[test]
    fn early_minutes_holiday_returns_zero() {
        let date = NaiveDate::from_ymd_opt(2026, 1, 26).unwrap();
        assert_eq!(
            compute_early_minutes(dt(date, 17, 0), dt(date, 18, 0), true),
            0
        );
    }

    // ── detect_absence ──

    #[test]
    fn absence_no_violation() {
        let date = NaiveDate::from_ymd_opt(2026, 1, 26).unwrap();
        assert!(!detect_absence(
            dt(date, 9, 0),
            dt(date, 18, 0),
            dt(date, 9, 0),
            dt(date, 18, 0),
            5,
            false
        ));
    }

    #[test]
    fn absence_late_beyond_grace() {
        let date = NaiveDate::from_ymd_opt(2026, 1, 26).unwrap();
        assert!(detect_absence(
            dt(date, 9, 10),
            dt(date, 18, 0),
            dt(date, 9, 0),
            dt(date, 18, 0),
            5,
            false
        ));
    }

    #[test]
    fn absence_early_departure() {
        let date = NaiveDate::from_ymd_opt(2026, 1, 26).unwrap();
        assert!(detect_absence(
            dt(date, 9, 0),
            dt(date, 17, 30),
            dt(date, 9, 0),
            dt(date, 18, 0),
            5,
            false
        ));
    }

    #[test]
    fn absence_holiday_always_false() {
        let date = NaiveDate::from_ymd_opt(2026, 1, 26).unwrap();
        assert!(!detect_absence(
            dt(date, 11, 0),
            dt(date, 15, 0),
            dt(date, 9, 0),
            dt(date, 18, 0),
            0,
            true
        ));
    }

    #[test]
    fn absence_both_late_and_early() {
        let date = NaiveDate::from_ymd_opt(2026, 1, 26).unwrap();
        assert!(detect_absence(
            dt(date, 9, 20),
            dt(date, 17, 0),
            dt(date, 9, 0),
            dt(date, 18, 0),
            5,
            false
        ));
    }

    // ── resolve_shift_window ──

    #[test]
    fn shift_window_normal_day() {
        let date = NaiveDate::from_ymd_opt(2026, 1, 26).unwrap();
        let interval = test_interval(
            NaiveTime::from_hms_opt(9, 0, 0).unwrap(),
            NaiveTime::from_hms_opt(18, 0, 0).unwrap(),
            0,
        );
        let (start, end) = resolve_shift_window(tz(), date, &interval).unwrap();
        assert_eq!(start, dt(date, 9, 0));
        assert_eq!(end, dt(date, 18, 0));
    }

    #[test]
    fn shift_window_overnight_adds_day() {
        let date = NaiveDate::from_ymd_opt(2026, 1, 26).unwrap();
        let interval = test_interval(
            NaiveTime::from_hms_opt(22, 0, 0).unwrap(),
            NaiveTime::from_hms_opt(6, 0, 0).unwrap(),
            0,
        );
        let (start, end) = resolve_shift_window(tz(), date, &interval).unwrap();
        assert_eq!(start, dt(date, 22, 0));
        assert_eq!(end, dt(date + chrono::Duration::days(1), 6, 0));
    }

    // ── ScoringContext construction ──

    #[test]
    fn scoring_context_from_arc_refs() {
        use std::sync::Arc;

        let holidays = Arc::new(HashMap::new());
        let catalog = Arc::new(HashMap::new());
        let conditions: Vec<crate::models::WorkingCondition> = vec![];

        let ctx = ScoringContext::new(chrono_tz::UTC, &conditions, &holidays, &catalog);
        assert_eq!(ctx.tz(), chrono_tz::UTC);
        assert!(ctx.holidays_map().is_empty());
    }

    #[test]
    fn scoring_context_from_plain_refs() {
        let holidays = HashMap::new();
        let catalog = HashMap::new();
        let conditions: Vec<crate::models::WorkingCondition> = vec![];

        let ctx = ScoringContext::new(chrono_tz::UTC, &conditions, &holidays, &catalog);
        assert_eq!(ctx.tz(), chrono_tz::UTC);
    }

    // ── resolve_shift_window DST tests ──────────────────────────────────────

    #[test]
    fn resolve_shift_window_overnight_spring_forward() {
        // Shift 22:00-06:00 on Mar 7 (day before spring-forward in NY).
        // End should be 06:00 EDT on Mar 8, duration = 7h (not 8h).
        let ny_tz: chrono_tz::Tz = "America/New_York".parse().unwrap();
        let date = NaiveDate::from_ymd_opt(2026, 3, 7).unwrap();
        let interval = test_interval(
            NaiveTime::from_hms_opt(22, 0, 0).unwrap(),
            NaiveTime::from_hms_opt(6, 0, 0).unwrap(),
            0,
        );

        let (start, end) = resolve_shift_window(ny_tz, date, &interval).expect("should resolve");

        assert_eq!(start.date_naive(), date);
        assert_eq!(start.hour(), 22);

        // Wall-clock preserved: 06:00 on Mar 8 (EDT)
        assert_eq!(
            end.date_naive(),
            NaiveDate::from_ymd_opt(2026, 3, 8).unwrap()
        );
        assert_eq!(end.hour(), 6);

        // Duration: 7h (spring-forward lost 1h)
        assert_eq!(end.signed_duration_since(start).num_hours(), 7);
    }

    #[test]
    fn resolve_shift_window_overnight_fall_back() {
        // Shift 22:00-06:00 on Oct 31 (day before fall-back in NY).
        // End should be 06:00 EST on Nov 1, duration = 9h (not 8h).
        let ny_tz: chrono_tz::Tz = "America/New_York".parse().unwrap();
        let date = NaiveDate::from_ymd_opt(2026, 10, 31).unwrap();
        let interval = test_interval(
            NaiveTime::from_hms_opt(22, 0, 0).unwrap(),
            NaiveTime::from_hms_opt(6, 0, 0).unwrap(),
            0,
        );

        let (start, end) = resolve_shift_window(ny_tz, date, &interval).expect("should resolve");

        assert_eq!(start.date_naive(), date);
        assert_eq!(
            end.date_naive(),
            NaiveDate::from_ymd_opt(2026, 11, 1).unwrap()
        );
        assert_eq!(end.hour(), 6);

        // Duration: 9h (fall-back gained 1h)
        assert_eq!(end.signed_duration_since(start).num_hours(), 9);
    }
}
