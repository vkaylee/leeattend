use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::models::enums::{EarlyArrivalStrategy, RoleScopeType};

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
#[allow(clippy::struct_excessive_bools)]
pub struct PolicyRules {
    grace_period_minutes: i32,
    authorized_early_leave_minutes: i32,
    is_early_leave_paid: bool,
    early_arrival_strategy: EarlyArrivalStrategy,
    is_break_auto_deduct: bool,
    is_break_overtime: bool,
    allow_early_ot: bool,
}

impl PolicyRules {
    #[allow(clippy::fn_params_excessive_bools)]
    pub fn new(
        grace_period_minutes: i32,
        authorized_early_leave_minutes: i32,
        is_early_leave_paid: bool,
        early_arrival_strategy: EarlyArrivalStrategy,
        is_break_auto_deduct: bool,
        is_break_overtime: bool,
        allow_early_ot: bool,
    ) -> Self {
        Self {
            grace_period_minutes,
            authorized_early_leave_minutes,
            is_early_leave_paid,
            early_arrival_strategy,
            is_break_auto_deduct,
            is_break_overtime,
            allow_early_ot,
        }
    }

    pub fn grace_period_minutes(&self) -> i32 {
        self.grace_period_minutes
    }

    pub fn set_grace_period_minutes(&mut self, val: i32) {
        self.grace_period_minutes = val;
    }

    pub fn authorized_early_leave_minutes(&self) -> i32 {
        self.authorized_early_leave_minutes
    }

    pub fn set_authorized_early_leave_minutes(&mut self, val: i32) {
        self.authorized_early_leave_minutes = val;
    }

    pub fn is_early_leave_paid(&self) -> bool {
        self.is_early_leave_paid
    }

    pub fn set_is_early_leave_paid(&mut self, val: bool) {
        self.is_early_leave_paid = val;
    }

    pub fn early_arrival_strategy(&self) -> EarlyArrivalStrategy {
        self.early_arrival_strategy
    }

    pub fn set_early_arrival_strategy(&mut self, val: EarlyArrivalStrategy) {
        self.early_arrival_strategy = val;
    }

    pub fn is_break_auto_deduct(&self) -> bool {
        self.is_break_auto_deduct
    }

    pub fn set_is_break_auto_deduct(&mut self, val: bool) {
        self.is_break_auto_deduct = val;
    }

    pub fn is_break_overtime(&self) -> bool {
        self.is_break_overtime
    }

    pub fn set_is_break_overtime(&mut self, val: bool) {
        self.is_break_overtime = val;
    }

    pub fn allow_early_ot(&self) -> bool {
        self.allow_early_ot
    }

