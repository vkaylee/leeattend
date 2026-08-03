use axum::{
    extract::{Multipart, Path, State},
    response::Json,
};
use std::sync::Arc;
use uuid::Uuid;

use crate::domain::permissions::Permission;
use crate::errors::{AppError, AppResult};
use crate::handlers::common::MemberContext;
use crate::models::{enums::ImportTargetType, AttendanceImportJob};
use crate::repositories::import_job_repository::ImportJobRepository;
use crate::repositories::queue_repository::QueueRepository;
use crate::utils::i18n::I18nKey;
use crate::AppState;

#[utoipa::path(
    post,
    path = "/v1/workspaces/{ws_id}/attendance/upload",
    params(
        ("ws_id" = Uuid, Path, description = "Workspace ID")
    ),
    request_body(content = String, content_type = "multipart/form-data"),
    responses(
        (status = 200, description = "Attendance adjustment uploaded", body = serde_json::Value)
    ),
    security(
        ("bearer_auth" = [])
    )
)]
#[tracing::instrument(skip_all, fields(workspace_id = %ctx.workspace_id, user_id = %ctx.user_id))]
pub async fn upload_attendance_excel(
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    mut multipart: Multipart,
) -> AppResult<Json<serde_json::Value>> {
    // RBAC: Only Admin/Owner
    ctx.require_permission(Permission::AttendanceLogsImport)?;

    let path = crate::utils::upload::extract_uploaded_file(
        &state,
        Some(&ctx.auth_token),
        &mut multipart,
        "file",
        crate::utils::upload::MAX_UPLOAD_FILE_SIZE,
        "imports",
    )
    .await?;
    let path = path.to_string_lossy().to_string();

    let job_id = state
        .attendance()
        .import_service()
        .batch_import_from_file(
            ctx.workspace_id,
            ctx.user_id,
            path,
            ImportTargetType::Adjustment,
        )
        .await?;

    Ok(Json(serde_json::json!({ "job_id": job_id })))
}

#[utoipa::path(
    post,
    path = "/v1/workspaces/{ws_id}/attendance/logs/upload",
    params(
        ("ws_id" = Uuid, Path, description = "Workspace ID")
    ),
    request_body(content = String, content_type = "multipart/form-data"),
    responses(
        (status = 200, description = "Attendance logs uploaded", body = serde_json::Value)
    ),
    security(
        ("bearer_auth" = [])
    )
)]
#[tracing::instrument(skip_all, fields(workspace_id = %ctx.workspace_id, user_id = %ctx.user_id))]
pub async fn upload_attendance_logs(
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    mut multipart: Multipart,
) -> AppResult<Json<serde_json::Value>> {
    // RBAC: Only Admin/Owner
    ctx.require_permission(Permission::AttendanceLogsImport)?;

    let path = crate::utils::upload::extract_uploaded_file(
        &state,
        Some(&ctx.auth_token),
        &mut multipart,
        "file",
        crate::utils::upload::MAX_UPLOAD_FILE_SIZE,
        "imports",
    )
    .await?;
    let path = path.to_string_lossy().to_string();

    let job_id = state
        .attendance()
        .import_service()
        .batch_import_from_file(ctx.workspace_id, ctx.user_id, path, ImportTargetType::Log)
        .await?;

    Ok(Json(serde_json::json!({ "job_id": job_id })))
}

