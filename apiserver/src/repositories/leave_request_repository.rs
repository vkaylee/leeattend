use crate::errors::AppError;
use crate::models::{LeaveDurationType, LeaveRequest, LeaveStatus};
use async_trait::async_trait;
#[allow(unused_imports)] // Utc used in query! macro (L67), not visible under SQLX_OFFLINE
use chrono::{NaiveDate, Utc};
use sqlx::{query, query_as, PgPool, Row};
use uuid::Uuid;

// ---------------------------------------------------------------------------
// Trait: LeaveRequestRepository — DIP abstraction
// ---------------------------------------------------------------------------
#[async_trait]
pub trait LeaveRequestRepository: super::PoolBacked + Send + Sync {
    async fn create_leave_request(&self, request: &LeaveRequest) -> Result<(), AppError>;
    async fn get_leave_request_by_id(
        &self,
        workspace_id: Uuid,
        id: Uuid,
    ) -> Result<Option<LeaveRequest>, AppError>;
    async fn update_leave_request_status(
        &self,
        workspace_id: Uuid,
        id: Uuid,
        status: LeaveStatus,
        approved_by: Option<Uuid>,
    ) -> Result<(), AppError>;
    async fn get_employee_leave_requests(
        &self,
        workspace_id: Uuid,
        employee_id: Uuid,
    ) -> Result<Vec<LeaveRequest>, AppError>;
    async fn get_workspace_leave_requests(
        &self,
        workspace_id: Uuid,
        status: Option<LeaveStatus>,
    ) -> Result<Vec<LeaveRequest>, AppError>;
    async fn get_approved_leave_breakdown(
        &self,
        employee_id: Uuid,
        start_date: NaiveDate,
        end_date: NaiveDate,
        workspace_id: Uuid,
    ) -> Result<LeaveBreakdown, AppError>;
    /// Get the employee's current department for a given leave request.
    /// Used for scoped permission checks in approval flows.
    async fn get_employee_department_for_request(
        &self,
        request_id: Uuid,
        workspace_id: Uuid,
    ) -> Result<Option<Uuid>, AppError>;
    /// List all leave types (code, name) for a workspace.
    async fn get_leave_types(&self, workspace_id: Uuid) -> Result<Vec<(String, String)>, AppError>;
}

// ---------------------------------------------------------------------------
// SqlxLeaveRequestRepository — production implementation backed by PgPool
// ---------------------------------------------------------------------------
pub struct SqlxLeaveRequestRepository {
    pub pool: PgPool,
}

