use async_trait::async_trait;
use chrono::Utc;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;

use crate::diesel_pool::{DieselPools, TenantAuthToken};
use crate::errors::AppError;
use crate::models::compliance::ComplianceRules;
use crate::models::diesel::compliance::{NewComplianceRule, WorkspaceComplianceRuleEntity};
use crate::repositories::compliance_repository::ComplianceRepository;
use crate::schema::workspace_compliance_rules;

pub struct DieselComplianceRepository {
    diesel_pools: DieselPools,
}

impl DieselComplianceRepository {
    pub fn new(diesel_pools: DieselPools) -> Self {
        Self { diesel_pools }
    }
}

#[async_trait]
impl ComplianceRepository for DieselComplianceRepository {
    async fn get_rules(&self, auth_token: &TenantAuthToken) -> Result<ComplianceRules, AppError> {
        let mut conn = self
            .diesel_pools
            .tenant_transaction_conn(auth_token)
            .await?;
        let workspace_id = auth_token.workspace_id();

        let entity: Option<WorkspaceComplianceRuleEntity> = workspace_compliance_rules::table
            .filter(workspace_compliance_rules::workspace_id.eq(workspace_id))
            .select(WorkspaceComplianceRuleEntity::as_select())
            .first(&mut *conn)
            .await
            .optional()
            .map_err(AppError::from)?;

        if let Some(e) = entity {
            Ok(e.into())
        } else {
            // Return defaults — no row exists yet for this workspace
            Ok(ComplianceRules {
                workspace_id,
                ..Default::default()
            })
        }
    }

    async fn upsert_rules(
        &self,
        auth_token: &TenantAuthToken,
        rules: &ComplianceRules,
    ) -> Result<ComplianceRules, AppError> {
        let mut conn = self
            .diesel_pools
            .tenant_transaction_conn(auth_token)
            .await?;
        let workspace_id = auth_token.workspace_id();

        let new_row = NewComplianceRule {
            workspace_id,
            min_rest_period_hours: rules.min_rest_period_hours,
            max_weekly_hours: rules.max_weekly_hours,
            max_monthly_ot_hours: rules.max_monthly_ot_hours,
            allow_manager_override: rules.allow_manager_override,
            is_enabled: rules.is_enabled,
        };

        diesel::insert_into(workspace_compliance_rules::table)
            .values(&new_row)
            .on_conflict(workspace_compliance_rules::workspace_id)
            .do_update()
            .set((
                workspace_compliance_rules::min_rest_period_hours.eq(rules.min_rest_period_hours),
                workspace_compliance_rules::max_weekly_hours.eq(rules.max_weekly_hours),
                workspace_compliance_rules::max_monthly_ot_hours.eq(rules.max_monthly_ot_hours),
                workspace_compliance_rules::allow_manager_override.eq(rules.allow_manager_override),
                workspace_compliance_rules::is_enabled.eq(rules.is_enabled),
                workspace_compliance_rules::updated_at.eq(Utc::now()),
            ))
            .returning(WorkspaceComplianceRuleEntity::as_returning())
            .get_result(&mut *conn)
            .await
            .map_err(AppError::from)
            .map(Into::into)
    }

    async fn is_enabled(&self, auth_token: &TenantAuthToken) -> Result<bool, AppError> {
        let mut conn = self
            .diesel_pools
            .tenant_transaction_conn(auth_token)
            .await?;
        let workspace_id = auth_token.workspace_id();

        let enabled = workspace_compliance_rules::table
            .filter(workspace_compliance_rules::workspace_id.eq(workspace_id))
            .select(workspace_compliance_rules::is_enabled)
            .first::<bool>(&mut *conn)
            .await
            .optional()
            .map_err(AppError::from)?;

        Ok(enabled.unwrap_or(true)) // default: enabled
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
        let _repo = DieselComplianceRepository::new(pools);
        // DI connection test passes if it compiles
    }
}
