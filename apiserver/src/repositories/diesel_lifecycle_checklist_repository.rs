use crate::diesel_pool::{DieselPools, TenantAuthToken};
use crate::errors::AppError;
use crate::models::diesel::lifecycle_checklist::{
    LifecycleChecklistEntity, LifecycleChecklistItemEntity, LifecycleChecklistTemplateEntity,
    LifecycleChecklistTemplateItemEntity,
};
use crate::models::{
    ChecklistCategory, ChecklistItemStatus, ChecklistStatus,
    CreateLifecycleChecklistTemplateItemRequest, CreateLifecycleChecklistTemplateRequest,
    LifecycleChecklist, LifecycleChecklistItem, LifecycleChecklistTemplate,
    LifecycleChecklistTemplateItem, LifecycleChecklistType, PendingTaskResponse,
    UpdateLifecycleChecklistTemplateItemRequest, UpdateLifecycleChecklistTemplateRequest,
};
use crate::repositories::lifecycle_checklist_repository::LifecycleChecklistRepository;
use crate::schema::{
    employees, employments, lifecycle_checklist_items, lifecycle_checklist_template_items,
    lifecycle_checklist_templates, lifecycle_checklists,
};
use crate::utils::i18n::I18nKey;
use async_trait::async_trait;
use chrono::{NaiveDate, Utc};
use diesel::prelude::*;
use diesel_async::{AsyncConnection, RunQueryDsl};
use std::str::FromStr;
use uuid::Uuid;

#[derive(Clone)]
pub struct DieselLifecycleChecklistRepository {
    diesel_pools: DieselPools,
}

impl DieselLifecycleChecklistRepository {
    pub fn new(diesel_pools: DieselPools) -> Self {
        Self { diesel_pools }
    }
}

#[async_trait]
impl LifecycleChecklistRepository for DieselLifecycleChecklistRepository {
    // --- Templates ---
    async fn create_template(
        &self,
        auth_token: &TenantAuthToken,
        data: CreateLifecycleChecklistTemplateRequest,
    ) -> Result<LifecycleChecklistTemplate, AppError> {
        let mut conn = self
            .diesel_pools
            .tenant_transaction_conn(auth_token)
            .await?;
        let workspace_id = auth_token.workspace_id();

        conn.transaction::<_, AppError, _>(|conn| {
            Box::pin(async move {
                // If this is set to default, unset other defaults for this checklist_type
                if data.is_default {
                    diesel::update(lifecycle_checklist_templates::table)
                        .filter(lifecycle_checklist_templates::workspace_id.eq(workspace_id))
                        .filter(
                            lifecycle_checklist_templates::checklist_type
                                .eq(data.checklist_type.as_str()),
                        )
                        .filter(lifecycle_checklist_templates::deleted_at.is_null())
                        .set(lifecycle_checklist_templates::is_default.eq(false))
                        .execute(conn)
                        .await
                        .map_err(|e| {
                            AppError::Internal(format!("db unset default template: {e}"))
                        })?;
                }

                let entity = LifecycleChecklistTemplateEntity {
                    id: Uuid::new_v4(),
                    workspace_id,
                    name: data.name,
                    description: data.description,
                    checklist_type: data.checklist_type.as_str().to_string(),
                    is_default: data.is_default,
                    is_active: true,
                    deleted_at: None,
                    created_at: Utc::now(),
                    updated_at: Utc::now(),
                };

                let result = diesel::insert_into(lifecycle_checklist_templates::table)
                    .values(&entity)
                    .get_result::<LifecycleChecklistTemplateEntity>(conn)
                    .await
                    .map_err(|e| AppError::Internal(format!("db create template: {e}")))?;

                Ok(result.into())
            })
        })
        .await
    }

