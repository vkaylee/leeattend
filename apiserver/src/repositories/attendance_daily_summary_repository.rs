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
}

pub async fn get_batch_ot_statuses(
    pool: &sqlx::PgPool,
    workspace_id: Uuid,
    employee_ids: &[Uuid],
    start_date: NaiveDate,
    end_date: NaiveDate,
) -> Result<std::collections::HashMap<(Uuid, NaiveDate), String>, AppError> {
    let rows = sqlx::query!(
        r#"SELECT employee_id, date, ot_status as "ot_status: crate::models::OtStatus"
           FROM attendance_daily_summaries
           WHERE workspace_id = $1
             AND employee_id = ANY($2)
             AND date >= $3 AND date <= $4"#,
        workspace_id,
        employee_ids,
        start_date,
        end_date
    )
    .fetch_all(pool)
    .await
    .map_err(|e| AppError::Internal(format!("db: {e}")))?;

    Ok(rows
        .into_iter()
        .map(|r| {
            (
                (r.employee_id, r.date),
                r.ot_status.to_string().to_lowercase(),
            )
        })
        .collect())
}
