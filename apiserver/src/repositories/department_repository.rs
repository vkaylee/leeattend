use chrono::NaiveDate;
use sqlx::PgPool;
use uuid::Uuid;

use crate::models::{Department, EmployeeDepartmentHistory};

pub async fn create_department(
    pool: &PgPool,
    workspace_id: Uuid,
    name: String,
    description: Option<String>,
    manager_id: Option<Uuid>,
    default_working_pattern_id: Option<Uuid>,
) -> Result<Department, sqlx::Error> {
    sqlx::query_as::<_, Department>(
        r#"INSERT INTO departments (workspace_id, name, description, manager_id, default_working_pattern_id)
           VALUES ($1, $2, $3, $4, $5)
           RETURNING id, workspace_id, name, description, manager_id, default_working_pattern_id, created_at, updated_at"#,
    )
    .bind(workspace_id)
    .bind(name)
    .bind(description)
    .bind(manager_id)
    .bind(default_working_pattern_id)
    .fetch_one(pool)
    .await
}

pub async fn get_department_by_id(
    pool: &PgPool,
    workspace_id: Uuid,
    id: Uuid,
) -> Result<Option<Department>, sqlx::Error> {
    sqlx::query_as::<_, Department>(
        r#"SELECT id, workspace_id, name, description, manager_id, default_working_pattern_id, created_at, updated_at 
           FROM departments WHERE id = $1 AND workspace_id = $2"#,
    )
    .bind(id)
    .bind(workspace_id)
    .fetch_optional(pool)
    .await
}

pub async fn update_department(
    pool: &PgPool,
    workspace_id: Uuid,
    id: Uuid,
    name: Option<String>,
    description: Option<String>,
    manager_id: Option<Uuid>,
    default_working_pattern_id: Option<Option<Uuid>>,
) -> Result<Department, sqlx::Error> {
    if name.is_none()
        && description.is_none()
        && manager_id.is_none()
        && default_working_pattern_id.is_none()
    {
        return get_department_by_id(pool, workspace_id, id)
            .await?
            .ok_or(sqlx::Error::RowNotFound);
    }

    let mut query = String::from("UPDATE departments SET ");
    let mut updates = Vec::new();
    let mut next_bind = 1;

    if name.is_some() {
        updates.push(format!("name = ${}", next_bind));
        next_bind += 1;
    }
    if description.is_some() {
        updates.push(format!("description = ${}", next_bind));
        next_bind += 1;
    }
    if manager_id.is_some() {
        updates.push(format!("manager_id = ${}", next_bind));
        next_bind += 1;
    }
    if default_working_pattern_id.is_some() {
        updates.push(format!("default_working_pattern_id = ${}", next_bind));
        next_bind += 1;
    }

    query.push_str(&updates.join(", "));
    query.push_str(&format!(
        " WHERE id = ${} AND workspace_id = ${}",
        next_bind,
        next_bind + 1
    ));
    query.push_str(r#" RETURNING id, workspace_id, name, description, manager_id, default_working_pattern_id, created_at, updated_at"#);

    let mut q = sqlx::query_as::<_, Department>(&query);

    if let Some(v) = name {
        q = q.bind(v);
    }
    if let Some(v) = description {
        q = q.bind(v);
    }
    if let Some(v) = manager_id {
        q = q.bind(v);
    }
    if let Some(v) = default_working_pattern_id {
        q = q.bind(v); // Some(None) will bind NULL
    }

    q.bind(id).bind(workspace_id).fetch_one(pool).await
}

pub async fn delete_department(
    pool: &PgPool,
    workspace_id: Uuid,
    id: Uuid,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        "DELETE FROM departments WHERE id = $1 AND workspace_id = $2",
        id,
        workspace_id
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn get_departments(
    pool: &PgPool,
    workspace_id: Uuid,
) -> Result<Vec<Department>, sqlx::Error> {
    sqlx::query_as::<_, Department>(
        r#"SELECT id, workspace_id, name, description, manager_id, default_working_pattern_id, created_at, updated_at 
           FROM departments WHERE workspace_id = $1 ORDER BY name"#,
    )
    .bind(workspace_id)
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
    sqlx::query_as::<_, EmployeeDepartmentHistory>(
        r#"INSERT INTO employee_department_history (employee_id, department_id, effective_date, notes)
           VALUES ($1, $2, $3, $4)
           RETURNING id, employee_id, department_id, effective_date, notes, created_at"#,
    )
    .bind(employee_id)
    .bind(department_id)
    .bind(effective_date)
    .bind(notes)
    .fetch_one(pool)
    .await
}

pub async fn get_employee_department_history(
    pool: &PgPool,
    employee_id: Uuid,
) -> Result<Vec<EmployeeDepartmentHistory>, sqlx::Error> {
    sqlx::query_as::<_, EmployeeDepartmentHistory>(
        r#"SELECT id, employee_id, department_id, effective_date, notes, created_at 
           FROM employee_department_history WHERE employee_id = $1 ORDER BY effective_date DESC"#,
    )
    .bind(employee_id)
    .fetch_all(pool)
    .await
}

pub async fn get_current_department(
    pool: &PgPool,
    employee_id: Uuid,
) -> Result<Option<EmployeeDepartmentHistory>, sqlx::Error> {
    sqlx::query_as::<_, EmployeeDepartmentHistory>(
        r#"SELECT id, employee_id, department_id, effective_date, notes, created_at 
           FROM employee_department_history 
           WHERE employee_id = $1 AND effective_date <= CURRENT_DATE 
           ORDER BY effective_date DESC LIMIT 1"#,
    )
    .bind(employee_id)
    .fetch_optional(pool)
    .await
}