    async fn get_template_by_id(
        &self,
        auth_token: &TenantAuthToken,
        id: Uuid,
    ) -> Result<Option<LifecycleChecklistTemplate>, AppError> {
        let mut conn = self
            .diesel_pools
            .tenant_transaction_conn(auth_token)
            .await?;
        let workspace_id = auth_token.workspace_id();

        let result = lifecycle_checklist_templates::table
            .filter(lifecycle_checklist_templates::id.eq(id))
            .filter(lifecycle_checklist_templates::workspace_id.eq(workspace_id))
            .filter(lifecycle_checklist_templates::deleted_at.is_null())
            .first::<LifecycleChecklistTemplateEntity>(&mut *conn)
            .await
            .optional()
            .map_err(|e| AppError::Internal(format!("db get template: {e}")))?;

        Ok(result.map(Into::into))
    }

    async fn list_active_templates(
        &self,
        auth_token: &TenantAuthToken,
        checklist_type: Option<LifecycleChecklistType>,
    ) -> Result<Vec<LifecycleChecklistTemplate>, AppError> {
        let mut conn = self
            .diesel_pools
            .tenant_transaction_conn(auth_token)
            .await?;
        let workspace_id = auth_token.workspace_id();

        let mut query = lifecycle_checklist_templates::table
            .filter(lifecycle_checklist_templates::workspace_id.eq(workspace_id))
            .filter(lifecycle_checklist_templates::is_active.eq(true))
            .filter(lifecycle_checklist_templates::deleted_at.is_null())
            .into_boxed();

        if let Some(ct) = checklist_type {
            query = query.filter(lifecycle_checklist_templates::checklist_type.eq(ct.as_str()));
        }

        let results = query
            .order(lifecycle_checklist_templates::name.asc())
            .load::<LifecycleChecklistTemplateEntity>(&mut *conn)
            .await
            .map_err(|e| AppError::Internal(format!("db list templates: {e}")))?;

        Ok(results.into_iter().map(Into::into).collect())
    }

    async fn update_template(
        &self,
        auth_token: &TenantAuthToken,
        id: Uuid,
        data: UpdateLifecycleChecklistTemplateRequest,
    ) -> Result<LifecycleChecklistTemplate, AppError> {
        let mut conn = self
            .diesel_pools
            .tenant_transaction_conn(auth_token)
            .await?;
        let workspace_id = auth_token.workspace_id();

        conn.transaction::<_, AppError, _>(|conn| {
            Box::pin(async move {
                let mut existing = lifecycle_checklist_templates::table
                    .filter(lifecycle_checklist_templates::id.eq(id))
                    .filter(lifecycle_checklist_templates::workspace_id.eq(workspace_id))
                    .filter(lifecycle_checklist_templates::deleted_at.is_null())
                    .first::<LifecycleChecklistTemplateEntity>(conn)
                    .await
                    .map_err(|_err| AppError::not_found(I18nKey::TemplateNotFound))?;

                // If is_default is changing to true, unset others first
                if data.is_default == Some(true) {
                    diesel::update(lifecycle_checklist_templates::table)
                        .filter(lifecycle_checklist_templates::workspace_id.eq(workspace_id))
                        .filter(
                            lifecycle_checklist_templates::checklist_type
                                .eq(&existing.checklist_type),
                        )
                        .filter(lifecycle_checklist_templates::deleted_at.is_null())
                        .set(lifecycle_checklist_templates::is_default.eq(false))
                        .execute(conn)
                        .await
                        .map_err(|e| {
                            AppError::Internal(format!("db unset default template: {e}"))
                        })?;
                }

                if let Some(name) = data.name {
                    existing.name = name;
                }
                if let Some(desc) = data.description {
                    existing.description = Some(desc);
                }
                if let Some(is_default) = data.is_default {
                    existing.is_default = is_default;
                }
                if let Some(is_active) = data.is_active {
                    existing.is_active = is_active;
                }
                existing.updated_at = Utc::now();

                let result = diesel::update(
                    lifecycle_checklist_templates::table
                        .filter(lifecycle_checklist_templates::id.eq(id)),
                )
                .set(&existing)
                .get_result::<LifecycleChecklistTemplateEntity>(conn)
                .await
                .map_err(|e| AppError::Internal(format!("db update template: {e}")))?;

                Ok(result.into())
            })
        })
        .await
    }

