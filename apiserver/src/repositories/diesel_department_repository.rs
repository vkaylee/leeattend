use async_trait::async_trait;
use chrono::{NaiveDate, Utc};
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use rust_decimal::Decimal;
use uuid::Uuid;

use crate::errors::AppError;
use crate::models::diesel::department::{
    DieselDepartment, NewDieselDepartment, UpdateDieselDepartment,
};
use crate::models::{Department, DepartmentPayrollConfig, EmployeeDepartmentHistory};
use crate::repositories::department_repository::DepartmentRepository;
use crate::schema::departments;

const ACTIVE_EMPLOYEE_COUNT_SQL: &str = r"
    COALESCE((
        SELECT COUNT(DISTINCT e.id) FROM employees e
        JOIN employments em ON em.employee_id = e.id AND em.status = 'active' AND em.deleted_at IS NULL
        WHERE e.workspace_id = departments.workspace_id
          AND (
              SELECT department_id FROM employee_department_history edh
              WHERE edh.employee_id = e.id
              ORDER BY edh.effective_date DESC
              LIMIT 1
          ) = departments.id
    ), 0)
";

#[derive(Debug, Queryable, QueryableByName)]
pub struct DepartmentWithCountEntity {
    #[diesel(sql_type = diesel::sql_types::Uuid)]
    pub id: Uuid,
    #[diesel(sql_type = diesel::sql_types::Uuid)]
    pub workspace_id: Uuid,
    #[diesel(sql_type = diesel::sql_types::Text)]
    pub name: String,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
    pub description: Option<String>,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Uuid>)]
    pub manager_id: Option<Uuid>,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Timestamptz>)]
    pub created_at: Option<chrono::DateTime<Utc>>,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Timestamptz>)]
    pub updated_at: Option<chrono::DateTime<Utc>>,
    #[diesel(sql_type = diesel::sql_types::BigInt)]
    pub active_employee_count: i64,
}

impl From<DepartmentWithCountEntity> for Department {
    fn from(entity: DepartmentWithCountEntity) -> Self {
        Department::new(
            entity.id,
            entity.workspace_id,
            entity.name,
            entity.description,
            entity.manager_id,
            entity.created_at.unwrap_or_else(Utc::now),
            entity.updated_at.unwrap_or_else(Utc::now),
            entity.active_employee_count,
        )
    }
}

#[derive(Debug, Queryable, QueryableByName)]
pub struct EmployeeDepartmentHistoryEntity {
    #[diesel(sql_type = diesel::sql_types::Uuid)]
    pub id: Uuid,
    #[diesel(sql_type = diesel::sql_types::Uuid)]
    pub employee_id: Uuid,
    #[diesel(sql_type = diesel::sql_types::Uuid)]
    pub department_id: Uuid,
    #[diesel(sql_type = diesel::sql_types::Date)]
    pub effective_date: NaiveDate,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
    pub notes: Option<String>,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Timestamptz>)]
    pub created_at: Option<chrono::DateTime<Utc>>,
}

impl From<EmployeeDepartmentHistoryEntity> for EmployeeDepartmentHistory {
    fn from(entity: EmployeeDepartmentHistoryEntity) -> Self {
        EmployeeDepartmentHistory::new(
            entity.id,
            entity.employee_id,
            entity.department_id,
            entity.effective_date,
            entity.notes,
            entity.created_at.unwrap_or_else(Utc::now),
        )
    }
}

pub struct DieselDepartmentRepository {
    diesel_pools: crate::diesel_pool::DieselPools,
}

impl DieselDepartmentRepository {
    pub fn new(diesel_pools: crate::diesel_pool::DieselPools) -> Self {
        Self { diesel_pools }
    }
}

