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
