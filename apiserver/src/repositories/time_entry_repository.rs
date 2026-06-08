use crate::errors::AppError;
use crate::models::{TimeEntry, TimeEntryWithEmployee};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use uuid::Uuid;

// ---------------------------------------------------------------------------
// Trait: TimeEntryRepository — DIP abstraction over time entry data access
// ---------------------------------------------------------------------------
#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait TimeEntryRepository: Send + Sync {
    async fn create_time_entry(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        entry: &TimeEntry,
    ) -> Result<(), AppError>;
    async fn get_time_entry_by_id(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        id: Uuid,
    ) -> Result<Option<TimeEntry>, AppError>;
    async fn get_active_time_entry(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        employment_id: Uuid,
    ) -> Result<Option<TimeEntry>, AppError>;
    async fn get_latest_active_entry(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        employment_id: Uuid,
    ) -> Result<Option<TimeEntry>, AppError>;

    async fn get_latest_entry_for_employment(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        employment_id: Uuid,
    ) -> Result<Option<TimeEntry>, AppError>;
    async fn update_time_entry(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        entry: &TimeEntry,
    ) -> Result<(), AppError>;
    async fn delete_time_entry(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        id: Uuid,
    ) -> Result<(), AppError>;
    async fn get_employment_time_entries(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        employment_id: Uuid,
        from: DateTime<Utc>,
        to: DateTime<Utc>,
    ) -> Result<Vec<TimeEntry>, AppError>;
    async fn get_paginated_logs(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        filter: crate::models::TimeEntryLogFilter,
    ) -> Result<Vec<TimeEntryWithEmployee>, AppError>;
    async fn count_logs(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        p_employment_id: Option<Uuid>,
        p_from: Option<DateTime<Utc>>,
        p_to: Option<DateTime<Utc>>,
    ) -> Result<i64, AppError>;
    async fn get_distinct_work_days(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        employment_id: Uuid,
        start_time: DateTime<Utc>,
        end_time: DateTime<Utc>,
        tz: &str,
    ) -> Result<Vec<chrono::NaiveDate>, AppError>;
    async fn get_workspace_time_entries_batch_by_employments(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        employment_ids: &[Uuid],
        from: DateTime<Utc>,
        to: DateTime<Utc>,
    ) -> Result<HashMap<Uuid, Vec<TimeEntry>>, AppError>;
    async fn replace_time_entries(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        employment_id: Uuid,
        start_time: DateTime<Utc>,
        end_time: DateTime<Utc>,
        entries: Vec<TimeEntry>,
    ) -> Result<(), AppError>;
}
