use sqlx::PgPool;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use crate::models::ImportJobStatus;
use crate::repositories::import_job_repository::ImportJobRepository;
use crate::repositories::DieselImportJobRepository;

/// Guard to prevent import jobs from getting stuck in "Processing" state
/// when the job runner terminates the task due to timeout.
pub struct ImportTimeoutGuard {
    db: PgPool,
    job_id: uuid::Uuid,
    completed: Arc<AtomicBool>,
    diesel_pools: crate::diesel_pool::DieselPools,
}

impl ImportTimeoutGuard {
    pub fn new(
        db: PgPool,
        job_id: uuid::Uuid,
        completed: Arc<AtomicBool>,
        diesel_pools: crate::diesel_pool::DieselPools,
    ) -> Self {
        Self {
            db,
            job_id,
            completed,
            diesel_pools,
        }
    }
}

impl Drop for ImportTimeoutGuard {
    fn drop(&mut self) {
        if !self.completed.load(Ordering::SeqCst) {
            let db = self.db.clone();
            let job_id = self.job_id;
            let diesel_pools = self.diesel_pools.clone();
            tokio::spawn(async move {
                let repo = DieselImportJobRepository::new(db.clone(), diesel_pools.clone());
                let current_rows = if let Ok(Some(job)) = repo.get_job_by_id(job_id).await {
                    job.processed_rows
                } else {
                    0
                };
                if let Err(e) = repo
                    .update_job_progress(
                        job_id,
                        current_rows,
                        ImportJobStatus::Failed,
                        Some(
                            "System timeout: Job took too long and was terminated by the worker."
                                .to_string(),
                        ),
                    )
                    .await
                {
                    tracing::error!("Timeout Update Error: {}", e);
                }
            });
        }
    }
}

/// Get the chunk size for import jobs from the environment, defaulting to 500.
pub fn get_import_chunk_size() -> usize {
    std::env::var("IMPORT_CHUNK_SIZE")
        .unwrap_or_else(|_| "500".to_string())
        .parse::<usize>()
        .unwrap_or(500)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;
    use std::env;

    #[test]
    #[serial]
    fn test_get_import_chunk_size_default() {
        env::remove_var("IMPORT_CHUNK_SIZE");
        assert_eq!(get_import_chunk_size(), 500);
    }

    #[test]
    #[serial]
    fn test_get_import_chunk_size_custom() {
        env::set_var("IMPORT_CHUNK_SIZE", "1000");
        assert_eq!(get_import_chunk_size(), 1000);
        env::remove_var("IMPORT_CHUNK_SIZE");
    }

    #[test]
    #[serial]
    fn test_get_import_chunk_size_invalid() {
        env::set_var("IMPORT_CHUNK_SIZE", "invalid");
        assert_eq!(get_import_chunk_size(), 500);
        env::remove_var("IMPORT_CHUNK_SIZE");
    }
}
