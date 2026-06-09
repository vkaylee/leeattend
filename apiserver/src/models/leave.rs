use chrono::{DateTime, NaiveDate, NaiveTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::models::enums::{
    AccrualFrequency, LeaveDurationType, LeaveStatus, LeaveTransactionType,
};

#[derive(Debug, Serialize, Deserialize, Clone, utoipa::ToSchema)]
pub struct LeaveRequest {
    id: Uuid,
    employment_id: Uuid,
    leave_type_id: Uuid,
    start_date: NaiveDate,
    end_date: NaiveDate,
    reason: Option<String>,
    status: LeaveStatus,
    duration_type: LeaveDurationType,
    start_time: Option<NaiveTime>,
    end_time: Option<NaiveTime>,
    requested_by_user_id: Option<Uuid>,
    approved_by_user_id: Option<Uuid>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl LeaveRequest {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: Uuid,
        employment_id: Uuid,
        leave_type_id: Uuid,
        start_date: NaiveDate,
        end_date: NaiveDate,
        reason: Option<String>,
        status: LeaveStatus,
        duration_type: LeaveDurationType,
        start_time: Option<NaiveTime>,
        end_time: Option<NaiveTime>,
        requested_by_user_id: Option<Uuid>,
        approved_by_user_id: Option<Uuid>,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id,
            employment_id,
            leave_type_id,
            start_date,
            end_date,
            reason,
            status,
            duration_type,
            start_time,
            end_time,
            requested_by_user_id,
            approved_by_user_id,
            created_at,
            updated_at,
        }
    }

    pub fn id(&self) -> Uuid {
        self.id
    }

    pub fn employment_id(&self) -> Uuid {
        self.employment_id
    }

    pub fn leave_type_id(&self) -> Uuid {
        self.leave_type_id
    }

    pub fn start_date(&self) -> NaiveDate {
        self.start_date
    }

    pub fn end_date(&self) -> NaiveDate {
        self.end_date
    }

    pub fn reason(&self) -> Option<&str> {
        self.reason.as_deref()
    }

    pub fn status(&self) -> LeaveStatus {
        self.status
    }

    pub fn duration_type(&self) -> LeaveDurationType {
        self.duration_type
    }

    pub fn start_time(&self) -> Option<NaiveTime> {
        self.start_time
    }

    pub fn end_time(&self) -> Option<NaiveTime> {
        self.end_time
    }

    pub fn requested_by_user_id(&self) -> Option<Uuid> {
        self.requested_by_user_id
    }

    pub fn approved_by_user_id(&self) -> Option<Uuid> {
        self.approved_by_user_id
    }

    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }

    pub fn updated_at(&self) -> DateTime<Utc> {
        self.updated_at
    }
}

/// Response wrapper with leave type details for API responses
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LeaveRequestWithType {
    #[serde(flatten)]
    request: LeaveRequest,
    leave_type_code: String,
    leave_type_name: String,
    is_paid: bool,
}

impl LeaveRequestWithType {
    pub fn new(
        request: LeaveRequest,
        leave_type_code: String,
        leave_type_name: String,
        is_paid: bool,
    ) -> Self {
        Self {
            request,
            leave_type_code,
            leave_type_name,
            is_paid,
        }
    }

    pub fn request(&self) -> &LeaveRequest {
        &self.request
    }

    pub fn leave_type_code(&self) -> &str {
        &self.leave_type_code
    }

    pub fn leave_type_name(&self) -> &str {
        &self.leave_type_name
    }

