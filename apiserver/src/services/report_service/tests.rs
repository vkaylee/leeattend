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
            crate::services::attendance_utils::HolidayKind::NotHoliday,
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
        break_split: None,
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

// ── Segment report: break-split scenario (issue #092523baa) ──

#[cfg(test)]
mod segment_break_split_tests {
    use crate::models::{AttendanceRules, NewTimeEntryParams, ResolvedSegment, TimeEntry};
    use crate::services::pairing::types::EntryType;
    use crate::services::report_service::daily::build_segment_calc_inputs;
    use chrono::{NaiveDate, NaiveTime, TimeZone, Utc};
    use chrono_tz::Asia::Ho_Chi_Minh;
    use std::collections::HashMap;
    use uuid::Uuid;

    fn make_time_entry(
        punch_in_utc: chrono::DateTime<Utc>,
        punch_out_utc: chrono::DateTime<Utc>,
    ) -> TimeEntry {
        TimeEntry::new(NewTimeEntryParams {
            id: Uuid::new_v4(),
            employment_id: Uuid::new_v4(),
            workspace_id: Uuid::new_v4(),
            shift_id: None,
            segment_id: None,
            punch_in_time: punch_in_utc,
            punch_out_time: Some(punch_out_utc),
            duration_minutes: None,
            is_break_overtime: false,
            entry_type: EntryType::Work,
            note: None,
            generation_metadata: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        })
    }

    #[test]
    fn test_build_segment_calc_inputs_with_break_split() {
        // Shift: 07:30-16:30 HCM with break 11:30-12:30
        // Two sessions: morning (07:30-11:30) and afternoon (12:30-16:30)
        let date = NaiveDate::from_ymd_opt(2026, 7, 1).unwrap();
        let tz = Ho_Chi_Minh;

        // Morning entry: 07:30-11:30 HCM = 00:30-04:30 UTC
        let morning = make_time_entry(
            Utc.with_ymd_and_hms(2026, 7, 1, 0, 30, 0).unwrap(),
            Utc.with_ymd_and_hms(2026, 7, 1, 4, 30, 0).unwrap(),
        );

        // Afternoon entry: 12:30-16:30 HCM = 05:30-09:30 UTC
        let afternoon = make_time_entry(
            Utc.with_ymd_and_hms(2026, 7, 1, 5, 30, 0).unwrap(),
            Utc.with_ymd_and_hms(2026, 7, 1, 9, 30, 0).unwrap(),
        );

        let entries = vec![morning, afternoon];
        let entry_refs: Vec<&TimeEntry> = entries.iter().collect();

        // Single segment covering full shift
        let segment = ResolvedSegment {
            template_id: Uuid::new_v4(),
            interval_id: Some(Uuid::new_v4()),
            sort_order: 0,
            scheduled_start: NaiveTime::from_hms_opt(7, 30, 0).unwrap(),
            scheduled_end: NaiveTime::from_hms_opt(16, 30, 0).unwrap(),
            source: "roster".to_string(),
            date: Some(date),
            shift_name: None,
            shift_symbol: None,
            interval_name: None,
            segment_type: "work".to_string(),
            break_duration_minutes: Some(60),
            break_start: Some(NaiveTime::from_hms_opt(11, 30, 0).unwrap()),
            break_end: Some(NaiveTime::from_hms_opt(12, 30, 0).unwrap()),
            is_break_auto_deduct: Some(true),
            break_is_overtime: Some(false),
            allow_early_ot: false,
            max_early_ot_minutes: None,
            early_punch_in_grace_period_minutes: 15,
            ot_aggregation_mode: Default::default(),
            is_night_shift: false,
            traces: vec![],
            is_holiday: false,
            is_holiday_paid: false,
            holiday_name: None,
        };

        let segments = vec![&segment];
        let working_conditions = vec![];
        let att_rules = AttendanceRules {
            work_unit_minutes: 60, // Match daily path
            rounding_policy: Default::default(),
            early_arrival_strategy: Default::default(),
            grace_period_minutes: None,
        };
        let condition_type_catalog = HashMap::new();

        let (matched, _rules, calc_inputs) = build_segment_calc_inputs(
            &segments,
            &entry_refs,
            date,
            tz,
            &working_conditions,
            &att_rules,
            &condition_type_catalog,
        );

        assert_eq!(matched.len(), 1, "One segment");
        assert_eq!(matched[0].len(), 2, "Two entries matched");
        assert_eq!(calc_inputs.len(), 1, "One segment's calc inputs");
        assert_eq!(
            calc_inputs[0].len(),
            2,
            "Two DailyCalcInput (one per session entry)"
        );

        // Verify scheduled_break_minutes is 0 for multi-session
        for input in &calc_inputs[0] {
            assert_eq!(
                input.scheduled_break_minutes, 0,
                "Multi-session: scheduled_break_minutes must be 0 (gap already excluded)"
            );
        }
    }

