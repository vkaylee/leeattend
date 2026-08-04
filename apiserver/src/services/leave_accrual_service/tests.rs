use super::*;
use crate::models::{AccrualFrequency, EmployeeContract, LeavePolicyConfig, NewEmployeeContractParams, NewTimeEntryParams, SeniorityBonus};
use chrono::NaiveDate;
use uuid::Uuid;

fn nd(y: i32, m: u32, d: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(y, m, d).expect("valid test date")
}

fn mock_contract(
    hire: NaiveDate,
    term: Option<NaiveDate>,
) -> crate::models::contract::EmployeeContract {
    EmployeeContract::new(NewEmployeeContractParams {
        id: Uuid::new_v4(),
        workspace_id: Uuid::new_v4(),
        employment_id: Uuid::new_v4(),
        contract_number: None,
        contract_type: crate::models::enums::ContractType::Indefinite,
        status: crate::models::enums::ContractStatus::Active,
        start_date: hire,
        end_date: term,
        document_id: None,
        notes: None,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    })
}

#[test]
fn test_calculate_accrual_amount() {
    let config = LeavePolicyConfig {
        frequency: AccrualFrequency::Monthly,
        accrual_amount: 1.0,
        accrual_timing: "start_of_period".to_string(),
        seniority_bonus: vec![
            SeniorityBonus {
                months: 12,
                bonus_amount: 0.5,
            },
            SeniorityBonus {
                months: 60,
                bonus_amount: 1.0,
            },
        ],
        carry_over_limit: Some(5),
        max_balance: None,
        reset_month: None,
        min_work_days_threshold: Some(14.0),
    };

    // 0 months service, enough work days
    assert!(
        (LeaveAccrualService::calculate_accrual_amount(&config, 0, 15.0) - 1.0).abs()
            < f64::EPSILON
    );

    // 0 months service, NOT enough work days
    assert!(
        (LeaveAccrualService::calculate_accrual_amount(&config, 0, 10.0) - 0.0).abs()
            < f64::EPSILON
    );

    // 12 months service milestone
    assert!(
        (LeaveAccrualService::calculate_accrual_amount(&config, 12, 15.0) - 1.5).abs()
            < f64::EPSILON
    );
}

#[test]
fn test_carry_over_limit_logic() {
    assert!((LeaveAccrualService::calculate_carry_over_amount(10.0, 5) - 5.0).abs() < f64::EPSILON);
    assert!((LeaveAccrualService::calculate_carry_over_amount(3.0, 5) - 3.0).abs() < f64::EPSILON);
    assert!((LeaveAccrualService::calculate_carry_over_amount(0.0, 5) - 0.0).abs() < f64::EPSILON);
}

// -------------------------------------------------------------------------
// should_run_accrual
// -------------------------------------------------------------------------

#[test]
fn test_should_run_accrual_monthly_start() {
    // Monthly start_of_period → day == 1
    assert!(LeaveAccrualService::should_run_accrual(
        &AccrualFrequency::Monthly,
        "start_of_period",
        nd(2026, 3, 1)
    ));
    assert!(!LeaveAccrualService::should_run_accrual(
        &AccrualFrequency::Monthly,
        "start_of_period",
        nd(2026, 3, 15)
    ));
}

#[test]
fn test_should_run_accrual_monthly_end() {
    // Monthly end_of_period → last day of month (next day is a different month)
    assert!(LeaveAccrualService::should_run_accrual(
        &AccrualFrequency::Monthly,
        "end_of_period",
        nd(2026, 3, 31) // last day of March
    ));
    assert!(!LeaveAccrualService::should_run_accrual(
        &AccrualFrequency::Monthly,
        "end_of_period",
        nd(2026, 3, 30)
    ));
    // February non-leap year
    assert!(LeaveAccrualService::should_run_accrual(
        &AccrualFrequency::Monthly,
        "end_of_period",
        nd(2026, 2, 28)
    ));
}

