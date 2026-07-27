use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::models::enums::{EarlyArrivalStrategy, RoleScopeType};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, utoipa::ToSchema)]
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
    id: Uuid,
    employment_id: Uuid,
    type_: String,
    condition_type_id: Option<Uuid>,
    minute_allowance: i32,
    is_paid: bool,
    start_date: NaiveDate,
    end_date: Option<NaiveDate>,
}

impl WorkingCondition {
    pub fn new(
        id: Uuid,
        employment_id: Uuid,
        type_: String,
        condition_type_id: Option<Uuid>,
        minute_allowance: i32,
        is_paid: bool,
        start_date: NaiveDate,
        end_date: Option<NaiveDate>,
    ) -> Self {
        Self {
            id,
            employment_id,
            type_,
            condition_type_id,
            minute_allowance,
            is_paid,
            start_date,
            end_date,
        }
    }

    pub fn id(&self) -> Uuid {
        self.id
    }

    pub fn employment_id(&self) -> Uuid {
        self.employment_id
    }

    pub fn type_(&self) -> &str {
        &self.type_
    }

    pub fn condition_type_id(&self) -> Option<Uuid> {
        self.condition_type_id
    }

    pub fn minute_allowance(&self) -> i32 {
        self.minute_allowance
    }

    pub fn is_paid(&self) -> bool {
        self.is_paid
    }

    pub fn start_date(&self) -> NaiveDate {
        self.start_date
    }

    pub fn end_date(&self) -> Option<NaiveDate> {
        self.end_date
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, utoipa::ToSchema)]
pub struct CreateWorkingConditionRequest {
    pub type_: String,
    pub condition_type_id: Option<Uuid>,
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
    workspace_id: Uuid,
    user_id: Uuid,
    role_id: Uuid,
}

impl WorkspaceMember {
    pub fn new(workspace_id: Uuid, user_id: Uuid, role_id: Uuid) -> Self {
        Self {
            workspace_id,
            user_id,
            role_id,
        }
    }

    pub fn workspace_id(&self) -> Uuid {
        self.workspace_id
    }

    pub fn user_id(&self) -> Uuid {
        self.user_id
    }

    pub fn role_id(&self) -> Uuid {
        self.role_id
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, utoipa::ToSchema)]
pub struct AddMemberRequest {
    pub email: String,
    pub role_id: Uuid,
}

#[derive(Debug, Serialize, Deserialize, Clone, utoipa::ToSchema)]
pub struct MemberExtraRole {
    id: Uuid,
    workspace_id: Uuid,
    user_id: Uuid,
    role_id: Uuid,
    assigned_at: DateTime<Utc>,
    expires_at: DateTime<Utc>,
}

impl MemberExtraRole {
    pub fn new(
        id: Uuid,
        workspace_id: Uuid,
        user_id: Uuid,
        role_id: Uuid,
        assigned_at: DateTime<Utc>,
        expires_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id,
            workspace_id,
            user_id,
            role_id,
            assigned_at,
            expires_at,
        }
    }

    pub fn id(&self) -> Uuid {
        self.id
    }

    pub fn workspace_id(&self) -> Uuid {
        self.workspace_id
    }

    pub fn user_id(&self) -> Uuid {
        self.user_id
    }

    pub fn role_id(&self) -> Uuid {
        self.role_id
    }

    pub fn assigned_at(&self) -> DateTime<Utc> {
        self.assigned_at
    }

    pub fn expires_at(&self) -> DateTime<Utc> {
        self.expires_at
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, utoipa::ToSchema)]
pub struct AssignRoleRequest {
    pub role_id: Uuid,
    pub expires_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone, utoipa::ToSchema)]
pub struct WorkspaceMemberDetail {
    workspace_id: Uuid,
    user_id: Uuid,
    role_id: Uuid,
    email: String,
    full_name: Option<String>,
    avatar_url: Option<String>,
}

