use super::ApprovalWorkflowService;
use crate::errors::AppResult;
use crate::models::ApprovalWorkflowTask;
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
            role_ids.push(r.role_id());
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
        let workspace_tz = self
            .workspace_repo
            .get_workspace_by_id(auth_token.workspace_id())
            .await
            .ok()
            .flatten()
            .map(|ws| crate::utils::tz::parse_workspace_tz(ws.timezone()))
            .unwrap_or(chrono_tz::UTC);
        let today = crate::utils::tz::utc_now_date_in_tz(&workspace_tz);
        self.workspace_config_repo
            .get_effective_delegations(auth_token, auth_token.workspace_id(), user_id, today)
            .await
            .unwrap_or_default()
    }

    pub async fn get_pending_tasks_for_user(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        user_id: Uuid,
        page: i64,
        limit: i64,
    ) -> AppResult<(Vec<ApprovalWorkflowTask>, i64)> {
        let workspace_id = auth_token.workspace_id();

        // 1. Resolve context for the main user
        let user_ctx = self
            .resolve_user_approval_context(auth_token, workspace_id, user_id)
            .await?;

        // 2. Fetch delegators
        let delegators = self.get_delegators_for_user(auth_token, user_id).await;

        // 3. Fetch pending tasks for the main user
        let (mut all_tasks, _) = self
            .approval_workflow_repo
            .get_pending_tasks_for_user(
                auth_token,
                user_id,
                user_ctx.role_ids,
                user_ctx.permissions,
                user_ctx.managed_dept_ids,
                1,
                i64::MAX,
            )
            .await?;

        // 4. Fetch pending tasks for each delegator
        for &delegator_id in &delegators {
            let del_ctx = self
                .resolve_user_approval_context(auth_token, workspace_id, delegator_id)
                .await?;
            let (del_tasks, _) = self
                .approval_workflow_repo
                .get_pending_tasks_for_user(
                    auth_token,
                    delegator_id,
                    del_ctx.role_ids,
                    del_ctx.permissions,
                    del_ctx.managed_dept_ids,
                    1,
                    i64::MAX,
                )
                .await?;
            all_tasks.extend(del_tasks);
        }

        all_tasks.sort_by_key(ApprovalWorkflowTask::id);
        all_tasks.dedup_by_key(|t| t.id());

        // Batch-fetch all instance IDs to avoid N+1 per-task instance lookup
        let instance_ids: Vec<Uuid> = all_tasks
            .iter()
            .map(crate::models::approval_workflow::ApprovalWorkflowTask::instance_id)
            .collect();
        let instances = self
            .approval_workflow_repo
            .get_instances_by_ids(auth_token, &instance_ids)
            .await
            .unwrap_or_default();
        let instances_by_id: std::collections::HashMap<Uuid, _> =
            instances.into_iter().map(|inst| (inst.id, inst)).collect();

        // Batch-resolve beneficiaries to avoid N+1 queries
        let instances_vec: Vec<_> = instances_by_id.values().cloned().collect();
        let beneficiaries_map = self
            .resolve_beneficiaries_batch(auth_token, &instances_vec)
            .await;

        let mut final_tasks = Vec::new();
        for task in all_tasks {
            if let Some(instance) = instances_by_id.get(&task.instance_id()) {
                let beneficiary_user_id = beneficiaries_map
                    .get(&(instance.entity_type.clone(), instance.entity_id))
                    .copied()
                    .flatten();

                let is_self_approving = beneficiary_user_id
                    .is_some_and(|b_id| b_id == user_id || delegators.contains(&b_id));
                if !is_self_approving {
                    final_tasks.push(task);
                }
            } else {
                final_tasks.push(task);
            }
        }

        // Apply pagination
        let total = i64::try_from(final_tasks.len()).unwrap_or(i64::MAX);
        let limit_usize = usize::try_from(limit).unwrap_or(usize::MAX);
        let offset =
            usize::try_from(page.saturating_sub(1).saturating_mul(limit)).unwrap_or(usize::MAX);
        let paginated_tasks = if offset >= final_tasks.len() {
            Vec::new()
        } else {
            final_tasks
                .into_iter()
                .skip(offset)
                .take(limit_usize)
                .collect()
        };

        Ok((paginated_tasks, total))
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
                        Some(current_dept.department_id()),
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