    #[test]
    fn test_segment_vs_daily_path_consistency() {
        // Both paths should produce the same std_work_units for the same input
        use crate::services::AttendanceService;

        let date = NaiveDate::from_ymd_opt(2026, 7, 1).unwrap();
        let tz = Ho_Chi_Minh;

        // Morning entry: 07:30-11:30 HCM = 00:30-04:30 UTC
        let morning = make_time_entry(
            Utc.with_ymd_and_hms(2026, 7, 1, 0, 30, 0).unwrap(),
            Utc.with_ymd_and_hms(2026, 7, 1, 4, 30, 0).unwrap(),
        );

        // Afternoon entry: 12:30-16:30 HCM = 05:30-09:30 UTC
        let afternoon = make_time_entry(
            Utc.with_ymd_and_hms(2026, 7, 1, 5, 30, 0).unwrap(),
            Utc.with_ymd_and_hms(2026, 7, 1, 9, 30, 0).unwrap(),
        );

        let entries = vec![morning, afternoon];
        let entry_refs: Vec<&TimeEntry> = entries.iter().collect();

        // Daily path: compute_session_attendance with all entries
        let shift_start = Some(
            tz.with_ymd_and_hms(2026, 7, 1, 7, 30, 0)
                .unwrap()
                .with_timezone(&tz),
        );
        let shift_end = Some(
            tz.with_ymd_and_hms(2026, 7, 1, 16, 30, 0)
                .unwrap()
                .with_timezone(&tz),
        );
        let break_start = Some(
            tz.with_ymd_and_hms(2026, 7, 1, 11, 30, 0)
                .unwrap()
                .with_timezone(&tz),
        );
        let break_end = Some(
            tz.with_ymd_and_hms(2026, 7, 1, 12, 30, 0)
                .unwrap()
                .with_timezone(&tz),
        );

        let daily_result = AttendanceService::compute_session_attendance(
            &entry_refs,
            tz,
            shift_start,
            shift_end,
            break_start,
            break_end,
            60, // scheduled_break_minutes
            60, // work_unit_minutes (1 unit = 1 hour)
            Default::default(),
            &crate::models::PolicyRules::new(
                15,    // grace_period_minutes
                0,     // authorized_early_leave_minutes
                false, // is_early_leave_paid
                crate::models::EarlyArrivalStrategy::Allow,
                true,  // is_break_auto_deduct
                false, // is_break_overtime
                false, // allow_early_ot
            ),
            None,
            None,
            None,
            Default::default(),
        );

        // Segment path: build_segment_calc_inputs + compute_segment_attendance_sessions
        let segment = ResolvedSegment {
            template_id: Uuid::new_v4(),
            interval_id: Some(Uuid::new_v4()),
            sort_order: 0,
            scheduled_start: NaiveTime::from_hms_opt(7, 30, 0).unwrap(),
            scheduled_end: NaiveTime::from_hms_opt(16, 30, 0).unwrap(),
            source: "roster".to_string(),
            date: Some(date),
            shift_name: None,
            shift_symbol: None,
            interval_name: None,
            segment_type: "work".to_string(),
            break_duration_minutes: Some(60),
            break_start: Some(NaiveTime::from_hms_opt(11, 30, 0).unwrap()),
            break_end: Some(NaiveTime::from_hms_opt(12, 30, 0).unwrap()),
            is_break_auto_deduct: Some(true),
            break_is_overtime: Some(false),
            allow_early_ot: false,
            max_early_ot_minutes: None,
            early_punch_in_grace_period_minutes: 15,
            ot_aggregation_mode: Default::default(),
            is_night_shift: false,
            traces: vec![],
            is_holiday: false,
            is_holiday_paid: false,
            holiday_name: None,
        };

        let segments = vec![&segment];
        let working_conditions = vec![];
        let att_rules = AttendanceRules {
            work_unit_minutes: 60, // Match daily path
            rounding_policy: Default::default(),
            early_arrival_strategy: Default::default(),
            grace_period_minutes: None,
        };
        let condition_type_catalog = HashMap::new();

        let (_matched, _rules, calc_inputs) = build_segment_calc_inputs(
            &segments,
            &entry_refs,
            date,
            tz,
            &working_conditions,
            &att_rules,
            &condition_type_catalog,
        );

        let input_groups: Vec<&[crate::services::attendance_calculator::DailyCalcInput<_>]> =
            calc_inputs.iter().map(|v| v.as_slice()).collect();

        let segment_result = AttendanceService::compute_segment_attendance_sessions(&input_groups);

        // Both paths should produce the same standard units
        assert_eq!(
            daily_result.total_standard_units(),
            segment_result.total_standard_units(),
            "Daily and segment paths must produce the same std_work_units"
        );

        // Expected: 8 hours = 480 minutes = 8 units (with work_unit_minutes=60)
        assert_eq!(
            segment_result.total_standard_units(),
            8,
            "Break-split scenario: 4h morning + 4h afternoon = 8h total"
        );
    }