    async fn soft_delete_template(
        &self,
        auth_token: &TenantAuthToken,
        id: Uuid,
    ) -> Result<(), AppError> {
        let mut conn = self
            .diesel_pools
            .tenant_transaction_conn(auth_token)
            .await?;
        let workspace_id = auth_token.workspace_id();

        let rows = diesel::update(
            lifecycle_checklist_templates::table
                .filter(lifecycle_checklist_templates::id.eq(id))
                .filter(lifecycle_checklist_templates::workspace_id.eq(workspace_id)),
        )
        .set(lifecycle_checklist_templates::deleted_at.eq(Some(Utc::now())))
        .execute(&mut *conn)
        .await
        .map_err(|e| AppError::Internal(format!("db soft delete template: {e}")))?;

        if rows == 0 {
            return Err(AppError::not_found(I18nKey::TemplateNotFound));
        }

        Ok(())
    }

    async fn get_default_template(
        &self,
        auth_token: &TenantAuthToken,
        checklist_type: LifecycleChecklistType,
    ) -> Result<Option<LifecycleChecklistTemplate>, AppError> {
        let mut conn = self
            .diesel_pools
            .tenant_transaction_conn(auth_token)
            .await?;
        let workspace_id = auth_token.workspace_id();

        let result = lifecycle_checklist_templates::table
            .filter(lifecycle_checklist_templates::workspace_id.eq(workspace_id))
            .filter(lifecycle_checklist_templates::checklist_type.eq(checklist_type.as_str()))
            .filter(lifecycle_checklist_templates::is_default.eq(true))
            .filter(lifecycle_checklist_templates::is_active.eq(true))
            .filter(lifecycle_checklist_templates::deleted_at.is_null())
            .first::<LifecycleChecklistTemplateEntity>(&mut *conn)
            .await
            .optional()
            .map_err(|e| AppError::Internal(format!("db get default template: {e}")))?;

        Ok(result.map(Into::into))
    }

    // --- Template Items ---
    async fn create_template_item(
        &self,
        auth_token: &TenantAuthToken,
        template_id: Uuid,
        data: CreateLifecycleChecklistTemplateItemRequest,
    ) -> Result<LifecycleChecklistTemplateItem, AppError> {
        let mut conn = self
            .diesel_pools
            .tenant_transaction_conn(auth_token)
            .await?;

        // Verify parent template belongs to this workspace
        let count: i64 = lifecycle_checklist_templates::table
            .filter(lifecycle_checklist_templates::id.eq(template_id))
            .filter(lifecycle_checklist_templates::workspace_id.eq(auth_token.workspace_id()))
            .count()
            .get_result(&mut *conn)
            .await
            .map_err(|e| AppError::Internal(format!("db verification count template: {e}")))?;
        if count == 0 {
            return Err(AppError::forbidden(
                crate::utils::i18n::I18nKey::TenantMismatch,
            ));
        }

        let entity = LifecycleChecklistTemplateItemEntity {
            id: Uuid::new_v4(),
            template_id,
            title: data.title,
            description: data.description,
            category: data.category.as_str().to_string(),
            assignee_type: data.assignee_type.as_str().to_string(),
            assignee_user_id: data.assignee_user_id,
            is_required: data.is_required,
            due_days_offset: data.due_days_offset,
            display_order: data.display_order,
            created_at: Utc::now(),
        };

        let result = diesel::insert_into(lifecycle_checklist_template_items::table)
            .values(&entity)
            .get_result::<LifecycleChecklistTemplateItemEntity>(&mut *conn)
            .await
            .map_err(|e| AppError::Internal(format!("db create template item: {e}")))?;

        Ok(result.into())
    }

