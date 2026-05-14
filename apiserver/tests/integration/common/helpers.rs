//! Diesel-backed helper shims for integration tests.
//!
//! These functions replace the old SQLx-based free functions that were removed
//! during the Diesel migration. They maintain the same call signatures so that
//! existing tests require minimal changes.

#![allow(dead_code)]

use apiserver::diesel_pool::DieselPools;
use apiserver::errors::AppError;
use apiserver::models::TimeEntry;
use apiserver::models::{RawAttendanceLog, RawAttendanceLogWithEmployee};
use apiserver::repositories::{
    raw_attendance_log_repository::RawAttendanceLogRepository,
    staging_log_repository::StagingLogRepository, time_entry_repository::TimeEntryRepository,
};
use chrono::{DateTime, Utc};
use uuid::Uuid;

// ── Staging Log Helpers ───────────────────────────────────────────────────────

pub async fn create_staging_log(
    diesel_pools: &DieselPools,
    workspace_id: Uuid,
    device_id: Option<String>,
    raw_user_id: &str,
    punch_time: DateTime<Utc>,
    raw_data: Option<serde_json::Value>,
    fingerprint: &str,
) -> Result<bool, AppError> {
    let repo = apiserver::repositories::DieselStagingLogRepository::new(diesel_pools.clone());
    repo.create_staging_log(
        workspace_id,
        device_id,
        raw_user_id,
        punch_time,
        raw_data,
        fingerprint,
    )
    .await
}

pub async fn count_pending_staging_logs(
    diesel_pools: &DieselPools,
    workspace_id: Uuid,
) -> Result<i64, AppError> {
    let repo = apiserver::repositories::DieselStagingLogRepository::new(diesel_pools.clone());
    repo.count_pending_staging_logs(workspace_id).await
}

// ── Raw Attendance Log Helpers ────────────────────────────────────────────────

pub async fn create_raw_log(
    diesel_pools: &DieselPools,
    log: &RawAttendanceLog,
) -> Result<bool, AppError> {
    let repo = apiserver::repositories::DieselRawAttendanceLogRepository::new(diesel_pools.clone());
    repo.create_raw_log(log).await
}

pub async fn get_paginated_raw_logs(
    diesel_pools: &DieselPools,
    workspace_id: Uuid,
    p_employee_id: Option<Uuid>,
    p_from: Option<DateTime<Utc>>,
    p_to: Option<DateTime<Utc>>,
    p_limit: i64,
    p_offset: i64,
) -> Result<Vec<RawAttendanceLogWithEmployee>, AppError> {
    let repo = apiserver::repositories::DieselRawAttendanceLogRepository::new(diesel_pools.clone());
    repo.get_paginated_raw_logs(workspace_id, p_employee_id, p_from, p_to, p_limit, p_offset)
        .await
}

// ── Time Entry Helpers ────────────────────────────────────────────────────────

pub async fn create_time_entry(
    diesel_pools: &DieselPools,
    entry: &TimeEntry,
) -> Result<(), AppError> {
    let repo = apiserver::repositories::DieselTimeEntryRepository::new(diesel_pools.clone());
    repo.create_time_entry(entry).await
}

pub async fn get_employee_time_entries(
    diesel_pools: &DieselPools,
    workspace_id: Uuid,
    employee_id: Uuid,
    from: DateTime<Utc>,
    to: DateTime<Utc>,
) -> Result<Vec<apiserver::models::TimeEntry>, AppError> {
    let repo = apiserver::repositories::DieselTimeEntryRepository::new(diesel_pools.clone());
    repo.get_employee_time_entries(workspace_id, employee_id, from, to)
        .await
}