#[async_trait]
impl DepartmentRepository for DieselDepartmentRepository {
    async fn create_department(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        name: String,
        description: Option<String>,
        manager_id: Option<Uuid>,
    ) -> Result<Department, AppError> {
        let mut conn = self
            .diesel_pools
            .tenant_transaction_conn(auth_token)
            .await?;
        let workspace_id = auth_token.workspace_id();

        let new_dept = NewDieselDepartment {
            id: Uuid::new_v4(),
            workspace_id,
            name,
            description,
            manager_id,
        };

        let result: DieselDepartment = diesel::insert_into(departments::table)
            .values(&new_dept)
            .get_result(&mut *conn)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        let entity = departments::table
            .filter(departments::id.eq(result.id))
            .filter(departments::workspace_id.eq(workspace_id))
            .filter(departments::deleted_at.is_null())
            .select((
                departments::all_columns,
                diesel::dsl::sql::<diesel::sql_types::BigInt>(ACTIVE_EMPLOYEE_COUNT_SQL),
            ))
            .first::<(DieselDepartment, i64)>(&mut *conn)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        let entity = DepartmentWithCountEntity {
            id: entity.0.id,
            workspace_id: entity.0.workspace_id,
            name: entity.0.name,
            description: entity.0.description,
            manager_id: entity.0.manager_id,
            created_at: entity.0.created_at,
            updated_at: entity.0.updated_at,
            active_employee_count: entity.1,
        };

        Ok(entity.into())
    }

    async fn get_department_by_id(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        id: Uuid,
    ) -> Result<Option<Department>, AppError> {
        let mut conn = self
            .diesel_pools
            .tenant_transaction_conn(auth_token)
            .await?;
        let workspace_id = auth_token.workspace_id();

        let entity = departments::table
            .filter(departments::id.eq(id))
            .filter(departments::workspace_id.eq(workspace_id))
            .filter(departments::deleted_at.is_null())
            .select((
                departments::all_columns,
                diesel::dsl::sql::<diesel::sql_types::BigInt>(ACTIVE_EMPLOYEE_COUNT_SQL),
            ))
            .first::<(DieselDepartment, i64)>(&mut *conn)
            .await
            .optional()
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        let entity = entity.map(|(d, count)| DepartmentWithCountEntity {
            id: d.id,
            workspace_id: d.workspace_id,
            name: d.name,
            description: d.description,
            manager_id: d.manager_id,
            created_at: d.created_at,
            updated_at: d.updated_at,
            active_employee_count: count,
        });

        Ok(entity.map(Into::into))
    }

    async fn get_department_by_name(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        name: &str,
    ) -> Result<Option<Uuid>, AppError> {
        #[derive(Debug, diesel::QueryableByName)]
        struct IdOnly {
            #[diesel(sql_type = diesel::sql_types::Uuid)]
            id: Uuid,
        }

        let mut conn = self
            .diesel_pools
            .tenant_transaction_conn(auth_token)
            .await?;
        let workspace_id = auth_token.workspace_id();

        let result: Option<IdOnly> = departments::table
            .filter(departments::workspace_id.eq(workspace_id))
            .filter(departments::deleted_at.is_null())
            .filter(
                diesel::dsl::sql::<diesel::sql_types::Bool>("LOWER(name) = ")
                    .bind::<diesel::sql_types::Text, _>(name.to_lowercase()),
            )
            .select(departments::id)
            .first::<Uuid>(&mut *conn)
            .await
            .map(|id| IdOnly { id })
            .optional()
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(result.map(|r| r.id))
    }

    async fn get_department_ids_by_names(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        names: &[String],
    ) -> Result<std::collections::HashMap<String, Uuid>, AppError> {
        if names.is_empty() {
            return Ok(std::collections::HashMap::new());
        }

        #[derive(Debug, diesel::Queryable)]
        struct IdNameOnly {
            id: Uuid,
            name: String,
        }

        let mut conn = self
            .diesel_pools
            .tenant_transaction_conn(auth_token)
            .await?;
        let workspace_id = auth_token.workspace_id();
        let lower_names: Vec<String> = names.iter().map(|n| n.to_lowercase()).collect();

        let results: Vec<IdNameOnly> = departments::table
            .filter(departments::workspace_id.eq(workspace_id))
            .filter(departments::deleted_at.is_null())
            .filter(
                diesel::dsl::sql::<diesel::sql_types::Bool>("LOWER(name) = ANY(")
                    .bind::<diesel::sql_types::Array<diesel::sql_types::Text>, _>(lower_names)
                    .sql(")"),
            )
            .select((departments::id, departments::name))
            .load::<IdNameOnly>(&mut *conn)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        let mut map = std::collections::HashMap::new();
        for r in results {
            map.insert(r.name.to_lowercase(), r.id);
        }

        Ok(map)
    }

