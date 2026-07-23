use super::*;
use crate::errors::AppError;
use crate::models::diesel::payroll::*;
use crate::models::{EmployeeSalaryComponent, SalaryComponentDefinition};
use chrono::{NaiveDate, Utc};
use diesel_async::{AsyncConnection, RunQueryDsl};
use std::str::FromStr;
use uuid::Uuid;

#[allow(clippy::too_many_lines, clippy::too_many_arguments)]
impl DieselPayrollRepository {
    pub(super) async fn get_salary_component_definitions(
        &self,
        workspace_id: Uuid,
    ) -> Result<Vec<SalaryComponentDefinition>, AppError> {
        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .tenants_tx
                .get_connection(&workspace_id)
                .await?,
        );

        let entities = salary_component_definitions::table
            .filter(salary_component_definitions::workspace_id.eq(workspace_id))
            .select(SalaryComponentDefinitionEntity::as_select())
            .load::<SalaryComponentDefinitionEntity>(&mut *conn)
            .await
            .map_err(AppError::from)?;

        Ok(entities.into_iter().map(Into::into).collect())
    }

    pub(super) async fn create_salary_component_definition(
        &self,
        workspace_id: Uuid,
        name: String,
        code: String,
        base_group_ids: Option<Vec<Uuid>>,
    ) -> Result<SalaryComponentDefinition, AppError> {
        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .tenants_tx
                .get_connection(&workspace_id)
                .await?,
        );

        let entity = conn
            .transaction::<_, AppError, _>(|conn| {
                Box::pin(async move {
                    let entity = SalaryComponentDefinitionEntity {
                        id: Uuid::new_v4(),
                        workspace_id,
                        name,
                        code,
                        created_at: Some(Utc::now()),
                        updated_at: Some(Utc::now()),
                        component_type: "earning".to_string(),
                        linked_rule_id: None,
                    };

                    let result = diesel::insert_into(salary_component_definitions::table)
                        .values(&entity)
                        .get_result::<SalaryComponentDefinitionEntity>(&mut *conn)
                        .await
                        .map_err(AppError::from)?;

                    if let Some(base_group_ids) = base_group_ids {
                        if !base_group_ids.is_empty() {
                            let valid_groups: Vec<Uuid> = salary_base_groups::table
                                .filter(salary_base_groups::id.eq_any(&base_group_ids))
                                .filter(
                                    salary_base_groups::workspace_id
                                        .eq(workspace_id)
                                        .or(salary_base_groups::workspace_id.is_null()),
                                )
                                .select(salary_base_groups::id)
                                .load(&mut *conn)
                                .await
                                .map_err(|e| {
                                    AppError::Internal(format!("Failed to validate groups: {e}"))
                                })?;

                            if valid_groups.len() != base_group_ids.len() {
                                return Err(AppError::forbidden(
                                    crate::utils::i18n::I18nKey::Forbidden,
                                ));
                            }

                            let new_members: Vec<SalaryBaseGroupMemberRow> = base_group_ids
                                .into_iter()
                                .map(|group_id| SalaryBaseGroupMemberRow {
                                    component_definition_id: result.id,
                                    base_group_id: group_id,
                                    created_at: Utc::now(),
                                })
                                .collect();

                            diesel::insert_into(salary_component_base_members::table)
                                .values(&new_members)
                                .on_conflict((
                                    salary_component_base_members::component_definition_id,
                                    salary_component_base_members::base_group_id,
                                ))
                                .do_nothing()
                                .execute(&mut *conn)
                                .await
                                .map_err(AppError::from)?;
                        }
                    }

                    Ok(result)
                })
            })
            .await?;

        Ok(entity.into())
    }

    pub(super) async fn assign_employment_salary_component(
        &self,
        workspace_id: Uuid,
        employment_id: Uuid,
        component_definition_id: Uuid,
        amount: Decimal,
        effective_date: NaiveDate,
    ) -> Result<EmployeeSalaryComponent, AppError> {
        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .tenants_tx
                .get_connection(&workspace_id)
                .await?,
        );

        let entity = EmployeeSalaryComponentEntity {
            id: Uuid::new_v4(),
            employment_id,
            component_definition_id,
            amount: bigdecimal::BigDecimal::from_str(&amount.to_string()).unwrap_or_default(),
            effective_date,
            created_at: Some(Utc::now()),
            updated_at: Some(Utc::now()),
        };

        let result = diesel::insert_into(employee_salary_components::table)
            .values(&entity)
            .get_result::<EmployeeSalaryComponentEntity>(&mut *conn)
            .await
            .map_err(AppError::from)?;

        Ok(result.into())
    }

    pub(super) async fn get_employment_salary_components(
        &self,
        workspace_id: Uuid,
        employment_id: Uuid,
        reference_date: NaiveDate,
    ) -> Result<Vec<(EmployeeSalaryComponent, SalaryComponentDefinition)>, AppError> {
        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .tenants_tx
                .get_connection(&workspace_id)
                .await?,
        );

        let rows = employee_salary_components::table
            .inner_join(salary_component_definitions::table)
            .filter(employee_salary_components::employment_id.eq(employment_id))
            .filter(employee_salary_components::effective_date.le(reference_date))
            .filter(salary_component_definitions::workspace_id.eq(workspace_id))
            .distinct_on((
                employee_salary_components::employment_id,
                employee_salary_components::component_definition_id,
            ))
            .order((
                employee_salary_components::employment_id,
                employee_salary_components::component_definition_id,
                employee_salary_components::effective_date.desc(),
                employee_salary_components::created_at.desc(),
            ))
            .load::<(
                EmployeeSalaryComponentEntity,
                SalaryComponentDefinitionEntity,
            )>(&mut *conn)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(rows
            .into_iter()
            .map(|(esc, scd)| (esc.into(), scd.into()))
            .collect())
    }

    pub(super) async fn get_salary_components_history_for_employments(
        &self,
        workspace_id: Uuid,
        employment_ids: &[Uuid],
    ) -> Result<Vec<(EmployeeSalaryComponent, SalaryComponentDefinition)>, AppError> {
        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .tenants_tx
                .get_connection(&workspace_id)
                .await?,
        );

        let rows = employee_salary_components::table
            .inner_join(salary_component_definitions::table)
            .filter(employee_salary_components::employment_id.eq_any(employment_ids))
            .filter(salary_component_definitions::workspace_id.eq(workspace_id))
            .order((
                employee_salary_components::employment_id,
                employee_salary_components::effective_date.asc(),
                employee_salary_components::created_at.asc(),
            ))
            .load::<(
                EmployeeSalaryComponentEntity,
                SalaryComponentDefinitionEntity,
            )>(&mut *conn)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(rows
            .into_iter()
            .map(|(esc, scd)| (esc.into(), scd.into()))
            .collect())
    }

    pub(super) async fn get_salary_components_history(
        &self,
        workspace_id: Uuid,
        employment_id: Uuid,
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> Result<Vec<(EmployeeSalaryComponent, SalaryComponentDefinition)>, AppError> {
        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .tenants_tx
                .get_connection(&workspace_id)
                .await?,
        );

        // Last component before start_date
        let last_before = employee_salary_components::table
            .inner_join(salary_component_definitions::table)
            .filter(employee_salary_components::employment_id.eq(employment_id))
            .filter(employee_salary_components::effective_date.lt(start_date))
            .filter(salary_component_definitions::workspace_id.eq(workspace_id))
            .distinct_on((
                employee_salary_components::employment_id,
                employee_salary_components::component_definition_id,
            ))
            .order((
                employee_salary_components::employment_id,
                employee_salary_components::component_definition_id,
                employee_salary_components::effective_date.desc(),
                employee_salary_components::created_at.desc(),
            ))
            .load::<(
                EmployeeSalaryComponentEntity,
                SalaryComponentDefinitionEntity,
            )>(&mut *conn)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        // Components within range
        let within_range = employee_salary_components::table
            .inner_join(salary_component_definitions::table)
            .filter(employee_salary_components::employment_id.eq(employment_id))
            .filter(employee_salary_components::effective_date.between(start_date, end_date))
            .filter(salary_component_definitions::workspace_id.eq(workspace_id))
            .order(employee_salary_components::effective_date.asc())
            .load::<(
                EmployeeSalaryComponentEntity,
                SalaryComponentDefinitionEntity,
            )>(&mut *conn)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        let mut all = last_before;
        all.extend(within_range);
        all.sort_by_key(|(esc, _)| esc.effective_date);

        Ok(all
            .into_iter()
            .map(|(esc, scd)| (esc.into(), scd.into()))
            .collect())
    }

    pub(super) async fn get_salary_components_history_batch(
        &self,
        workspace_id: Uuid,
        employment_ids: &[Uuid],
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> Result<
        std::collections::HashMap<Uuid, Vec<(EmployeeSalaryComponent, SalaryComponentDefinition)>>,
        AppError,
    > {
        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .tenants_tx
                .get_connection(&workspace_id)
                .await?,
        );

        // Fetch all components for these employments up to end_date
        let rows = employee_salary_components::table
            .inner_join(salary_component_definitions::table)
            .filter(employee_salary_components::employment_id.eq_any(employment_ids))
            .filter(employee_salary_components::effective_date.le(end_date))
            .filter(salary_component_definitions::workspace_id.eq(workspace_id))
            .order((
                employee_salary_components::employment_id,
                employee_salary_components::component_definition_id,
                employee_salary_components::effective_date.desc(),
                employee_salary_components::created_at.desc(),
            ))
            .load::<(
                EmployeeSalaryComponentEntity,
                SalaryComponentDefinitionEntity,
            )>(&mut *conn)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        let mut map: std::collections::HashMap<
            Uuid,
            Vec<(EmployeeSalaryComponent, SalaryComponentDefinition)>,
        > = std::collections::HashMap::new();

        // We need to keep only the latest before start_date, and all within range.
        // Group by employment and component id
        let mut employment_components: std::collections::HashMap<
            Uuid,
            std::collections::HashMap<
                Uuid,
                Vec<(
                    EmployeeSalaryComponentEntity,
                    SalaryComponentDefinitionEntity,
                )>,
            >,
        > = std::collections::HashMap::new();

        for row in rows {
            let emp_id = row.0.employment_id;
            let comp_id = row.0.component_definition_id;
            employment_components
                .entry(emp_id)
                .or_default()
                .entry(comp_id)
                .or_default()
                .push(row);
        }

        for (emp_id, comp_map) in employment_components {
            let mut emp_history: Vec<(
                crate::models::EmployeeSalaryComponent,
                crate::models::SalaryComponentDefinition,
            )> = Vec::new();
            for (_, history) in comp_map {
                // history is sorted desc by effective_date
                // Find all within range
                let mut within_range = Vec::new();
                let mut last_before = None;

                for item in history {
                    if item.0.effective_date >= start_date {
                        within_range.push(item);
                    } else if last_before.is_none() {
                        last_before = Some(item);
                    }
                }

                if let Some(item) = last_before {
                    emp_history.push((item.0.into(), item.1.into()));
                }
                for item in within_range {
                    emp_history.push((item.0.into(), item.1.into()));
                }
            }
            emp_history.sort_by_key(|(esc, _)| esc.effective_date);
            map.insert(emp_id, emp_history);
        }

        Ok(map)
    }
}
