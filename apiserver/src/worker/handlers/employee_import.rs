//! Employee Import Job Handlers
//!
//! Processes bulk employee imports from Excel files as a background job.
//! Implements a Chunking & Fan-out architecture for large files.

use chrono_tz::Tz;
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use crate::models::ImportJobStatus;
use crate::queue::{EmployeeImportChunkPayload, Job, JobPayload, JobPriority, TaskType};
use crate::services::attendance_import_service::EmployeeImportStrategy;
use crate::services::import_messages::AttendanceImportMessages;
use crate::services::import_pipeline::ImportPipeline;
use crate::worker::handlers::utils::ImportTimeoutGuard;

pub struct EmployeeImportHandler;

impl EmployeeImportHandler {
    /// Handle the master employee import job.
    /// Splits the file into chunks and enqueues them.
    #[allow(clippy::too_many_lines)]
    pub async fn handle(
        job: &Job,
        cache: &std::sync::Arc<dyn crate::repositories::CacheRepository>,
        queue_repo: &std::sync::Arc<dyn crate::repositories::QueueRepository>,
        deps: &crate::worker::deps::WorkerDeps,
    ) -> Result<(), String> {
        let payload = job
            .employee_import_payload()
            .ok_or("Missing employee import payload")?;

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
            user_id = %payload.user_id,
            import_job_id = %payload.import_job_id,
            "Starting employee import chunking"
        );

        let import_job = import_repo
            .get_job_by_id(
                &crate::diesel_pool::TenantAuthToken::system_override(job.workspace_id),
                payload.import_job_id,
            )
            .await
            .map_err(|e| format!("DB error: {e}"))?
            .ok_or_else(|| "Job not found".to_string())?;

        let file_path = import_job
            .file_path
            .as_ref()
            .ok_or_else(|| "No file path provided".to_string())?;

        let file_bytes = tokio::fs::read(file_path)
            .await
            .map_err(|e| format!("Could not read file: {e}"))?;

        tracing::info!("Extracting mapped rows");
        let rows = crate::utils::file_parser::extract_mapped_rows(&file_bytes, &["employee_code"])
            .map_err(|e| format!("Failed to parse file: {e}"))?;

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

        tracing::info!("Setting Redis keys");
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