    async fn bulk_create_departments_skip_duplicates(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        deps: &[Department],
    ) -> Result<(), AppError> {
        if deps.is_empty() {
            return Ok(());
        }
        let mut conn = self
            .diesel_pools
            .tenant_transaction_conn(auth_token)
            .await?;

        let new_entities: Vec<NewDieselDepartment> = deps
            .iter()
            .map(|d| NewDieselDepartment {
                id: d.id(),
                workspace_id: d.workspace_id(),
                name: d.name().to_string(),
                description: d.description().map(String::from),
                manager_id: d.manager_id(),
            })
            .collect();

        for chunk in new_entities.chunks(1000) {
            diesel::insert_into(departments::table)
                .values(chunk)
                .execute(&mut *conn)
                .await
                .map_err(|e| AppError::Internal(format!("db: {e}")))?;
        }

        Ok(())
    }

    async fn update_department(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        id: Uuid,
        req: crate::models::UpdateDepartmentRequest,
    ) -> Result<Department, AppError> {
        let mut conn = self
            .diesel_pools
            .tenant_transaction_conn(auth_token)
            .await?;
        let workspace_id = auth_token.workspace_id();

        let update_model = UpdateDieselDepartment {
            name: req.name,
            description: req.description,
            manager_id: req.manager_id,
        };

        diesel::update(departments::table)
            .filter(departments::id.eq(id))
            .filter(departments::workspace_id.eq(workspace_id))
            .set(&update_model)
            .execute(&mut *conn)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        let entity = departments::table
            .filter(departments::id.eq(id))
            .filter(departments::workspace_id.eq(workspace_id))
            .filter(departments::deleted_at.is_null())
            .select((
                departments::all_columns,
                diesel::dsl::sql::<diesel::sql_types::BigInt>(ACTIVE_EMPLOYEE_COUNT_SQL),
            ))
            .first::<(DieselDepartment, i64)>(&mut *conn)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        let entity = DepartmentWithCountEntity {
            id: entity.0.id,
            workspace_id: entity.0.workspace_id,
            name: entity.0.name,
            description: entity.0.description,
            manager_id: entity.0.manager_id,
            created_at: entity.0.created_at,
            updated_at: entity.0.updated_at,
            active_employee_count: entity.1,
        };

        Ok(entity.into())
    }

    async fn delete_department(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        id: Uuid,
    ) -> Result<(), AppError> {
        let mut conn = self
            .diesel_pools
            .tenant_transaction_conn(auth_token)
            .await?;
        let workspace_id = auth_token.workspace_id();

        diesel::update(departments::table)
            .filter(departments::id.eq(id))
            .filter(departments::workspace_id.eq(workspace_id))
            .set(departments::deleted_at.eq(Utc::now()))
            .execute(&mut *conn)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(())
    }

    async fn get_departments(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        page: i64,
        limit: i64,
    ) -> Result<(Vec<Department>, i64), AppError> {
        let mut conn = self
            .diesel_pools
            .tenant_transaction_conn(auth_token)
            .await?;
        let workspace_id = auth_token.workspace_id();

        let total: i64 = departments::table
            .filter(departments::workspace_id.eq(workspace_id))
            .filter(departments::deleted_at.is_null())
            .count()
            .get_result(&mut *conn)
            .await
            .map_err(|e| AppError::Internal(format!("db count: {e}")))?;

        let results = departments::table
            .filter(departments::workspace_id.eq(workspace_id))
            .filter(departments::deleted_at.is_null())
            .select((
                departments::all_columns,
                diesel::dsl::sql::<diesel::sql_types::BigInt>(ACTIVE_EMPLOYEE_COUNT_SQL),
            ))
            .order(departments::name)
            .limit(limit)
            .offset((page - 1) * limit)
            .load::<(DieselDepartment, i64)>(&mut *conn)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        let data = results
            .into_iter()
            .map(|(d, count)| DepartmentWithCountEntity {
                id: d.id,
                workspace_id: d.workspace_id,
                name: d.name,
                description: d.description,
                manager_id: d.manager_id,
                created_at: d.created_at,
                updated_at: d.updated_at,
                active_employee_count: count,
            })
            .map(Into::into)
            .collect();

        Ok((data, total))
    }