impl SqlxLeaveRequestRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl LeaveRequestRepository for SqlxLeaveRequestRepository {
    async fn create_leave_request(&self, request: &LeaveRequest) -> Result<(), AppError> {
        create_leave_request(&self.pool, request)
            .await
            .map_err(Into::into)
    }
    async fn get_leave_request_by_id(
        &self,
        workspace_id: Uuid,
        id: Uuid,
    ) -> Result<Option<LeaveRequest>, AppError> {
        get_leave_request_by_id(&self.pool, workspace_id, id)
            .await
            .map_err(Into::into)
    }
    async fn update_leave_request_status(
        &self,
        workspace_id: Uuid,
        id: Uuid,
        status: LeaveStatus,
        approved_by: Option<Uuid>,
    ) -> Result<(), AppError> {
        update_leave_request_status(&self.pool, workspace_id, id, status, approved_by)
            .await
            .map_err(Into::into)
    }
    async fn get_employee_leave_requests(
        &self,
        workspace_id: Uuid,
        employee_id: Uuid,
    ) -> Result<Vec<LeaveRequest>, AppError> {
        get_employee_leave_requests(&self.pool, workspace_id, employee_id)
            .await
            .map_err(Into::into)
    }
    async fn get_workspace_leave_requests(
        &self,
        workspace_id: Uuid,
        status: Option<LeaveStatus>,
    ) -> Result<Vec<LeaveRequest>, AppError> {
        get_workspace_leave_requests(&self.pool, workspace_id, status)
            .await
            .map_err(Into::into)
    }
    async fn get_approved_leave_breakdown(
        &self,
        employee_id: Uuid,
        start_date: NaiveDate,
        end_date: NaiveDate,
        workspace_id: Uuid,
    ) -> Result<LeaveBreakdown, AppError> {
        get_approved_leave_breakdown(&self.pool, employee_id, start_date, end_date, workspace_id)
            .await
            .map_err(Into::into)
    }
    async fn get_employee_department_for_request(
        &self,
        request_id: Uuid,
        workspace_id: Uuid,
    ) -> Result<Option<Uuid>, AppError> {
        sqlx::query_scalar(
            r#"
            SELECT edh.department_id
            FROM leave_requests lr
            JOIN employee_department_history edh ON lr.employee_id = edh.employee_id
            JOIN employees e ON lr.employee_id = e.id
            WHERE lr.id = $1 AND e.workspace_id = $2
            ORDER BY edh.effective_date DESC
            LIMIT 1
            "#,
        )
        .bind(request_id)
        .bind(workspace_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(Into::into)
    }
    async fn get_leave_types(&self, workspace_id: Uuid) -> Result<Vec<(String, String)>, AppError> {
        sqlx::query("SELECT code, name FROM leave_types WHERE workspace_id = $1 ORDER BY code ASC")
            .bind(workspace_id)
            .fetch_all(&self.pool)
            .await
            .map_err(Into::into)
            .map(|rows| rows.iter().map(|r| (r.get(0), r.get(1))).collect())
    }
}

pub async fn create_leave_request(
    pool: &PgPool,
    request: &LeaveRequest,
) -> Result<(), sqlx::Error> {
    query!(
        r#"
        INSERT INTO leave_requests
            (id, employee_id, leave_type_id, start_date, end_date, reason, status,
             duration_type, start_time, end_time,
             requested_by_user_id, approved_by_user_id, created_at, updated_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)
        "#,
        request.id,
        request.employee_id,
        request.leave_type_id,
        request.start_date,
        request.end_date,
        request.reason,
        request.status as LeaveStatus,
        request.duration_type as LeaveDurationType,
        request.start_time,
        request.end_time,
        request.requested_by_user_id,
        request.approved_by_user_id,
        request.created_at,
        request.updated_at,
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn get_leave_request_by_id(
    pool: &PgPool,
    workspace_id: Uuid,
    id: Uuid,
) -> Result<Option<LeaveRequest>, sqlx::Error> {
    query_as!(
        LeaveRequest,
        r#"
        SELECT lr.id, lr.employee_id, lr.leave_type_id, lr.start_date, lr.end_date, lr.reason,
               lr.status as "status: LeaveStatus",
               lr.duration_type as "duration_type: LeaveDurationType",
               lr.start_time, lr.end_time,
               lr.requested_by_user_id, lr.approved_by_user_id,
               lr.created_at as "created_at!", lr.updated_at as "updated_at!"
        FROM leave_requests lr
        JOIN employees e ON lr.employee_id = e.id
        WHERE lr.id = $1 AND e.workspace_id = $2
        "#,
        id,
        workspace_id
    )
    .fetch_optional(pool)
    .await
}

pub async fn update_leave_request_status(
    pool: &PgPool,
    workspace_id: Uuid,
    id: Uuid,
    status: LeaveStatus,
    approved_by: Option<Uuid>,
) -> Result<(), sqlx::Error> {
    query!(
        "UPDATE leave_requests SET status = $1, approved_by_user_id = $2, updated_at = $3 WHERE id = $4 AND employee_id IN (SELECT id FROM employees WHERE workspace_id = $5)",
        status as LeaveStatus,
        approved_by,
        Utc::now(),
        id,
        workspace_id
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn get_employee_leave_requests(
    pool: &PgPool,
    workspace_id: Uuid,
    employee_id: Uuid,
) -> Result<Vec<LeaveRequest>, sqlx::Error> {
    query_as!(
        LeaveRequest,
        r#"
        SELECT lr.id, lr.employee_id, lr.leave_type_id, lr.start_date, lr.end_date, lr.reason,
               lr.status as "status: LeaveStatus",
               lr.duration_type as "duration_type: LeaveDurationType",
               lr.start_time, lr.end_time,
               lr.requested_by_user_id, lr.approved_by_user_id,
               lr.created_at as "created_at!", lr.updated_at as "updated_at!"
        FROM leave_requests lr
        JOIN employees e ON lr.employee_id = e.id
        WHERE lr.employee_id = $1 AND e.workspace_id = $2
        ORDER BY lr.start_date DESC
        "#,
        employee_id,
        workspace_id
    )
    .fetch_all(pool)
    .await
}

pub async fn get_workspace_leave_requests(
    pool: &PgPool,
    workspace_id: Uuid,
    status: Option<LeaveStatus>,
) -> Result<Vec<LeaveRequest>, sqlx::Error> {
    let mut query_builder: sqlx::QueryBuilder<sqlx::Postgres> = sqlx::QueryBuilder::new(
        r#"SELECT lr.id, lr.employee_id, lr.leave_type_id, lr.start_date, lr.end_date,
           lr.reason, lr.status,
           lr.duration_type, lr.start_time, lr.end_time,
           lr.requested_by_user_id, lr.approved_by_user_id,
           lr.created_at, lr.updated_at
           FROM leave_requests lr
           JOIN employees e ON lr.employee_id = e.id
           WHERE e.workspace_id = "#,
    );
    query_builder.push_bind(workspace_id);

    if let Some(s) = status {
        query_builder.push(" AND lr.status = ");
        query_builder.push_bind(s);
    }

    query_builder.push(" ORDER BY lr.created_at DESC");

    query_builder
        .build_query_as::<LeaveRequest>()
        .fetch_all(pool)
        .await
}

pub struct LeaveBreakdown {
    pub total_days: f64,
    pub unpaid_days: f64,
    pub sick_days: f64,
}

pub async fn get_approved_leave_breakdown(
    pool: &PgPool,
    employee_id: Uuid,
    start_date: NaiveDate,
    end_date: NaiveDate,
    workspace_id: Uuid,
) -> Result<LeaveBreakdown, sqlx::Error> {
    // Step 1: Get raw leave days per type
    let rows = sqlx::query(
        r#"
        SELECT lt.code as leave_type_code, 
               SUM(LEAST(lr.end_date, $3) - GREATEST(lr.start_date, $2) + 1) as days
        FROM leave_requests lr
        JOIN leave_types lt ON lr.leave_type_id = lt.id
        JOIN employees e ON lr.employee_id = e.id
        WHERE lr.employee_id = $1
          AND e.workspace_id = $4
          AND lr.status IN ('approved', 'pre_approved')
          AND lr.start_date <= $3
          AND lr.end_date >= $2
        GROUP BY lt.code
        "#,
    )
    .bind(employee_id)
    .bind(start_date)
    .bind(end_date)
    .bind(workspace_id)
    .fetch_all(pool)
    .await?;

    // Step 2: Count paid holidays in the period to subtract from total
    let paid_holiday_count: i64 = sqlx::query_scalar(
        r#"SELECT COUNT(*) FROM holidays h
           WHERE h.workspace_id = $1
             AND h.is_paid = true
             AND h.date >= $2
             AND h.date <= $3
             AND EXISTS (
                 SELECT 1 FROM leave_requests lr
                 WHERE lr.employee_id = $4
                   AND lr.status IN ('approved', 'pre_approved')
                   AND lr.start_date <= h.date
                   AND lr.end_date >= h.date
             )"#,
    )
    .bind(workspace_id)
    .bind(start_date)
    .bind(end_date)
    .bind(employee_id)
    .fetch_one(pool)
    .await
    .unwrap_or(0);

    let mut breakdown = LeaveBreakdown {
        total_days: 0.0,
        unpaid_days: 0.0,
        sick_days: 0.0,
    };

    for row in rows {
        let days: i64 = row.get("days");
        let days_f = days as f64;
        breakdown.total_days += days_f;
        let leave_type_code: String = row.get("leave_type_code");
        if leave_type_code == "unpaid" || leave_type_code == "personal" {
            breakdown.unpaid_days += days_f;
        } else if leave_type_code == "sick" {
            breakdown.sick_days += days_f;
        }
    }

    // Subtract paid holidays that overlap with leave periods
    breakdown.total_days -= paid_holiday_count as f64;
    if breakdown.total_days < 0.0 {
        breakdown.total_days = 0.0;
    }

    Ok(breakdown)
}

impl super::PoolBacked for SqlxLeaveRequestRepository {
    fn from_pool(pool: sqlx::PgPool) -> Self {
        Self::new(pool)
    }
    fn pool(&self) -> &sqlx::PgPool {
        &self.pool
    }
}
