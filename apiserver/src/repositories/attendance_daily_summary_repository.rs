use crate::errors::AppError;
use crate::models::{AttendanceDailySummary, OtStatus};
use async_trait::async_trait;
use chrono::NaiveDate;
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

    #[allow(clippy::too_many_arguments)]
    async fn upsert_calculated_summary(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        employment_id: Uuid,
        date: NaiveDate,
        actual_work_minutes: i32,
        system_detected_ot_minutes: i32,
        standard_minutes: i32,
        early_ot_minutes: i32,
        late_ot_minutes: i32,
        break_ot_minutes: i32,
        is_weekend: bool,
        new_status: OtStatus,
        is_holiday: bool,
        holiday_name: Option<String>,
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

    #[allow(clippy::too_many_arguments)]
    async fn upsert_paper_ot(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        employment_id: Uuid,
        date: NaiveDate,
        claimed_minutes: i32,
        reason: String,
        entered_by_user_id: Uuid,
        photo_url: Option<String>,
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
    #[allow(clippy::too_many_arguments)]
    async fn get_summaries_paginated(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        employment_id: Option<Uuid>,
        start_date: NaiveDate,
        end_date: NaiveDate,
        ot_status: Option<OtStatus>,
        page: i64,
        limit: i64,
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
}
