use super::*;
use crate::models::DailyAttendanceReport;
use crate::services::attendance_utils::AttendanceStatus;
use chrono::{NaiveDate, NaiveTime, TimeZone};

fn make_report(has_punch_in: bool, has_shift: bool) -> DailyAttendanceReport {
    let date = NaiveDate::from_ymd_opt(2026, 3, 12).unwrap();
    let punch_in = if has_punch_in {
        Some(chrono::Utc.with_ymd_and_hms(2026, 3, 12, 8, 0, 0).unwrap())
    } else {
        None
    };
    let punch_out = if has_punch_in {
        Some(chrono::Utc.with_ymd_and_hms(2026, 3, 12, 17, 0, 0).unwrap())
    } else {
        None
    };
    let (scheduled_start, scheduled_end) = if has_shift {
        (
            Some(NaiveTime::from_hms_opt(8, 0, 0).unwrap()),
            Some(NaiveTime::from_hms_opt(17, 0, 0).unwrap()),
        )
    } else {
        (None, None)
    };

    let status = crate::services::attendance_utils::determine_daily_status(
        &crate::services::attendance_utils::DailyStatusInput::new(
            has_punch_in,
            punch_in.map(|p| p.time()),
            scheduled_start,
            false,
            false,
            has_shift,
            false,
            None,
        ),
    );

    DailyAttendanceReport {
        date,
        employment_id: Uuid::new_v4(),
        employee_id: Uuid::new_v4(),
        employee_name: "Test Employee".to_string(),
        employee_id_number: "EMP001".to_string(),
        shift_symbol: None,
        shift_name: None,
        scheduled_start,
        scheduled_end,
        grace_period_minutes: None,
        break_duration_minutes: None,
        break_start: None,
        break_end: None,
        is_break_auto_deduct: None,
        actual_punch_in: punch_in,
        actual_punch_out: punch_out,
        std_work_units: 0,
        ot_work_units: 0,
        ot_claim_status: None,
        unit_type: crate::models::enums::TimekeepingUnit::DAY,
        unit_coefficient_hours: 8.0,
        computed_std_hours: 0.0,
        computed_ot_hours: 0.0,
        status,
        is_holiday: false,
        is_unscheduled: status == AttendanceStatus::Unscheduled,
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
    }
}

#[test]
fn test_is_unscheduled_punch_in_no_shift() {
    let report = make_report(true, false);
    assert!(
        report.is_unscheduled,
        "punch_in + no shift → is_unscheduled must be true"
    );
    assert!(report.actual_punch_in.is_some());
    assert!(report.scheduled_start.is_none());
}

#[test]
fn test_is_not_unscheduled_punch_in_with_shift() {
    let report = make_report(true, true);
    assert!(
        !report.is_unscheduled,
        "punch_in + has shift → is_unscheduled must be false"
    );
}

#[test]
fn test_is_not_unscheduled_no_punch_no_shift() {
    let report = make_report(false, false);
    assert!(
        !report.is_unscheduled,
        "no punch + no shift → is_unscheduled must be false"
    );
}

#[test]
fn test_is_not_unscheduled_no_punch_with_shift() {
    let report = make_report(false, true);
    assert!(
        !report.is_unscheduled,
        "no punch + has shift → is_unscheduled must be false"
    );
}

#[test]
fn test_is_unscheduled_serialization_roundtrip_true() {
    let report = make_report(true, false);
    assert!(report.is_unscheduled);

    let json = serde_json::to_string(&report).unwrap();
    let decoded: DailyAttendanceReport = serde_json::from_str(&json).unwrap();

    assert!(
        decoded.is_unscheduled,
        "is_unscheduled=true should survive JSON roundtrip"
    );
}

#[test]
fn test_is_not_unscheduled_serialization_roundtrip_false() {
    let report = make_report(true, true);
    assert!(!report.is_unscheduled);

    let json = serde_json::to_string(&report).unwrap();
    let decoded: DailyAttendanceReport = serde_json::from_str(&json).unwrap();

    assert!(
        !decoded.is_unscheduled,
        "is_unscheduled=false should survive JSON roundtrip"
    );
}

#[test]
fn test_is_unscheduled_serde_default_when_missing() {
    let report = make_report(true, false);
    let mut json_value: serde_json::Value = serde_json::to_value(&report).unwrap();

    json_value.as_object_mut().unwrap().remove("is_unscheduled");
    assert!(json_value.get("is_unscheduled").is_none());

    let decoded: DailyAttendanceReport = serde_json::from_value(json_value).unwrap();

    assert!(
        !decoded.is_unscheduled,
        "missing is_unscheduled in JSON should default to false via #[serde(default)]"
    );
}

