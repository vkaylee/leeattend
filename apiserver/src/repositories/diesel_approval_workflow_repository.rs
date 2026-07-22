use crate::errors::AppError;
use crate::models::{
    ApprovalAction, ApprovalActionLog, ApprovalWorkflow, ApprovalWorkflowInstance,
    ApprovalWorkflowStep, ApprovalWorkflowTask, WorkflowCompletedPayload,
};
use crate::repositories::ApprovalWorkflowRepository;
use crate::schema::{
    approval_action_log, approval_workflow_instances, approval_workflow_steps,
    approval_workflow_tasks, approval_workflows,
};
use crate::utils::i18n::I18nKey;
use async_trait::async_trait;
use chrono::Utc;
use diesel::prelude::*;
use diesel_async::{AsyncConnection, RunQueryDsl};
use uuid::Uuid;

pub struct DieselApprovalWorkflowRepository {
    diesel_pools: crate::diesel_pool::DieselPools,
}

impl DieselApprovalWorkflowRepository {
    pub fn new(diesel_pools: crate::diesel_pool::DieselPools) -> Self {
        Self { diesel_pools }
    }
}

#[async_trait]
impl ApprovalWorkflowRepository for DieselApprovalWorkflowRepository {
    async fn create_workflow(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        workflow: ApprovalWorkflow,
    ) -> Result<ApprovalWorkflow, AppError> {
        if workflow.workspace_id != auth_token.workspace_id() {
            return Err(AppError::forbidden(I18nKey::TenantMismatch));
        }

        let mut conn = self
            .diesel_pools
            .tenant_transaction_conn(auth_token)
            .await?;

        let result = diesel::insert_into(approval_workflows::table)
            .values(&workflow)
            .get_result(&mut *conn)
            .await
            .map_err(|e| AppError::Internal(format!("Failed to create approval workflow: {e}")))?;

        Ok(result)
    }

    async fn create_steps(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        steps: Vec<ApprovalWorkflowStep>,
    ) -> Result<Vec<ApprovalWorkflowStep>, AppError> {
        let mut conn = self
            .diesel_pools
            .tenant_transaction_conn(auth_token)
            .await?;

        // Verify steps belong to a workflow in this workspace
        if let Some(first_step) = steps.as_slice().first() {
            let count: i64 = approval_workflows::table
                .filter(approval_workflows::id.eq(first_step.workflow_id))
                .filter(approval_workflows::workspace_id.eq(auth_token.workspace_id()))
                .count()
                .get_result(&mut *conn)
                .await
                .map_err(AppError::from)?;
            if count == 0 {
                return Err(AppError::forbidden(I18nKey::TenantMismatch));
            }
        }

        let result = diesel::insert_into(approval_workflow_steps::table)
            .values(&steps)
            .get_results(&mut *conn)
            .await
            .map_err(|e| AppError::Internal(format!("Failed to create workflow steps: {e}")))?;

        Ok(result)
    }

    async fn create_workflow_transactional(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        workflow: ApprovalWorkflow,
        steps: Vec<ApprovalWorkflowStep>,
    ) -> Result<(ApprovalWorkflow, Vec<ApprovalWorkflowStep>), AppError> {
        if workflow.workspace_id != auth_token.workspace_id() {
            return Err(AppError::forbidden(I18nKey::TenantMismatch));
        }

        let mut conn = self
            .diesel_pools
            .tenant_transaction_conn(auth_token)
            .await?;

        conn.transaction::<_, AppError, _>(|conn| {
            Box::pin(async move {
                let result = diesel::insert_into(approval_workflows::table)
                    .values(&workflow)
                    .get_result::<ApprovalWorkflow>(&mut *conn)
                    .await
                    .map_err(|e| {
                        AppError::Internal(format!("Failed to create approval workflow: {e}"))
                    })?;

                let mut steps_to_create = steps;
                for step in &mut steps_to_create {
                    step.workflow_id = result.id;
                }

                let created_steps = if steps_to_create.is_empty() {
                    Vec::new()
                } else {
                    diesel::insert_into(approval_workflow_steps::table)
                        .values(&steps_to_create)
                        .get_results(&mut *conn)
                        .await
                        .map_err(|e| {
                            AppError::Internal(format!("Failed to create workflow steps: {e}"))
                        })?
                };

                Ok((result, created_steps))
            })
        })
        .await
    }

