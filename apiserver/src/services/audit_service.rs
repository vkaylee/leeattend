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
            .map(std::string::ToString::to_string);

        let ip_address = args
            .headers
            .get("X-Forwarded-For")
            .and_then(|h| h.to_str().ok())
            .map(|s| s.split(',').next().unwrap_or("").trim().to_string())
            .or_else(|| {
                args.headers
                    .get("X-Real-IP")
                    .and_then(|h| h.to_str().ok())
                    .map(std::string::ToString::to_string)
            });

        let log = AuditLog {
            id: crate::utils::id::new_id(),
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::audit::AuditLog;
    use crate::errors::AppError;
    use crate::repositories::audit_log_repository::AuditLogRepository;
    use async_trait::async_trait;
    use axum::http::HeaderMap;
    use chrono::Utc;
    use std::sync::{Arc, Mutex};
    use uuid::Uuid;

    struct MockAuditRepo {
        logged: Mutex<bool>,
    }

    impl MockAuditRepo {
        fn new() -> Arc<Self> {
            Arc::new(Self { logged: Mutex::new(false) })
        }
    }

    #[async_trait]
    impl AuditLogRepository for MockAuditRepo {
        async fn create_log(&self, log: &AuditLog) -> Result<AuditLog, AppError> {
            *self.logged.lock().unwrap() = true;
            Ok(log.clone())
        }

        async fn find_by_workspace(&self, _w: Uuid, _l: i64, _o: i64, _a: Option<String>, _s: Option<chrono::DateTime<Utc>>, _e: Option<chrono::DateTime<Utc>>) -> Result<(Vec<AuditLog>, i64), AppError> {
            Ok((vec![], 0))
        }
    }

    #[tokio::test]
    async fn test_audit_log_basic() {
        let repo: Arc<dyn AuditLogRepository> = MockAuditRepo::new();
        let arc_repo = Arc::new(repo);
        let headers = HeaderMap::new();

        AuditHelper::log(AuditLogArgs {
            repo: &arc_repo,
            workspace_id: Uuid::new_v4(),
            actor_id: Some(Uuid::new_v4()),
            target_user_id: None,
            target_resource_id: None,
            action: "employee.create",
            details: None,
            headers: &headers,
        }).await;
        // No panic means success — logs errors internally
    }

    #[tokio::test]
    async fn test_audit_log_with_user_agent_and_ip() {
        let repo: Arc<dyn AuditLogRepository> = MockAuditRepo::new();
        let arc_repo = Arc::new(repo);
        let mut headers = HeaderMap::new();
        headers.insert(axum::http::header::USER_AGENT, "Mozilla/5.0".parse().unwrap());
        headers.insert("X-Forwarded-For", "192.168.1.1, 10.0.0.1".parse().unwrap());

        AuditHelper::log(AuditLogArgs {
            repo: &arc_repo,
            workspace_id: Uuid::new_v4(),
            actor_id: Some(Uuid::new_v4()),
            target_user_id: Some(Uuid::new_v4()),
            target_resource_id: Some(Uuid::new_v4()),
            action: "leave.approve",
            details: Some(serde_json::json!({"status": "approved"})),
            headers: &headers,
        }).await;
    }

    #[tokio::test]
    async fn test_audit_log_with_x_real_ip() {
        let repo: Arc<dyn AuditLogRepository> = MockAuditRepo::new();
        let arc_repo = Arc::new(repo);
        let mut headers = HeaderMap::new();
        headers.insert("X-Real-IP", "10.0.0.1".parse().unwrap());

        AuditHelper::log(AuditLogArgs {
            repo: &arc_repo,
            workspace_id: Uuid::new_v4(),
            actor_id: None,
            target_user_id: None,
            target_resource_id: None,
            action: "workspace.update",
            details: None,
            headers: &headers,
        }).await;
    }
}