#[test]
fn test_should_run_accrual_quarterly_start() {
    // Quarter starts: Jan 1, Apr 1, Jul 1, Oct 1
    for &(m, d, expected) in &[
        (1u32, 1u32, true),
        (4, 1, true),
        (7, 1, true),
        (10, 1, true),
        (2, 1, false),
        (1, 5, false),
    ] {
        assert_eq!(
            LeaveAccrualService::should_run_accrual(
                &AccrualFrequency::Quarterly,
                "start_of_period",
                nd(2026, m, d)
            ),
            expected,
            "Quarterly start m={m} d={d}"
        );
    }
}

#[test]
fn test_should_run_accrual_yearly() {
    // Yearly start → Jan 1
    assert!(LeaveAccrualService::should_run_accrual(
        &AccrualFrequency::Yearly,
        "start_of_period",
        nd(2026, 1, 1)
    ));
    assert!(!LeaveAccrualService::should_run_accrual(
        &AccrualFrequency::Yearly,
        "start_of_period",
        nd(2026, 6, 1)
    ));
    // Yearly end → Dec 31
    assert!(LeaveAccrualService::should_run_accrual(
        &AccrualFrequency::Yearly,
        "end_of_period",
        nd(2026, 12, 31)
    ));
    assert!(!LeaveAccrualService::should_run_accrual(
        &AccrualFrequency::Yearly,
        "end_of_period",
        nd(2026, 12, 30)
    ));
}

// -------------------------------------------------------------------------
// calculate_total_months_seniority
// -------------------------------------------------------------------------

#[test]
fn test_seniority_single_period_no_termination() {
    // Hired 2025-01-01, as_of 2026-01-01 → ~12 months (365 days / 30 ≈ 12)
    let periods = vec![mock_contract(nd(2025, 1, 1), None)];
    let months = LeaveAccrualService::calculate_total_months_seniority(&periods, nd(2026, 1, 1));
    // 366 days (2025 not a leap year but 2025-01-01 to 2026-01-01 = 365 days +1 = 366 / 30 = 12)
    assert_eq!(months, 12);
}

#[test]
fn test_seniority_terminated_before_as_of() {
    // Hired 2024-01-01, terminated 2024-06-30 → 181 days / 30 = 6 months
    let periods = vec![mock_contract(nd(2024, 1, 1), Some(nd(2024, 6, 30)))];
    let months = LeaveAccrualService::calculate_total_months_seniority(&periods, nd(2026, 1, 1));
    assert_eq!(months, 6);
}

#[test]
fn test_seniority_future_hire_date_excluded() {
    // Hired in the future → 0 months
    let periods = vec![mock_contract(nd(2030, 1, 1), None)];
    let months = LeaveAccrualService::calculate_total_months_seniority(&periods, nd(2026, 1, 1));
    assert_eq!(months, 0);
}

#[test]
fn test_seniority_rehire_gap_accumulates() {
    // Period 1: 2020-01-01 → 2021-01-01 (12 months)
    // Period 2: 2022-01-01 → unspecified, as_of = 2022-07-01 (6 months)
    // Total ≈ 18 months
    let periods = vec![
        mock_contract(nd(2020, 1, 1), Some(nd(2021, 1, 1))),
        mock_contract(nd(2022, 1, 1), None),
    ];
    let months = LeaveAccrualService::calculate_total_months_seniority(&periods, nd(2022, 7, 1));
    // Phase 1: 366+1 = 367 days ≈ 12 months
    // Phase 2: 181 days ≈ 6 months
    assert!(
        (17..=18).contains(&months),
        "Expected 17-18 months, got {months}"
    );
}

// -------------------------------------------------------------------------
// get_accrual_dates
// -------------------------------------------------------------------------

#[test]
fn test_get_accrual_dates_monthly_start() {
    // Monthly start_of_period: 1st of each month
    let dates = LeaveAccrualService::get_accrual_dates(
        &AccrualFrequency::Monthly,
        "start_of_period",
        nd(2024, 1, 1),
        nd(2026, 2, 28),
    );
    // Jan 2024 through Feb 2026 = 26 months
    assert_eq!(dates.len(), 26);
    assert_eq!(dates[0], nd(2024, 1, 1));
    assert_eq!(dates[25], nd(2026, 2, 1));
}

