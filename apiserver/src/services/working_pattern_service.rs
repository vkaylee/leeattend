use crate::models::{WorkingPattern, WorkingPatternDetail, EmployeeWorkingPattern, CreateWorkingPatternRequest, AssignPatternRequest};
use crate::repositories::working_pattern_repository as repo;
use sqlx::PgPool;
use uuid::Uuid;

pub async fn create_working_pattern(
    pool: &PgPool,
    workspace_id: Uuid,
    req: CreateWorkingPatternRequest,
) -> Result<WorkingPattern, sqlx::Error> {
    let mut tx = pool.begin().await?;

    // 1. Create Pattern Header
    let pattern = repo::create_pattern(&mut *tx, workspace_id, &req.name, req.description.as_deref()).await?;

    // 2. Add Details
    for detail in req.details {
        repo::add_pattern_detail(&mut *tx, pattern.id, detail.day_of_week, detail.shift_id).await?;
    }

    tx.commit().await?;
    Ok(pattern)
}

pub async fn list_patterns(pool: &PgPool, workspace_id: Uuid) -> Result<Vec<WorkingPattern>, sqlx::Error> {
    repo::get_patterns(pool, workspace_id).await
}

pub async fn get_pattern_details(pool: &PgPool, pattern_id: Uuid) -> Result<Vec<WorkingPatternDetail>, sqlx::Error> {
    repo::get_pattern_details(pool, pattern_id).await
}

pub async fn assign_pattern(
    pool: &PgPool,
    workspace_id: Uuid,
    employee_id: Uuid,
    req: AssignPatternRequest,
) -> Result<EmployeeWorkingPattern, sqlx::Error> {
    repo::assign_pattern(
        pool,
        workspace_id,
        employee_id,
        req.pattern_id,
        req.start_date,
        req.end_date,
    ).await
}

pub async fn get_employee_pattern_history(
    pool: &PgPool,
    employee_id: Uuid,
) -> Result<Vec<EmployeeWorkingPattern>, sqlx::Error> {
    repo::get_employee_pattern_history(pool, employee_id).await
}