impl WorkspaceMemberDetail {
    pub fn new(
        workspace_id: Uuid,
        user_id: Uuid,
        role_id: Uuid,
        email: String,
        full_name: Option<String>,
        avatar_url: Option<String>,
    ) -> Self {
        Self {
            workspace_id,
            user_id,
            role_id,
            email,
            full_name,
            avatar_url,
        }
    }

    pub fn workspace_id(&self) -> Uuid {
        self.workspace_id
    }

    pub fn user_id(&self) -> Uuid {
        self.user_id
    }

    pub fn role_id(&self) -> Uuid {
        self.role_id
    }

    pub fn email(&self) -> &str {
        &self.email
    }

    pub fn full_name(&self) -> Option<&str> {
        self.full_name.as_deref()
    }

    pub fn avatar_url(&self) -> Option<&str> {
        self.avatar_url.as_deref()
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, utoipa::ToSchema)]
pub struct WorkspaceMemberDetailResponse {
    #[serde(flatten)]
    pub detail: WorkspaceMemberDetail,
    pub extra_role_ids: Vec<Uuid>,
}

#[derive(Debug, Serialize, Deserialize, Clone, utoipa::ToSchema)]
pub struct MemberExtraPermission {
    id: Uuid,
    workspace_id: Uuid,
    user_id: Uuid,
    permission: String,
    assigned_at: Option<DateTime<Utc>>,
    expires_at: DateTime<Utc>,
    assigned_by: Option<Uuid>,
}

impl MemberExtraPermission {
    pub fn new(
        id: Uuid,
        workspace_id: Uuid,
        user_id: Uuid,
        permission: String,
        assigned_at: Option<DateTime<Utc>>,
        expires_at: DateTime<Utc>,
        assigned_by: Option<Uuid>,
    ) -> Self {
        Self {
            id,
            workspace_id,
            user_id,
            permission,
            assigned_at,
            expires_at,
            assigned_by,
        }
    }

    pub fn id(&self) -> Uuid {
        self.id
    }

    pub fn workspace_id(&self) -> Uuid {
        self.workspace_id
    }

    pub fn user_id(&self) -> Uuid {
        self.user_id
    }

    pub fn permission(&self) -> &str {
        &self.permission
    }

    pub fn assigned_at(&self) -> Option<DateTime<Utc>> {
        self.assigned_at
    }

    pub fn expires_at(&self) -> DateTime<Utc> {
        self.expires_at
    }

    pub fn assigned_by(&self) -> Option<Uuid> {
        self.assigned_by
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, utoipa::ToSchema)]
pub struct AssignPermissionRequest {
    pub permission: String,
    pub expires_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone, utoipa::ToSchema)]
pub struct WorkspaceRole {
    id: Uuid,
    workspace_id: Option<Uuid>,
    name: String,
    description: Option<String>,
    color: String,
    is_system: bool,
    scope_type: RoleScopeType,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl WorkspaceRole {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: Uuid,
        workspace_id: Option<Uuid>,
        name: String,
        description: Option<String>,
        color: String,
        is_system: bool,
        scope_type: RoleScopeType,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id,
            workspace_id,
            name,
            description,
            color,
            is_system,
            scope_type,
            created_at,
            updated_at,
        }
    }

    pub fn id(&self) -> Uuid {
        self.id
    }

    pub fn workspace_id(&self) -> Option<Uuid> {
        self.workspace_id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }

    pub fn color(&self) -> &str {
        &self.color
    }

    pub fn is_system(&self) -> bool {
        self.is_system
    }

    pub fn scope_type(&self) -> RoleScopeType {
        self.scope_type
    }

    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }

    pub fn updated_at(&self) -> DateTime<Utc> {
        self.updated_at
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, utoipa::ToSchema)]
pub struct WorkspaceRolePermission {
    role_id: Uuid,
    permission_name: String,
}

impl WorkspaceRolePermission {
    pub fn new(role_id: Uuid, permission_name: String) -> Self {
        Self {
            role_id,
            permission_name,
        }
    }

    pub fn role_id(&self) -> Uuid {
        self.role_id
    }

    pub fn permission_name(&self) -> &str {
        &self.permission_name
    }
}