#[test]
fn test_shift_name_serialization_roundtrip() {
    let mut report = make_report(true, true);
    report.shift_name = Some("Morning Shift".to_string());

    let json = serde_json::to_string(&report).unwrap();
    let decoded: DailyAttendanceReport = serde_json::from_str(&json).unwrap();

    assert_eq!(
        decoded.shift_name,
        Some("Morning Shift".to_string()),
        "shift_name should survive JSON roundtrip"
    );
}

// ── No-show zone support: grace_period_minutes field ──

#[test]
fn test_grace_period_minutes_serialization_roundtrip() {
    let mut report = make_report(false, true);
    report.grace_period_minutes = Some(15);

    let json = serde_json::to_string(&report).unwrap();
    let decoded: DailyAttendanceReport = serde_json::from_str(&json).unwrap();

    assert_eq!(
        decoded.grace_period_minutes,
        Some(15),
        "grace_period_minutes should survive JSON roundtrip"
    );
}

#[test]
fn test_grace_period_minutes_serde_default_when_missing() {
    let report = make_report(false, true);
    assert_eq!(report.grace_period_minutes, None);

    let mut json_value: serde_json::Value = serde_json::to_value(&report).unwrap();
    json_value
        .as_object_mut()
        .unwrap()
        .remove("grace_period_minutes");
    assert!(json_value.get("grace_period_minutes").is_none());

    let decoded: DailyAttendanceReport = serde_json::from_value(json_value).unwrap();
    assert_eq!(
        decoded.grace_period_minutes, None,
        "missing grace_period_minutes should default to None via #[serde(default)]"
    );
}

// ── No-show zone support: shift_type filter (pure helper) ──

fn make_report_with_shift(
    has_punch_in: bool,
    has_shift: bool,
    shift_name: Option<&str>,
) -> DailyAttendanceReport {
    let mut report = make_report(has_punch_in, has_shift);
    report.shift_name = shift_name.map(String::from);
    report
}

#[test]
fn test_shift_type_filter_includes_matching() {
    let rows = vec![
        make_report_with_shift(false, true, Some("Morning Shift")),
        make_report_with_shift(false, true, Some("Night Shift")),
    ];
    let filtered =
        super::daily::filter_by_shift_type(rows, Some(vec!["Morning Shift".to_string()]));
    assert_eq!(filtered.len(), 1);
    assert_eq!(filtered[0].shift_name, Some("Morning Shift".to_string()));
}

#[test]
fn test_shift_type_filter_excludes_non_matching() {
    let rows = vec![
        make_report_with_shift(false, true, Some("Morning Shift")),
        make_report_with_shift(false, true, Some("Night Shift")),
    ];
    let filtered =
        super::daily::filter_by_shift_type(rows, Some(vec!["Afternoon Shift".to_string()]));
    assert!(
        filtered.is_empty(),
        "no rows match the filter → empty result"
    );
}

#[test]
fn test_shift_type_filter_none_returns_all() {
    let rows = vec![
        make_report_with_shift(false, true, Some("Morning Shift")),
        make_report_with_shift(false, true, Some("Night Shift")),
        make_report_with_shift(true, true, None),
    ];
    let filtered = super::daily::filter_by_shift_type(rows, None);
    assert_eq!(filtered.len(), 3, "no filter → all rows returned");
}

#[test]
fn test_shift_type_filter_comma_separated_multiple() {
    let rows = vec![
        make_report_with_shift(false, true, Some("Morning Shift")),
        make_report_with_shift(false, true, Some("Night Shift")),
        make_report_with_shift(false, true, Some("Afternoon Shift")),
    ];
    let filtered = super::daily::filter_by_shift_type(
        rows,
        Some(vec!["Morning Shift".to_string(), "Night Shift".to_string()]),
    );
    assert_eq!(filtered.len(), 2);
    let names: Vec<_> = filtered
        .iter()
        .filter_map(|r| r.shift_name.clone())
        .collect();
    assert!(names.contains(&"Morning Shift".to_string()));
    assert!(names.contains(&"Night Shift".to_string()));
}

#[test]
fn test_shift_type_filter_handles_null_shift_name() {
    let rows = vec![make_report_with_shift(true, true, None)];
    let filtered =
        super::daily::filter_by_shift_type(rows, Some(vec!["Morning Shift".to_string()]));
    assert!(
        filtered.is_empty(),
        "rows with null shift_name are excluded by a shift filter"
    );
}