    async fn transfer_employee(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        employee_id: Uuid,
        department_id: Uuid,
        effective_date: NaiveDate,
        notes: Option<String>,
    ) -> Result<EmployeeDepartmentHistory, AppError> {
        let mut conn = self
            .diesel_pools
            .tenant_transaction_conn(auth_token)
            .await?;
        let _workspace_id = auth_token.workspace_id();

        use crate::schema::employee_department_history;

        let entity: EmployeeDepartmentHistoryEntity =
            diesel::insert_into(employee_department_history::table)
                .values((
                    employee_department_history::employee_id.eq(employee_id),
                    employee_department_history::department_id.eq(department_id),
                    employee_department_history::effective_date.eq(effective_date),
                    employee_department_history::notes.eq(notes),
                ))
                .returning(employee_department_history::all_columns)
                .get_result(&mut *conn)
                .await
                .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(entity.into())
    }

    async fn get_employee_department_history(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        employee_id: Uuid,
    ) -> Result<Vec<EmployeeDepartmentHistory>, AppError> {
        let mut conn = self
            .diesel_pools
            .tenant_transaction_conn(auth_token)
            .await?;
        let workspace_id = auth_token.workspace_id();

        use crate::schema::{employee_department_history, employees};

        let results: Vec<EmployeeDepartmentHistoryEntity> = employee_department_history::table
            .inner_join(
                employees::table.on(employees::id.eq(employee_department_history::employee_id)),
            )
            .filter(employee_department_history::employee_id.eq(employee_id))
            .filter(employees::workspace_id.eq(workspace_id))
            .select(employee_department_history::all_columns)
            .order(employee_department_history::effective_date.desc())
            .load(&mut *conn)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(results.into_iter().map(Into::into).collect())
    }

    async fn get_employee_department_history_batch(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        employee_ids: &[Uuid],
    ) -> Result<std::collections::HashMap<Uuid, Vec<EmployeeDepartmentHistory>>, AppError> {
        let mut conn = self
            .diesel_pools
            .tenant_transaction_conn(auth_token)
            .await?;
        let workspace_id = auth_token.workspace_id();

        use crate::schema::{employee_department_history, employees};

        let results: Vec<EmployeeDepartmentHistoryEntity> = employee_department_history::table
            .inner_join(
                employees::table.on(employees::id.eq(employee_department_history::employee_id)),
            )
            .filter(employee_department_history::employee_id.eq_any(employee_ids))
            .filter(employees::workspace_id.eq(workspace_id))
            .select(employee_department_history::all_columns)
            .order((
                employee_department_history::employee_id,
                employee_department_history::effective_date.desc(),
            ))
            .load(&mut *conn)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        let mut map: std::collections::HashMap<Uuid, Vec<EmployeeDepartmentHistory>> =
            std::collections::HashMap::new();
        for entity in results {
            map.entry(entity.employee_id)
                .or_default()
                .push(entity.into());
        }

        Ok(map)
    }

    async fn get_current_department(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        employee_id: Uuid,
    ) -> Result<Option<EmployeeDepartmentHistory>, AppError> {
        let mut conn = self
            .diesel_pools
            .tenant_transaction_conn(auth_token)
            .await?;
        let workspace_id = auth_token.workspace_id();

        use crate::schema::{employee_department_history, employees};

        let today = Utc::now().date_naive();
        let result: Option<EmployeeDepartmentHistoryEntity> = employee_department_history::table
            .inner_join(
                employees::table.on(employees::id.eq(employee_department_history::employee_id)),
            )
            .filter(employee_department_history::employee_id.eq(employee_id))
            .filter(employees::workspace_id.eq(workspace_id))
            .filter(employee_department_history::effective_date.le(today))
            .select(employee_department_history::all_columns)
            .order(employee_department_history::effective_date.desc())
            .first(&mut *conn)
            .await
            .optional()
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(result.map(Into::into))
    }