    async fn get_template_item_by_id(
        &self,
        auth_token: &TenantAuthToken,
        id: Uuid,
    ) -> Result<Option<LifecycleChecklistTemplateItem>, AppError> {
        let mut conn = self
            .diesel_pools
            .tenant_transaction_conn(auth_token)
            .await?;

        let result = lifecycle_checklist_template_items::table
            .inner_join(lifecycle_checklist_templates::table)
            .filter(lifecycle_checklist_templates::workspace_id.eq(auth_token.workspace_id()))
            .filter(lifecycle_checklist_template_items::id.eq(id))
            .select(lifecycle_checklist_template_items::all_columns)
            .first::<LifecycleChecklistTemplateItemEntity>(&mut *conn)
            .await
            .optional()
            .map_err(|e| AppError::Internal(format!("db get template item: {e}")))?;

        Ok(result.map(Into::into))
    }

    async fn list_template_items(
        &self,
        auth_token: &TenantAuthToken,
        template_id: Uuid,
    ) -> Result<Vec<LifecycleChecklistTemplateItem>, AppError> {
        let mut conn = self
            .diesel_pools
            .tenant_transaction_conn(auth_token)
            .await?;

        let results = lifecycle_checklist_template_items::table
            .inner_join(lifecycle_checklist_templates::table)
            .filter(lifecycle_checklist_templates::workspace_id.eq(auth_token.workspace_id()))
            .filter(lifecycle_checklist_template_items::template_id.eq(template_id))
            .order(lifecycle_checklist_template_items::display_order.asc())
            .select(lifecycle_checklist_template_items::all_columns)
            .load::<LifecycleChecklistTemplateItemEntity>(&mut *conn)
            .await
            .map_err(|e| AppError::Internal(format!("db list template items: {e}")))?;

        Ok(results.into_iter().map(Into::into).collect())
    }

    async fn update_template_item(
        &self,
        auth_token: &TenantAuthToken,
        id: Uuid,
        data: UpdateLifecycleChecklistTemplateItemRequest,
    ) -> Result<LifecycleChecklistTemplateItem, AppError> {
        let mut conn = self
            .diesel_pools
            .tenant_transaction_conn(auth_token)
            .await?;

        let mut existing = lifecycle_checklist_template_items::table
            .inner_join(lifecycle_checklist_templates::table)
            .filter(lifecycle_checklist_templates::workspace_id.eq(auth_token.workspace_id()))
            .filter(lifecycle_checklist_template_items::id.eq(id))
            .select(lifecycle_checklist_template_items::all_columns)
            .first::<LifecycleChecklistTemplateItemEntity>(&mut *conn)
            .await
            .map_err(|_err| AppError::not_found(I18nKey::TemplateItemNotFound))?;

        if let Some(title) = data.title {
            existing.title = title;
        }
        if let Some(desc) = data.description {
            existing.description = Some(desc);
        }
        if let Some(cat) = data.category {
            existing.category = cat.as_str().to_string();
        }
        if let Some(at) = data.assignee_type {
            existing.assignee_type = at.as_str().to_string();
        }
        if let Some(assignee_user_id) = data.assignee_user_id {
            existing.assignee_user_id = assignee_user_id;
        }
        if let Some(is_required) = data.is_required {
            existing.is_required = is_required;
        }
        if let Some(due_days_offset) = data.due_days_offset {
            existing.due_days_offset = due_days_offset;
        }
        if let Some(display_order) = data.display_order {
            existing.display_order = display_order;
        }

        let result = diesel::update(
            lifecycle_checklist_template_items::table
                .filter(lifecycle_checklist_template_items::id.eq(id))
                .filter(
                    lifecycle_checklist_template_items::template_id.eq_any(
                        lifecycle_checklist_templates::table
                            .select(lifecycle_checklist_templates::id)
                            .filter(
                                lifecycle_checklist_templates::workspace_id
                                    .eq(auth_token.workspace_id()),
                            ),
                    ),
                ),
        )
        .set(&existing)
        .get_result::<LifecycleChecklistTemplateItemEntity>(&mut *conn)
        .await
        .map_err(|e| AppError::Internal(format!("db update template item: {e}")))?;

        Ok(result.into())
    }

