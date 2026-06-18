use crate::diesel_pool::DieselPools;
use crate::errors::AppError;
use crate::models::diesel::leave::*;
use crate::models::{
    LeavePolicyAssignment, LeavePolicyAssignmentDetail, LeavePolicyConfig, LeavePolicyDefinition,
};
use crate::repositories::leave_policy_repository::LeavePolicyRepository;
use crate::utils::i18n::I18nKey;

use crate::schema::*;
use async_trait::async_trait;
use chrono::{NaiveDate, Utc};
use diesel::prelude::*;
use diesel_async::{AsyncConnection, RunQueryDsl};
use uuid::Uuid;

pub struct DieselLeavePolicyRepository {
    diesel_pools: DieselPools,
}

impl DieselLeavePolicyRepository {
    pub fn new(diesel_pools: DieselPools) -> Self {
        Self { diesel_pools }
    }
}

#[allow(clippy::too_many_lines)]
#[async_trait]
impl LeavePolicyRepository for DieselLeavePolicyRepository {
    async fn get_leave_types(
        &self,
        workspace_id: Uuid,
    ) -> Result<Vec<crate::models::LeaveType>, AppError> {
        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .tenants_tx
                .get_connection(&workspace_id)
                .await
                .map_err(|e| AppError::Internal(format!("db: {e}")))?,
        );

