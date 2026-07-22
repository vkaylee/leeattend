use super::ApprovalWorkflowService;
use crate::errors::{AppError, AppResult};
use crate::models::{ApprovalAction, ApprovalWorkflowInstance, WorkflowInstanceStepSnapshot};
use crate::utils::i18n::I18nKey;
use chrono::Utc;
use serde_json::json;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct BeneficiaryInfo {
    pub user_id: Option<Uuid>,
    pub employee_id: Option<Uuid>,
}

impl ApprovalWorkflowService {
    pub(crate) async fn resolve_beneficiary_info(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        entity_type: &str,
        entity_id: Uuid,
    ) -> BeneficiaryInfo {
        let mut info = BeneficiaryInfo {
            user_id: None,
            employee_id: None,
        };
        match entity_type {
            "leave_request" => {
                if let Ok(Some(req)) = self
                    .leave_request_repo
                    .get_leave_request_by_id(auth_token, entity_id)
                    .await
                {
                    if let Ok(Some(employment)) = self
                        .employment_repo
                        .get_employment_by_id(auth_token, req.employment_id())
                        .await
                    {
                        let emp_id = employment.employee_id();
                        info.employee_id = Some(emp_id);
                        if let Ok(Some(emp)) = self
                            .employee_repo
                            .get_employee_by_id(emp_id, auth_token)
                            .await
                        {
                            info.user_id = emp.user_id();
                        }
                    }
                }
            }
            "ot_request" => {
                if let Ok(Some(req)) = self
                    .overtime_request_repo
                    .get_request(auth_token, auth_token.workspace_id(), entity_id)
                    .await
                {
                    info.user_id = Some(req.requested_by_user_id());
                    if let Ok(Some(emp_id)) = self
                        .employee_repo
                        .get_employee_id_by_user_id(auth_token, req.requested_by_user_id())
                        .await
                    {
                        info.employee_id = Some(emp_id);
                    }
                }
            }
            "ot_claim" => {
                if let Ok(Some(summary)) = self
                    .daily_summary_repo
                    .get_summary_by_id(auth_token, entity_id)
                    .await
                {
                    if let Ok(Some(employment)) = self
                        .employment_repo
                        .get_employment_by_id(auth_token, summary.employment_id)
                        .await
                    {
                        let emp_id = employment.employee_id();
                        info.employee_id = Some(emp_id);
                        if let Ok(Some(emp)) = self
                            .employee_repo
                            .get_employee_by_id(emp_id, auth_token)
                            .await
                        {
                            info.user_id = emp.user_id();
                        }
                    }
                }
            }
            _ => {}
        }
        info
    }

