use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

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
    pub period_start: NaiveDate,
    pub period_end: NaiveDate,
    pub base_pay_amount: rust_decimal::Decimal,
    pub overtime_pay_amount: rust_decimal::Decimal,
    pub deductions_amount: rust_decimal::Decimal,
    pub net_pay_amount: rust_decimal::Decimal,
    pub standard_units: i32,
    pub overtime_units: i32,
    pub status: String,
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
