use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct SavePayrollSettings {
    pub std_working_day_mode: String,
    pub fixed_std_days: rust_decimal::Decimal,
    pub working_days_per_week: i32,
    pub hours_per_day: rust_decimal::Decimal,
    pub max_ot_hours_per_day: Option<rust_decimal::Decimal>,
    pub max_ot_hours_per_month: Option<rust_decimal::Decimal>,
    pub effective_date: NaiveDate,
}

#[derive(Debug, Clone)]
pub struct CreatePayslipSegmentArgs {
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
    pub department_id: Option<Uuid>,
    pub applied_base_salary: rust_decimal::Decimal,
    pub applied_std_days: rust_decimal::Decimal,
    pub hours_per_day: rust_decimal::Decimal,
    pub actual_work_days: rust_decimal::Decimal,
    pub paid_work_days: rust_decimal::Decimal,
    pub amount: rust_decimal::Decimal,
    pub std_days_source: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow, Clone)]
pub struct PayslipSegment {
    pub id: Uuid,
    pub payslip_id: Uuid,
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
    pub department_id: Option<Uuid>,
    pub applied_base_salary: rust_decimal::Decimal,
    pub applied_std_days: rust_decimal::Decimal,
    pub applied_hours_per_day: rust_decimal::Decimal,
    pub actual_work_days: rust_decimal::Decimal,
    pub paid_work_days: rust_decimal::Decimal,
    pub amount: rust_decimal::Decimal,
    pub reason: Option<String>,
    pub std_days_source: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow, Clone)]
pub struct Payslip {
    pub id: Uuid,
    pub employee_id: Uuid,
    pub workspace_id: Uuid,
    pub period_start: NaiveDate,
    pub period_end: NaiveDate,
    pub base_pay_amount: rust_decimal::Decimal,
    pub overtime_pay_amount: rust_decimal::Decimal,
    pub deductions_amount: rust_decimal::Decimal,
    pub net_pay_amount: rust_decimal::Decimal,
    pub standard_units: i32,
    pub overtime_units: i32,
    pub status: String,
    /// Premium paid when employee works on a holiday (configurable per-holiday via ot_multiplier).
    pub holiday_work_premium_amount: rust_decimal::Decimal,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow, Clone)]
pub struct PayrollIncentivePolicy {
    pub id: Uuid,
    pub workspace_id: Uuid,
    pub name: String,
    pub base_amount: f64,
    pub payout_type: String, // String to match DB, can be parsed to PayoutType
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow, Clone)]
pub struct IncentiveTier {
    pub id: Uuid,
    pub policy_id: Uuid,
    pub priority: i32,
    pub max_late_early_count: Option<i32>,
    pub max_unpaid_leave_days: Option<f64>,
    pub max_sick_leave_days: Option<f64>,
    pub max_total_leave_days: Option<f64>,
    pub payout_amount: f64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct IncentiveEvaluationResult {
    pub policy_name: String,
    pub amount: rust_decimal::Decimal,
    pub standard_amount: rust_decimal::Decimal,
    pub tier_id: Option<Uuid>,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow, Clone)]
