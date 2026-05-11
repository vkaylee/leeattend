use crate::diesel_pool::DieselPools;
use crate::models::diesel::leave::*;
use crate::models::{
    LeavePolicyAssignment, LeavePolicyAssignmentDetail, LeavePolicyConfig, LeavePolicyDefinition,
};
use crate::repositories::leave_policy_repository::{
    create_policy_assignment, create_policy_definition, delete_policy_definition,
    find_active_assignment_for_employee, get_policy_assignments, update_policy_definition,
    LeavePolicyRepository,
};
use crate::repositories::PoolBacked;
use crate::schema::*;
use async_trait::async_trait;
use chrono::NaiveDate;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use sqlx::PgPool;
use uuid::Uuid;

pub struct DieselLeavePolicyRepository {
    pub sqlx_pool: PgPool,
    pub diesel_pools: DieselPools,
}

impl DieselLeavePolicyRepository {
    pub fn new(sqlx_pool: PgPool, diesel_pools: DieselPools) -> Self {
        Self {
            sqlx_pool,
            diesel_pools,
        }
    }
}

impl PoolBacked for DieselLeavePolicyRepository {
    #[allow(clippy::panic)]
    #[allow(clippy::panic)]
    #[allow(clippy::panic)]
    fn from_pool(_pool: PgPool) -> Self {
        panic!(
            "DieselLeavePolicyRepository requires diesel_pools to be manually injected via new()"
        );
    }

    fn pool(&self) -> &PgPool {
        &self.sqlx_pool
    }
}

#[async_trait]
impl LeavePolicyRepository for DieselLeavePolicyRepository {
    async fn create_policy_definition(
        &self,
        workspace_id: Uuid,
        name: &str,
        description: Option<&str>,
        leave_type_id: Uuid,
        rule_config: LeavePolicyConfig,
    ) -> Result<LeavePolicyDefinition, sqlx::Error> {
        create_policy_definition(
            &self.sqlx_pool,
            workspace_id,
            name,
            description,
            leave_type_id,
            rule_config,
        )
        .await
    }

    async fn get_policy_definitions(
        &self,
        workspace_id: Uuid,
    ) -> Result<Vec<LeavePolicyDefinition>, sqlx::Error> {
        let mut conn = self
            .diesel_pools
            .api
            .get()
            .await
            .map_err(|e| sqlx::Error::Configuration(Box::new(e)))?;

        let entities = leave_policy_definitions::table
            .filter(leave_policy_definitions::workspace_id.eq(workspace_id))
            .order(leave_policy_definitions::created_at.desc())
            .load::<LeavePolicyDefinitionEntity>(&mut conn)
            .await
            .map_err(|e| sqlx::Error::Configuration(Box::new(e)))?;

        Ok(entities.into_iter().map(Into::into).collect())
    }

    async fn update_policy_definition(
        &self,
        id: Uuid,
        workspace_id: Uuid,
        name: Option<&str>,
        description: Option<&str>,
        leave_type_id: Option<Uuid>,
        rule_config: Option<LeavePolicyConfig>,
    ) -> Result<LeavePolicyDefinition, sqlx::Error> {
        update_policy_definition(
            &self.sqlx_pool,
            id,
            workspace_id,
            name,
            description,
            leave_type_id,
            rule_config,
        )
        .await
    }

    async fn delete_policy_definition(
        &self,
        id: Uuid,
        workspace_id: Uuid,
    ) -> Result<u64, sqlx::Error> {
        delete_policy_definition(&self.sqlx_pool, id, workspace_id).await
    }

    async fn create_policy_assignment(
        &self,
        assignment: &LeavePolicyAssignment,
    ) -> Result<LeavePolicyAssignment, sqlx::Error> {
        create_policy_assignment(&self.sqlx_pool, assignment).await
    }

    async fn find_active_assignment_for_employee(
        &self,
        workspace_id: Uuid,
        employee_id: Uuid,
        department_id: Option<Uuid>,
        date: NaiveDate,
    ) -> Result<Option<(LeavePolicyDefinition, Uuid, String)>, sqlx::Error> {
        find_active_assignment_for_employee(
            &self.sqlx_pool,
            workspace_id,
            employee_id,
            department_id,
            date,
        )
        .await
    }

    async fn get_policy_assignments(
        &self,
        workspace_id: Uuid,
    ) -> Result<Vec<LeavePolicyAssignmentDetail>, sqlx::Error> {
        get_policy_assignments(&self.sqlx_pool, workspace_id).await
    }
}
