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

#[derive(Debug, Serialize, Deserialize, Clone, utoipa::ToSchema)]
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

#[derive(Debug, Serialize, Deserialize, Clone, utoipa::ToSchema)]
pub struct Payslip {
    pub id: Uuid,
    pub employment_id: Uuid,
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

#[derive(Debug, Serialize, Deserialize, Clone, utoipa::ToSchema)]
pub struct PayrollIncentivePolicy {
    pub id: Uuid,
    pub workspace_id: Uuid,
    pub name: String,
    pub base_amount: f64,
    pub payout_type: String, // String to match DB, can be parsed to PayoutType
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub effective_from: chrono::NaiveDate,
    pub effective_to: Option<chrono::NaiveDate>,
}

#[derive(Debug, Serialize, Deserialize, Clone, utoipa::ToSchema)]
pub struct IncentiveTier {
    pub id: Uuid,
    pub policy_id: Uuid,
    pub priority: i32,
    pub rule_config: serde_json::Value,
    pub payout_amount: f64,
}

#[derive(Debug, Serialize, Deserialize, Clone, utoipa::ToSchema)]
pub struct IncentiveEvaluationResult {
    pub policy_name: String,
    pub amount: rust_decimal::Decimal,
    pub standard_amount: rust_decimal::Decimal,
    pub tier_id: Option<Uuid>,
}

#[derive(Debug, Serialize, Deserialize, Clone, utoipa::ToSchema)]
pub struct PayslipRuleEvaluation {
    pub id: Uuid,
    pub payslip_id: Uuid,
    pub rule_code: String,
    pub amount: rust_decimal::Decimal,
    pub created_at: DateTime<Utc>,
}

fn default_rule_config() -> serde_json::Value {
    serde_json::json!({})
}

#[derive(Debug, Serialize, Deserialize, Clone, utoipa::ToSchema)]
pub struct CreateIncentiveTierRequest {
    pub priority: i32,
    #[serde(default = "default_rule_config")]
    pub rule_config: serde_json::Value,
    pub payout_amount: f64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PayrollIncentiveAssignment {
    pub id: Uuid,
    pub workspace_id: Uuid,
    pub policy_id: Uuid,
    pub assignee_type: String,
    pub assignee_id: Option<Uuid>,
    pub effective_from: NaiveDate,
    pub effective_to: Option<NaiveDate>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone, utoipa::ToSchema)]
pub struct CreatePayrollIncentivePolicyRequest {
    pub name: String,
    pub base_amount: f64,
    pub payout_type: String,
    pub effective_from: chrono::NaiveDate,
    pub tiers: Vec<CreateIncentiveTierRequest>,
}

#[derive(Debug, Serialize, Deserialize, Clone, utoipa::ToSchema)]
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

#[derive(Debug, Serialize, Deserialize, utoipa::ToSchema)]
pub struct BatchCalculatePayrollRequest {
    pub employee_ids: Vec<Uuid>,
    pub year: i32,
    pub month: u32,
}

#[derive(Debug, Serialize, Deserialize, utoipa::ToSchema)]
pub struct BatchCalculateResult {
    pub employee_id: Uuid,
    pub status: String, // "success" or "error"
    pub payslip: Option<Payslip>,
    pub error_message: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, utoipa::ToSchema)]
pub struct BatchCalculatePayrollResponse {
    pub results: Vec<BatchCalculateResult>,
}

#[derive(Debug, Serialize, Deserialize, Clone, utoipa::ToSchema)]
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

pub mod serde_decimal_as_str {
    use rust_decimal::Decimal;
    use serde::{self, Deserialize, Deserializer, Serializer};
    use std::str::FromStr;

    pub fn serialize<S>(value: &Decimal, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&value.normalize().to_string())
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Decimal, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(untagged)]
        enum StringOrNumber {
            String(String),
            Number(serde_json::Number),
        }

        match StringOrNumber::deserialize(deserializer)? {
            StringOrNumber::String(s) => Decimal::from_str(&s).map_err(serde::de::Error::custom),
            StringOrNumber::Number(n) => {
                Decimal::from_str(&n.to_string()).map_err(serde::de::Error::custom)
            }
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, utoipa::ToSchema)]
pub struct DepartmentPayrollConfig {
    pub id: Uuid,
    pub department_id: Uuid,
    #[serde(with = "serde_decimal_as_str")]
    pub std_working_days: rust_decimal::Decimal,
    #[serde(with = "serde_decimal_as_str")]
    pub hours_per_day: rust_decimal::Decimal,
    pub effective_date: NaiveDate,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone, utoipa::ToSchema)]
pub struct EmployeePayrollConfig {
    pub id: Uuid,
    pub employee_id: Uuid,
    #[serde(with = "serde_decimal_as_str")]
    pub std_working_days: rust_decimal::Decimal,
    #[serde(with = "serde_decimal_as_str")]
    pub hours_per_day: rust_decimal::Decimal,
    pub effective_date: NaiveDate,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone, utoipa::ToSchema)]
pub struct SalaryComponentDefinition {
    pub id: Uuid,
    pub workspace_id: Uuid,
    pub name: String,
    pub code: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone, utoipa::ToSchema)]