pub struct PayslipRuleEvaluation {
    pub id: Uuid,
    pub payslip_id: Uuid,
    pub rule_code: String,
    pub amount: rust_decimal::Decimal,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CreateIncentiveTierRequest {
    pub priority: i32,
    pub max_late_early_count: Option<i32>,
    pub max_unpaid_leave_days: Option<f64>,
    pub max_sick_leave_days: Option<f64>,
    pub max_total_leave_days: Option<f64>,
    pub payout_amount: f64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CreatePayrollIncentivePolicyRequest {
    pub name: String,
    pub base_amount: f64,
    pub payout_type: String,
    pub tiers: Vec<CreateIncentiveTierRequest>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PayrollIncentivePolicyResponse {
    #[serde(flatten)]
    pub policy: PayrollIncentivePolicy,
    pub tiers: Vec<IncentiveTier>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PayslipResponse {
    #[serde(flatten)]
    pub payslip: Payslip,
    pub segments: Vec<PayslipSegment>,
    pub incentives: Vec<IncentiveEvaluationResult>,
    pub rule_evaluations: Vec<PayslipRuleEvaluation>,
    pub salary_components: Vec<SalaryComponentWithDefinition>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BatchCalculatePayrollRequest {
    pub employee_ids: Vec<Uuid>,
    pub year: i32,
    pub month: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BatchCalculateResult {
    pub employee_id: Uuid,
    pub status: String, // "success" or "error"
    pub payslip: Option<Payslip>,
    pub error_message: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BatchCalculatePayrollResponse {
    pub results: Vec<BatchCalculateResult>,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow, Clone)]
pub struct PayrollSettings {
    pub workspace_id: Uuid,
    pub std_working_day_mode: String,
    pub fixed_std_days: rust_decimal::Decimal,
    pub working_days_per_week: i32,
    pub hours_per_day: rust_decimal::Decimal,
    pub max_ot_hours_per_day: Option<rust_decimal::Decimal>,
    pub max_ot_hours_per_month: Option<rust_decimal::Decimal>,
    pub effective_date: NaiveDate,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow, Clone)]
pub struct DepartmentPayrollConfig {
    pub id: Uuid,
    pub department_id: Uuid,
    pub std_working_days: rust_decimal::Decimal,
    pub hours_per_day: rust_decimal::Decimal,
    pub effective_date: NaiveDate,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow, Clone)]
pub struct EmployeePayrollConfig {
    pub id: Uuid,
    pub employee_id: Uuid,
    pub std_working_days: rust_decimal::Decimal,
    pub hours_per_day: rust_decimal::Decimal,
    pub effective_date: NaiveDate,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SalaryComponentDefinition {
    pub id: Uuid,
    pub workspace_id: Uuid,
    pub name: String,
    pub code: String,
    pub is_base_for_ot: bool,
    pub is_taxable: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EmployeeSalaryComponent {
    pub id: Uuid,
    pub employee_id: Uuid,
    pub component_definition_id: Uuid,
    pub amount: rust_decimal::Decimal,
    pub effective_date: NaiveDate,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct UpdatePayrollSettingsRequest {
    pub std_working_day_mode: Option<String>,
    pub fixed_std_days: Option<rust_decimal::Decimal>,
    pub working_days_per_week: Option<i32>,
    pub hours_per_day: Option<rust_decimal::Decimal>,
    pub max_ot_hours_per_day: Option<Option<rust_decimal::Decimal>>,
    pub max_ot_hours_per_month: Option<Option<rust_decimal::Decimal>>,
    pub effective_date: Option<NaiveDate>,
}

#[derive(Debug, Deserialize)]
pub struct SetDepartmentPayrollConfigRequest {
    pub std_working_days: rust_decimal::Decimal,
    pub hours_per_day: rust_decimal::Decimal,
    pub effective_date: NaiveDate,
}

#[derive(Debug, Deserialize)]
pub struct SetEmployeePayrollConfigRequest {
    pub std_working_days: rust_decimal::Decimal,
    pub hours_per_day: rust_decimal::Decimal,
    pub effective_date: NaiveDate,
}

#[derive(Debug, Deserialize)]
pub struct CreateSalaryComponentRequest {
    pub name: String,
    pub code: String,
    pub is_base_for_ot: bool,
    pub is_taxable: bool,
}

#[derive(Debug, Deserialize)]
pub struct AssignSalaryComponentRequest {
    pub component_definition_id: Uuid,
    pub amount: rust_decimal::Decimal,
    pub effective_date: NaiveDate,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SalaryComponentWithDefinition {
    pub component: EmployeeSalaryComponent,
    pub definition: SalaryComponentDefinition,
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal::Decimal;

    #[test]
    fn test_create_salary_component_request_serde() {
        let json = r#"{
            "name": "Housing Allowance",
            "code": "HOUSE",
            "is_base_for_ot": false,
            "is_taxable": true
        }"#;
        let req: CreateSalaryComponentRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.name, "Housing Allowance");
        assert_eq!(req.code, "HOUSE");
        assert!(!req.is_base_for_ot);
        assert!(req.is_taxable);
    }

    #[test]
    fn test_assign_salary_component_request_serde() {
        let json = r#"{
            "component_definition_id": "00000000-0000-0000-0000-000000000001",
            "amount": "5000.50",
            "effective_date": "2026-01-01"
        }"#;
        let req: AssignSalaryComponentRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.amount, Decimal::new(500050, 2));
        assert_eq!(
            req.effective_date,
            NaiveDate::from_ymd_opt(2026, 1, 1).unwrap()
        );
    }

    #[test]
    fn test_batch_calculate_payroll_request_serde() {
        let json = r#"{
            "employee_ids": [
                "00000000-0000-0000-0000-000000000001",
                "00000000-0000-0000-0000-000000000002"
            ],
            "year": 2026,
            "month": 3
        }"#;
        let req: BatchCalculatePayrollRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.employee_ids.len(), 2);
        assert_eq!(req.year, 2026);
        assert_eq!(req.month, 3);
    }

    #[test]
    fn test_batch_calculate_result_success() {
        let result = BatchCalculateResult {
            employee_id: Uuid::new_v4(),
            status: "success".to_string(),
            payslip: None,
            error_message: None,
        };
        let json = serde_json::to_string(&result).unwrap();
        let back: BatchCalculateResult = serde_json::from_str(&json).unwrap();
        assert_eq!(back.status, "success");
        assert!(back.payslip.is_none());
        assert!(back.error_message.is_none());
    }

    #[test]
    fn test_batch_calculate_result_error() {
        let result = BatchCalculateResult {
            employee_id: Uuid::new_v4(),
            status: "error".to_string(),
            payslip: None,
            error_message: Some("No salary data found".to_string()),
        };
        let json = serde_json::to_string(&result).unwrap();
        let back: BatchCalculateResult = serde_json::from_str(&json).unwrap();
        assert_eq!(back.status, "error");
        assert_eq!(back.error_message.as_deref(), Some("No salary data found"));
    }

    #[test]
    fn test_set_department_payroll_config_request_serde() {
        let json = r#"{
            "std_working_days": "22",
            "hours_per_day": "8",
            "effective_date": "2026-01-01"
        }"#;
        let req: SetDepartmentPayrollConfigRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.std_working_days, Decimal::from(22));
        assert_eq!(req.hours_per_day, Decimal::from(8));
    }