        tracing::info!("Enqueueing chunk jobs");
        // Enqueue chunk jobs
        let mut sub_jobs = Vec::new();
        for (idx, (start_line, end_line)) in chunks.into_iter().enumerate() {
            sub_jobs.push(Job::with_priority(
                payload.import_job_id,
                idx as u32,
                TaskType::EmployeeImportChunk,
                job.workspace_id,
                JobPayload::EmployeeImportChunk(EmployeeImportChunkPayload {
                    import_job_id: payload.import_job_id,
                    user_id: payload.user_id,
                    file_path: file_path.clone(),
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

        Ok(())
    }
}

pub struct EmployeeImportChunkHandler;

impl EmployeeImportChunkHandler {
    #[allow(clippy::too_many_lines)]
    pub async fn handle(
        job: &Job,
        cache: &std::sync::Arc<dyn crate::repositories::CacheRepository>,
        queue_repo: &std::sync::Arc<dyn crate::repositories::QueueRepository>,
        deps: &crate::worker::deps::WorkerDeps,
    ) -> Result<(), String> {
        let payload = job
            .employee_import_chunk_payload()
            .ok_or("Missing chunk payload")?;

        tracing::info!(
            chunk = %job.batch_index,
            import_job_id = %payload.import_job_id,
            start_index = payload.start_index,
            end_index = payload.end_index,
            "Processing employee import chunk"
        );

        let completed = Arc::new(AtomicBool::new(false));
        // Using timeout guard but errors shouldn't set job to failed globally right away,
        // Wait, if a chunk times out, the guard sets the whole job to Failed.
        // This might be correct to halt the process, but we rely on chunk completion tracking.
        let _guard = ImportTimeoutGuard::new(
            job.workspace_id,
            payload.import_job_id,
            Arc::clone(&completed),
            Arc::clone(&deps.import_job_repo),
        );

        let import_repo = Arc::clone(&deps.import_job_repo);
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
                AttendanceImportMessages::for_lang(ws.default_language()),
            ),
            _ => (Tz::UTC, AttendanceImportMessages::for_lang("vi")),
        };

        let strategy = EmployeeImportStrategy {
            msgs,
            employee_repo: std::sync::Arc::clone(&deps.employee_repo),
            department_repo: std::sync::Arc::clone(&deps.department_repo),
            staging_repo: std::sync::Arc::clone(&deps.staging_repo),
            dept_cache: HashMap::new(),
            emp_cache: HashMap::new(),
            newly_added_emps: std::collections::HashSet::new(),
        };

        let pipeline = ImportPipeline {
            strategy,
            deps: (*deps).clone(),
            import_repo: &*import_repo,
            job: &import_job,
            file_bytes,
            workspace_tz,
            start_index: Some(payload.start_index),
            end_index: Some(payload.end_index),
            is_chunk: true,
            queue_repo: Some(std::sync::Arc::clone(queue_repo)),
        };

        let _ = pipeline.run().await; // Returns Ok/Err but we don't fail the whole job here on validation errors

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
    use crate::queue::{EmployeeImportPayload, JobPriority, TaskType};
    use crate::worker::test_utils::MockWorkerDepsBuilder;
    use std::io::Write;
    use std::sync::Arc;
    use tempfile::NamedTempFile;
    use uuid::Uuid;

    #[tokio::test]
    async fn test_employee_import_handler_master() {
        let import_job_id = Uuid::new_v4();
        let ws_id = Uuid::new_v4();

        let temp_file = NamedTempFile::new().unwrap();
        // Create a fake excel file or use CSV? The parser extract_mapped_rows uses csv/excel. Wait, file_parser checks extension. Let's make it a .csv
        let file_path = temp_file.path().to_str().unwrap().to_string() + ".csv";
        let mut actual_file = std::fs::File::create(&file_path).unwrap();
        writeln!(actual_file, "employee_code,full_name").unwrap();
        writeln!(actual_file, "EMP01,John Doe").unwrap();
        writeln!(actual_file, "EMP02,Jane Doe").unwrap();

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
            .expect_get_job_by_id()
            .with(
                mockall::predicate::always(),
                mockall::predicate::eq(import_job_id),
            )
            .returning(move |_, _| {
                Ok(Some(crate::models::AttendanceImportJob {
                    id: import_job_id,
                    workspace_id: ws_id,
                    user_id: Uuid::new_v4(),
                    target_type: crate::models::enums::ImportTargetType::Employee,
                    status: ImportJobStatus::Pending,
                    file_path: Some(file_path.clone()),
                    total_rows: 0,
                    processed_rows: 0,
                    error_log: None,
                    created_at: chrono::Utc::now(),
                    updated_at: chrono::Utc::now(),
                }))
            });

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
            assert_eq!(jobs[0].task_type, TaskType::EmployeeImportChunk);
            Ok(())
        });

        let mut cache = crate::repositories::cache_repository::MockCacheRepository::new();
        cache.expect_set_json().returning(|_, _, _| Ok(()));

        let cache_arc: Arc<dyn crate::repositories::CacheRepository> = Arc::new(cache);
        let queue_arc: Arc<dyn crate::repositories::QueueRepository> = Arc::new(queue);

        let mut builder = MockWorkerDepsBuilder::new();
        builder.import_job_repo = import_repo;
        let deps = builder.build();

        let job = Job {
            job_id: Uuid::new_v4(),
            parent_job_id: import_job_id,
            batch_index: 0,
            task_type: TaskType::EmployeeImport,
            workspace_id: ws_id,
            payload: JobPayload::EmployeeImport(EmployeeImportPayload {
                import_job_id,
                user_id: Uuid::new_v4(),
            }),
            priority: JobPriority::Normal,
            created_at: chrono::Utc::now(),
            retry_count: 0,
            max_retries: 3,
        };

        let result = EmployeeImportHandler::handle(&job, &cache_arc, &queue_arc, &deps).await;
        assert!(result.is_ok(), "Handler failed: {:?}", result.err());
    }
}
