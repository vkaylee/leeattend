use crate::domain::audit::AuditLog;
use axum::http::HeaderMap;
use std::sync::Arc;
use uuid::Uuid;

pub struct AuditLogArgs<'a> {
    pub repo: &'a Arc<dyn crate::repositories::audit_log_repository::AuditLogRepository>,
    pub workspace_id: Uuid,
    pub actor_id: Option<Uuid>,
    pub target_user_id: Option<Uuid>,
    pub target_resource_id: Option<Uuid>,
    pub action: &'a str,
    pub details: Option<serde_json::Value>,
    pub headers: &'a HeaderMap,
}

pub struct AuditHelper;

impl AuditHelper {
    #[tracing::instrument(skip_all)]
    pub async fn log(args: AuditLogArgs<'_>) {
        let user_agent = args
            .headers
            .get(axum::http::header::USER_AGENT)
            .and_then(|h| h.to_str().ok())
            .map(|s| s.to_string());

        let ip_address = args
            .headers
            .get("X-Forwarded-For")
            .and_then(|h| h.to_str().ok())
            .map(|s| s.split(',').next().unwrap_or("").trim().to_string())
            .or_else(|| {
                args.headers
                    .get("X-Real-IP")
                    .and_then(|h| h.to_str().ok())
                    .map(|s| s.to_string())
            });

        let log = AuditLog {
            id: Uuid::new_v4(),
            workspace_id: args.workspace_id,
            actor_id: args.actor_id,
            target_user_id: args.target_user_id,
            target_resource_id: args.target_resource_id,
            action: args.action.to_string(),
            details: args.details,
            ip_address,
            user_agent,
            created_at: chrono::Utc::now(),
        };

        if let Err(e) = args.repo.create_log(&log).await {
            tracing::error!("Failed to create audit log: {}", e);
        }
    }
}
