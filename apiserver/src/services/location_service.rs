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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repositories::audit_log_repository::MockAuditLogRepository;
    use crate::repositories::location_repository::MockLocationRepository;
    use chrono::Utc;
    use std::sync::Arc;

    fn make_location(ws: Uuid) -> WorkLocation {
        WorkLocation {
            id: Uuid::new_v4(),
            workspace_id: ws,
            name: "HQ Office".into(),
            address: Some("123 Main St".into()),
            timezone: "Asia/Ho_Chi_Minh".into(),
            is_headquarters: false,
            status: "active".into(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    fn make_headquarters(ws: Uuid) -> WorkLocation {
        WorkLocation {
            id: Uuid::new_v4(),
            workspace_id: ws,
            name: "HQ Office".into(),
            address: Some("123 Main St".into()),
            timezone: "Asia/Ho_Chi_Minh".into(),
            is_headquarters: true,
            status: "active".into(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    fn build_service(repo: MockLocationRepository) -> LocationService {
        LocationService::new(
            Arc::new(repo),
            Arc::new(MockAuditLogRepository::new()),
        )
    }

    // ── list_locations tests ──────────────────────────────────────────────

    #[tokio::test]
    async fn list_locations_success() {
        let ws = Uuid::new_v4();
        let location = make_location(ws);
        let expected = vec![location.clone()];

        let mut repo = MockLocationRepository::new();
        repo.expect_find_all_by_workspace()
            .returning(move |_, _, _| Ok((expected.clone(), 1)));

        let svc = build_service(repo);
        let result = svc.list_locations(ws, 1, 10).await;

        assert!(result.is_ok());
        let (list, total) = result.unwrap();
        assert_eq!(list.len(), 1);
        assert_eq!(total, 1);
    }

    // ── get_location tests ────────────────────────────────────────────────

    #[tokio::test]
    async fn get_location_found() {
        let ws = Uuid::new_v4();
        let location = make_location(ws);
        let loc_id = location.id;
        let expected = location.clone();

        let mut repo = MockLocationRepository::new();
        repo.expect_find_by_id()
            .returning(move |_, _| Ok(Some(expected.clone())));

        let svc = build_service(repo);
        let result = svc.get_location(ws, loc_id).await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap().id, loc_id);
    }

    #[tokio::test]
    async fn get_location_not_found() {
        let ws = Uuid::new_v4();
        let mut repo = MockLocationRepository::new();
        repo.expect_find_by_id()
            .returning(|_, _| Ok(None));

        let svc = build_service(repo);
        let result = svc.get_location(ws, Uuid::new_v4()).await;

        assert!(matches!(result.unwrap_err(), AppError::NotFound(_)));
    }

    // ── create_location tests ─────────────────────────────────────────────

    #[tokio::test]
    async fn create_location_headquarters_rejected() {
        let ws = Uuid::new_v4();
        let repo = MockLocationRepository::new();
        let svc = build_service(repo);

        let result = svc
            .create_location(
                ws,
                Uuid::new_v4(),
                CreateWorkLocationRequest {
                    name: "HQ".into(),
                    address: None,
                    timezone: None,
                    is_headquarters: Some(true),
                    status: None,
                },
            )
            .await;

        assert!(matches!(result.unwrap_err(), AppError::Validation(_)));
    }

    // ── delete_location tests ─────────────────────────────────────────────

    #[tokio::test]
    async fn delete_location_headquarters_rejected() {
        let ws = Uuid::new_v4();
        let hq = make_headquarters(ws);
        let expected = hq.clone();

        let mut repo = MockLocationRepository::new();
        repo.expect_find_by_id()
            .returning(move |_, _| Ok(Some(expected.clone())));

        let svc = build_service(repo);
        let result = svc.delete_location(ws, hq.id, Uuid::new_v4()).await;

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("cannot_delete_headquarters"));
    }

    #[tokio::test]
    async fn delete_location_in_use() {
        let ws = Uuid::new_v4();
        let location = make_location(ws);
        let expected = location.clone();
        let loc_id = location.id;

        let mut repo = MockLocationRepository::new();
        repo.expect_find_by_id()
            .returning(move |_, _| Ok(Some(expected.clone())));
        repo.expect_count_employees_in_location()
            .returning(|_, _| Ok(5));

        let svc = build_service(repo);
        let result = svc.delete_location(ws, loc_id, Uuid::new_v4()).await;

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("cannot_delete_location_in_use"));
    }
}
