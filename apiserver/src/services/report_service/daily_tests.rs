//! Unit tests for extracted segment-report functions.
//!
//! Tests `compute_employment_segment_rows`, `generate_no_shift_fallback_rows`,
//! and `filter_by_shift_type` in isolation.  All functions are pure (no I/O)
//! so no mock repositories are needed.

use super::ReportExtras;
use crate::models::DailyAttendanceReport;
use chrono::{NaiveDate, NaiveTime, TimeZone, Utc};
use chrono_tz::Tz;
use std::collections::HashMap;
use uuid::Uuid;

// ── Reuse helpers from the existing daily.rs test module ────────────────────

/// Create a time entry at the given local times (in tz), expressed as UTC.
fn make_test_entry(
    punch_in_local: chrono::DateTime<Tz>,
    punch_out_local: Option<chrono::DateTime<Tz>>,
    entry_type: crate::services::pairing::types::EntryType,
) -> crate::models::TimeEntry {
    crate::models::TimeEntry::new(
        Uuid::new_v4(),
        Uuid::new_v4(),
        Uuid::new_v4(),
        None,
        None,
        punch_in_local.with_timezone(&Utc),
        punch_out_local.map(|t| t.with_timezone(&Utc)),
        Some(60),
        false,
        entry_type,
        None,
        None,
        Utc::now(),
        Utc::now(),
    )
}

/// Build a test `ResolvedSegment` with just the start/end times filled.
fn make_test_segment(start_h: u32, start_m: u32, end_h: u32, end_m: u32) -> crate::models::ResolvedSegment {
    crate::models::ResolvedSegment {
        template_id: Uuid::new_v4(),
        interval_id: Some(Uuid::new_v4()),
        sort_order: 0,
        scheduled_start: NaiveTime::from_hms_opt(start_h, start_m, 0).unwrap(),
        scheduled_end: NaiveTime::from_hms_opt(end_h, end_m, 0).unwrap(),
        source: "fixed".to_string(),
        date: None,
        shift_name: Some("Test Shift".to_string()),
        shift_symbol: Some("TS".to_string()),
        interval_name: None,
        segment_type: "work".to_string(),
        break_duration_minutes: None,
        break_start: None,
        break_end: None,
        is_break_auto_deduct: None,
        break_is_overtime: None,
        allow_early_ot: false,
        max_early_ot_minutes: None,
        early_punch_in_grace_period_minutes: 0,
        ot_aggregation_mode: crate::services::attendance_calculator::OtAggregationMode::Aggregate,
        is_night_shift: false,
        traces: vec![],
        is_holiday: false,
        is_holiday_paid: false,
        holiday_name: None,
    }
}

fn hcm_tz() -> Tz {
    "Asia/Ho_Chi_Minh".parse().unwrap()
}

// ── Test-data builders ─────────────────────────────────────────────────────

/// Build a segment with a specific date.
fn make_dated_segment(
    start_h: u32, start_m: u32, end_h: u32, end_m: u32,
    date: NaiveDate,
) -> crate::models::ResolvedSegment {
    let mut seg = make_test_segment(start_h, start_m, end_h, end_m);
    seg.date = Some(date);
    seg
}

/// Build an empty `ReportExtras`.
fn make_test_extras() -> ReportExtras {
    ReportExtras::new(
        HashMap::new(), // holidays_map
        HashMap::new(), // ot_status_map
        HashMap::new(), // leave_details_map
    )
}

/// Build a minimal preamble with the given maps populated.
fn make_test_preamble(
    tz: Tz,
    employment_to_employee: HashMap<Uuid, Uuid>,
    entries_map: HashMap<Uuid, Vec<crate::models::TimeEntry>>,
    employments: Vec<crate::models::Employment>,
    paginated_employment_ids: Vec<Uuid>,
    paginated_employees: Vec<crate::repositories::employee::EmployeeBasicInfo>,
) -> crate::services::report_service::ReportDataPreamble {
    let emp_ids: Vec<Uuid> = paginated_employees.iter().map(|e| e.id).collect();
    crate::services::report_service::ReportDataPreamble::test_new(
        tz,
        Uuid::new_v4(),
        1,                                               // page
        20,                                              // limit
        paginated_employees,
        i64::try_from(emp_ids.len()).unwrap_or(0),       // total_employees
        employments,
        paginated_employment_ids,
        employment_to_employee,
        emp_ids,
        entries_map,
        HashMap::new(), // attendance_rules_map (defaults used)
        HashMap::new(), // working_conditions_map
        HashMap::new(), // condition_type_catalog
        HashMap::new(), // employer_name_map
    )
}