    pub fn set_allow_early_ot(&mut self, val: bool) {
        self.allow_early_ot = val;
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct WorkingCondition {
    pub id: Uuid,
    pub employee_id: Uuid,
    pub type_: String,
    pub minute_allowance: i32,
    pub is_paid: bool,
    pub start_date: NaiveDate,
    pub end_date: Option<NaiveDate>,
}

#[derive(Debug, Serialize, Deserialize, Clone, utoipa::ToSchema)]
pub struct CreateWorkingConditionRequest {
    pub type_: String,
    pub minute_allowance: i32,
    pub is_paid: bool,
    pub start_date: NaiveDate,
    pub end_date: Option<NaiveDate>,
}

#[derive(Debug, Serialize, Deserialize, Clone, utoipa::ToSchema)]
pub struct Workspace {
    id: Uuid,
    name: String,
    date_format: String,
    time_format: String,
    timezone: String,
    theme_primary_color: String,
    default_language: String,
    logo_url: Option<String>,
    tagline: Option<String>,
    holiday_overnight_strategy: String,
    default_holiday_ot_multiplier: Decimal,
    created_at: DateTime<Utc>,
    #[serde(skip_serializing)]
    custom_db_url_encrypted: Option<String>,
}

impl Workspace {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: Uuid,
        name: String,
        date_format: String,
        time_format: String,
        timezone: String,
        theme_primary_color: String,
        default_language: String,
        logo_url: Option<String>,
        tagline: Option<String>,
        holiday_overnight_strategy: String,
        default_holiday_ot_multiplier: Decimal,
        created_at: DateTime<Utc>,
        custom_db_url_encrypted: Option<String>,
    ) -> Self {
        Self {
            id,
            name,
            date_format,
            time_format,
            timezone,
            theme_primary_color,
            default_language,
            logo_url,
            tagline,
            holiday_overnight_strategy,
            default_holiday_ot_multiplier,
            created_at,
            custom_db_url_encrypted,
        }
    }

    pub fn id(&self) -> Uuid {
        self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn date_format(&self) -> &str {
        &self.date_format
    }

    pub fn time_format(&self) -> &str {
        &self.time_format
    }

    pub fn timezone(&self) -> &str {
        &self.timezone
    }

    pub fn theme_primary_color(&self) -> &str {
        &self.theme_primary_color
    }

    pub fn default_language(&self) -> &str {
        &self.default_language
    }

    pub fn logo_url(&self) -> Option<&str> {
        self.logo_url.as_deref()
    }

    pub fn tagline(&self) -> Option<&str> {
        self.tagline.as_deref()
    }

    pub fn holiday_overnight_strategy(&self) -> &str {
        &self.holiday_overnight_strategy
    }

    pub fn default_holiday_ot_multiplier(&self) -> Decimal {
        self.default_holiday_ot_multiplier
    }

    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }

    pub fn custom_db_url_encrypted(&self) -> Option<&str> {
        self.custom_db_url_encrypted.as_deref()
    }

    #[cfg(test)]
    pub fn set_id(&mut self, id: Uuid) {
        self.id = id;
    }

    #[cfg(test)]
    pub fn set_name(&mut self, name: String) {
        self.name = name;
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, utoipa::ToSchema)]
pub struct CreateWorkspaceRequest {
    pub name: String,
    pub database_url: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, utoipa::ToSchema)]
pub struct UpdateWorkspaceRequest {
    pub name: Option<String>,
    pub date_format: Option<String>,
    pub time_format: Option<String>,
    pub timezone: Option<String>,
    pub theme_primary_color: Option<String>,
    pub default_language: Option<String>,
    pub logo_url: Option<String>,
    pub tagline: Option<String>,
    pub holiday_overnight_strategy: Option<String>,
    pub default_holiday_ot_multiplier: Option<Decimal>,
}

#[derive(Debug, Serialize, Deserialize, Clone, utoipa::ToSchema)]
pub struct WorkspaceMember {
    pub workspace_id: Uuid,
    pub user_id: Uuid,
    pub role_id: Uuid,
}

#[derive(Debug, Serialize, Deserialize, Clone, utoipa::ToSchema)]
pub struct AddMemberRequest {
    pub email: String,
    pub role_id: Uuid,
}

#[derive(Debug, Serialize, Deserialize, Clone, utoipa::ToSchema)]
pub struct MemberExtraRole {
    pub id: Uuid,
    pub workspace_id: Uuid,
    pub user_id: Uuid,
    pub role_id: Uuid,
    pub assigned_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone, utoipa::ToSchema)]
pub struct AssignRoleRequest {
    pub role_id: Uuid,
    pub expires_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone, utoipa::ToSchema)]
pub struct WorkspaceMemberDetail {
    pub workspace_id: Uuid,
    pub user_id: Uuid,
    pub role_id: Uuid,
    pub email: String,
    pub full_name: Option<String>,
    pub avatar_url: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, utoipa::ToSchema)]
pub struct WorkspaceMemberDetailResponse {
    #[serde(flatten)]
    pub detail: WorkspaceMemberDetail,
    pub extra_role_ids: Vec<Uuid>,
}

#[derive(Debug, Serialize, Deserialize, Clone, utoipa::ToSchema)]
pub struct MemberExtraPermission {
    pub id: Uuid,
    pub workspace_id: Uuid,
    pub user_id: Uuid,
    pub permission: String,
    pub assigned_at: Option<DateTime<Utc>>,
    pub expires_at: DateTime<Utc>,
    pub assigned_by: Option<Uuid>,
}

#[derive(Debug, Serialize, Deserialize, Clone, utoipa::ToSchema)]
pub struct AssignPermissionRequest {
    pub permission: String,
    pub expires_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone, utoipa::ToSchema)]
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

#[derive(Debug, Serialize, Deserialize, Clone, utoipa::ToSchema)]
pub struct WorkspaceRolePermission {
    pub role_id: Uuid,
    pub permission_name: String,
}