#[test]
fn test_get_accrual_dates_monthly_start_mid_month_from() {
    // When from is mid-month, skip that month's 1st (it's before from)
    let dates = LeaveAccrualService::get_accrual_dates(
        &AccrualFrequency::Monthly,
        "start_of_period",
        nd(2026, 1, 15),
        nd(2026, 3, 31),
    );
    assert_eq!(dates.len(), 2); // Feb 1, Mar 1
    assert_eq!(dates[0], nd(2026, 2, 1));
    assert_eq!(dates[1], nd(2026, 3, 1));
}

#[test]
fn test_get_accrual_dates_monthly_end() {
    let dates = LeaveAccrualService::get_accrual_dates(
        &AccrualFrequency::Monthly,
        "end_of_period",
        nd(2026, 1, 1),
        nd(2026, 3, 31),
    );
    assert_eq!(dates.len(), 3);
    assert_eq!(dates[0], nd(2026, 1, 31));
    assert_eq!(dates[1], nd(2026, 2, 28)); // 2026 is not leap year
    assert_eq!(dates[2], nd(2026, 3, 31));
}

#[test]
fn test_get_accrual_dates_quarterly_start() {
    let dates = LeaveAccrualService::get_accrual_dates(
        &AccrualFrequency::Quarterly,
        "start_of_period",
        nd(2025, 1, 1),
        nd(2026, 6, 30),
    );
    // Q1-Q4 2025 + Q1-Q2 2026 = 6
    assert_eq!(dates.len(), 6);
    assert_eq!(dates[0], nd(2025, 1, 1));
    assert_eq!(dates[3], nd(2025, 10, 1));
    assert_eq!(dates[5], nd(2026, 4, 1));
}

#[test]
fn test_get_accrual_dates_yearly_start() {
    let dates = LeaveAccrualService::get_accrual_dates(
        &AccrualFrequency::Yearly,
        "start_of_period",
        nd(2024, 1, 1),
        nd(2026, 6, 15),
    );
    assert_eq!(dates.len(), 3);
    assert_eq!(dates[0], nd(2024, 1, 1));
    assert_eq!(dates[1], nd(2025, 1, 1));
    assert_eq!(dates[2], nd(2026, 1, 1));
}

#[test]
fn test_get_accrual_dates_yearly_end() {
    let dates = LeaveAccrualService::get_accrual_dates(
        &AccrualFrequency::Yearly,
        "end_of_period",
        nd(2024, 1, 1),
        nd(2026, 6, 15),
    );
    assert_eq!(dates.len(), 2); // Dec 31 2024, Dec 31 2025
    assert_eq!(dates[0], nd(2024, 12, 31));
    assert_eq!(dates[1], nd(2025, 12, 31));
}

#[test]
fn test_get_accrual_dates_empty_range() {
    // To < first possible accrual date
    let dates = LeaveAccrualService::get_accrual_dates(
        &AccrualFrequency::Monthly,
        "start_of_period",
        nd(2026, 3, 15),
        nd(2026, 3, 20),
    );
    assert!(dates.is_empty());
}

// =====================================================================
// Additional edge case tests
// =====================================================================

#[test]
fn test_carry_over_negative_remaining() {
    assert!(
        (LeaveAccrualService::calculate_carry_over_amount(-5.0, 10) - 0.0).abs() < f64::EPSILON
    );
}

#[test]
fn test_carry_over_zero_limit() {
    assert!((LeaveAccrualService::calculate_carry_over_amount(10.0, 0) - 0.0).abs() < f64::EPSILON);
}

#[test]
fn test_carry_over_exact_limit() {
    assert!((LeaveAccrualService::calculate_carry_over_amount(5.0, 5) - 5.0).abs() < f64::EPSILON);
}

#[test]
fn test_seniority_empty_periods() {
    let months = LeaveAccrualService::calculate_total_months_seniority(&[], nd(2026, 1, 1));
    assert_eq!(months, 0);
}

