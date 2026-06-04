use super::ApprovalWorkflowService;
use crate::errors::AppResult;
use crate::models::ApprovalWorkflowTask;
use chrono::Utc;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct UserApprovalContext {
    pub role_ids: Vec<Uuid>,
    pub permissions: Vec<String>,
    pub managed_dept_ids: Vec<Uuid>,
}

impl ApprovalWorkflowService {
    async fn resolve_user_approval_context(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        workspace_id: Uuid,
        user_id: Uuid,
    ) -> AppResult<UserApprovalContext> {
        let mut role_ids = Vec::new();
        if let Some(r_id) = self
            .workspace_repo
            .find_primary_role(workspace_id, user_id)
            .await?
        {
            role_ids.push(r_id);
        }

        let extra_roles = self
            .workspace_repo
            .find_extra_roles(workspace_id, user_id)
            .await?;
        for r in extra_roles {
            role_ids.push(r.role_id);
        }
        role_ids.sort();
        role_ids.dedup();

        let mut permissions = self
            .workspace_repo
            .find_extra_permissions(workspace_id, user_id)
            .await?;

        if !role_ids.is_empty() {
            if let Ok(role_perms) = self
                .workspace_repo
                .get_permissions_for_roles(&role_ids)
                .await
            {
                permissions.extend(role_perms);
            }
        }
        permissions.sort();
        permissions.dedup();

        let managed_dept_ids = self
            .employee_repo
            .get_managed_departments(auth_token, user_id)
            .await?;

        Ok(UserApprovalContext {
            role_ids,
            permissions,
            managed_dept_ids,
        })
    }

