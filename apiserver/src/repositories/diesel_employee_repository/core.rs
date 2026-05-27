#![allow(unused_imports)]
use crate::errors::AppError;
use async_trait::async_trait;
use chrono::NaiveDate;
use diesel::prelude::*;
use diesel_async::AsyncConnection;
use diesel_async::RunQueryDsl;
use rust_decimal::Decimal;
use uuid::Uuid;

use crate::models::diesel::employee::{
    CountRow, DeptRow, EmployeeDocumentEntity, EmployeeEmergencyContactEntity, EmployeeEntity,
    EmployeeProfileEntity, EmployeeRow, RecruitmentSourceBreakdownRow, RecruitmentSourceEntity,
    TenureRow, UpdateDieselEmployee,
};
use crate::models::{
    CreateEmergencyContactRequest, Employee, EmployeeDocument, EmployeeEmergencyContact,
    EmployeeFilterOptions, EmployeePaginationArgs, EmployeeProfile, RecruitmentSource,
    RecruitmentSourceBreakdown, TurnoverAnalytics, TurnoverByDepartment, TurnoverBySource,
    TurnoverTenureBucket, UpdateEmployeeProfileRequest, UpdateEmployeeRequest, WorkingCondition,
};
use crate::repositories::employee::*;
use crate::schema::{
    attendance_staging_logs, employee_documents, employee_emergency_contacts, employee_profiles,
    employees, recruitment_sources,
};
use diesel::sql_types::{Date, Uuid as UuidType};

use super::{
    build_employee_search_sql, DieselEmployeeRepository, EmployeeSearchSql, EMPLOYEE_DEPT_JOIN,
    EMPLOYEE_SELECT_QUERY,
};

#[allow(clippy::too_many_lines)]
#[async_trait]
impl EmployeeCoreRepository for DieselEmployeeRepository {
    async fn find_by_number(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        employee_id_number: &str,
    ) -> Result<Option<Employee>, AppError> {
        let workspace_id = auth_token.workspace_id();
        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .tenants_tx
                .get_connection(&workspace_id)
                .await
                .map_err(|e| AppError::Internal(format!("db: {e}")))?,
        );