    async fn update_workflow_definition_transactional(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        workflow: ApprovalWorkflow,
        steps: Vec<ApprovalWorkflowStep>,
    ) -> Result<(ApprovalWorkflow, Vec<ApprovalWorkflowStep>), AppError> {
        if workflow.workspace_id != auth_token.workspace_id() {
            return Err(AppError::forbidden(I18nKey::TenantMismatch));
        }

        let mut conn = self
            .diesel_pools
            .tenant_transaction_conn(auth_token)
            .await?;

        conn.transaction::<_, AppError, _>(|conn| {
            Box::pin(async move {
                let updated_wf = diesel::update(
                    approval_workflows::table
                        .filter(approval_workflows::id.eq(workflow.id))
                        .filter(approval_workflows::workspace_id.eq(auth_token.workspace_id())),
                )
                .set((
                    approval_workflows::name.eq(workflow.name),
                    approval_workflows::description.eq(workflow.description),
                    approval_workflows::total_steps.eq(workflow.total_steps),
                    approval_workflows::is_active.eq(workflow.is_active),
                    approval_workflows::updated_at.eq(chrono::Utc::now()),
                ))
                .get_result::<ApprovalWorkflow>(&mut *conn)
                .await
                .map_err(|e| match e {
                    diesel::result::Error::NotFound => {
                        AppError::not_found(I18nKey::WorkflowNotFound)
                    }
                    _ => AppError::Internal(format!("Failed to update approval workflow: {e}")),
                })?;

                diesel::delete(
                    approval_workflow_steps::table
                        .filter(approval_workflow_steps::workflow_id.eq(updated_wf.id)),
                )
                .execute(&mut *conn)
                .await
                .map_err(|e| {
                    AppError::Internal(format!("Failed to delete old workflow steps: {e}"))
                })?;

                let mut steps_to_create = steps;
                for step in &mut steps_to_create {
                    step.workflow_id = updated_wf.id;
                }

                let created_steps = if steps_to_create.is_empty() {
                    Vec::new()
                } else {
                    diesel::insert_into(approval_workflow_steps::table)
                        .values(&steps_to_create)
                        .get_results(&mut *conn)
                        .await
                        .map_err(|e| {
                            AppError::Internal(format!("Failed to create workflow steps: {e}"))
                        })?
                };

                Ok((updated_wf, created_steps))
            })
        })
        .await
    }

    async fn get_active_workflow(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        workspace_id: Uuid,
        entity_type: &str,
    ) -> Result<Option<ApprovalWorkflow>, AppError> {
        let mut conn = self
            .diesel_pools
            .tenant_transaction_conn(auth_token)
            .await?;

        let result = approval_workflows::table
            .filter(approval_workflows::workspace_id.eq(workspace_id))
            .filter(approval_workflows::entity_type.eq(entity_type))
            .filter(approval_workflows::is_active.eq(true))
            .first::<ApprovalWorkflow>(&mut *conn)
            .await
            .optional()
            .map_err(|e| AppError::Internal(format!("Failed to get active workflow: {e}")))?;

        Ok(result)
    }

    async fn get_steps(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        workflow_id: Uuid,
    ) -> Result<Vec<ApprovalWorkflowStep>, AppError> {
        let mut conn = self
            .diesel_pools
            .tenant_transaction_conn(auth_token)
            .await?;

        let result = approval_workflow_steps::table
            .filter(approval_workflow_steps::workflow_id.eq(workflow_id))
            .filter(
                approval_workflow_steps::workflow_id.eq_any(
                    approval_workflows::table
                        .select(approval_workflows::id)
                        .filter(approval_workflows::workspace_id.eq(auth_token.workspace_id())),
                ),
            )
            .order(approval_workflow_steps::step_order.asc())
            .load::<ApprovalWorkflowStep>(&mut *conn)
            .await
            .map_err(|e| AppError::Internal(format!("Failed to get workflow steps: {e}")))?;

        Ok(result)
    }

    async fn log_action(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        log: ApprovalActionLog,
    ) -> Result<ApprovalActionLog, AppError> {
        if log.workspace_id != auth_token.workspace_id() {
            return Err(AppError::forbidden(I18nKey::TenantMismatch));
        }

        let mut conn = self
            .diesel_pools
            .tenant_transaction_conn(auth_token)
            .await?;

        let result = diesel::insert_into(approval_action_log::table)
            .values(&log)
            .get_result(&mut *conn)
            .await
            .map_err(|e| AppError::Internal(format!("Failed to log approval action: {e}")))?;

        Ok(result)
    }