/// Build a minimal Employment for test use.
fn make_test_employment(employment_id: Uuid, employee_id: Uuid, employer_id: Uuid) -> crate::models::Employment {
    crate::models::Employment::new(
        employment_id,
        Uuid::new_v4(),             // workspace_id
        employee_id,
        employer_id,
        None,                       // primary_location_id
        None,                       // department_id
        "active".to_string(),       // status
        NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
        None,                       // end_date
        Some("Worker".to_string()), // job_title
        "full_time".to_string(),    // employment_type
        None,                       // manager_employment_id
        Utc::now(),
        Utc::now(),
        None, // termination_reason
        None, // notes
    )
}

// ═══════════════════════════════════════════════════════════════════════════
// compute_employment_segment_rows tests
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn compute_single_segment_with_entry_produces_one_row() {
    let tz = hcm_tz();
    let date = NaiveDate::from_ymd_opt(2026, 7, 15).unwrap();
    let employee_id = Uuid::new_v4();
    let employment_id = Uuid::new_v4();

    // Segment 08:00-12:00 on that date.
    let seg = make_dated_segment(8, 0, 12, 0, date);
    let emp_segs = crate::models::EmployeeSegments::new(
        employment_id,
        employee_id,
        "EMP001".to_string(),
        "Test User".to_string(),
        vec![seg],
    );

    // Entry from 08:05 to 11:55 (within segment).
    let entry = make_test_entry(
        tz.with_ymd_and_hms(2026, 7, 15, 8, 5, 0).unwrap(),
        Some(tz.with_ymd_and_hms(2026, 7, 15, 11, 55, 0).unwrap()),
        crate::services::pairing::types::EntryType::Work,
    );

    let mut entries_map = HashMap::new();
    entries_map.insert(employment_id, vec![entry]);
    let mut etoe = HashMap::new();
    etoe.insert(employment_id, employee_id);

    let employment = make_test_employment(employment_id, employee_id, Uuid::new_v4());
    let employee_info = crate::repositories::employee::EmployeeBasicInfo {
        id: employee_id,
        full_name: "Test User".to_string(),
        employee_id_number: "EMP001".to_string(),
    };

    let preamble = make_test_preamble(
        tz,
        etoe,
        entries_map,
        vec![employment],
        vec![employment_id],
        vec![employee_info],
    );
    let extras = make_test_extras();

    let rows = super::compute_employment_segment_rows(&emp_segs, &preamble, &extras, tz);

    assert_eq!(rows.len(), 1, "should produce exactly 1 row");
    let row = &rows[0];
    assert_eq!(row.employee_id, employee_id);
    assert_eq!(row.employment_id, employment_id);
    assert_eq!(row.date, date);
    assert_eq!(row.shift_name.as_deref(), Some("Test Shift"));
    assert_eq!(row.segment_type.as_deref(), Some("work"));
    assert!(row.segment_id.is_some(), "segment should have an interval_id");
}

#[test]
fn compute_multiple_segments_two_entries_produces_two_rows() {
    let tz = hcm_tz();
    let date = NaiveDate::from_ymd_opt(2026, 7, 15).unwrap();
    let employee_id = Uuid::new_v4();
    let employment_id = Uuid::new_v4();

    let morning = make_dated_segment(8, 0, 12, 0, date);
    let afternoon = make_dated_segment(13, 0, 17, 0, date);
    let emp_segs = crate::models::EmployeeSegments::new(
        employment_id,
        employee_id,
        "EMP002".to_string(),
        "Multi Segment User".to_string(),
        vec![morning, afternoon],
    );

    let entry1 = make_test_entry(
        tz.with_ymd_and_hms(2026, 7, 15, 8, 5, 0).unwrap(),
        Some(tz.with_ymd_and_hms(2026, 7, 15, 12, 0, 0).unwrap()),
        crate::services::pairing::types::EntryType::Work,
    );
    let entry2 = make_test_entry(
        tz.with_ymd_and_hms(2026, 7, 15, 13, 1, 0).unwrap(),
        Some(tz.with_ymd_and_hms(2026, 7, 15, 16, 59, 0).unwrap()),
        crate::services::pairing::types::EntryType::Work,
    );

    let mut entries_map = HashMap::new();
    entries_map.insert(employment_id, vec![entry1, entry2]);
    let mut etoe = HashMap::new();
    etoe.insert(employment_id, employee_id);

    let employment = make_test_employment(employment_id, employee_id, Uuid::new_v4());
    let employee_info = crate::repositories::employee::EmployeeBasicInfo {
        id: employee_id,
        full_name: "Multi Segment User".to_string(),
        employee_id_number: "EMP002".to_string(),
    };
    let preamble = make_test_preamble(
        tz, etoe, entries_map,
        vec![employment], vec![employment_id], vec![employee_info],
    );
    let extras = make_test_extras();

    let rows = super::compute_employment_segment_rows(&emp_segs, &preamble, &extras, tz);

    assert_eq!(rows.len(), 2, "should produce 2 rows (one per segment)");
    // Segments should be in order; first row is morning.
    assert_eq!(rows[0].sort_order, Some(0));
    assert_eq!(rows[1].sort_order, Some(1));
}

