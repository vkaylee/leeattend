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

/// Repository for holiday calendar CRUD and batch lookups.
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
    ) -> Result<Holiday, sqlx::Error> {
        sqlx::query_as::<_, Holiday>(
            "INSERT INTO holidays
                (workspace_id, name, date, holiday_type, is_paid, leave_type_id, ot_multiplier, description)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
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
                updated_at = NOW()
             WHERE id = $1 AND workspace_id = $9
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
