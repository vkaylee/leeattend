//! Leave Import Job Handler
//!
//! Processes bulk leave request imports from Excel files as a background job.
//! Manages job tracking via the `attendance_import_jobs` table.

use crate::models::ImportJobStatus;
use crate::queue::{Job, JobPayload, JobPriority, LeaveImportChunkPayload, TaskType};
use crate::services::import_messages::LeaveImportMessages;
use crate::services::import_pipeline::ImportPipeline;
use crate::services::leave_import_service::LeaveImportStrategy;
use crate::services::leave_service::LeaveService;
use chrono_tz::Tz;
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use crate::worker::handlers::utils::ImportTimeoutGuard;

pub struct LeaveImportHandler;

impl LeaveImportHandler {
    #[allow(clippy::too_many_lines)]
    pub async fn handle(
        job: &Job,
        cache: &std::sync::Arc<dyn crate::repositories::CacheRepository>,
        queue_repo: &std::sync::Arc<dyn crate::repositories::QueueRepository>,
        deps: &crate::worker::deps::WorkerDeps,
    ) -> Result<(), String> {
        let payload = job
            .leave_import_payload()
            .ok_or("Missing leave import payload")?;

        let import_repo = &*deps.import_job_repo;

        // Update import job → Processing
        let _ = import_repo
            .update_job_progress(
                &crate::diesel_pool::TenantAuthToken::system_override(job.workspace_id),
                payload.import_job_id,
                0,
                ImportJobStatus::Processing,
                None,
            )
            .await;

        tracing::info!(
            workspace_id = %job.workspace_id,
            file_path = %payload.file_path,
            auto_approve = payload.auto_approve,
            import_job_id = %payload.import_job_id,
            "Starting leave import chunking"
        );

        let workspace_repo = &*deps.workspace_repo;
        let lang = match workspace_repo.get_workspace_by_id(job.workspace_id).await {
            Ok(Some(ws)) => ws.default_language().to_string(),
            _ => "vi".to_string(),
        };

        let file_bytes = tokio::fs::read(&payload.file_path)
            .await
            .map_err(|e| format!("Could not read file: {e}"))?;

        let rows = crate::utils::file_parser::extract_mapped_rows(&file_bytes, &["employee_code"])
            .map_err(|e| e.translate(&lang))?;

        let total_rows = rows.len();
        let _ = import_repo
            .update_job_total_rows(
                &crate::diesel_pool::TenantAuthToken::system_override(job.workspace_id),
                payload.import_job_id,
                total_rows as i32,
            )
            .await;

        if total_rows == 0 {
            let _ = import_repo
                .update_job_progress(
                    &crate::diesel_pool::TenantAuthToken::system_override(job.workspace_id),
                    payload.import_job_id,
                    0,
                    ImportJobStatus::Completed,
                    None,
                )
                .await;
            return Ok(());
        }

        let chunk_size = crate::worker::handlers::utils::get_import_chunk_size();
        let mut chunks = Vec::new();
        let mut i = 0;
        while i < rows.len() {
            let start = i;
            let end = (i + chunk_size).min(rows.len()) - 1;
            let start_line = rows[start].0;
            let end_line = rows[end].0;
            chunks.push((start_line, end_line));
            i += chunk_size;
        }

        let total_chunks = chunks.len() as u32;

        // Initialize Redis keys for progress tracking
        let ttl = std::time::Duration::from_secs(86400); // 24 hours
        let _ = cache
            .set_json(
                &format!("import_progress:{}", payload.import_job_id),
                "0",
                ttl,
            )
            .await;
        let _ = cache
            .set_json(
                &format!("import_chunks_done:{}", payload.import_job_id),
                "0",
                ttl,
            )
            .await;
        let _ = cache
            .set_json(
                &format!("import_total_chunks:{}", payload.import_job_id),
                &total_chunks.to_string(),
                ttl,
            )
            .await;

        // Clear previous error log if any
        let _ = cache
            .delete(&format!("import_errors:{}", payload.import_job_id))
            .await;

        // Enqueue chunk jobs
        let mut sub_jobs = Vec::new();
        for (idx, (start_line, end_line)) in chunks.into_iter().enumerate() {
            sub_jobs.push(Job::with_priority(
                payload.import_job_id,
                idx as u32,
                TaskType::LeaveImportChunk,
                job.workspace_id,
                JobPayload::LeaveImportChunk(LeaveImportChunkPayload {
                    import_job_id: payload.import_job_id,
                    user_id: payload.user_id,
                    file_path: payload.file_path.clone(),
                    auto_approve: payload.auto_approve,
                    start_index: start_line,
                    end_index: end_line,
                }),
                JobPriority::High,
            ));
        }

        queue_repo
            .enqueue_jobs(sub_jobs)
            .await
            .map_err(|e| format!("Failed to enqueue chunks: {e}"))?;

        tracing::info!(
            import_job_id = %payload.import_job_id,
            total_chunks = total_chunks,
            "Successfully split leave import job"
        );

        Ok(())
    }
}

