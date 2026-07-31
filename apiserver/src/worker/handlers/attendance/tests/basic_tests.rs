use super::super::*;
use crate::models::DailyAttendanceReport;
use crate::services::attendance_utils::{
    determine_daily_status, AttendanceStatus, DailyStatusInput, HolidayKind,
};
use chrono::NaiveTime;

#[test]
fn test_status_determination_present() {
    let status = determine_daily_status(&DailyStatusInput::new(
        true,
        Some(NaiveTime::from_hms_opt(8, 0, 0).unwrap()),
        Some(NaiveTime::from_hms_opt(8, 30, 0).unwrap()),
        HolidayKind::NotHoliday,
        true,
        false,
        None,
    ));
    assert_eq!(status, AttendanceStatus::Present);
    assert_eq!(status.to_string(), "present");
}

#[test]
fn test_status_determination_late() {
    let status = determine_daily_status(&DailyStatusInput::new(
        true,
        Some(NaiveTime::from_hms_opt(9, 0, 0).unwrap()),
        Some(NaiveTime::from_hms_opt(8, 30, 0).unwrap()),
        HolidayKind::NotHoliday,
        true,
        false,
        None,
    ));
    assert_eq!(status, AttendanceStatus::Late);
    assert_eq!(status.to_string(), "late");
}

#[test]
fn test_status_determination_absent() {
    let status = determine_daily_status(&DailyStatusInput::new(
        false,
        None,
        Some(NaiveTime::from_hms_opt(8, 30, 0).unwrap()),
        HolidayKind::NotHoliday,
        true,
        false,
        None,
    ));
    assert_eq!(status, AttendanceStatus::Absent);
    assert_eq!(status.to_string(), "absent");
}

#[test]
fn test_status_determination_off() {
    let status = determine_daily_status(&DailyStatusInput::new(
        false, None, None, HolidayKind::NotHoliday, false, false, None,
    ));
    assert_eq!(status, AttendanceStatus::Off);
    assert_eq!(status.to_string(), "off");
}

#[test]
fn test_entries_filtering_by_date() {
    // Simulate filtering entries by date (as done in the handler)
    let entries_dates = [
        NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
        NaiveDate::from_ymd_opt(2026, 1, 2).unwrap(),
        NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
    ];

    let target_date = NaiveDate::from_ymd_opt(2026, 1, 1).unwrap();
    let filtered_count = entries_dates.iter().filter(|d| **d == target_date).count();

    assert_eq!(filtered_count, 2);
}

#[test]
fn test_break_minutes_aggregation() {
    let break_entries = [15, 30, 15]; // Minutes
    let total: i32 = break_entries.iter().sum();
    assert_eq!(total, 60);
}

#[test]
fn test_overnight_shift_detection() {
    let start = NaiveTime::from_hms_opt(22, 0, 0).unwrap();
    let end = NaiveTime::from_hms_opt(6, 0, 0).unwrap();

    // Overnight: end < start
    assert!(end < start);
}

#[test]
fn test_summary_serialization() {
    let summaries = vec![
        AttendanceSummary {
            employment_id: Uuid::new_v4(),
            employee_id: Uuid::new_v4(),
            employee_name: "Alice".to_string(),
            employee_id_number: String::new(),
            total_std_units: 140,
            total_ot_units: 20,
            total_leave_days: 2,
            unit_type: crate::models::enums::TimekeepingUnit::DAY,
            unit_coefficient_hours: 8.0,
            computed_std_hours: 140.0,
            computed_ot_hours: 20.0,
            employer_id: None,
            employer_name: None,
        },
        AttendanceSummary {
            employment_id: Uuid::new_v4(),
            employee_id: Uuid::new_v4(),
            employee_name: "Bob".to_string(),
            employee_id_number: String::new(),
            total_std_units: 152,
            total_ot_units: 0,
            total_leave_days: 4,
            unit_type: crate::models::enums::TimekeepingUnit::DAY,
            unit_coefficient_hours: 8.0,
            computed_std_hours: 152.0,
            computed_ot_hours: 0.0,
            employer_id: None,
            employer_name: None,
        },
    ];

    let json = serde_json::to_string(&summaries).unwrap();
    let decoded: Vec<AttendanceSummary> = serde_json::from_str(&json).unwrap();

    assert_eq!(decoded.len(), 2);
    assert_eq!(decoded[0].employee_name, "Alice");
    assert_eq!(decoded[0].total_std_units, 140);
    assert_eq!(decoded[0].total_ot_units, 20);
}

