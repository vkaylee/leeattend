use crate::diesel_pool::TenantAuthToken;
use crate::errors::AppError;
use crate::models::{
    ChecklistItemStatus, ChecklistStatus, CreateLifecycleChecklistTemplateItemRequest,
    CreateLifecycleChecklistTemplateRequest, LifecycleChecklistItem, LifecycleChecklistResponse,
    LifecycleChecklistTemplate, LifecycleChecklistTemplateItem, LifecycleChecklistType,
    PendingTaskResponse, UpdateLifecycleChecklistTemplateItemRequest,
    UpdateLifecycleChecklistTemplateRequest,
};
use crate::repositories::employee::EmployeeRepository;
use crate::repositories::employment_repository::EmploymentRepository;
use crate::repositories::lifecycle_checklist_repository::LifecycleChecklistRepository;
use crate::repositories::user_repository::UserRepository;
use crate::repositories::workspace_repository::WorkspaceRepository;
use crate::utils::i18n::I18nKey;
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
        auth_token: &TenantAuthToken,
        data: CreateLifecycleChecklistTemplateRequest,
    ) -> Result<LifecycleChecklistTemplate, AppError> {
        if data.name.trim().is_empty() {
            return Err(AppError::validation(I18nKey::TemplateNameEmpty));
        }

        self.repo.create_template(auth_token, data).await
    }

    pub async fn get_template(
        &self,
        auth_token: &TenantAuthToken,
        id: Uuid,
    ) -> Result<LifecycleChecklistTemplate, AppError> {
        self.repo
            .get_template_by_id(auth_token, id)
            .await?
            .ok_or_else(|| AppError::not_found(I18nKey::TemplateNotFound))
    }

    pub async fn list_active_templates(
        &self,
        auth_token: &TenantAuthToken,
        checklist_type: Option<LifecycleChecklistType>,
    ) -> Result<Vec<LifecycleChecklistTemplate>, AppError> {
        self.repo
            .list_active_templates(auth_token, checklist_type)
            .await
    }

    pub async fn update_template(
        &self,
        auth_token: &TenantAuthToken,
        id: Uuid,
        data: UpdateLifecycleChecklistTemplateRequest,
    ) -> Result<LifecycleChecklistTemplate, AppError> {
        self.repo.update_template(auth_token, id, data).await
    }

    pub async fn delete_template(
        &self,
        auth_token: &TenantAuthToken,
        id: Uuid,
    ) -> Result<(), AppError> {
        self.repo.soft_delete_template(auth_token, id).await
    }

    // --- Template Items ---
    pub async fn create_template_item(
        &self,
        auth_token: &TenantAuthToken,
        template_id: Uuid,
        data: CreateLifecycleChecklistTemplateItemRequest,
    ) -> Result<LifecycleChecklistTemplateItem, AppError> {
        if data.title.trim().is_empty() {
            return Err(AppError::validation(I18nKey::TemplateItemTitleEmpty));
        }

        // Verify template exists
        let _ = self.get_template(auth_token, template_id).await?;
        self.repo
            .create_template_item(auth_token, template_id, data)
            .await
    }

    pub async fn get_template_item(
        &self,
        auth_token: &TenantAuthToken,
        id: Uuid,
    ) -> Result<LifecycleChecklistTemplateItem, AppError> {
        self.repo
            .get_template_item_by_id(auth_token, id)
            .await?
            .ok_or_else(|| AppError::not_found(I18nKey::TemplateItemNotFound))
    }

    pub async fn list_template_items(
        &self,
        auth_token: &TenantAuthToken,
        template_id: Uuid,
    ) -> Result<Vec<LifecycleChecklistTemplateItem>, AppError> {
        self.repo.list_template_items(auth_token, template_id).await
    }

    pub async fn update_template_item(
        &self,
        auth_token: &TenantAuthToken,
        id: Uuid,
        data: UpdateLifecycleChecklistTemplateItemRequest,
    ) -> Result<LifecycleChecklistTemplateItem, AppError> {
        self.repo.update_template_item(auth_token, id, data).await
    }

    pub async fn delete_template_item(
        &self,
        auth_token: &TenantAuthToken,
        id: Uuid,
    ) -> Result<(), AppError> {
        self.repo.delete_template_item(auth_token, id).await
    }

    // --- Checklist Instances ---
    #[allow(clippy::too_many_lines)]
    pub async fn create_checklist_from_template(
        &self,
        auth_token: &TenantAuthToken,
        employment_id: Uuid,
        template_id: Uuid,
        actor_id: Option<Uuid>,
    ) -> Result<LifecycleChecklistResponse, AppError> {
        let workspace_id = auth_token.workspace_id();

        let template = self
            .repo
            .get_template_by_id(auth_token, template_id)
            .await?
            .ok_or_else(|| AppError::not_found(I18nKey::TemplateNotFound))?;

        let template_items = self
            .repo
            .list_template_items(auth_token, template_id)
            .await?;

        let employment = self
            .employment_repo
            .get_employment_by_id(auth_token, employment_id)
            .await?
            .ok_or_else(|| AppError::not_found(I18nKey::EmploymentNotFound))?;

        let employee = self
            .employee_repo
            .get_employee_by_id(employment.employee_id(), auth_token)
            .await?
            .ok_or_else(|| AppError::not_found(I18nKey::EmployeeNotFound))?;

        let base_date = match template.checklist_type() {
            LifecycleChecklistType::Onboarding
            | LifecycleChecklistType::Transfer
            | LifecycleChecklistType::ProbationEnd => employment.start_date(),
            LifecycleChecklistType::Offboarding => employment
                .end_date()
                .unwrap_or_else(|| chrono::Utc::now().naive_utc().date()),
        };

        let checklist_id = Uuid::new_v4();
        let checklist_entity =
            crate::models::diesel::lifecycle_checklist::LifecycleChecklistEntity {
                id: checklist_id,
                workspace_id,
                employment_id,
                template_id: Some(template_id),
                checklist_type: template.checklist_type().as_str().to_string(),
                status: ChecklistStatus::InProgress.as_str().to_string(),
                progress_percent: 0,
                started_at: Utc::now(),
                completed_at: None,
                created_by: actor_id,
                created_at: Utc::now(),
                updated_at: Utc::now(),
            };

        // Pre-resolve assignees that don't vary per item to avoid N+1 queries
        let has_direct_manager = template_items.iter().any(|it| {
            matches!(
                it.assignee_type(),
                crate::models::ChecklistAssigneeType::DirectManager
            )
        });
        let has_hr_manager = template_items.iter().any(|it| {
            matches!(
                it.assignee_type(),
                crate::models::ChecklistAssigneeType::HrManager
            )
        });
        let has_it_admin = template_items.iter().any(|it| {
            matches!(
                it.assignee_type(),
                crate::models::ChecklistAssigneeType::ItAdmin
            )
        });

        // DirectManager: resolve once (all DirectManager items reference the same employment.manager_employment_id)
        let direct_manager_user_id: Option<Uuid> = if has_direct_manager {
            if let Some(mgr_emp_id) = employment.manager_employment_id() {
                if let Ok(Some(mgr_emp)) = self
                    .employment_repo
                    .get_employment_by_id(auth_token, mgr_emp_id)
                    .await
                {
                    if let Ok(Some(mgr_employee)) = self
                        .employee_repo
                        .get_employee_by_id(mgr_emp.employee_id(), auth_token)
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
        } else {
            None
        };

        // HrManager: resolve once (workspace-scoped, same result for all items)
        let hr_manager_user_id: Option<Uuid> = if has_hr_manager {
            let users = self
                .workspace_repo
                .get_users_with_permission(workspace_id, "manage_onboarding")
                .await
                .unwrap_or_default();
            if let Some(&uid) = users.first() {
                Some(uid)
            } else {
                self.workspace_repo
                    .get_users_with_permission(workspace_id, "admin")
                    .await
                    .unwrap_or_default()
                    .first()
                    .copied()
            }
        } else {
            None
        };

        // ItAdmin: resolve once (workspace-scoped, same result for all items)
        let it_admin_user_id: Option<Uuid> = if has_it_admin {
            let users = self
                .workspace_repo
                .get_users_with_permission(workspace_id, "manage_it")
                .await
                .unwrap_or_default();
            if let Some(&uid) = users.first() {
                Some(uid)
            } else {
                self.workspace_repo
                    .get_users_with_permission(workspace_id, "admin")
                    .await
                    .unwrap_or_default()
                    .first()
                    .copied()
            }
        } else {
            None
        };

        let mut item_entities = Vec::new();
        for item in template_items {
            let due_date =
                Some(base_date + chrono::Duration::days(i64::from(item.due_days_offset())));

            let assignee_user_id = match item.assignee_type() {
                crate::models::ChecklistAssigneeType::SpecificUser => item.assignee_user_id(),
                crate::models::ChecklistAssigneeType::Employee => employee.user_id(),
                crate::models::ChecklistAssigneeType::DirectManager => direct_manager_user_id,
                crate::models::ChecklistAssigneeType::HrManager => hr_manager_user_id,
                crate::models::ChecklistAssigneeType::ItAdmin => it_admin_user_id,
            };

            item_entities.push(
                crate::models::diesel::lifecycle_checklist::LifecycleChecklistItemEntity {
                    id: Uuid::new_v4(),
                    checklist_id,
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

        let (checklist, items) = self
            .repo
            .create_checklist_with_items(auth_token, checklist_entity, item_entities)
            .await?;

        // If template had no items, complete checklist immediately
        let (updated_checklist, updated_items) = if items.is_empty() {
            let cl = self
                .repo
                .update_checklist_status_and_progress(
                    auth_token,
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
        auth_token: &TenantAuthToken,
        id: Uuid,
    ) -> Result<LifecycleChecklistResponse, AppError> {
        let checklist = self
            .repo
            .get_checklist_by_id(auth_token, id)
            .await?
            .ok_or_else(|| AppError::not_found(I18nKey::ChecklistNotFound))?;

        let items = self.repo.list_checklist_items(auth_token, id).await?;

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
        auth_token: &TenantAuthToken,
        employment_id: Uuid,
    ) -> Result<Vec<LifecycleChecklistResponse>, AppError> {
        let checklists = self
            .repo
            .list_checklists_for_employment(auth_token, employment_id)
            .await?;

        let mut responses = Vec::new();
        if checklists.is_empty() {
            return Ok(responses);
        }
        let checklist_ids: Vec<Uuid> = checklists
            .iter()
            .map(crate::models::lifecycle_checklist::LifecycleChecklist::id)
            .collect();
        let all_items = self
            .repo
            .list_items_for_checklist_ids(auth_token, &checklist_ids)
            .await?;

        // Group items by checklist_id for efficient lookup
        let mut items_by_checklist: std::collections::HashMap<Uuid, Vec<LifecycleChecklistItem>> =
            std::collections::HashMap::new();
        for item in all_items {
            items_by_checklist
                .entry(item.checklist_id())
                .or_default()
                .push(item);
        }

        for cl in checklists {
            let items = items_by_checklist.remove(&cl.id()).unwrap_or_default();
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

    #[allow(clippy::cast_precision_loss, clippy::cast_possible_truncation)]
    pub async fn update_item_status(
        &self,
        auth_token: &TenantAuthToken,
        item_id: Uuid,
        status: ChecklistItemStatus,
        notes: Option<String>,
        completed_by: Option<Uuid>,
    ) -> Result<LifecycleChecklistResponse, AppError> {
        let item = self
            .repo
            .get_checklist_item_by_id(auth_token, item_id)
            .await?
            .ok_or_else(|| AppError::not_found(I18nKey::ChecklistItemNotFound))?;

        self.repo
            .update_checklist_item_status(auth_token, item_id, status, completed_by, notes)
            .await?;

        let items = self
            .repo
            .list_checklist_items(auth_token, item.checklist_id())
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
                auth_token,
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
        auth_token: &TenantAuthToken,
        user_id: Uuid,
    ) -> Result<Vec<PendingTaskResponse>, AppError> {
        self.repo
            .get_pending_tasks_for_user(auth_token, user_id)
            .await
    }

    // --- Auto Trigger Rules ---
    pub async fn auto_trigger_onboarding(
        &self,
        auth_token: &TenantAuthToken,
        employment_id: Uuid,
        actor_id: Option<Uuid>,
    ) -> Result<(), AppError> {
        if let Some(template) = self
            .repo
            .get_default_template(auth_token, LifecycleChecklistType::Onboarding)
            .await?
        {
            let _ = self
                .create_checklist_from_template(auth_token, employment_id, template.id(), actor_id)
                .await?;
        }
        Ok(())
    }

    pub async fn auto_trigger_offboarding(
        &self,
        auth_token: &TenantAuthToken,
        employment_id: Uuid,
        actor_id: Option<Uuid>,
    ) -> Result<(), AppError> {
        if let Some(template) = self
            .repo
            .get_default_template(auth_token, LifecycleChecklistType::Offboarding)
            .await?
        {
            let _ = self
                .create_checklist_from_template(auth_token, employment_id, template.id(), actor_id)
                .await?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{
        ChecklistAssigneeType, ChecklistCategory, ChecklistItemStatus, ChecklistStatus, Employment,
        LifecycleChecklist, LifecycleChecklistItem, LifecycleChecklistTemplate,
        LifecycleChecklistTemplateItem,
    };
    use crate::repositories::employee::MockEmployeeRepository;
    use crate::repositories::employment_repository::MockEmploymentRepository;
    use crate::repositories::lifecycle_checklist_repository::MockLifecycleChecklistRepository;
    use crate::repositories::user_repository::MockUserRepository;
    use crate::repositories::workspace_repository::MockWorkspaceRepository;
    use chrono::{NaiveDate, Utc};
    use std::sync::Arc;
    use uuid::Uuid;

    // ── helpers ───────────────────────────────────────────────────────────

    fn make_token(ws: Uuid) -> TenantAuthToken {
        TenantAuthToken::system_override(ws)
    }

    fn make_template(ws: Uuid) -> LifecycleChecklistTemplate {
        LifecycleChecklistTemplate::new(
            Uuid::new_v4(),
            ws,
            "Test Template".into(),
            Some("Description".into()),
            LifecycleChecklistType::Onboarding,
            false,
            true,
            Utc::now(),
            Utc::now(),
        )
    }

    fn make_template_item(template_id: Uuid) -> LifecycleChecklistTemplateItem {
        LifecycleChecklistTemplateItem::new(
            Uuid::new_v4(),
            template_id,
            "Complete paperwork".into(),
            Some("HR paperwork".into()),
            ChecklistCategory::HrAdmin,
            ChecklistAssigneeType::Employee,
            None,
            true,
            3,
            1,
        )
    }

    fn make_employment(ws: Uuid, employee_id: Uuid) -> Employment {
        Employment::new(
            Uuid::new_v4(),
            ws,
            employee_id,
            Uuid::new_v4(), // employer_id
            None,           // primary_location_id
            None,           // department_id
            "active".into(),
            NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
            None, // end_date
            Some("Developer".into()),
            "full_time".into(),
            None, // manager_employment_id
            Utc::now(),
            Utc::now(),
            None, // termination_reason
            None, // notes
        )
    }

    fn make_checklist(ws: Uuid, employment_id: Uuid, template_id: Uuid) -> LifecycleChecklist {
        LifecycleChecklist::new(
            Uuid::new_v4(),
            ws,
            employment_id,
            Some(template_id),
            LifecycleChecklistType::Onboarding,
            ChecklistStatus::InProgress,
            50,
            Utc::now(),
            None,
            None,
            Utc::now(),
            Utc::now(),
        )
    }

    fn make_checklist_item(checklist_id: Uuid) -> LifecycleChecklistItem {
        LifecycleChecklistItem::new(
            Uuid::new_v4(),
            checklist_id,
            None,
            "HR paperwork".into(),
            None,
            ChecklistCategory::HrAdmin,
            None,
            true,
            Some(NaiveDate::from_ymd_opt(2026, 1, 4).unwrap()),
            ChecklistItemStatus::Pending,
            None,
            None,
            None,
            1,
            Utc::now(),
            Utc::now(),
        )
    }

    fn make_employee(ws: Uuid) -> crate::models::Employee {
        crate::models::Employee::new(
            Uuid::new_v4(),
            ws,
            None,
            "Test Employee".into(),
            "EMP001".into(),
            vec![],
            Utc::now(),
            Utc::now(),
            None,
        )
    }

    fn build_service(repo: MockLifecycleChecklistRepository) -> LifecycleService {
        LifecycleService::new(
            Arc::new(repo),
            Arc::new(MockEmployeeRepository::new()),
            Arc::new(MockEmploymentRepository::new()),
            Arc::new(MockUserRepository::new()),
            Arc::new(MockWorkspaceRepository::new()),
        )
    }

    fn build_service_full(
        lifecycle_repo: MockLifecycleChecklistRepository,
        employee_repo: MockEmployeeRepository,
        employment_repo: MockEmploymentRepository,
    ) -> LifecycleService {
        LifecycleService::new(
            Arc::new(lifecycle_repo),
            Arc::new(employee_repo),
            Arc::new(employment_repo),
            Arc::new(MockUserRepository::new()),
            Arc::new(MockWorkspaceRepository::new()),
        )
    }

    // ── Template tests ────────────────────────────────────────────────────

    #[tokio::test]
    async fn create_template_success() {
        let ws = Uuid::new_v4();
        let template = make_template(ws);
        let expected = template.clone();

        let mut repo = MockLifecycleChecklistRepository::new();
        repo.expect_create_template()
            .returning(move |_, _| Ok(expected.clone()));

        let svc = build_service(repo);
        let result = svc
            .create_template(
                &make_token(ws),
                CreateLifecycleChecklistTemplateRequest {
                    name: "Test Template".into(),
                    description: None,
                    checklist_type: LifecycleChecklistType::Onboarding,
                    is_default: false,
                },
            )
            .await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap().id(), template.id());
    }

    #[tokio::test]
    async fn create_template_empty_name() {
        let repo = MockLifecycleChecklistRepository::new();
        let svc = build_service(repo);
        let result = svc
            .create_template(
                &make_token(Uuid::new_v4()),
                CreateLifecycleChecklistTemplateRequest {
                    name: "   ".into(),
                    description: None,
                    checklist_type: LifecycleChecklistType::Onboarding,
                    is_default: false,
                },
            )
            .await;

        assert!(matches!(result.unwrap_err(), AppError::Validation(_)));
    }

    #[tokio::test]
    async fn get_template_found() {
        let ws = Uuid::new_v4();
        let template = make_template(ws);
        let template_id = template.id();
        let expected = template.clone();

        let mut repo = MockLifecycleChecklistRepository::new();
        repo.expect_get_template_by_id()
            .returning(move |_, _| Ok(Some(expected.clone())));

        let svc = build_service(repo);
        let result = svc.get_template(&make_token(ws), template_id).await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap().id(), template_id);
    }

    #[tokio::test]
    async fn get_template_not_found() {
        let ws = Uuid::new_v4();
        let mut repo = MockLifecycleChecklistRepository::new();
        repo.expect_get_template_by_id().returning(|_, _| Ok(None));

        let svc = build_service(repo);
        let result = svc.get_template(&make_token(ws), Uuid::new_v4()).await;

        assert!(matches!(result.unwrap_err(), AppError::NotFound(_)));
    }

    #[tokio::test]
    async fn list_active_templates_with_filter() {
        let ws = Uuid::new_v4();
        let template = make_template(ws);
        let expected = vec![template.clone()];

        let mut repo = MockLifecycleChecklistRepository::new();
        repo.expect_list_active_templates()
            .returning(move |_, _| Ok(expected.clone()));

        let svc = build_service(repo);
        let result = svc
            .list_active_templates(&make_token(ws), Some(LifecycleChecklistType::Onboarding))
            .await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 1);
    }

    #[tokio::test]
    async fn list_active_templates_none_filter() {
        let ws = Uuid::new_v4();
        let mut repo = MockLifecycleChecklistRepository::new();
        repo.expect_list_active_templates()
            .returning(|_, _| Ok(vec![]));

        let svc = build_service(repo);
        let result = svc.list_active_templates(&make_token(ws), None).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn update_template_success() {
        let ws = Uuid::new_v4();
        let updated = make_template(ws);
        let expected = updated.clone();

        let mut repo = MockLifecycleChecklistRepository::new();
        repo.expect_update_template()
            .returning(move |_, _, _| Ok(expected.clone()));

        let svc = build_service(repo);
        let result = svc
            .update_template(
                &make_token(ws),
                updated.id(),
                UpdateLifecycleChecklistTemplateRequest {
                    name: Some("Updated".into()),
                    description: None,
                    is_default: None,
                    is_active: None,
                },
            )
            .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn delete_template_success() {
        let ws = Uuid::new_v4();
        let mut repo = MockLifecycleChecklistRepository::new();
        repo.expect_soft_delete_template().returning(|_, _| Ok(()));

        let svc = build_service(repo);
        let result = svc.delete_template(&make_token(ws), Uuid::new_v4()).await;

        assert!(result.is_ok());
    }

    // ── Template Item tests ───────────────────────────────────────────────

    #[tokio::test]
    async fn create_template_item_success() {
        let ws = Uuid::new_v4();
        let template = make_template(ws);
        let template_id = template.id();
        let item = make_template_item(template_id);
        let expected_item = item.clone();
        let expected_template = template.clone();

        let mut repo = MockLifecycleChecklistRepository::new();
        repo.expect_get_template_by_id()
            .returning(move |_, _| Ok(Some(expected_template.clone())));
        repo.expect_create_template_item()
            .returning(move |_, _, _| Ok(expected_item.clone()));

        let svc = build_service(repo);
        let result = svc
            .create_template_item(
                &make_token(ws),
                template_id,
                CreateLifecycleChecklistTemplateItemRequest {
                    title: "Complete paperwork".into(),
                    description: None,
                    category: ChecklistCategory::HrAdmin,
                    assignee_type: ChecklistAssigneeType::Employee,
                    assignee_user_id: None,
                    is_required: true,
                    due_days_offset: 3,
                    display_order: 1,
                },
            )
            .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn create_template_item_empty_title() {
        let ws = Uuid::new_v4();
        let repo = MockLifecycleChecklistRepository::new();
        let svc = build_service(repo);
        let result = svc
            .create_template_item(
                &make_token(ws),
                Uuid::new_v4(),
                CreateLifecycleChecklistTemplateItemRequest {
                    title: "".into(),
                    description: None,
                    category: ChecklistCategory::HrAdmin,
                    assignee_type: ChecklistAssigneeType::Employee,
                    assignee_user_id: None,
                    is_required: true,
                    due_days_offset: 3,
                    display_order: 1,
                },
            )
            .await;

        assert!(matches!(result.unwrap_err(), AppError::Validation(_)));
    }

    #[tokio::test]
    async fn create_template_item_template_not_found() {
        let ws = Uuid::new_v4();
        let mut repo = MockLifecycleChecklistRepository::new();
        repo.expect_get_template_by_id().returning(|_, _| Ok(None));

        let svc = build_service(repo);
        let result = svc
            .create_template_item(
                &make_token(ws),
                Uuid::new_v4(),
                CreateLifecycleChecklistTemplateItemRequest {
                    title: "Task".into(),
                    description: None,
                    category: ChecklistCategory::HrAdmin,
                    assignee_type: ChecklistAssigneeType::Employee,
                    assignee_user_id: None,
                    is_required: true,
                    due_days_offset: 1,
                    display_order: 1,
                },
            )
            .await;

        assert!(matches!(result.unwrap_err(), AppError::NotFound(_)));
    }

    #[tokio::test]
    async fn get_template_item_found() {
        let ws = Uuid::new_v4();
        let template = make_template(ws);
        let item = make_template_item(template.id());
        let item_id = item.id();
        let expected = item.clone();

        let mut repo = MockLifecycleChecklistRepository::new();
        repo.expect_get_template_item_by_id()
            .returning(move |_, _| Ok(Some(expected.clone())));

        let svc = build_service(repo);
        let result = svc.get_template_item(&make_token(ws), item_id).await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap().id(), item_id);
    }

    #[tokio::test]
    async fn get_template_item_not_found() {
        let ws = Uuid::new_v4();
        let mut repo = MockLifecycleChecklistRepository::new();
        repo.expect_get_template_item_by_id()
            .returning(|_, _| Ok(None));

        let svc = build_service(repo);
        let result = svc.get_template_item(&make_token(ws), Uuid::new_v4()).await;

        assert!(matches!(result.unwrap_err(), AppError::NotFound(_)));
    }

    #[tokio::test]
    async fn list_template_items_success() {
        let ws = Uuid::new_v4();
        let template_id = Uuid::new_v4();
        let mut repo = MockLifecycleChecklistRepository::new();
        repo.expect_list_template_items()
            .returning(|_, _| Ok(vec![]));

        let svc = build_service(repo);
        let result = svc.list_template_items(&make_token(ws), template_id).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn update_template_item_success() {
        let ws = Uuid::new_v4();
        let template = make_template(ws);
        let item = make_template_item(template.id());
        let expected = item.clone();

        let mut repo = MockLifecycleChecklistRepository::new();
        repo.expect_update_template_item()
            .returning(move |_, _, _| Ok(expected.clone()));

        let svc = build_service(repo);
        let result = svc
            .update_template_item(
                &make_token(ws),
                item.id(),
                UpdateLifecycleChecklistTemplateItemRequest {
                    title: Some("Updated task".into()),
                    description: None,
                    category: None,
                    assignee_type: None,
                    assignee_user_id: None,
                    is_required: None,
                    due_days_offset: None,
                    display_order: None,
                },
            )
            .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn delete_template_item_success() {
        let ws = Uuid::new_v4();
        let mut repo = MockLifecycleChecklistRepository::new();
        repo.expect_delete_template_item().returning(|_, _| Ok(()));

        let svc = build_service(repo);
        let result = svc
            .delete_template_item(&make_token(ws), Uuid::new_v4())
            .await;

        assert!(result.is_ok());
    }

    // ── Checklist Instance tests ──────────────────────────────────────────

    #[tokio::test]
    async fn get_checklist_found() {
        let ws = Uuid::new_v4();
        let employment_id = Uuid::new_v4();
        let template_id = Uuid::new_v4();
        let checklist = make_checklist(ws, employment_id, template_id);
        let checklist_id = checklist.id();
        let expected_cl = checklist.clone();

        let mut repo = MockLifecycleChecklistRepository::new();
        repo.expect_get_checklist_by_id()
            .returning(move |_, _| Ok(Some(expected_cl.clone())));
        repo.expect_list_checklist_items()
            .returning(|_, _| Ok(vec![]));

        let svc = build_service(repo);
        let result = svc.get_checklist(&make_token(ws), checklist_id).await;

        assert!(result.is_ok());
        let resp = result.unwrap();
        assert_eq!(resp.id, checklist_id);
        assert_eq!(resp.items.len(), 0);
    }

    #[tokio::test]
    async fn get_checklist_not_found() {
        let ws = Uuid::new_v4();
        let mut repo = MockLifecycleChecklistRepository::new();
        repo.expect_get_checklist_by_id().returning(|_, _| Ok(None));

        let svc = build_service(repo);
        let result = svc.get_checklist(&make_token(ws), Uuid::new_v4()).await;

        assert!(matches!(result.unwrap_err(), AppError::NotFound(_)));
    }

    #[tokio::test]
    async fn list_checklists_for_employment_empty() {
        let ws = Uuid::new_v4();
        let employment_id = Uuid::new_v4();

        let mut repo = MockLifecycleChecklistRepository::new();
        repo.expect_list_checklists_for_employment()
            .returning(|_, _| Ok(vec![]));

        let svc = build_service(repo);
        let result = svc
            .list_checklists_for_employment(&make_token(ws), employment_id)
            .await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 0);
    }

    #[tokio::test]
    async fn list_checklists_for_employment_with_items() {
        let ws = Uuid::new_v4();
        let employment_id = Uuid::new_v4();
        let template_id = Uuid::new_v4();
        let checklist = make_checklist(ws, employment_id, template_id);
        let checklist_id = checklist.id();
        let item = make_checklist_item(checklist_id);

        let expected_cl = checklist.clone();
        let expected_items = vec![item.clone()];

        let mut repo = MockLifecycleChecklistRepository::new();
        repo.expect_list_checklists_for_employment()
            .returning(move |_, _| Ok(vec![expected_cl.clone()]));
        repo.expect_list_items_for_checklist_ids()
            .returning(move |_, _| Ok(expected_items.clone()));

        let svc = build_service(repo);
        let result = svc
            .list_checklists_for_employment(&make_token(ws), employment_id)
            .await;

        assert!(result.is_ok());
        let responses = result.unwrap();
        assert_eq!(responses.len(), 1);
        assert_eq!(responses[0].items.len(), 1);
        assert_eq!(responses[0].items[0].id(), item.id());
    }

    #[tokio::test]
    async fn get_pending_tasks_for_user_success() {
        let ws = Uuid::new_v4();
        let user_id = Uuid::new_v4();

        let mut repo = MockLifecycleChecklistRepository::new();
        repo.expect_get_pending_tasks_for_user()
            .returning(|_, _| Ok(vec![]));

        let svc = build_service(repo);
        let result = svc
            .get_pending_tasks_for_user(&make_token(ws), user_id)
            .await;

        assert!(result.is_ok());
    }

    // ── Item Status tests ─────────────────────────────────────────────────

    #[tokio::test]
    async fn update_item_status_item_not_found() {
        let ws = Uuid::new_v4();
        let item_id = Uuid::new_v4();

        let mut repo = MockLifecycleChecklistRepository::new();
        repo.expect_get_checklist_item_by_id()
            .returning(|_, _| Ok(None));

        let svc = build_service(repo);
        let result = svc
            .update_item_status(
                &make_token(ws),
                item_id,
                ChecklistItemStatus::Completed,
                None,
                None,
            )
            .await;

        assert!(matches!(result.unwrap_err(), AppError::NotFound(_)));
    }

    #[tokio::test]
    async fn update_item_status_to_completed_progress_100() {
        let ws = Uuid::new_v4();
        let checklist_id = Uuid::new_v4();
        let item_id = Uuid::new_v4();
        let employment_id = Uuid::new_v4();
        let template_id = Uuid::new_v4();

        let item = make_checklist_item(checklist_id);
        let completed_item = LifecycleChecklistItem::new(
            item.id(),
            checklist_id,
            None,
            "HR paperwork".into(),
            None,
            ChecklistCategory::HrAdmin,
            None,
            true,
            item.due_date(),
            ChecklistItemStatus::Completed,
            Some(Utc::now()),
            Some(Uuid::new_v4()),
            None,
            1,
            Utc::now(),
            Utc::now(),
        );

        let completed_checklist = LifecycleChecklist::new(
            checklist_id,
            ws,
            employment_id,
            Some(template_id),
            LifecycleChecklistType::Onboarding,
            ChecklistStatus::Completed,
            100,
            Utc::now(),
            Some(Utc::now()),
            None,
            Utc::now(),
            Utc::now(),
        );

        let expected_item = item.clone();
        let expected_completed_item = completed_item.clone();
        let expected_checklist = completed_checklist.clone();

        let mut repo = MockLifecycleChecklistRepository::new();
        repo.expect_get_checklist_item_by_id()
            .returning(move |_, _| Ok(Some(expected_item.clone())));
        repo.expect_update_checklist_item_status()
            .returning(move |_, _, _, _, _| Ok(expected_completed_item.clone()));
        repo.expect_list_checklist_items()
            .returning(move |_, _| Ok(vec![completed_item.clone()]));
        repo.expect_update_checklist_status_and_progress()
            .returning(move |_, _, _, _, _| Ok(expected_checklist.clone()));

        let svc = build_service(repo);
        let result = svc
            .update_item_status(
                &make_token(ws),
                item_id,
                ChecklistItemStatus::Completed,
                None,
                Some(Uuid::new_v4()),
            )
            .await;

        assert!(result.is_ok());
        let resp = result.unwrap();
        assert_eq!(resp.status, ChecklistStatus::Completed);
        assert_eq!(resp.progress_percent, 100);
        assert!(resp.completed_at.is_some());
    }

    #[tokio::test]
    async fn update_item_status_partial_progress() {
        let ws = Uuid::new_v4();
        let checklist_id = Uuid::new_v4();
        let item_id = Uuid::new_v4();
        let employment_id = Uuid::new_v4();
        let template_id = Uuid::new_v4();

        let item = make_checklist_item(checklist_id);
        let completed_item = LifecycleChecklistItem::new(
            item.id(),
            checklist_id,
            None,
            "HR paperwork".into(),
            None,
            ChecklistCategory::HrAdmin,
            None,
            true,
            item.due_date(),
            ChecklistItemStatus::Completed,
            Some(Utc::now()),
            Some(Uuid::new_v4()),
            None,
            1,
            Utc::now(),
            Utc::now(),
        );

        // Two items: one completed, one still pending → 50%
        let pending_item = LifecycleChecklistItem::new(
            Uuid::new_v4(),
            checklist_id,
            None,
            "IT Setup".into(),
            None,
            ChecklistCategory::ItSetup,
            None,
            true,
            None,
            ChecklistItemStatus::Pending,
            None,
            None,
            None,
            2,
            Utc::now(),
            Utc::now(),
        );

        let in_progress_checklist = LifecycleChecklist::new(
            checklist_id,
            ws,
            employment_id,
            Some(template_id),
            LifecycleChecklistType::Onboarding,
            ChecklistStatus::InProgress,
            50,
            Utc::now(),
            None,
            None,
            Utc::now(),
            Utc::now(),
        );

        let expected_item = item.clone();
        let expected_completed_item = completed_item.clone();
        let expected_checklist = in_progress_checklist.clone();

        let mut repo = MockLifecycleChecklistRepository::new();
        repo.expect_get_checklist_item_by_id()
            .returning(move |_, _| Ok(Some(expected_item.clone())));
        repo.expect_update_checklist_item_status()
            .returning(move |_, _, _, _, _| Ok(expected_completed_item.clone()));
        repo.expect_list_checklist_items()
            .returning(move |_, _| Ok(vec![completed_item.clone(), pending_item.clone()]));
        repo.expect_update_checklist_status_and_progress()
            .returning(move |_, _, _, _, _| Ok(expected_checklist.clone()));

        let svc = build_service(repo);
        let result = svc
            .update_item_status(
                &make_token(ws),
                item_id,
                ChecklistItemStatus::Completed,
                None,
                Some(Uuid::new_v4()),
            )
            .await;

        assert!(result.is_ok());
        let resp = result.unwrap();
        assert_eq!(resp.status, ChecklistStatus::InProgress);
        assert_eq!(resp.progress_percent, 50);
        assert!(resp.completed_at.is_none());
    }

    // ── Auto-trigger tests ────────────────────────────────────────────────

    #[tokio::test]
    async fn auto_trigger_onboarding_no_template() {
        let ws = Uuid::new_v4();
        let employment_id = Uuid::new_v4();

        let mut repo = MockLifecycleChecklistRepository::new();
        repo.expect_get_default_template()
            .returning(|_, _| Ok(None));

        let svc = build_service(repo);
        let result = svc
            .auto_trigger_onboarding(&make_token(ws), employment_id, None)
            .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn auto_trigger_offboarding_no_template() {
        let ws = Uuid::new_v4();
        let employment_id = Uuid::new_v4();

        let mut repo = MockLifecycleChecklistRepository::new();
        repo.expect_get_default_template()
            .returning(|_, _| Ok(None));

        let svc = build_service(repo);
        let result = svc
            .auto_trigger_offboarding(&make_token(ws), employment_id, None)
            .await;

        assert!(result.is_ok());
    }

    // ── create_checklist_from_template (integration-style with all mocks) ──

    #[tokio::test]
    async fn create_checklist_from_template_with_items_success() {
        let ws = Uuid::new_v4();
        let employee_id = Uuid::new_v4();
        let employment_id = Uuid::new_v4();
        let template = make_template(ws);
        let template_id = template.id();

        let item = make_template_item(template_id);
        let employee = make_employee(ws);
        let employment = make_employment(ws, employee_id);

        let checklist = make_checklist(ws, employment_id, template_id);
        let checklist_item = make_checklist_item(checklist.id());

        let expected_template = template.clone();
        let expected_item = item.clone();
        let expected_employee = employee.clone();
        let expected_employment = employment.clone();
        let expected_checklist = checklist.clone();
        let expected_checklist_item = checklist_item.clone();

        let mut lifecycle_repo = MockLifecycleChecklistRepository::new();
        lifecycle_repo
            .expect_get_template_by_id()
            .returning(move |_, _| Ok(Some(expected_template.clone())));
        lifecycle_repo
            .expect_list_template_items()
            .returning(move |_, _| Ok(vec![expected_item.clone()]));
        lifecycle_repo
            .expect_create_checklist_with_items()
            .returning(move |_, _, _| {
                Ok((
                    expected_checklist.clone(),
                    vec![expected_checklist_item.clone()],
                ))
            });

        let mut employee_repo = MockEmployeeRepository::new();
        employee_repo
            .expect_get_employee_by_id()
            .returning(move |_, _| Ok(Some(expected_employee.clone())));

        let mut employment_repo = MockEmploymentRepository::new();
        employment_repo
            .expect_get_employment_by_id()
            .returning(move |_, _| Ok(Some(expected_employment.clone())));

        let svc = build_service_full(lifecycle_repo, employee_repo, employment_repo);
        let result = svc
            .create_checklist_from_template(&make_token(ws), employment_id, template_id, None)
            .await;

        assert!(result.is_ok());
        let resp = result.unwrap();
        assert_eq!(resp.employment_id, employment_id);
        assert_eq!(resp.items.len(), 1);
    }

    #[tokio::test]
    async fn create_checklist_from_template_not_found() {
        let ws = Uuid::new_v4();

        let mut lifecycle_repo = MockLifecycleChecklistRepository::new();
        lifecycle_repo
            .expect_get_template_by_id()
            .returning(|_, _| Ok(None));

        let employee_repo = MockEmployeeRepository::new();
        let employment_repo = MockEmploymentRepository::new();

        let svc = build_service_full(lifecycle_repo, employee_repo, employment_repo);
        let result = svc
            .create_checklist_from_template(&make_token(ws), Uuid::new_v4(), Uuid::new_v4(), None)
            .await;

        assert!(matches!(result.unwrap_err(), AppError::NotFound(_)));
    }

    #[tokio::test]
    async fn create_checklist_from_template_employment_not_found() {
        let ws = Uuid::new_v4();
        let template = make_template(ws);
        let template_id = template.id();
        let item = make_template_item(template_id);
        let employee = make_employee(ws);

        let expected_template = template.clone();
        let expected_item = item.clone();
        let expected_employee = employee.clone();

        let mut lifecycle_repo = MockLifecycleChecklistRepository::new();
        lifecycle_repo
            .expect_get_template_by_id()
            .returning(move |_, _| Ok(Some(expected_template.clone())));
        lifecycle_repo
            .expect_list_template_items()
            .returning(move |_, _| Ok(vec![expected_item.clone()]));

        let mut employee_repo = MockEmployeeRepository::new();
        employee_repo
            .expect_get_employee_by_id()
            .returning(move |_, _| Ok(Some(expected_employee.clone())));

        let mut employment_repo = MockEmploymentRepository::new();
        employment_repo
            .expect_get_employment_by_id()
            .returning(|_, _| Ok(None));

        let svc = build_service_full(lifecycle_repo, employee_repo, employment_repo);
        let result = svc
            .create_checklist_from_template(&make_token(ws), Uuid::new_v4(), template_id, None)
            .await;

        assert!(matches!(result.unwrap_err(), AppError::NotFound(_)));
    }

    #[tokio::test]
    async fn create_checklist_from_template_employee_not_found() {
        let ws = Uuid::new_v4();
        let employee_id = Uuid::new_v4();
        let employment_id = Uuid::new_v4();
        let template = make_template(ws);
        let template_id = template.id();
        let item = make_template_item(template_id);
        let employment = make_employment(ws, employee_id);

        let expected_template = template.clone();
        let expected_item = item.clone();
        let expected_employment = employment.clone();

        let mut lifecycle_repo = MockLifecycleChecklistRepository::new();
        lifecycle_repo
            .expect_get_template_by_id()
            .returning(move |_, _| Ok(Some(expected_template.clone())));
        lifecycle_repo
            .expect_list_template_items()
            .returning(move |_, _| Ok(vec![expected_item.clone()]));

        let mut employment_repo = MockEmploymentRepository::new();
        employment_repo
            .expect_get_employment_by_id()
            .returning(move |_, _| Ok(Some(expected_employment.clone())));

        let mut employee_repo = MockEmployeeRepository::new();
        employee_repo
            .expect_get_employee_by_id()
            .returning(|_, _| Ok(None));

        let svc = build_service_full(lifecycle_repo, employee_repo, employment_repo);
        let result = svc
            .create_checklist_from_template(&make_token(ws), employment_id, template_id, None)
            .await;

        assert!(matches!(result.unwrap_err(), AppError::NotFound(_)));
    }
}