pub struct LeaveImportChunkHandler;

impl LeaveImportChunkHandler {
    #[allow(clippy::too_many_lines)]
    pub async fn handle(
        job: &Job,
        cache: &std::sync::Arc<dyn crate::repositories::CacheRepository>,
        queue_repo: &std::sync::Arc<dyn crate::repositories::QueueRepository>,
        deps: &crate::worker::deps::WorkerDeps,
    ) -> Result<(), String> {
        let payload = job
            .leave_import_chunk_payload()
            .ok_or("Missing chunk payload")?;

        tracing::info!(
            chunk = %job.batch_index,
            import_job_id = %payload.import_job_id,
            start_index = payload.start_index,
            end_index = payload.end_index,
            "Processing leave import chunk"
        );

        let completed = Arc::new(AtomicBool::new(false));
        let _guard = ImportTimeoutGuard::new(
            job.workspace_id,
            payload.import_job_id,
            Arc::clone(&completed),
            Arc::clone(&deps.import_job_repo),
        );

        let import_repo = &*deps.import_job_repo;
        let import_job = import_repo
            .get_job_by_id(
                &crate::diesel_pool::TenantAuthToken::system_override(job.workspace_id),
                payload.import_job_id,
            )
            .await
            .map_err(|e| format!("DB error: {e}"))?
            .ok_or_else(|| "Job not found".to_string())?;

        let file_bytes = tokio::fs::read(&payload.file_path)
            .await
            .map_err(|e| format!("Could not read file: {e}"))?;

        let workspace_repo = Arc::clone(&deps.workspace_repo);
        let (workspace_tz, msgs) = match workspace_repo
            .get_workspace_by_id(import_job.workspace_id)
            .await
        {
            Ok(Some(ws)) => (
                ws.timezone().parse::<Tz>().unwrap_or(Tz::UTC),
                LeaveImportMessages::for_lang(ws.default_language()),
            ),
            _ => (Tz::UTC, LeaveImportMessages::for_lang("vi")),
        };

        let employee_repo = Arc::clone(&deps.employee_repo);
        let employment_repo = Arc::clone(&deps.employment_repo);
        let leave_balance_repo = Arc::clone(&deps.leave_balance_repo);
        let leave_request_repo = Arc::clone(&deps.leave_request_repo);
        let roster_repo = Arc::clone(&deps.roster_repo);
        let leave_accrual_log_repo = Arc::clone(&deps.leave_accrual_log_repo);
        let leave_service = LeaveService::new(
            leave_balance_repo,
            leave_request_repo,
            Arc::clone(&employee_repo),
            employment_repo,
            roster_repo,
            leave_accrual_log_repo,
            Arc::clone(&deps.workspace_config_repo),
            Arc::clone(&deps.user_repo),
            Arc::clone(&deps.workspace_repo),
            Some(Arc::clone(cache)),
            Some(deps.diesel_pools.clone()),
        );

        let strategy = LeaveImportStrategy {
            msgs,
            auto_approve: payload.auto_approve,
            user_id: payload.user_id,
            employees_cache: HashMap::new(),
            leave_types_cache: HashMap::new(),
            leave_service,
        };

        let pipeline = ImportPipeline {
            strategy,
            deps: (*deps).clone(),
            import_repo,
            job: &import_job,
            file_bytes,
            workspace_tz,
            start_index: Some(payload.start_index),
            end_index: Some(payload.end_index),
            is_chunk: true,
            queue_repo: Some(std::sync::Arc::clone(queue_repo)),
        };

        let _ = pipeline.run().await;

        // Chunk finished
        completed.store(true, Ordering::SeqCst);

        let chunks_done = cache
            .increment(&format!("import_chunks_done:{}", payload.import_job_id))
            .await
            .unwrap_or(0);

        let total_chunks: i64 = cache
            .get_json(&format!("import_total_chunks:{}", payload.import_job_id))
            .await
            .unwrap_or_else(|_| Some("0".to_string()))
            .unwrap_or_else(|| "0".to_string())
            .parse()
            .unwrap_or(0);

        if chunks_done >= total_chunks {
            tracing::info!(import_job_id = %payload.import_job_id, "All chunks completed. Triggering finalize.");
            let finalize_job = Job::with_priority(
                payload.import_job_id,
                0,
                TaskType::FinalizeImport,
                import_job.workspace_id,
                JobPayload::FinalizeImport(crate::queue::FinalizeImportPayload {
                    import_job_id: payload.import_job_id,
                }),
                JobPriority::High,
            );
            let _ = queue_repo.enqueue_jobs(vec![finalize_job]).await;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::queue::{JobPriority, LeaveImportPayload, TaskType};
    use crate::worker::test_utils::MockWorkerDepsBuilder;

    use std::sync::Arc;
    use tempfile::NamedTempFile;
    use uuid::Uuid;

    #[tokio::test]
    async fn test_leave_import_handler_master() {
        let import_job_id = Uuid::new_v4();
        let ws_id = Uuid::new_v4();

        let temp_file = NamedTempFile::new().unwrap();
        let file_path = temp_file.path().to_str().unwrap().to_string() + ".xlsx";
        let mut workbook = rust_xlsxwriter::Workbook::new();
        let sheet = workbook.add_worksheet();
        sheet.write_string(0, 0, "employee_code").unwrap();
        sheet.write_string(0, 1, "leave_type").unwrap();
        sheet.write_string(1, 0, "EMP01").unwrap();
        sheet.write_string(1, 1, "Annual").unwrap();
        sheet.write_string(2, 0, "EMP02").unwrap();
        sheet.write_string(2, 1, "Sick").unwrap();
        workbook.save(&file_path).unwrap();

        let mut import_repo =
            crate::repositories::import_job_repository::MockImportJobRepository::new();
        import_repo
            .expect_update_job_progress()
            .with(
                mockall::predicate::always(),
                mockall::predicate::eq(import_job_id),
                mockall::predicate::eq(0),
                mockall::predicate::eq(ImportJobStatus::Processing),
                mockall::predicate::eq(None),
            )
            .returning(|_, _, _, _, _| Ok(()));

        import_repo
            .expect_update_job_total_rows()
            .with(
                mockall::predicate::always(),
                mockall::predicate::eq(import_job_id),
                mockall::predicate::eq(2),
            )
            .returning(|_, _, _| Ok(()));

        let mut queue = crate::repositories::queue_repository::MockQueueRepository::new();
        queue.expect_enqueue_jobs().returning(|jobs| {
            assert_eq!(jobs.len(), 1); // 2 rows < chunk_size (500)
            assert_eq!(jobs[0].task_type, TaskType::LeaveImportChunk);
            Ok(())
        });

        let mut cache = crate::repositories::cache_repository::MockCacheRepository::new();
        cache.expect_set_json().returning(|_, _, _| Ok(()));
        cache.expect_delete().returning(|_| Ok(()));

        let cache_arc: Arc<dyn crate::repositories::CacheRepository> = Arc::new(cache);
        let queue_arc: Arc<dyn crate::repositories::QueueRepository> = Arc::new(queue);

        let mut workspace_repo =
            crate::repositories::workspace_repository::MockWorkspaceRepository::new();
        workspace_repo.expect_get_workspace_by_id().returning(|id| {
            Box::pin(async move {
                Ok(Some(crate::models::Workspace::new(
                    id,
                    "Mocked Workspace".into(),
                    "YYYY-MM-DD".into(),
                    "HH:mm:ss".into(),
                    "UTC".into(),
                    "#000000".into(),
                    "vi".into(),
                    None,
                    None,
                    "split".into(),
                    rust_decimal::Decimal::new(15, 1),
                    chrono::Utc::now(),
                    None,
                )))
            })
        });

        let mut builder = MockWorkerDepsBuilder::new();
        builder.import_job_repo = import_repo;
        builder.workspace_repo = workspace_repo;
        let deps = builder.build();

        let job = Job {
            job_id: Uuid::new_v4(),
            parent_job_id: import_job_id,
            batch_index: 0,
            task_type: TaskType::LeaveImport,
            workspace_id: ws_id,
            payload: JobPayload::LeaveImport(LeaveImportPayload {
                import_job_id,
                user_id: Uuid::new_v4(),
                file_path,
                auto_approve: true,
            }),
            priority: JobPriority::Normal,
            created_at: chrono::Utc::now(),
            retry_count: 0,
            max_retries: 3,
        };

        let result = LeaveImportHandler::handle(&job, &cache_arc, &queue_arc, &deps).await;
        assert!(result.is_ok(), "Handler failed: {:?}", result.err());
    }
}
