use crate::errors::AppError;
use crate::models::{AttendanceDailySummary, OtStatus};
use crate::repositories::attendance_daily_summary_repository::AttendanceDailySummaryRepository;
use async_trait::async_trait;
use chrono::NaiveDate;
use sqlx::PgPool;
use uuid::Uuid;

pub struct DieselAttendanceDailySummaryRepository {
    pub sqlx_pool: PgPool,
    pub diesel_pools: crate::diesel_pool::DieselPools,
}

impl DieselAttendanceDailySummaryRepository {
    pub fn new(sqlx_pool: PgPool, diesel_pools: crate::diesel_pool::DieselPools) -> Self {
        Self {
            sqlx_pool,
            diesel_pools,
        }
    }
}

#[async_trait]
impl AttendanceDailySummaryRepository for DieselAttendanceDailySummaryRepository {
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
        .fetch_optional(&self.sqlx_pool)
        .await
        .map_err(|e| AppError::Internal(format!("db: {e}")))
    }

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
        .execute(&self.sqlx_pool)
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
            WHERE employee_id = $3 AND workspace_id = $4 AND date = $5 AND ot_status IN ('pending_employee', 'rejected')
            "#,
            claimed_minutes,
            reason,
            employee_id,
            workspace_id,
            date
        )
        .execute(&self.sqlx_pool)
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
            WHERE employee_id = $4 AND workspace_id = $5 AND date = $6 AND ot_status = 'pending_manager'
            "#,
            approved_minutes,
            manager_note,
            status as OtStatus,
            employee_id,
            workspace_id,
            date
        )
        .execute(&self.sqlx_pool)
        .await
        .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(result.rows_affected() as usize)
    }

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
    ) -> Result<AttendanceDailySummary, AppError> {
        let row = sqlx::query_as!(
            AttendanceDailySummary,
            r#"
            INSERT INTO attendance_daily_summaries (
                workspace_id, employee_id, date, 
                claimed_ot_minutes, ot_reason, 
                ot_status, entered_via, claim_entered_by_user_id, paper_form_photo_url, claim_entered_at,
                actual_work_minutes, system_detected_ot_minutes
            )
            VALUES ($1, $2, $3, $4, $5, 'pending_manager', 'paper_form', $6, $7, CURRENT_TIMESTAMP, 0, 0)
            ON CONFLICT (employee_id, date) DO UPDATE SET
                claimed_ot_minutes = EXCLUDED.claimed_ot_minutes,
                ot_reason = EXCLUDED.ot_reason,
                ot_status = 'pending_manager',
                entered_via = 'paper_form',
                claim_entered_by_user_id = EXCLUDED.claim_entered_by_user_id,
                paper_form_photo_url = EXCLUDED.paper_form_photo_url,
                claim_entered_at = CURRENT_TIMESTAMP,
                updated_at = CURRENT_TIMESTAMP
            RETURNING id, workspace_id, employee_id, date, actual_work_minutes, system_detected_ot_minutes, claimed_ot_minutes, approved_ot_minutes, ot_status as "ot_status: OtStatus", ot_reason, manager_note, claim_entered_by_user_id, claim_entered_at, paper_form_photo_url, entered_via, is_holiday, holiday_name, created_at, updated_at
            "#,
            workspace_id,
            employee_id,
            date,
            claimed_minutes,
            reason,
            entered_by_user_id,
            photo_url
        )
        .fetch_one(&self.sqlx_pool)
        .await
        .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(row)
    }
}