    async fn get_action_logs(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        entity_id: Uuid,
    ) -> Result<Vec<ApprovalActionLog>, AppError> {
        let mut conn = self
            .diesel_pools
            .tenant_transaction_conn(auth_token)
            .await?;

        let result = approval_action_log::table
            .filter(approval_action_log::entity_id.eq(entity_id))
            .filter(approval_action_log::workspace_id.eq(auth_token.workspace_id()))
            .order(approval_action_log::acted_at.asc())
            .load::<ApprovalActionLog>(&mut *conn)
            .await
            .map_err(|e| AppError::Internal(format!("Failed to get action logs: {e}")))?;

        Ok(result)
    }

    async fn create_instance(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        instance: ApprovalWorkflowInstance,
    ) -> Result<ApprovalWorkflowInstance, AppError> {
        if instance.workspace_id != auth_token.workspace_id() {
            return Err(AppError::forbidden(I18nKey::TenantMismatch));
        }

        let mut conn = self
            .diesel_pools
            .tenant_transaction_conn(auth_token)
            .await?;

        let result = diesel::insert_into(approval_workflow_instances::table)
            .values(&instance)
            .get_result(&mut *conn)
            .await
            .map_err(|e| {
                AppError::Internal(format!("Failed to create approval workflow instance: {e}"))
            })?;

        Ok(result)
    }

    async fn update_instance(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        instance: ApprovalWorkflowInstance,
    ) -> Result<ApprovalWorkflowInstance, AppError> {
        let mut conn = self
            .diesel_pools
            .tenant_transaction_conn(auth_token)
            .await?;

        let result = diesel::update(
            approval_workflow_instances::table
                .filter(approval_workflow_instances::id.eq(instance.id))
                .filter(approval_workflow_instances::workspace_id.eq(auth_token.workspace_id())),
        )
        .set(&instance)
        .get_result(&mut *conn)
        .await
        .map_err(|e| {
            AppError::Internal(format!("Failed to update approval workflow instance: {e}"))
        })?;

        Ok(result)
    }

    async fn get_instance_by_entity(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        entity_type: &str,
        entity_id: Uuid,
    ) -> Result<Option<ApprovalWorkflowInstance>, AppError> {
        let mut conn = self
            .diesel_pools
            .tenant_transaction_conn(auth_token)
            .await?;

        let result = approval_workflow_instances::table
            .filter(approval_workflow_instances::entity_type.eq(entity_type))
            .filter(approval_workflow_instances::entity_id.eq(entity_id))
            .filter(approval_workflow_instances::workspace_id.eq(auth_token.workspace_id()))
            .first::<ApprovalWorkflowInstance>(&mut *conn)
            .await
            .optional()
            .map_err(|e| {
                AppError::Internal(format!(
                    "Failed to get approval workflow instance by entity: {e}"
                ))
            })?;

        Ok(result)
    }

    async fn get_instance_by_id(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        instance_id: Uuid,
    ) -> Result<Option<ApprovalWorkflowInstance>, AppError> {
        let mut conn = self
            .diesel_pools
            .tenant_transaction_conn(auth_token)
            .await?;

        let result = approval_workflow_instances::table
            .filter(approval_workflow_instances::id.eq(instance_id))
            .filter(approval_workflow_instances::workspace_id.eq(auth_token.workspace_id()))
            .first::<ApprovalWorkflowInstance>(&mut *conn)
            .await
            .optional()
            .map_err(|e| {
                AppError::Internal(format!(
                    "Failed to get approval workflow instance by id: {e}"
                ))
            })?;

        Ok(result)
    }

    async fn get_instances_by_ids(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        instance_ids: &[Uuid],
    ) -> Result<Vec<ApprovalWorkflowInstance>, AppError> {
        if instance_ids.is_empty() {
            return Ok(Vec::new());
        }

        let mut conn = self
            .diesel_pools
            .tenant_transaction_conn(auth_token)
            .await?;

        let results = approval_workflow_instances::table
            .filter(approval_workflow_instances::id.eq_any(instance_ids))
            .filter(approval_workflow_instances::workspace_id.eq(auth_token.workspace_id()))
            .load::<ApprovalWorkflowInstance>(&mut *conn)
            .await
            .map_err(|e| {
                AppError::Internal(format!(
                    "Failed to get approval workflow instances by ids: {e}"
                ))
            })?;

        Ok(results)
    }

