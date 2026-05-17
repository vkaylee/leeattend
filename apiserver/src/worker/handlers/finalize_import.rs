//! Finalize Import Job Handler
//!
//! Merges progress and errors from Redis when all chunks are completed.

use crate::models::ImportJobStatus;
use crate::queue::Job;

pub struct FinalizeImportHandler;

impl FinalizeImportHandler {
    pub async fn handle(
        job: &Job,
        cache: &std::sync::Arc<dyn crate::repositories::CacheRepository>,
        deps: &crate::worker::deps::WorkerDeps,
    ) -> Result<(), String> {
        let payload = job
            .finalize_import_payload()
            .ok_or("Missing finalize payload")?;

        tracing::info!(
            import_job_id = %payload.import_job_id,
            "Finalizing import job"
        );

        let import_repo = &*deps.import_job_repo;

        let progress_key = format!("import_progress:{}", payload.import_job_id);
        let errors_key = format!("import_errors:{}", payload.import_job_id);

        let processed_rows: i32 = cache
            .get_json(&progress_key)
            .await
            .unwrap_or_default()
            .unwrap_or_else(|| "0".to_string())
            .parse()
            .unwrap_or(0);

        let errors_map = cache.hgetall(&errors_key).await.unwrap_or_default();

        let mut combined_errors = Vec::new();
        // Extract values from HashMap (ignoring the chunk field keys)
        for val in errors_map.values() {
            combined_errors.push(val.clone());
        }

        let final_log = if combined_errors.is_empty() {
            None
        } else {
            Some(combined_errors.join("\n\n"))
        };

        let status = if final_log.is_none() || processed_rows > 0 {
            ImportJobStatus::Completed
        } else {
            ImportJobStatus::Failed
        };

        let _ = import_repo
            .update_job_progress(payload.import_job_id, processed_rows, status, final_log)
            .await;

        // Cleanup Redis keys
        let _ = cache.delete(&progress_key).await;
        let _ = cache.delete(&errors_key).await;
        let _ = cache.delete(&format!("import_chunks_done:{}", payload.import_job_id)).await;
        let _ = cache.delete(&format!("import_total_chunks:{}", payload.import_job_id)).await;

        // Cleanup file
        if let Ok(Some(import_job)) = import_repo.get_job_by_id(payload.import_job_id).await {
            if let Some(file_path) = &import_job.file_path {
                let _ = tokio::fs::remove_file(file_path).await;
            }
        }

        tracing::info!(
            import_job_id = %payload.import_job_id,
            processed_rows = processed_rows,
            "Import job successfully finalized"
        );

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::queue::{FinalizeImportPayload, JobPayload, JobPriority, TaskType};
    use crate::worker::test_utils::MockWorkerDepsBuilder;
    use std::collections::HashMap;
    use std::sync::Arc;
    use uuid::Uuid;

    #[tokio::test]
    async fn test_finalize_import_handler_success() {
        let import_job_id = Uuid::new_v4();
        let ws_id = Uuid::new_v4();
        
        let mut cache = crate::repositories::cache_repository::MockCacheRepository::new();
        let progress_key = format!("import_progress:{}", import_job_id);
        let errors_key = format!("import_errors:{}", import_job_id);

        cache.expect_get_json()
            .with(mockall::predicate::eq(progress_key.clone()))
            .returning(|_| Ok(Some("5".to_string())));

        cache.expect_hgetall()
            .with(mockall::predicate::eq(errors_key.clone()))
            .returning(|_| Ok(HashMap::new()));

        cache.expect_delete()
            .returning(|_| Ok(()));

        let cache_arc: Arc<dyn crate::repositories::CacheRepository> = Arc::new(cache);

        let mut import_repo = crate::repositories::import_job_repository::MockImportJobRepository::new();
        import_repo.expect_update_job_progress()
            .withf(|_, rows, status, err| *rows == 5 && *status == ImportJobStatus::Completed && err.is_none())
            .returning(|_, _, _, _| Ok(()));
            
        import_repo.expect_get_job_by_id()
            .returning(|_| Ok(None)); // Skip file cleanup

        let mut builder = MockWorkerDepsBuilder::new();
        builder.import_job_repo = import_repo;
        let deps = builder.build();

        let job = Job {
            job_id: Uuid::new_v4(),
            parent_job_id: Uuid::new_v4(),
            batch_index: 0,
            task_type: TaskType::FinalizeImport,
            workspace_id: ws_id,
            payload: JobPayload::FinalizeImport(FinalizeImportPayload {
                import_job_id,
            }),
            priority: JobPriority::Normal,
            created_at: chrono::Utc::now(),
            retry_count: 0,
            max_retries: 3,
        };

        let result = FinalizeImportHandler::handle(&job, &cache_arc, &deps).await;
        assert!(result.is_ok(), "Handler failed: {:?}", result.err());
    }
}