#[test]
fn test_seniority_same_day_hire_and_as_of() {
    // Hired today, as_of today → 1 day / 30 = 0 months
    let periods = vec![mock_contract(nd(2026, 1, 1), None)];
    let months = LeaveAccrualService::calculate_total_months_seniority(&periods, nd(2026, 1, 1));
    assert_eq!(months, 0); // 1 day / 30 = 0
}

#[test]
fn test_accrual_amount_no_threshold() {
    let config = LeavePolicyConfig {
        frequency: AccrualFrequency::Monthly,
        accrual_amount: 1.5,
        accrual_timing: "start_of_period".to_string(),
        seniority_bonus: vec![],
        carry_over_limit: None,
        max_balance: None,
        reset_month: None,
        min_work_days_threshold: None,
    };

    // No threshold → work_days_in_period doesn't matter
    assert!(
        (LeaveAccrualService::calculate_accrual_amount(&config, 0, 0.0) - 1.5).abs() < f64::EPSILON
    );
}

#[test]
fn test_accrual_amount_highest_seniority_wins() {
    let config = LeavePolicyConfig {
        frequency: AccrualFrequency::Monthly,
        accrual_amount: 1.0,
        accrual_timing: "start_of_period".to_string(),
        seniority_bonus: vec![
            SeniorityBonus {
                months: 6,
                bonus_amount: 0.25,
            },
            SeniorityBonus {
                months: 12,
                bonus_amount: 0.5,
            },
            SeniorityBonus {
                months: 24,
                bonus_amount: 1.0,
            },
        ],
        carry_over_limit: None,
        max_balance: None,
        reset_month: None,
        min_work_days_threshold: None,
    };

    // 36 months > all milestones → highest (24 months → 1.0 bonus)
    assert!(
        (LeaveAccrualService::calculate_accrual_amount(&config, 36, 20.0) - 2.0).abs()
            < f64::EPSILON
    );

    // 18 months → 12 months milestone (0.5 bonus)
    assert!(
        (LeaveAccrualService::calculate_accrual_amount(&config, 18, 20.0) - 1.5).abs()
            < f64::EPSILON
    );

    // 3 months → no milestone
    assert!(
        (LeaveAccrualService::calculate_accrual_amount(&config, 3, 20.0) - 1.0).abs()
            < f64::EPSILON
    );
}

#[test]
fn test_get_accrual_dates_quarterly_end() {
    let dates = LeaveAccrualService::get_accrual_dates(
        &AccrualFrequency::Quarterly,
        "end_of_period",
        nd(2026, 1, 1),
        nd(2026, 12, 31),
    );
    assert_eq!(dates.len(), 4);
    assert_eq!(dates[0], nd(2026, 3, 31));
    assert_eq!(dates[1], nd(2026, 6, 30));
    assert_eq!(dates[2], nd(2026, 9, 30));
    assert_eq!(dates[3], nd(2026, 12, 31));
}

#[tokio::test]
async fn test_process_workspace_accruals_empty_employees() {
    use crate::repositories::attendance_daily_summary_repository::MockAttendanceDailySummaryRepository;
    use crate::repositories::department_repository::MockDepartmentRepository;
    use crate::repositories::employee::MockEmployeeRepository;
    use crate::repositories::employee_contract_repository::MockEmployeeContractRepository;
    use crate::repositories::leave_accrual_log_repository::MockLeaveAccrualLogRepository;
    use crate::repositories::leave_policy_repository::MockLeavePolicyRepository;
    use crate::repositories::leave_request_repository::MockLeaveRequestRepository;
    use crate::repositories::time_entry_repository::MockTimeEntryRepository;
    use std::sync::Arc;

    let mut mock_emp_repo = MockEmployeeRepository::new();
    let mut mock_accrual_repo = MockLeaveAccrualLogRepository::new();

    mock_accrual_repo
        .expect_get_workspace_timezone()
        .returning(|_| Ok("UTC".to_string()));

    mock_emp_repo
        .expect_get_active_employees()
        .returning(|_| Ok(vec![]));

    let service = LeaveAccrualService {
        employee_repo: Arc::new(mock_emp_repo),
        department_repo: Arc::new(MockDepartmentRepository::new()),
        leave_policy_repo: Arc::new(MockLeavePolicyRepository::new()),
        accrual_log_repo: Arc::new(mock_accrual_repo),
        time_entry_repo: Arc::new(MockTimeEntryRepository::new()),
        leave_request_repo: Arc::new(MockLeaveRequestRepository::new()),
        contract_repo: Arc::new(MockEmployeeContractRepository::new()),
        daily_summary_repo: Arc::new(MockAttendanceDailySummaryRepository::new()),
    };

    let result = service
        .process_workspace_accruals(Uuid::new_v4(), nd(2026, 1, 1), false, None)
        .await
        .unwrap();

    assert!(result.is_empty());
}