    async fn get_workflow_by_id(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        workflow_id: Uuid,
    ) -> Result<Option<ApprovalWorkflow>, AppError> {
        let mut conn = self
            .diesel_pools
            .tenant_transaction_conn(auth_token)
            .await?;

        let result = approval_workflows::table
            .filter(approval_workflows::id.eq(workflow_id))
            .filter(approval_workflows::workspace_id.eq(auth_token.workspace_id()))
            .first::<ApprovalWorkflow>(&mut *conn)
            .await
            .optional()
            .map_err(|e| {
                AppError::Internal(format!("Failed to get approval workflow by id: {e}"))
            })?;

        Ok(result)
    }

    async fn list_workflows(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        workspace_id: Uuid,
        entity_type: Option<String>,
        page: i64,
        limit: i64,
    ) -> Result<(Vec<ApprovalWorkflow>, i64), AppError> {
        let mut conn = self
            .diesel_pools
            .tenant_transaction_conn(auth_token)
            .await?;

        let mut count_query = approval_workflows::table
            .filter(approval_workflows::workspace_id.eq(workspace_id))
            .into_boxed();

        if let Some(ref et) = entity_type {
            count_query = count_query.filter(approval_workflows::entity_type.eq(et.clone()));
        }

        let total: i64 = count_query
            .count()
            .get_result(&mut *conn)
            .await
            .map_err(|e| AppError::Internal(format!("Failed to count approval workflows: {e}")))?;

        let mut select_query = approval_workflows::table
            .filter(approval_workflows::workspace_id.eq(workspace_id))
            .into_boxed();

        if let Some(et) = entity_type {
            select_query = select_query.filter(approval_workflows::entity_type.eq(et));
        }

        let result = select_query
            .limit(limit)
            .offset((page - 1) * limit)
            .load::<ApprovalWorkflow>(&mut *conn)
            .await
            .map_err(|e| AppError::Internal(format!("Failed to list approval workflows: {e}")))?;

        Ok((result, total))
    }

    async fn update_workflow(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        workflow: ApprovalWorkflow,
    ) -> Result<ApprovalWorkflow, AppError> {
        let mut conn = self
            .diesel_pools
            .tenant_transaction_conn(auth_token)
            .await?;

        let result = diesel::update(
            approval_workflows::table
                .filter(approval_workflows::id.eq(workflow.id))
                .filter(approval_workflows::workspace_id.eq(auth_token.workspace_id())),
        )
        .set((
            approval_workflows::name.eq(workflow.name),
            approval_workflows::description.eq(workflow.description),
            approval_workflows::total_steps.eq(workflow.total_steps),
            approval_workflows::is_active.eq(workflow.is_active),
            approval_workflows::updated_at.eq(chrono::Utc::now()),
        ))
        .get_result(&mut *conn)
        .await
        .map_err(|e| match e {
            diesel::result::Error::NotFound => AppError::not_found(I18nKey::WorkflowNotFound),
            _ => AppError::Internal(format!("Failed to update approval workflow: {e}")),
        })?;

        Ok(result)
    }

    async fn delete_workflow(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        workflow_id: Uuid,
    ) -> Result<(), AppError> {
        let mut conn = self
            .diesel_pools
            .tenant_transaction_conn(auth_token)
            .await?;

        let rows_deleted = diesel::delete(
            approval_workflows::table
                .filter(approval_workflows::id.eq(workflow_id))
                .filter(approval_workflows::workspace_id.eq(auth_token.workspace_id())),
        )
        .execute(&mut *conn)
        .await
        .map_err(|e| AppError::Internal(format!("Failed to delete approval workflow: {e}")))?;

        if rows_deleted == 0 {
            return Err(AppError::not_found(I18nKey::WorkflowNotFound));
        }

        Ok(())
    }

    async fn delete_steps_by_workflow_id(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        workflow_id: Uuid,
    ) -> Result<(), AppError> {
        let mut conn = self
            .diesel_pools
            .tenant_transaction_conn(auth_token)
            .await?;

        diesel::delete(
            approval_workflow_steps::table
                .filter(approval_workflow_steps::workflow_id.eq(workflow_id))
                .filter(
                    approval_workflow_steps::workflow_id.eq_any(
                        approval_workflows::table
                            .select(approval_workflows::id)
                            .filter(approval_workflows::workspace_id.eq(auth_token.workspace_id())),
                    ),
                ),
        )
        .execute(&mut *conn)
        .await
        .map_err(|e| AppError::Internal(format!("Failed to delete workflow steps: {e}")))?;

        Ok(())
    }