    async fn get_current_departments_batch(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        employee_ids: &[Uuid],
    ) -> Result<std::collections::HashMap<Uuid, EmployeeDepartmentHistory>, AppError> {
        let mut conn = self
            .diesel_pools
            .tenant_transaction_conn(auth_token)
            .await?;
        let workspace_id = auth_token.workspace_id();

        use crate::schema::{employee_department_history, employees};

        let today = Utc::now().date_naive();
        let results: Vec<EmployeeDepartmentHistoryEntity> = employee_department_history::table
            .inner_join(
                employees::table.on(employees::id.eq(employee_department_history::employee_id)),
            )
            .filter(employees::workspace_id.eq(workspace_id))
            .filter(employee_department_history::employee_id.eq_any(employee_ids))
            .filter(employee_department_history::effective_date.le(today))
            .select(employee_department_history::all_columns)
            .order((
                employee_department_history::employee_id.asc(),
                employee_department_history::effective_date.desc(),
            ))
            .load(&mut *conn)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        let mut map = std::collections::HashMap::new();
        for entity in results {
            // Because they are ordered by employee_id ASC and effective_date DESC,
            // the first one we encounter for a given employee_id is their current department.
            map.entry(entity.employee_id)
                .or_insert_with(|| entity.into());
        }

        Ok(map)
    }

    async fn upsert_department_payroll_config(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        department_id: Uuid,
        std_working_days: Decimal,
        hours_per_day: Decimal,
        effective_date: NaiveDate,
    ) -> Result<DepartmentPayrollConfig, AppError> {
        let mut conn = self
            .diesel_pools
            .tenant_transaction_conn(auth_token)
            .await?;
        let workspace_id = auth_token.workspace_id();

        // Check if department exists and belongs to the workspace of this tenant
        let department_exists = departments::table
            .filter(departments::id.eq(department_id))
            .filter(departments::workspace_id.eq(workspace_id))
            .filter(departments::deleted_at.is_null())
            .select(departments::id)
            .first::<Uuid>(&mut *conn)
            .await
            .optional()
            .map_err(|e| AppError::Internal(format!("db check department: {e}")))?;

        if department_exists.is_none() {
            return Err(AppError::NotFound("error.department_not_found".to_string()));
        }

        use crate::schema::workspace_config_entries;

        let keys_and_values = vec![
            ("payroll.fixed_std_days", std_working_days),
            ("payroll.hours_per_day", hours_per_day),
        ];

        let mut first_id = Uuid::new_v4();

        for (key, val) in keys_and_values {
            let existing = workspace_config_entries::table
                .filter(workspace_config_entries::workspace_id.eq(workspace_id))
                .filter(workspace_config_entries::scope_type.eq("department"))
                .filter(workspace_config_entries::scope_id.eq(department_id))
                .filter(workspace_config_entries::config_key.eq(key))
                .filter(workspace_config_entries::effective_from.eq(effective_date))
                .first::<crate::models::WorkspaceConfigEntry>(&mut *conn)
                .await
                .optional()
                .map_err(|e| AppError::Internal(format!("db: {e}")))?;

            if let Some(entry) = existing {
                first_id = entry.id();
                diesel::update(
                    workspace_config_entries::table
                        .filter(workspace_config_entries::id.eq(entry.id())),
                )
                .set((
                    workspace_config_entries::value_decimal.eq(val),
                    workspace_config_entries::updated_at.eq(chrono::Utc::now()),
                ))
                .execute(&mut *conn)
                .await
                .map_err(|e| AppError::Internal(format!("db: {e}")))?;
            } else {
                let id = Uuid::new_v4();
                if key == "payroll.fixed_std_days" {
                    first_id = id;
                }
                let entry = crate::models::WorkspaceConfigEntry::new(
                    id,
                    workspace_id,
                    "department".to_string(),
                    Some(department_id),
                    key.to_string(),
                    None,
                    Some(val),
                    None,
                    None,
                    None,
                    effective_date,
                    None,
                    None,
                    chrono::Utc::now(),
                    chrono::Utc::now(),
                );
                diesel::insert_into(workspace_config_entries::table)
                    .values(&entry)
                    .execute(&mut *conn)
                    .await
                    .map_err(|e| AppError::Internal(format!("db: {e}")))?;
            }
        }

        Ok(DepartmentPayrollConfig {
            id: first_id,
            department_id,
            std_working_days,
            hours_per_day,
            effective_date,
            created_at: chrono::Utc::now(),
        })
    }

