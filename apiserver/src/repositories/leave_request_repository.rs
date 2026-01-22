use crate::models::{LeaveRequest, LeaveStatus};
use chrono::{Utc, NaiveDate};
use sqlx::{query, query_as, PgPool, Row};
use uuid::Uuid;

pub async fn create_leave_request(
    pool: &PgPool,
    request: &LeaveRequest,
) -> Result<(), sqlx::Error> {
    query!(
        "INSERT INTO leave_requests (id, employee_id, leave_type, start_date, end_date, reason, status, requested_by_user_id, approved_by_user_id, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)",
        request.id,
        request.employee_id,
        request.leave_type,
        request.start_date,
        request.end_date,
        request.reason,
        request.status as LeaveStatus,
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
    id: Uuid,
) -> Result<Option<LeaveRequest>, sqlx::Error> {
    query_as!(
        LeaveRequest,
        "SELECT id, employee_id, leave_type, start_date, end_date, reason, status as \"status: LeaveStatus\", requested_by_user_id, approved_by_user_id, created_at as \"created_at!\", updated_at as \"updated_at!\" FROM leave_requests WHERE id = $1",
        id
    )
    .fetch_optional(pool)
    .await
}

pub async fn update_leave_request_status(
    pool: &PgPool,
    id: Uuid,
    status: LeaveStatus,
    approved_by: Option<Uuid>,
) -> Result<(), sqlx::Error> {
    query!(
        "UPDATE leave_requests SET status = $1, approved_by_user_id = $2, updated_at = $3 WHERE id = $4",
        status as LeaveStatus,
        approved_by,
        Utc::now(),
        id
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn get_employee_leave_requests(
    pool: &PgPool,
    employee_id: Uuid,
) -> Result<Vec<LeaveRequest>, sqlx::Error> {
    query_as!(
        LeaveRequest,
        "SELECT id, employee_id, leave_type, start_date, end_date, reason, status as \"status: LeaveStatus\", requested_by_user_id, approved_by_user_id, created_at as \"created_at!\", updated_at as \"updated_at!\" FROM leave_requests WHERE employee_id = $1 ORDER BY start_date DESC",
        employee_id
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
        "SELECT lr.id, lr.employee_id, lr.leave_type, lr.start_date, lr.end_date, lr.reason, lr.status, lr.requested_by_user_id, lr.approved_by_user_id, lr.created_at, lr.updated_at FROM leave_requests lr JOIN employees e ON lr.employee_id = e.id WHERE e.workspace_id = "
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

) -> Result<LeaveBreakdown, sqlx::Error> {

    let rows = sqlx::query(

        r#"SELECT leave_type, SUM(end_date - start_date + 1) as days

           FROM leave_requests

           WHERE employee_id = 
 

             AND status = 'approved'

             AND start_date <= $3

             AND end_date >= $2

           GROUP BY leave_type"#,

    )

    .bind(employee_id)

    .bind(start_date)

    .bind(end_date)

    .fetch_all(pool)

    .await?;



    let mut breakdown = LeaveBreakdown { total_days: 0.0, unpaid_days: 0.0, sick_days: 0.0 };

    for row in rows {

        let days: i64 = row.get("days");

        let days_f = days as f64;

        breakdown.total_days += days_f;

        let leave_type: String = row.get("leave_type");

        if leave_type == "unpaid" || leave_type == "personal" {

            breakdown.unpaid_days += days_f;

        } else if leave_type == "sick" {

            breakdown.sick_days += days_f;

        }

    }



    Ok(breakdown)

}