    #[test]
    fn test_set_employee_payroll_config_request_serde() {
        let json = r#"{
            "std_working_days": "26",
            "hours_per_day": "7.5",
            "effective_date": "2026-06-01"
        }"#;
        let req: SetEmployeePayrollConfigRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.std_working_days, Decimal::from(26));
    }

    #[test]
    fn test_update_payroll_settings_empty() {
        let json = r#"{}"#;
        let req: UpdatePayrollSettingsRequest = serde_json::from_str(json).unwrap();
        assert!(req.std_working_day_mode.is_none());
        assert!(req.fixed_std_days.is_none());
        assert!(req.working_days_per_week.is_none());
        assert!(req.hours_per_day.is_none());
        assert!(req.max_ot_hours_per_day.is_none());
        assert!(req.max_ot_hours_per_month.is_none());
    }

    #[test]
    fn test_update_payroll_settings_with_values() {
        let json = r#"{
            "std_working_day_mode": "fixed",
            "fixed_std_days": "26",
            "working_days_per_week": 5,
            "hours_per_day": "8"
        }"#;
        let req: UpdatePayrollSettingsRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.std_working_day_mode.as_deref(), Some("fixed"));
        assert_eq!(req.fixed_std_days, Some(Decimal::from(26)));
        assert_eq!(req.working_days_per_week, Some(5));
    }

    #[test]
    fn test_create_incentive_tier_request_serde() {
        let json = r#"{
            "priority": 1,
            "max_late_early_count": 3,
            "max_unpaid_leave_days": 2.0,
            "payout_amount": 500.0
        }"#;
        let req: CreateIncentiveTierRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.priority, 1);
        assert_eq!(req.max_late_early_count, Some(3));
        assert_eq!(req.max_unpaid_leave_days, Some(2.0));
        assert_eq!(req.payout_amount, 500.0);
        assert!(req.max_sick_leave_days.is_none());
        assert!(req.max_total_leave_days.is_none());
    }

    #[test]
    fn test_create_payroll_incentive_policy_request_serde() {
        let json = r#"{
            "name": "Attendance Bonus",
            "base_amount": 1000.0,
            "payout_type": "fixed",
            "tiers": [
                {"priority": 1, "payout_amount": 1000.0},
                {"priority": 2, "max_late_early_count": 2, "payout_amount": 800.0}
            ]
        }"#;
        let req: CreatePayrollIncentivePolicyRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.name, "Attendance Bonus");
        assert_eq!(req.base_amount, 1000.0);
        assert_eq!(req.tiers.len(), 2);
    }

    #[test]
    fn test_incentive_evaluation_result_serde() {
        let result = IncentiveEvaluationResult {
            policy_name: "Attendance Bonus".to_string(),
            amount: Decimal::from(1000),
            standard_amount: Decimal::from(1000),
            tier_id: Some(Uuid::new_v4()),
        };
        let json = serde_json::to_string(&result).unwrap();
        let back: IncentiveEvaluationResult = serde_json::from_str(&json).unwrap();
        assert_eq!(back.policy_name, "Attendance Bonus");
        assert_eq!(back.amount, Decimal::from(1000));
    }
}