#[test]
fn compute_no_segments_returns_empty() {
    let tz = hcm_tz();
    let employee_id = Uuid::new_v4();
    let employment_id = Uuid::new_v4();

    let emp_segs = crate::models::EmployeeSegments::new(
        employment_id, employee_id,
        "EMP003".to_string(), "Empty User".to_string(),
        vec![], // no segments
    );

    let mut etoe = HashMap::new();
    etoe.insert(employment_id, employee_id);
    let employment = make_test_employment(employment_id, employee_id, Uuid::new_v4());
    let employee_info = crate::repositories::employee::EmployeeBasicInfo {
        id: employee_id,
        full_name: "Empty User".to_string(),
        employee_id_number: "EMP003".to_string(),
    };
    let preamble = make_test_preamble(
        tz, etoe, HashMap::new(),
        vec![employment], vec![employment_id], vec![employee_info],
    );
    let extras = make_test_extras();

    let rows = super::compute_employment_segment_rows(&emp_segs, &preamble, &extras, tz);

    assert!(rows.is_empty(), "no segments → no rows");
}

#[test]
fn compute_no_entries_produces_row_with_zero_units() {
    let tz = hcm_tz();
    let date = NaiveDate::from_ymd_opt(2026, 7, 15).unwrap();
    let employee_id = Uuid::new_v4();
    let employment_id = Uuid::new_v4();

    let seg = make_dated_segment(8, 0, 17, 0, date);
    let emp_segs = crate::models::EmployeeSegments::new(
        employment_id, employee_id,
        "EMP004".to_string(), "No Entry User".to_string(),
        vec![seg],
    );

    let mut entries_map: HashMap<Uuid, Vec<crate::models::TimeEntry>> = HashMap::new();
    entries_map.insert(employment_id, vec![]); // employment exists but has no entries
    let mut etoe = HashMap::new();
    etoe.insert(employment_id, employee_id);

    let employment = make_test_employment(employment_id, employee_id, Uuid::new_v4());
    let employee_info = crate::repositories::employee::EmployeeBasicInfo {
        id: employee_id,
        full_name: "No Entry User".to_string(),
        employee_id_number: "EMP004".to_string(),
    };
    let preamble = make_test_preamble(
        tz, etoe, entries_map,
        vec![employment], vec![employment_id], vec![employee_info],
    );
    let extras = make_test_extras();

    let rows = super::compute_employment_segment_rows(&emp_segs, &preamble, &extras, tz);

    assert_eq!(rows.len(), 1, "segment exists → one row even without entries");
    assert_eq!(rows[0].std_work_units, 0);
    assert_eq!(rows[0].ot_work_units, 0);
}

