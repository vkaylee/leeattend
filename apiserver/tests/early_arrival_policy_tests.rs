use apiserver::models::EarlyArrivalStrategy;
use chrono::{NaiveDate, NaiveTime, TimeZone, Utc};
use apiserver::services::AttendanceCalculator;

#[tokio::test]
async fn test_early_arrival_strategies_logic() {
    // We don't necessarily need a full DB for testing the Calculator logic,
    // as it's a pure function. Let's test the calculator first.

    let shift_start = Some(Utc.from_local_datetime(
        &NaiveDate::from_ymd_opt(2026, 1, 1).unwrap().and_time(NaiveTime::from_hms_opt(8, 0, 0).unwrap())
    ).single().unwrap());
    
    let shift_end = Some(Utc.from_local_datetime(
        &NaiveDate::from_ymd_opt(2026, 1, 1).unwrap().and_time(NaiveTime::from_hms_opt(17, 0, 0).unwrap())
    ).single().unwrap());

    // Punch in 7:00 (60m early), Grace 20m (threshold 7:40)
    let punch_in = Utc.from_local_datetime(
        &NaiveDate::from_ymd_opt(2026, 1, 1).unwrap().and_time(NaiveTime::from_hms_opt(7, 0, 0).unwrap())
    ).single().unwrap();
    
    let punch_out = shift_end.clone().unwrap();

    // 1. Allow strategy: Should count full 60m OT
    let b_allow = AttendanceCalculator::calculate_buckets(
        punch_in, punch_out, shift_start, shift_end, 0, 20, 0, false, false, true, 0, false, EarlyArrivalStrategy::Allow
    );
    assert_eq!(b_allow.early_ot_minutes, 60);

    // 2. Block strategy: (In calculator it behaves like Allow, because Block is enforced at the API layer)
    // Actually, in the calculator, Block and Allow currently share the same logic: "If outside grace, count FULL OT".
    let b_block = AttendanceCalculator::calculate_buckets(
        punch_in, punch_out, shift_start, shift_end, 0, 20, 0, false, false, true, 0, false, EarlyArrivalStrategy::Block
    );
    assert_eq!(b_block.early_ot_minutes, 60);

    // 3. Clamp strategy: Should ONLY count 20m OT (from 7:40 to 8:00)
    let b_clamp = AttendanceCalculator::calculate_buckets(
        punch_in, punch_out, shift_start, shift_end, 0, 20, 0, false, false, true, 0, false, EarlyArrivalStrategy::Clamp
    );
    assert_eq!(b_clamp.early_ot_minutes, 20);

    // 4. Within Grace window (e.g. 7:45)
    // In Clamp mode, it should count the 15m (8:00 - 7:45).
    let punch_in_grace = Utc.from_local_datetime(
        &NaiveDate::from_ymd_opt(2026, 1, 1).unwrap().and_time(NaiveTime::from_hms_opt(7, 45, 0).unwrap())
    ).single().unwrap();

    let b_grace = AttendanceCalculator::calculate_buckets(
        punch_in_grace, punch_out, shift_start, shift_end, 0, 20, 0, false, false, true, 0, false, EarlyArrivalStrategy::Clamp
    );
    assert_eq!(b_grace.early_ot_minutes, 15); 
}
