use crate::diesel_pool::TenantAuthToken;
use crate::errors::AppError;
use crate::models::OutboxJob;
use crate::repositories::outbox_job_repository::OutboxJobRepository;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Clone)]
pub struct OutboxJobService {
    outbox_repo: Arc<dyn OutboxJobRepository>,
}

impl OutboxJobService {
    pub fn new(outbox_repo: Arc<dyn OutboxJobRepository>) -> Self {
        Self { outbox_repo }
    }

    pub async fn create_job(
        &self,
        auth_token: &TenantAuthToken,
        id: Uuid,
        job_type: &str,
        payload: serde_json::Value,
    ) -> Result<OutboxJob, AppError> {
        let mut payload = payload;
        if let Some(trace_ctx) = crate::utils::logging::get_trace_context() {
            if let Some(obj) = payload.as_object_mut() {
                obj.insert(
                    "_trace_context".to_string(),
                    serde_json::Value::String(trace_ctx),
                );
            }
        }
        self.outbox_repo
            .insert_job(auth_token, id, job_type, payload)
            .await
    }

    pub async fn complete_job(
        &self,
        auth_token: &TenantAuthToken,
        id: Uuid,
    ) -> Result<(), AppError> {
        self.outbox_repo
            .update_job_status(auth_token, id, "completed", None)
            .await
    }

    pub async fn fail_job(
        &self,
        auth_token: &TenantAuthToken,
        id: Uuid,
        error: &str,
    ) -> Result<(), AppError> {
        self.outbox_repo
            .update_job_status(auth_token, id, "failed", Some(error.to_string()))
            .await
    }

    pub async fn update_job_status_if_current(
        &self,
        auth_token: &TenantAuthToken,
        id: Uuid,
        expected_status: &str,
        status: &str,
        error_message: Option<String>,
    ) -> Result<(), AppError> {
        self.outbox_repo
            .update_job_status_if_current(auth_token, id, expected_status, status, error_message)
            .await
    }

    pub async fn get_pending_jobs(
        &self,
        auth_token: &TenantAuthToken,
        limit: i64,
    ) -> Result<Vec<OutboxJob>, AppError> {
        self.outbox_repo.get_pending_jobs(auth_token, limit).await
    }

    pub async fn get_stuck_jobs(
        &self,
        auth_token: &TenantAuthToken,
        older_than: chrono::DateTime<chrono::Utc>,
        limit: i64,
    ) -> Result<Vec<OutboxJob>, AppError> {
        self.outbox_repo
            .get_stuck_jobs(auth_token, older_than, limit)
            .await
    }

    pub async fn get_all_stuck_jobs(
        &self,
        older_than: chrono::DateTime<chrono::Utc>,
        limit: i64,
    ) -> Result<Vec<OutboxJob>, AppError> {
        self.outbox_repo.get_all_stuck_jobs(older_than, limit).await
    }

    pub async fn get_all_stuck_jobs_by_type(
        &self,
        job_type: &str,
        older_than: chrono::DateTime<chrono::Utc>,
        limit: i64,
    ) -> Result<Vec<OutboxJob>, AppError> {
        self.outbox_repo
            .get_all_stuck_jobs_by_type(job_type, older_than, limit)
            .await
    }