#[tokio::test]
async fn test_process_workspace_accruals_with_target_employees() {
    use crate::repositories::attendance_daily_summary_repository::MockAttendanceDailySummaryRepository;
    use crate::repositories::department_repository::MockDepartmentRepository;
    use crate::repositories::employee::MockEmployeeRepository;
    use crate::repositories::employee_contract_repository::MockEmployeeContractRepository;
    use crate::repositories::leave_accrual_log_repository::MockLeaveAccrualLogRepository;
    use crate::repositories::leave_policy_repository::MockLeavePolicyRepository;
    use crate::repositories::leave_request_repository::MockLeaveRequestRepository;
    use crate::repositories::time_entry_repository::MockTimeEntryRepository;
    use std::sync::Arc;

    let mut mock_emp_repo = MockEmployeeRepository::new();
    let mut mock_accrual_repo = MockLeaveAccrualLogRepository::new();

    mock_accrual_repo
        .expect_get_workspace_timezone()
        .returning(|_| Ok("UTC".to_string()));

    let target_emp_id = Uuid::new_v4();

    mock_emp_repo
        .expect_get_employees_by_ids()
        .withf(move |ids, _| ids == [target_emp_id])
        .times(1)
        .returning(|_, _| Ok(vec![]));

    let service = LeaveAccrualService {
        employee_repo: Arc::new(mock_emp_repo),
        department_repo: Arc::new(MockDepartmentRepository::new()),
        leave_policy_repo: Arc::new(MockLeavePolicyRepository::new()),
        accrual_log_repo: Arc::new(mock_accrual_repo),
        time_entry_repo: Arc::new(MockTimeEntryRepository::new()),
        leave_request_repo: Arc::new(MockLeaveRequestRepository::new()),
        contract_repo: Arc::new(MockEmployeeContractRepository::new()),
        daily_summary_repo: Arc::new(MockAttendanceDailySummaryRepository::new()),
    };

    let result = service
        .process_workspace_accruals(
            Uuid::new_v4(),
            nd(2026, 1, 1),
            false,
            Some(&[target_emp_id]),
        )
        .await
        .unwrap();

    assert!(result.is_empty());
}

