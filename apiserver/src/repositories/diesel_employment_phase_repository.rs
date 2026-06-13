use crate::diesel_pool::DieselPools;
use crate::errors::AppError;
use crate::models::diesel::employment_phase::{
    EmployeePhaseEntity, EmploymentPhaseDefinitionEntity,
};
use crate::models::{EmployeePhase, EmploymentPhaseDefinition};
use crate::schema::{employee_phases, employment_phase_definitions};
use crate::utils::i18n::I18nKey;
use chrono::{NaiveDate, Utc};
use diesel::prelude::*;
use diesel_async::{AsyncConnection, RunQueryDsl};
use uuid::Uuid;

pub struct DieselEmploymentPhaseRepository {
    diesel_pools: DieselPools,
}

impl DieselEmploymentPhaseRepository {
    pub fn new(diesel_pools: DieselPools) -> Self {
        Self { diesel_pools }
    }

    pub async fn create_definition(
        &self,
        workspace_id: Uuid,
        name: &str,
        description: Option<&str>,
        phase_type: &str,
        rule_config: &serde_json::Value,
        display_order: i32,
    ) -> Result<EmploymentPhaseDefinition, AppError> {
        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .tenants_tx
                .get_connection(&workspace_id)
                .await
                .map_err(|e| AppError::Internal(format!("db: {e}")))?,
        );

        let entity = EmploymentPhaseDefinitionEntity {
            id: crate::utils::id::new_id(),
            workspace_id,
            name: name.to_string(),
            description: description.map(ToString::to_string),
            phase_type: phase_type.to_string(),
            rule_config: rule_config.clone(),
            is_active: true,
            display_order,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let result = diesel::insert_into(employment_phase_definitions::table)
            .values(&entity)
            .get_result::<EmploymentPhaseDefinitionEntity>(&mut *conn)
            .await
            .map_err(|e| {
                if let diesel::result::Error::DatabaseError(
                    diesel::result::DatabaseErrorKind::UniqueViolation,
                    ref info,
                ) = e
                {
                    if info
                        .constraint_name()
                        .is_some_and(|c| c == "employment_phase_definitions_workspace_id_name_key")
                    {
                        return AppError::validation_raw(
                            "Phase definition with this name already exists in the workspace.",
                        );
                    }
                }
                AppError::Internal(format!("Failed to create employment phase definition: {e}"))
            })?;

        Ok(result.into())
    }

    pub async fn get_definitions(
        &self,
        workspace_id: Uuid,
    ) -> Result<Vec<EmploymentPhaseDefinition>, AppError> {
        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .tenants_tx
                .get_connection(&workspace_id)
                .await
                .map_err(|e| AppError::Internal(format!("db: {e}")))?,
        );

        let entities = employment_phase_definitions::table
            .filter(employment_phase_definitions::workspace_id.eq(workspace_id))
            .order((
                employment_phase_definitions::display_order.asc(),
                employment_phase_definitions::created_at.desc(),
            ))
            .load::<EmploymentPhaseDefinitionEntity>(&mut *conn)
            .await
            .map_err(|e| {
                AppError::Internal(format!("Failed to fetch employment phase definitions: {e}"))
            })?;

        Ok(entities.into_iter().map(Into::into).collect())
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn assign_phase(
        &self,
        workspace_id: Uuid,
        employment_id: Uuid,
        phase_definition_id: Uuid,
        start_date: NaiveDate,
        end_date: NaiveDate,
        override_config: Option<&serde_json::Value>,
    ) -> Result<EmployeePhase, AppError> {
        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .tenants_tx
                .get_connection(&workspace_id)
                .await
                .map_err(|e| AppError::Internal(format!("db: {e}")))?,
        );

        let auto_close_date = start_date
            .checked_sub_days(chrono::Days::new(1))
            .unwrap_or(start_date);