    async fn get_steps_for_workflows(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        workflow_ids: Vec<Uuid>,
    ) -> Result<Vec<ApprovalWorkflowStep>, AppError> {
        let mut conn = self
            .diesel_pools
            .tenant_transaction_conn(auth_token)
            .await?;

        let result = approval_workflow_steps::table
            .filter(approval_workflow_steps::workflow_id.eq_any(workflow_ids))
            .filter(
                approval_workflow_steps::workflow_id.eq_any(
                    approval_workflows::table
                        .select(approval_workflows::id)
                        .filter(approval_workflows::workspace_id.eq(auth_token.workspace_id())),
                ),
            )
            .order((
                approval_workflow_steps::workflow_id,
                approval_workflow_steps::step_order.asc(),
            ))
            .load::<ApprovalWorkflowStep>(&mut *conn)
            .await
            .map_err(|e| AppError::Internal(format!("Failed to get steps for workflows: {e}")))?;

        Ok(result)
    }

    async fn create_tasks(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        tasks: Vec<ApprovalWorkflowTask>,
    ) -> Result<Vec<ApprovalWorkflowTask>, AppError> {
        let mut conn = self
            .diesel_pools
            .tenant_transaction_conn(auth_token)
            .await?;

        // Verify tasks are for an instance belonging to this workspace
        if let Some(first_task) = tasks.as_slice().first() {
            let count: i64 = approval_workflow_instances::table
                .filter(approval_workflow_instances::id.eq(first_task.instance_id()))
                .filter(approval_workflow_instances::workspace_id.eq(auth_token.workspace_id()))
                .count()
                .get_result(&mut *conn)
                .await
                .map_err(AppError::from)?;
            if count == 0 {
                return Err(AppError::forbidden(I18nKey::TenantMismatch));
            }
        }

        let result = diesel::insert_into(approval_workflow_tasks::table)
            .values(&tasks)
            .get_results(&mut *conn)
            .await
            .map_err(|e| {
                AppError::Internal(format!("Failed to create approval workflow tasks: {e}"))
            })?;

        Ok(result)
    }

    async fn update_task(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        task: ApprovalWorkflowTask,
    ) -> Result<ApprovalWorkflowTask, AppError> {
        let mut conn = self
            .diesel_pools
            .tenant_transaction_conn(auth_token)
            .await?;

        let result = diesel::update(
            approval_workflow_tasks::table
                .filter(approval_workflow_tasks::id.eq(task.id()))
                .filter(
                    approval_workflow_tasks::instance_id.eq_any(
                        approval_workflow_instances::table
                            .select(approval_workflow_instances::id)
                            .filter(
                                approval_workflow_instances::workspace_id
                                    .eq(auth_token.workspace_id()),
                            ),
                    ),
                ),
        )
        .set(&task)
        .get_result(&mut *conn)
        .await
        .map_err(|e| AppError::Internal(format!("Failed to update approval workflow task: {e}")))?;

        Ok(result)
    }

    async fn get_active_tasks_for_instance(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        instance_id: Uuid,
    ) -> Result<Vec<ApprovalWorkflowTask>, AppError> {
        let mut conn = self
            .diesel_pools
            .tenant_transaction_conn(auth_token)
            .await?;

        let result = approval_workflow_tasks::table
            .filter(approval_workflow_tasks::instance_id.eq(instance_id))
            .filter(
                approval_workflow_tasks::instance_id.eq_any(
                    approval_workflow_instances::table
                        .select(approval_workflow_instances::id)
                        .filter(
                            approval_workflow_instances::workspace_id.eq(auth_token.workspace_id()),
                        ),
                ),
            )
            .filter(approval_workflow_tasks::status.eq("pending"))
            .load::<ApprovalWorkflowTask>(&mut *conn)
            .await
            .map_err(|e| {
                AppError::Internal(format!("Failed to get active tasks for instance: {e}"))
            })?;

        Ok(result)
    }