    async fn get_department_payroll_config_at(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        department_id: Uuid,
        date: NaiveDate,
    ) -> Result<Option<DepartmentPayrollConfig>, AppError> {
        let mut conn = self
            .diesel_pools
            .tenant_transaction_conn(auth_token)
            .await?;
        let workspace_id = auth_token.workspace_id();

        use crate::schema::{departments, workspace_config_entries};

        let department_exists = departments::table
            .filter(departments::id.eq(department_id))
            .filter(departments::workspace_id.eq(workspace_id))
            .filter(departments::deleted_at.is_null())
            .select(departments::id)
            .first::<Uuid>(&mut *conn)
            .await
            .optional()
            .map_err(|e| AppError::Internal(format!("db check department: {e}")))?;

        if department_exists.is_none() {
            return Ok(None);
        }

        let entries = workspace_config_entries::table
            .filter(workspace_config_entries::workspace_id.eq(workspace_id))
            .filter(workspace_config_entries::scope_type.eq("department"))
            .filter(workspace_config_entries::scope_id.eq(department_id))
            .filter(workspace_config_entries::effective_from.le(date))
            .load::<crate::models::WorkspaceConfigEntry>(&mut *conn)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        if entries.is_empty() {
            return Ok(None);
        }

        let mut fixed_std_days = None;
        let mut hours_per_day = None;
        let mut max_effective_from = None;
        let mut entry_id = Uuid::new_v4();

        for entry in &entries {
            if entry.config_key() == "payroll.fixed_std_days"
                && max_effective_from.is_none_or(|max_d| entry.effective_from() >= max_d)
            {
                fixed_std_days = entry.value_decimal();
                max_effective_from = Some(entry.effective_from());
                entry_id = entry.id();
            } else if entry.config_key() == "payroll.hours_per_day"
                && max_effective_from.is_none_or(|max_d| entry.effective_from() >= max_d)
            {
                hours_per_day = entry.value_decimal();
                max_effective_from = Some(entry.effective_from());
            }
        }

        if fixed_std_days.is_none() && hours_per_day.is_none() {
            return Ok(None);
        }

        let effective_date = max_effective_from.unwrap_or(date);

        Ok(Some(DepartmentPayrollConfig {
            id: entry_id,
            department_id,
            std_working_days: fixed_std_days.unwrap_or_else(|| rust_decimal::Decimal::from(0)),
            hours_per_day: hours_per_day.unwrap_or_else(|| rust_decimal::Decimal::from(0)),
            effective_date,
            created_at: chrono::Utc::now(),
        }))
    }

    async fn get_department_payroll_configs(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        department_id: Uuid,
    ) -> Result<Vec<DepartmentPayrollConfig>, AppError> {
        let mut conn = self
            .diesel_pools
            .tenant_transaction_conn(auth_token)
            .await?;
        let workspace_id = auth_token.workspace_id();

        use crate::schema::{departments, workspace_config_entries};

        let department_exists = departments::table
            .filter(departments::id.eq(department_id))
            .filter(departments::workspace_id.eq(workspace_id))
            .filter(departments::deleted_at.is_null())
            .select(departments::id)
            .first::<Uuid>(&mut *conn)
            .await
            .optional()
            .map_err(|e| AppError::Internal(format!("db check department: {e}")))?;

        if department_exists.is_none() {
            return Ok(Vec::new());
        }

        let entries = workspace_config_entries::table
            .filter(workspace_config_entries::workspace_id.eq(workspace_id))
            .filter(workspace_config_entries::scope_type.eq("department"))
            .filter(workspace_config_entries::scope_id.eq(department_id))
            .load::<crate::models::WorkspaceConfigEntry>(&mut *conn)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        let mut dates: Vec<chrono::NaiveDate> = entries
            .iter()
            .map(crate::models::WorkspaceConfigEntry::effective_from)
            .collect();
        dates.sort_unstable();
        dates.dedup();
        dates.reverse();

        let mut configs = Vec::new();
        for d in dates {
            let mut fixed_std_days = None;
            let mut hours_per_day = None;
            let mut entry_id = Uuid::new_v4();
            for entry in &entries {
                if entry.effective_from() == d {
                    if entry.config_key() == "payroll.fixed_std_days" {
                        fixed_std_days = entry.value_decimal();
                        entry_id = entry.id();
                    } else if entry.config_key() == "payroll.hours_per_day" {
                        hours_per_day = entry.value_decimal();
                    }
                }
            }
            if fixed_std_days.is_some() || hours_per_day.is_some() {
                configs.push(DepartmentPayrollConfig {
                    id: entry_id,
                    department_id,
                    std_working_days: fixed_std_days
                        .unwrap_or_else(|| rust_decimal::Decimal::from(0)),
                    hours_per_day: hours_per_day.unwrap_or_else(|| rust_decimal::Decimal::from(0)),
                    effective_date: d,
                    created_at: chrono::Utc::now(),
                });
            }
        }

        Ok(configs)
    }