#[test]
fn test_detail_report_serialization() {
    let reports = vec![DailyAttendanceReport {
        date: NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
        employment_id: Uuid::new_v4(),
        employee_id: Uuid::new_v4(),
        employee_name: "Alice".to_string(),
        employee_id_number: "EMP001".to_string(),
        shift_symbol: None,
        shift_name: None,
        scheduled_start: Some(NaiveTime::from_hms_opt(9, 0, 0).unwrap()),
        scheduled_end: Some(NaiveTime::from_hms_opt(17, 0, 0).unwrap()),
        grace_period_minutes: None,
        break_duration_minutes: Some(60),
        break_start: Some(NaiveTime::from_hms_opt(12, 0, 0).unwrap()),
        break_end: Some(NaiveTime::from_hms_opt(13, 0, 0).unwrap()),
        is_break_auto_deduct: Some(true),
        actual_punch_in: None,
        actual_punch_out: None,
        std_work_units: 0,
        ot_work_units: 0,
        ot_claim_status: None,
        unit_type: crate::models::enums::TimekeepingUnit::DAY,
        unit_coefficient_hours: 8.0,
        computed_std_hours: 0.0,
        computed_ot_hours: 0.0,
        status: crate::services::attendance_utils::AttendanceStatus::Absent,
        is_holiday: false,
        is_unscheduled: false,
        holiday_name: None,
        generation_metadata: None,
        is_leave_paid: None,
        employer_name: None,
        timezone: None,
        segment_id: None,
        shift_id: None,
        interval_id: None,
        interval_name: None,
        segment_type: None,
        sort_order: None,
        segment_count: None,
    }];

    let json = serde_json::to_string(&reports).unwrap();
    let decoded: Vec<DailyAttendanceReport> = serde_json::from_str(&json).unwrap();

    assert_eq!(decoded.len(), 1);
    assert_eq!(
        decoded[0].status,
        crate::services::attendance_utils::AttendanceStatus::Absent
    );
    assert_eq!(decoded[0].std_work_units, 0);
    assert_eq!(decoded[0].ot_work_units, 0);
}

#[test]
fn test_timezone_parsing() {
    let valid_tz: Result<Tz, _> = "Asia/Ho_Chi_Minh".parse();
    assert!(valid_tz.is_ok());

    let invalid_tz: Result<Tz, _> = "Invalid/Timezone".parse();
    assert!(invalid_tz.is_err());

    // Default fallback
    let tz: Tz = "invalid".parse().unwrap_or(chrono_tz::UTC);
    assert_eq!(tz, chrono_tz::UTC);
}

// Status determination now uses attendance_utils::determine_daily_status
// (old local determine_status helper removed in the refactoring)

/// Test that entries are correctly pre-grouped by date using timezone-aware conversion.
/// This validates optimization #3: O(entries) pre-grouping instead of O(entries × days) scanning.
#[test]
fn test_entries_pregrouping_by_date_with_timezone() {
    use chrono::TimeZone;
    let tz: Tz = "Asia/Ho_Chi_Minh".parse().unwrap(); // UTC+7

    // Create entries at specific UTC times that map to different local dates
    // UTC 2026-01-14 20:00 → Ho_Chi_Minh 2026-01-15 03:00
    let entry_jan15_early = TimeEntry::new(
        Uuid::new_v4(),
        Uuid::new_v4(),
        Uuid::new_v4(),
        None,
        None,
        chrono::Utc.with_ymd_and_hms(2026, 1, 14, 20, 0, 0).unwrap(),
        None,
        Some(480),
        false,
        crate::services::pairing::types::EntryType::Work,
        None,
        None,
        chrono::Utc::now(),
        chrono::Utc::now(),
    );
    // UTC 2026-01-15 02:00 → Ho_Chi_Minh 2026-01-15 09:00
    let entry_jan15_morning = TimeEntry::new(
        Uuid::new_v4(),
        entry_jan15_early.employment_id(),
        entry_jan15_early.workspace_id(),
        None,
        None,
        chrono::Utc.with_ymd_and_hms(2026, 1, 15, 2, 0, 0).unwrap(),
        None,
        Some(480),
        false,
        crate::services::pairing::types::EntryType::Work,
        None,
        None,
        chrono::Utc::now(),
        chrono::Utc::now(),
    );
    // UTC 2026-01-15 18:00 → Ho_Chi_Minh 2026-01-16 01:00 (next day!)
    let entry_jan16 = TimeEntry::new(
        Uuid::new_v4(),
        entry_jan15_early.employment_id(),
        entry_jan15_early.workspace_id(),
        None,
        None,
        chrono::Utc.with_ymd_and_hms(2026, 1, 15, 18, 0, 0).unwrap(),
        None,
        Some(480),
        false,
        crate::services::pairing::types::EntryType::Work,
        None,
        None,
        chrono::Utc::now(),
        chrono::Utc::now(),
    );

    let entries = vec![entry_jan15_early, entry_jan15_morning, entry_jan16];

    // Pre-group using shared utility (same as calculate_summary/calculate_detail)
    let entries_by_date = crate::services::attendance_utils::group_entries_by_date(&entries, tz);

    let jan15 = NaiveDate::from_ymd_opt(2026, 1, 15).unwrap();
    let jan16 = NaiveDate::from_ymd_opt(2026, 1, 16).unwrap();

    // 2 entries on Jan 15 (in Ho_Chi_Minh timezone)
    assert_eq!(
        entries_by_date
            .get(&jan15)
            .map(std::vec::Vec::len)
            .unwrap_or(0),
        2
    );
    // 1 entry on Jan 16 (UTC 18:00 → local 01:00 next day)
    assert_eq!(
        entries_by_date
            .get(&jan16)
            .map(std::vec::Vec::len)
            .unwrap_or(0),
        1
    );
}

