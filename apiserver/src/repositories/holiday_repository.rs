use async_trait::async_trait;
use chrono::NaiveDate;
use sqlx::PgPool;
use std::collections::HashMap;
use uuid::Uuid;

use crate::models::Holiday;

/// Result of processing holiday leave deductions for employees.
#[derive(Debug, Clone, serde::Serialize)]
pub enum HolidayLeaveResult {
    /// Holiday is paid — no deduction needed.
    Paid,
    /// Leave was auto-deducted from employee balance.
    LeaveDeducted {
        employee_id: Uuid,
        leave_request_id: Uuid,
        leave_type_name: String,
    },
    /// Employee has no remaining leave — unpaid day.
    Unpaid {
        employee_id: Uuid,
    },
}

// ---------------------------------------------------------------------------
// Trait: HolidayRepositoryTrait — DIP abstraction
// ---------------------------------------------------------------------------
#[async_trait]
#[allow(clippy::too_many_arguments)]
pub trait HolidayRepositoryTrait: super::PoolBacked + Send + Sync {
    async fn list(&self, workspace_id: Uuid, year: Option<i32>) -> Result<Vec<Holiday>, sqlx::Error>;
    async fn get_by_id(&self, workspace_id: Uuid, id: Uuid) -> Result<Option<Holiday>, sqlx::Error>;
    async fn create_holiday(&self, workspace_id: Uuid, name: &str, date: NaiveDate, holiday_type: &str, is_paid: bool, leave_type_id: Option<Uuid>, ot_multiplier: Option<rust_decimal::Decimal>, description: Option<&str>, work_threshold_percent: Option<i32>, is_credit_on_work: bool) -> Result<Holiday, sqlx::Error>;
    async fn update_holiday(&self, workspace_id: Uuid, id: Uuid, name: Option<&str>, date: Option<NaiveDate>, holiday_type: Option<&str>, is_paid: Option<bool>, leave_type_id: Option<Uuid>, ot_multiplier: Option<rust_decimal::Decimal>, description: Option<&str>, work_threshold_percent: Option<i32>, is_credit_on_work: Option<bool>) -> Result<Option<Holiday>, sqlx::Error>;
    async fn delete(&self, workspace_id: Uuid, id: Uuid) -> Result<bool, sqlx::Error>;
    async fn get_holidays_in_range(&self, workspace_id: Uuid, start_date: NaiveDate, end_date: NaiveDate) -> Result<HashMap<NaiveDate, Holiday>, sqlx::Error>;
    async fn is_holiday(&self, workspace_id: Uuid, date: NaiveDate) -> Result<Option<Holiday>, sqlx::Error>;
}

// ---------------------------------------------------------------------------
// SqlxHolidayRepository — production implementation backed by PgPool
// ---------------------------------------------------------------------------
pub struct SqlxHolidayRepository {
    pub pool: PgPool,
}

impl SqlxHolidayRepository {
    pub fn new(pool: PgPool) -> Self { Self { pool } }
}

