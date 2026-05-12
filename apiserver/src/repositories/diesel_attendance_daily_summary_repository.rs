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

impl crate::repositories::PoolBacked for DieselAttendanceDailySummaryRepository {
    #[allow(clippy::panic)]
    fn from_pool(_pool: PgPool) -> Self {
        panic!("DieselAttendanceDailySummaryRepository requires diesel_pools to be manually injected via new()");
    }

    fn pool(&self) -> &PgPool {
        &self.sqlx_pool
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
        // Fallback to SQLx implementation temporarily to speed up service layer cleanup
        let repo = crate::repositories::attendance_daily_summary_repository::SqlxAttendanceDailySummaryRepository::new(
            self.sqlx_pool.clone(),
        );
        repo.get_summary(workspace_id, employee_id, date).await
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
        let repo = crate::repositories::attendance_daily_summary_repository::SqlxAttendanceDailySummaryRepository::new(
            self.sqlx_pool.clone(),
        );
        repo.upsert_calculated_summary(
            workspace_id,
            employee_id,
            date,
            actual_work_minutes,
            system_detected_ot_minutes,
            new_status,
            is_holiday,
            holiday_name,
        )
        .await
    }

    async fn claim_ot(
        &self,
        workspace_id: Uuid,
        employee_id: Uuid,
        date: NaiveDate,
        claimed_minutes: i32,
        reason: Option<String>,
    ) -> Result<usize, AppError> {
        let repo = crate::repositories::attendance_daily_summary_repository::SqlxAttendanceDailySummaryRepository::new(
            self.sqlx_pool.clone(),
        );
        repo.claim_ot(workspace_id, employee_id, date, claimed_minutes, reason)
            .await
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
        let repo = crate::repositories::attendance_daily_summary_repository::SqlxAttendanceDailySummaryRepository::new(
            self.sqlx_pool.clone(),
        );
        repo.approve_ot(
            workspace_id,
            employee_id,
            date,
            approved_minutes,
            manager_note,
            status,
        )
        .await
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
        let repo = crate::repositories::attendance_daily_summary_repository::SqlxAttendanceDailySummaryRepository::new(
            self.sqlx_pool.clone(),
        );
        repo.upsert_paper_ot(
            workspace_id,
            employee_id,
            date,
            claimed_minutes,
            reason,
            entered_by_user_id,
            photo_url,
        )
        .await
    }
}
