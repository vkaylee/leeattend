use async_trait::async_trait;
use chrono::Utc;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use uuid::Uuid;

use crate::diesel_pool::{DieselPools, DieselPoolsTenantTxExt};
use crate::errors::AppError;
use crate::models::diesel::assignment_rule::AssignmentRuleEntity;
use crate::models::employee::{
    AssignmentRule, CreateAssignmentRuleRequest, UpdateAssignmentRuleRequest,
};
use crate::repositories::assignment_rule_repository::AssignmentRuleRepository;
use crate::schema::assignment_rules;

pub struct DieselAssignmentRuleRepository {
    diesel_pools: DieselPools,
}

impl DieselAssignmentRuleRepository {
    pub fn new(diesel_pools: DieselPools) -> Self {
        Self { diesel_pools }
    }
}

#[async_trait]
impl AssignmentRuleRepository for DieselAssignmentRuleRepository {
    async fn create_rule(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        req: &CreateAssignmentRuleRequest,
    ) -> Result<AssignmentRule, AppError> {
        let workspace_id = auth_token.workspace_id();
        let conditions = serde_json::to_value(&req.conditions)
            .map_err(|e| AppError::Internal(format!("serde: {e}")))?;
        let now = Utc::now();
        let entity = AssignmentRuleEntity {
            id: Uuid::new_v4(),
            workspace_id,
            pattern_id: req.pattern_id,
            name: req.name.clone(),
            priority: req.priority,
            conditions,
            is_active: true,
            created_at: Some(now),
            updated_at: Some(now),
            start_date: req.start_date,
            end_date: req.end_date,
        };

        self.diesel_pools
            .with_tenant_tx(auth_token, |conn| {
                Box::pin(async move {
                    diesel::insert_into(assignment_rules::table)
                        .values(&entity)
                        .returning(AssignmentRuleEntity::as_returning())
                        .get_result(conn)
                        .await
                        .map_err(AppError::from)
                        .map(Into::into)
                })
            })
            .await
    }

    async fn get_rules(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
    ) -> Result<Vec<AssignmentRule>, AppError> {
        let workspace_id = auth_token.workspace_id();

        self.diesel_pools
            .with_tenant_tx(auth_token, |conn| {
                Box::pin(async move {
                    assignment_rules::table
                        .filter(assignment_rules::workspace_id.eq(workspace_id))
                        .order((
                            assignment_rules::priority.desc(),
                            assignment_rules::created_at.asc(),
                        ))
                        .select(AssignmentRuleEntity::as_select())
                        .load::<AssignmentRuleEntity>(conn)
                        .await
                        .map_err(AppError::from)
                        .map(|rows| rows.into_iter().map(Into::into).collect())
                })
            })
            .await
    }

    async fn get_active_rules(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
    ) -> Result<Vec<AssignmentRule>, AppError> {
        let workspace_id = auth_token.workspace_id();

        self.diesel_pools
            .with_tenant_tx(auth_token, |conn| {
                Box::pin(async move {
                    assignment_rules::table
                        .filter(assignment_rules::workspace_id.eq(workspace_id))
                        .filter(assignment_rules::is_active.eq(true))
                        .order((
                            assignment_rules::priority.desc(),
                            assignment_rules::created_at.asc(),
                        ))
                        .select(AssignmentRuleEntity::as_select())
                        .load::<AssignmentRuleEntity>(conn)
                        .await
                        .map_err(AppError::from)
                        .map(|rows| rows.into_iter().map(Into::into).collect())
                })
            })
            .await
    }

    async fn update_rule(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        id: Uuid,
        req: &UpdateAssignmentRuleRequest,
    ) -> Result<AssignmentRule, AppError> {
        let workspace_id = auth_token.workspace_id();
        let conditions_json = req
            .conditions
            .as_ref()
            .map(|c| serde_json::to_value(c).map_err(|e| AppError::Internal(format!("serde: {e}"))))
            .transpose()?;

        self.diesel_pools
            .with_tenant_tx(auth_token, |conn| {
                Box::pin(async move {
                    let target = assignment_rules::table
                        .filter(assignment_rules::id.eq(id))
                        .filter(assignment_rules::workspace_id.eq(workspace_id));

                    diesel::update(target)
                        .set((
                            req.pattern_id.map(|v| assignment_rules::pattern_id.eq(v)),
                            req.name
                                .as_ref()
                                .map(|v| assignment_rules::name.eq(v.clone())),
                            req.priority.map(|v| assignment_rules::priority.eq(v)),
                            conditions_json.map(|v| assignment_rules::conditions.eq(v)),
                            req.is_active.map(|v| assignment_rules::is_active.eq(v)),
                            req.start_date.map(|v| assignment_rules::start_date.eq(v)),
                            req.end_date.map(|v| assignment_rules::end_date.eq(v)),
                            assignment_rules::updated_at.eq(Some(Utc::now())),
                        ))
                        .returning(AssignmentRuleEntity::as_returning())
                        .get_result(conn)
                        .await
                        .map_err(AppError::from)
                        .map(Into::into)
                })
            })
            .await
    }

    async fn delete_rule(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        id: Uuid,
    ) -> Result<(), AppError> {
        let workspace_id = auth_token.workspace_id();

        self.diesel_pools
            .with_tenant_tx(auth_token, |conn| {
                Box::pin(async move {
                    diesel::delete(
                        assignment_rules::table
                            .filter(assignment_rules::id.eq(id))
                            .filter(assignment_rules::workspace_id.eq(workspace_id)),
                    )
                    .execute(conn)
                    .await
                    .map_err(AppError::from)?;

                    Ok(())
                })
            })
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diesel_pool::DieselPools;

    #[tokio::test]
    async fn test_repo_initialization() {
        let test_db_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
            "postgres://user:pass@localhost:5432/leeattend_test_db".to_string()
        });

        let pools = DieselPools::for_test(&test_db_url);
        let _repo = DieselAssignmentRuleRepository::new(pools);
        // DI connection test passes if it compiles
    }
}
