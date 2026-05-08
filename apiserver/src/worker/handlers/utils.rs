use sqlx::PgPool;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use crate::models::ImportJobStatus;
use crate::repositories::import_job_repository::{ImportJobRepository, SqlxImportJobRepository};

/// Guard to prevent import jobs from getting stuck in "Processing" state
/// when the job runner terminates the task due to timeout.
pub struct ImportTimeoutGuard {
    db: PgPool,
    job_id: uuid::Uuid,
    completed: Arc<AtomicBool>,
}

impl ImportTimeoutGuard {
    pub fn new(db: PgPool, job_id: uuid::Uuid, completed: Arc<AtomicBool>) -> Self {
        Self {
            db,
            job_id,
            completed,
        }
    }
}

impl Drop for ImportTimeoutGuard {
    fn drop(&mut self) {
        if !self.completed.load(Ordering::SeqCst) {
            let db = self.db.clone();
            let job_id = self.job_id;
            tokio::spawn(async move {
                let repo = SqlxImportJobRepository::new(db);
                let current_rows = if let Ok(Some(job)) = repo.get_job_by_id(job_id).await {
                    job.processed_rows
                } else {
                    0
                };
                let _ = repo
                    .update_job_progress(
                        job_id,
                        current_rows,
                        ImportJobStatus::Failed,
                        Some(
                            "System timeout: Job took too long and was terminated by the worker."
                                .to_string(),
                        ),
                    )
                    .await;
            });
        }
    }
}
