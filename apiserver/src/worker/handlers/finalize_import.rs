//! Finalize Import Job Handler
//!
//! Merges progress and errors from Redis when all chunks are completed.

use sqlx::PgPool;

use crate::models::ImportJobStatus;
use crate::queue::{Job, RedisPool};
use crate::repositories::import_job_repository::ImportJobRepository;
use crate::repositories::DieselImportJobRepository;

pub struct FinalizeImportHandler;

impl FinalizeImportHandler {
    pub async fn handle(
        _db: &PgPool,
        diesel: &crate::diesel_pool::DieselPools,
        job: &Job,
        redis: &RedisPool,
    ) -> Result<(), String> {
        let payload = job
            .finalize_import_payload()
            .ok_or("Missing finalize payload")?;

        tracing::info!(
            import_job_id = %payload.import_job_id,
            "Finalizing import job"
        );

        let import_repo = DieselImportJobRepository::new(diesel.clone());

        let mut conn = redis.conn();

        let progress_key = format!("import_progress:{}", payload.import_job_id);
        let errors_key = format!("import_errors:{}", payload.import_job_id);

        let processed_rows: i32 = redis::cmd("GET")
            .arg(&progress_key)
            .query_async(&mut conn)
            .await
            .unwrap_or(0);

        let errors_map: std::collections::HashMap<String, String> = redis::cmd("HGETALL")
            .arg(&errors_key)
            .query_async(&mut conn)
            .await
            .unwrap_or_default();

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
        let _: () = redis::cmd("DEL")
            .arg(&progress_key)
            .arg(&errors_key)
            .arg(format!("import_chunks_done:{}", payload.import_job_id))
            .arg(format!("import_total_chunks:{}", payload.import_job_id))
            .query_async(&mut conn)
            .await
            .unwrap_or(());

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