        let override_config_owned = override_config.cloned();
        let result = conn
            .transaction(|conn| {
                Box::pin(async move {
                    // Auto-close any active phases that started before or on the new start_date
                    diesel::update(
                        employee_phases::table.filter(
                            employee_phases::workspace_id
                                .eq(workspace_id)
                                .and(employee_phases::employment_id.eq(employment_id))
                                .and(employee_phases::status.eq("active"))
                                .and(employee_phases::start_date.le(start_date)),
                        ),
                    )
                    .set((
                        employee_phases::status.eq("completed"),
                        employee_phases::end_date.eq(diesel::dsl::sql::<diesel::sql_types::Date>(
                            &format!("LEAST(end_date, GREATEST(start_date, '{auto_close_date}'))"),
                        )),
                        employee_phases::completed_at.eq(Utc::now()),
                        employee_phases::updated_at.eq(Utc::now()),
                    ))
                    .execute(&mut *conn)
                    .await?;

                    // Insert new phase
                    let new_entity = EmployeePhaseEntity {
                        id: crate::utils::id::new_id(),
                        workspace_id,
                        employment_id,
                        phase_definition_id,
                        start_date,
                        end_date,
                        status: "active".to_string(),
                        override_config: override_config_owned,
                        evaluation_notes: None,
                        evaluated_by: None,
                        completed_at: None,
                        created_at: Utc::now(),
                        updated_at: Utc::now(),
                    };

                    let result = diesel::insert_into(employee_phases::table)
                        .values(&new_entity)
                        .get_result::<EmployeePhaseEntity>(&mut *conn)
                        .await?;

                    Ok::<_, diesel::result::Error>(result)
                })
            })
            .await
            .map_err(|e| AppError::Internal(format!("Failed to assign employment phase: {e}")))?;

        Ok(result.into())
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn update_phase_status(
        &self,
        workspace_id: Uuid,
        employment_id: Uuid,
        phase_id: Uuid,
        status: &str,
        evaluation_notes: Option<&str>,
        evaluated_by: Uuid,
    ) -> Result<EmployeePhase, AppError> {
        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .tenants_tx
                .get_connection(&workspace_id)
                .await
                .map_err(|e| AppError::Internal(format!("db: {e}")))?,
        );

        let result = diesel::update(
            employee_phases::table.filter(
                employee_phases::id
                    .eq(phase_id)
                    .and(employee_phases::workspace_id.eq(workspace_id))
                    .and(employee_phases::employment_id.eq(employment_id)),
            ),
        )
        .set((
            employee_phases::status.eq(status),
            employee_phases::evaluation_notes.eq(evaluation_notes),
            employee_phases::evaluated_by.eq(evaluated_by),
            employee_phases::completed_at.eq(Utc::now()),
            employee_phases::updated_at.eq(Utc::now()),
        ))
        .get_result::<EmployeePhaseEntity>(&mut *conn)
        .await
        .optional()
        .map_err(|e| {
            AppError::Internal(format!("Failed to update employment phase status: {e}"))
        })?;

        result
            .map(Into::into)
            .ok_or_else(|| AppError::not_found(I18nKey::EmployeePhaseNotFound))
    }

    pub async fn get_employment_phases(
        &self,
        workspace_id: Uuid,
        employment_id: Uuid,
    ) -> Result<Vec<EmployeePhase>, AppError> {
        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .tenants_tx
                .get_connection(&workspace_id)
                .await
                .map_err(|e| AppError::Internal(format!("db: {e}")))?,
        );

        let entities = employee_phases::table
            .filter(
                employee_phases::workspace_id
                    .eq(workspace_id)
                    .and(employee_phases::employment_id.eq(employment_id)),
            )
            .order((
                employee_phases::start_date.asc(),
                employee_phases::created_at.desc(),
            ))
            .load::<EmployeePhaseEntity>(&mut *conn)
            .await
            .map_err(|e| AppError::Internal(format!("Failed to fetch employment phases: {e}")))?;

        Ok(entities.into_iter().map(Into::into).collect())
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
        let _repo = DieselEmploymentPhaseRepository::new(pools);
        // DI connection test passes if it compiles
    }
}