    #[allow(clippy::too_many_lines)]
    pub async fn resolve_beneficiaries_batch(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        instances: &[crate::models::ApprovalWorkflowInstance],
    ) -> std::collections::HashMap<(String, Uuid), Option<Uuid>> {
        use std::collections::HashMap;

        let mut result: HashMap<(String, Uuid), Option<Uuid>> = HashMap::new();

        // Group instances by entity_type
        let mut leave_request_ids = Vec::new();
        let mut ot_request_ids = Vec::new();
        let mut ot_claim_ids = Vec::new();

        for instance in instances {
            match instance.entity_type.as_str() {
                "leave_request" => leave_request_ids.push(instance.entity_id),
                "ot_request" => ot_request_ids.push(instance.entity_id),
                "ot_claim" => ot_claim_ids.push(instance.entity_id),
                _ => {}
            }
        }

        // Batch fetch leave requests
        if !leave_request_ids.is_empty() {
            if let Ok(requests) = self
                .leave_request_repo
                .get_leave_requests_by_ids(auth_token, &leave_request_ids)
                .await
            {
                let employment_ids: Vec<Uuid> = requests
                    .iter()
                    .map(crate::models::leave::LeaveRequest::employment_id)
                    .collect();
                if let Ok(employments) = self
                    .employment_repo
                    .get_employments_by_ids(auth_token, &employment_ids)
                    .await
                {
                    let employee_ids: Vec<Uuid> = employments
                        .iter()
                        .map(crate::models::employment::Employment::employee_id)
                        .collect();
                    if let Ok(employees) = self
                        .employee_repo
                        .get_employees_by_ids(&employee_ids, auth_token)
                        .await
                    {
                        let emp_to_user: HashMap<Uuid, Uuid> = employees
                            .iter()
                            .filter_map(|e| e.user_id().map(|uid| (e.id(), uid)))
                            .collect();
                        let employment_to_emp: HashMap<Uuid, Uuid> = employments
                            .iter()
                            .map(|e| (e.id(), e.employee_id()))
                            .collect();

                        for req in requests {
                            if let Some(emp_id) = employment_to_emp.get(&req.employment_id()) {
                                if let Some(user_id) = emp_to_user.get(emp_id) {
                                    result.insert(
                                        ("leave_request".to_string(), req.id()),
                                        Some(*user_id),
                                    );
                                }
                            }
                        }
                    }
                }
            }
        }

        // Batch fetch overtime requests
        if !ot_request_ids.is_empty() {
            if let Ok(requests) = self
                .overtime_request_repo
                .get_requests_by_ids(auth_token, &ot_request_ids)
                .await
            {
                for req in requests {
                    result.insert(
                        ("ot_request".to_string(), req.id()),
                        Some(req.requested_by_user_id()),
                    );
                }
            }
        }

        // Batch fetch attendance daily summaries (ot_claim)
        if !ot_claim_ids.is_empty() {
            if let Ok(summaries) = self
                .daily_summary_repo
                .get_summaries_by_ids(auth_token, &ot_claim_ids)
                .await
            {
                let employment_ids: Vec<Uuid> = summaries.iter().map(|s| s.employment_id).collect();
                if let Ok(employments) = self
                    .employment_repo
                    .get_employments_by_ids(auth_token, &employment_ids)
                    .await
                {
                    let employee_ids: Vec<Uuid> = employments
                        .iter()
                        .map(crate::models::employment::Employment::employee_id)
                        .collect();
                    if let Ok(employees) = self
                        .employee_repo
                        .get_employees_by_ids(&employee_ids, auth_token)
                        .await
                    {
                        let emp_to_user: HashMap<Uuid, Uuid> = employees
                            .iter()
                            .filter_map(|e| e.user_id().map(|uid| (e.id(), uid)))
                            .collect();
                        let employment_to_emp: HashMap<Uuid, Uuid> = employments
                            .iter()
                            .map(|e| (e.id(), e.employee_id()))
                            .collect();

                        for summary in summaries {
                            if let Some(emp_id) = employment_to_emp.get(&summary.employment_id) {
                                if let Some(user_id) = emp_to_user.get(emp_id) {
                                    result.insert(
                                        ("ot_claim".to_string(), summary.id),
                                        Some(*user_id),
                                    );
                                }
                            }
                        }
                    }
                }
            }
        }

        result
    }