#[utoipa::path(
    post,
    path = "/v1/workspaces/{ws_id}/employees/upload",
    params(
        ("ws_id" = Uuid, Path, description = "Workspace ID")
    ),
    request_body(content = String, content_type = "multipart/form-data"),
    responses(
        (status = 200, description = "Employees uploaded", body = serde_json::Value)
    ),
    security(
        ("bearer_auth" = [])
    )
)]
#[tracing::instrument(skip_all, fields(workspace_id = %ctx.workspace_id, user_id = %ctx.user_id))]
pub async fn upload_employee_excel(
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    mut multipart: Multipart,
) -> AppResult<Json<serde_json::Value>> {
    // RBAC: Only Admin/Owner
    ctx.require_permission(Permission::EmployeesUpload)?;

    let path = crate::utils::upload::extract_uploaded_file(
        &state,
        Some(&ctx.auth_token),
        &mut multipart,
        "file",
        crate::utils::upload::MAX_UPLOAD_FILE_SIZE,
        "imports",
    )
    .await?;
    let path = path.to_string_lossy().to_string();

    // Create import job record in DB
    let import_job_id = crate::utils::id::new_id();
    let import_job = crate::models::AttendanceImportJob {
        id: import_job_id,
        workspace_id: ctx.workspace_id,
        user_id: ctx.user_id,
        status: crate::models::ImportJobStatus::Pending,
        total_rows: 0,
        processed_rows: 0,
        file_path: Some(path.clone()),
        error_log: None,
        target_type: ImportTargetType::Employee,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };

    state
        .attendance()
        .import_repo()
        .create_job(&ctx.auth_token, &import_job)
        .await
        .map_err(|e| AppError::Internal(format!("Failed to create import job: {e}")))?;

    use crate::queue::{EmployeeImportPayload, Job, JobPayload, TaskType};

    let payload = EmployeeImportPayload {
        import_job_id,
        user_id: ctx.user_id,
    };

    let job = Job::new(
        import_job_id,
        0,
        TaskType::EmployeeImport,
        ctx.workspace_id,
        JobPayload::EmployeeImport(payload),
    );

    let queue = state.infra().job_queue()?;
    queue
        .enqueue_jobs(vec![job])
        .await
        .map_err(|e| AppError::Internal(format!("Failed to enqueue employee import job: {e}")))?;

    Ok(Json(serde_json::json!({ "job_id": import_job_id })))
}

/// Check if a Processing import job has timed out (>10 min since last update)
/// and rescue it (mark Failed). Also syncs live progress from the job queue
/// into `job.processed_rows`.
async fn check_and_rescue_timeout_job(
    job: &mut AttendanceImportJob,
    import_repo: &Arc<dyn ImportJobRepository>,
    queue: Option<&Arc<dyn QueueRepository>>,
    auth_token: &crate::diesel_pool::TenantAuthToken,
) {
    if job.status != crate::models::ImportJobStatus::Processing {
        return;
    }

    let now = chrono::Utc::now();
    let duration = now - job.updated_at;
    if duration.num_minutes() > 10 {
        let _ = import_repo
            .update_job_progress(
                auth_token,
                job.id,
                job.processed_rows,
                crate::models::enums::ImportJobStatus::Failed,
                Some("Import job timed out or background worker restarted.".to_string()),
            )
            .await;
        job.status = crate::models::enums::ImportJobStatus::Failed;
        job.error_log =
            Some("Import job timed out or background worker restarted.".to_string());
    } else if let Some(queue_repo) = queue {
        if let Ok(Some(count)) = queue_repo.get_import_progress(job.id).await {
            job.processed_rows = count;
        }
    }
}

#[derive(serde::Deserialize)]
pub struct ImportJobQuery {
    pub target_type: Option<ImportTargetType>,
}

#[utoipa::path(
    get,
    path = "/v1/workspaces/{ws_id}/attendance/import-jobs/latest",
    params(
        ("ws_id" = Uuid, Path, description = "Workspace ID"),
        ("target_type" = Option<ImportTargetType>, Query, description = "Target type filter")
    ),
    responses(
        (status = 200, description = "Latest import job", body = Option<AttendanceImportJob>)
    ),
    security(
        ("bearer_auth" = [])
    )
)]
#[tracing::instrument(skip_all, fields(workspace_id = %ctx.workspace_id, user_id = %ctx.user_id))]
pub async fn get_latest_import_job(
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    axum::extract::Query(query): axum::extract::Query<ImportJobQuery>,
) -> AppResult<Json<Option<AttendanceImportJob>>> {
    // RBAC: viewing import jobs requires AttendanceLogsImport permission
    ctx.require_permission(Permission::AttendanceLogsImport)?;

    let mut job_opt = state
        .attendance()
        .import_repo()
        .get_latest_job(&ctx.auth_token, query.target_type)
        .await?;

    if let Some(ref mut job) = job_opt {
        check_and_rescue_timeout_job(
            job,
            state.attendance().import_repo(),
            state.infra().queue(),
            &ctx.auth_token,
        )
        .await;
    }

    Ok(Json(job_opt))
}

