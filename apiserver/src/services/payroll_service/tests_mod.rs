use super::*;

fn d(y: i32, m: u32, day: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(y, m, day).expect("valid date")
}

// --- Tests for calculate_daily_rate ---

#[test]
fn test_daily_rate_normal() {
    // Base salary 26_000 / 26 working days = 1_000 per day
    let rate = FinancialCalculator::calculate_daily_rate(Decimal::from(26_000), Decimal::from(26));
    assert_eq!(rate, Decimal::from(1_000));
}

#[test]
fn test_daily_rate_zero_working_days_returns_zero() {
    // Edge case: zero working days should not panic/divide-by-zero
    let rate = FinancialCalculator::calculate_daily_rate(Decimal::from(30_000), Decimal::ZERO);
    assert_eq!(rate, Decimal::ZERO);
}

#[test]
fn test_daily_rate_zero_base_returns_zero() {
    let rate = FinancialCalculator::calculate_daily_rate(Decimal::ZERO, Decimal::from(26));
    assert_eq!(rate, Decimal::ZERO);
}

// --- Tests for apply_ot_cap ---

#[test]
fn test_ot_cap_no_cap_needed() {
    // 10 OT units, cap allows 20 hours (= 40 units at 30-min work_unit)
    let (units, pay) = FinancialCalculator::apply_ot_cap(
        10,                   // total_ot_units
        Decimal::from(5_000), // total_overtime_pay
        Decimal::from(20),    // max_monthly_hours = 20h
        30,                   // work_unit_minutes = 30 min
    );
    // 20h * 60 = 1200 min / 30 = 40 max units; 10 < 40 → no cap
    assert_eq!(units, 10);
    assert_eq!(pay, Decimal::from(5_000));
}

#[test]
fn test_ot_cap_applied() {
    // 50 OT units exceed cap of 40 → should be reduced
    let (units, pay) = FinancialCalculator::apply_ot_cap(
        50,                    // total_ot_units
        Decimal::from(10_000), // total_overtime_pay
        Decimal::from(20),     // max_monthly_hours = 20h
        30,                    // work_unit_minutes = 30 min
    );
    // Max units = 20*60/30 = 40
    assert_eq!(units, 40);
    // Pay proportional: 10_000 * (40/50) = 8_000
    assert_eq!(pay, Decimal::from(8_000));
}

#[test]
fn test_ot_cap_exact_limit() {
    let (units, pay) = FinancialCalculator::apply_ot_cap(
        40, // exactly at limit
        Decimal::from(8_000),
        Decimal::from(20), // 20h * 60 / 30 = 40
        30,
    );
    assert_eq!(units, 40); // no reduction
    assert_eq!(pay, Decimal::from(8_000));
}

#[test]
fn test_ot_cap_zero_units() {
    let (units, pay) = FinancialCalculator::apply_ot_cap(0, Decimal::ZERO, Decimal::from(20), 30);
    assert_eq!(units, 0);
    assert_eq!(pay, Decimal::ZERO);
}

// --- Tests for SegmentSnapshot struct ---

#[test]
fn test_segment_snapshot_construction() {
    let snap = SegmentSnapshot {
        seg_start: d(2026, 3, 1),
        seg_end: d(2026, 3, 15),
        department_id: Some(Uuid::new_v4()),
        total_base: Decimal::from(13_000),
        std_working_days: Decimal::from(26),
        hours_per_day: Decimal::from(8),
        actual_worked_days: Decimal::from(11),
        paid_days: Decimal::from(11),
        amount: Decimal::from(5_500),
        source: "workspace".to_string(),
    };

    assert_eq!(snap.seg_start, d(2026, 3, 1));
    assert_eq!(snap.seg_end, d(2026, 3, 15));
    assert!(snap.department_id.is_some());
    assert_eq!(snap.source, "workspace");
}

#[test]
fn test_segment_snapshot_daily_rate_via_helper() {
    let snap = SegmentSnapshot {
        seg_start: d(2026, 3, 1),
        seg_end: d(2026, 3, 31),
        department_id: None,
        total_base: Decimal::from(26_000),
        std_working_days: Decimal::from(26),
        hours_per_day: Decimal::from(8),
        actual_worked_days: Decimal::from(22),
        paid_days: Decimal::from(22),
        amount: Decimal::from(22_000),
        source: "system".to_string(),
    };

    let daily_rate =
        FinancialCalculator::calculate_daily_rate(snap.total_base, snap.std_working_days);
    assert_eq!(daily_rate, Decimal::from(1_000));
}

// --- Tests for build_payroll_contexts error paths ---

