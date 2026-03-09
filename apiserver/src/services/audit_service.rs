use crate::domain::audit::AuditLog;
use axum::http::HeaderMap;
use std::sync::Arc;
use uuid::Uuid;

pub struct AuditHelper;

impl AuditHelper {
    #[allow(clippy::too_many_arguments)]
    #[tracing::instrument(skip_all)]
    pub async fn log(
        repo: &Arc<dyn crate::repositories::audit_log_repository::AuditLogRepository>,
        workspace_id: Uuid,
        actor_id: Option<Uuid>,
        target_user_id: Option<Uuid>,
        target_resource_id: Option<Uuid>,
        action: &str,
        details: Option<serde_json::Value>,
        headers: &HeaderMap,
    ) {
        let user_agent = headers
            .get(axum::http::header::USER_AGENT)
            .and_then(|h| h.to_str().ok())
            .map(|s| s.to_string());

        let ip_address = headers
            .get("X-Forwarded-For")
            .and_then(|h| h.to_str().ok())
            .map(|s| s.split(',').next().unwrap_or("").trim().to_string())
            .or_else(|| {
                headers
                    .get("X-Real-IP")
                    .and_then(|h| h.to_str().ok())
                    .map(|s| s.to_string())
            });

        let log = AuditLog {
            id: Uuid::new_v4(),
            workspace_id,
            actor_id,
            target_user_id,
            target_resource_id,
            action: action.to_string(),
            details,
            ip_address,
            user_agent,
            created_at: chrono::Utc::now(),
        };

        if let Err(e) = repo.create_log(&log).await {
            tracing::error!("Failed to create audit log: {}", e);
        }
    }
}
