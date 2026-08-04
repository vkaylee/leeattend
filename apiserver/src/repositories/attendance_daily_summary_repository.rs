use crate::errors::AppError;
use crate::models::{AttendanceDailySummary, OtStatus};
use async_trait::async_trait;
use chrono::NaiveDate;
use diesel_async::AsyncPgConnection;
use uuid::Uuid;

/// Input for a single day's batch upsert. Used by
/// [`AttendanceDailySummaryRepository::batch_upsert_calculated_summaries`].
pub struct BatchSummaryInput {
    pub date: NaiveDate,
    pub actual_work_minutes: i32,
    pub system_detected_ot_minutes: i32,
    pub standard_minutes: i32,
    pub early_ot_minutes: i32,
    pub late_ot_minutes: i32,
    pub break_ot_minutes: i32,
    pub is_weekend: bool,
    pub ot_status: OtStatus,
    pub is_holiday: bool,
    pub holiday_name: Option<String>,
}

pub struct UpsertCalculatedSummaryArgs {
    pub auth_token: crate::diesel_pool::TenantAuthToken,
    pub employment_id: Uuid,
    pub date: NaiveDate,
    pub actual_work_minutes: i32,
    pub system_detected_ot_minutes: i32,
    pub standard_minutes: i32,
    pub early_ot_minutes: i32,
    pub late_ot_minutes: i32,
    pub break_ot_minutes: i32,
    pub is_weekend: bool,
    pub new_status: OtStatus,
    pub is_holiday: bool,
    pub holiday_name: Option<String>,
}

pub struct UpsertCalculatedSummaryTxArgs {
    pub workspace_id: Uuid,
    pub employment_id: Uuid,
    pub date: NaiveDate,
    pub actual_work_minutes: i32,
    pub system_detected_ot_minutes: i32,
    pub standard_minutes: i32,
    pub early_ot_minutes: i32,
    pub late_ot_minutes: i32,
    pub break_ot_minutes: i32,
    pub is_weekend: bool,
    pub new_status: OtStatus,
    pub is_holiday: bool,
    pub holiday_name: Option<String>,
}

pub struct UpsertPaperOtArgs {
    pub auth_token: crate::diesel_pool::TenantAuthToken,
    pub employment_id: Uuid,
    pub date: NaiveDate,
    pub claimed_minutes: i32,
    pub reason: String,
    pub entered_by_user_id: Uuid,
    pub photo_url: Option<String>,
}

pub struct GetSummariesPaginatedArgs {
    pub auth_token: crate::diesel_pool::TenantAuthToken,
    pub employment_id: Option<Uuid>,
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
    pub ot_status: Option<OtStatus>,
    pub page: i64,
    pub limit: i64,
}

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait AttendanceDailySummaryRepository: Send + Sync {
    async fn get_summary(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        employment_id: Uuid,
        date: NaiveDate,
    ) -> Result<Option<AttendanceDailySummary>, AppError>;

    async fn get_summary_by_id(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        id: Uuid,
    ) -> Result<Option<AttendanceDailySummary>, AppError>;

    async fn upsert_calculated_summary(
        &self,
        args: UpsertCalculatedSummaryArgs,
    ) -> Result<(), AppError>;

    /// Batch upsert multiple daily summaries in a single transaction.
    ///
    /// Semantically equivalent to calling [`upsert_calculated_summary`] for each entry,
    /// but executes within one DB transaction to eliminate per-row connection overhead.
    async fn batch_upsert_calculated_summaries(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        employment_id: Uuid,
        summaries: &[BatchSummaryInput],
    ) -> Result<(), AppError>;

    async fn claim_ot(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        employment_id: Uuid,
        date: NaiveDate,
        claimed_minutes: i32,
        reason: Option<String>,
    ) -> Result<usize, AppError>;

    async fn approve_ot(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        employment_id: Uuid,
        date: NaiveDate,
        approved_minutes: i32,
        manager_note: Option<String>,
        status: OtStatus,
    ) -> Result<usize, AppError>;

    async fn upsert_paper_ot(
        &self,
        args: UpsertPaperOtArgs,
    ) -> Result<AttendanceDailySummary, AppError>;

    async fn get_batch_ot_statuses(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        employment_ids: &[Uuid],
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> Result<std::collections::HashMap<(Uuid, NaiveDate), String>, AppError>;

    async fn get_summaries(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        employment_id: Option<Uuid>,
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> Result<Vec<AttendanceDailySummary>, AppError>;

    /// Fetch summaries for a single employment in a date range, ordered by date.
    async fn get_summaries_for_period(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        employment_id: Uuid,
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> Result<Vec<AttendanceDailySummary>, AppError>;

    /// Batch fetch summaries for multiple employment_ids in a date range.
    /// Returns a map keyed by (employment_id, date).
    async fn get_summaries_by_employments_batch(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        employment_ids: &[Uuid],
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> Result<std::collections::HashMap<(Uuid, NaiveDate), AttendanceDailySummary>, AppError>;

    /// Paginated variant with optional ot_status filter.
    /// Returns (data, total_count).
    async fn get_summaries_paginated(
        &self,
        args: GetSummariesPaginatedArgs,
    ) -> Result<(Vec<AttendanceDailySummary>, usize), AppError>;

    async fn get_distinct_work_days_from_summary(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        employment_id: Uuid,
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> Result<Vec<NaiveDate>, AppError>;

    async fn get_summaries_by_ids(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        ids: &[Uuid],
    ) -> Result<Vec<AttendanceDailySummary>, AppError>;

    /// Transaction-aware variant of [`upsert_calculated_summary`].
    ///
    /// Uses the provided connection instead of acquiring one from the pool,
    /// allowing the caller to coordinate this write with other operations
    /// inside a single database transaction.
    async fn upsert_calculated_summary_tx(
        &self,
        conn: &mut AsyncPgConnection,
        args: UpsertCalculatedSummaryTxArgs,
    ) -> Result<(), AppError>;

    /// Transaction-aware variant of [`batch_upsert_calculated_summaries`].
    async fn batch_upsert_calculated_summaries_tx(
        &self,
        conn: &mut AsyncPgConnection,
        workspace_id: Uuid,
        employment_id: Uuid,
        summaries: &[BatchSummaryInput],
    ) -> Result<(), AppError>;
}
