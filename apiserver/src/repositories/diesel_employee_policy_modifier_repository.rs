use crate::diesel_pool::{DieselPools, TenantAuthToken};
use crate::errors::AppError;
use crate::models::diesel::employee_policy_modifier::EmployeePolicyModifierEntity;
use crate::repositories::employee_policy_modifier_repository::EmployeePolicyModifierRepository;
use async_trait::async_trait;
use chrono::NaiveDate;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use uuid::Uuid;

#[derive(Clone)]
pub struct DieselEmployeePolicyModifierRepository {
    diesel_pools: DieselPools,
}

impl DieselEmployeePolicyModifierRepository {
    pub fn new(diesel_pools: DieselPools) -> Self {
        Self { diesel_pools }
    }
}

#[async_trait]
impl EmployeePolicyModifierRepository for DieselEmployeePolicyModifierRepository {
    async fn get_active_modifiers(
        &self,
        auth_token: &TenantAuthToken,
        employee_id: Uuid,
        date: NaiveDate,
    ) -> Result<Vec<EmployeePolicyModifierEntity>, AppError> {
        use crate::schema::employee_policy_modifiers::dsl as epm;

        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .tenants_tx
                .get_connection(&auth_token.workspace_id())
                .await?,
        );

        // effective_from <= (date end of day) AND (effective_to IS NULL OR effective_to > (date start))
        let day_start = date
            .and_hms_opt(0, 0, 0)
            .ok_or_else(|| AppError::Internal("invalid date start".into()))?;
        let day_end = date
            .succ_opt()
            .and_then(|next| next.and_hms_opt(0, 0, 0))
            .ok_or_else(|| AppError::Internal("invalid date end".into()))?;

        let modifiers = epm::employee_policy_modifiers
            .filter(epm::workspace_id.eq(auth_token.workspace_id()))
            .filter(epm::employee_id.eq(employee_id))
            .filter(epm::effective_from.le(day_end))
            .filter(
                epm::effective_to
                    .is_null()
                    .or(epm::effective_to.gt(day_start)),
            )
            .order(epm::priority_rank.desc())
            .select(EmployeePolicyModifierEntity::as_select())
            .load(&mut *conn)
            .await
            .map_err(|e| AppError::Internal(format!("fetch modifiers: {e}")))?;

        Ok(modifiers)
    }
}
