use chrono::NaiveDate;
use sqlx::{query_as, PgPool};
use uuid::Uuid;

use crate::models::{Department, EmployeeDepartmentHistory};

pub async fn create_department(
    pool: &PgPool,
    workspace_id: Uuid,
    name: String,
    description: Option<String>,
    manager_id: Option<Uuid>,
) -> Result<Department, sqlx::Error> {
    query_as!(
        Department,
        r#"INSERT INTO departments (workspace_id, name, description, manager_id)
           VALUES ($1, $2, $3, $4)
           RETURNING id, workspace_id, name, description, manager_id, created_at as "created_at!", updated_at as "updated_at!""#,
        workspace_id,
        name,
        description,
        manager_id
    )
    .fetch_one(pool)
    .await
}

pub async fn get_departments(
    pool: &PgPool,
    workspace_id: Uuid,
) -> Result<Vec<Department>, sqlx::Error> {
    query_as!(
        Department,
        r#"SELECT id, workspace_id, name, description, manager_id, created_at as "created_at!", updated_at as "updated_at!" 
           FROM departments WHERE workspace_id = $1 ORDER BY name"#,
        workspace_id
    )
    .fetch_all(pool)
    .await
}

pub async fn transfer_employee(
    pool: &PgPool,
    employee_id: Uuid,
    department_id: Uuid,
    effective_date: NaiveDate,
    notes: Option<String>,
) -> Result<EmployeeDepartmentHistory, sqlx::Error> {
    query_as!(
        EmployeeDepartmentHistory,
        r#"INSERT INTO employee_department_history (employee_id, department_id, effective_date, notes)
           VALUES ($1, $2, $3, $4)
           RETURNING id, employee_id, department_id, effective_date, notes, created_at as "created_at!""#,
        employee_id,
        department_id,
        effective_date,
        notes
    )
    .fetch_one(pool)
    .await
}

pub async fn get_employee_department_history(
    pool: &PgPool,
    employee_id: Uuid,
) -> Result<Vec<EmployeeDepartmentHistory>, sqlx::Error> {
    query_as!(
        EmployeeDepartmentHistory,
        r#"SELECT id, employee_id, department_id, effective_date, notes, created_at as "created_at!" 
           FROM employee_department_history WHERE employee_id = $1 ORDER BY effective_date DESC"#,
        employee_id
    )
    .fetch_all(pool)
    .await
}

pub async fn get_current_department(
    pool: &PgPool,
    employee_id: Uuid,
) -> Result<Option<EmployeeDepartmentHistory>, sqlx::Error> {
    query_as!(
        EmployeeDepartmentHistory,
        r#"SELECT id, employee_id, department_id, effective_date, notes, created_at as "created_at!" 
           FROM employee_department_history 
           WHERE employee_id = $1 AND effective_date <= CURRENT_DATE 
           ORDER BY effective_date DESC LIMIT 1"#,
        employee_id
    )
    .fetch_optional(pool)
    .await
}
