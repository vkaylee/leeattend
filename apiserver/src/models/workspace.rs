use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::models::enums::{EarlyArrivalStrategy, RoleScopeType};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyRules {
    pub grace_period_minutes: i32,
    pub authorized_early_leave_minutes: i32,
    pub is_early_leave_paid: bool,
    pub early_arrival_strategy: EarlyArrivalStrategy,
    pub is_break_auto_deduct: bool,
    pub is_break_overtime: bool,
    pub allow_early_ot: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct WorkingCondition {
    pub id: Uuid,
    pub employee_id: Uuid,
    #[sqlx(rename = "type")]
    pub type_: String, // Keep as string for now to match DB, but we can parse to enum
    pub minute_allowance: i32,
    pub is_paid: bool,
    pub start_date: NaiveDate,
    pub end_date: Option<NaiveDate>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CreateWorkingConditionRequest {
    pub type_: String,
    pub minute_allowance: i32,
    pub is_paid: bool,
    pub start_date: NaiveDate,
    pub end_date: Option<NaiveDate>,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow, Clone)]
pub struct Workspace {
    pub id: Uuid,
    pub name: String,
    pub date_format: String,
    pub time_format: String,
    pub timezone: String,
    pub theme_primary_color: String,
    pub default_language: String,
    pub early_arrival_strategy: EarlyArrivalStrategy,
    pub holiday_overnight_strategy: String,
    pub default_holiday_ot_multiplier: Decimal,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CreateWorkspaceRequest {
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UpdateWorkspaceRequest {
    pub name: Option<String>,
    pub date_format: Option<String>,
    pub time_format: Option<String>,
    pub timezone: Option<String>,
    pub theme_primary_color: Option<String>,
    pub default_language: Option<String>,
    pub early_arrival_strategy: Option<EarlyArrivalStrategy>,
    pub holiday_overnight_strategy: Option<String>,
    pub default_holiday_ot_multiplier: Option<Decimal>,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow, Clone)]
pub struct WorkspaceMember {
    pub workspace_id: Uuid,
    pub user_id: Uuid,
    pub role_id: Uuid,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AddMemberRequest {
    pub email: String,
    pub role_id: Uuid,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow, Clone)]
pub struct MemberExtraRole {
    pub id: Uuid,
    pub workspace_id: Uuid,
    pub user_id: Uuid,
    pub role_id: Uuid,
    pub assigned_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AssignRoleRequest {
    pub role_id: Uuid,
    pub expires_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow, Clone)]
pub struct WorkspaceMemberDetail {
    pub workspace_id: Uuid,
    pub user_id: Uuid,
    pub role_id: Uuid,
    pub email: String,
    pub full_name: Option<String>,
    pub avatar_url: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WorkspaceMemberDetailResponse {
    #[serde(flatten)]
    pub detail: WorkspaceMemberDetail,
    pub extra_role_ids: Vec<Uuid>,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow, Clone)]
pub struct MemberExtraPermission {
    pub id: Uuid,
    pub workspace_id: Uuid,
    pub user_id: Uuid,
    pub permission: String,
    pub assigned_at: Option<DateTime<Utc>>,
    pub expires_at: DateTime<Utc>,
    pub assigned_by: Option<Uuid>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AssignPermissionRequest {
    pub permission: String,
    pub expires_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow, Clone)]
pub struct WorkspaceRole {
    pub id: Uuid,
    pub workspace_id: Option<Uuid>,
    pub name: String,
    pub description: Option<String>,
    pub color: String,
    pub is_system: bool,
    pub scope_type: RoleScopeType,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow, Clone)]
pub struct WorkspaceRolePermission {
    pub role_id: Uuid,
    pub permission_name: String,
}