#[async_trait]
impl HolidayRepositoryTrait for SqlxHolidayRepository {
    async fn list(&self, workspace_id: Uuid, year: Option<i32>) -> Result<Vec<Holiday>, sqlx::Error> {
        HolidayRepository::list(&self.pool, workspace_id, year).await
    }
    async fn get_by_id(&self, workspace_id: Uuid, id: Uuid) -> Result<Option<Holiday>, sqlx::Error> {
        HolidayRepository::get_by_id(&self.pool, workspace_id, id).await
    }
    #[allow(clippy::too_many_arguments)]
    async fn create_holiday(&self, workspace_id: Uuid, name: &str, date: NaiveDate, holiday_type: &str, is_paid: bool, leave_type_id: Option<Uuid>, ot_multiplier: Option<rust_decimal::Decimal>, description: Option<&str>, work_threshold_percent: Option<i32>, is_credit_on_work: bool) -> Result<Holiday, sqlx::Error> {
        HolidayRepository::create(&self.pool, workspace_id, name, date, holiday_type, is_paid, leave_type_id, ot_multiplier, description, work_threshold_percent, is_credit_on_work).await
    }
    #[allow(clippy::too_many_arguments)]
    async fn update_holiday(&self, workspace_id: Uuid, id: Uuid, name: Option<&str>, date: Option<NaiveDate>, holiday_type: Option<&str>, is_paid: Option<bool>, leave_type_id: Option<Uuid>, ot_multiplier: Option<rust_decimal::Decimal>, description: Option<&str>, work_threshold_percent: Option<i32>, is_credit_on_work: Option<bool>) -> Result<Option<Holiday>, sqlx::Error> {
        HolidayRepository::update(&self.pool, workspace_id, id, name, date, holiday_type, is_paid, leave_type_id, ot_multiplier, description, work_threshold_percent, is_credit_on_work).await
    }
    async fn delete(&self, workspace_id: Uuid, id: Uuid) -> Result<bool, sqlx::Error> {
        HolidayRepository::delete(&self.pool, workspace_id, id).await
    }
    async fn get_holidays_in_range(&self, workspace_id: Uuid, start_date: NaiveDate, end_date: NaiveDate) -> Result<HashMap<NaiveDate, Holiday>, sqlx::Error> {
        HolidayRepository::get_holidays_in_range(&self.pool, workspace_id, start_date, end_date).await
    }
    async fn is_holiday(&self, workspace_id: Uuid, date: NaiveDate) -> Result<Option<Holiday>, sqlx::Error> {
        HolidayRepository::is_holiday(&self.pool, workspace_id, date).await
    }
}

/// Repository for holiday calendar CRUD and batch lookups.
/// Kept for backwards compatibility — callers use HolidayRepository::method(pool, ...).
pub struct HolidayRepository;

impl HolidayRepository {

    /// Lists all holidays for a workspace, optionally filtered by year.
    pub async fn list(
        pool: &PgPool,
        workspace_id: Uuid,
        year: Option<i32>,
    ) -> Result<Vec<Holiday>, sqlx::Error> {
        if let Some(y) = year {
            sqlx::query_as::<_, Holiday>(
                "SELECT * FROM holidays
                 WHERE workspace_id = $1
                   AND EXTRACT(YEAR FROM date) = $2
                 ORDER BY date ASC",
            )
            .bind(workspace_id)
            .bind(y)
            .fetch_all(pool)
            .await
        } else {
            sqlx::query_as::<_, Holiday>(
                "SELECT * FROM holidays
                 WHERE workspace_id = $1
                 ORDER BY date ASC",
            )
            .bind(workspace_id)
            .fetch_all(pool)
            .await
        }
    }

    /// Gets a single holiday by ID.
    pub async fn get_by_id(
        pool: &PgPool,
        workspace_id: Uuid,
        id: Uuid,
    ) -> Result<Option<Holiday>, sqlx::Error> {
        sqlx::query_as::<_, Holiday>(
            "SELECT * FROM holidays WHERE id = $1 AND workspace_id = $2",
        )
        .bind(id)
        .bind(workspace_id)
        .fetch_optional(pool)
        .await
    }

    /// Creates a new holiday. Returns error if (workspace_id, date) already exists.
    #[allow(clippy::too_many_arguments)]
    pub async fn create(
        pool: &PgPool,
        workspace_id: Uuid,
        name: &str,
        date: NaiveDate,
        holiday_type: &str,
        is_paid: bool,
        leave_type_id: Option<Uuid>,
        ot_multiplier: Option<rust_decimal::Decimal>,
        description: Option<&str>,
        work_threshold_percent: Option<i32>,
        is_credit_on_work: bool,
    ) -> Result<Holiday, sqlx::Error> {
        sqlx::query_as::<_, Holiday>(
            "INSERT INTO holidays
                (workspace_id, name, date, holiday_type, is_paid, leave_type_id, ot_multiplier, description, work_threshold_percent, is_credit_on_work)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
             RETURNING *",
        )
        .bind(workspace_id)
        .bind(name)
        .bind(date)
        .bind(holiday_type)
        .bind(is_paid)
        .bind(leave_type_id)
        .bind(ot_multiplier)
        .bind(description)
        .bind(work_threshold_percent)
        .bind(is_credit_on_work)
        .fetch_one(pool)
        .await
    }

