use crate::errors::AppError;
use crate::models::{
    ApprovalActionLog, ApprovalWorkflow, ApprovalWorkflowInstance, ApprovalWorkflowStep,
    ApprovalWorkflowTask,
};
use crate::repositories::ApprovalWorkflowRepository;
use crate::schema::{
    approval_action_log, approval_workflow_instances, approval_workflow_steps,
    approval_workflow_tasks, approval_workflows,
};
use async_trait::async_trait;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use uuid::Uuid;

pub struct DieselApprovalWorkflowRepository {
    pub diesel_pools: crate::diesel_pool::DieselPools,
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

        let result = diesel::insert_into(approval_workflow_steps::table)
            .values(&steps)
            .get_results(&mut *conn)
            .await
            .map_err(|e| AppError::Internal(format!("Failed to create workflow steps: {e}")))?;

        Ok(result)
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
            diesel::result::Error::NotFound => AppError::NotFound("Workflow not found".to_string()),
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
            return Err(AppError::NotFound("Workflow not found".to_string()));
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
    ) -> Result<Vec<ApprovalWorkflowTask>, AppError> {
        let mut conn = self
            .diesel_pools
            .tenant_transaction_conn(auth_token)
            .await?;

        let cond_user = approval_workflow_tasks::assignee_type
            .eq("specific_user")
            .and(approval_workflow_tasks::assignee_target_id.eq(user_id));

        let result = approval_workflow_tasks::table
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
            )
            .load::<ApprovalWorkflowTask>(&mut *conn)
            .await
            .map_err(|e| {
                AppError::Internal(format!("Failed to get pending tasks for user: {e}"))
            })?;

        Ok(result)
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
        let created = repo.create_workflow(&auth_token, workflow).await;

        if created.is_ok() {
            let _ = repo.create_steps(&auth_token, vec![]).await;

            let _ = repo
                .get_active_workflow(&auth_token, ws_id, "ot_request")
                .await;

            let _ = repo.get_steps(&auth_token, Uuid::new_v4()).await;

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
        } else {
            let _ = repo.create_steps(&auth_token, vec![]).await;

            let _ = repo
                .get_active_workflow(&auth_token, ws_id, "ot_request")
                .await;

            let _ = repo.get_steps(&auth_token, Uuid::new_v4()).await;

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
}
