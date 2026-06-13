use crate::diesel_pool::TenantAuthToken;
use crate::errors::AppError;
use crate::models::{
    ChecklistItemStatus, ChecklistStatus, CreateLifecycleChecklistTemplateItemRequest,
    CreateLifecycleChecklistTemplateRequest, LifecycleChecklistResponse,
    LifecycleChecklistTemplate, LifecycleChecklistTemplateItem, LifecycleChecklistType,
    PendingTaskResponse, UpdateLifecycleChecklistTemplateItemRequest,
    UpdateLifecycleChecklistTemplateRequest,
};
use crate::repositories::employee::EmployeeRepository;
use crate::repositories::employment_repository::EmploymentRepository;
use crate::repositories::lifecycle_checklist_repository::LifecycleChecklistRepository;
use crate::repositories::user_repository::UserRepository;
use crate::repositories::workspace_repository::WorkspaceRepository;
use chrono::Utc;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Clone)]
pub struct LifecycleService {
    repo: Arc<dyn LifecycleChecklistRepository>,
    employee_repo: Arc<dyn EmployeeRepository>,
    employment_repo: Arc<dyn EmploymentRepository>,
    #[allow(dead_code)]
    user_repo: Arc<dyn UserRepository>,
    workspace_repo: Arc<dyn WorkspaceRepository>,
}

impl LifecycleService {
    pub fn new(
        repo: Arc<dyn LifecycleChecklistRepository>,
        employee_repo: Arc<dyn EmployeeRepository>,
        employment_repo: Arc<dyn EmploymentRepository>,
        user_repo: Arc<dyn UserRepository>,
        workspace_repo: Arc<dyn WorkspaceRepository>,
    ) -> Self {
        Self {
            repo,
            employee_repo,
            employment_repo,
            user_repo,
            workspace_repo,
        }
    }

    // --- Templates ---
    pub async fn create_template(
        &self,
        workspace_id: Uuid,
        data: CreateLifecycleChecklistTemplateRequest,
    ) -> Result<LifecycleChecklistTemplate, AppError> {
        if data.name.trim().is_empty() {
            return Err(AppError::Validation("error.template_name_empty".into()));
        }
        let auth_token = TenantAuthToken::system_override(workspace_id);
        self.repo.create_template(&auth_token, data).await
    }

    pub async fn get_template(
        &self,
        workspace_id: Uuid,
        id: Uuid,
    ) -> Result<LifecycleChecklistTemplate, AppError> {
        let auth_token = TenantAuthToken::system_override(workspace_id);
        self.repo
            .get_template_by_id(&auth_token, id)
            .await?
            .ok_or_else(|| AppError::NotFound("error.template_not_found".into()))
    }

    pub async fn list_active_templates(
        &self,
        workspace_id: Uuid,
        checklist_type: Option<LifecycleChecklistType>,
    ) -> Result<Vec<LifecycleChecklistTemplate>, AppError> {
        let auth_token = TenantAuthToken::system_override(workspace_id);
        self.repo
            .list_active_templates(&auth_token, checklist_type)
            .await
    }

    pub async fn update_template(
        &self,
        workspace_id: Uuid,
        id: Uuid,
        data: UpdateLifecycleChecklistTemplateRequest,
    ) -> Result<LifecycleChecklistTemplate, AppError> {
        let auth_token = TenantAuthToken::system_override(workspace_id);
        self.repo.update_template(&auth_token, id, data).await
    }

    pub async fn delete_template(&self, workspace_id: Uuid, id: Uuid) -> Result<(), AppError> {
        let auth_token = TenantAuthToken::system_override(workspace_id);
        self.repo.soft_delete_template(&auth_token, id).await
    }

    // --- Template Items ---
    pub async fn create_template_item(
        &self,
        workspace_id: Uuid,
        template_id: Uuid,
        data: CreateLifecycleChecklistTemplateItemRequest,
    ) -> Result<LifecycleChecklistTemplateItem, AppError> {
        if data.title.trim().is_empty() {
            return Err(AppError::Validation(
                "error.template_item_title_empty".into(),
            ));
        }
        let auth_token = TenantAuthToken::system_override(workspace_id);
        // Verify template exists
        let _ = self.get_template(workspace_id, template_id).await?;
        self.repo
            .create_template_item(&auth_token, template_id, data)
            .await
    }