#[test]
fn compute_overnight_shift_produces_row() {
    let tz = hcm_tz();
    let date = NaiveDate::from_ymd_opt(2026, 7, 15).unwrap();
    let employee_id = Uuid::new_v4();
    let employment_id = Uuid::new_v4();

    // Night shift 22:00 → 06:00 next day
    let seg = make_dated_segment(22, 0, 6, 0, date);
    let emp_segs = crate::models::EmployeeSegments::new(
        employment_id, employee_id,
        "EMP005".to_string(), "Night User".to_string(),
        vec![seg],
    );

    // Entry at 23:00 on the same date
    let entry = make_test_entry(
        tz.with_ymd_and_hms(2026, 7, 15, 23, 0, 0).unwrap(),
        Some(tz.with_ymd_and_hms(2026, 7, 16, 5, 59, 0).unwrap()),
        crate::services::pairing::types::EntryType::Work,
    );

    let mut entries_map = HashMap::new();
    entries_map.insert(employment_id, vec![entry]);
    let mut etoe = HashMap::new();
    etoe.insert(employment_id, employee_id);

    let employment = make_test_employment(employment_id, employee_id, Uuid::new_v4());
    let employee_info = crate::repositories::employee::EmployeeBasicInfo {
        id: employee_id,
        full_name: "Night User".to_string(),
        employee_id_number: "EMP005".to_string(),
    };
    let preamble = make_test_preamble(
        tz, etoe, entries_map,
        vec![employment], vec![employment_id], vec![employee_info],
    );
    let extras = make_test_extras();

    let rows = super::compute_employment_segment_rows(&emp_segs, &preamble, &extras, tz);

    assert_eq!(rows.len(), 1, "overnight segment should produce one row");
    assert_eq!(rows[0].date, date, "row date is the segment date");
}

#[test]
fn compute_employment_mapping_miss_gracefully_skips() {
    let tz = hcm_tz();
    let date = NaiveDate::from_ymd_opt(2026, 7, 15).unwrap();
    let employee_id = Uuid::new_v4();
    let employment_id = Uuid::new_v4();

    let seg = make_dated_segment(8, 0, 12, 0, date);
    let emp_segs = crate::models::EmployeeSegments::new(
        employment_id, employee_id,
        "EMP006".to_string(), "Orphan User".to_string(),
        vec![seg],
    );

    // employment_to_employee does NOT contain the mapping.
    let etoe: HashMap<Uuid, Uuid> = HashMap::new();
    let employment = make_test_employment(employment_id, employee_id, Uuid::new_v4());
    let employee_info = crate::repositories::employee::EmployeeBasicInfo {
        id: employee_id,
        full_name: "Orphan User".to_string(),
        employee_id_number: "EMP006".to_string(),
    };
    let preamble = make_test_preamble(
        tz, etoe, HashMap::new(),
        vec![employment], vec![employment_id], vec![employee_info],
    );
    let extras = make_test_extras();

    let rows = super::compute_employment_segment_rows(&emp_segs, &preamble, &extras, tz);

    assert!(rows.is_empty(), "mapping miss → gracefully return empty");
}

#[test]
fn compute_multiple_dates_produces_rows_per_date() {
    let tz = hcm_tz();
    let date1 = NaiveDate::from_ymd_opt(2026, 7, 15).unwrap();
    let date2 = NaiveDate::from_ymd_opt(2026, 7, 16).unwrap();
    let employee_id = Uuid::new_v4();
    let employment_id = Uuid::new_v4();

    let seg1 = make_dated_segment(8, 0, 12, 0, date1);
    let seg2 = make_dated_segment(8, 0, 12, 0, date2);
    let emp_segs = crate::models::EmployeeSegments::new(
        employment_id, employee_id,
        "EMP007".to_string(), "Multi Date User".to_string(),
        vec![seg1, seg2],
    );

    // Entries for both dates.
    let entry_day1 = make_test_entry(
        tz.with_ymd_and_hms(2026, 7, 15, 8, 5, 0).unwrap(),
        Some(tz.with_ymd_and_hms(2026, 7, 15, 11, 59, 0).unwrap()),
        crate::services::pairing::types::EntryType::Work,
    );
    let entry_day2 = make_test_entry(
        tz.with_ymd_and_hms(2026, 7, 16, 8, 1, 0).unwrap(),
        Some(tz.with_ymd_and_hms(2026, 7, 16, 12, 0, 0).unwrap()),
        crate::services::pairing::types::EntryType::Work,
    );

    let mut entries_map = HashMap::new();
    entries_map.insert(employment_id, vec![entry_day1, entry_day2]);
    let mut etoe = HashMap::new();
    etoe.insert(employment_id, employee_id);

    let employment = make_test_employment(employment_id, employee_id, Uuid::new_v4());
    let employee_info = crate::repositories::employee::EmployeeBasicInfo {
        id: employee_id,
        full_name: "Multi Date User".to_string(),
        employee_id_number: "EMP007".to_string(),
    };
    let preamble = make_test_preamble(
        tz, etoe, entries_map,
        vec![employment], vec![employment_id], vec![employee_info],
    );
    let extras = make_test_extras();

    let rows = super::compute_employment_segment_rows(&emp_segs, &preamble, &extras, tz);

    assert_eq!(rows.len(), 2, "two segments on two dates → two rows");
    let dates: Vec<NaiveDate> = rows.iter().map(|r| r.date).collect();
    assert!(dates.contains(&date1));
    assert!(dates.contains(&date2));
}