    pub fn is_paid(&self) -> bool {
        self.is_paid
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, utoipa::ToSchema)]
pub struct SubmitLeaveRequest {
    pub employment_id: Uuid,
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

#[derive(Debug, Serialize, Deserialize, utoipa::ToSchema)]
pub struct ApproveLeaveRequest {
    pub status: LeaveStatus,
}

#[derive(Debug, Serialize, Deserialize, Clone, utoipa::ToSchema)]
pub struct LeaveBalance {
    id: Uuid,
    employment_id: Uuid,
    leave_type_id: Uuid,
    year: i32,
    entitlement: i32,
    accrued_amount: rust_decimal::Decimal,
    carried_over: i32,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl LeaveBalance {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: Uuid,
        employment_id: Uuid,
        leave_type_id: Uuid,
        year: i32,
        entitlement: i32,
        accrued_amount: rust_decimal::Decimal,
        carried_over: i32,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id,
            employment_id,
            leave_type_id,
            year,
            entitlement,
            accrued_amount,
            carried_over,
            created_at,
            updated_at,
        }
    }

    pub fn id(&self) -> Uuid {
        self.id
    }

    pub fn employment_id(&self) -> Uuid {
        self.employment_id
    }

    pub fn leave_type_id(&self) -> Uuid {
        self.leave_type_id
    }

    pub fn year(&self) -> i32 {
        self.year
    }

    pub fn entitlement(&self) -> i32 {
        self.entitlement
    }

    pub fn accrued_amount(&self) -> rust_decimal::Decimal {
        self.accrued_amount
    }

    pub fn carried_over(&self) -> i32 {
        self.carried_over
    }

    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }

    pub fn updated_at(&self) -> DateTime<Utc> {
        self.updated_at
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, utoipa::ToSchema)]
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

#[derive(Debug, Serialize, Deserialize, Clone, utoipa::ToSchema)]
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

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct UpdateLeaveBalanceRequest {
    pub entitlement: Option<i32>,
    pub carried_over: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default, utoipa::ToSchema)]
pub struct SeniorityBonus {
    pub months: i32,
    pub bonus_amount: f64,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default, utoipa::ToSchema)]
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

#[derive(Debug, Serialize, Deserialize, Clone, utoipa::ToSchema)]
pub struct LeaveType {
    id: Uuid,
    workspace_id: Uuid,
    code: String,
    name: String,
    is_paid: bool,
    policy_config: LeaveTypeConfig,
    created_at: DateTime<Utc>,
}

impl LeaveType {
    pub fn new(
        id: Uuid,
        workspace_id: Uuid,
        code: String,
        name: String,
        is_paid: bool,
        policy_config: LeaveTypeConfig,
        created_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id,
            workspace_id,
            code,
            name,
            is_paid,
            policy_config,
            created_at,
        }
    }

    pub fn id(&self) -> Uuid {
        self.id
    }

    pub fn workspace_id(&self) -> Uuid {
        self.workspace_id
    }

    pub fn code(&self) -> &str {
        &self.code
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn is_paid(&self) -> bool {
        self.is_paid
    }

    pub fn policy_config(&self) -> &LeaveTypeConfig {
        &self.policy_config
    }

    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Default, utoipa::ToSchema)]
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

#[derive(Debug, Serialize, Deserialize, Clone, utoipa::ToSchema)]
pub struct LeavePolicyDefinition {
    id: Uuid,
    workspace_id: Uuid,
    name: String,
    description: Option<String>,
    leave_type_id: Uuid,
    rule_config: LeavePolicyConfig,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl LeavePolicyDefinition {
    pub fn new(
        id: Uuid,
        workspace_id: Uuid,
        name: String,
        description: Option<String>,
        leave_type_id: Uuid,
        rule_config: LeavePolicyConfig,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id,
            workspace_id,
            name,
            description,
            leave_type_id,
            rule_config,
            created_at,
            updated_at,
        }
    }

    pub fn id(&self) -> Uuid {
        self.id
    }

    pub fn workspace_id(&self) -> Uuid {
        self.workspace_id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }

    pub fn leave_type_id(&self) -> Uuid {
        self.leave_type_id
    }