#[tokio::test]
async fn test_count_qualifying_days_success() {
    use crate::repositories::attendance_daily_summary_repository::MockAttendanceDailySummaryRepository;
    use crate::repositories::department_repository::MockDepartmentRepository;
    use crate::repositories::employee::MockEmployeeRepository;
    use crate::repositories::employee_contract_repository::MockEmployeeContractRepository;
    use crate::repositories::leave_accrual_log_repository::MockLeaveAccrualLogRepository;
    use crate::repositories::leave_policy_repository::MockLeavePolicyRepository;
    use crate::repositories::leave_request_repository::MockLeaveRequestRepository;
    use crate::repositories::time_entry_repository::MockTimeEntryRepository;
    use std::sync::Arc;

    let mut mock_time_entry = MockTimeEntryRepository::new();
    let mut mock_leave_req = MockLeaveRequestRepository::new();

    mock_time_entry
        .expect_get_distinct_work_days()
        .returning(|_, _, _, _, _| Ok(vec![nd(2026, 1, 15), nd(2026, 1, 16)]));

    mock_leave_req
        .expect_get_approved_leave_periods()
        .returning(|_, _, _, _| {
            Box::pin(std::future::ready(Ok(vec![(
                nd(2026, 1, 20),
                nd(2026, 1, 21),
            )])))
        });

    let service = LeaveAccrualService {
        employee_repo: Arc::new(MockEmployeeRepository::new()),
        department_repo: Arc::new(MockDepartmentRepository::new()),
        leave_policy_repo: Arc::new(MockLeavePolicyRepository::new()),
        accrual_log_repo: Arc::new(MockLeaveAccrualLogRepository::new()),
        time_entry_repo: Arc::new(mock_time_entry),
        leave_request_repo: Arc::new(mock_leave_req),
        contract_repo: Arc::new(MockEmployeeContractRepository::new()),
        daily_summary_repo: Arc::new(MockAttendanceDailySummaryRepository::new()),
    };

    let result = service
        .count_qualifying_days(
            Uuid::new_v4(),
            Uuid::new_v4(),
            nd(2026, 1, 1),
            nd(2026, 1, 31),
            "UTC".parse().unwrap(),
        )
        .await
        .unwrap();

    // 2 work days + 2 leave days = 4 qualifying days
    assert_eq!(result, 4);
}

