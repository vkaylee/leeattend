use crate::errors::AppError;
use crate::models::{AttendanceDailySummary, OtStatus};
use async_trait::async_trait;
use chrono::NaiveDate;
use uuid::Uuid;

#[async_trait]
pub trait AttendanceDailySummaryRepository: Send + Sync {
    async fn get_summary(
        &self,
        workspace_id: Uuid,
        employee_id: Uuid,
        date: NaiveDate,
    ) -> Result<Option<AttendanceDailySummary>, AppError>;

    #[allow(clippy::too_many_arguments)]
    async fn upsert_calculated_summary(
        &self,
        workspace_id: Uuid,
        employee_id: Uuid,
        date: NaiveDate,
        actual_work_minutes: i32,
        system_detected_ot_minutes: i32,
        new_status: OtStatus,
        is_holiday: bool,
        holiday_name: Option<String>,
    ) -> Result<(), AppError>;

    async fn claim_ot(
        &self,
        workspace_id: Uuid,
        employee_id: Uuid,
        date: NaiveDate,
        claimed_minutes: i32,
        reason: Option<String>,
    ) -> Result<usize, AppError>;

    async fn approve_ot(
        &self,
        workspace_id: Uuid,
        employee_id: Uuid,
        date: NaiveDate,
        approved_minutes: i32,
        manager_note: Option<String>,
        status: OtStatus,
    ) -> Result<usize, AppError>;

    #[allow(clippy::too_many_arguments)]
    async fn upsert_paper_ot(
        &self,
        workspace_id: Uuid,
        employee_id: Uuid,
        date: NaiveDate,
        claimed_minutes: i32,
        reason: String,
        entered_by_user_id: Uuid,
        photo_url: Option<String>,
    ) -> Result<AttendanceDailySummary, AppError>;

    async fn get_batch_ot_statuses(
        &self,
        workspace_id: Uuid,
        employee_ids: &[Uuid],
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> Result<std::collections::HashMap<(Uuid, NaiveDate), String>, AppError>;

    async fn get_summaries(
        &self,
        workspace_id: Uuid,
        employee_id: Option<Uuid>,
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> Result<Vec<AttendanceDailySummary>, AppError>;
}