    async fn get_pending_tasks_for_user(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        user_id: Uuid,
        role_ids: Vec<Uuid>,
        permissions: Vec<String>,
        managed_dept_ids: Vec<Uuid>,
        page: i64,
        limit: i64,
    ) -> Result<(Vec<ApprovalWorkflowTask>, i64), AppError> {
        let mut conn = self
            .diesel_pools
            .tenant_transaction_conn(auth_token)
            .await?;

        let cond_user = approval_workflow_tasks::assignee_type
            .eq("specific_user")
            .and(approval_workflow_tasks::assignee_target_id.eq(user_id));

        let base_filter = approval_workflow_tasks::table
            .filter(approval_workflow_tasks::status.eq("pending"))
            .filter(
                approval_workflow_tasks::instance_id.eq_any(
                    approval_workflow_instances::table
                        .select(approval_workflow_instances::id)
                        .filter(
                            approval_workflow_instances::workspace_id.eq(auth_token.workspace_id()),
                        ),
                ),
            )
            .filter(
                cond_user
                    .or(approval_workflow_tasks::assignee_type
                        .eq("role")
                        .and(approval_workflow_tasks::assignee_target_id.eq_any(role_ids)))
                    .or(approval_workflow_tasks::assignee_type
                        .eq("permission")
                        .and(approval_workflow_tasks::assignee_target_name.eq_any(permissions)))
                    .or(approval_workflow_tasks::assignee_type
                        .eq("department_manager")
                        .and(approval_workflow_tasks::assignee_target_id.eq_any(managed_dept_ids))),
            );

        let total: i64 = base_filter
            .clone()
            .count()
            .get_result(&mut *conn)
            .await
            .map_err(|e| {
                AppError::Internal(format!("Failed to count pending tasks for user: {e}"))
            })?;

        let result = base_filter
            .order(approval_workflow_tasks::created_at.desc())
            .limit(limit)
            .offset((page - 1) * limit)
            .load::<ApprovalWorkflowTask>(&mut *conn)
            .await
            .map_err(|e| {
                AppError::Internal(format!("Failed to get pending tasks for user: {e}"))
            })?;

        Ok((result, total))
    }