    /// Updates an existing holiday by ID.
    #[allow(clippy::too_many_arguments)]
    pub async fn update(
        pool: &PgPool,
        workspace_id: Uuid,
        id: Uuid,
        name: Option<&str>,
        date: Option<NaiveDate>,
        holiday_type: Option<&str>,
        is_paid: Option<bool>,
        leave_type_id: Option<Uuid>,
        ot_multiplier: Option<rust_decimal::Decimal>,
        description: Option<&str>,
        work_threshold_percent: Option<i32>,
        is_credit_on_work: Option<bool>,
    ) -> Result<Option<Holiday>, sqlx::Error> {
        sqlx::query_as::<_, Holiday>(
            "UPDATE holidays SET
                name = COALESCE($2, name),
                date = COALESCE($3, date),
                holiday_type = COALESCE($4, holiday_type),
                is_paid = COALESCE($5, is_paid),
                leave_type_id = COALESCE($6, leave_type_id),
                ot_multiplier = COALESCE($7, ot_multiplier),
                description = COALESCE($8, description),
                work_threshold_percent = COALESCE($9, work_threshold_percent),
                is_credit_on_work = COALESCE($10, is_credit_on_work),
                updated_at = NOW()
             WHERE id = $1 AND workspace_id = $11
             RETURNING *",
        )
        .bind(id)
        .bind(name)
        .bind(date)
        .bind(holiday_type)
        .bind(is_paid)
        .bind(leave_type_id)
        .bind(ot_multiplier)
        .bind(description)
        .bind(work_threshold_percent)
        .bind(is_credit_on_work)
        .bind(workspace_id)
        .fetch_optional(pool)
        .await
    }

    /// Deletes a holiday by ID. Returns true if a row was deleted.
    pub async fn delete(
        pool: &PgPool,
        workspace_id: Uuid,
        id: Uuid,
    ) -> Result<bool, sqlx::Error> {
        let result = sqlx::query("DELETE FROM holidays WHERE id = $1 AND workspace_id = $2")
            .bind(id)
            .bind(workspace_id)
            .execute(pool)
            .await?;
        Ok(result.rows_affected() > 0)
    }

    /// Batch lookup: fetches all holidays in a date range for a workspace.
    /// Returns a HashMap keyed by date for O(1) per-day lookups.
    pub async fn get_holidays_in_range(
        pool: &PgPool,
        workspace_id: Uuid,
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> Result<HashMap<NaiveDate, Holiday>, sqlx::Error> {
        let holidays = sqlx::query_as::<_, Holiday>(
            "SELECT * FROM holidays
             WHERE workspace_id = $1
               AND date >= $2
               AND date <= $3
             ORDER BY date ASC",
        )
        .bind(workspace_id)
        .bind(start_date)
        .bind(end_date)
        .fetch_all(pool)
        .await?;

        Ok(holidays.into_iter().map(|h| (h.date, h)).collect())
    }

    /// Single date check: returns the holiday if the given date is a holiday.
    pub async fn is_holiday(
        pool: &PgPool,
        workspace_id: Uuid,
        date: NaiveDate,
    ) -> Result<Option<Holiday>, sqlx::Error> {
        sqlx::query_as::<_, Holiday>(
            "SELECT * FROM holidays
             WHERE workspace_id = $1 AND date = $2",
        )
        .bind(workspace_id)
        .bind(date)
        .fetch_optional(pool)
        .await
    }
}

impl super::PoolBacked for SqlxHolidayRepository {
    fn from_pool(pool: sqlx::PgPool) -> Self { Self { pool } }
    fn pool(&self) -> &sqlx::PgPool { &self.pool }
}
