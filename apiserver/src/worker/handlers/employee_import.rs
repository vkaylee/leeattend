//! Employee Import Job Handlers
//!
//! Processes bulk employee imports from Excel files as a background job.
//! Implements a Chunking & Fan-out architecture for large files.

use chrono_tz::Tz;
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use crate::models::ImportJobStatus;
use crate::queue::{
    EmployeeImportChunkPayload, Job, JobPayload, JobPriority, JobQueue, RedisPool, TaskType,
};
use crate::repositories::import_job_repository::ImportJobRepository;
use crate::repositories::workspace_repository::WorkspaceRepository;
use crate::repositories::DieselImportJobRepository;
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
        diesel: &crate::diesel_pool::DieselPools,
        job: &Job,
        redis: &RedisPool,
    ) -> Result<(), String> {
        let payload = job
            .employee_import_payload()
            .ok_or("Missing employee import payload")?;

        let import_repo = DieselImportJobRepository::new(diesel.clone());

        // Update import job → Processing
        let _ = import_repo
            .update_job_progress(payload.import_job_id, 0, ImportJobStatus::Processing, None)
            .await;

        tracing::info!(
            workspace_id = %job.workspace_id,
            user_id = %payload.user_id,
            import_job_id = %payload.import_job_id,
            "Starting employee import chunking"
        );

        let import_job = import_repo
            .get_job_by_id(payload.import_job_id)
            .await
            .map_err(|e| format!("DB error: {e}"))?
            .ok_or("Job not found".to_string())?;

        let file_path = import_job
            .file_path
            .as_ref()
            .ok_or("No file path provided".to_string())?;

        let file_bytes = tokio::fs::read(file_path)
            .await
            .map_err(|e| format!("Could not read file: {e}"))?;

        let rows = crate::utils::file_parser::extract_mapped_rows(&file_bytes, &["employee_code"])
            .map_err(|e| format!("Failed to parse file: {e}"))?;

        let total_rows = rows.len();
        let _ = import_repo
            .update_job_total_rows(payload.import_job_id, total_rows as i32)
            .await;

        if total_rows == 0 {
            let _ = import_repo
                .update_job_progress(payload.import_job_id, 0, ImportJobStatus::Completed, None)
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

        let mut conn = redis.conn();

        // Initialize Redis keys for progress tracking
        let _: () = redis::cmd("SET")
            .arg(format!("import_progress:{}", payload.import_job_id))
            .arg(0)
            .query_async(&mut conn)
            .await
            .unwrap_or(());

        let _: () = redis::cmd("SET")
            .arg(format!("import_chunks_done:{}", payload.import_job_id))
            .arg(0)
            .query_async(&mut conn)
            .await
            .unwrap_or(());

        let _: () = redis::cmd("SET")
            .arg(format!("import_total_chunks:{}", payload.import_job_id))
            .arg(total_chunks)
            .query_async(&mut conn)
            .await
            .unwrap_or(());

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

        let queue = JobQueue::new(redis.clone());
        queue
            .enqueue_jobs(sub_jobs)
            .await
            .map_err(|e| format!("Failed to enqueue chunks: {e}"))?;

        tracing::info!(
            import_job_id = %payload.import_job_id,
            total_chunks = total_chunks,
            "Successfully split employee import job"
        );

        Ok(())
    }
}

pub struct EmployeeImportChunkHandler;

impl EmployeeImportChunkHandler {
    #[allow(clippy::too_many_lines)]
    pub async fn handle(
        diesel: &crate::diesel_pool::DieselPools,
        job: &Job,
        redis: &RedisPool,
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
            payload.import_job_id,
            Arc::clone(&completed),
            diesel.clone(),
        );

        let import_repo = DieselImportJobRepository::new(diesel.clone());
        let import_job = import_repo
            .get_job_by_id(payload.import_job_id)
            .await
            .map_err(|e| format!("DB error: {e}"))?
            .ok_or("Job not found".to_string())?;

        let file_bytes = tokio::fs::read(&payload.file_path)
            .await
            .map_err(|e| format!("Could not read file: {e}"))?;

        let workspace_repo = Arc::new(crate::repositories::DieselWorkspaceRepository::new(
            diesel.clone(),
        ));
        let workspace_tz = match workspace_repo
            .get_workspace_by_id(import_job.workspace_id)
            .await
        {
            Ok(Some(ws)) => ws.timezone.parse::<Tz>().unwrap_or(Tz::UTC),
            _ => Tz::UTC,
        };

        let msgs = match workspace_repo
            .get_workspace_by_id(import_job.workspace_id)
            .await
        {
            Ok(Some(ws)) => AttendanceImportMessages::for_lang(&ws.default_language),
            _ => AttendanceImportMessages::for_lang("vi"),
        };

        let strategy = EmployeeImportStrategy {
            msgs: msgs.clone(),
            employee_repo: Arc::new(crate::repositories::DieselEmployeeRepository::new(
                diesel.clone(),
            )),
            department_repo: Arc::new(
                crate::repositories::diesel_department_repository::DieselDepartmentRepository::new(
                    diesel.clone(),
                ),
            ),
            dept_cache: HashMap::new(),
        };

        let pipeline = ImportPipeline {
            strategy,
            diesel: diesel.clone(),
            import_repo: &import_repo,
            job: &import_job,
            file_bytes,
            workspace_tz,
            start_index: Some(payload.start_index),
            end_index: Some(payload.end_index),
            is_chunk: true,
            redis: Some(redis.clone()),
        };

        let _ = pipeline.run().await; // Returns Ok/Err but we don't fail the whole job here on validation errors

        // Chunk finished
        completed.store(true, Ordering::SeqCst);

        // Increment chunks done
        let mut conn = redis.conn();
        let chunks_done: u32 = redis::cmd("INCR")
            .arg(format!("import_chunks_done:{}", payload.import_job_id))
            .query_async(&mut conn)
            .await
            .unwrap_or(0);

        let total_chunks: u32 = redis::cmd("GET")
            .arg(format!("import_total_chunks:{}", payload.import_job_id))
            .query_async(&mut conn)
            .await
            .unwrap_or(0);

        if chunks_done == total_chunks {
            tracing::info!(import_job_id = %payload.import_job_id, "All chunks completed. Triggering finalize.");
            let queue = JobQueue::new(redis.clone());
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
            let _ = queue.enqueue_jobs(vec![finalize_job]).await;
        }

        Ok(())
    }
}
