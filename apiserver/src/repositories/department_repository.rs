use async_trait::async_trait;
use chrono::NaiveDate;
use rust_decimal::Decimal;
use sqlx::PgPool;
use uuid::Uuid;

use crate::errors::AppError;
use crate::models::{Department, DepartmentPayrollConfig, EmployeeDepartmentHistory};

// ---------------------------------------------------------------------------
// Trait: DepartmentRepository — DIP abstraction
// ---------------------------------------------------------------------------
#[async_trait]
#[async_trait]
pub trait DepartmentRepository: super::PoolBacked + Send + Sync {
    async fn create_department(
        &self,
        workspace_id: Uuid,
        name: String,
        description: Option<String>,
        manager_id: Option<Uuid>,
        default_working_pattern_id: Option<Uuid>,
    ) -> Result<Department, AppError>;
    async fn get_department_by_id(
        &self,
        workspace_id: Uuid,
        id: Uuid,
    ) -> Result<Option<Department>, AppError>;
    async fn get_department_by_name(
        &self,
        workspace_id: Uuid,
        name: &str,
    ) -> Result<Option<Uuid>, AppError>;
    async fn update_department(
        &self,
        workspace_id: Uuid,
        id: Uuid,
        req: crate::models::UpdateDepartmentRequest,
    ) -> Result<Department, AppError>;
    async fn delete_department(&self, workspace_id: Uuid, id: Uuid) -> Result<(), AppError>;
    async fn get_departments(&self, workspace_id: Uuid) -> Result<Vec<Department>, AppError>;
    async fn transfer_employee(
        &self,
        employee_id: Uuid,
        department_id: Uuid,
        effective_date: NaiveDate,
        notes: Option<String>,
    ) -> Result<EmployeeDepartmentHistory, AppError>;
    async fn get_employee_department_history(
        &self,
        workspace_id: Uuid,
        employee_id: Uuid,
    ) -> Result<Vec<EmployeeDepartmentHistory>, AppError>;
    async fn get_current_department(
        &self,
        workspace_id: Uuid,
        employee_id: Uuid,
    ) -> Result<Option<EmployeeDepartmentHistory>, AppError>;
    async fn upsert_department_payroll_config(
        &self,
        department_id: Uuid,
        std_working_days: Decimal,
        hours_per_day: Decimal,
        effective_date: NaiveDate,
    ) -> Result<DepartmentPayrollConfig, AppError>;
    async fn get_department_payroll_config_at(
        &self,
        department_id: Uuid,
        date: NaiveDate,
    ) -> Result<Option<DepartmentPayrollConfig>, AppError>;
}

// ---------------------------------------------------------------------------
// SqlxDepartmentRepository — production implementation backed by PgPool
// ---------------------------------------------------------------------------
pub struct SqlxDepartmentRepository {
    pub pool: PgPool,
}