    pub async fn get_template_item(
        &self,
        workspace_id: Uuid,
        id: Uuid,
    ) -> Result<LifecycleChecklistTemplateItem, AppError> {
        let auth_token = TenantAuthToken::system_override(workspace_id);
        self.repo
            .get_template_item_by_id(&auth_token, id)
            .await?
            .ok_or_else(|| AppError::NotFound("error.template_item_not_found".into()))
    }

    pub async fn list_template_items(
        &self,
        workspace_id: Uuid,
        template_id: Uuid,
    ) -> Result<Vec<LifecycleChecklistTemplateItem>, AppError> {
        let auth_token = TenantAuthToken::system_override(workspace_id);
        self.repo
            .list_template_items(&auth_token, template_id)
            .await
    }

    pub async fn update_template_item(
        &self,
        workspace_id: Uuid,
        id: Uuid,
        data: UpdateLifecycleChecklistTemplateItemRequest,
    ) -> Result<LifecycleChecklistTemplateItem, AppError> {
        let auth_token = TenantAuthToken::system_override(workspace_id);
        self.repo.update_template_item(&auth_token, id, data).await
    }

    pub async fn delete_template_item(&self, workspace_id: Uuid, id: Uuid) -> Result<(), AppError> {
        let auth_token = TenantAuthToken::system_override(workspace_id);
        self.repo.delete_template_item(&auth_token, id).await
    }