    async fn get_department_payroll_configs_batch(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        department_ids: &[Uuid],
    ) -> Result<std::collections::HashMap<Uuid, Vec<DepartmentPayrollConfig>>, AppError> {
        if department_ids.is_empty() {
            return Ok(std::collections::HashMap::new());
        }

        let mut conn = self
            .diesel_pools
            .tenant_transaction_conn(auth_token)
            .await?;
        let workspace_id = auth_token.workspace_id();

        use crate::schema::workspace_config_entries;

        let entries = workspace_config_entries::table
            .filter(workspace_config_entries::workspace_id.eq(workspace_id))
            .filter(workspace_config_entries::scope_type.eq("department"))
            .filter(workspace_config_entries::scope_id.eq_any(department_ids))
            .load::<crate::models::WorkspaceConfigEntry>(&mut *conn)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        let mut department_entries: std::collections::HashMap<
            Uuid,
            Vec<crate::models::WorkspaceConfigEntry>,
        > = std::collections::HashMap::new();
        for entry in entries {
            if let Some(dept_id) = entry.scope_id() {
                department_entries.entry(dept_id).or_default().push(entry);
            }
        }

        let mut map = std::collections::HashMap::new();
        for &department_id in department_ids {
            if let Some(entries) = department_entries.get(&department_id) {
                let mut dates: Vec<chrono::NaiveDate> = entries
                    .iter()
                    .map(crate::models::WorkspaceConfigEntry::effective_from)
                    .collect();
                dates.sort_unstable();
                dates.dedup();
                dates.reverse();

                let mut configs = Vec::new();
                for d in dates {
                    let mut fixed_std_days = None;
                    let mut hours_per_day = None;
                    let mut entry_id = Uuid::new_v4();
                    for entry in entries {
                        if entry.effective_from() == d {
                            if entry.config_key() == "payroll.fixed_std_days" {
                                fixed_std_days = entry.value_decimal();
                                entry_id = entry.id();
                            } else if entry.config_key() == "payroll.hours_per_day" {
                                hours_per_day = entry.value_decimal();
                            }
                        }
                    }
                    if fixed_std_days.is_some() || hours_per_day.is_some() {
                        configs.push(DepartmentPayrollConfig {
                            id: entry_id,
                            department_id,
                            std_working_days: fixed_std_days
                                .unwrap_or_else(|| rust_decimal::Decimal::from(0)),
                            hours_per_day: hours_per_day
                                .unwrap_or_else(|| rust_decimal::Decimal::from(0)),
                            effective_date: d,
                            created_at: chrono::Utc::now(),
                        });
                    }
                }
                map.insert(department_id, configs);
            } else {
                map.insert(department_id, Vec::new());
            }
        }

        Ok(map)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diesel_pool::DieselPools;

    #[tokio::test]
    async fn test_repo_initialization() {
        let test_db_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
            "postgres://user:pass@localhost:5432/leeattend_test_db".to_string()
        });