impl SqlxDepartmentRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl DepartmentRepository for SqlxDepartmentRepository {
    async fn create_department(
        &self,
        workspace_id: Uuid,
        name: String,
        description: Option<String>,
        manager_id: Option<Uuid>,
        default_working_pattern_id: Option<Uuid>,
    ) -> Result<Department, AppError> {
        create_department(
            &self.pool,
            workspace_id,
            name,
            description,
            manager_id,
            default_working_pattern_id,
        )
        .await
        .map_err(Into::into)
    }
    async fn get_department_by_id(
        &self,
        workspace_id: Uuid,
        id: Uuid,
    ) -> Result<Option<Department>, AppError> {
        get_department_by_id(&self.pool, workspace_id, id)
            .await
            .map_err(Into::into)
    }
    async fn get_department_by_name(
        &self,
        workspace_id: Uuid,
        name: &str,
    ) -> Result<Option<Uuid>, AppError> {
        let name_lower = name.to_lowercase();
        let record = sqlx::query!(
            "SELECT id FROM departments WHERE workspace_id = $1 AND LOWER(name) = $2 LIMIT 1",
            workspace_id,
            name_lower
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::Internal(format!("db: {e}")))?;
        Ok(record.map(|r| r.id))
    }
    async fn update_department(
        &self,
        workspace_id: Uuid,
        id: Uuid,
        req: crate::models::UpdateDepartmentRequest,
    ) -> Result<Department, AppError> {
        update_department(&self.pool, workspace_id, id, req)
            .await
            .map_err(Into::into)
    }
    async fn delete_department(&self, workspace_id: Uuid, id: Uuid) -> Result<(), AppError> {
        delete_department(&self.pool, workspace_id, id)
            .await
            .map_err(Into::into)
    }
    async fn get_departments(&self, workspace_id: Uuid) -> Result<Vec<Department>, AppError> {
        get_departments(&self.pool, workspace_id)
            .await
            .map_err(Into::into)
    }
    async fn transfer_employee(
        &self,
        employee_id: Uuid,
        department_id: Uuid,
        effective_date: NaiveDate,
        notes: Option<String>,
    ) -> Result<EmployeeDepartmentHistory, AppError> {
        transfer_employee(
            &self.pool,
            employee_id,
            department_id,
            effective_date,
            notes,
        )
        .await
        .map_err(Into::into)
    }
    async fn get_employee_department_history(
        &self,
        workspace_id: Uuid,
        employee_id: Uuid,
    ) -> Result<Vec<EmployeeDepartmentHistory>, AppError> {
        get_employee_department_history(&self.pool, workspace_id, employee_id)
            .await
            .map_err(Into::into)
    }
    async fn get_current_department(
        &self,
        workspace_id: Uuid,
        employee_id: Uuid,
    ) -> Result<Option<EmployeeDepartmentHistory>, AppError> {
        get_current_department(&self.pool, workspace_id, employee_id)
            .await
            .map_err(Into::into)
    }
    async fn upsert_department_payroll_config(
        &self,
        department_id: Uuid,
        std_working_days: Decimal,
        hours_per_day: Decimal,
        effective_date: NaiveDate,
    ) -> Result<DepartmentPayrollConfig, AppError> {
        upsert_department_payroll_config(
            &self.pool,
            department_id,
            std_working_days,
            hours_per_day,
            effective_date,
        )
        .await
        .map_err(Into::into)
    }
    async fn get_department_payroll_config_at(
        &self,
        department_id: Uuid,
        date: NaiveDate,
    ) -> Result<Option<DepartmentPayrollConfig>, AppError> {
        get_department_payroll_config_at(&self.pool, department_id, date)
            .await
            .map_err(Into::into)
    }
}

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
        r#"SELECT d.id, d.workspace_id, d.name, d.description, d.manager_id, d.default_working_pattern_id, d.created_at, d.updated_at,
           COALESCE((SELECT COUNT(DISTINCT e.id) FROM employees e
             JOIN (SELECT DISTINCT ON (employee_id) employee_id, department_id FROM employee_department_history ORDER BY employee_id, effective_date DESC) edh ON e.id = edh.employee_id
             WHERE edh.department_id = d.id AND e.is_active = true), 0) as active_employee_count
           FROM departments d WHERE d.id = $1 AND d.workspace_id = $2"#,
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
    req: crate::models::UpdateDepartmentRequest,
) -> Result<Department, sqlx::Error> {
    if req.name.is_none()
        && req.description.is_none()
        && req.manager_id.is_none()
        && req.default_working_pattern_id.is_none()
    {
        return get_department_by_id(pool, workspace_id, id)
            .await?
            .ok_or(sqlx::Error::RowNotFound);
    }

    let mut query = String::from("UPDATE departments SET ");
    let mut updates = Vec::new();
    let mut next_bind = 1;

    if req.name.is_some() {
        updates.push(format!("name = ${next_bind}"));
        next_bind += 1;
    }
    if req.description.is_some() {
        updates.push(format!("description = ${next_bind}"));
        next_bind += 1;
    }
    if req.manager_id.is_some() {
        updates.push(format!("manager_id = ${next_bind}"));
        next_bind += 1;
    }
    if req.default_working_pattern_id.is_some() {
        updates.push(format!("default_working_pattern_id = ${next_bind}"));
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

    if let Some(v) = req.name {
        q = q.bind(v);
    }
    if let Some(v) = req.description {
        q = q.bind(v);
    }
    if let Some(v) = req.manager_id {
        q = q.bind(v);
    }
    if let Some(v) = req.default_working_pattern_id {
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
        r#"SELECT d.id, d.workspace_id, d.name, d.description, d.manager_id, d.default_working_pattern_id, d.created_at, d.updated_at,
           COALESCE((SELECT COUNT(DISTINCT e.id) FROM employees e
             JOIN (SELECT DISTINCT ON (employee_id) employee_id, department_id FROM employee_department_history ORDER BY employee_id, effective_date DESC) edh ON e.id = edh.employee_id
             WHERE edh.department_id = d.id AND e.is_active = true), 0) as active_employee_count
           FROM departments d WHERE d.workspace_id = $1 ORDER BY d.name"#,
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
    workspace_id: Uuid,
    employee_id: Uuid,
) -> Result<Vec<EmployeeDepartmentHistory>, sqlx::Error> {
    sqlx::query_as::<_, EmployeeDepartmentHistory>(
        r#"SELECT edh.id, edh.employee_id, edh.department_id, edh.effective_date, edh.notes, edh.created_at 
           FROM employee_department_history edh
           JOIN employees e ON e.id = edh.employee_id
           WHERE edh.employee_id = $1 AND e.workspace_id = $2
           ORDER BY edh.effective_date DESC"#,
    )
    .bind(employee_id)
    .bind(workspace_id)
    .fetch_all(pool)
    .await
}

pub async fn get_current_department(
    pool: &PgPool,
    workspace_id: Uuid,
    employee_id: Uuid,
) -> Result<Option<EmployeeDepartmentHistory>, sqlx::Error> {
    sqlx::query_as::<_, EmployeeDepartmentHistory>(
        r#"SELECT edh.id, edh.employee_id, edh.department_id, edh.effective_date, edh.notes, edh.created_at 
           FROM employee_department_history edh
           JOIN employees e ON e.id = edh.employee_id
           WHERE edh.employee_id = $1 AND e.workspace_id = $2 AND edh.effective_date <= CURRENT_DATE 
           ORDER BY edh.effective_date DESC LIMIT 1"#,
    )
    .bind(employee_id)
    .bind(workspace_id)
    .fetch_optional(pool)
    .await
}

pub async fn upsert_department_payroll_config(
    pool: &PgPool,
    department_id: Uuid,
    std_working_days: Decimal,
    hours_per_day: Decimal,
    effective_date: NaiveDate,
) -> Result<DepartmentPayrollConfig, sqlx::Error> {
    sqlx::query_as!(
        DepartmentPayrollConfig,
        r#"INSERT INTO department_payroll_configs (department_id, std_working_days, hours_per_day, effective_date)
           VALUES ($1, $2, $3, $4)
           ON CONFLICT (department_id, effective_date) DO UPDATE 
           SET std_working_days = EXCLUDED.std_working_days,
               hours_per_day = EXCLUDED.hours_per_day
           RETURNING id, department_id, std_working_days, hours_per_day, effective_date, created_at as "created_at!""#,
        department_id,
        std_working_days,
        hours_per_day,
        effective_date
    )
    .fetch_one(pool)
    .await
}

pub async fn get_department_payroll_config_at(
    pool: &PgPool,
    department_id: Uuid,
    date: NaiveDate,
) -> Result<Option<DepartmentPayrollConfig>, sqlx::Error> {
    sqlx::query_as!(
        DepartmentPayrollConfig,
        r#"SELECT id, department_id, std_working_days, hours_per_day, effective_date, created_at as "created_at!"
           FROM department_payroll_configs
           WHERE department_id = $1 AND effective_date <= $2
           ORDER BY effective_date DESC LIMIT 1"#,
        department_id,
        date
    )
    .fetch_optional(pool)
    .await
}

impl super::PoolBacked for SqlxDepartmentRepository {
    fn from_pool(pool: sqlx::PgPool) -> Self {
        Self::new(pool)
    }
    fn pool(&self) -> &sqlx::PgPool {
        &self.pool
    }
}