    pub fn rule_config(&self) -> &LeavePolicyConfig {
        &self.rule_config
    }

    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }

    pub fn updated_at(&self) -> DateTime<Utc> {
        self.updated_at
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, utoipa::ToSchema)]
pub struct LeavePolicyAssignment {
    id: Uuid,
    workspace_id: Uuid,
    assignee_type: String,
    assignee_id: Option<Uuid>,
    policy_id: Uuid,
    effective_from: NaiveDate,
    effective_to: Option<NaiveDate>,
    created_at: DateTime<Utc>,
    deleted_at: Option<DateTime<Utc>>,
}

impl LeavePolicyAssignment {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: Uuid,
        workspace_id: Uuid,
        assignee_type: String,
        assignee_id: Option<Uuid>,
        policy_id: Uuid,
        effective_from: NaiveDate,
        effective_to: Option<NaiveDate>,
        created_at: DateTime<Utc>,
        deleted_at: Option<DateTime<Utc>>,
    ) -> Self {
        Self {
            id,
            workspace_id,
            assignee_type,
            assignee_id,
            policy_id,
            effective_from,
            effective_to,
            created_at,
            deleted_at,
        }
    }

    pub fn id(&self) -> Uuid {
        self.id
    }

    pub fn workspace_id(&self) -> Uuid {
        self.workspace_id
    }

    pub fn assignee_type(&self) -> &str {
        &self.assignee_type
    }

    pub fn assignee_id(&self) -> Option<Uuid> {
        self.assignee_id
    }

    pub fn policy_id(&self) -> Uuid {
        self.policy_id
    }

    pub fn effective_from(&self) -> NaiveDate {
        self.effective_from
    }

    pub fn effective_to(&self) -> Option<NaiveDate> {
        self.effective_to
    }

    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }

    pub fn deleted_at(&self) -> Option<DateTime<Utc>> {
        self.deleted_at
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, utoipa::ToSchema)]
pub struct LeaveAccrualLog {
    id: Uuid,
    workspace_id: Uuid,
    employment_id: Uuid,
    policy_assignment_id: Option<Uuid>,
    accrual_date: NaiveDate,
    amount: rust_decimal::Decimal,
    leave_type_id: Uuid,
    notes: Option<String>,
    created_at: DateTime<Utc>,
}

impl LeaveAccrualLog {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: Uuid,
        workspace_id: Uuid,
        employment_id: Uuid,
        policy_assignment_id: Option<Uuid>,
        accrual_date: NaiveDate,
        amount: rust_decimal::Decimal,
        leave_type_id: Uuid,
        notes: Option<String>,
        created_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id,
            workspace_id,
            employment_id,
            policy_assignment_id,
            accrual_date,
            amount,
            leave_type_id,
            notes,
            created_at,
        }
    }

    pub fn id(&self) -> Uuid {
        self.id
    }

    pub fn workspace_id(&self) -> Uuid {
        self.workspace_id
    }

    pub fn employment_id(&self) -> Uuid {
        self.employment_id
    }

    pub fn policy_assignment_id(&self) -> Option<Uuid> {
        self.policy_assignment_id
    }

    pub fn accrual_date(&self) -> NaiveDate {
        self.accrual_date
    }

    pub fn amount(&self) -> rust_decimal::Decimal {
        self.amount
    }

    pub fn leave_type_id(&self) -> Uuid {
        self.leave_type_id
    }

    pub fn notes(&self) -> Option<&str> {
        self.notes.as_deref()
    }

    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, utoipa::ToSchema)]
pub struct CreateLeavePolicyRequest {
    pub name: String,
    pub description: Option<String>,
    pub leave_type_id: Uuid,
    pub rule_config: LeavePolicyConfig,
}

#[derive(Debug, Serialize, Deserialize, Clone, utoipa::ToSchema)]
pub struct UpdateLeavePolicyRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub leave_type_id: Option<Uuid>,
    pub rule_config: Option<LeavePolicyConfig>,
}

#[derive(Debug, Serialize, Deserialize, Clone, utoipa::ToSchema)]
pub struct AssignLeavePolicyRequest {
    pub assignee_type: String,
    #[serde(default)]
    pub assignee_id: Option<Uuid>,
    pub policy_id: Uuid,
    pub effective_from: NaiveDate,
    #[serde(default)]
    pub effective_to: Option<NaiveDate>,
}

