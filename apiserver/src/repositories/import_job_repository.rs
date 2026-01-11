use sqlx::PgPool;
use uuid::Uuid;
use chrono::Utc;
use crate::models::AttendanceImportJob;

pub async fn create_job(
    pool: &PgPool,
    job: &AttendanceImportJob,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO attendance_import_jobs (id, workspace_id, user_id, status, total_rows, processed_rows, file_path, error_log, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)"
    )
    .bind(job.id)
    .bind(job.workspace_id)
    .bind(job.user_id)
    .bind(&job.status)
    .bind(job.total_rows)
    .bind(job.processed_rows)
    .bind(&job.file_path)
    .bind(&job.error_log)
    .bind(job.created_at)
    .bind(job.updated_at)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn update_job_progress(
    pool: &PgPool,
    id: Uuid,
    processed_rows: i32,
    status: &str,
    error_log: Option<String>,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "UPDATE attendance_import_jobs SET processed_rows = $1, status = $2, error_log = $3, updated_at = $4 WHERE id = $5"
    )
    .bind(processed_rows)
    .bind(status)
    .bind(error_log)
    .bind(Utc::now())
    .bind(id)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn get_job_by_id(
    pool: &PgPool,
    id: Uuid,
) -> Result<Option<AttendanceImportJob>, sqlx::Error> {
    sqlx::query_as::<_, AttendanceImportJob>(
        "SELECT id, workspace_id, user_id, status, total_rows, processed_rows, file_path, error_log, created_at, updated_at FROM attendance_import_jobs WHERE id = $1"
    )
    .bind(id)
    .fetch_optional(pool)
    .await
}

pub async fn get_latest_job(
    pool: &PgPool,
    workspace_id: Uuid,
) -> Result<Option<AttendanceImportJob>, sqlx::Error> {
    sqlx::query_as::<_, AttendanceImportJob>(
        "SELECT id, workspace_id, user_id, status, total_rows, processed_rows, file_path, error_log, created_at, updated_at FROM attendance_import_jobs WHERE workspace_id = $1 ORDER BY created_at DESC LIMIT 1"
    )
    .bind(workspace_id)
    .fetch_optional(pool)
    .await
}

pub async fn get_pending_jobs(
    pool: &PgPool,
) -> Result<Vec<AttendanceImportJob>, sqlx::Error> {
    sqlx::query_as::<_, AttendanceImportJob>(
        "SELECT id, workspace_id, user_id, status, total_rows, processed_rows, file_path, error_log, created_at, updated_at FROM attendance_import_jobs WHERE status = 'processing'"
    )
    .fetch_all(pool)
    .await
}
