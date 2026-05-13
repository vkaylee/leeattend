use crate::errors::AppError;
use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::models::AttendancePolicyVersion;

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
}