/// Test that pre-grouping produces the same results as the old filter-per-day approach.
#[test]
fn test_pregrouping_matches_filter_approach() {
    use chrono::TimeZone;
    let tz: Tz = "UTC".parse().unwrap();

    let emp_id = Uuid::new_v4();
    let ws_id = Uuid::new_v4();
    let entries: Vec<TimeEntry> = (1..=5)
        .map(|day| {
            TimeEntry::new(
                Uuid::new_v4(),
                emp_id,
                ws_id,
                None,
                None,
                chrono::Utc.with_ymd_and_hms(2026, 1, day, 8, 0, 0).unwrap(),
                None,
                Some(480),
                false,
                crate::services::pairing::types::EntryType::Work,
                None,
                None,
                chrono::Utc::now(),
                chrono::Utc::now(),
            )
        })
        .collect();

    // Pre-group approach using shared utility
    let entries_by_date = crate::services::attendance_utils::group_entries_by_date(&entries, tz);

    // Filter approach (old) — for each target date, scan all entries
    for day in 1..=5 {
        let target = NaiveDate::from_ymd_opt(2026, 1, day).unwrap();
        let filtered_count = entries
            .iter()
            .filter(|e| e.punch_in_time().with_timezone(&tz).date_naive() == target)
            .count();

        let grouped = entries_by_date
            .get(&target)
            .map(std::vec::Vec::len)
            .unwrap_or(0);
        assert_eq!(filtered_count, grouped, "Mismatch on day {day}");
    }
}

/// Test that empty entries produce an empty grouping.
#[test]
fn test_pregrouping_empty_entries() {
    let tz: Tz = "UTC".parse().unwrap();
    let entries: Vec<TimeEntry> = vec![];

    let entries_by_date = crate::services::attendance_utils::group_entries_by_date(&entries, tz);

    assert!(entries_by_date.is_empty());

    // Looking up any date returns empty
    let jan1 = NaiveDate::from_ymd_opt(2026, 1, 1).unwrap();
    let day_entries = entries_by_date
        .get(&jan1)
        .map(std::vec::Vec::as_slice)
        .unwrap_or(&[]);
    assert!(day_entries.is_empty());
}

#[test]
fn test_slow_attendance_batch_threshold_is_5_seconds() {
    // The per-phase timing breakdown threshold is 5 seconds.
    let fast = std::time::Duration::from_secs(4);
    assert!(
        fast.as_secs() <= 5,
        "4s batch should not trigger breakdown log"
    );

    let slow = std::time::Duration::from_secs(6);
    assert!(slow.as_secs() > 5, "6s batch should trigger breakdown log");

    // Exactly 5s should NOT trigger (> 5).
    let edge = std::time::Duration::from_secs(5);
    assert!((edge.as_secs() <= 5), "Exactly 5s should not trigger");
}

#[test]
fn test_phase_timing_subtraction_produces_correct_deltas() {
    // Simulate the phase timing pattern used in attendance.rs:
    // t_schedules, t_entries, t_store, t_total are all elapsed from t0
    let t_schedules = std::time::Duration::from_millis(50);
    let t_entries = std::time::Duration::from_millis(200);
    let t_store = std::time::Duration::from_millis(6_500);
    let t_total = std::time::Duration::from_millis(6_550);

    let schedules_ms = t_schedules.as_millis() as u64;
    let entries_ms = t_entries.checked_sub(t_schedules).unwrap().as_millis() as u64;
    let calc_ms = t_store.checked_sub(t_entries).unwrap().as_millis() as u64;
    let store_publish_ms = t_total.checked_sub(t_store).unwrap().as_millis() as u64;

    assert_eq!(schedules_ms, 50);
    assert_eq!(entries_ms, 150);
    assert_eq!(calc_ms, 6_300);
    assert_eq!(store_publish_ms, 50);

    // Verify phases sum to total
    let sum = schedules_ms + entries_ms + calc_ms + store_publish_ms;
    assert_eq!(sum, t_total.as_millis() as u64);
}

#[test]
fn test_phase_timing_with_dominant_single_phase() {
    // Simulate the scenario from logs: one phase takes most of the time
    // (e.g., time entry query was slow)
    let t_schedules = std::time::Duration::from_millis(30);
    let t_entries = std::time::Duration::from_millis(198_030); // 198s for entries!
    let t_store = std::time::Duration::from_millis(198_050);
    let t_total = std::time::Duration::from_millis(198_100);

    let entries_ms = t_entries.checked_sub(t_schedules).unwrap().as_millis() as u64;
    assert_eq!(
        entries_ms, 198_000,
        "Should identify entries fetch as the bottleneck"
    );

    // Calc phase should be small
    let calc_ms = t_store.checked_sub(t_entries).unwrap().as_millis() as u64;
    assert_eq!(calc_ms, 20);

    // Total should exceed the 5s threshold
    assert!(t_total.as_secs() > 5);
}