#[utoipa::path(
    get,
    path = "/v1/workspaces/{ws_id}/attendance/import-jobs/{id}",
    params(
        ("ws_id" = Uuid, Path, description = "Workspace ID"),
        ("id" = Uuid, Path, description = "Job ID")
    ),
    responses(
        (status = 200, description = "Import job status", body = AttendanceImportJob)
    ),
    security(
        ("bearer_auth" = [])
    )
)]
#[tracing::instrument(skip_all, fields(workspace_id = %ctx.workspace_id, user_id = %ctx.user_id, job_id = %__path_params.1))]
pub async fn get_import_job_status(
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    Path(__path_params): Path<(Uuid, Uuid)>,
) -> AppResult<Json<AttendanceImportJob>> {
    let (_, job_id) = __path_params;

    // RBAC: viewing import job status requires AttendanceLogsImport permission
    ctx.require_permission(Permission::AttendanceLogsImport)?;

    let mut job = state
        .attendance()
        .import_repo()
        .get_job_by_id(&ctx.auth_token, job_id)
        .await?
        .ok_or_else(|| AppError::not_found(I18nKey::JobNotFound))?;

    check_and_rescue_timeout_job(
        &mut job,
        state.attendance().import_repo(),
        state.infra().queue(),
        &ctx.auth_token,
    )
    .await;

    Ok(Json(job))
}