    async fn get_delegators_for_user(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        user_id: Uuid,
    ) -> Vec<Uuid> {
        let mut delegators = Vec::new();
        if let Ok(entries) = self
            .workspace_config_repo
            .list_entries(auth_token, auth_token.workspace_id(), None)
            .await
        {
            for entry in entries {
                if entry.config_key.starts_with("delegation:") {
                    if let Some(ref val_text) = entry.value_text {
                        if val_text == &user_id.to_string() {
                            if let Some(uuid_str) = entry.config_key.strip_prefix("delegation:") {
                                if let Ok(delegator_uuid) = Uuid::parse_str(uuid_str) {
                                    let today = Utc::now().date_naive();
                                    let is_effective = entry.effective_from <= today
                                        && entry.effective_to.is_none_or(|to| to >= today);
                                    if is_effective {
                                        delegators.push(delegator_uuid);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        delegators
    }

    pub async fn get_pending_tasks_for_user(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        user_id: Uuid,
    ) -> AppResult<Vec<ApprovalWorkflowTask>> {
        let workspace_id = auth_token.workspace_id();

        // 1. Resolve context for the main user
        let user_ctx = self
            .resolve_user_approval_context(auth_token, workspace_id, user_id)
            .await?;

        // 2. Fetch delegators
        let delegators = self.get_delegators_for_user(auth_token, user_id).await;

        // 3. Fetch pending tasks for the main user
        let mut all_tasks = self
            .approval_workflow_repo
            .get_pending_tasks_for_user(
                auth_token,
                user_id,
                user_ctx.role_ids,
                user_ctx.permissions,
                user_ctx.managed_dept_ids,
            )
            .await?;

        // 4. Fetch pending tasks for each delegator
        for &delegator_id in &delegators {
            let del_ctx = self
                .resolve_user_approval_context(auth_token, workspace_id, delegator_id)
                .await?;
            let del_tasks = self
                .approval_workflow_repo
                .get_pending_tasks_for_user(
                    auth_token,
                    delegator_id,
                    del_ctx.role_ids,
                    del_ctx.permissions,
                    del_ctx.managed_dept_ids,
                )
                .await?;
            all_tasks.extend(del_tasks);
        }

        all_tasks.sort_by_key(ApprovalWorkflowTask::id);
        all_tasks.dedup_by_key(|t| t.id());

        let mut final_tasks = Vec::new();
        for task in all_tasks {
            if let Ok(Some(instance)) = self
                .approval_workflow_repo
                .get_instance_by_id(auth_token, task.instance_id())
                .await
            {
                let beneficiary_user_id = self
                    .get_beneficiary_user_id_for_entity(
                        auth_token,
                        &instance.entity_type,
                        instance.entity_id,
                    )
                    .await;

                let is_self_approving = beneficiary_user_id
                    .is_some_and(|b_id| b_id == user_id || delegators.contains(&b_id));
                if !is_self_approving {
                    final_tasks.push(task);
                }
            } else {
                final_tasks.push(task);
            }
        }

        Ok(final_tasks)
    }

    pub(crate) async fn spawn_tasks_for_step(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        instance_id: Uuid,
        step_order: i32,
        approver_type: &str,
        approver_config: &serde_json::Value,
        beneficiary_employee_id: Uuid,
    ) -> AppResult<Vec<ApprovalWorkflowTask>> {
        let mut tasks = Vec::new();
        match approver_type {
            "department_manager" => {
                if let Ok(Some(current_dept)) = self
                    .department_repo
                    .get_current_department(auth_token, beneficiary_employee_id)
                    .await
                {
                    tasks.push(ApprovalWorkflowTask::new(
                        instance_id,
                        step_order,
                        "department_manager".to_string(),
                        Some(current_dept.department_id),
                        None,
                    ));
                } else if let Ok(admins) = self
                    .workspace_repo
                    .get_admins_with_permissions(auth_token.workspace_id(), &[])
                    .await
                {
                    if let Some(admin_id) = admins.first() {
                        tasks.push(ApprovalWorkflowTask::new(
                            instance_id,
                            step_order,
                            "specific_user".to_string(),
                            Some(*admin_id),
                            None,
                        ));
                    }
                }
            }
            "specific_user" => {
                if let Some(uid_str) = approver_config.get("user_id").and_then(|v| v.as_str()) {
                    if let Ok(uid) = Uuid::parse_str(uid_str) {
                        tasks.push(ApprovalWorkflowTask::new(
                            instance_id,
                            step_order,
                            "specific_user".to_string(),
                            Some(uid),
                            None,
                        ));
                    }
                }
            }
            "role" => {
                if let Some(role_id_str) = approver_config.get("role_id").and_then(|v| v.as_str()) {
                    if let Ok(target_role_id) = Uuid::parse_str(role_id_str) {
                        tasks.push(ApprovalWorkflowTask::new(
                            instance_id,
                            step_order,
                            "role".to_string(),
                            Some(target_role_id),
                            None,
                        ));
                    }
                }
            }
            "permission" => {
                if let Some(perm_name) = approver_config
                    .get("permission_name")
                    .and_then(|v| v.as_str())
                {
                    tasks.push(ApprovalWorkflowTask::new(
                        instance_id,
                        step_order,
                        "permission".to_string(),
                        None,
                        Some(perm_name.to_string()),
                    ));
                }
            }
            _ => {}
        }

        if tasks.is_empty() {
            if let Ok(admins) = self
                .workspace_repo
                .get_admins_with_permissions(auth_token.workspace_id(), &[])
                .await
            {
                if let Some(admin_id) = admins.first() {
                    tasks.push(ApprovalWorkflowTask::new(
                        instance_id,
                        step_order,
                        "specific_user".to_string(),
                        Some(*admin_id),
                        None,
                    ));
                }
            }
        }

        let created_tasks = self
            .approval_workflow_repo
            .create_tasks(auth_token, tasks)
            .await?;
        Ok(created_tasks)
    }

    #[allow(clippy::unused_async)]
    pub async fn process_sla_escalations(
        &self,
        _auth_token: &crate::diesel_pool::TenantAuthToken,
    ) -> AppResult<()> {
        Ok(())
    }
}