pub struct EmployeeSalaryComponent {
    pub id: Uuid,
    pub employment_id: Uuid,
    pub component_definition_id: Uuid,
    pub amount: rust_decimal::Decimal,
    pub effective_date: NaiveDate,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct UpdatePayrollSettingsRequest {
    pub std_working_day_mode: Option<String>,
    pub fixed_std_days: Option<rust_decimal::Decimal>,
    pub working_days_per_week: Option<i32>,
    pub hours_per_day: Option<rust_decimal::Decimal>,
    pub max_ot_hours_per_day: Option<Option<rust_decimal::Decimal>>,
    pub max_ot_hours_per_month: Option<Option<rust_decimal::Decimal>>,
    pub effective_date: Option<NaiveDate>,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct SetDepartmentPayrollConfigRequest {
    pub std_working_days: rust_decimal::Decimal,
    pub hours_per_day: rust_decimal::Decimal,
    pub effective_date: NaiveDate,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct SetEmployeePayrollConfigRequest {
    pub std_working_days: rust_decimal::Decimal,
    pub hours_per_day: rust_decimal::Decimal,
    pub effective_date: NaiveDate,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct CreateSalaryComponentRequest {
    pub name: String,
    pub code: String,
    pub base_group_ids: Option<Vec<Uuid>>,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct AssignSalaryComponentRequest {
    pub component_definition_id: Uuid,
    pub amount: rust_decimal::Decimal,
    pub effective_date: NaiveDate,
}

#[derive(Debug, Serialize, Deserialize, Clone, utoipa::ToSchema)]
pub struct SalaryComponentWithDefinition {
    pub component: EmployeeSalaryComponent,
    pub definition: SalaryComponentDefinition,
}

#[derive(Debug, Serialize, Deserialize, Clone, utoipa::ToSchema)]
pub struct CompensationEmploymentContext {
    pub id: Uuid,
    pub employer_name: String,
    pub department_name: Option<String>,
    pub job_title: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, utoipa::ToSchema)]
pub struct CompensationChangeDetails {
    pub previous_total: Option<rust_decimal::Decimal>,
    pub delta: rust_decimal::Decimal,
}

#[derive(Debug, Serialize, Deserialize, Clone, utoipa::ToSchema)]
pub struct CompensationTimelineEvent {
    pub effective_date: NaiveDate,
    pub employment: CompensationEmploymentContext,
    pub components: Vec<SalaryComponentWithDefinition>,
    pub total_compensation: rust_decimal::Decimal,
    pub change_type: String, // "initial" | "promotion" | "adjustment" | "transfer"
    pub change_details: CompensationChangeDetails,
}

#[derive(Debug, Serialize, Deserialize, Clone, utoipa::ToSchema)]
pub struct CompensationHistoryResponse {
    pub timeline: Vec<CompensationTimelineEvent>,
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
            "base_group_ids": ["00000000-0000-0000-0000-000000000001"]
        }"#;
        let req: CreateSalaryComponentRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.name, "Housing Allowance");
        assert_eq!(req.code, "HOUSE");
        assert_eq!(req.base_group_ids.unwrap().len(), 1);
    }

    #[test]
    fn test_assign_salary_component_request_serde() {
        let json = r#"{
            "component_definition_id": "00000000-0000-0000-0000-000000000001",
            "amount": "5000.50",
            "effective_date": "2026-01-01"
        }"#;
        let req: AssignSalaryComponentRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.amount, Decimal::new(500_050, 2));
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
        let json = r"{}";
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
            "rule_config": {
                "max_late_early_count": 3
            },
            "payout_amount": 500.0
        }"#;
        let req: CreateIncentiveTierRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.priority, 1);
        assert_eq!(req.rule_config["max_late_early_count"], 3);
        assert!((req.payout_amount - 500.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_create_payroll_incentive_policy_request_serde() {
        let json = r#"{
            "name": "Attendance Bonus",
            "base_amount": 1000.0,
            "payout_type": "fixed",
            "effective_from": "2026-01-01",
            "tiers": [
                {"priority": 1, "payout_amount": 1000.0},
                {"priority": 2, "rule_config": {"max_late_early_count": 2}, "payout_amount": 800.0}
            ]
        }"#;
        let req: CreatePayrollIncentivePolicyRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.name, "Attendance Bonus");
        assert!((req.base_amount - 1000.0).abs() < f64::EPSILON);
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

    #[test]
    fn test_payroll_config_serde_serialization() {
        let config = DepartmentPayrollConfig {
            id: Uuid::new_v4(),
            department_id: Uuid::new_v4(),
            std_working_days: Decimal::new(2200, 2),
            hours_per_day: Decimal::new(800, 2),
            effective_date: chrono::NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
            created_at: chrono::Utc::now(),
        };
        let val: serde_json::Value = serde_json::to_value(&config).unwrap();
        assert_eq!(val["std_working_days"], serde_json::json!("22"));
        assert_eq!(val["hours_per_day"], serde_json::json!("8"));
    }
}