    pub async fn update_job_status(
        &self,
        auth_token: &TenantAuthToken,
        id: Uuid,
        status: &str,
        error_message: Option<String>,
    ) -> Result<(), AppError> {
        self.outbox_repo
            .update_job_status(auth_token, id, status, error_message)
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::*;
    use crate::repositories::outbox_job_repository::MockOutboxJobRepository;
    use serde_json::json;

    #[tokio::test]
    async fn test_create_job() {
        let ws_id = Uuid::new_v4();
        let job_id = Uuid::new_v4();
        let payload = json!({"test": "data"});
        let job_type = "TestJob";

        let mut mock_repo = MockOutboxJobRepository::new();
        let expected_job = OutboxJob::new(NewOutboxJobParams {
            id: job_id,
            workspace_id: ws_id,
            job_type: job_type.to_string(),
            payload: payload.clone(),
            status: "pending".to_string(),
            error_message: None,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        });

        let expected_clone = expected_job.clone();
        mock_repo
            .expect_insert_job()
            .withf(move |auth_token, id, j_type, p| {
                auth_token.workspace_id() == ws_id
                    && *id == job_id
                    && j_type == "TestJob"
                    && *p == json!({"test": "data"})
            })
            .returning(move |_, _, _, _| Ok(expected_clone.clone()));

        let service = OutboxJobService::new(Arc::new(mock_repo));
        let result = service
            .create_job(
                &TenantAuthToken::system_override(ws_id),
                job_id,
                job_type,
                payload,
            )
            .await;

        assert!(result.is_ok());
        let job = result.unwrap();
        assert_eq!(job.id(), job_id);
        assert_eq!(job.workspace_id(), ws_id);
        assert_eq!(job.job_type(), "TestJob");
        assert_eq!(job.status(), "pending");
    }

    #[tokio::test]
    async fn test_complete_job() {
        let ws_id = Uuid::new_v4();
        let job_id = Uuid::new_v4();

        let mut mock_repo = MockOutboxJobRepository::new();
        mock_repo
            .expect_update_job_status()
            .withf(move |auth_token, id, status, err_msg| {
                auth_token.workspace_id() == ws_id
                    && *id == job_id
                    && status == "completed"
                    && err_msg.is_none()
            })
            .returning(|_, _, _, _| Ok(()));

        let service = OutboxJobService::new(Arc::new(mock_repo));
        let result = service
            .complete_job(&TenantAuthToken::system_override(ws_id), job_id)
            .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_fail_job() {
        let ws_id = Uuid::new_v4();
        let job_id = Uuid::new_v4();
        let error_msg = "Something went wrong";

        let mut mock_repo = MockOutboxJobRepository::new();
        mock_repo
            .expect_update_job_status()
            .withf(move |auth_token, id, status, err_msg| {
                auth_token.workspace_id() == ws_id
                    && *id == job_id
                    && status == "failed"
                    && err_msg.as_deref() == Some("Something went wrong")
            })
            .returning(|_, _, _, _| Ok(()));

        let service = OutboxJobService::new(Arc::new(mock_repo));
        let result = service
            .fail_job(&TenantAuthToken::system_override(ws_id), job_id, error_msg)
            .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_get_pending_jobs() {
        let ws_id = Uuid::new_v4();
        let limit = 5;

        let mut mock_repo = MockOutboxJobRepository::new();
        let expected_jobs = vec![OutboxJob::new(NewOutboxJobParams {
            id: Uuid::new_v4(),
            workspace_id: ws_id,
            job_type: "TestJob".to_string(),
            payload: json!({}),
            status: "pending".to_string(),
            error_message: None,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        })];

        let expected_clone = expected_jobs.clone();
        mock_repo
            .expect_get_pending_jobs()
            .withf(move |auth_token, lim| auth_token.workspace_id() == ws_id && *lim == limit)
            .returning(move |_, _| Ok(expected_clone.clone()));

        let service = OutboxJobService::new(Arc::new(mock_repo));
        let result = service
            .get_pending_jobs(&TenantAuthToken::system_override(ws_id), limit)
            .await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 1);
    }

    #[tokio::test]
    async fn test_get_stuck_jobs() {
        let ws_id = Uuid::new_v4();
        let older_than = chrono::Utc::now();
        let limit = 10;

        let mut mock_repo = MockOutboxJobRepository::new();
        mock_repo
            .expect_get_stuck_jobs()
            .withf(move |auth_token, old_t, lim| {
                auth_token.workspace_id() == ws_id && *old_t == older_than && *lim == limit
            })
            .returning(move |_, _, _| Ok(vec![]));

        let service = OutboxJobService::new(Arc::new(mock_repo));
        let result = service
            .get_stuck_jobs(&TenantAuthToken::system_override(ws_id), older_than, limit)
            .await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 0);
    }

    #[tokio::test]
    async fn test_update_job_status() {
        let ws_id = Uuid::new_v4();
        let job_id = Uuid::new_v4();
        let status = "running";
        let err_msg = Some("In progress".to_string());

        let mut mock_repo = MockOutboxJobRepository::new();
        mock_repo
            .expect_update_job_status()
            .withf(move |auth_token, id, s, e| {
                auth_token.workspace_id() == ws_id
                    && *id == job_id
                    && s == "running"
                    && *e == Some("In progress".to_string())
            })
            .returning(|_, _, _, _| Ok(()));

        let service = OutboxJobService::new(Arc::new(mock_repo));
        let result = service
            .update_job_status(
                &TenantAuthToken::system_override(ws_id),
                job_id,
                status,
                err_msg,
            )
            .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_create_job_with_trace_context() {
        let ws_id = Uuid::new_v4();
        let job_id = Uuid::new_v4();
        let payload = json!({"test": "data"});
        let job_type = "TestJob";

        let mut mock_repo = MockOutboxJobRepository::new();
        let expected_job = OutboxJob::new(NewOutboxJobParams {
            id: job_id,
            workspace_id: ws_id,
            job_type: job_type.to_string(),
            payload: json!({"test": "data", "_trace_context": "test-request-id-123"}),
            status: "pending".to_string(),
            error_message: None,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        });

        let expected_clone = expected_job.clone();
        mock_repo
            .expect_insert_job()
            .withf(move |auth_token, id, j_type, p| {
                auth_token.workspace_id() == ws_id
                    && *id == job_id
                    && j_type == "TestJob"
                    && *p == json!({"test": "data", "_trace_context": "test-request-id-123"})
            })
            .returning(move |_, _, _, _| Ok(expected_clone.clone()));

        let service = OutboxJobService::new(Arc::new(mock_repo));

        let result = crate::utils::logging::CURRENT_TRACE_CONTEXT
            .scope("test-request-id-123".to_string(), async {
                service
                    .create_job(
                        &TenantAuthToken::system_override(ws_id),
                        job_id,
                        job_type,
                        payload,
                    )
                    .await
            })
            .await;

        assert!(result.is_ok());
        let job = result.unwrap();
        assert_eq!(job.trace_context(), Some("test-request-id-123"));
    }
}
