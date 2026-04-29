use chrono::{DateTime, NaiveDate, NaiveTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::models::enums::{
    AccrualFrequency, LeaveDurationType, LeaveStatus, LeaveTransactionType,
};

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow, Clone)]
pub struct LeaveRequest {
    pub id: Uuid,
    pub employee_id: Uuid,
    pub leave_type_id: Uuid,
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
    pub reason: Option<String>,
    pub status: LeaveStatus,
    pub duration_type: LeaveDurationType,
    pub start_time: Option<NaiveTime>,
    pub end_time: Option<NaiveTime>,
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
    #[serde(default)]
    pub duration_type: LeaveDurationType,
    #[serde(default)]
    pub start_time: Option<NaiveTime>,
    #[serde(default)]
    pub end_time: Option<NaiveTime>,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_leave_type_config_default() {
        let config = LeaveTypeConfig::default();
        assert!(!config.requires_document);
        assert!(!config.allow_negative_balance);
        assert!(config.negative_balance_limit.is_none());
        assert!(!config.approval_required);
    }

    #[test]
    fn test_leave_type_config_serde_with_defaults() {
        let json = r#"{}"#;
        let config: LeaveTypeConfig = serde_json::from_str(json).unwrap();
        assert!(!config.requires_document);
        assert!(!config.allow_negative_balance);
        assert!(config.negative_balance_limit.is_none());
        assert!(!config.approval_required);
    }

    #[test]
    fn test_leave_type_config_serde_full() {
        let json = r#"{
            "requires_document": true,
            "allow_negative_balance": true,
            "negative_balance_limit": 5.0,
            "approval_required": true
        }"#;
        let config: LeaveTypeConfig = serde_json::from_str(json).unwrap();
        assert!(config.requires_document);
        assert!(config.allow_negative_balance);
        assert_eq!(config.negative_balance_limit, Some(5.0));
        assert!(config.approval_required);
    }

    #[test]
    fn test_leave_type_config_roundtrip() {
        let config = LeaveTypeConfig {
            requires_document: true,
            allow_negative_balance: false,
            negative_balance_limit: Some(3.0),
            approval_required: true,
        };
        let json = serde_json::to_string(&config).unwrap();
        let back: LeaveTypeConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(back.requires_document, config.requires_document);
        assert_eq!(back.allow_negative_balance, config.allow_negative_balance);
        assert_eq!(back.negative_balance_limit, config.negative_balance_limit);
    }

    #[test]
    fn test_leave_policy_config_serde() {
        let json = r#"{
            "frequency": "monthly",
            "accrual_amount": 1.5,
            "accrual_timing": "start_of_period",
            "carry_over_limit": 10,
            "max_balance": 30.0,
            "reset_month": 1,
            "min_work_days_threshold": 22.0
        }"#;
        let config: LeavePolicyConfig = serde_json::from_str(json).unwrap();
        assert_eq!(config.accrual_amount, 1.5);
        assert_eq!(config.accrual_timing, "start_of_period");
        assert_eq!(config.carry_over_limit, Some(10));
        assert_eq!(config.max_balance, Some(30.0));
        assert_eq!(config.reset_month, Some(1));
        assert!(config.seniority_bonus.is_empty()); // default empty vec
    }

    #[test]
    fn test_leave_policy_config_with_seniority_bonus() {
        let json = r#"{
            "frequency": "yearly",
            "accrual_amount": 12.0,
            "accrual_timing": "end_of_period",
            "seniority_bonus": [
                {"months": 12, "bonus_amount": 1.0},
                {"months": 36, "bonus_amount": 3.0}
            ]
        }"#;
        let config: LeavePolicyConfig = serde_json::from_str(json).unwrap();
        assert_eq!(config.seniority_bonus.len(), 2);
        assert_eq!(config.seniority_bonus[0].months, 12);
        assert_eq!(config.seniority_bonus[0].bonus_amount, 1.0);
    }

    #[test]
    fn test_submit_leave_request_serde() {
        let json = r#"{
            "employee_id": "00000000-0000-0000-0000-000000000001",
            "leave_type_id": "00000000-0000-0000-0000-000000000002",
            "start_date": "2026-03-01",
            "end_date": "2026-03-05",
            "reason": "Vacation"
        }"#;
        let req: SubmitLeaveRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.start_date, NaiveDate::from_ymd_opt(2026, 3, 1).unwrap());
        assert_eq!(req.end_date, NaiveDate::from_ymd_opt(2026, 3, 5).unwrap());
        assert_eq!(req.reason.as_deref(), Some("Vacation"));
    }

    #[test]
    fn test_approve_leave_request_serde() {
        let json = r#"{"status": "approved"}"#;
        let req: ApproveLeaveRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.status, LeaveStatus::Approved);
    }

    #[test]
    fn test_assign_leave_policy_request_defaults() {
        let json = r#"{
            "assignee_type": "workspace",
            "policy_id": "00000000-0000-0000-0000-000000000001",
            "effective_from": "2026-01-01"
        }"#;
        let req: AssignLeavePolicyRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.assignee_type, "workspace");
        assert!(req.assignee_id.is_none()); // serde default
        assert!(req.effective_to.is_none()); // serde default
    }

    #[test]
    fn test_adjust_balance_request_serde() {
        let json = r#"{
            "leave_type_id": "00000000-0000-0000-0000-000000000001",
            "amount": -2.0,
            "reason": "Correction",
            "transaction_date": "2026-03-15"
        }"#;
        let req: AdjustBalanceRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.amount, -2.0);
        assert_eq!(req.reason, "Correction");
    }

    #[test]
    fn test_update_leave_balance_request_empty() {
        let json = r#"{}"#;
        let req: UpdateLeaveBalanceRequest = serde_json::from_str(json).unwrap();
        assert!(req.entitlement.is_none());
        assert!(req.carried_over.is_none());
    }

    #[test]
    fn test_employee_leave_balance_serde() {
        let balance = EmployeeLeaveBalance {
            leave_type_id: Uuid::new_v4(),
            leave_type_code: "AL".to_string(),
            leave_type_name: "Annual Leave".to_string(),
            is_paid: true,
            entitlement: 12,
            accrued_amount: 6.0,
            carried_over: 2,
            used: 3,
            remaining: 5.0,
        };
        let json = serde_json::to_string(&balance).unwrap();
        let back: EmployeeLeaveBalance = serde_json::from_str(&json).unwrap();
        assert_eq!(back.leave_type_code, "AL");
        assert_eq!(back.remaining, 5.0);
    }

    #[test]
    fn test_create_leave_policy_request_serde() {
        let json = r#"{
            "name": "Standard AL Policy",
            "leave_type_id": "00000000-0000-0000-0000-000000000001",
            "rule_config": {
                "frequency": "monthly",
                "accrual_amount": 1.0,
                "accrual_timing": "start_of_period"
            }
        }"#;
        let req: CreateLeavePolicyRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.name, "Standard AL Policy");
        assert!(req.description.is_none());
    }

    #[test]
    fn test_update_leave_policy_request_all_none() {
        let json = r#"{}"#;
        let req: UpdateLeavePolicyRequest = serde_json::from_str(json).unwrap();
        assert!(req.name.is_none());
        assert!(req.description.is_none());
        assert!(req.leave_type_id.is_none());
        assert!(req.rule_config.is_none());
    }
}
