use crate::errors::AppError;
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use uuid::Uuid;

use crate::domain::policy::cascade::base::BaseRules;
use crate::domain::policy::cascade::partial::PartialPolicyRules;
use crate::models::AttendancePolicyVersion;

#[cfg_attr(test, mockall::automock)]
#[async_trait::async_trait]
pub trait AttendancePolicyRepository: Send + Sync {
    async fn get_employee_effective_policy(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        employee_id: Uuid,
        effective_time: DateTime<Utc>,
    ) -> Result<Option<AttendancePolicyVersion>, AppError>;

    async fn get_employee_effective_policies_batch(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        employee_ids: &[Uuid],
        effective_time: DateTime<Utc>,
    ) -> Result<std::collections::HashMap<Uuid, AttendancePolicyVersion>, AppError>;

    async fn get_employment_effective_policy(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        employment_id: Uuid,
        effective_time: DateTime<Utc>,
    ) -> Result<Option<AttendancePolicyVersion>, AppError>;

    async fn get_employment_effective_policies_batch(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        employment_ids: &[Uuid],
        effective_time: DateTime<Utc>,
    ) -> Result<std::collections::HashMap<Uuid, AttendancePolicyVersion>, AppError>;

    async fn get_default_policy_rules(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
    ) -> Result<Option<crate::models::AttendanceRules>, AppError>;

    async fn get_default_policy_versions(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
    ) -> Result<Vec<AttendancePolicyVersion>, AppError>;

    async fn update_default_policy_rules(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        user_id: Uuid,
        req: crate::handlers::attendance::policies::UpdateAttendanceRulesRequest,
    ) -> Result<AttendancePolicyVersion, AppError>;

    async fn list_policy_assignments(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
    ) -> Result<
        Vec<crate::handlers::attendance::policy_assignments::PolicyAssignmentResponse>,
        AppError,
    >;

    async fn upsert_policy_assignment(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        user_id: Uuid,
        req: crate::handlers::attendance::policy_assignments::UpsertPolicyAssignmentRequest,
    ) -> Result<crate::handlers::attendance::policy_assignments::PolicyAssignmentResponse, AppError>;

    async fn delete_policy_assignment(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        assignment_id: Uuid,
    ) -> Result<(), AppError>;

    /// Load `base_rules` JSONB for a policy version and deserialize into `BaseRules`.
    /// Returns `None` when the stored JSONB is empty (`{}`) — caller should fall
    /// back to `BaseRules::default()`.
    async fn get_policy_version_base_rules(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        policy_version_id: Uuid,
    ) -> Result<Option<BaseRules>, AppError>;

    /// Load L2a day_type overrides for a policy version.
    /// Returns a map from day_type key → PartialPolicyRules contributions.
    async fn get_day_type_overrides_for_version(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        policy_version_id: Uuid,
    ) -> Result<HashMap<String, PartialPolicyRules>, AppError>;

    /// Load L2b shift_template overrides for a policy version.
    async fn get_shift_template_overrides_for_version(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        policy_version_id: Uuid,
    ) -> Result<HashMap<Uuid, PartialPolicyRules>, AppError>;

    /// Load L2c shift_version overrides for a policy version.
    async fn get_shift_version_overrides_for_version(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        policy_version_id: Uuid,
    ) -> Result<HashMap<Uuid, PartialPolicyRules>, AppError>;
}