    async fn delete_template_item(
        &self,
        auth_token: &TenantAuthToken,
        id: Uuid,
    ) -> Result<(), AppError> {
        let mut conn = self
            .diesel_pools
            .tenant_transaction_conn(auth_token)
            .await?;

        let rows = diesel::delete(
            lifecycle_checklist_template_items::table
                .filter(lifecycle_checklist_template_items::id.eq(id))
                .filter(
                    lifecycle_checklist_template_items::template_id.eq_any(
                        lifecycle_checklist_templates::table
                            .select(lifecycle_checklist_templates::id)
                            .filter(
                                lifecycle_checklist_templates::workspace_id
                                    .eq(auth_token.workspace_id()),
                            ),
                    ),
                ),
        )
        .execute(&mut *conn)
        .await
        .map_err(|e| AppError::Internal(format!("db delete template item: {e}")))?;

        if rows == 0 {
            return Err(AppError::not_found(I18nKey::TemplateItemNotFound));
        }

        Ok(())
    }

    // --- Checklist Instances ---
    async fn create_checklist(
        &self,
        auth_token: &TenantAuthToken,
        employment_id: Uuid,
        template_id: Option<Uuid>,
        checklist_type: LifecycleChecklistType,
        created_by: Option<Uuid>,
    ) -> Result<LifecycleChecklist, AppError> {
        let mut conn = self
            .diesel_pools
            .tenant_transaction_conn(auth_token)
            .await?;
        let workspace_id = auth_token.workspace_id();

        let entity = LifecycleChecklistEntity {
            id: Uuid::new_v4(),
            workspace_id,
            employment_id,
            template_id,
            checklist_type: checklist_type.as_str().to_string(),
            status: ChecklistStatus::InProgress.as_str().to_string(),
            progress_percent: 0,
            started_at: Utc::now(),
            completed_at: None,
            created_by,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let result = diesel::insert_into(lifecycle_checklists::table)
            .values(&entity)
            .get_result::<LifecycleChecklistEntity>(&mut *conn)
            .await
            .map_err(|e| AppError::Internal(format!("db create checklist instance: {e}")))?;

        Ok(result.into())
    }

    async fn get_checklist_by_id(
        &self,
        auth_token: &TenantAuthToken,
        id: Uuid,
    ) -> Result<Option<LifecycleChecklist>, AppError> {
        let mut conn = self
            .diesel_pools
            .tenant_transaction_conn(auth_token)
            .await?;
        let workspace_id = auth_token.workspace_id();

        let result = lifecycle_checklists::table
            .filter(lifecycle_checklists::id.eq(id))
            .filter(lifecycle_checklists::workspace_id.eq(workspace_id))
            .first::<LifecycleChecklistEntity>(&mut *conn)
            .await
            .optional()
            .map_err(|e| AppError::Internal(format!("db get checklist: {e}")))?;

        Ok(result.map(Into::into))
    }

    async fn list_checklists_for_employment(
        &self,
        auth_token: &TenantAuthToken,
        employment_id: Uuid,
    ) -> Result<Vec<LifecycleChecklist>, AppError> {
        let mut conn = self
            .diesel_pools
            .tenant_transaction_conn(auth_token)
            .await?;
        let workspace_id = auth_token.workspace_id();

        let results = lifecycle_checklists::table
            .filter(lifecycle_checklists::workspace_id.eq(workspace_id))
            .filter(lifecycle_checklists::employment_id.eq(employment_id))
            .order(lifecycle_checklists::created_at.desc())
            .load::<LifecycleChecklistEntity>(&mut *conn)
            .await
            .map_err(|e| AppError::Internal(format!("db list checklists: {e}")))?;

        Ok(results.into_iter().map(Into::into).collect())
    }

    async fn update_checklist_status_and_progress(
        &self,
        auth_token: &TenantAuthToken,
        id: Uuid,
        status: ChecklistStatus,
        progress_percent: i32,
        completed_at: Option<chrono::DateTime<chrono::Utc>>,
    ) -> Result<LifecycleChecklist, AppError> {
        let mut conn = self
            .diesel_pools
            .tenant_transaction_conn(auth_token)
            .await?;
        let workspace_id = auth_token.workspace_id();

        let result = diesel::update(
            lifecycle_checklists::table
                .filter(lifecycle_checklists::id.eq(id))
                .filter(lifecycle_checklists::workspace_id.eq(workspace_id)),
        )
        .set((
            lifecycle_checklists::status.eq(status.as_str()),
            lifecycle_checklists::progress_percent.eq(progress_percent),
            lifecycle_checklists::completed_at.eq(completed_at),
            lifecycle_checklists::updated_at.eq(Utc::now()),
        ))
        .get_result::<LifecycleChecklistEntity>(&mut *conn)
        .await
        .map_err(|e| AppError::Internal(format!("db update checklist progress: {e}")))?;

        Ok(result.into())
    }

    // --- Checklist Item Instances ---
    async fn create_checklist_items(
        &self,
        auth_token: &TenantAuthToken,
        items: Vec<LifecycleChecklistItemEntity>,
    ) -> Result<Vec<LifecycleChecklistItem>, AppError> {
        if items.is_empty() {
            return Ok(vec![]);
        }

        let mut conn = self
            .diesel_pools
            .tenant_transaction_conn(auth_token)
            .await?;

        // Verify parent checklist belongs to this workspace
        if let Some(first_item) = items.as_slice().first() {
            let count: i64 = lifecycle_checklists::table
                .filter(lifecycle_checklists::id.eq(first_item.checklist_id))
                .filter(lifecycle_checklists::workspace_id.eq(auth_token.workspace_id()))
                .count()
                .get_result(&mut *conn)
                .await
                .map_err(|e| AppError::Internal(format!("db verification count checklist: {e}")))?;
            if count == 0 {
                return Err(AppError::forbidden(
                    crate::utils::i18n::I18nKey::TenantMismatch,
                ));
            }
        }

        let results = diesel::insert_into(lifecycle_checklist_items::table)
            .values(&items)
            .get_results::<LifecycleChecklistItemEntity>(&mut *conn)
            .await
            .map_err(|e| AppError::Internal(format!("db insert checklist items: {e}")))?;

        Ok(results.into_iter().map(Into::into).collect())
    }

    async fn get_checklist_item_by_id(
        &self,
        auth_token: &TenantAuthToken,
        id: Uuid,
    ) -> Result<Option<LifecycleChecklistItem>, AppError> {
        let mut conn = self
            .diesel_pools
            .tenant_transaction_conn(auth_token)
            .await?;

        let result = lifecycle_checklist_items::table
            .inner_join(lifecycle_checklists::table)
            .filter(lifecycle_checklists::workspace_id.eq(auth_token.workspace_id()))
            .filter(lifecycle_checklist_items::id.eq(id))
            .select(lifecycle_checklist_items::all_columns)
            .first::<LifecycleChecklistItemEntity>(&mut *conn)
            .await
            .optional()
            .map_err(|e| AppError::Internal(format!("db get checklist item: {e}")))?;

        Ok(result.map(Into::into))
    }

    async fn list_checklist_items(
        &self,
        auth_token: &TenantAuthToken,
        checklist_id: Uuid,
    ) -> Result<Vec<LifecycleChecklistItem>, AppError> {
        let mut conn = self
            .diesel_pools
            .tenant_transaction_conn(auth_token)
            .await?;

        let results = lifecycle_checklist_items::table
            .inner_join(lifecycle_checklists::table)
            .filter(lifecycle_checklists::workspace_id.eq(auth_token.workspace_id()))
            .filter(lifecycle_checklist_items::checklist_id.eq(checklist_id))
            .order(lifecycle_checklist_items::display_order.asc())
            .select(lifecycle_checklist_items::all_columns)
            .load::<LifecycleChecklistItemEntity>(&mut *conn)
            .await
            .map_err(|e| AppError::Internal(format!("db list checklist items: {e}")))?;

        Ok(results.into_iter().map(Into::into).collect())
    }

    async fn update_checklist_item_status(
        &self,
        auth_token: &TenantAuthToken,
        id: Uuid,
        status: ChecklistItemStatus,
        completed_by: Option<Uuid>,
        notes: Option<String>,
    ) -> Result<LifecycleChecklistItem, AppError> {
        let mut conn = self
            .diesel_pools
            .tenant_transaction_conn(auth_token)
            .await?;

        let completed_at = if status == ChecklistItemStatus::Completed {
            Some(Utc::now())
        } else {
            None
        };

        let result = diesel::update(
            lifecycle_checklist_items::table
                .filter(lifecycle_checklist_items::id.eq(id))
                .filter(
                    lifecycle_checklist_items::checklist_id.eq_any(
                        lifecycle_checklists::table
                            .select(lifecycle_checklists::id)
                            .filter(
                                lifecycle_checklists::workspace_id.eq(auth_token.workspace_id()),
                            ),
                    ),
                ),
        )
        .set((
            lifecycle_checklist_items::status.eq(status.as_str()),
            lifecycle_checklist_items::completed_at.eq(completed_at),
            lifecycle_checklist_items::completed_by.eq(completed_by),
            lifecycle_checklist_items::notes.eq(notes),
            lifecycle_checklist_items::updated_at.eq(Utc::now()),
        ))
        .get_result::<LifecycleChecklistItemEntity>(&mut *conn)
        .await
        .map_err(|e| AppError::Internal(format!("db update checklist item: {e}")))?;

        Ok(result.into())
    }

    async fn get_pending_tasks_for_user(
        &self,
        auth_token: &TenantAuthToken,
        user_id: Uuid,
    ) -> Result<Vec<PendingTaskResponse>, AppError> {
        let mut conn = self
            .diesel_pools
            .tenant_transaction_conn(auth_token)
            .await?;
        let workspace_id = auth_token.workspace_id();

        // Join: lifecycle_checklist_items -> lifecycle_checklists -> employments -> employees (for employee name)
        let query_results = lifecycle_checklist_items::table
            .inner_join(lifecycle_checklists::table)
            .inner_join(
                employments::table.on(lifecycle_checklists::employment_id.eq(employments::id)),
            )
            .inner_join(employees::table.on(employments::employee_id.eq(employees::id)))
            .filter(lifecycle_checklist_items::assignee_user_id.eq(Some(user_id)))
            .filter(lifecycle_checklist_items::status.eq(ChecklistItemStatus::Pending.as_str()))
            .filter(lifecycle_checklists::workspace_id.eq(workspace_id))
            .select((
                lifecycle_checklist_items::id,
                lifecycle_checklist_items::checklist_id,
                employees::full_name,
                lifecycle_checklists::checklist_type,
                lifecycle_checklist_items::title,
                lifecycle_checklist_items::description,
                lifecycle_checklist_items::category,
                lifecycle_checklist_items::due_date,
                lifecycle_checklist_items::is_required,
            ))
            .order(lifecycle_checklist_items::due_date.asc().nulls_last())
            .load::<(
                Uuid,              // id
                Uuid,              // checklist_id
                String,            // full_name
                String,            // checklist_type
                String,            // title
                Option<String>,    // description
                String,            // category
                Option<NaiveDate>, // due_date
                bool,              // is_required
            )>(&mut *conn)
            .await
            .map_err(|e| AppError::Internal(format!("db get pending checklist tasks: {e}")))?;

        let responses = query_results
            .into_iter()
            .map(|(id, cl_id, name, cl_type, title, desc, cat, due, req)| {
                let checklist_type = LifecycleChecklistType::from_str(&cl_type)
                    .unwrap_or(LifecycleChecklistType::Onboarding);
                let category =
                    ChecklistCategory::from_str(&cat).unwrap_or(ChecklistCategory::HrAdmin);
                PendingTaskResponse {
                    id,
                    checklist_id: cl_id,
                    employee_name: name,
                    checklist_type,
                    title,
                    description: desc,
                    category,
                    due_date: due,
                    is_required: req,
                }
            })
            .collect();

        Ok(responses)
    }
}
