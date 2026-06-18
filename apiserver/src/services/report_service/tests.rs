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