    #[allow(clippy::too_many_lines)]
    async fn apply_approval_action_transactional(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        instance_id: Uuid,
        user_id: Uuid,
        action: ApprovalAction,
        note: Option<String>,
        new_approver_user_id: Option<Uuid>,
        is_admin: bool,
        matching_task_id: Option<Uuid>,
    ) -> Result<(ApprovalWorkflowInstance, Option<WorkflowCompletedPayload>), AppError> {
        let mut conn = self
            .diesel_pools
            .tenant_transaction_conn(auth_token)
            .await?;

        conn.transaction::<_, AppError, _>(|conn| {
            Box::pin(async move {
                // 1. SELECT ... FOR UPDATE on instance
                let mut instance = approval_workflow_instances::table
                    .filter(approval_workflow_instances::id.eq(instance_id))
                    .filter(approval_workflow_instances::workspace_id.eq(auth_token.workspace_id()))
                    .for_update()
                    .first::<ApprovalWorkflowInstance>(&mut *conn)
                    .await
                    .map_err(|e| match e {
                        diesel::result::Error::NotFound => {
                            AppError::not_found(I18nKey::WorkflowInstanceNotFound)
                        }
                        _ => AppError::Internal(format!("Failed to lock instance: {e}")),
                    })?;

                // 2. Re-check status
                if instance.status != "pending" {
                    return Err(AppError::validation_args(
                        I18nKey::Validation,
                        vec![(
                            "details",
                            format!("Workflow instance is already {}", instance.status),
                        )],
                    ));
                }

                // 3. Parse steps snapshot
                let mut steps: Vec<crate::models::WorkflowInstanceStepSnapshot> =
                    serde_json::from_value(instance.steps_snapshot.clone()).map_err(|e| {
                        AppError::Internal(format!("Failed to parse steps snapshot: {e}"))
                    })?;

                let current_step_idx = steps
                    .iter()
                    .position(|s| s.step_order == instance.current_step_order)
                    .ok_or_else(|| {
                        AppError::Internal(format!(
                            "Current step not found in snapshot: {}",
                            instance.current_step_order
                        ))
                    })?;

                let current_step_order = instance.current_step_order;

                // 4. Load active tasks in transaction
                let active_tasks = approval_workflow_tasks::table
                    .filter(approval_workflow_tasks::instance_id.eq(instance_id))
                    .filter(approval_workflow_tasks::status.eq("pending"))
                    .load::<ApprovalWorkflowTask>(&mut *conn)
                    .await
                    .map_err(|e| AppError::Internal(format!("Failed to load active tasks: {e}")))?;

                // 5. Insert audit log
                let log_entry = ApprovalActionLog {
                    id: Uuid::new_v4(),
                    workspace_id: auth_token.workspace_id(),
                    entity_type: instance.entity_type.clone(),
                    entity_id: instance.entity_id,
                    step_order: current_step_order,
                    step_name: Some(steps[current_step_idx].step_name.clone()),
                    acted_by_user_id: user_id,
                    action: action.as_str().to_string(),
                    note: note.clone(),
                    acted_at: Utc::now(),
                };
                diesel::insert_into(approval_action_log::table)
                    .values(&log_entry)
                    .execute(&mut *conn)
                    .await
                    .map_err(|e| AppError::Internal(format!("Failed to insert audit log: {e}")))?;

                // 6. Update approvers_acted in snapshot
                steps[current_step_idx]
                    .approvers_acted
                    .push(crate::models::ApproverActionInfo {
                        user_id,
                        action: action.as_str().to_string(),
                        note: note.clone(),
                        acted_at: Utc::now(),
                    });

                // 7. Process action
                match action {
                    ApprovalAction::Reassign => {
                        let target_uid = new_approver_user_id.unwrap_or(Uuid::nil());

                        for mut task in active_tasks {
                            if task.step_order() == current_step_order {
                                task.cancel();
                                diesel::update(
                                    approval_workflow_tasks::table
                                        .filter(approval_workflow_tasks::id.eq(task.id())),
                                )
                                .set(&task)
                                .execute(&mut *conn)
                                .await
                                .map_err(|e| {
                                    AppError::Internal(format!("Failed to cancel task: {e}"))
                                })?;
                            }
                        }

                        let reassigned_task = ApprovalWorkflowTask::new(
                            instance_id,
                            current_step_order,
                            "specific_user".to_string(),
                            Some(target_uid),
                            None,
                        );
                        diesel::insert_into(approval_workflow_tasks::table)
                            .values(&reassigned_task)
                            .execute(&mut *conn)
                            .await
                            .map_err(|e| {
                                AppError::Internal(format!("Failed to create reassigned task: {e}"))
                            })?;

                        steps[current_step_idx].resolved_approver_ids = vec![target_uid];
                    }
                    ApprovalAction::Reject | ApprovalAction::Return => {
                        let new_status = if action == ApprovalAction::Reject {
                            "rejected"
                        } else {
                            "returned"
                        };
                        steps[current_step_idx].status = new_status.to_string();
                        instance.status = new_status.to_string();

                        for mut task in active_tasks {
                            if task.step_order() == current_step_order {
                                if let Some(mt_id) = matching_task_id {
                                    if task.id() == mt_id {
                                        task.complete(user_id);
                                    } else {
                                        task.cancel();
                                    }
                                } else {
                                    task.cancel();
                                }
                                diesel::update(
                                    approval_workflow_tasks::table
                                        .filter(approval_workflow_tasks::id.eq(task.id())),
                                )
                                .set(&task)
                                .execute(&mut *conn)
                                .await
                                .map_err(|e| {
                                    AppError::Internal(format!("Failed to update task: {e}"))
                                })?;
                            }
                        }
                    }
                    ApprovalAction::Approve | ApprovalAction::Skip => {
                        let consensus_mode = &steps[current_step_idx].consensus_mode;

                        if let Some(mt_id) = matching_task_id {
                            if let Some(mt) = active_tasks.iter().find(|t| t.id() == mt_id) {
                                let mut task_to_update = mt.clone();
                                task_to_update.complete(user_id);
                                diesel::update(
                                    approval_workflow_tasks::table.filter(
                                        approval_workflow_tasks::id.eq(task_to_update.id()),
                                    ),
                                )
                                .set(&task_to_update)
                                .execute(&mut *conn)
                                .await
                                .map_err(|e| {
                                    AppError::Internal(format!("Failed to complete task: {e}"))
                                })?;
                            }
                        }

                        let step_approved = if is_admin {
                            for mut task in active_tasks {
                                if task.step_order() == current_step_order {
                                    task.cancel();
                                    diesel::update(
                                        approval_workflow_tasks::table
                                            .filter(approval_workflow_tasks::id.eq(task.id())),
                                    )
                                    .set(&task)
                                    .execute(&mut *conn)
                                    .await
                                    .map_err(|e| {
                                        AppError::Internal(format!("Failed to cancel task: {e}"))
                                    })?;
                                }
                            }
                            true
                        } else if consensus_mode == "any" {
                            for mut task in active_tasks {
                                if task.step_order() == current_step_order {
                                    if let Some(mt_id) = matching_task_id {
                                        if task.id() != mt_id {
                                            task.cancel();
                                            diesel::update(
                                                approval_workflow_tasks::table.filter(
                                                    approval_workflow_tasks::id.eq(task.id()),
                                                ),
                                            )
                                            .set(&task)
                                            .execute(&mut *conn)
                                            .await
                                            .map_err(
                                                |e| {
                                                    AppError::Internal(format!(
                                                        "Failed to cancel task: {e}"
                                                    ))
                                                },
                                            )?;
                                        }
                                    }
                                }
                            }
                            true
                        } else {
                            // consensus_mode == "all" — check if any pending tasks remain for this step
                            let remaining: i64 = approval_workflow_tasks::table
                                .filter(approval_workflow_tasks::instance_id.eq(instance_id))
                                .filter(approval_workflow_tasks::step_order.eq(current_step_order))
                                .filter(approval_workflow_tasks::status.eq("pending"))
                                .count()
                                .get_result(&mut *conn)
                                .await
                                .map_err(|e| {
                                    AppError::Internal(format!(
                                        "Failed to count remaining tasks: {e}"
                                    ))
                                })?;
                            remaining == 0
                        };

                        if step_approved {
                            steps[current_step_idx].status = "approved".to_string();

                            let next_step = steps
                                .iter_mut()
                                .filter(|s| s.step_order > current_step_order)
                                .min_by_key(|s| s.step_order);

                            if let Some(next) = next_step {
                                instance.current_step_order = next.step_order;
                                next.status = "pending".to_string();
                                // Note: spawn_tasks_for_step will be called outside transaction
                                // For now, we just update the snapshot
                            } else {
                                instance.status = "approved".to_string();
                            }
                        }
                    }
                }

                // 8. Update instance snapshot/status
                instance.steps_snapshot = serde_json::to_value(&steps)
                    .map_err(|e| AppError::Internal(format!("Failed to serialize steps: {e}")))?;
                instance.updated_at = Utc::now();

                let updated_instance = diesel::update(
                    approval_workflow_instances::table
                        .filter(approval_workflow_instances::id.eq(instance_id)),
                )
                .set(&instance)
                .get_result::<ApprovalWorkflowInstance>(&mut *conn)
                .await
                .map_err(|e| AppError::Internal(format!("Failed to update instance: {e}")))?;

                // 9. If workflow completed, create outbox payload
                let outbox_payload = if updated_instance.status == "pending" {
                    None
                } else {
                    Some(WorkflowCompletedPayload {
                        entity_type: updated_instance.entity_type.clone(),
                        entity_id: updated_instance.entity_id,
                        final_status: updated_instance.status.clone(),
                        actor_user_id: user_id,
                        trace_context: None,
                    })
                };

                Ok((updated_instance, outbox_payload))
            })
        })
        .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diesel_pool::{DieselPools, TenantAuthToken};
    use chrono::Utc;
    use uuid::Uuid;

    #[tokio::test]
    async fn test_repo_methods_error_handling() {
        let test_db_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
            "postgres://user:pass@localhost:5432/leeattend_test_db".to_string()
        });
        let pools = DieselPools::for_test(&test_db_url);
        let repo = DieselApprovalWorkflowRepository::new(pools);
        let ws_id = Uuid::new_v4();
        let auth_token = TenantAuthToken::system_override(ws_id);

