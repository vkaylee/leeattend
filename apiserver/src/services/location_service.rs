use crate::errors::AppError;
use crate::models::{CreateWorkLocationRequest, UpdateWorkLocationRequest, WorkLocation};
use crate::repositories::audit_log_repository::AuditLogRepository;
use crate::repositories::location_repository::LocationRepository;
use crate::services::audit_service::{AuditHelper, AuditLogArgs};
use crate::utils::i18n::I18nKey;
use axum::http::HeaderMap;
use serde_json::json;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Clone)]
pub struct LocationService {
    repo: Arc<dyn LocationRepository>,
    audit_repo: Arc<dyn AuditLogRepository>,
}

impl LocationService {
    pub fn new(repo: Arc<dyn LocationRepository>, audit_repo: Arc<dyn AuditLogRepository>) -> Self {
        Self { repo, audit_repo }
    }

    pub async fn list_locations(
        &self,
        workspace_id: Uuid,
        page: i64,
        limit: i64,
    ) -> Result<(Vec<WorkLocation>, i64), AppError> {
        self.repo
            .find_all_by_workspace(workspace_id, page, limit)
            .await
            .map_err(|e| AppError::Internal(format!("Failed to list locations: {e}")))
    }

    pub async fn get_location(
        &self,
        workspace_id: Uuid,
        id: Uuid,
    ) -> Result<WorkLocation, AppError> {
        self.repo
            .find_by_id(workspace_id, id)
            .await
            .map_err(|e| AppError::Internal(format!("Failed to get location: {e}")))?
            .ok_or_else(|| AppError::not_found(I18nKey::LocationNotFound))
    }

    pub async fn create_location(
        &self,
        workspace_id: Uuid,
        actor_id: Uuid,
        data: CreateWorkLocationRequest,
    ) -> Result<WorkLocation, AppError> {
        if data.is_headquarters == Some(true) {
            return Err(AppError::validation_args(
                I18nKey::Validation,
                vec![("details", "cannot_create_headquarters".to_string())],
            ));
        }

        let location = self
            .repo
            .create(workspace_id, data.clone())
            .await
            .map_err(|e| AppError::Internal(format!("Failed to create location: {e}")))?;

        let empty_headers = HeaderMap::new();
        AuditHelper::log(AuditLogArgs {
            repo: &self.audit_repo,
            workspace_id,
            actor_id: Some(actor_id),
            actor_type: Some("user".to_string()),
            actor_name: None,
            actor_email: None,
            target_user_id: None,
            target_resource_id: Some(location.id),
            action: "location.created",
            details: Some(json!({ "name": location.name, "timezone": location.timezone })),
            headers: &empty_headers,
        })
        .await;

        Ok(location)
    }

    pub async fn update_location(
        &self,
        workspace_id: Uuid,
        id: Uuid,
        actor_id: Uuid,
        data: UpdateWorkLocationRequest,
    ) -> Result<WorkLocation, AppError> {
        let existing = self.get_location(workspace_id, id).await?;

        if data.is_headquarters == Some(true) && !existing.is_headquarters {
            return Err(AppError::validation_args(
                I18nKey::Validation,
                vec![(
                    "details",
                    "cannot_change_location_to_headquarters".to_string(),
                )],
            ));
        }

        if existing.is_headquarters && data.is_headquarters == Some(false) {
            return Err(AppError::validation_args(
                I18nKey::Validation,
                vec![("details", "cannot_remove_headquarters_flag".to_string())],
            ));
        }

        let location = self
            .repo
            .update(workspace_id, id, data)
            .await
            .map_err(|e| AppError::Internal(format!("Failed to update location: {e}")))?;

        let empty_headers = HeaderMap::new();
        AuditHelper::log(AuditLogArgs {
            repo: &self.audit_repo,
            workspace_id,
            actor_id: Some(actor_id),
            actor_type: Some("user".to_string()),
            actor_name: None,
            actor_email: None,
            target_user_id: None,
            target_resource_id: Some(location.id),
            action: "location.updated",
            details: Some(json!({ "name": location.name })),
            headers: &empty_headers,
        })
        .await;

        Ok(location)
    }

    pub async fn delete_location(
        &self,
        workspace_id: Uuid,
        id: Uuid,
        actor_id: Uuid,
    ) -> Result<(), AppError> {
        let existing = self.get_location(workspace_id, id).await?;

        if existing.is_headquarters {
            return Err(AppError::validation_args(
                I18nKey::Validation,
                vec![("details", "cannot_delete_headquarters".to_string())],
            ));
        }

        let employee_count = self
            .repo
            .count_employees_in_location(workspace_id, id)
            .await
            .map_err(|e| AppError::Internal(format!("Failed to count employees: {e}")))?;

        if employee_count > 0 {
            return Err(AppError::validation_args(
                I18nKey::Validation,
                vec![
                    ("details", "cannot_delete_location_in_use".to_string()),
                    ("employee_count", employee_count.to_string()),
                ],
            ));
        }

        self.repo
            .delete(workspace_id, id)
            .await
            .map_err(|e| AppError::Internal(format!("Failed to delete location: {e}")))?;

        let empty_headers = HeaderMap::new();
        AuditHelper::log(AuditLogArgs {
            repo: &self.audit_repo,
            workspace_id,
            actor_id: Some(actor_id),
            actor_type: Some("user".to_string()),
            actor_name: None,
            actor_email: None,
            target_user_id: None,
            target_resource_id: Some(existing.id),
            action: "location.deleted",
            details: Some(json!({ "name": existing.name })),
            headers: &empty_headers,
        })
        .await;

        Ok(())
    }
}
