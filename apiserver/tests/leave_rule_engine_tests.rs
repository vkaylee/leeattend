use apiserver::services::leave_accrual_service::LeaveAccrualService;
use apiserver::models::{AccrualFrequency, LeavePolicyConfig, SeniorityBonus};

#[test]
fn test_accrual_with_work_days_threshold() {
    let config = LeavePolicyConfig {
        frequency: AccrualFrequency::Monthly,
        accrual_amount: 1.0,
        accrual_timing: "end_of_period".to_string(),
        carry_over_limit: Some(5),
        max_balance: None,
        reset_month: None,
        min_work_days_threshold: Some(14.0),
        seniority_bonus: vec![],
    };

    // Case 1: Work 15 days (>= 14) -> Should get 1.0
    let amount = LeaveAccrualService::calculate_accrual_amount(&config, 0, 15.0);
    assert_eq!(amount, 1.0);

    // Case 2: Work 10 days (< 14) -> Should get 0.0
    let amount = LeaveAccrualService::calculate_accrual_amount(&config, 0, 10.0);
    assert_eq!(amount, 0.0);
}

#[test]
fn test_accrual_with_seniority_milestones() {
    let config = LeavePolicyConfig {
        frequency: AccrualFrequency::Monthly,
        accrual_amount: 1.0,
        accrual_timing: "end_of_period".to_string(),
        carry_over_limit: Some(5),
        max_balance: None,
        reset_month: None,
        min_work_days_threshold: None,
        seniority_bonus: vec![
            SeniorityBonus { months: 3, bonus_amount: 1.0 },
            SeniorityBonus { months: 6, bonus_amount: 1.0 },
            SeniorityBonus { months: 12, bonus_amount: 2.0 }, // 1 (quarterly) + 1 (annual)
        ],
    };

    // Month 1 & 2: Just base amount
    assert_eq!(LeaveAccrualService::calculate_accrual_amount(&config, 1, 20.0), 1.0);
    assert_eq!(LeaveAccrualService::calculate_accrual_amount(&config, 2, 20.0), 1.0);

    // Month 3: Base (1.0) + Bonus (1.0) = 2.0
    assert_eq!(LeaveAccrualService::calculate_accrual_amount(&config, 3, 20.0), 2.0);

    // Month 12: Base (1.0) + Bonus (2.0) = 3.0
    assert_eq!(LeaveAccrualService::calculate_accrual_amount(&config, 12, 20.0), 3.0);
}
