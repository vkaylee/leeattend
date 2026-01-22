use apiserver::models::{EarlyArrivalStrategy, PolicyRules};
use chrono::{NaiveDate, NaiveTime, TimeZone, Utc};
use apiserver::services::AttendanceCalculator;

#[tokio::test]
async fn test_early_arrival_strategies_logic() {
    let shift_start = Some(Utc.from_local_datetime(
        &NaiveDate::from_ymd_opt(2026, 1, 1).unwrap().and_time(NaiveTime::from_hms_opt(8, 0, 0).unwrap())
    ).single().unwrap());
    
    let shift_end = Some(Utc.from_local_datetime(
        &NaiveDate::from_ymd_opt(2026, 1, 1).unwrap().and_time(NaiveTime::from_hms_opt(17, 0, 0).unwrap())
    ).single().unwrap());

    let punch_in = Utc.from_local_datetime(
        &NaiveDate::from_ymd_opt(2026, 1, 1).unwrap().and_time(NaiveTime::from_hms_opt(7, 0, 0).unwrap())
    ).single().unwrap();
    
    let punch_out = shift_end.clone().unwrap();

    let mut rules = PolicyRules {
        grace_period_minutes: 20,
        authorized_early_leave_minutes: 0,
        is_early_leave_paid: false,
        early_arrival_strategy: EarlyArrivalStrategy::Allow,
        is_break_auto_deduct: false,
        is_break_overtime: false,
        allow_early_ot: true,
    };

    // 1. Allow strategy: Should count full 60m OT
    let b_allow = AttendanceCalculator::calculate_buckets(
        punch_in, punch_out, shift_start, shift_end, 0, 0, &rules
    );
    assert_eq!(b_allow.early_ot_minutes, 60);

    // 2. Block strategy
    rules.early_arrival_strategy = EarlyArrivalStrategy::Block;
    let b_block = AttendanceCalculator::calculate_buckets(
        punch_in, punch_out, shift_start, shift_end, 0, 0, &rules
    );
    assert_eq!(b_block.early_ot_minutes, 60);

    // 3. Clamp strategy: Should ONLY count 20m OT (from 7:40 to 8:00)
    rules.early_arrival_strategy = EarlyArrivalStrategy::Clamp;
    let b_clamp = AttendanceCalculator::calculate_buckets(
        punch_in, punch_out, shift_start, shift_end, 0, 0, &rules
    );
    assert_eq!(b_clamp.early_ot_minutes, 20);

    // 4. Within Grace window (e.g. 7:45)
    let punch_in_grace = Utc.from_local_datetime(
        &NaiveDate::from_ymd_opt(2026, 1, 1).unwrap().and_time(NaiveTime::from_hms_opt(7, 45, 0).unwrap())
    ).single().unwrap();

    let b_grace = AttendanceCalculator::calculate_buckets(
        punch_in_grace, punch_out, shift_start, shift_end, 0, 0, &rules
    );
    assert_eq!(b_grace.early_ot_minutes, 15); 
}