use super::*;
use crate::cache::{RosterData, RosterShift};
use crate::models::enums::EarlyArrivalStrategy;
use crate::models::{AttendanceRules, Holiday, TimeEntry};
use crate::services::attendance_context::AttendanceContext;
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

    let entry = TimeEntry::new(
        Uuid::new_v4(),
        employee_id,
        ws_id,
        None,
        None,
        punch_in,
        Some(punch_out),
        Some(240),
        false,
        crate::services::pairing::types::EntryType::Work,
        None,
        None,
        Utc::now(),
        Utc::now(),
    );

    let mut daily_entries_map = HashMap::new();
    daily_entries_map.insert(Utc::now().date_naive(), vec![entry]);

    let attendance_rules = AttendanceRules {
        work_unit_minutes: 30,
        rounding_policy: crate::models::RoundingPolicy::Floor,
        early_arrival_strategy: EarlyArrivalStrategy::Block,
        grace_period_minutes: None,
    };

    let att_ctx = AttendanceContext::builder()
        .employee_id(employee_id)
        .tz(chrono_tz::UTC)
        .start_date(Utc::now().date_naive())
        .end_date(Utc::now().date_naive())
        .daily_entries_map(daily_entries_map)
        .emp_id_number("EMP001".to_string())
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

    let date = Utc::now().date_naive();
    // Time entry: 09:00 to 18:00 (matching 9-18 shift)
    let punch_in = date.and_hms_opt(9, 0, 0).unwrap().and_utc();
    let punch_out = date.and_hms_opt(18, 0, 0).unwrap().and_utc();

    let entry = TimeEntry::new(
        Uuid::new_v4(),
        employee_id,
        ws_id,
        None,
        None,
        punch_in,
        Some(punch_out),
        Some(540),
        false,
        crate::services::pairing::types::EntryType::Work,
        None,
        None,
        Utc::now(),
        Utc::now(),
    );

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
    let employment = crate::models::employment::Employment::new(
        employment_id,
        ws_id,
        employee_id,
        employer_id,
        None,
        None,
        "active".to_string(),
        date - chrono::Duration::days(10),
        None,
        None,
        "full_time".to_string(),
        None,
        Utc::now(),
        Utc::now(),
        None,
        None,
    );

    let mut payroll_roster_map = HashMap::new();
    payroll_roster_map.insert((employment_id, date), Some(shift_id));

    let attendance_rules = AttendanceRules {
        work_unit_minutes: 30,
        rounding_policy: crate::models::RoundingPolicy::Floor,
        early_arrival_strategy: EarlyArrivalStrategy::Block,
        grace_period_minutes: None,
    };

    let att_ctx = AttendanceContext::builder()
        .employee_id(employee_id)
        .tz(chrono_tz::UTC)
        .start_date(date)
        .end_date(date)
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

    let result = AttendanceMatcher::process_segment_attendance(args).await;
    // Total work duration = 9 hours (540 mins) minus 60 min break = 8 hours (480 mins) = 16 units of 30 mins.
    assert_eq!(result.std_units, 16);
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
        rounding_policy: crate::models::RoundingPolicy::Floor,
        early_arrival_strategy: EarlyArrivalStrategy::Block,
        grace_period_minutes: None,
    };

    let att_ctx = AttendanceContext::builder()
        .employee_id(employee_id)
        .tz(chrono_tz::UTC)
        .start_date(date)
        .end_date(date)
        .holidays_map(Arc::new(holidays_map))
        .emp_id_number("EMP001".to_string())
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

    let result = AttendanceMatcher::process_segment_attendance(args).await;
    // Auto-credit standard day (16 units) because it's a paid holiday with no attendance
    assert_eq!(result.std_units, 16);
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

    let entry = TimeEntry::new(
        Uuid::new_v4(),
        employee_id,
        ws_id,
        None,
        None,
        punch_in,
        Some(punch_out),
        Some(540),
        false,
        crate::services::pairing::types::EntryType::Work,
        None,
        None,
        Utc::now(),
        Utc::now(),
    );

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
    let employment = crate::models::employment::Employment::new(
        employment_id,
        ws_id,
        employee_id,
        employer_id,
        None,
        None,
        "active".to_string(),
        date - chrono::Duration::days(10),
        None,
        None,
        "full_time".to_string(),
        None,
        Utc::now(),
        Utc::now(),
        None,
        None,
    );

    let mut payroll_roster_map = HashMap::new();
    payroll_roster_map.insert((employment_id, date), Some(shift_id));

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
        rounding_policy: crate::models::RoundingPolicy::Floor,
        early_arrival_strategy: EarlyArrivalStrategy::Block,
        grace_period_minutes: None,
    };

    let att_ctx = AttendanceContext::builder()
        .employee_id(employee_id)
        .tz(chrono_tz::UTC)
        .start_date(date)
        .end_date(date)
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

    let date = NaiveDate::from_ymd_opt(2026, 1, 26).unwrap(); // Monday
                                                              // Time entry: 09:00 to 20:00 (matching 9-18 shift) -> 2 hours late OT
    let punch_in = date.and_hms_opt(9, 0, 0).unwrap().and_utc();
    let punch_out = date.and_hms_opt(20, 0, 0).unwrap().and_utc();

    let entry = TimeEntry::new(
        Uuid::new_v4(),
        employee_id,
        ws_id,
        None,
        None,
        punch_in,
        Some(punch_out),
        Some(660),
        false,
        crate::services::pairing::types::EntryType::Work,
        None,
        None,
        Utc::now(),
        Utc::now(),
    );

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
    let employment = crate::models::employment::Employment::new(
        employment_id,
        ws_id,
        employee_id,
        employer_id,
        None,
        None,
        "active".to_string(),
        date - chrono::Duration::days(10),
        None,
        None,
        "full_time".to_string(),
        None,
        Utc::now(),
        Utc::now(),
        None,
        None,
    );

    let mut payroll_roster_map = HashMap::new();
    payroll_roster_map.insert((employment_id, date), Some(shift_id));

    let attendance_rules = AttendanceRules {
        work_unit_minutes: 30,
        rounding_policy: crate::models::RoundingPolicy::Floor,
        early_arrival_strategy: EarlyArrivalStrategy::Block,
        grace_period_minutes: None,
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

    let overtime_data = crate::repositories::overtime_policy_repository::WorkspaceOvertimeData {
        policies,
        assignments,
    };

    let att_ctx = AttendanceContext::builder()
        .employee_id(employee_id)
        .tz(chrono_tz::UTC)
        .start_date(date)
        .end_date(date)
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
    assert_eq!(result.std_units, 16);
    assert_eq!(result.ot_units, 4); // 2 hours = 4 units
                                    // 2 hours OT * 10 hourly_rate * 2.0 multiplier = 40.0
    assert_eq!(result.ot_pay, Decimal::new(40, 0));
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

    let entry = TimeEntry::new(
        Uuid::new_v4(),
        employee_id,
        ws_id,
        None,
        None,
        punch_in,
        Some(punch_out),
        Some(540),
        false,
        crate::services::pairing::types::EntryType::Work,
        None,
        None,
        Utc::now(),
        Utc::now(),
    );

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
    let employment = crate::models::employment::Employment::new(
        employment_id,
        ws_id,
        employee_id,
        employer_id,
        None,
        None,
        "active".to_string(),
        date - chrono::Duration::days(10),
        None,
        None,
        "full_time".to_string(),
        None,
        Utc::now(),
        Utc::now(),
        None,
        None,
    );

    let mut payroll_roster_map = HashMap::new();
    payroll_roster_map.insert((employment_id, date), Some(shift_id));

    let attendance_rules = AttendanceRules {
        work_unit_minutes: 30,
        rounding_policy: crate::models::RoundingPolicy::Floor,
        early_arrival_strategy: EarlyArrivalStrategy::Block,
        grace_period_minutes: None,
    };

    let att_ctx = AttendanceContext::builder()
        .employee_id(employee_id)
        .tz(chrono_tz::UTC)
        .start_date(date)
        .end_date(date)
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

    let result = AttendanceMatcher::process_segment_attendance(args).await;
    assert_eq!(result.std_units, 16);
    assert_eq!(result.ot_units, 2);
    Ok(())
}
