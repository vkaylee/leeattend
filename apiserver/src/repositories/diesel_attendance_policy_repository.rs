use crate::repositories::PoolBacked;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::PgPool;
use std::collections::HashMap;
use uuid::Uuid;

use crate::models::AttendancePolicyVersion;
use crate::repositories::attendance_policy_repository::{
    AttendancePolicyRepository, SqlxAttendancePolicyRepository,
};

pub struct DieselAttendancePolicyRepository {
    pub sqlx_pool: PgPool,
    pub diesel_pools: crate::diesel_pool::DieselPools,
}

impl DieselAttendancePolicyRepository {
    pub fn new(sqlx_pool: PgPool, diesel_pools: crate::diesel_pool::DieselPools) -> Self {
        Self {
            sqlx_pool,
            diesel_pools,
        }
    }
}

#[async_trait]
impl AttendancePolicyRepository for DieselAttendancePolicyRepository {
    async fn get_employee_effective_policy(
        &self,
        workspace_id: Uuid,
        employee_id: Uuid,
        effective_time: DateTime<Utc>,
    ) -> Result<Option<AttendancePolicyVersion>, sqlx::Error> {
        // Complex CTE query - fallback to SQLx for now
        let repo = SqlxAttendancePolicyRepository::from_pool(self.sqlx_pool.clone());
        repo.get_employee_effective_policy(workspace_id, employee_id, effective_time)
            .await
    }

    async fn get_employee_effective_policies_batch(
        &self,
        workspace_id: Uuid,
        employee_ids: &[Uuid],
        effective_time: DateTime<Utc>,
    ) -> Result<HashMap<Uuid, AttendancePolicyVersion>, sqlx::Error> {
        // Complex CTE query - fallback to SQLx for now
        let repo = SqlxAttendancePolicyRepository::from_pool(self.sqlx_pool.clone());
        repo.get_employee_effective_policies_batch(workspace_id, employee_ids, effective_time)
            .await
    }
}

impl crate::repositories::PoolBacked for DieselAttendancePolicyRepository {
    #[allow(clippy::panic)]
    #[allow(clippy::panic)]
    #[allow(clippy::panic)]
    fn from_pool(_pool: PgPool) -> Self {
        panic!("DieselAttendancePolicyRepository requires diesel_pools to be manually injected via new()");
    }

    fn pool(&self) -> &PgPool {
        &self.sqlx_pool
    }
}
