use crate::errors::AppError;
use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::models::AttendancePolicyVersion;

#[cfg_attr(test, mockall::automock)]
#[async_trait::async_trait]
pub trait AttendancePolicyRepository: Send + Sync {
    async fn get_employee_effective_policy(
        &self,
        workspace_id: Uuid,
        employee_id: Uuid,
        effective_time: DateTime<Utc>,
    ) -> Result<Option<AttendancePolicyVersion>, AppError>;

    async fn get_employee_effective_policies_batch(
        &self,
        workspace_id: Uuid,
        employee_ids: &[Uuid],
        effective_time: DateTime<Utc>,
    ) -> Result<std::collections::HashMap<Uuid, AttendancePolicyVersion>, AppError>;

    async fn get_default_policy_rules(
        &self,
        workspace_id: Uuid,
    ) -> Result<Option<crate::models::AttendanceRules>, AppError>;

    async fn get_default_policy_versions(
        &self,
        workspace_id: Uuid,
    ) -> Result<Vec<AttendancePolicyVersion>, AppError>;

    async fn update_default_policy_rules(
        &self,
        workspace_id: Uuid,
        user_id: Uuid,
        req: crate::handlers::attendance::policies::UpdateAttendanceRulesRequest,
    ) -> Result<AttendancePolicyVersion, AppError>;

    async fn list_policy_assignments(
        &self,
        workspace_id: Uuid,
    ) -> Result<
        Vec<crate::handlers::attendance::policy_assignments::PolicyAssignmentResponse>,
        AppError,
    >;

    async fn upsert_policy_assignment(
        &self,
        workspace_id: Uuid,
        user_id: Uuid,
        req: crate::handlers::attendance::policy_assignments::UpsertPolicyAssignmentRequest,
    ) -> Result<crate::handlers::attendance::policy_assignments::PolicyAssignmentResponse, AppError>;

    async fn delete_policy_assignment(
        &self,
        workspace_id: Uuid,
        assignment_id: Uuid,
    ) -> Result<(), AppError>;
}
