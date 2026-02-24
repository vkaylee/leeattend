use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::models::enums::{AccrualFrequency, LeaveStatus, LeaveTransactionType};

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow, Clone)]
pub struct LeaveRequest {
    pub id: Uuid,
    pub employee_id: Uuid,
    pub leave_type_id: Uuid,
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
    pub reason: Option<String>,
    pub status: LeaveStatus,
    pub requested_by_user_id: Option<Uuid>,
    pub approved_by_user_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Response wrapper with leave type details for API responses
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LeaveRequestWithType {
    #[serde(flatten)]
    pub request: LeaveRequest,
    pub leave_type_code: String,
    pub leave_type_name: String,
    pub is_paid: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SubmitLeaveRequest {
    pub employee_id: Uuid,
    pub leave_type_id: Uuid,
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
    pub reason: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ApproveLeaveRequest {
    pub status: LeaveStatus,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow, Clone)]
pub struct LeaveBalance {
    pub id: Uuid,
    pub employee_id: Uuid,
    pub leave_type_id: Uuid,
    pub year: i32,
    pub entitlement: i32,
    pub accrued_amount: rust_decimal::Decimal,
    pub carried_over: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EmployeeLeaveBalance {
    pub leave_type_id: Uuid,
    pub leave_type_code: String,
    pub leave_type_name: String,
    pub is_paid: bool,
    pub entitlement: i32,
    pub accrued_amount: f64,
    pub carried_over: i32,
    pub used: i32,
    pub remaining: f64,
}

#[derive(Debug, Serialize, Deserialize, Clone, sqlx::FromRow)]
pub struct WorkspaceLeaveBalance {
    pub employee_id: Uuid,
    pub employee_name: String,
    pub leave_type_id: Uuid,
    pub leave_type_code: String,
    pub leave_type_name: String,
    pub is_paid: bool,
    pub entitlement: i32,
    pub accrued_amount: f64,
    pub carried_over: i32,
    pub used: i32,
    pub remaining: f64,
    pub policy_name: Option<String>,
    pub assignment_type: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateLeaveBalanceRequest {
    pub entitlement: Option<i32>,
    pub carried_over: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SeniorityBonus {
    pub months: i32,
    pub bonus_amount: f64,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct LeaveTypeConfig {
    #[serde(default)]
    pub requires_document: bool,
    #[serde(default)]
    pub allow_negative_balance: bool,
    #[serde(default)]
    pub negative_balance_limit: Option<f64>,
    #[serde(default)]
    pub approval_required: bool,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow, Clone)]
pub struct LeaveType {
    pub id: Uuid,
    pub workspace_id: Uuid,
    pub code: String,
    pub name: String,
    pub is_paid: bool,
    pub policy_config: sqlx::types::Json<LeaveTypeConfig>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LeavePolicyConfig {
    pub frequency: AccrualFrequency,
    pub accrual_amount: f64,
    pub accrual_timing: String, // "start_of_period" or "end_of_period"
    #[serde(default)]
    pub seniority_bonus: Vec<SeniorityBonus>,
    pub carry_over_limit: Option<i32>,
    pub max_balance: Option<f64>,
    pub reset_month: Option<i32>,
    pub min_work_days_threshold: Option<f64>,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow, Clone)]
pub struct LeavePolicyDefinition {
    pub id: Uuid,
    pub workspace_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub leave_type_id: Uuid,
    pub rule_config: sqlx::types::Json<LeavePolicyConfig>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow, Clone)]
pub struct LeavePolicyAssignment {
    pub id: Uuid,
    pub workspace_id: Uuid,
    pub assignee_type: String,
    pub assignee_id: Option<Uuid>,
    pub policy_id: Uuid,
    pub effective_from: NaiveDate,
    pub effective_to: Option<NaiveDate>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow, Clone)]
pub struct LeaveAccrualLog {
    pub id: Uuid,
    pub workspace_id: Uuid,
    pub employee_id: Uuid,
    pub policy_assignment_id: Option<Uuid>,
    pub accrual_date: NaiveDate,
    pub amount: rust_decimal::Decimal,
    pub leave_type_id: Uuid,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CreateLeavePolicyRequest {
    pub name: String,
    pub description: Option<String>,
    pub leave_type_id: Uuid,
    pub rule_config: LeavePolicyConfig,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UpdateLeavePolicyRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub leave_type_id: Option<Uuid>,
    pub rule_config: Option<LeavePolicyConfig>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AssignLeavePolicyRequest {
    pub assignee_type: String,
    #[serde(default)]
    pub assignee_id: Option<Uuid>,
    pub policy_id: Uuid,
    pub effective_from: NaiveDate,
    #[serde(default)]
    pub effective_to: Option<NaiveDate>,
}

#[derive(Debug, Serialize, Deserialize, Clone, sqlx::FromRow)]
pub struct LeaveHistoryItem {
    pub id: Uuid,
    pub date: NaiveDate,
    pub transaction_type: LeaveTransactionType,
    pub amount: f64,
    pub reason: Option<String>,
    pub reference_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AdjustBalanceRequest {
    pub leave_type_id: Uuid,
    pub amount: f64, // Positive to add, Negative to deduct
    pub reason: String,
    pub transaction_date: NaiveDate,
}

#[derive(Debug, Serialize, Deserialize, Clone, sqlx::FromRow)]
pub struct LeavePolicyAssignmentDetail {
    pub id: Uuid,
    pub workspace_id: Uuid,
    pub assignee_type: String,
    pub assignee_id: Option<Uuid>,
    pub policy_id: Uuid,
    pub effective_from: NaiveDate,
    pub effective_to: Option<NaiveDate>,
    pub created_at: DateTime<Utc>,
    pub policy_name: String,
    pub leave_type_name: String,
}