fn make_chunk_shared_data() -> Arc<ChunkSharedData> {
    Arc::new(ChunkSharedData {
        workspace: crate::models::Workspace::new(
            Uuid::new_v4(),
            "Test".into(),
            "YYYY-MM-DD".into(),
            "HH:mm".into(),
            "UTC".into(),
            "#000".into(),
            "en".into(),
            None,
            None,
            "split".into(),
            Decimal::from(2),
            chrono::Utc::now(),
            None,
        ),
        tz: chrono_tz::UTC,
        start_date: d(2026, 3, 1),
        end_date: d(2026, 3, 31),
        start_dt: chrono::Utc::now(),
        end_dt: chrono::Utc::now(),
        token: crate::diesel_pool::TenantAuthToken::system_override(Uuid::new_v4()),
        condition_type_catalog: Arc::new(HashMap::new()),
        overtime_data: Arc::new(
            crate::repositories::overtime_policy_repository::WorkspaceOvertimeData {
                policies: HashMap::new(),
                assignments: Vec::new(),
            },
        ),
        holidays_map: Arc::new(HashMap::new()),
        roster_data: Arc::new(crate::cache::RosterData {
            shifts: HashMap::new(),
            fixed_assignments: HashMap::new(),
            pattern_assignments: HashMap::new(),
            active_rules: Vec::new(),
            rule_patterns: HashMap::new(),
            dept_history: HashMap::new(),
            shift_intervals: HashMap::new(),
        }),
        ws_payroll_settings_history: Arc::new(Vec::new()),
        month_end_payroll_settings: Arc::new(None),
        active_policies_with_tiers: Arc::new(Vec::new()),
        salary_base_groups: Arc::new(Vec::new()),
        daily_summaries: HashMap::new(),
    })
}

#[test]
fn test_build_contexts_employment_without_employee() {
    // When an employment_id has no mapping in employment_to_employee,
    // it should return Err with a descriptive error message.
    let employment_id = Uuid::new_v4();
    let d = make_chunk_shared_data();

    let results = build_payroll_contexts(
        &[employment_id],
        &HashMap::new(), // empty: no employment→employee mapping
        &HashMap::new(),
        &HashMap::new(),
        &HashMap::new(),
        &HashMap::new(),
        &std::collections::HashSet::new(),
        &mut BatchMaps {
            employees: HashMap::new(),
            time_map: HashMap::new(),
            roster_map: HashMap::new(),
            conditions_map: HashMap::new(),
            policies_map: HashMap::new(),
            emp_configs_map: HashMap::new(),
            dept_map: HashMap::new(),
            salary_map: HashMap::new(),
            leave_breakdowns_map: HashMap::new(),
            unpaid_holidays_map: HashMap::new(),
            month_end_policies_map: HashMap::new(),
            assignments_map: HashMap::new(),
        },
        &d,
    );

    assert_eq!(results.len(), 1);
    let (eid, result) = &results[0];
    assert_eq!(*eid, employment_id);
    match result {
        Err(error_result) => {
            assert_eq!(error_result.status, "error");
            assert!(error_result
                .error_message
                .as_ref()
                .unwrap()
                .contains("no associated employee"));
            assert_eq!(error_result.employment_id, employment_id);
            assert!(error_result.payslip.is_none());
        }
        Ok(_) => panic!("Expected Err, got Ok"),
    }
}

#[test]
fn test_build_contexts_employee_data_not_found() {
    // When employee_id exists in mapping but employee data
    // was not fetched (not in employees map), return Err.
    let employment_id = Uuid::new_v4();
    let employee_id = Uuid::new_v4();
    let d = make_chunk_shared_data();

    let mut emp_to_employee = HashMap::new();
    emp_to_employee.insert(employment_id, employee_id);

    let results = build_payroll_contexts(
        &[employment_id],
        &emp_to_employee, // mapping exists...
        &HashMap::new(),
        &HashMap::new(),
        &HashMap::new(),
        &HashMap::new(),
        &std::collections::HashSet::new(),
        &mut BatchMaps {
            employees: HashMap::new(), // ...but employee data doesn't
            time_map: HashMap::new(),
            roster_map: HashMap::new(),
            conditions_map: HashMap::new(),
            policies_map: HashMap::new(),
            emp_configs_map: HashMap::new(),
            dept_map: HashMap::new(),
            salary_map: HashMap::new(),
            leave_breakdowns_map: HashMap::new(),
            unpaid_holidays_map: HashMap::new(),
            month_end_policies_map: HashMap::new(),
            assignments_map: HashMap::new(),
        },
        &d,
    );

    assert_eq!(results.len(), 1);
    let (eid, result) = &results[0];
    assert_eq!(*eid, employment_id);
    match result {
        Err(error_result) => {
            assert_eq!(error_result.status, "error");
            assert!(
                error_result
                    .error_message
                    .as_ref()
                    .unwrap()
                    .contains("not found in batch fetch"),
                "Expected 'not found in batch fetch', got: {:?}",
                error_result.error_message
            );
            assert_eq!(error_result.employment_id, employment_id);
            assert_eq!(error_result.employee_id, employee_id);
            assert!(error_result.payslip.is_none());
        }
        Ok(_) => panic!("Expected Err, got Ok"),
    }
}