    /// Create a TimeEntry with `break_split` generation metadata so the
    /// report layer can detect and decompose break-split sessions.
    fn make_break_split_entry(
        punch_in_utc: chrono::DateTime<Utc>,
        punch_out_utc: chrono::DateTime<Utc>,
        split_half: &str,
    ) -> TimeEntry {
        TimeEntry::new(NewTimeEntryParams {
            id: Uuid::new_v4(),
            employment_id: Uuid::new_v4(),
            workspace_id: Uuid::new_v4(),
            shift_id: None,
            segment_id: None,
            punch_in_time: punch_in_utc,
            punch_out_time: Some(punch_out_utc),
            duration_minutes: None,
            is_break_overtime: false,
            entry_type: EntryType::Work,
            note: None,
            generation_metadata: Some(serde_json::json!({ "break_split": split_half })),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        })
    }

    fn make_vp_segment() -> ResolvedSegment {
        ResolvedSegment {
            template_id: Uuid::new_v4(),
            interval_id: Some(Uuid::new_v4()),
            sort_order: 0,
            scheduled_start: NaiveTime::from_hms_opt(7, 30, 0).unwrap(),
            scheduled_end: NaiveTime::from_hms_opt(16, 30, 0).unwrap(),
            source: "roster".to_string(),
            date: Some(NaiveDate::from_ymd_opt(2026, 7, 1).unwrap()),
            shift_name: Some("VP".to_string()),
            shift_symbol: Some("VP".to_string()),
            interval_name: Some("Ca VP".to_string()),
            segment_type: "work".to_string(),
            break_duration_minutes: Some(60),
            break_start: Some(NaiveTime::from_hms_opt(11, 30, 0).unwrap()),
            break_end: Some(NaiveTime::from_hms_opt(12, 30, 0).unwrap()),
            is_break_auto_deduct: Some(true),
            break_is_overtime: Some(false),
            allow_early_ot: false,
            max_early_ot_minutes: None,
            early_punch_in_grace_period_minutes: 15,
            ot_aggregation_mode: Default::default(),
            is_night_shift: false,
            traces: vec![],
            is_holiday: false,
            is_holiday_paid: false,
            holiday_name: None,
        }
    }

    #[test]
    fn test_break_split_sessions_json_two_halves() {
        use crate::services::report_service::daily::build_break_split_sessions_json;

        let date = NaiveDate::from_ymd_opt(2026, 7, 1).unwrap();
        let tz = Ho_Chi_Minh;
        let segment = make_vp_segment();

        // Morning: 07:30–11:30 HCM = 00:30–04:30 UTC
        let morning = make_break_split_entry(
            Utc.with_ymd_and_hms(2026, 7, 1, 0, 30, 0).unwrap(),
            Utc.with_ymd_and_hms(2026, 7, 1, 4, 30, 0).unwrap(),
            "first_half",
        );
        // Afternoon: 12:30–16:30 HCM = 05:30–09:30 UTC
        let afternoon = make_break_split_entry(
            Utc.with_ymd_and_hms(2026, 7, 1, 5, 30, 0).unwrap(),
            Utc.with_ymd_and_hms(2026, 7, 1, 9, 30, 0).unwrap(),
            "second_half",
        );

        let entries = vec![morning, afternoon];
        let entry_refs: Vec<&TimeEntry> = entries.iter().collect();

        // Build calc inputs matching the entries
        let att_rules = AttendanceRules {
            work_unit_minutes: 60,
            rounding_policy: Default::default(),
            early_arrival_strategy: Default::default(),
            grace_period_minutes: None,
        };

        let (_matched, _rules, segment_calc_inputs) = build_segment_calc_inputs(
            &[&segment],
            &entry_refs,
            date,
            tz,
            &[],
            &att_rules,
            &HashMap::new(),
        );

        let sessions = build_break_split_sessions_json(
            &segment,
            &entry_refs,
            &segment_calc_inputs[0],
            &att_rules,
            tz,
            None,  // no holiday
            false, // not on leave
            15,    // grace period
        )
        .expect("break-split entries must produce sessions");

        let arr = sessions.as_array().expect("sessions must be array");
        assert_eq!(arr.len(), 2);

        // First half
        assert_eq!(arr[0]["break_split"], "first_half");
        assert_eq!(arr[0]["status"], "present");
        assert_eq!(arr[0]["std_work_units"], 4);
        assert_eq!(arr[0]["computed_std_hours"], 4.0);
        assert_eq!(arr[0]["scheduled_start"], "07:30:00");
        assert_eq!(arr[0]["scheduled_end"], "11:30:00");

        // Second half
        assert_eq!(arr[1]["break_split"], "second_half");
        assert_eq!(arr[1]["status"], "present");
        assert_eq!(arr[1]["std_work_units"], 4);
        assert_eq!(arr[1]["computed_std_hours"], 4.0);
        assert_eq!(arr[1]["scheduled_start"], "12:30:00");
        assert_eq!(arr[1]["scheduled_end"], "16:30:00");
    }