// ═══════════════════════════════════════════════════════════════════════════
// generate_no_shift_fallback_rows tests
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn no_shift_fallback_single_employment_produces_off_row() {
    let tz = hcm_tz();
    let start_date = NaiveDate::from_ymd_opt(2026, 7, 15).unwrap();
    let end_date = NaiveDate::from_ymd_opt(2026, 7, 15).unwrap();
    let employee_id = Uuid::new_v4();
    let employment_id = Uuid::new_v4();

    // No employee segments at all.
    let employee_segments: Vec<crate::models::EmployeeSegments> = vec![];

    let employment = make_test_employment(employment_id, employee_id, Uuid::new_v4());
    let mut etoe = HashMap::new();
    etoe.insert(employment_id, employee_id);
    let employee_info = crate::repositories::employee::EmployeeBasicInfo {
        id: employee_id,
        full_name: "Fallback User".to_string(),
        employee_id_number: "EMP010".to_string(),
    };
    let preamble = make_test_preamble(
        tz, etoe, HashMap::new(),
        vec![employment], vec![employment_id], vec![employee_info],
    );

    let rows = super::generate_no_shift_fallback_rows(
        &preamble, &employee_segments, start_date, end_date,
    );

    assert_eq!(rows.len(), 1, "one employment, one date → one Off row");
    assert_eq!(rows[0].date, start_date);
    assert_eq!(rows[0].employee_id, employee_id);
    assert_eq!(rows[0].employment_id, employment_id);
    assert_eq!(rows[0].status, crate::services::attendance_utils::AttendanceStatus::Off);
    assert_eq!(rows[0].std_work_units, 0);
    assert_eq!(rows[0].ot_work_units, 0);
    assert!(rows[0].shift_name.is_none());
    assert!(rows[0].actual_punch_in.is_none());
}

#[test]
fn no_shift_fallback_skips_employments_with_segments() {
    let tz = hcm_tz();
    let start_date = NaiveDate::from_ymd_opt(2026, 7, 15).unwrap();
    let end_date = NaiveDate::from_ymd_opt(2026, 7, 15).unwrap();
    let employee_id = Uuid::new_v4();
    let employment_id = Uuid::new_v4();

    // This employment HAS a segment — should be skipped by fallback.
    let seg = make_dated_segment(8, 0, 17, 0, start_date);
    let employee_segments = vec![
        crate::models::EmployeeSegments::new(
            employment_id, employee_id,
            "EMP011".to_string(), "Has Segment User".to_string(),
            vec![seg],
        ),
    ];

    let employment = make_test_employment(employment_id, employee_id, Uuid::new_v4());
    let mut etoe = HashMap::new();
    etoe.insert(employment_id, employee_id);
    let employee_info = crate::repositories::employee::EmployeeBasicInfo {
        id: employee_id,
        full_name: "Has Segment User".to_string(),
        employee_id_number: "EMP011".to_string(),
    };
    let preamble = make_test_preamble(
        tz, etoe, HashMap::new(),
        vec![employment], vec![employment_id], vec![employee_info],
    );

    let rows = super::generate_no_shift_fallback_rows(
        &preamble, &employee_segments, start_date, end_date,
    );

    assert!(rows.is_empty(), "employment with segments → no fallback rows");
}