#[utoipa::path(
    get,
    path = "/v1/workspaces/{ws_id}/attendance/import-jobs/{id}/error-file",
    params(
        ("ws_id" = Uuid, Path, description = "Workspace ID"),
        ("id" = Uuid, Path, description = "Job ID")
    ),
    responses(
        (status = 200, description = "Error file downloaded", body = Vec<u8>, content_type = "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
#[tracing::instrument(skip_all, fields(workspace_id = %ctx.workspace_id, user_id = %ctx.user_id, job_id = %__path_params.1))]
pub async fn download_import_error_file(
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    Path(__path_params): Path<(Uuid, Uuid)>,
) -> AppResult<impl axum::response::IntoResponse> {
    let (_, job_id) = __path_params;
    let workspace_id = ctx.workspace_id;

    // RBAC: downloading error files requires AttendanceLogsImport permission
    ctx.require_permission(Permission::AttendanceLogsImport)?;

    let job = state
        .attendance()
        .import_repo()
        .get_job_by_id(&ctx.auth_token, job_id)
        .await?
        .ok_or_else(|| AppError::not_found(I18nKey::JobNotFound))?;

    if job.workspace_id != workspace_id {
        return Err(AppError::forbidden(I18nKey::Forbidden));
    }

    let base_dir = crate::utils::get_storage_dir();
    let file_path = format!("{base_dir}/imports/error_{job_id}.xlsx");

    let data = tokio::fs::read(&file_path)
        .await
        .map_err(|_e| AppError::not_found(I18nKey::ErrorFileNotFound))?;

    use axum::http::header;
    let headers = [
        (
            header::CONTENT_TYPE,
            "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet".to_string(),
        ),
        (
            header::CONTENT_DISPOSITION,
            format!("attachment; filename=\"error_{job_id}.xlsx\""),
        ),
    ];

    Ok((headers, data))
}

#[cfg(test)]
mod tests {
    use super::check_and_rescue_timeout_job;
    use crate::diesel_pool::TenantAuthToken;
    use crate::errors::AppError;
    use crate::models::{AttendanceImportJob, ImportJobStatus};
    use crate::repositories::import_job_repository::ImportJobRepository;
    use crate::repositories::queue_repository::MockQueueRepository;
    use async_trait::async_trait;
    use chrono::{Duration, Utc};
    use std::sync::{Arc, Mutex};
    use uuid::Uuid;

    // ── MockImportJobRepo ──────────────────────────────────────────────

    struct MockImportJobInner {
        last_update: Option<(Uuid, i32, ImportJobStatus, Option<String>)>,
    }

    struct MockImportJobRepo {
        inner: Arc<Mutex<MockImportJobInner>>,
    }

    #[async_trait]
    impl ImportJobRepository for MockImportJobRepo {
        async fn create_job(
            &self,
            _auth_token: &TenantAuthToken,
            _job: &AttendanceImportJob,
        ) -> Result<(), AppError> {
            unimplemented!()
        }

        async fn update_job_progress(
            &self,
            _auth_token: &TenantAuthToken,
            id: Uuid,
            processed_rows: i32,
            status: ImportJobStatus,
            error_log: Option<String>,
        ) -> Result<(), AppError> {
            self.inner.lock().unwrap().last_update =
                Some((id, processed_rows, status, error_log));
            Ok(())
        }

        async fn batch_update_job_progress(
            &self,
            _auth_token: &TenantAuthToken,
            _updates: Vec<(Uuid, i32, ImportJobStatus, Option<String>)>,
        ) -> Result<(), AppError> {
            unimplemented!()
        }

        async fn get_job_by_id(
            &self,
            _auth_token: &TenantAuthToken,
            _id: Uuid,
        ) -> Result<Option<AttendanceImportJob>, AppError> {
            unimplemented!()
        }

        async fn get_latest_job(
            &self,
            _auth_token: &TenantAuthToken,
            _target_type: Option<crate::models::enums::ImportTargetType>,
        ) -> Result<Option<AttendanceImportJob>, AppError> {
            unimplemented!()
        }

        async fn get_pending_jobs(
            &self,
            _auth_token: &TenantAuthToken,
        ) -> Result<Vec<AttendanceImportJob>, AppError> {
            unimplemented!()
        }

        async fn update_job_total_rows(
            &self,
            _auth_token: &TenantAuthToken,
            _id: Uuid,
            _total_rows: i32,
        ) -> Result<(), AppError> {
            unimplemented!()
        }
    }

    // ── Helpers ──────────────────────────────────────────────────────

    fn make_auth_token() -> TenantAuthToken {
        TenantAuthToken(Uuid::new_v4())
    }

    fn make_processing_job(updated_at: chrono::DateTime<Utc>) -> AttendanceImportJob {
        let now = Utc::now();
        AttendanceImportJob {
            id: Uuid::new_v4(),
            workspace_id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            status: ImportJobStatus::Processing,
            total_rows: 100,
            processed_rows: 42,
            file_path: Some("/tmp/test.xlsx".to_string()),
            error_log: None,
            target_type: crate::models::enums::ImportTargetType::Log,
            created_at: now - Duration::hours(1),
            updated_at,
        }
    }

    fn setup_mock_import_repo() -> (Arc<dyn ImportJobRepository>, Arc<Mutex<MockImportJobInner>>)
    {
        let inner = Arc::new(Mutex::new(MockImportJobInner { last_update: None }));
        let repo: Arc<dyn ImportJobRepository> = Arc::new(MockImportJobRepo {
            inner: inner.clone(),
        });
        (repo, inner)
    }

    // ── Tests ────────────────────────────────────────────────────────

    /// Non-Processing jobs should be untouched (no DB write, no progress sync).
    #[tokio::test]
    async fn test_non_processing_job_is_noop() {
        let (import_repo, inner) = setup_mock_import_repo();
        let queue: Arc<dyn crate::repositories::QueueRepository> =
            Arc::new(MockQueueRepository::new());
        let auth = make_auth_token();

        let mut job = make_processing_job(Utc::now());
        job.status = ImportJobStatus::Completed;

        check_and_rescue_timeout_job(&mut job, &import_repo, Some(&queue), &auth).await;

        // Job status must NOT be changed
        assert_eq!(job.status, ImportJobStatus::Completed);
        // No DB write should have happened
        assert!(inner.lock().unwrap().last_update.is_none());
    }

    /// A Processing job updated <10 min ago should only sync progress,
    /// NOT be marked Failed.
    #[tokio::test]
    async fn test_processing_within_timeout_syncs_progress() {
        let (import_repo, inner) = setup_mock_import_repo();
        let mut queue = MockQueueRepository::new();
        let auth = make_auth_token();

        // Set up queue to return progress = 55
        let job = make_processing_job(Utc::now() - Duration::minutes(3));
        let job_id = job.id;
        queue
            .expect_get_import_progress()
            .with(mockall::predicate::eq(job_id))
            .returning(|_| Ok(Some(55)));

        let queue: Arc<dyn crate::repositories::QueueRepository> = Arc::new(queue);
        let mut job = job;

        check_and_rescue_timeout_job(&mut job, &import_repo, Some(&queue), &auth).await;

        // Status must remain Processing
        assert_eq!(job.status, ImportJobStatus::Processing);
        // Progress must be synced from queue
        assert_eq!(job.processed_rows, 55);
        // No DB write
        assert!(inner.lock().unwrap().last_update.is_none());
    }

    /// A Processing job stale for >10 min must be rescued (marked Failed).
    #[tokio::test]
    async fn test_processing_beyond_timeout_marks_failed() {
        let (import_repo, inner) = setup_mock_import_repo();
        let queue: Arc<dyn crate::repositories::QueueRepository> =
            Arc::new(MockQueueRepository::new());
        let auth = make_auth_token();

        let mut job = make_processing_job(Utc::now() - Duration::minutes(15));

        check_and_rescue_timeout_job(&mut job, &import_repo, Some(&queue), &auth).await;

        // Must be marked Failed
        assert_eq!(job.status, ImportJobStatus::Failed);
        assert!(job
            .error_log
            .as_deref()
            .unwrap_or("")
            .contains("timed out"));

        // DB must have been updated
        let update = inner
            .lock()
            .unwrap()
            .last_update
            .clone()
            .expect("update_job_progress must be called");
        assert_eq!(update.0, job.id);
        assert_eq!(update.2, ImportJobStatus::Failed);
    }

    /// When no queue repo is available, timeout rescue should still work
    /// (just without progress sync).
    #[tokio::test]
    async fn test_processing_beyond_timeout_without_queue() {
        let (import_repo, inner) = setup_mock_import_repo();
        let auth = make_auth_token();

        let mut job = make_processing_job(Utc::now() - Duration::minutes(20));

        // No queue available
        check_and_rescue_timeout_job(&mut job, &import_repo, None, &auth).await;

        // Must still be rescued
        assert_eq!(job.status, ImportJobStatus::Failed);
        assert!(inner.lock().unwrap().last_update.is_some());
    }

    /// When the queue repo returns an error, the handler should silently
    /// skip the progress sync (not panic or propagate).
    #[tokio::test]
    async fn test_queue_error_is_silently_ignored() {
        let (import_repo, _inner) = setup_mock_import_repo();
        let mut queue = MockQueueRepository::new();
        let auth = make_auth_token();

        let job = make_processing_job(Utc::now() - Duration::minutes(3));
        let job_id = job.id;
        queue
            .expect_get_import_progress()
            .with(mockall::predicate::eq(job_id))
            .returning(|_| Err("redis connection lost".to_string()));

        let queue: Arc<dyn crate::repositories::QueueRepository> = Arc::new(queue);
        let mut job = job;

        check_and_rescue_timeout_job(&mut job, &import_repo, Some(&queue), &auth).await;

        // Must NOT fail — just skip progress sync
        assert_eq!(job.status, ImportJobStatus::Processing);
        // processed_rows must stay at original value
        assert_eq!(job.processed_rows, 42);
    }

    /// Pending jobs are non-Processing and should be a no-op.
    #[tokio::test]
    async fn test_pending_job_is_noop() {
        let (import_repo, inner) = setup_mock_import_repo();
        let queue: Arc<dyn crate::repositories::QueueRepository> =
            Arc::new(MockQueueRepository::new());
        let auth = make_auth_token();

        let mut job = make_processing_job(Utc::now());
        job.status = ImportJobStatus::Pending;

        check_and_rescue_timeout_job(&mut job, &import_repo, Some(&queue), &auth).await;

        assert_eq!(job.status, ImportJobStatus::Pending);
        assert!(inner.lock().unwrap().last_update.is_none());
    }

    /// Failed jobs should not be double-rescued.
    #[tokio::test]
    async fn test_already_failed_job_is_noop() {
        let (import_repo, inner) = setup_mock_import_repo();
        let queue: Arc<dyn crate::repositories::QueueRepository> =
            Arc::new(MockQueueRepository::new());
        let auth = make_auth_token();

        let mut job = make_processing_job(Utc::now() - Duration::minutes(30));
        job.status = ImportJobStatus::Failed;
        job.error_log = Some("already failed".to_string());

        check_and_rescue_timeout_job(&mut job, &import_repo, Some(&queue), &auth).await;

        // Must NOT be overwritten
        assert_eq!(job.status, ImportJobStatus::Failed);
        assert_eq!(job.error_log.as_deref(), Some("already failed"));
        assert!(inner.lock().unwrap().last_update.is_none());
    }
}