#[derive(Debug, Serialize, Deserialize, Clone, utoipa::ToSchema)]
pub struct LeaveHistoryItem {
    pub id: Uuid,
    pub date: NaiveDate,
    pub transaction_type: LeaveTransactionType,
    pub amount: f64,
    pub reason: Option<String>,
    pub reference_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, utoipa::ToSchema)]
pub struct AdjustBalanceRequest {
    pub leave_type_id: Uuid,
    pub amount: f64, // Positive to add, Negative to deduct
    pub reason: String,
    pub transaction_date: NaiveDate,
}

#[derive(Debug, Serialize, Deserialize, Clone, utoipa::ToSchema)]
pub struct LeavePolicyAssignmentDetail {
    id: Uuid,
    workspace_id: Uuid,
    assignee_type: String,
    assignee_id: Option<Uuid>,
    policy_id: Uuid,
    effective_from: NaiveDate,
    effective_to: Option<NaiveDate>,
    created_at: DateTime<Utc>,
    policy_name: String,
    leave_type_name: String,
}

impl LeavePolicyAssignmentDetail {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: Uuid,
        workspace_id: Uuid,
        assignee_type: String,
        assignee_id: Option<Uuid>,
        policy_id: Uuid,
        effective_from: NaiveDate,
        effective_to: Option<NaiveDate>,
        created_at: DateTime<Utc>,
        policy_name: String,
        leave_type_name: String,
    ) -> Self {
        Self {
            id,
            workspace_id,
            assignee_type,
            assignee_id,
            policy_id,
            effective_from,
            effective_to,
            created_at,
            policy_name,
            leave_type_name,
        }
    }

    pub fn id(&self) -> Uuid {
        self.id
    }

    pub fn workspace_id(&self) -> Uuid {
        self.workspace_id
    }

    pub fn assignee_type(&self) -> &str {
        &self.assignee_type
    }

    pub fn assignee_id(&self) -> Option<Uuid> {
        self.assignee_id
    }

    pub fn policy_id(&self) -> Uuid {
        self.policy_id
    }

    pub fn effective_from(&self) -> NaiveDate {
        self.effective_from
    }

    pub fn effective_to(&self) -> Option<NaiveDate> {
        self.effective_to
    }

    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }

    pub fn policy_name(&self) -> &str {
        &self.policy_name
    }

    pub fn leave_type_name(&self) -> &str {
        &self.leave_type_name
    }
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
        let json = r"{}";
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
        assert!((config.negative_balance_limit.unwrap() - 5.0).abs() < f64::EPSILON);
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
        assert!(
            (back.negative_balance_limit.unwrap() - config.negative_balance_limit.unwrap()).abs()
                < f64::EPSILON
        );
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
        assert!((config.accrual_amount - 1.5).abs() < f64::EPSILON);
        assert_eq!(config.accrual_timing, "start_of_period");
        assert_eq!(config.carry_over_limit, Some(10));
        assert!((config.max_balance.unwrap() - 30.0).abs() < f64::EPSILON);
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
        assert!((config.seniority_bonus[0].bonus_amount - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_submit_leave_request_serde() {
        let json = r#"{
            "employment_id": "00000000-0000-0000-0000-000000000001",
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
        assert!((req.amount - -2.0).abs() < f64::EPSILON);
        assert_eq!(req.reason, "Correction");
    }

    #[test]
    fn test_update_leave_balance_request_empty() {
        let json = r"{}";
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
        assert!((back.remaining - 5.0).abs() < f64::EPSILON);
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
        let json = r"{}";
        let req: UpdateLeavePolicyRequest = serde_json::from_str(json).unwrap();
        assert!(req.name.is_none());
        assert!(req.description.is_none());
        assert!(req.leave_type_id.is_none());
        assert!(req.rule_config.is_none());
    }
}