        let pools = DieselPools::for_test(&test_db_url);
        let _repo = DieselDepartmentRepository::new(pools);
        // DI connection test passes if it compiles
    }

    #[tokio::test]
    async fn test_repo_methods_error_handling() {
        let bad_url = "postgres://invalid:invalid@localhost:12345/invalid_db";
        let pools = DieselPools::for_test(bad_url);
        let repo = DieselDepartmentRepository::new(pools);
        let ws_id = Uuid::new_v4();
        let auth_token = crate::diesel_pool::TenantAuthToken::system_override(ws_id);

        let err = repo
            .create_department(&auth_token, "Name".into(), None, None)
            .await;
        assert!(err.is_err());

        let err = repo.get_department_by_id(&auth_token, Uuid::new_v4()).await;
        assert!(err.is_err());

        let err = repo.get_department_by_name(&auth_token, "Name").await;
        assert!(err.is_err());

        let err = repo
            .get_department_ids_by_names(&auth_token, &["Name".into()])
            .await;
        assert!(err.is_err());

        let err = repo
            .bulk_create_departments_skip_duplicates(
                &auth_token,
                &[crate::models::Department::new(
                    Uuid::new_v4(),
                    ws_id,
                    "N".into(),
                    None,
                    None,
                    chrono::Utc::now(),
                    chrono::Utc::now(),
                    0,
                )],
            )
            .await;
        assert!(err.is_err());

        let err = repo
            .update_department(
                &auth_token,
                Uuid::new_v4(),
                crate::models::UpdateDepartmentRequest {
                    name: Some("Name".to_string()),
                    description: None,
                    manager_id: None,
                },
            )
            .await;
        assert!(err.is_err());

        let err = repo.delete_department(&auth_token, Uuid::new_v4()).await;
        assert!(err.is_err());

        let err = repo.get_departments(&auth_token, 1, 10).await;
        assert!(err.is_err());

        let err = repo
            .transfer_employee(
                &auth_token,
                Uuid::new_v4(),
                Uuid::new_v4(),
                chrono::NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
                None,
            )
            .await;
        assert!(err.is_err());

        let err = repo
            .get_employee_department_history(&auth_token, Uuid::new_v4())
            .await;
        assert!(err.is_err());

        let err = repo
            .get_employee_department_history_batch(&auth_token, &[Uuid::new_v4()])
            .await;
        assert!(err.is_err());

        let err = repo
            .get_current_department(&auth_token, Uuid::new_v4())
            .await;
        assert!(err.is_err());

        let err = repo
            .get_current_departments_batch(&auth_token, &[Uuid::new_v4()])
            .await;
        assert!(err.is_err());

        let err = repo
            .upsert_department_payroll_config(
                &auth_token,
                Uuid::new_v4(),
                Decimal::new(1, 0),
                Decimal::new(8, 0),
                chrono::NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            )
            .await;
        assert!(err.is_err());

        let err = repo
            .get_department_payroll_config_at(
                &auth_token,
                Uuid::new_v4(),
                chrono::NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            )
            .await;
        assert!(err.is_err());

        let err = repo
            .get_department_payroll_configs(&auth_token, Uuid::new_v4())
            .await;
        assert!(err.is_err());
    }

    #[tokio::test]
    async fn test_upsert_department_payroll_config_non_existent_department() {
        let test_db_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
            "postgres://user:pass@localhost:5432/leeattend_test_db".to_string()
        });
        let pools = DieselPools::for_test(&test_db_url);
        let repo = DieselDepartmentRepository::new(pools);
        let ws_id = Uuid::new_v4();
        let auth_token = crate::diesel_pool::TenantAuthToken::system_override(ws_id);

        let res = repo
            .upsert_department_payroll_config(
                &auth_token,
                Uuid::new_v4(),
                Decimal::new(22, 0),
                Decimal::new(8, 0),
                chrono::NaiveDate::from_ymd_opt(2026, 6, 6).unwrap(),
            )
            .await;

        assert!(res.is_err());
        match res.unwrap_err() {
            AppError::NotFound(msg) => {
                assert_eq!(msg, "error.department_not_found");
            }
            AppError::Internal(msg) => {
                assert!(
                    msg.contains("connecting") || msg.contains("connection") || msg.contains("db")
                );
            }
            other => panic!(
                "Expected AppError::NotFound department or db connection error, got {other:?}"
            ),
        }
    }
}