        let workflow = ApprovalWorkflow {
            id: Uuid::new_v4(),
            workspace_id: ws_id,
            entity_type: "ot_request".into(),
            name: "Test".into(),
            description: None,
            total_steps: 1,
            is_active: true,
            created_by: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        // Try to create the workflow. Since it's a real DB, it might actually succeed!
        let _ = repo.create_workflow(&auth_token, workflow).await;

        let _ = repo.create_steps(&auth_token, vec![]).await;

        let _ = repo
            .get_active_workflow(&auth_token, ws_id, "ot_request")
            .await;

        let _ = repo.get_steps(&auth_token, Uuid::new_v4()).await;

        let empty_batch_result = repo.get_instances_by_ids(&auth_token, &[]).await;
        assert!(
            matches!(empty_batch_result, Ok(ref instances) if instances.is_empty()),
            "Empty input should return Ok([]), got: {empty_batch_result:?}"
        );

        let log = ApprovalActionLog {
            id: Uuid::new_v4(),
            workspace_id: ws_id,
            entity_type: "ot_request".into(),
            entity_id: Uuid::new_v4(),
            step_order: 1,
            step_name: None,
            acted_by_user_id: Uuid::new_v4(),
            action: "approve".into(),
            note: None,
            acted_at: Utc::now(),
        };

        let _ = repo.log_action(&auth_token, log).await;
        let _ = repo.get_action_logs(&auth_token, Uuid::new_v4()).await;
    }
}
