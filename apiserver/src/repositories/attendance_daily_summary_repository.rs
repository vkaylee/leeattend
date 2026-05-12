use crate::errors::AppError;
use crate::models::{AttendanceDailySummary, OtStatus};
use async_trait::async_trait;
use chrono::NaiveDate;
use uuid::Uuid;

#[async_trait]
pub trait AttendanceDailySummaryRepository: super::PoolBacked + Send + Sync {
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

pub struct SqlxAttendanceDailySummaryRepository {
    pub pool: sqlx::PgPool,
}

impl SqlxAttendanceDailySummaryRepository {
    pub fn new(pool: sqlx::PgPool) -> Self {
        Self { pool }
    }
}

impl super::PoolBacked for SqlxAttendanceDailySummaryRepository {
    fn from_pool(pool: sqlx::PgPool) -> Self {
        Self::new(pool)
    }

    fn pool(&self) -> &sqlx::PgPool {
        &self.pool
    }
}

#[async_trait]
impl AttendanceDailySummaryRepository for SqlxAttendanceDailySummaryRepository {
    async fn get_summary(
        &self,
        workspace_id: Uuid,
        employee_id: Uuid,
        date: NaiveDate,
    ) -> Result<Option<AttendanceDailySummary>, AppError> {
        sqlx::query_as!(
            AttendanceDailySummary,
            "SELECT id, workspace_id, employee_id, date, actual_work_minutes, system_detected_ot_minutes, claimed_ot_minutes, approved_ot_minutes, ot_status as \"ot_status: OtStatus\", ot_reason, manager_note, claim_entered_by_user_id, claim_entered_at, paper_form_photo_url, entered_via, is_holiday, holiday_name, created_at, updated_at FROM attendance_daily_summaries WHERE employee_id = $1 AND workspace_id = $2 AND date = $3",
            employee_id, workspace_id, date
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::Internal(format!("db: {e}")))
    }

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
    ) -> Result<(), AppError> {
        sqlx::query!(
            r#"
            INSERT INTO attendance_daily_summaries (
                workspace_id, employee_id, date, 
                actual_work_minutes, system_detected_ot_minutes, 
                ot_status, is_holiday, holiday_name
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            ON CONFLICT (employee_id, date) DO UPDATE SET
                actual_work_minutes = EXCLUDED.actual_work_minutes,
                system_detected_ot_minutes = EXCLUDED.system_detected_ot_minutes,
                is_holiday = EXCLUDED.is_holiday,
                holiday_name = EXCLUDED.holiday_name,
                ot_status = CASE 
                    WHEN attendance_daily_summaries.ot_status = 'none'::ot_status_type AND EXCLUDED.system_detected_ot_minutes > 0 THEN 'pending_employee'::ot_status_type
                    ELSE attendance_daily_summaries.ot_status 
                END,
                updated_at = CURRENT_TIMESTAMP
            "#,
            workspace_id,
            employee_id,
            date,
            actual_work_minutes,
            system_detected_ot_minutes,
            new_status as OtStatus,
            is_holiday,
            holiday_name
        )
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(())
    }

    async fn claim_ot(
        &self,
        workspace_id: Uuid,
        employee_id: Uuid,
        date: NaiveDate,
        claimed_minutes: i32,
        reason: Option<String>,
    ) -> Result<usize, AppError> {
        let result = sqlx::query!(
            r#"
            UPDATE attendance_daily_summaries
            SET 
                claimed_ot_minutes = $1,
                ot_reason = $2,
                ot_status = 'pending_manager',
                updated_at = CURRENT_TIMESTAMP
            WHERE employee_id = $3 AND workspace_id = $4 AND date = $5
            "#,
            claimed_minutes,
            reason,
            employee_id,
            workspace_id,
            date
        )
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(result.rows_affected() as usize)
    }

    async fn approve_ot(
        &self,
        workspace_id: Uuid,
        employee_id: Uuid,
        date: NaiveDate,
        approved_minutes: i32,
        manager_note: Option<String>,
        status: OtStatus,
    ) -> Result<usize, AppError> {
        let result = sqlx::query!(
            r#"
            UPDATE attendance_daily_summaries
            SET
                approved_ot_minutes = $1,
                manager_note = $2,
                ot_status = $3,
                updated_at = CURRENT_TIMESTAMP
            WHERE employee_id = $4 AND workspace_id = $5 AND date = $6
            "#,
            approved_minutes,
            manager_note,
            status as OtStatus,
            employee_id,
            workspace_id,
            date
        )
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(result.rows_affected() as usize)
    }

    async fn upsert_paper_ot(
        &self,
        workspace_id: Uuid,
        employee_id: Uuid,
        date: NaiveDate,
        claimed_minutes: i32,
        reason: String,
        entered_by_user_id: Uuid,
        photo_url: Option<String>,
    ) -> Result<AttendanceDailySummary, AppError> {
        let now = chrono::Utc::now();
        let result = sqlx::query_as!(
            AttendanceDailySummary,
            r#"
            INSERT INTO attendance_daily_summaries (
                workspace_id, employee_id, date,
                claimed_ot_minutes, ot_reason, ot_status,
                claim_entered_by_user_id, claim_entered_at,
                paper_form_photo_url, entered_via
            )
            SELECT
                e.workspace_id, $1, $2,
                $3, $4, 'pending_manager',
                $5, $6, $7, 'paper_form'
            FROM employees e WHERE e.id = $1 AND e.workspace_id = $8
            ON CONFLICT (employee_id, date) DO UPDATE SET
                claimed_ot_minutes = EXCLUDED.claimed_ot_minutes,
                ot_reason = EXCLUDED.ot_reason,
                ot_status = 'pending_manager',
                claim_entered_by_user_id = EXCLUDED.claim_entered_by_user_id,
                claim_entered_at = EXCLUDED.claim_entered_at,
                paper_form_photo_url = EXCLUDED.paper_form_photo_url,
                entered_via = 'paper_form',
                updated_at = CURRENT_TIMESTAMP
            RETURNING id, workspace_id, employee_id, date, actual_work_minutes, system_detected_ot_minutes, claimed_ot_minutes, approved_ot_minutes, ot_status as "ot_status: OtStatus", ot_reason, manager_note, claim_entered_by_user_id, claim_entered_at, paper_form_photo_url, entered_via, is_holiday, holiday_name, created_at, updated_at
            "#,
            employee_id, date, claimed_minutes, reason,
            entered_by_user_id, now, photo_url, workspace_id
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(result)
    }
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