    #[test]
    fn test_break_split_sessions_status_late_second_half() {
        use crate::services::report_service::daily::build_break_split_sessions_json;

        let date = NaiveDate::from_ymd_opt(2026, 7, 1).unwrap();
        let tz = Ho_Chi_Minh;
        let segment = make_vp_segment();

        // Morning: on time
        let morning = make_break_split_entry(
            Utc.with_ymd_and_hms(2026, 7, 1, 0, 30, 0).unwrap(),
            Utc.with_ymd_and_hms(2026, 7, 1, 4, 30, 0).unwrap(),
            "first_half",
        );
        // Afternoon: late — punch-in at 12:45 instead of 12:30
        let afternoon = make_break_split_entry(
            Utc.with_ymd_and_hms(2026, 7, 1, 5, 45, 0).unwrap(),
            Utc.with_ymd_and_hms(2026, 7, 1, 9, 30, 0).unwrap(),
            "second_half",
        );

        let entries = vec![morning, afternoon];
        let entry_refs: Vec<&TimeEntry> = entries.iter().collect();

        let att_rules = AttendanceRules {
            work_unit_minutes: 60,
            rounding_policy: Default::default(),
            early_arrival_strategy: Default::default(),
            grace_period_minutes: None,
        };

        let (_matched, _rules, segment_calc_inputs) = build_segment_calc_inputs(
            &[&segment],
            &entry_refs,
            date,
            tz,
            &[],
            &att_rules,
            &HashMap::new(),
        );

        let sessions = build_break_split_sessions_json(
            &segment,
            &entry_refs,
            &segment_calc_inputs[0],
            &att_rules,
            tz,
            None,
            false,
            5, // grace: 5 min — 12:45 vs 12:30 = 15 min late > grace
        )
        .expect("must produce sessions");

        let arr = sessions.as_array().unwrap();

        // Morning: on time
        assert_eq!(arr[0]["status"], "present");

        // Afternoon: 15 min late with 5-min grace → "late"
        assert_eq!(arr[1]["status"], "late");
    }

    #[test]
    fn test_non_break_split_segment_no_sessions() {
        use crate::services::report_service::daily::build_break_split_sessions_json;

        let tz = Ho_Chi_Minh;
        let segment = make_vp_segment();

        // Normal entry — no break_split metadata
        let entry = make_time_entry(
            Utc.with_ymd_and_hms(2026, 7, 1, 0, 30, 0).unwrap(),
            Utc.with_ymd_and_hms(2026, 7, 1, 9, 30, 0).unwrap(),
        );

        let entries = vec![entry];
        let entry_refs: Vec<&TimeEntry> = entries.iter().collect();

        let att_rules = AttendanceRules {
            work_unit_minutes: 60,
            rounding_policy: Default::default(),
            early_arrival_strategy: Default::default(),
            grace_period_minutes: None,
        };

        let sessions = build_break_split_sessions_json(
            &segment,
            &entry_refs,
            &[], // no matched calc inputs needed for the None path
            &att_rules,
            tz,
            None,
            false,
            15,
        );

        assert!(
            sessions.is_none(),
            "Non-break-split segment must return None (no sessions)"
        );
    }

    #[test]
    fn test_entry_break_split_helper() {
        use crate::services::report_service::daily::entry_break_split;

        let e = make_break_split_entry(
            Utc.with_ymd_and_hms(2026, 7, 1, 0, 30, 0).unwrap(),
            Utc.with_ymd_and_hms(2026, 7, 1, 4, 30, 0).unwrap(),
            "first_half",
        );
        assert_eq!(entry_break_split(&e).as_deref(), Some("first_half"));

        let e2 = make_break_split_entry(
            Utc.with_ymd_and_hms(2026, 7, 1, 5, 30, 0).unwrap(),
            Utc.with_ymd_and_hms(2026, 7, 1, 9, 30, 0).unwrap(),
            "second_half",
        );
        assert_eq!(entry_break_split(&e2).as_deref(), Some("second_half"));

        let normal = make_time_entry(
            Utc.with_ymd_and_hms(2026, 7, 1, 0, 30, 0).unwrap(),
            Utc.with_ymd_and_hms(2026, 7, 1, 9, 30, 0).unwrap(),
        );
        assert_eq!(entry_break_split(&normal), None);
    }
}