    #[allow(clippy::too_many_lines)]
    pub async fn submit_to_workflow(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        entity_type: String,
        entity_id: Uuid,
        beneficiary_employee_id: Uuid,
    ) -> AppResult<ApprovalWorkflowInstance> {
        let workspace_id = auth_token.workspace_id();

        let active_wf = self.get_active_workflow(auth_token, &entity_type).await?;
        let active_wf_id = active_wf.as_ref().map(|w| w.id);

        let steps_snapshot = match active_wf {
            None => {
                let step = WorkflowInstanceStepSnapshot {
                    step_order: 1,
                    step_name: "Direct Manager Approval (Fallback)".to_string(),
                    approver_type: "department_manager".to_string(),
                    approver_config: json!({}),
                    resolved_approver_ids: vec![],
                    consensus_mode: "any".to_string(),
                    action_required: "approve".to_string(),
                    can_skip: false,
                    status: "pending".to_string(),
                    sla_hours: None,
                    escalation_action: None,
                    approvers_acted: vec![],
                };
                serde_json::to_value(vec![step]).map_err(|e| AppError::Internal(e.to_string()))?
            }
            Some(wf) => {
                let wf_steps = self.get_steps(auth_token, wf.id).await?;
                let mut snapshots = Vec::new();
                for step in wf_steps {
                    let consensus_mode = step
                        .approver_config
                        .get("consensus_mode")
                        .and_then(|v| v.as_str())
                        .unwrap_or("any")
                        .to_string();

                    let sla_hours = step
                        .approver_config
                        .get("sla_hours")
                        .and_then(serde_json::Value::as_i64)
                        .and_then(|v| i32::try_from(v).ok());

                    let escalation_action = step.approver_config.get("escalation_action").cloned();

                    snapshots.push(WorkflowInstanceStepSnapshot {
                        step_order: step.step_order,
                        step_name: step.step_name,
                        approver_type: step.approver_type,
                        approver_config: step.approver_config,
                        resolved_approver_ids: vec![],
                        consensus_mode,
                        action_required: step.action_required,
                        can_skip: step.can_skip,
                        status: if step.step_order == 1 {
                            "pending".to_string()
                        } else {
                            "queued".to_string()
                        },
                        sla_hours,
                        escalation_action,
                        approvers_acted: vec![],
                    });
                }
                serde_json::to_value(snapshots).map_err(|e| AppError::Internal(e.to_string()))?
            }
        };

        if let Ok(Some(existing)) = self
            .approval_workflow_repo
            .get_instance_by_entity(auth_token, &entity_type, entity_id)
            .await
        {
            let old_instance_id = existing.id;
            let mut old_instance = existing;
            old_instance.status = "cancelled".to_string();
            let _ = self
                .approval_workflow_repo
                .update_instance(auth_token, old_instance)
                .await;

            if let Ok(active_tasks) = self
                .approval_workflow_repo
                .get_active_tasks_for_instance(auth_token, old_instance_id)
                .await
            {
                for mut task in active_tasks {
                    task.cancel();
                    let _ = self
                        .approval_workflow_repo
                        .update_task(auth_token, task)
                        .await;
                }
            }
        }

        let instance = ApprovalWorkflowInstance {
            id: Uuid::new_v4(),
            workspace_id,
            workflow_id: active_wf_id,
            entity_type,
            entity_id,
            status: "pending".to_string(),
            current_step_order: 1,
            steps_snapshot,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let created_instance = self
            .approval_workflow_repo
            .create_instance(auth_token, instance)
            .await?;

        let steps: Vec<WorkflowInstanceStepSnapshot> =
            serde_json::from_value(created_instance.steps_snapshot.clone())
                .map_err(|e| AppError::Internal(format!("Failed to parse steps snapshot: {e}")))?;

        if let Some(first_step) = steps.iter().find(|s| s.step_order == 1) {
            self.spawn_tasks_for_step(
                auth_token,
                created_instance.id,
                1,
                &first_step.approver_type,
                &first_step.approver_config,
                beneficiary_employee_id,
            )
            .await?;
        }

        Ok(created_instance)
    }

    #[allow(clippy::too_many_lines)]
    pub async fn process_approval_action(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        user_id: Uuid,
        instance_id: Uuid,
        action: ApprovalAction,
        note: Option<String>,
        new_approver_user_id: Option<Uuid>,
    ) -> AppResult<ApprovalWorkflowInstance> {
        let workspace_id = auth_token.workspace_id();

        // 1. Authorization checks (BEFORE transaction)
        let instance = self
            .approval_workflow_repo
            .get_instance_by_id(auth_token, instance_id)
            .await?
            .ok_or_else(|| AppError::not_found(I18nKey::WorkflowInstanceNotFound))?;

        if instance.status != "pending" {
            return Err(AppError::validation_args(
                I18nKey::Validation,
                vec![(
                    "details",
                    format!("Workflow instance is already {}", instance.status),
                )],
            ));
        }

        let steps: Vec<WorkflowInstanceStepSnapshot> =
            serde_json::from_value(instance.steps_snapshot.clone())
                .map_err(|e| AppError::Internal(format!("Failed to parse steps snapshot: {e}")))?;

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

        let active_tasks = self
            .approval_workflow_repo
            .get_active_tasks_for_instance(auth_token, instance_id)
            .await?;

        let admins = self
            .workspace_repo
            .get_admins_with_permissions(workspace_id, &[])
            .await?;
        let is_admin = admins.contains(&user_id);

        let (user_pending_tasks, _) = self
            .get_pending_tasks_for_user(auth_token, user_id, 1, i64::MAX)
            .await?;

        let matching_task = active_tasks
            .iter()
            .find(|at| {
                at.step_order() == current_step_order
                    && user_pending_tasks.iter().any(|pt| pt.id() == at.id())
            })
            .cloned();

        let is_authorized = is_admin || matching_task.is_some();
        if !is_authorized {
            return Err(AppError::forbidden(I18nKey::Forbidden));
        }

        // 2. Validation checks (BEFORE transaction)
        if action == ApprovalAction::Skip && !steps[current_step_idx].can_skip {
            return Err(AppError::validation_args(
                I18nKey::Validation,
                vec![("details", "This step cannot be skipped".to_string())],
            ));
        }

        if action == ApprovalAction::Reassign {
            let target_uid = new_approver_user_id.ok_or_else(|| {
                AppError::validation_args(
                    I18nKey::Validation,
                    vec![(
                        "details",
                        "new_approver_user_id is required for reassignment".to_string(),
                    )],
                )
            })?;

            let (members, _) = self
                .workspace_repo
                .get_workspace_members(workspace_id, 1, i64::MAX)
                .await?;
            if !members.iter().any(|m| m.user_id() == target_uid) {
                return Err(AppError::validation_args(
                    I18nKey::Validation,
                    vec![(
                        "details",
                        "Reassign target user is not a workspace member".to_string(),
                    )],
                ));
            }
        }

        // 3. Call transactional method (atomic: audit log + task updates + instance update)
        let matching_task_id = matching_task
            .as_ref()
            .map(crate::models::approval_workflow::ApprovalWorkflowTask::id);
        let (updated_instance, outbox_payload) = self
            .approval_workflow_repo
            .apply_approval_action_transactional(
                auth_token,
                instance_id,
                user_id,
                action,
                note,
                new_approver_user_id,
                is_admin,
                matching_task_id,
            )
            .await?;

        // 4. Post-transaction: spawn tasks for next step if we advanced
        let updated_steps: Vec<WorkflowInstanceStepSnapshot> =
            serde_json::from_value(updated_instance.steps_snapshot.clone())
                .map_err(|e| AppError::Internal(format!("Failed to parse updated steps: {e}")))?;

        if updated_instance.current_step_order > current_step_order {
            // We advanced to a new step - spawn tasks for it
            if let Some(next_step) = updated_steps
                .iter()
                .find(|s| s.step_order == updated_instance.current_step_order)
            {
                let beneficiary_employee_id = self
                    .resolve_beneficiary_info(
                        auth_token,
                        &updated_instance.entity_type,
                        updated_instance.entity_id,
                    )
                    .await
                    .employee_id
                    .unwrap_or(Uuid::nil());

                self.spawn_tasks_for_step(
                    auth_token,
                    updated_instance.id,
                    next_step.step_order,
                    &next_step.approver_type,
                    &next_step.approver_config,
                    beneficiary_employee_id,
                )
                .await?;
            }
        }

        // 5. Create outbox job if workflow completed, or call callback directly if no outbox
        if let Some(payload) = outbox_payload {
            if let Some(outbox_service) = &self.outbox_service {
                let job_id = Uuid::new_v4();
                let payload_json = serde_json::to_value(&payload).map_err(|e| {
                    AppError::Internal(format!("Failed to serialize outbox payload: {e}"))
                })?;

                outbox_service
                    .create_job(workspace_id, job_id, "WorkflowCompleted", payload_json)
                    .await?;
            } else {
                // Fallback: call callback directly (for tests and simple cases)
                if let Some(handler) = self.handlers.get(&payload.entity_type) {
                    handler
                        .on_workflow_completed(
                            auth_token,
                            payload.entity_id,
                            &payload.final_status,
                            payload.actor_user_id,
                        )
                        .await?;
                }
            }
        }

        Ok(updated_instance)
    }
}