    // --- Checklist Instances ---
    #[allow(clippy::too_many_lines)]
    pub async fn create_checklist_from_template(
        &self,
        workspace_id: Uuid,
        employment_id: Uuid,
        template_id: Uuid,
        actor_id: Option<Uuid>,
    ) -> Result<LifecycleChecklistResponse, AppError> {
        let auth_token = TenantAuthToken::system_override(workspace_id);

        let template = self
            .repo
            .get_template_by_id(&auth_token, template_id)
            .await?
            .ok_or_else(|| AppError::NotFound("error.template_not_found".into()))?;

        let template_items = self
            .repo
            .list_template_items(&auth_token, template_id)
            .await?;

        let employment = self
            .employment_repo
            .get_employment_by_id(&auth_token, employment_id)
            .await?
            .ok_or_else(|| AppError::NotFound("error.employment_not_found".into()))?;

        let employee = self
            .employee_repo
            .get_employee_by_id(employment.employee_id(), &auth_token)
            .await?
            .ok_or_else(|| AppError::NotFound("error.employee_not_found".into()))?;

        let base_date = match template.checklist_type() {
            LifecycleChecklistType::Onboarding
            | LifecycleChecklistType::Transfer
            | LifecycleChecklistType::ProbationEnd => employment.start_date(),
            LifecycleChecklistType::Offboarding => employment
                .end_date()
                .unwrap_or_else(|| chrono::Utc::now().naive_utc().date()),
        };

        let checklist = self
            .repo
            .create_checklist(
                &auth_token,
                employment_id,
                Some(template_id),
                template.checklist_type(),
                actor_id,
            )
            .await?;

        let mut item_entities = Vec::new();
        for item in template_items {
            let due_date =
                Some(base_date + chrono::Duration::days(i64::from(item.due_days_offset())));

            let assignee_user_id = match item.assignee_type() {
                crate::models::ChecklistAssigneeType::SpecificUser => item.assignee_user_id(),
                crate::models::ChecklistAssigneeType::Employee => employee.user_id(),
                crate::models::ChecklistAssigneeType::DirectManager => {
                    if let Some(mgr_emp_id) = employment.manager_employment_id() {
                        if let Ok(Some(mgr_emp)) = self
                            .employment_repo
                            .get_employment_by_id(&auth_token, mgr_emp_id)
                            .await
                        {
                            if let Ok(Some(mgr_employee)) = self
                                .employee_repo
                                .get_employee_by_id(mgr_emp.employee_id(), &auth_token)
                                .await
                            {
                                mgr_employee.user_id()
                            } else {
                                None
                            }
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                }
                crate::models::ChecklistAssigneeType::HrManager => {
                    if let Ok(users) = self
                        .workspace_repo
                        .get_users_with_permission(workspace_id, "manage_onboarding")
                        .await
                    {
                        users.first().copied()
                    } else if let Ok(users) = self
                        .workspace_repo
                        .get_users_with_permission(workspace_id, "admin")
                        .await
                    {
                        users.first().copied()
                    } else {
                        None
                    }
                }
                crate::models::ChecklistAssigneeType::ItAdmin => {
                    if let Ok(users) = self
                        .workspace_repo
                        .get_users_with_permission(workspace_id, "manage_it")
                        .await
                    {
                        users.first().copied()
                    } else if let Ok(users) = self
                        .workspace_repo
                        .get_users_with_permission(workspace_id, "admin")
                        .await
                    {
                        users.first().copied()
                    } else {
                        None
                    }
                }
            };

            item_entities.push(
                crate::models::diesel::lifecycle_checklist::LifecycleChecklistItemEntity {
                    id: Uuid::new_v4(),
                    checklist_id: checklist.id(),
                    template_item_id: Some(item.id()),
                    title: item.title().to_string(),
                    description: item.description().map(String::from),
                    category: item.category().as_str().to_string(),
                    assignee_user_id,
                    is_required: item.is_required(),
                    due_date,
                    status: ChecklistItemStatus::Pending.as_str().to_string(),
                    completed_at: None,
                    completed_by: None,
                    notes: None,
                    display_order: item.display_order(),
                    created_at: Utc::now(),
                    updated_at: Utc::now(),
                },
            );
        }

        let items = self
            .repo
            .create_checklist_items(&auth_token, item_entities)
            .await?;

        // If template had no items, complete checklist immediately
        let (updated_checklist, updated_items) = if items.is_empty() {
            let cl = self
                .repo
                .update_checklist_status_and_progress(
                    &auth_token,
                    checklist.id(),
                    ChecklistStatus::Completed,
                    100,
                    Some(Utc::now()),
                )
                .await?;
            (cl, items)
        } else {
            (checklist, items)
        };

        Ok(LifecycleChecklistResponse {
            id: updated_checklist.id(),
            workspace_id: updated_checklist.workspace_id(),
            employment_id: updated_checklist.employment_id(),
            template_id: updated_checklist.template_id(),
            checklist_type: updated_checklist.checklist_type(),
            status: updated_checklist.status(),
            progress_percent: updated_checklist.progress_percent(),
            started_at: updated_checklist.started_at(),
            completed_at: updated_checklist.completed_at(),
            created_by: updated_checklist.created_by(),
            created_at: updated_checklist.created_at(),
            updated_at: updated_checklist.updated_at(),
            items: updated_items,
        })
    }

    pub async fn get_checklist(
        &self,
        workspace_id: Uuid,
        id: Uuid,
    ) -> Result<LifecycleChecklistResponse, AppError> {
        let auth_token = TenantAuthToken::system_override(workspace_id);
        let checklist = self
            .repo
            .get_checklist_by_id(&auth_token, id)
            .await?
            .ok_or_else(|| AppError::NotFound("error.checklist_not_found".into()))?;

        let items = self.repo.list_checklist_items(&auth_token, id).await?;

        Ok(LifecycleChecklistResponse {
            id: checklist.id(),
            workspace_id: checklist.workspace_id(),
            employment_id: checklist.employment_id(),
            template_id: checklist.template_id(),
            checklist_type: checklist.checklist_type(),
            status: checklist.status(),
            progress_percent: checklist.progress_percent(),
            started_at: checklist.started_at(),
            completed_at: checklist.completed_at(),
            created_by: checklist.created_by(),
            created_at: checklist.created_at(),
            updated_at: checklist.updated_at(),
            items,
        })
    }

    pub async fn list_checklists_for_employment(
        &self,
        workspace_id: Uuid,
        employment_id: Uuid,
    ) -> Result<Vec<LifecycleChecklistResponse>, AppError> {
        let auth_token = TenantAuthToken::system_override(workspace_id);
        let checklists = self
            .repo
            .list_checklists_for_employment(&auth_token, employment_id)
            .await?;

        let mut responses = Vec::new();
        for cl in checklists {
            let items = self.repo.list_checklist_items(&auth_token, cl.id()).await?;
            responses.push(LifecycleChecklistResponse {
                id: cl.id(),
                workspace_id: cl.workspace_id(),
                employment_id: cl.employment_id(),
                template_id: cl.template_id(),
                checklist_type: cl.checklist_type(),
                status: cl.status(),
                progress_percent: cl.progress_percent(),
                started_at: cl.started_at(),
                completed_at: cl.completed_at(),
                created_by: cl.created_by(),
                created_at: cl.created_at(),
                updated_at: cl.updated_at(),
                items,
            });
        }
        Ok(responses)
    }

    pub async fn update_item_status(
        &self,
        workspace_id: Uuid,
        item_id: Uuid,
        status: ChecklistItemStatus,
        notes: Option<String>,
        completed_by: Option<Uuid>,
    ) -> Result<LifecycleChecklistResponse, AppError> {
        let auth_token = TenantAuthToken::system_override(workspace_id);

        let item = self
            .repo
            .get_checklist_item_by_id(&auth_token, item_id)
            .await?
            .ok_or_else(|| AppError::NotFound("error.checklist_item_not_found".into()))?;

        self.repo
            .update_checklist_item_status(&auth_token, item_id, status, completed_by, notes)
            .await?;

        let items = self
            .repo
            .list_checklist_items(&auth_token, item.checklist_id())
            .await?;

        let total_items = items.len();
        let completed_items = items
            .iter()
            .filter(|i| {
                i.status() == ChecklistItemStatus::Completed
                    || i.status() == ChecklistItemStatus::Skipped
            })
            .count();

        let progress_percent = if total_items == 0 {
            100
        } else {
            ((completed_items as f64 / total_items as f64) * 100.0).round() as i32
        };

        let new_status = if progress_percent == 100 {
            ChecklistStatus::Completed
        } else {
            ChecklistStatus::InProgress
        };

        let completed_at = if new_status == ChecklistStatus::Completed {
            Some(Utc::now())
        } else {
            None
        };

        let checklist = self
            .repo
            .update_checklist_status_and_progress(
                &auth_token,
                item.checklist_id(),
                new_status,
                progress_percent,
                completed_at,
            )
            .await?;

        Ok(LifecycleChecklistResponse {
            id: checklist.id(),
            workspace_id: checklist.workspace_id(),
            employment_id: checklist.employment_id(),
            template_id: checklist.template_id(),
            checklist_type: checklist.checklist_type(),
            status: checklist.status(),
            progress_percent: checklist.progress_percent(),
            started_at: checklist.started_at(),
            completed_at: checklist.completed_at(),
            created_by: checklist.created_by(),
            created_at: checklist.created_at(),
            updated_at: checklist.updated_at(),
            items,
        })
    }

    pub async fn get_pending_tasks_for_user(
        &self,
        workspace_id: Uuid,
        user_id: Uuid,
    ) -> Result<Vec<PendingTaskResponse>, AppError> {
        let auth_token = TenantAuthToken::system_override(workspace_id);
        self.repo
            .get_pending_tasks_for_user(&auth_token, user_id)
            .await
    }

    // --- Auto Trigger Rules ---
    pub async fn auto_trigger_onboarding(
        &self,
        workspace_id: Uuid,
        employment_id: Uuid,
        actor_id: Option<Uuid>,
    ) -> Result<(), AppError> {
        let auth_token = TenantAuthToken::system_override(workspace_id);
        if let Some(template) = self
            .repo
            .get_default_template(&auth_token, LifecycleChecklistType::Onboarding)
            .await?
        {
            let _ = self
                .create_checklist_from_template(
                    workspace_id,
                    employment_id,
                    template.id(),
                    actor_id,
                )
                .await?;
        }
        Ok(())
    }

    pub async fn auto_trigger_offboarding(
        &self,
        workspace_id: Uuid,
        employment_id: Uuid,
        actor_id: Option<Uuid>,
    ) -> Result<(), AppError> {
        let auth_token = TenantAuthToken::system_override(workspace_id);
        if let Some(template) = self
            .repo
            .get_default_template(&auth_token, LifecycleChecklistType::Offboarding)
            .await?
        {
            let _ = self
                .create_checklist_from_template(
                    workspace_id,
                    employment_id,
                    template.id(),
                    actor_id,
                )
                .await?;
        }
        Ok(())
    }
}
