use crate::diesel_pool::TenantAuthToken;
use crate::errors::AppError;
use crate::models::{
    ChecklistItemStatus, ChecklistStatus, CreateLifecycleChecklistTemplateItemRequest,
    CreateLifecycleChecklistTemplateRequest, LifecycleChecklist, LifecycleChecklistItem,
    LifecycleChecklistTemplate, LifecycleChecklistTemplateItem, LifecycleChecklistType,
    PendingTaskResponse, UpdateLifecycleChecklistTemplateItemRequest,
    UpdateLifecycleChecklistTemplateRequest,
};
use async_trait::async_trait;
use uuid::Uuid;

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait LifecycleChecklistRepository: Send + Sync {
    // --- Templates ---
    async fn create_template(
        &self,
        auth_token: &TenantAuthToken,
        data: CreateLifecycleChecklistTemplateRequest,
    ) -> Result<LifecycleChecklistTemplate, AppError>;

    async fn get_template_by_id(
        &self,
        auth_token: &TenantAuthToken,
        id: Uuid,
    ) -> Result<Option<LifecycleChecklistTemplate>, AppError>;

    async fn list_active_templates(
        &self,
        auth_token: &TenantAuthToken,
        checklist_type: Option<LifecycleChecklistType>,
    ) -> Result<Vec<LifecycleChecklistTemplate>, AppError>;

    async fn update_template(
        &self,
        auth_token: &TenantAuthToken,
        id: Uuid,
        data: UpdateLifecycleChecklistTemplateRequest,
    ) -> Result<LifecycleChecklistTemplate, AppError>;

    async fn soft_delete_template(
        &self,
        auth_token: &TenantAuthToken,
        id: Uuid,
    ) -> Result<(), AppError>;

    async fn get_default_template(
        &self,
        auth_token: &TenantAuthToken,
        checklist_type: LifecycleChecklistType,
    ) -> Result<Option<LifecycleChecklistTemplate>, AppError>;

    // --- Template Items ---
    async fn create_template_item(
        &self,
        auth_token: &TenantAuthToken,
        template_id: Uuid,
        data: CreateLifecycleChecklistTemplateItemRequest,
    ) -> Result<LifecycleChecklistTemplateItem, AppError>;

    async fn get_template_item_by_id(
        &self,
        auth_token: &TenantAuthToken,
        id: Uuid,
    ) -> Result<Option<LifecycleChecklistTemplateItem>, AppError>;

    async fn list_template_items(
        &self,
        auth_token: &TenantAuthToken,
        template_id: Uuid,
    ) -> Result<Vec<LifecycleChecklistTemplateItem>, AppError>;

    async fn update_template_item(
        &self,
        auth_token: &TenantAuthToken,
        id: Uuid,
        data: UpdateLifecycleChecklistTemplateItemRequest,
    ) -> Result<LifecycleChecklistTemplateItem, AppError>;

    async fn delete_template_item(
        &self,
        auth_token: &TenantAuthToken,
        id: Uuid,
    ) -> Result<(), AppError>;

    async fn create_checklist(
        &self,
        auth_token: &TenantAuthToken,
        employment_id: Uuid,
        template_id: Option<Uuid>,
        checklist_type: LifecycleChecklistType,
        created_by: Option<Uuid>,
    ) -> Result<LifecycleChecklist, AppError>;

    async fn create_checklist_with_items(
        &self,
        auth_token: &TenantAuthToken,
        checklist: crate::models::diesel::lifecycle_checklist::LifecycleChecklistEntity,
        items: Vec<crate::models::diesel::lifecycle_checklist::LifecycleChecklistItemEntity>,
    ) -> Result<(LifecycleChecklist, Vec<LifecycleChecklistItem>), AppError>;

    async fn get_checklist_by_id(
        &self,
        auth_token: &TenantAuthToken,
        id: Uuid,
    ) -> Result<Option<LifecycleChecklist>, AppError>;

    async fn list_checklists_for_employment(
        &self,
        auth_token: &TenantAuthToken,
        employment_id: Uuid,
    ) -> Result<Vec<LifecycleChecklist>, AppError>;

    async fn update_checklist_status_and_progress(
        &self,
        auth_token: &TenantAuthToken,
        id: Uuid,
        status: ChecklistStatus,
        progress_percent: i32,
        completed_at: Option<chrono::DateTime<chrono::Utc>>,
    ) -> Result<LifecycleChecklist, AppError>;

    // --- Checklist Item Instances ---
    async fn create_checklist_items(
        &self,
        auth_token: &TenantAuthToken,
        items: Vec<crate::models::diesel::lifecycle_checklist::LifecycleChecklistItemEntity>,
    ) -> Result<Vec<LifecycleChecklistItem>, AppError>;

    async fn get_checklist_item_by_id(
        &self,
        auth_token: &TenantAuthToken,
        id: Uuid,
    ) -> Result<Option<LifecycleChecklistItem>, AppError>;

    async fn list_checklist_items(
        &self,
        auth_token: &TenantAuthToken,
        checklist_id: Uuid,
    ) -> Result<Vec<LifecycleChecklistItem>, AppError>;

    async fn list_items_for_checklist_ids(
        &self,
        auth_token: &TenantAuthToken,
        checklist_ids: &[Uuid],
    ) -> Result<Vec<LifecycleChecklistItem>, AppError>;

    async fn update_checklist_item_status(
        &self,
        auth_token: &TenantAuthToken,
        id: Uuid,
        status: ChecklistItemStatus,
        completed_by: Option<Uuid>,
        notes: Option<String>,
    ) -> Result<LifecycleChecklistItem, AppError>;

    async fn get_pending_tasks_for_user(
        &self,
        auth_token: &TenantAuthToken,
        user_id: Uuid,
    ) -> Result<Vec<PendingTaskResponse>, AppError>;
}