#[test]
fn no_shift_fallback_multiple_dates_produces_multiple_off_rows() {
    let tz = hcm_tz();
    let start_date = NaiveDate::from_ymd_opt(2026, 7, 15).unwrap();
    let end_date = NaiveDate::from_ymd_opt(2026, 7, 17).unwrap(); // 3 days
    let employee_id = Uuid::new_v4();
    let employment_id = Uuid::new_v4();

    let employee_segments: Vec<crate::models::EmployeeSegments> = vec![];
    let employment = make_test_employment(employment_id, employee_id, Uuid::new_v4());
    let mut etoe = HashMap::new();
    etoe.insert(employment_id, employee_id);
    let employee_info = crate::repositories::employee::EmployeeBasicInfo {
        id: employee_id,
        full_name: "Multi Day Fallback".to_string(),
        employee_id_number: "EMP012".to_string(),
    };
    let preamble = make_test_preamble(
        tz, etoe, HashMap::new(),
        vec![employment], vec![employment_id], vec![employee_info],
    );

    let rows = super::generate_no_shift_fallback_rows(
        &preamble, &employee_segments, start_date, end_date,
    );

    assert_eq!(rows.len(), 3, "3-day range → 3 Off rows");
    assert_eq!(rows[0].date, NaiveDate::from_ymd_opt(2026, 7, 15).unwrap());
    assert_eq!(rows[1].date, NaiveDate::from_ymd_opt(2026, 7, 16).unwrap());
    assert_eq!(rows[2].date, NaiveDate::from_ymd_opt(2026, 7, 17).unwrap());
    // All rows are Off.
    for row in &rows {
        assert_eq!(row.status, crate::services::attendance_utils::AttendanceStatus::Off);
    }
}

#[test]
fn no_shift_fallback_empty_input_returns_empty() {
    let tz = hcm_tz();
    let start_date = NaiveDate::from_ymd_opt(2026, 7, 15).unwrap();
    let end_date = NaiveDate::from_ymd_opt(2026, 7, 15).unwrap();

    let employee_segments: Vec<crate::models::EmployeeSegments> = vec![];
    // Preamble with no employments at all.
    let preamble = crate::services::report_service::ReportDataPreamble::empty(
        tz, Uuid::new_v4(), 1, 20, vec![], 0,
    );

    let rows = super::generate_no_shift_fallback_rows(
        &preamble, &employee_segments, start_date, end_date,
    );

    assert!(rows.is_empty(), "no employments → empty result");
}

// ═══════════════════════════════════════════════════════════════════════════
// filter_by_shift_type tests (exercise the existing public function)
// ═══════════════════════════════════════════════════════════════════════════

/// Build a minimal `DailyAttendanceReport` for filter testing.
fn make_minimal_report_row(shift_name: Option<String>) -> DailyAttendanceReport {
    DailyAttendanceReport {
        date: NaiveDate::from_ymd_opt(2026, 7, 15).unwrap(),
        employment_id: Uuid::new_v4(),
        employee_id: Uuid::new_v4(),
        employee_name: "Test".to_string(),
        employee_id_number: "T001".to_string(),
        shift_symbol: None,
        shift_name,
        scheduled_start: None,
        scheduled_end: None,
        grace_period_minutes: Some(0),
        break_duration_minutes: None,
        break_start: None,
        break_end: None,
        is_break_auto_deduct: None,
        actual_punch_in: None,
        actual_punch_out: None,
        std_work_units: 0,
        ot_work_units: 0,
        ot_claim_status: None,
        unit_type: crate::models::enums::TimekeepingUnit::DAY,
        unit_coefficient_hours: 1.0,
        computed_std_hours: 0.0,
        computed_ot_hours: 0.0,
        status: crate::services::attendance_utils::AttendanceStatus::Off,
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
        break_split: None,
    }
}

#[test]
fn filter_by_shift_type_no_filter_passes_all() {
    let rows = vec![
        make_minimal_report_row(Some("Morning".to_string())),
        make_minimal_report_row(Some("Night".to_string())),
        make_minimal_report_row(None),
    ];
    let filtered = super::filter_by_shift_type(rows, None);
    assert_eq!(filtered.len(), 3, "no filter → all rows pass");
}

#[test]
fn filter_by_shift_type_filters_by_name() {
    let rows = vec![
        make_minimal_report_row(Some("Morning".to_string())),
        make_minimal_report_row(Some("Night".to_string())),
        make_minimal_report_row(Some("Morning".to_string())),
        make_minimal_report_row(None),
    ];
    let filtered = super::filter_by_shift_type(
        rows,
        Some(vec!["Morning".to_string()]),
    );
    assert_eq!(filtered.len(), 2, "only Morning rows survive");
    for row in &filtered {
        assert_eq!(row.shift_name.as_deref(), Some("Morning"));
    }
}

#[test]
fn filter_by_shift_type_none_shift_name_excluded() {
    let rows = vec![
        make_minimal_report_row(Some("Morning".to_string())),
        make_minimal_report_row(None),
    ];
    let filtered = super::filter_by_shift_type(
        rows,
        Some(vec!["Morning".to_string()]),
    );
    assert_eq!(filtered.len(), 1, "None shift_name is excluded by explicit filter");
}