        let entities = leave_types::table
            .filter(leave_types::workspace_id.eq(workspace_id))
            .filter(leave_types::deleted_at.is_null())
            .order(leave_types::name.asc())
            .load::<LeaveTypeEntity>(&mut *conn)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(entities.into_iter().map(Into::into).collect())
    }

    async fn create_leave_type(
        &self,
        workspace_id: Uuid,
        code: String,
        name: String,
        is_paid: bool,
        policy_config: crate::models::LeaveTypeConfig,
    ) -> Result<crate::models::LeaveType, AppError> {
        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .tenants_tx
                .get_connection(&workspace_id)
                .await
                .map_err(|e| AppError::Internal(format!("db: {e}")))?,
        );

        let entity = LeaveTypeEntity {
            id: Uuid::new_v4(),
            workspace_id,
            code,
            name,
            is_paid,
            policy_config: serde_json::to_value(policy_config).unwrap_or_default(),
            created_at: Utc::now(),
            deleted_at: None,
        };

        let result = diesel::insert_into(leave_types::table)
            .values(&entity)
            .get_result::<LeaveTypeEntity>(&mut *conn)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(result.into())
    }

    async fn update_leave_type_config(
        &self,
        id: Uuid,
        workspace_id: Uuid,
        policy_config: crate::models::LeaveTypeConfig,
    ) -> Result<crate::models::LeaveType, AppError> {
        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .tenants_tx
                .get_connection(&workspace_id)
                .await
                .map_err(|e| AppError::Internal(format!("db: {e}")))?,
        );

        let result = diesel::update(
            leave_types::table
                .filter(leave_types::id.eq(id))
                .filter(leave_types::workspace_id.eq(workspace_id)),
        )
        .set(leave_types::policy_config.eq(serde_json::to_value(policy_config).unwrap_or_default()))
        .get_result::<LeaveTypeEntity>(&mut *conn)
        .await
        .map_err(|e| match e {
            diesel::result::Error::NotFound => AppError::not_found(I18nKey::LeaveTypeNotFound),
            _ => AppError::Internal(format!("db: {e}")),
        })?;

        Ok(result.into())
    }

    async fn delete_leave_type(&self, id: Uuid, workspace_id: Uuid) -> Result<u64, AppError> {
        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .tenants_tx
                .get_connection(&workspace_id)
                .await
                .map_err(|e| AppError::Internal(format!("db: {e}")))?,
        );

        // First check if there are any balances using this leave type
        use crate::schema::{employments, leave_balances};
        let in_balances: Option<i32> = leave_balances::table
            .inner_join(employments::table)
            .filter(leave_balances::leave_type_id.eq(id))
            .filter(employments::workspace_id.eq(workspace_id))
            .select(diesel::dsl::sql::<diesel::sql_types::Integer>("1"))
            .first(&mut *conn)
            .await
            .optional()
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        if in_balances.is_some() {
            return Err(AppError::conflict_args(
                I18nKey::Conflict,
                vec![("details", "leave_type_in_use_balances".to_string())],
            ));
        }

        let affected = diesel::update(
            leave_types::table
                .filter(leave_types::id.eq(id))
                .filter(leave_types::workspace_id.eq(workspace_id)),
        )
        .set(leave_types::deleted_at.eq(Utc::now()))
        .execute(&mut *conn)
        .await
        .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(affected as u64)
    }

    async fn create_policy_definition(
        &self,
        workspace_id: Uuid,
        name: &str,
        description: Option<&str>,
        leave_type_id: Uuid,
        rule_config: LeavePolicyConfig,
    ) -> Result<LeavePolicyDefinition, AppError> {
        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .tenants_tx
                .get_connection(&workspace_id)
                .await
                .map_err(|e| AppError::Internal(format!("db: {e}")))?,
        );

        let entity = LeavePolicyDefinitionEntity {
            id: Uuid::new_v4(),
            workspace_id,
            name: name.to_string(),
            description: description.map(ToString::to_string),
            leave_type_id,
            rule_config: serde_json::to_value(rule_config).unwrap_or_default(),
            created_at: Some(Utc::now()),
            updated_at: Some(Utc::now()),
            deleted_at: None,
        };

        let result = diesel::insert_into(leave_policy_definitions::table)
            .values(&entity)
            .get_result::<LeavePolicyDefinitionEntity>(&mut *conn)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(result.into())
    }

    async fn get_policy_definitions(
        &self,
        workspace_id: Uuid,
    ) -> Result<Vec<LeavePolicyDefinition>, AppError> {
        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .tenants_tx
                .get_connection(&workspace_id)
                .await
                .map_err(|e| AppError::Internal(format!("db: {e}")))?,
        );

        let entities = leave_policy_definitions::table
            .filter(leave_policy_definitions::workspace_id.eq(workspace_id))
            .filter(leave_policy_definitions::deleted_at.is_null())
            .order(leave_policy_definitions::created_at.desc())
            .load::<LeavePolicyDefinitionEntity>(&mut *conn)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

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
    ) -> Result<LeavePolicyDefinition, AppError> {
        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .tenants_tx
                .get_connection(&workspace_id)
                .await
                .map_err(|e| AppError::Internal(format!("db: {e}")))?,
        );

        let result = diesel::update(
            leave_policy_definitions::table
                .filter(leave_policy_definitions::id.eq(id))
                .filter(leave_policy_definitions::workspace_id.eq(workspace_id)),
        )
        .set((
            name.map(|s| leave_policy_definitions::name.eq(s.to_string())),
            description.map(|s| leave_policy_definitions::description.eq(s.to_string())),
            leave_type_id.map(|v| leave_policy_definitions::leave_type_id.eq(v)),
            rule_config.map(|v| {
                leave_policy_definitions::rule_config
                    .eq(serde_json::to_value(v).unwrap_or_default())
            }),
            leave_policy_definitions::updated_at.eq(Some(Utc::now())),
        ))
        .get_result::<LeavePolicyDefinitionEntity>(&mut *conn)
        .await
        .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(result.into())
    }

    async fn delete_policy_definition(
        &self,
        id: Uuid,
        workspace_id: Uuid,
    ) -> Result<u64, AppError> {
        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .tenants_tx
                .get_connection(&workspace_id)
                .await
                .map_err(|e| AppError::Internal(format!("db: {e}")))?,
        );

        conn.transaction(|conn| {
            Box::pin(async move {
                // 1. Soft delete assignments
                diesel::update(
                    leave_policy_assignments::table
                        .filter(leave_policy_assignments::policy_id.eq(id))
                        .filter(leave_policy_assignments::workspace_id.eq(workspace_id))
                        .filter(leave_policy_assignments::deleted_at.is_null()),
                )
                .set(leave_policy_assignments::deleted_at.eq(Utc::now()))
                .execute(&mut *conn)
                .await?;

                // 2. Soft delete definition
                let affected = diesel::update(
                    leave_policy_definitions::table
                        .filter(leave_policy_definitions::id.eq(id))
                        .filter(leave_policy_definitions::workspace_id.eq(workspace_id)),
                )
                .set(leave_policy_definitions::deleted_at.eq(Utc::now()))
                .execute(&mut *conn)
                .await?;

                Ok::<_, diesel::result::Error>(affected as u64)
            })
        })
        .await
        .map_err(|e| AppError::Internal(format!("db transaction: {e}")))
    }

    async fn create_policy_assignment(
        &self,
        workspace_id: Uuid,
        assignment: &LeavePolicyAssignment,
    ) -> Result<LeavePolicyAssignment, AppError> {
        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .tenants_tx
                .get_connection(&workspace_id)
                .await
                .map_err(|e| AppError::Internal(format!("db: {e}")))?,
        );

        let assignment_owned = assignment.clone();
        let result = conn
            .transaction(|conn| {
                Box::pin(async move {
                    // 1. Get leave_type_id
                    let lt_id: Uuid = leave_policy_definitions::table
                        .filter(leave_policy_definitions::id.eq(assignment_owned.policy_id()))
                        .filter(
                            leave_policy_definitions::workspace_id
                                .eq(assignment_owned.workspace_id()),
                        )
                        .filter(leave_policy_definitions::deleted_at.is_null())
                        .select(leave_policy_definitions::leave_type_id)
                        .first(&mut *conn)
                        .await?;

                    // 2a. Soft delete future assignments
                    diesel::update(
                        leave_policy_assignments::table
                            .filter(
                                leave_policy_assignments::workspace_id
                                    .eq(assignment_owned.workspace_id()),
                            )
                            .filter(
                                leave_policy_assignments::assignee_type
                                    .eq(assignment_owned.assignee_type().to_string()),
                            )
                            .filter(
                                leave_policy_assignments::assignee_id
                                    .is_not_distinct_from(assignment_owned.assignee_id()),
                            )
                            .filter(
                                leave_policy_assignments::policy_id.eq_any(
                                    leave_policy_definitions::table
                                        .filter(leave_policy_definitions::leave_type_id.eq(lt_id))
                                        .select(leave_policy_definitions::id),
                                ),
                            )
                            .filter(
                                leave_policy_assignments::effective_from
                                    .ge(assignment_owned.effective_from()),
                            )
                            .filter(leave_policy_assignments::deleted_at.is_null()),
                    )
                    .set(leave_policy_assignments::deleted_at.eq(Utc::now()))
                    .execute(&mut *conn)
                    .await?;

                    // 2. Auto-close existing
                    diesel::update(
                        leave_policy_assignments::table
                            .filter(
                                leave_policy_assignments::workspace_id
                                    .eq(assignment_owned.workspace_id()),
                            )
                            .filter(
                                leave_policy_assignments::assignee_type
                                    .eq(assignment_owned.assignee_type().to_string()),
                            )
                            .filter(
                                leave_policy_assignments::assignee_id
                                    .is_not_distinct_from(assignment_owned.assignee_id()),
                            )
                            .filter(
                                leave_policy_assignments::policy_id.eq_any(
                                    leave_policy_definitions::table
                                        .filter(leave_policy_definitions::leave_type_id.eq(lt_id))
                                        .select(leave_policy_definitions::id),
                                ),
                            )
                            .filter(
                                leave_policy_assignments::effective_from
                                    .lt(assignment_owned.effective_from()),
                            )
                            .filter(
                                leave_policy_assignments::effective_to
                                    .is_null()
                                    .or(leave_policy_assignments::effective_to
                                        .ge(assignment_owned.effective_from())),
                            )
                            .filter(leave_policy_assignments::deleted_at.is_null()),
                    )
                    .set(
                        leave_policy_assignments::effective_to
                            .eq(assignment_owned.effective_from().pred_opt()),
                    )
                    .execute(&mut *conn)
                    .await?;

                    // 3. Insert new
                    let entity = LeavePolicyAssignmentEntity {
                        id: Uuid::new_v4(),
                        workspace_id: assignment_owned.workspace_id(),
                        assignee_type: assignment_owned.assignee_type().to_string(),
                        assignee_id: assignment_owned.assignee_id(),
                        policy_id: assignment_owned.policy_id(),
                        effective_from: assignment_owned.effective_from(),
                        effective_to: assignment_owned.effective_to(),
                        created_at: Some(Utc::now()),
                        deleted_at: None,
                    };

                    let saved = diesel::insert_into(leave_policy_assignments::table)
                        .values(&entity)
                        .get_result::<LeavePolicyAssignmentEntity>(&mut *conn)
                        .await?;

                    Ok::<_, diesel::result::Error>(saved.into())
                })
            })
            .await
            .map_err(|e| AppError::Internal(format!("db transaction: {e}")))?;

        Ok(result)
    }

    async fn find_active_assignment_for_employee(
        &self,
        workspace_id: Uuid,
        employee_id: Uuid,
        department_id: Option<Uuid>,
        date: NaiveDate,
    ) -> Result<Option<(LeavePolicyDefinition, Uuid, String)>, AppError> {
        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .tenants_tx
                .get_connection(&workspace_id)
                .await
                .map_err(|e| AppError::Internal(format!("db: {e}")))?,
        );

        // Priority 1: Employee
        let employee_result = leave_policy_assignments::table
            .inner_join(leave_policy_definitions::table)
            .inner_join(
                leave_types::table.on(leave_policy_definitions::leave_type_id.eq(leave_types::id)),
            )
            .filter(leave_policy_assignments::workspace_id.eq(workspace_id))
            .filter(leave_policy_assignments::deleted_at.is_null())
            .filter(leave_policy_definitions::deleted_at.is_null())
            .filter(leave_types::deleted_at.is_null())
            .filter(leave_policy_assignments::assignee_type.eq("employee"))
            .filter(leave_policy_assignments::assignee_id.eq(employee_id))
            .filter(leave_policy_assignments::effective_from.le(date))
            .filter(
                leave_policy_assignments::effective_to
                    .is_null()
                    .or(leave_policy_assignments::effective_to.gt(date)),
            )
            .select((
                leave_policy_definitions::all_columns,
                leave_policy_assignments::id,
                leave_types::code,
            ))
            .first::<(LeavePolicyDefinitionEntity, Uuid, String)>(&mut *conn)
            .await
            .optional()
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        if let Some((def, assignment_id, code)) = employee_result {
            return Ok(Some((def.into(), assignment_id, code)));
        }

        // Priority 2: Department
        if let Some(dept_id) = department_id {
            let dept_result = leave_policy_assignments::table
                .inner_join(leave_policy_definitions::table)
                .inner_join(
                    leave_types::table
                        .on(leave_policy_definitions::leave_type_id.eq(leave_types::id)),
                )
                .filter(leave_policy_assignments::workspace_id.eq(workspace_id))
                .filter(leave_policy_assignments::deleted_at.is_null())
                .filter(leave_policy_definitions::deleted_at.is_null())
                .filter(leave_types::deleted_at.is_null())
                .filter(leave_policy_assignments::assignee_type.eq("department"))
                .filter(leave_policy_assignments::assignee_id.eq(dept_id))
                .filter(leave_policy_assignments::effective_from.le(date))
                .filter(
                    leave_policy_assignments::effective_to
                        .is_null()
                        .or(leave_policy_assignments::effective_to.gt(date)),
                )
                .select((
                    leave_policy_definitions::all_columns,
                    leave_policy_assignments::id,
                    leave_types::code,
                ))
                .first::<(LeavePolicyDefinitionEntity, Uuid, String)>(&mut *conn)
                .await
                .optional()
                .map_err(|e| AppError::Internal(format!("db: {e}")))?;

            if let Some((def, assignment_id, code)) = dept_result {
                return Ok(Some((def.into(), assignment_id, code)));
            }
        }

        // Priority 3: Workspace
        let workspace_result = leave_policy_assignments::table
            .inner_join(leave_policy_definitions::table)
            .inner_join(
                leave_types::table.on(leave_policy_definitions::leave_type_id.eq(leave_types::id)),
            )
            .filter(leave_policy_assignments::workspace_id.eq(workspace_id))
            .filter(leave_policy_assignments::deleted_at.is_null())
            .filter(leave_policy_definitions::deleted_at.is_null())
            .filter(leave_types::deleted_at.is_null())
            .filter(leave_policy_assignments::assignee_type.eq("workspace"))
            .filter(leave_policy_assignments::effective_from.le(date))
            .filter(
                leave_policy_assignments::effective_to
                    .is_null()
                    .or(leave_policy_assignments::effective_to.gt(date)),
            )
            .select((
                leave_policy_definitions::all_columns,
                leave_policy_assignments::id,
                leave_types::code,
            ))
            .first::<(LeavePolicyDefinitionEntity, Uuid, String)>(&mut *conn)
            .await
            .optional()
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        if let Some((def, assignment_id, code)) = workspace_result {
            return Ok(Some((def.into(), assignment_id, code)));
        }

        Ok(None)
    }

    async fn find_all_active_assignments_for_workspace(
        &self,
        workspace_id: Uuid,
        date: NaiveDate,
    ) -> Result<Vec<(LeavePolicyAssignment, LeavePolicyDefinition, String)>, AppError> {
        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .tenants_tx
                .get_connection(&workspace_id)
                .await
                .map_err(|e| AppError::Internal(format!("db: {e}")))?,
        );

        let results = leave_policy_assignments::table
            .inner_join(leave_policy_definitions::table)
            .inner_join(
                leave_types::table.on(leave_policy_definitions::leave_type_id.eq(leave_types::id)),
            )
            .filter(leave_policy_assignments::workspace_id.eq(workspace_id))
            .filter(leave_policy_assignments::deleted_at.is_null())
            .filter(leave_policy_definitions::deleted_at.is_null())
            .filter(leave_types::deleted_at.is_null())
            .filter(leave_policy_assignments::effective_from.le(date))
            .filter(
                leave_policy_assignments::effective_to
                    .is_null()
                    .or(leave_policy_assignments::effective_to.gt(date)),
            )
            .select((
                leave_policy_assignments::all_columns,
                leave_policy_definitions::all_columns,
                leave_types::code,
            ))
            .load::<(
                LeavePolicyAssignmentEntity,
                LeavePolicyDefinitionEntity,
                String,
            )>(&mut *conn)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(results
            .into_iter()
            .map(|(assign, def, code)| (assign.into(), def.into(), code))
            .collect())
    }

    async fn get_policy_assignments(
        &self,
        workspace_id: Uuid,
    ) -> Result<Vec<LeavePolicyAssignmentDetail>, AppError> {
        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .tenants_tx
                .get_connection(&workspace_id)
                .await
                .map_err(|e| AppError::Internal(format!("db: {e}")))?,
        );

        let results = leave_policy_assignments::table
            .inner_join(leave_policy_definitions::table)
            .inner_join(
                leave_types::table.on(leave_policy_definitions::leave_type_id.eq(leave_types::id)),
            )
            .filter(leave_policy_assignments::workspace_id.eq(workspace_id))
            .filter(leave_policy_assignments::deleted_at.is_null())
            .filter(leave_policy_definitions::deleted_at.is_null())
            .filter(leave_types::deleted_at.is_null())
            .order(leave_policy_assignments::created_at.desc())
            .select((
                leave_policy_assignments::all_columns,
                leave_policy_definitions::name,
                leave_types::name,
            ))
            .load::<(LeavePolicyAssignmentEntity, String, String)>(&mut *conn)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(results
            .into_iter()
            .map(|(a, p_name, lt_name)| {
                LeavePolicyAssignmentDetail::new(
                    a.id,
                    a.workspace_id,
                    a.assignee_type,
                    a.assignee_id,
                    a.policy_id,
                    a.effective_from,
                    a.effective_to,
                    a.created_at.unwrap_or_else(Utc::now),
                    p_name,
                    lt_name,
                )
            })
            .collect())
    }

    async fn delete_policy_assignment(&self, workspace_id: Uuid, id: Uuid) -> Result<(), AppError> {
        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .tenants_tx
                .get_connection(&workspace_id)
                .await
                .map_err(|e| AppError::Internal(format!("db: {e}")))?,
        );

        conn.transaction(|conn| {
            Box::pin(async move {
                // 1. Get the assignment details to be deleted
                let assignment: Option<LeavePolicyAssignmentEntity> =
                    leave_policy_assignments::table
                        .filter(leave_policy_assignments::id.eq(id))
                        .filter(leave_policy_assignments::workspace_id.eq(workspace_id))
                        .filter(leave_policy_assignments::deleted_at.is_null())
                        .first::<LeavePolicyAssignmentEntity>(&mut *conn)
                        .await
                        .optional()?;

                let Some(assign) = assignment else {
                    return Err(diesel::result::Error::NotFound);
                };

                // 2. Get the leave_type_id of the associated policy definition
                let lt_id: Uuid = leave_policy_definitions::table
                    .filter(leave_policy_definitions::id.eq(assign.policy_id))
                    .filter(leave_policy_definitions::workspace_id.eq(workspace_id))
                    .select(leave_policy_definitions::leave_type_id)
                    .first(&mut *conn)
                    .await?;

                // 3. Soft-delete the assignment
                diesel::update(
                    leave_policy_assignments::table
                        .filter(leave_policy_assignments::id.eq(id))
                        .filter(leave_policy_assignments::workspace_id.eq(workspace_id)),
                )
                .set(leave_policy_assignments::deleted_at.eq(Utc::now()))
                .execute(&mut *conn)
                .await?;

                // 4. Reconcile/heal timeline:
                // Find a previous active assignment for the same target and same leave type
                // that ended exactly on `effective_from - 1 day`.
                if let Some(prev_end) = assign.effective_from.pred_opt() {
                    let prev_assignment_opt: Option<LeavePolicyAssignmentEntity> =
                        leave_policy_assignments::table
                            .filter(leave_policy_assignments::workspace_id.eq(workspace_id))
                            .filter(
                                leave_policy_assignments::assignee_type.eq(&assign.assignee_type),
                            )
                            .filter(
                                leave_policy_assignments::assignee_id
                                    .is_not_distinct_from(assign.assignee_id),
                            )
                            .filter(leave_policy_assignments::effective_to.eq(prev_end))
                            .filter(leave_policy_assignments::deleted_at.is_null())
                            .filter(
                                leave_policy_assignments::policy_id.eq_any(
                                    leave_policy_definitions::table
                                        .filter(
                                            leave_policy_definitions::workspace_id.eq(workspace_id),
                                        )
                                        .filter(leave_policy_definitions::leave_type_id.eq(lt_id))
                                        .select(leave_policy_definitions::id),
                                ),
                            )
                            .first::<LeavePolicyAssignmentEntity>(&mut *conn)
                            .await
                            .optional()?;

                    if let Some(prev_assign) = prev_assignment_opt {
                        // Restore previous assignment's effective_to to this assignment's effective_to
                        diesel::update(
                            leave_policy_assignments::table
                                .filter(leave_policy_assignments::id.eq(prev_assign.id))
                                .filter(leave_policy_assignments::workspace_id.eq(workspace_id)),
                        )
                        .set(leave_policy_assignments::effective_to.eq(assign.effective_to))
                        .execute(&mut *conn)
                        .await?;
                    }
                }

                Ok::<_, diesel::result::Error>(())
            })
        })
        .await
        .map_err(|e| match e {
            diesel::result::Error::NotFound => AppError::not_found(I18nKey::PolicyNotFound),
            _ => AppError::Internal(format!("db transaction: {e}")),
        })?;

        Ok(())
    }

    async fn find_default_paid_leave_type(
        &self,
        workspace_id: Uuid,
    ) -> Result<Option<Uuid>, AppError> {
        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .tenants_tx
                .get_connection(&workspace_id)
                .await
                .map_err(|e| AppError::Internal(format!("db: {e}")))?,
        );

        let result = leave_types::table
            .filter(leave_types::workspace_id.eq(workspace_id))
            .filter(leave_types::is_paid.eq(true))
            .filter(leave_types::deleted_at.is_null())
            .order(leave_types::name.asc())
            .select(leave_types::id)
            .first::<Uuid>(&mut *conn)
            .await
            .optional()
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(result)
    }

    async fn get_leave_type_name(&self, id: Uuid, workspace_id: Uuid) -> Result<String, AppError> {
        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .tenants_tx
                .get_connection(&workspace_id)
                .await
                .map_err(|e| AppError::Internal(format!("db: {e}")))?,
        );

        let result = leave_types::table
            .filter(leave_types::id.eq(id))
            .filter(leave_types::workspace_id.eq(workspace_id))
            .select(leave_types::name)
            .first::<String>(&mut *conn)
            .await
            .optional()
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(result.unwrap_or_else(|| "Leave".to_string()))
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
        let _repo = DieselLeavePolicyRepository::new(pools);
        // DI connection test passes if it compiles
    }
}