        let normalized_id = crate::utils::normalize_employee_id(employee_id_number);
        let query = format!(
            "{EMPLOYEE_SELECT_QUERY} WHERE e.workspace_id = $1 AND e.employee_id_number = $2"
        );
        let emp: Option<EmployeeRow> = diesel::sql_query(&query)
            .bind::<UuidType, _>(workspace_id)
            .bind::<diesel::sql_types::Text, _>(normalized_id)
            .get_result(&mut *conn)
            .await
            .optional()
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(emp.map(Into::into))
    }

    async fn get_employee_by_id(
        &self,
        id: Uuid,
        auth_token: &crate::diesel_pool::TenantAuthToken,
    ) -> Result<Option<Employee>, AppError> {
        let workspace_id = auth_token.workspace_id();
        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .tenants_tx
                .get_connection(&workspace_id)
                .await
                .map_err(|e| AppError::Internal(format!("db: {e}")))?,
        );

        let query = format!("{EMPLOYEE_SELECT_QUERY} WHERE e.id = $1 AND e.workspace_id = $2");
        let emp: Option<EmployeeRow> = diesel::sql_query(&query)
            .bind::<UuidType, _>(id)
            .bind::<UuidType, _>(workspace_id)
            .get_result(&mut *conn)
            .await
            .optional()
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(emp.map(Into::into))
    }

    async fn get_employee_by_id_number(
        &self,
        id_number: &str,
        auth_token: &crate::diesel_pool::TenantAuthToken,
    ) -> Result<Option<Employee>, AppError> {
        crate::repositories::employee::EmployeeCoreRepository::find_by_number(
            self, auth_token, id_number,
        )
        .await
    }

    async fn get_employees_by_ids(
        &self,
        ids: &[Uuid],
        auth_token: &crate::diesel_pool::TenantAuthToken,
    ) -> Result<Vec<Employee>, AppError> {
        let workspace_id = auth_token.workspace_id();
        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .tenants_tx
                .get_connection(&workspace_id)
                .await
                .map_err(|e| AppError::Internal(format!("db: {e}")))?,
        );

        let query = format!("{EMPLOYEE_SELECT_QUERY} WHERE e.workspace_id = $1 AND e.id = ANY($2)");
        let emps: Vec<EmployeeRow> = diesel::sql_query(&query)
            .bind::<UuidType, _>(workspace_id)
            .bind::<diesel::sql_types::Array<UuidType>, _>(ids)
            .load(&mut *conn)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(emps.into_iter().map(Into::into).collect())
    }

    async fn get_all_employees(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        options: EmployeeFilterOptions,
    ) -> Result<(Vec<Employee>, i64), AppError> {
        let workspace_id = auth_token.workspace_id();
        use diesel::sql_types::{Array, BigInt, Nullable, Text};

        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .tenants_tx
                .get_connection(&workspace_id)
                .await
                .map_err(|e| AppError::Internal(format!("db: {e}")))?,
        );

        let search_sql = build_employee_search_sql(
            options.search_query.as_deref(),
            options.sort_by.as_deref(),
            options.sort_dir.as_deref(),
        );

        let count_dept_param = if search_sql.has_search { "$3" } else { "$2" };
        let dept_filter = format!(
            " AND ({count_dept_param}::uuid[] IS NULL OR EXISTS (\
                SELECT 1 FROM employee_department_history edh \
                WHERE edh.employee_id = e.id AND edh.department_id = ANY({count_dept_param})\
            ))"
        );

        let where_clause = &search_sql.where_clause;
        let count_sql = format!(
            "SELECT COUNT(*)::bigint as count FROM employees e {where_clause}{dept_filter}"
        );

        let count_search_param = if search_sql.has_search {
            let raw_q = options
                .search_query
                .as_deref()
                .unwrap_or_default()
                .trim()
                .to_string();
            Some(format!("%{raw_q}%"))
        } else {
            None
        };

        let dept_ids = options.department_ids.clone();

        let total_row: CountRow = if search_sql.has_search {
            diesel::sql_query(&count_sql)
                .bind::<UuidType, _>(workspace_id)
                .bind::<Nullable<Text>, _>(count_search_param)
                .bind::<Nullable<Array<UuidType>>, _>(dept_ids)
                .get_result(&mut *conn)
                .await
                .map_err(|e| AppError::Internal(format!("db: {e}")))?
        } else {
            diesel::sql_query(&count_sql)
                .bind::<UuidType, _>(workspace_id)
                .bind::<Nullable<Array<UuidType>>, _>(dept_ids)
                .get_result(&mut *conn)
                .await
                .map_err(|e| AppError::Internal(format!("db: {e}")))?
        };

        let total = total_row.count;

        let data_dept_filter = " AND ($6::uuid[] IS NULL OR EXISTS (\
            SELECT 1 FROM employee_department_history edh \
            WHERE edh.employee_id = e.id AND edh.department_id = ANY($6)\
        ))";

        let dept_join = if search_sql.needs_dept_join {
            EMPLOYEE_DEPT_JOIN
        } else {
            ""
        };

        let where_clause = &search_sql.where_clause;
        let order_clause = &search_sql.order_clause;
        let sql = format!(
            "{EMPLOYEE_SELECT_QUERY} {dept_join} {where_clause}{data_dept_filter} {order_clause} LIMIT $3 OFFSET $4"
        );

        let (search_str_1, search_str_2) = if search_sql.has_search {
            let raw_q = options
                .search_query
                .as_deref()
                .unwrap_or_default()
                .trim()
                .to_string();
            (Some(format!("%{raw_q}%")), Some(raw_q))
        } else {
            (None, None)
        };

        let offset = (options.page - 1) * options.limit;

        let employee_rows: Vec<EmployeeRow> = diesel::sql_query(&sql)
            .bind::<UuidType, _>(workspace_id)
            .bind::<Nullable<Text>, _>(search_str_1)
            .bind::<BigInt, _>(options.limit)
            .bind::<BigInt, _>(offset)
            .bind::<Nullable<Text>, _>(search_str_2)
            .bind::<Nullable<Array<UuidType>>, _>(options.department_ids)
            .load(&mut *conn)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        let employees: Vec<Employee> = employee_rows.into_iter().map(Into::into).collect();

        Ok((employees, total))
    }

    async fn get_active_employees(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
    ) -> Result<Vec<Employee>, AppError> {
        let workspace_id = auth_token.workspace_id();
        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .tenants_tx
                .get_connection(&workspace_id)
                .await
                .map_err(|e| AppError::Internal(format!("db: {e}")))?,
        );

        let query = format!(
            "{EMPLOYEE_SELECT_QUERY} WHERE e.workspace_id = $1 AND e.is_active = true ORDER BY e.full_name ASC"
        );
        let emps: Vec<EmployeeRow> = diesel::sql_query(&query)
            .bind::<UuidType, _>(workspace_id)
            .load(&mut *conn)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(emps.into_iter().map(Into::into).collect())
    }

    async fn get_active_employee_ids(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        filter_ids: Option<&Vec<Uuid>>,
    ) -> Result<Vec<Uuid>, AppError> {
        let workspace_id = auth_token.workspace_id();
        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .tenants_tx
                .get_connection(&workspace_id)
                .await
                .map_err(|e| AppError::Internal(format!("db: {e}")))?,
        );

        let mut query = employees::table
            .select(employees::id)
            .filter(employees::workspace_id.eq(workspace_id))
            .filter(employees::is_active.eq(true))
            .into_boxed();

        if let Some(ids) = filter_ids {
            query = query.filter(employees::id.eq_any(ids.clone()));
        }

        let result: Vec<Uuid> = query
            .order(employees::created_at.asc())
            .load(&mut *conn)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(result)
    }

    async fn get_paginated_employee_ids(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        args: EmployeePaginationArgs<'_>,
    ) -> Result<(Vec<EmployeeBasicInfo>, i64), AppError> {
        let workspace_id = auth_token.workspace_id();
        use crate::schema::employee_department_history;

        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .tenants_tx
                .get_connection(&workspace_id)
                .await
                .map_err(|e| AppError::Internal(format!("db: {e}")))?,
        );

        let mut count_query = employees::table
            .filter(employees::workspace_id.eq(workspace_id))
            .into_boxed();

        if let Some(dept_id) = args.department_id {
            count_query = count_query.filter(diesel::dsl::exists(
                employee_department_history::table
                    .filter(employee_department_history::employee_id.eq(employees::id))
                    .filter(employee_department_history::department_id.eq(dept_id)),
            ));
        }

        if let Some(ids) = args.filter_ids {
            count_query = count_query.filter(employees::id.eq_any(ids));
        }

        let total: i64 = count_query
            .count()
            .get_result(&mut *conn)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        if total == 0 {
            return Ok((vec![], 0));
        }

        let mut data_query = employees::table
            .filter(employees::workspace_id.eq(workspace_id))
            .into_boxed();

        if let Some(dept_id) = args.department_id {
            data_query = data_query.filter(diesel::dsl::exists(
                employee_department_history::table
                    .filter(employee_department_history::employee_id.eq(employees::id))
                    .filter(employee_department_history::department_id.eq(dept_id)),
            ));
        }

        if let Some(ids) = args.filter_ids {
            data_query = data_query.filter(employees::id.eq_any(ids.clone()));
        }

        if args.sort_desc {
            data_query = data_query.order(employees::created_at.desc());
        } else {
            data_query = data_query.order(employees::created_at.asc());
        }

        let offset = (args.page - 1) * args.limit;

        let result: Vec<EmployeeBasicInfo> = data_query
            .select((
                employees::id,
                employees::full_name,
                employees::employee_id_number,
            ))
            .limit(args.limit)
            .offset(offset)
            .load(&mut *conn)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok((result, total))
    }

    async fn get_employee_departments_at(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        date: NaiveDate,
    ) -> Result<std::collections::HashMap<Uuid, (Option<Uuid>, String)>, AppError> {
        let workspace_id = auth_token.workspace_id();
        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .tenants_tx
                .get_connection(&workspace_id)
                .await
                .map_err(|e| AppError::Internal(format!("db: {e}")))?,
        );

        #[derive(Debug, diesel::QueryableByName)]
        struct DeptRow {
            #[diesel(sql_type = diesel::sql_types::Uuid)]
            employee_id: Uuid,
            #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Uuid>)]
            department_id: Option<Uuid>,
            #[diesel(sql_type = diesel::sql_types::Text)]
            department_name: String,
        }

        let query = r"
            SELECT e.id AS employee_id,
                   edh.department_id AS department_id,
                   COALESCE(d.name, 'Unassigned') AS department_name
            FROM employees e
            LEFT JOIN LATERAL (
                SELECT edh2.department_id
                FROM employee_department_history edh2
                WHERE edh2.employee_id = e.id AND edh2.effective_date <= $2
                ORDER BY edh2.effective_date DESC
                LIMIT 1
            ) edh ON TRUE
            LEFT JOIN departments d ON d.id = edh.department_id
            WHERE e.is_active = TRUE AND e.workspace_id = $1
        ";

        let rows: Vec<DeptRow> = diesel::sql_query(query)
            .bind::<diesel::sql_types::Uuid, _>(workspace_id)
            .bind::<diesel::sql_types::Date, _>(date)
            .load(&mut *conn)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        let mut map = std::collections::HashMap::new();
        for row in rows {
            map.insert(row.employee_id, (row.department_id, row.department_name));
        }

        Ok(map)
    }

    async fn update_employee(
        &self,
        id: Uuid,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        req: &UpdateEmployeeRequest,
    ) -> Result<Option<Employee>, AppError> {
        let workspace_id = auth_token.workspace_id();
        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .tenants_tx
                .get_connection(&workspace_id)
                .await
                .map_err(|e| AppError::Internal(format!("db: {e}")))?,
        );

        let normalized_id = req
            .employee_id_number
            .as_ref()
            .map(|id| id.as_str().to_string());

        let tags_mapped = req.tags.as_ref().map(|tags| {
            tags.iter()
                .map(|t| Some(t.clone()))
                .collect::<Vec<Option<String>>>()
        });

        let update_model = UpdateDieselEmployee {
            user_id: req.user_id,
            full_name: req.full_name.clone(),
            employee_id_number: normalized_id,
            tags: tags_mapped,
            is_active: req.is_active,
            hire_date: req.hire_date,
            termination_date: req.termination_date.map(Some),
            updated_at: Some(chrono::Utc::now()),
            primary_location_id: req.primary_location_id,
        };

        let result: Option<EmployeeEntity> = diesel::update(employees::table)
            .filter(employees::id.eq(id))
            .filter(employees::workspace_id.eq(workspace_id))
            .set(&update_model)
            .get_result(&mut *conn)
            .await
            .optional()
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(result.map(Into::into))
    }

    async fn create_employee(&self, employee: &Employee) -> Result<(), AppError> {
        let workspace_id = employee.workspace_id;
        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .tenants_tx
                .get_connection(&workspace_id)
                .await
                .map_err(|e| AppError::Internal(format!("db: {e}")))?,
        );

        let new_entity: EmployeeEntity = employee.clone().into();

        diesel::insert_into(crate::schema::employees::table)
            .values(&new_entity)
            .execute(&mut *conn)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(())
    }

    async fn create_employee_transactional(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        employee: &Employee,
        recruitment_source_id: Option<Uuid>,
    ) -> Result<(), AppError> {
        let mut conn = self
            .diesel_pools
            .tenant_transaction_conn(auth_token)
            .await?;

        let mut new_entity: EmployeeEntity = employee.clone().into();
        // The recruitment_source_id belongs on the employee entity in this schema
        new_entity.recruitment_source_id = recruitment_source_id;

        conn.transaction(|conn| {
            Box::pin(async move {
                diesel::insert_into(crate::schema::employees::table)
                    .values(&new_entity)
                    .execute(&mut *conn)
                    .await
                    .map_err(|e| AppError::Internal(format!("db: {e}")))?;

                Ok(())
            })
        })
        .await
    }

    async fn create_employee_skip_duplicates(&self, employee: &Employee) -> Result<bool, AppError> {
        let workspace_id = employee.workspace_id;
        use crate::schema::employees;

        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .tenants_tx
                .get_connection(&workspace_id)
                .await
                .map_err(|e| AppError::Internal(format!("db: {e}")))?,
        );

        let new_entity: EmployeeEntity = employee.clone().into();

        let affected = diesel::insert_into(employees::table)
            .values(&new_entity)
            .on_conflict((employees::workspace_id, employees::employee_id_number))
            .do_nothing()
            .execute(&mut *conn)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(affected > 0)
    }

    async fn rehire_employee(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        employee_id: Uuid,
        hire_date: chrono::NaiveDate,
    ) -> Result<Option<Employee>, AppError> {
        let workspace_id = auth_token.workspace_id();
        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .tenants_tx
                .get_connection(&workspace_id)
                .await
                .map_err(|e| AppError::Internal(format!("db: {e}")))?,
        );

        let result: Option<EmployeeEntity> = diesel::update(employees::table)
            .filter(employees::id.eq(employee_id))
            .filter(employees::workspace_id.eq(workspace_id))
            .set((
                employees::is_active.eq(true),
                employees::hire_date.eq(hire_date),
                employees::termination_date.eq(None::<chrono::NaiveDate>),
                employees::updated_at.eq(Some(chrono::Utc::now())),
            ))
            .get_result(&mut *conn)
            .await
            .optional()
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(result.map(Into::into))
    }

    async fn delete_employee(
        &self,
        id: Uuid,
        auth_token: &crate::diesel_pool::TenantAuthToken,
    ) -> Result<u64, AppError> {
        let workspace_id = auth_token.workspace_id();
        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .tenants_tx
                .get_connection(&workspace_id)
                .await
                .map_err(|e| AppError::Internal(format!("db: {e}")))?,
        );

        let affected = diesel::delete(employees::table)
            .filter(employees::id.eq(id))
            .filter(employees::workspace_id.eq(workspace_id))
            .execute(&mut *conn)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(affected as u64)
    }

    async fn get_existing_employee_ids_by_numbers(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        id_numbers: &[String],
    ) -> Result<Vec<String>, AppError> {
        let workspace_id = auth_token.workspace_id();
        if id_numbers.is_empty() {
            return Ok(Vec::new());
        }

        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .tenants_tx
                .get_connection(&workspace_id)
                .await
                .map_err(|e| AppError::Internal(format!("db: {e}")))?,
        );

        let result: Vec<String> = employees::table
            .select(employees::employee_id_number)
            .filter(employees::workspace_id.eq(workspace_id))
            .filter(employees::employee_id_number.eq_any(id_numbers))
            .load(&mut *conn)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(result)
    }

    async fn get_employee_ids_by_numbers(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        id_numbers: &[String],
    ) -> Result<std::collections::HashMap<String, Uuid>, AppError> {
        let workspace_id = auth_token.workspace_id();
        if id_numbers.is_empty() {
            return Ok(std::collections::HashMap::new());
        }

        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .tenants_tx
                .get_connection(&workspace_id)
                .await
                .map_err(|e| AppError::Internal(format!("db: {e}")))?,
        );

        let results: Vec<(String, Uuid)> = employees::table
            .select((employees::employee_id_number, employees::id))
            .filter(employees::workspace_id.eq(workspace_id))
            .filter(employees::employee_id_number.eq_any(id_numbers))
            .load(&mut *conn)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        let mut map = std::collections::HashMap::new();
        for (num, id) in results {
            map.insert(num, id);
        }

        Ok(map)
    }

    async fn bulk_create_employees_skip_duplicates(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        new_employees: &[Employee],
    ) -> Result<(), AppError> {
        if new_employees.is_empty() {
            return Ok(());
        }
        let workspace_id = auth_token.workspace_id();
        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .tenants_tx
                .get_connection(&workspace_id)
                .await
                .map_err(|e| AppError::Internal(format!("db: {e}")))?,
        );

        let new_entities: Vec<EmployeeEntity> =
            new_employees.iter().map(|e| e.clone().into()).collect();

        for chunk in new_entities.chunks(1000) {
            diesel::insert_into(employees::table)
                .values(chunk)
                .on_conflict((employees::workspace_id, employees::employee_id_number))
                .do_nothing()
                .execute(&mut *conn)
                .await
                .map_err(|e| AppError::Internal(format!("db: {e}")))?;
        }

        Ok(())
    }

    async fn get_managed_departments(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        user_id: Uuid,
    ) -> Result<Vec<Uuid>, AppError> {
        let workspace_id = auth_token.workspace_id();
        use crate::schema::employee_department_history;

        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .tenants_tx
                .get_connection(&workspace_id)
                .await
                .map_err(|e| AppError::Internal(format!("db: {e}")))?,
        );

        let result: Vec<Uuid> = employees::table
            .inner_join(employee_department_history::table)
            .select(employee_department_history::department_id)
            .filter(employees::workspace_id.eq(workspace_id))
            .filter(employees::user_id.eq(user_id))
            .distinct()
            .load(&mut *conn)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(result)
    }

    async fn get_employee_id_by_user_id(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        user_id: Uuid,
    ) -> Result<Option<Uuid>, AppError> {
        let workspace_id = auth_token.workspace_id();
        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .tenants_tx
                .get_connection(&workspace_id)
                .await
                .map_err(|e| AppError::Internal(format!("db: {e}")))?,
        );

        let result: Option<Uuid> = employees::table
            .select(employees::id)
            .filter(employees::workspace_id.eq(workspace_id))
            .filter(employees::user_id.eq(user_id))
            .filter(employees::is_active.eq(true))
            .order(employees::updated_at.desc())
            .first(&mut *conn)
            .await
            .optional()
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(result)
    }

    // --- ESS (Employee Self Service) ---

    async fn get_employee_timezone_fallback(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        employee_id: Uuid,
    ) -> Result<chrono_tz::Tz, AppError> {
        let workspace_id = auth_token.workspace_id();
        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .tenants_tx
                .get_connection(&workspace_id)
                .await
                .map_err(|e| AppError::Internal(format!("db: {e}")))?,
        );

        #[derive(Debug, diesel::QueryableByName)]
        struct TzRow {
            #[diesel(sql_type = diesel::sql_types::Text)]
            timezone: String,
        }

        let query = r#"
            SELECT wl.timezone as timezone
            FROM employees e
            JOIN work_locations wl ON wl.id = e.primary_location_id
            WHERE e.id = $1 AND e.workspace_id = $2
        "#;

        let row: TzRow = diesel::sql_query(query)
            .bind::<diesel::sql_types::Uuid, _>(employee_id)
            .bind::<diesel::sql_types::Uuid, _>(workspace_id)
            .get_result(&mut *conn)
            .await
            .map_err(|e| match e {
                diesel::result::Error::NotFound => AppError::NotFound("Employee not found".into()),
                _ => AppError::Internal(format!("db: {e}")),
            })?;

        Ok(crate::utils::tz::parse_workspace_tz(&row.timezone))
    }

    async fn get_employee_timezone_at_date(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        employee_id: Uuid,
        tentative_date: chrono::NaiveDate,
    ) -> Result<chrono_tz::Tz, AppError> {
        let workspace_id = auth_token.workspace_id();
        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .tenants_tx
                .get_connection(&workspace_id)
                .await
                .map_err(|e| AppError::Internal(format!("db: {e}")))?,
        );

        #[derive(Debug, diesel::QueryableByName)]
        struct TzRow {
            #[diesel(sql_type = diesel::sql_types::Text)]
            timezone: String,
        }

        let query = r#"
            SELECT COALESCE(
                (SELECT host_wl.timezone
                 FROM employee_assignments ea
                 JOIN work_locations host_wl ON host_wl.id = ea.host_location_id
                 WHERE ea.employee_id = e.id AND ea.status = 'active'
                   AND ea.start_date <= $3
                   AND (ea.end_date IS NULL OR ea.end_date >= $3)
                 ORDER BY ea.created_at DESC LIMIT 1),
                primary_wl.timezone
            ) as timezone
            FROM employees e
            JOIN work_locations primary_wl ON primary_wl.id = e.primary_location_id
            WHERE e.id = $1 AND e.workspace_id = $2
        "#;

        let row: TzRow = diesel::sql_query(query)
            .bind::<diesel::sql_types::Uuid, _>(employee_id)
            .bind::<diesel::sql_types::Uuid, _>(workspace_id)
            .bind::<diesel::sql_types::Date, _>(tentative_date)
            .get_result(&mut *conn)
            .await
            .map_err(|e| match e {
                diesel::result::Error::NotFound => AppError::NotFound("Employee not found".into()),
                _ => AppError::Internal(format!("db: {e}")),
            })?;

        Ok(crate::utils::tz::parse_workspace_tz(&row.timezone))
    }

    async fn get_headquarters_location_id(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
    ) -> Result<Option<Uuid>, AppError> {
        let workspace_id = auth_token.workspace_id();
        use crate::schema::work_locations;
        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .tenants_tx
                .get_connection(&workspace_id)
                .await
                .map_err(|e| AppError::Internal(format!("db: {e}")))?,
        );

        let hq_id: Option<Uuid> = work_locations::table
            .filter(work_locations::workspace_id.eq(workspace_id))
            .filter(work_locations::is_headquarters.eq(true))
            .select(work_locations::id)
            .first(&mut *conn)
            .await
            .optional()
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(hq_id)
    }
}
