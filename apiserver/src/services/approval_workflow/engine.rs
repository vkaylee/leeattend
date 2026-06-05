use super::ApprovalWorkflowService;
use crate::errors::{AppError, AppResult};
use crate::models::{
    ApprovalActionLog, ApprovalWorkflowInstance, ApprovalWorkflowTask, ApproverActionInfo,
    WorkflowInstanceStepSnapshot,
};
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
                    info.employee_id = Some(req.employee_id);
                    if let Ok(Some(emp)) = self
                        .employee_repo
                        .get_employee_by_id(req.employee_id, auth_token)
                        .await
                    {
                        info.user_id = emp.user_id();
                    }
                }
            }
            "ot_request" => {
                if let Ok(Some(req)) = self
                    .overtime_request_repo
                    .get_request(auth_token, auth_token.workspace_id(), entity_id)
                    .await
                {
                    info.user_id = Some(req.requested_by_user_id);
                    if let Ok(Some(emp_id)) = self
                        .employee_repo
                        .get_employee_id_by_user_id(auth_token, req.requested_by_user_id)
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
                    info.employee_id = Some(summary.employee_id);
                    if let Ok(Some(emp)) = self
                        .employee_repo
                        .get_employee_by_id(summary.employee_id, auth_token)
                        .await
                    {
                        info.user_id = emp.user_id();
                    }
                }
            }
            _ => {}
        }
        info
    }

    pub async fn get_beneficiary_user_id_for_entity(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        entity_type: &str,
        entity_id: Uuid,
    ) -> Option<Uuid> {
        self.resolve_beneficiary_info(auth_token, entity_type, entity_id)
            .await
            .user_id
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
                        .map(|v| v as i32);

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
        action: String,
        note: Option<String>,
        new_approver_user_id: Option<Uuid>,
    ) -> AppResult<ApprovalWorkflowInstance> {
        let workspace_id = auth_token.workspace_id();

        let mut instance = self
            .approval_workflow_repo
            .get_instance_by_id(auth_token, instance_id)
            .await?
            .ok_or_else(|| {
                AppError::NotFound(format!("Workflow instance not found: {instance_id}"))
            })?;

        if instance.status != "pending" {
            return Err(AppError::Validation(format!(
                "Workflow instance is already {}",
                instance.status
            )));
        }

        let mut steps: Vec<WorkflowInstanceStepSnapshot> =
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

        let user_pending_tasks = self.get_pending_tasks_for_user(auth_token, user_id).await?;

        let matching_task = active_tasks
            .iter()
            .find(|at| {
                at.step_order() == current_step_order
                    && user_pending_tasks.iter().any(|pt| pt.id() == at.id())
            })
            .cloned();

        let is_authorized = is_admin || matching_task.is_some();
        if !is_authorized {
            return Err(AppError::Forbidden(
                "You are not authorized to approve this step".to_string(),
            ));
        }

        let db_action = match action.as_str() {
            "approve" | "approved" => Some("approved".to_string()),
            "reject" | "rejected" => Some("rejected".to_string()),
            "return" | "returned" => Some("returned".to_string()),
            "skip" | "skipped" => Some("skipped".to_string()),
            _ => None,
        };

        if let Some(db_act) = db_action {
            let log_entry = ApprovalActionLog {
                id: Uuid::new_v4(),
                workspace_id,
                entity_type: instance.entity_type.clone(),
                entity_id: instance.entity_id,
                step_order: current_step_order,
                step_name: Some(steps[current_step_idx].step_name.clone()),
                acted_by_user_id: user_id,
                action: db_act,
                note: note.clone(),
                acted_at: Utc::now(),
            };
            self.log_action(auth_token, log_entry).await?;
        }

        steps[current_step_idx]
            .approvers_acted
            .push(ApproverActionInfo {
                user_id,
                action: action.clone(),
                note: note.clone(),
                acted_at: Utc::now(),
            });

        match action.as_str() {
            "reassign" => {
                let target_uid = new_approver_user_id.ok_or_else(|| {
                    AppError::Validation(
                        "new_approver_user_id is required for reassignment".to_string(),
                    )
                })?;

                for mut task in active_tasks {
                    if task.step_order() == current_step_order {
                        task.cancel();
                        self.approval_workflow_repo
                            .update_task(auth_token, task)
                            .await?;
                    }
                }

                let reassigned_task = ApprovalWorkflowTask::new(
                    instance_id,
                    current_step_order,
                    "specific_user".to_string(),
                    Some(target_uid),
                    None,
                );
                self.approval_workflow_repo
                    .create_tasks(auth_token, vec![reassigned_task])
                    .await?;

                steps[current_step_idx].resolved_approver_ids = vec![target_uid];
            }
            "reject" => {
                steps[current_step_idx].status = "rejected".to_string();
                instance.status = "rejected".to_string();

                for mut task in active_tasks {
                    if task.step_order() == current_step_order {
                        if let Some(mt) = matching_task.as_ref() {
                            if task.id() == mt.id() {
                                task.complete(user_id);
                            } else {
                                task.cancel();
                            }
                        } else {
                            task.cancel();
                        }
                        self.approval_workflow_repo
                            .update_task(auth_token, task)
                            .await?;
                    }
                }
            }
            "return" => {
                steps[current_step_idx].status = "returned".to_string();
                instance.status = "returned".to_string();

                for mut task in active_tasks {
                    if task.step_order() == current_step_order {
                        if let Some(mt) = matching_task.as_ref() {
                            if task.id() == mt.id() {
                                task.complete(user_id);
                            } else {
                                task.cancel();
                            }
                        } else {
                            task.cancel();
                        }
                        self.approval_workflow_repo
                            .update_task(auth_token, task)
                            .await?;
                    }
                }
            }
            "approve" => {
                let consensus_mode = &steps[current_step_idx].consensus_mode;

                if let Some(mt) = matching_task.as_ref() {
                    let mut task_to_update = mt.clone();
                    task_to_update.complete(user_id);
                    self.approval_workflow_repo
                        .update_task(auth_token, task_to_update)
                        .await?;
                }

                let step_approved = if is_admin {
                    for mut task in active_tasks {
                        if task.step_order() == current_step_order {
                            task.cancel();
                            self.approval_workflow_repo
                                .update_task(auth_token, task)
                                .await?;
                        }
                    }
                    true
                } else if consensus_mode == "any" {
                    for mut task in active_tasks {
                        if task.step_order() == current_step_order {
                            if let Some(mt) = matching_task.as_ref() {
                                if task.id() != mt.id() {
                                    task.cancel();
                                    self.approval_workflow_repo
                                        .update_task(auth_token, task)
                                        .await?;
                                }
                            }
                        }
                    }
                    true
                } else {
                    let remaining_active_tasks = self
                        .approval_workflow_repo
                        .get_active_tasks_for_instance(auth_token, instance_id)
                        .await?;
                    let has_remaining_for_step = remaining_active_tasks
                        .iter()
                        .any(|t| t.step_order() == current_step_order);
                    !has_remaining_for_step
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

                        let beneficiary_employee_id = self
                            .resolve_beneficiary_info(
                                auth_token,
                                &instance.entity_type,
                                instance.entity_id,
                            )
                            .await
                            .employee_id
                            .unwrap_or(Uuid::nil());

                        self.spawn_tasks_for_step(
                            auth_token,
                            instance.id,
                            next.step_order,
                            &next.approver_type,
                            &next.approver_config,
                            beneficiary_employee_id,
                        )
                        .await?;
                    } else {
                        instance.status = "approved".to_string();
                    }
                }
            }
            _ => return Err(AppError::Validation(format!("Invalid action: {action}"))),
        }

        instance.steps_snapshot =
            serde_json::to_value(&steps).map_err(|e| AppError::Internal(e.to_string()))?;
        instance.updated_at = Utc::now();

        let updated_instance = self
            .approval_workflow_repo
            .update_instance(auth_token, instance)
            .await?;

        if updated_instance.status != "pending" {
            let entity_id = updated_instance.entity_id;
            let entity_type = &updated_instance.entity_type;
            if let Some(handler) = self.handlers.get(entity_type) {
                handler
                    .on_workflow_completed(auth_token, entity_id, &updated_instance.status, user_id)
                    .await?;
            }
        }

        Ok(updated_instance)
    }
}