#[tokio::test]
async fn test_batch_count_qualifying_days_success() {
    use crate::repositories::attendance_daily_summary_repository::MockAttendanceDailySummaryRepository;
    use crate::repositories::department_repository::MockDepartmentRepository;
    use crate::repositories::employee::MockEmployeeRepository;
    use crate::repositories::employee_contract_repository::MockEmployeeContractRepository;
    use crate::repositories::leave_accrual_log_repository::MockLeaveAccrualLogRepository;
    use crate::repositories::leave_policy_repository::MockLeavePolicyRepository;
    use crate::repositories::leave_request_repository::MockLeaveRequestRepository;
    use crate::repositories::time_entry_repository::MockTimeEntryRepository;
    use std::sync::Arc;

    let mut mock_time_entry = MockTimeEntryRepository::new();
    let mut mock_leave_req = MockLeaveRequestRepository::new();

    mock_time_entry
        .expect_get_distinct_work_days()
        .returning(|_, _, _, _, _| Ok(vec![nd(2026, 1, 15), nd(2026, 2, 10)]));

    mock_leave_req
        .expect_get_approved_leave_periods()
        .returning(|_, _, _, _| {
            Box::pin(std::future::ready(Ok(vec![
                (nd(2026, 1, 20), nd(2026, 1, 21)),
                (nd(2026, 3, 5), nd(2026, 3, 5)),
            ])))
        });

    let service = LeaveAccrualService {
        employee_repo: Arc::new(MockEmployeeRepository::new()),
        department_repo: Arc::new(MockDepartmentRepository::new()),
        leave_policy_repo: Arc::new(MockLeavePolicyRepository::new()),
        accrual_log_repo: Arc::new(MockLeaveAccrualLogRepository::new()),
        time_entry_repo: Arc::new(mock_time_entry),
        leave_request_repo: Arc::new(mock_leave_req),
        contract_repo: Arc::new(MockEmployeeContractRepository::new()),
        daily_summary_repo: Arc::new(MockAttendanceDailySummaryRepository::new()),
    };

    let result = service
        .batch_count_qualifying_days(
            Uuid::new_v4(),
            Uuid::new_v4(),
            nd(2026, 1, 1),
            nd(2026, 3, 31),
            "UTC".parse().unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(result.get(&nd(2026, 1, 1)), Some(&3)); // 1 work day + 2 leave days
    assert_eq!(result.get(&nd(2026, 2, 1)), Some(&1)); // 1 work day
    assert_eq!(result.get(&nd(2026, 3, 1)), Some(&1)); // 1 leave day
}

#[tokio::test]
async fn test_batch_count_qualifying_days_for_workspace_success() {
    use crate::repositories::attendance_daily_summary_repository::MockAttendanceDailySummaryRepository;
    use crate::repositories::department_repository::MockDepartmentRepository;
    use crate::repositories::employee::MockEmployeeRepository;
    use crate::repositories::employee_contract_repository::MockEmployeeContractRepository;
    use crate::repositories::leave_accrual_log_repository::MockLeaveAccrualLogRepository;
    use crate::repositories::leave_policy_repository::MockLeavePolicyRepository;
    use crate::repositories::leave_request_repository::MockLeaveRequestRepository;
    use crate::repositories::time_entry_repository::MockTimeEntryRepository;
    use std::sync::Arc;

    let mut mock_time_entry = MockTimeEntryRepository::new();
    let mut mock_leave_req = MockLeaveRequestRepository::new();

    let emp1 = Uuid::new_v4();
    let emp2 = Uuid::new_v4();

    use chrono::TimeZone;
    mock_time_entry
        .expect_get_workspace_time_entries_batch_by_employments()
        .returning(move |_, _, _, _| {
            let mut map = std::collections::HashMap::new();
            map.insert(
                emp1,
                vec![crate::models::TimeEntry::new(NewTimeEntryParams {
                    id: Uuid::new_v4(),
                    employment_id: emp1,
                    workspace_id: Uuid::new_v4(),
                    shift_id: None,
                    segment_id: None,
                    punch_in_time: chrono::Utc.with_ymd_and_hms(2026, 1, 15, 8, 0, 0).unwrap(),
                    punch_out_time: None,
                    duration_minutes: None,
                    is_break_overtime: false,
                    entry_type: crate::services::pairing::types::EntryType::Work,
                    note: None,
                    generation_metadata: None,
                    created_at: chrono::Utc::now(),
                    updated_at: chrono::Utc::now(),
                })],
            );
            Ok(map)
        });

    mock_leave_req
        .expect_get_batch_leave_days()
        .returning(|_, _, _, _| Box::pin(std::future::ready(Ok(std::collections::HashMap::new()))));

    mock_leave_req
        .expect_get_approved_leave_dates()
        .returning(move |_, _, _, _| {
            Box::pin(std::future::ready(Ok(
                std::collections::HashSet::from_iter(vec![(emp2, nd(2026, 1, 20))]),
            )))
        });

    let mut mock_employee_repo = MockEmployeeRepository::new();
    mock_employee_repo
        .expect_get_employees_by_ids()
        .returning(move |ids, _| {
            let mut employees = Vec::new();
            for &id in ids {
                employees.push(
                    crate::models::Employee::new(
                        id,
                        Uuid::new_v4(),
                        None,
                        "John Doe".to_string(),
                        "EMP001".to_string(),
                        vec![],
                        chrono::Utc::now(),
                        chrono::Utc::now(),
                        None,
                    )
                    .with_employment_info(Some(id), None, None),
                );
            }
            Ok(employees)
        });

    let service = LeaveAccrualService {
        employee_repo: Arc::new(mock_employee_repo),
        department_repo: Arc::new(MockDepartmentRepository::new()),
        leave_policy_repo: Arc::new(MockLeavePolicyRepository::new()),
        accrual_log_repo: Arc::new(MockLeaveAccrualLogRepository::new()),
        time_entry_repo: Arc::new(mock_time_entry),
        leave_request_repo: Arc::new(mock_leave_req),
        contract_repo: Arc::new(MockEmployeeContractRepository::new()),
        daily_summary_repo: Arc::new(MockAttendanceDailySummaryRepository::new()),
    };

    let result = service
        .batch_count_qualifying_days_for_workspace(
            Uuid::new_v4(),
            &[emp1, emp2],
            nd(2026, 1, 1),
            nd(2026, 1, 31),
            "UTC".parse().unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(result.get(&emp1).unwrap().get(&nd(2026, 1, 1)), Some(&1));
    assert_eq!(result.get(&emp2).unwrap().get(&nd(2026, 1, 1)), Some(&1));
}
