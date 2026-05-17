use crate::errors::AppError;
use crate::models::{RawAttendanceLog, RawAttendanceLogWithEmployee};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use uuid::Uuid;

// ---------------------------------------------------------------------------
// Trait: RawAttendanceLogRepository — DIP abstraction
// ---------------------------------------------------------------------------
#[async_trait]
#[cfg_attr(test, mockall::automock)]
pub trait RawAttendanceLogRepository: Send + Sync {
    async fn create_raw_log(&self, workspace_id: Uuid, log: &RawAttendanceLog) -> Result<bool, AppError>;
    async fn get_raw_log_by_id(
        &self,
        workspace_id: Uuid,
        id: Uuid,
    ) -> Result<Option<RawAttendanceLog>, AppError>;
    async fn get_latest_log(
        &self,
        workspace_id: Uuid,
        employee_id: Uuid,
    ) -> Result<Option<RawAttendanceLog>, AppError>;
    async fn get_paginated_raw_logs(
        &self,
        workspace_id: Uuid,
        p_employee_id: Option<Uuid>,
        p_from: Option<DateTime<Utc>>,
        p_to: Option<DateTime<Utc>>,
        p_limit: i64,
        p_offset: i64,
    ) -> Result<Vec<RawAttendanceLogWithEmployee>, AppError>;
    async fn count_raw_logs(
        &self,
        workspace_id: Uuid,
        p_employee_id: Option<Uuid>,
        p_from: Option<DateTime<Utc>>,
        p_to: Option<DateTime<Utc>>,
    ) -> Result<i64, AppError>;
    async fn create_raw_logs_batch(&self, workspace_id: Uuid, logs: &[&RawAttendanceLog]) -> Result<usize, AppError>;
    async fn has_duplicate_raw_log(
        &self,
        workspace_id: Uuid,
        employee_id: Uuid,
        log_id: Uuid,
        punch_time: DateTime<Utc>,
    ) -> Result<bool, AppError>;
    async fn get_employee_logs_in_time_range(
        &self,
        workspace_id: Uuid,
        employee_id: Uuid,
        start_time: DateTime<Utc>,
        end_time: DateTime<Utc>,
    ) -> Result<Vec<RawAttendanceLog>, AppError>;
}
