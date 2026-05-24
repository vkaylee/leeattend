use crate::errors::AppError;
use async_trait::async_trait;
use chrono::NaiveDate;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
// removed rust_decimal::Decimal
use diesel_async::AsyncConnection;
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
use crate::repositories::employee::{EmployeeBasicInfo, EmployeeRepository};
use crate::schema::{
    employee_documents, employee_emergency_contacts, employee_profiles, employees,
    recruitment_sources,
};
use diesel::sql_types::{Date, Uuid as UuidType};

pub struct DieselEmployeeRepository {
    pub diesel_pools: crate::diesel_pool::DieselPools,
}

impl DieselEmployeeRepository {
    pub fn new(diesel_pools: crate::diesel_pool::DieselPools) -> Self {
        Self { diesel_pools }
    }
}

pub const EMPLOYEE_SELECT_QUERY: &str = r#"
    SELECT 
        e.id, e.workspace_id, e.user_id, e.full_name, e.employee_id_number, 
        e.original_hire_date, e.hire_date, e.termination_date, 
        e.tags, e.is_active, e.created_at, e.updated_at,
        d_cur.name as department_name
    FROM employees e
    LEFT JOIN (
        SELECT DISTINCT ON (employee_id) employee_id, department_id
        FROM employee_department_history
        ORDER BY employee_id, effective_date DESC
    ) edh_cur ON e.id = edh_cur.employee_id
    LEFT JOIN departments d_cur ON edh_cur.department_id = d_cur.id
"#;

pub const EMPLOYEE_DEPT_JOIN: &str = r#"
    LEFT JOIN (
        SELECT DISTINCT ON (employee_id) employee_id, department_id
        FROM employee_department_history
        ORDER BY employee_id, effective_date DESC
    ) edh_sort ON e.id = edh_sort.employee_id
    LEFT JOIN departments d_sort ON edh_sort.department_id = d_sort.id AND d_sort.workspace_id = e.workspace_id
"#;

struct EmployeeSearchSql {
    pub where_clause: String,
    pub order_clause: String,
    pub has_search: bool,
    pub needs_dept_join: bool,
}

fn build_employee_search_sql(
    search_query: &Option<String>,
    sort_by: Option<&str>,
    sort_dir: Option<&str>,
) -> EmployeeSearchSql {
    let has_search = search_query.as_ref().is_some_and(|q| !q.trim().is_empty());

    let mut where_clause = "WHERE e.workspace_id = $1".to_string();
    if has_search {
        where_clause.push_str(" AND (e.full_name ILIKE $2 OR e.employee_id_number ILIKE $2)");
    }

    let order_clause = if has_search {
        "ORDER BY \
            (e.employee_id_number = $5) DESC, \
            (e.employee_id_number ILIKE $5 || '%') DESC, \
            (e.full_name ILIKE $5) DESC, \
            (e.full_name ILIKE $5 || '%') DESC, \
            e.full_name ASC"
            .to_string()
    } else {
        let dir = match sort_dir {
            Some(d) if d.eq_ignore_ascii_case("desc") => "DESC",
            _ => "ASC",
        };
        match sort_by {
            Some("employee_id_number") => format!(
                "ORDER BY (CASE WHEN e.employee_id_number ~ '^[0-9]+$' \
                THEN CAST(e.employee_id_number AS NUMERIC) END) {dir} NULLS LAST, \
                e.employee_id_number {dir}"
            ),
            Some("department") => {
                format!("ORDER BY COALESCE(d_sort.name, '') {dir}, e.full_name ASC")
            }
            Some("status") => format!("ORDER BY e.is_active {dir}, e.full_name ASC"),
            _ => format!("ORDER BY e.full_name {dir}"),
        }
    };

    let needs_dept_join = !has_search && sort_by == Some("department");

    EmployeeSearchSql {
        where_clause,
        order_clause,
        has_search,
        needs_dept_join,
    }
}

#[allow(clippy::too_many_lines)]
#[async_trait]
impl EmployeeRepository for DieselEmployeeRepository {
    async fn find_by_number(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        employee_id_number: &str,
    ) -> Result<Option<Employee>, AppError> {
        let workspace_id = auth_token.workspace_id();
        let mut conn = self
            .diesel_pools
            .tenants
            .get_connection(&workspace_id)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        let normalized_id = crate::utils::normalize_employee_id(employee_id_number);
        let query = format!(
            "{EMPLOYEE_SELECT_QUERY} WHERE e.workspace_id = $1 AND e.employee_id_number = $2"
        );
        let emp: Option<EmployeeRow> = diesel::sql_query(&query)
            .bind::<UuidType, _>(workspace_id)
            .bind::<diesel::sql_types::Text, _>(normalized_id)
            .get_result(&mut conn)
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
        let mut conn = self
            .diesel_pools
            .tenants
            .get_connection(&workspace_id)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        let query = format!("{EMPLOYEE_SELECT_QUERY} WHERE e.id = $1 AND e.workspace_id = $2");
        let emp: Option<EmployeeRow> = diesel::sql_query(&query)
            .bind::<UuidType, _>(id)
            .bind::<UuidType, _>(workspace_id)
            .get_result(&mut conn)
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
        self.find_by_number(auth_token, id_number).await
    }

    async fn get_all_employees(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        options: EmployeeFilterOptions,
    ) -> Result<(Vec<Employee>, i64), AppError> {
        let workspace_id = auth_token.workspace_id();
        use diesel::sql_types::{Array, BigInt, Nullable, Text};

        let mut conn = self
            .diesel_pools
            .tenants
            .get_connection(&workspace_id)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        let search_sql = build_employee_search_sql(
            &options.search_query,
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
            #[allow(clippy::expect_used)]
            let raw_q = options
                .search_query
                .as_ref()
                .expect("search_query verified non-empty by has_search")
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
                .get_result(&mut conn)
                .await
                .map_err(|e| AppError::Internal(format!("db: {e}")))?
        } else {
            diesel::sql_query(&count_sql)
                .bind::<UuidType, _>(workspace_id)
                .bind::<Nullable<Array<UuidType>>, _>(dept_ids)
                .get_result(&mut conn)
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
            #[allow(clippy::expect_used)]
            let raw_q = options
                .search_query
                .as_ref()
                .expect("search_query verified non-empty by has_search")
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
            .load(&mut conn)
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
        let mut conn = self
            .diesel_pools
            .tenants
            .get_connection(&workspace_id)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        let query = format!(
            "{EMPLOYEE_SELECT_QUERY} WHERE e.workspace_id = $1 AND e.is_active = true ORDER BY e.full_name ASC"
        );
        let emps: Vec<EmployeeRow> = diesel::sql_query(&query)
            .bind::<UuidType, _>(workspace_id)
            .load(&mut conn)
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
        let mut conn = self
            .diesel_pools
            .tenants
            .get_connection(&workspace_id)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

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
            .load(&mut conn)
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

        let mut conn = self
            .diesel_pools
            .tenants
            .get_connection(&workspace_id)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

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
            .get_result(&mut conn)
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
            .load(&mut conn)
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
        let mut conn = self
            .diesel_pools
            .tenants
            .get_connection(&workspace_id)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        #[derive(Debug, diesel::QueryableByName)]
        struct DeptRow {
            #[diesel(sql_type = diesel::sql_types::Uuid)]
            employee_id: Uuid,
            #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Uuid>)]
            department_id: Option<Uuid>,
            #[diesel(sql_type = diesel::sql_types::Text)]
            department_name: String,
        }

        let query = r#"
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
        "#;

        let rows: Vec<DeptRow> = diesel::sql_query(query)
            .bind::<diesel::sql_types::Uuid, _>(workspace_id)
            .bind::<diesel::sql_types::Date, _>(date)
            .load(&mut conn)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        let mut map = std::collections::HashMap::new();
        for row in rows {
            map.insert(row.employee_id, (row.department_id, row.department_name));
        }

        Ok(map)
    }

    async fn create_employee(&self, employee: &Employee) -> Result<(), AppError> {
        let workspace_id = employee.workspace_id;
        let mut conn = self
            .diesel_pools
            .tenants
            .get_connection(&workspace_id)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        let new_entity: EmployeeEntity = employee.clone().into();

        diesel::insert_into(crate::schema::employees::table)
            .values(&new_entity)
            .execute(&mut conn)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(())
    }

    async fn create_employee_skip_duplicates(&self, employee: &Employee) -> Result<bool, AppError> {
        let workspace_id = employee.workspace_id;
        use crate::schema::employees;

        let mut conn = self
            .diesel_pools
            .tenants
            .get_connection(&workspace_id)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        let new_entity: EmployeeEntity = employee.clone().into();

        let affected = diesel::insert_into(employees::table)
            .values(&new_entity)
            .on_conflict((employees::workspace_id, employees::employee_id_number))
            .do_nothing()
            .execute(&mut conn)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(affected > 0)
    }

    async fn update_employee(
        &self,
        id: Uuid,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        req: &UpdateEmployeeRequest,
    ) -> Result<Option<Employee>, AppError> {
        let workspace_id = auth_token.workspace_id();
        let mut conn = self
            .diesel_pools
            .tenants
            .get_connection(&workspace_id)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

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
        };

        let result: Option<EmployeeEntity> = diesel::update(employees::table)
            .filter(employees::id.eq(id))
            .filter(employees::workspace_id.eq(workspace_id))
            .set(&update_model)
            .get_result(&mut conn)
            .await
            .optional()
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(result.map(Into::into))
    }

    async fn rehire_employee(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        employee_id: Uuid,
        hire_date: chrono::NaiveDate,
    ) -> Result<Option<Employee>, AppError> {
        let workspace_id = auth_token.workspace_id();
        let mut conn = self
            .diesel_pools
            .tenants
            .get_connection(&workspace_id)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        let result: Option<EmployeeEntity> = diesel::update(employees::table)
            .filter(employees::id.eq(employee_id))
            .filter(employees::workspace_id.eq(workspace_id))
            .set((
                employees::is_active.eq(true),
                employees::hire_date.eq(hire_date),
                employees::termination_date.eq(None::<chrono::NaiveDate>),
                employees::updated_at.eq(Some(chrono::Utc::now())),
            ))
            .get_result(&mut conn)
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
        let mut conn = self
            .diesel_pools
            .tenants
            .get_connection(&workspace_id)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        let affected = diesel::delete(employees::table)
            .filter(employees::id.eq(id))
            .filter(employees::workspace_id.eq(workspace_id))
            .execute(&mut conn)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(affected as u64)
    }

    async fn get_working_conditions(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        employee_id: Uuid,
    ) -> Result<Vec<WorkingCondition>, AppError> {
        let workspace_id = auth_token.workspace_id();
        use crate::schema::{employees, working_conditions};

        let mut conn = self
            .diesel_pools
            .tenants
            .get_connection(&workspace_id)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        let result = working_conditions::table
            .inner_join(employees::table)
            .select((
                working_conditions::id,
                working_conditions::employee_id,
                working_conditions::type_,
                working_conditions::minute_allowance,
                working_conditions::is_paid,
                working_conditions::start_date,
                working_conditions::end_date,
            ))
            .filter(working_conditions::employee_id.eq(employee_id))
            .filter(employees::workspace_id.eq(workspace_id))
            .order(working_conditions::start_date.desc())
            .load::<(
                Uuid,
                Uuid,
                String,
                i32,
                bool,
                chrono::NaiveDate,
                Option<chrono::NaiveDate>,
            )>(&mut conn)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(result
            .into_iter()
            .map(
                |(id, emp_id, type_, min, paid, start, end)| WorkingCondition {
                    id,
                    employee_id: emp_id,
                    type_,
                    minute_allowance: min,
                    is_paid: paid,
                    start_date: start,
                    end_date: end,
                },
            )
            .collect())
    }

    async fn get_working_conditions_at(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        employee_id: Uuid,
        date: NaiveDate,
    ) -> Result<Vec<WorkingCondition>, AppError> {
        let workspace_id = auth_token.workspace_id();
        use crate::schema::{employees, working_conditions};

        let mut conn = self
            .diesel_pools
            .tenants
            .get_connection(&workspace_id)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        let result = working_conditions::table
            .inner_join(employees::table)
            .select((
                working_conditions::id,
                working_conditions::employee_id,
                working_conditions::type_,
                working_conditions::minute_allowance,
                working_conditions::is_paid,
                working_conditions::start_date,
                working_conditions::end_date,
            ))
            .filter(working_conditions::employee_id.eq(employee_id))
            .filter(employees::workspace_id.eq(workspace_id))
            .filter(working_conditions::start_date.le(date))
            .filter(
                working_conditions::end_date
                    .is_null()
                    .or(working_conditions::end_date.ge(date)),
            )
            .load::<(
                Uuid,
                Uuid,
                String,
                i32,
                bool,
                chrono::NaiveDate,
                Option<chrono::NaiveDate>,
            )>(&mut conn)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(result
            .into_iter()
            .map(
                |(id, emp_id, type_, min, paid, start, end)| WorkingCondition {
                    id,
                    employee_id: emp_id,
                    type_,
                    minute_allowance: min,
                    is_paid: paid,
                    start_date: start,
                    end_date: end,
                },
            )
            .collect())
    }

    async fn get_working_conditions_batch(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        employee_ids: &[Uuid],
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> Result<std::collections::HashMap<Uuid, Vec<WorkingCondition>>, AppError> {
        let workspace_id = auth_token.workspace_id();
        use crate::schema::{employees, working_conditions};

        if employee_ids.is_empty() {
            return Ok(std::collections::HashMap::new());
        }

        let mut conn = self
            .diesel_pools
            .tenants
            .get_connection(&workspace_id)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        let result = working_conditions::table
            .inner_join(employees::table)
            .select((
                working_conditions::id,
                working_conditions::employee_id,
                working_conditions::type_,
                working_conditions::minute_allowance,
                working_conditions::is_paid,
                working_conditions::start_date,
                working_conditions::end_date,
            ))
            .filter(working_conditions::employee_id.eq_any(employee_ids))
            .filter(employees::workspace_id.eq(workspace_id))
            .filter(working_conditions::start_date.le(end_date))
            .filter(
                working_conditions::end_date
                    .is_null()
                    .or(working_conditions::end_date.ge(start_date)),
            )
            .order((
                working_conditions::employee_id,
                working_conditions::start_date,
            ))
            .load::<(
                Uuid,
                Uuid,
                String,
                i32,
                bool,
                chrono::NaiveDate,
                Option<chrono::NaiveDate>,
            )>(&mut conn)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        let mut map = std::collections::HashMap::with_capacity(employee_ids.len());
        for (id, emp_id, type_, min, paid, start, end) in result {
            map.entry(emp_id)
                .or_insert_with(Vec::new)
                .push(WorkingCondition {
                    id,
                    employee_id: emp_id,
                    type_,
                    minute_allowance: min,
                    is_paid: paid,
                    start_date: start,
                    end_date: end,
                });
        }
        Ok(map)
    }

    async fn create_working_condition(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        condition: &WorkingCondition,
    ) -> Result<(), AppError> {
        let workspace_id = auth_token.workspace_id();
        use crate::schema::working_conditions;

        let mut conn = self
            .diesel_pools
            .tenants
            .get_connection(&workspace_id)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        diesel::insert_into(working_conditions::table)
            .values((
                working_conditions::id.eq(condition.id),
                working_conditions::employee_id.eq(condition.employee_id),
                working_conditions::type_.eq(&condition.type_),
                working_conditions::minute_allowance.eq(condition.minute_allowance),
                working_conditions::is_paid.eq(condition.is_paid),
                working_conditions::start_date.eq(condition.start_date),
                working_conditions::end_date.eq(condition.end_date),
            ))
            .execute(&mut conn)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(())
    }

    async fn delete_working_condition(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        id: Uuid,
        employee_id: Uuid,
    ) -> Result<bool, AppError> {
        let workspace_id = auth_token.workspace_id();
        use crate::schema::working_conditions;

        let mut conn = self
            .diesel_pools
            .tenants
            .get_connection(&workspace_id)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        let affected = diesel::delete(working_conditions::table)
            .filter(working_conditions::id.eq(id))
            .filter(working_conditions::employee_id.eq(employee_id))
            .execute(&mut conn)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(affected > 0)
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

        let mut conn = self
            .diesel_pools
            .tenants
            .get_connection(&workspace_id)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        let result: Vec<String> = employees::table
            .select(employees::employee_id_number)
            .filter(employees::workspace_id.eq(workspace_id))
            .filter(employees::employee_id_number.eq_any(id_numbers))
            .load(&mut conn)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(result)
    }

    async fn upsert_employee_payroll_config(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        employee_id: Uuid,
        std_working_days: rust_decimal::Decimal,
        hours_per_day: rust_decimal::Decimal,
        effective_date: chrono::NaiveDate,
    ) -> Result<crate::models::EmployeePayrollConfig, AppError> {
        let workspace_id = auth_token.workspace_id();
        use crate::schema::employee_payroll_configs;

        let mut conn = self
            .diesel_pools
            .tenants
            .get_connection(&workspace_id)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        let result = diesel::insert_into(employee_payroll_configs::table)
            .values((
                employee_payroll_configs::employee_id.eq(employee_id),
                employee_payroll_configs::effective_date.eq(effective_date),
                employee_payroll_configs::std_working_days.eq(std_working_days),
                employee_payroll_configs::hours_per_day.eq(hours_per_day),
                employee_payroll_configs::created_at.eq(chrono::Utc::now()),
            ))
            .on_conflict((
                employee_payroll_configs::employee_id,
                employee_payroll_configs::effective_date,
            ))
            .do_update()
            .set((
                employee_payroll_configs::std_working_days.eq(std_working_days),
                employee_payroll_configs::hours_per_day.eq(hours_per_day),
            ))
            .returning((
                employee_payroll_configs::id,
                employee_payroll_configs::employee_id,
                employee_payroll_configs::std_working_days,
                employee_payroll_configs::hours_per_day,
                employee_payroll_configs::effective_date,
                employee_payroll_configs::created_at,
            ))
            .get_result::<(
                Uuid,
                Uuid,
                rust_decimal::Decimal,
                rust_decimal::Decimal,
                chrono::NaiveDate,
                Option<chrono::DateTime<chrono::Utc>>,
            )>(&mut conn)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(crate::models::EmployeePayrollConfig {
            id: result.0,
            employee_id: result.1,
            std_working_days: result.2,
            hours_per_day: result.3,
            effective_date: result.4,
            created_at: result.5.unwrap_or_else(chrono::Utc::now),
        })
    }

    async fn get_employee_payroll_config_at(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        employee_id: Uuid,
        date: chrono::NaiveDate,
    ) -> Result<Option<crate::models::EmployeePayrollConfig>, AppError> {
        let workspace_id = auth_token.workspace_id();
        use crate::schema::employee_payroll_configs;

        let mut conn = self
            .diesel_pools
            .tenants
            .get_connection(&workspace_id)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        let result = employee_payroll_configs::table
            .filter(employee_payroll_configs::employee_id.eq(employee_id))
            .filter(employee_payroll_configs::effective_date.le(date))
            .order(employee_payroll_configs::effective_date.desc())
            .select((
                employee_payroll_configs::id,
                employee_payroll_configs::employee_id,
                employee_payroll_configs::std_working_days,
                employee_payroll_configs::hours_per_day,
                employee_payroll_configs::effective_date,
                employee_payroll_configs::created_at,
            ))
            .first::<(
                Uuid,
                Uuid,
                rust_decimal::Decimal,
                rust_decimal::Decimal,
                chrono::NaiveDate,
                Option<chrono::DateTime<chrono::Utc>>,
            )>(&mut conn)
            .await
            .optional()
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(result.map(|t| crate::models::EmployeePayrollConfig {
            id: t.0,
            employee_id: t.1,
            std_working_days: t.2,
            hours_per_day: t.3,
            effective_date: t.4,
            created_at: t.5.unwrap_or_else(chrono::Utc::now),
        }))
    }

    async fn get_employee_payroll_configs(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        employee_id: Uuid,
    ) -> Result<Vec<crate::models::EmployeePayrollConfig>, AppError> {
        let workspace_id = auth_token.workspace_id();
        let mut conn = self
            .diesel_pools
            .tenants
            .get_connection(&workspace_id)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        #[derive(Debug, diesel::QueryableByName)]
        struct Entity {
            #[diesel(sql_type = diesel::sql_types::Uuid)]
            id: Uuid,
            #[diesel(sql_type = diesel::sql_types::Uuid)]
            employee_id: Uuid,
            #[diesel(sql_type = diesel::sql_types::Numeric)]
            std_working_days: bigdecimal::BigDecimal,
            #[diesel(sql_type = diesel::sql_types::Numeric)]
            hours_per_day: bigdecimal::BigDecimal,
            #[diesel(sql_type = diesel::sql_types::Date)]
            effective_date: chrono::NaiveDate,
            #[diesel(sql_type = diesel::sql_types::Timestamptz)]
            created_at: chrono::DateTime<chrono::Utc>,
        }

        let results: Vec<Entity> = diesel::sql_query(
            r#"SELECT epc.id, epc.employee_id, epc.std_working_days, epc.hours_per_day, epc.effective_date, epc.created_at
               FROM employee_payroll_configs epc
               JOIN employees e ON e.id = epc.employee_id
               WHERE epc.employee_id = $1 AND e.workspace_id = $2
               ORDER BY epc.effective_date DESC"#,
        )
        .bind::<diesel::sql_types::Uuid, _>(employee_id)
        .bind::<diesel::sql_types::Uuid, _>(workspace_id)
        .load(&mut conn)
        .await
        .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        use std::str::FromStr;
        Ok(results
            .into_iter()
            .map(|t| crate::models::EmployeePayrollConfig {
                id: t.id,
                employee_id: t.employee_id,
                std_working_days: rust_decimal::Decimal::from_str(&t.std_working_days.to_string())
                    .unwrap_or_default(),
                hours_per_day: rust_decimal::Decimal::from_str(&t.hours_per_day.to_string())
                    .unwrap_or_default(),
                effective_date: t.effective_date,
                created_at: t.created_at,
            })
            .collect())
    }

    async fn set_employee_recruitment_source(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        employee_id: Uuid,
        recruitment_source_id: Option<Uuid>,
    ) -> Result<(), AppError> {
        let workspace_id = auth_token.workspace_id();
        let mut conn = self
            .diesel_pools
            .tenants
            .get_connection(&workspace_id)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        diesel::update(employees::table)
            .filter(employees::id.eq(employee_id))
            .filter(employees::workspace_id.eq(workspace_id))
            .set((
                employees::recruitment_source_id.eq(recruitment_source_id),
                employees::updated_at.eq(Some(chrono::Utc::now())),
            ))
            .execute(&mut conn)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(())
    }

    async fn create_recruitment_source(&self, source: &RecruitmentSource) -> Result<(), AppError> {
        let workspace_id = source.workspace_id;
        let mut conn = self
            .diesel_pools
            .tenants
            .get_connection(&workspace_id)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        let entity: RecruitmentSourceEntity = source.clone().into();

        diesel::insert_into(recruitment_sources::table)
            .values(&entity)
            .execute(&mut conn)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(())
    }

    async fn get_recruitment_sources(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
    ) -> Result<Vec<RecruitmentSource>, AppError> {
        let workspace_id = auth_token.workspace_id();
        let mut conn = self
            .diesel_pools
            .tenants
            .get_connection(&workspace_id)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        let results: Vec<RecruitmentSourceEntity> = recruitment_sources::table
            .filter(recruitment_sources::workspace_id.eq(workspace_id))
            .order(recruitment_sources::created_at.desc())
            .load(&mut conn)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(results.into_iter().map(Into::into).collect())
    }

    async fn get_recruitment_source_by_id(
        &self,
        id: Uuid,
        auth_token: &crate::diesel_pool::TenantAuthToken,
    ) -> Result<Option<RecruitmentSource>, AppError> {
        let workspace_id = auth_token.workspace_id();
        let mut conn = self
            .diesel_pools
            .tenants
            .get_connection(&workspace_id)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        let result: Option<RecruitmentSourceEntity> = recruitment_sources::table
            .filter(recruitment_sources::id.eq(id))
            .filter(recruitment_sources::workspace_id.eq(workspace_id))
            .first(&mut conn)
            .await
            .optional()
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(result.map(Into::into))
    }

    async fn update_recruitment_source(
        &self,
        id: Uuid,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        req: &crate::models::UpdateRecruitmentSourceRequest,
    ) -> Result<Option<RecruitmentSource>, AppError> {
        let workspace_id = auth_token.workspace_id();
        let mut conn = self
            .diesel_pools
            .tenants
            .get_connection(&workspace_id)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        let result: Option<RecruitmentSourceEntity> = diesel::update(recruitment_sources::table)
            .filter(recruitment_sources::id.eq(id))
            .filter(recruitment_sources::workspace_id.eq(workspace_id))
            .set((
                req.name.as_ref().map(|n| recruitment_sources::name.eq(n)),
                req.referrer_employee_id
                    .map(|id| recruitment_sources::referrer_employee_id.eq(id)),
                req.referral_bonus_amount
                    .map(|a| recruitment_sources::referral_bonus_amount.eq(a)),
                req.referral_bonus_paid
                    .map(|p| recruitment_sources::referral_bonus_paid.eq(p)),
                req.agency_contact_name
                    .as_ref()
                    .map(|n| recruitment_sources::agency_contact_name.eq(n)),
                req.agency_contact_email
                    .as_ref()
                    .map(|e| recruitment_sources::agency_contact_email.eq(e)),
                req.agency_fee_amount
                    .map(|a| recruitment_sources::agency_fee_amount.eq(a)),
                req.agency_fee_paid
                    .map(|p| recruitment_sources::agency_fee_paid.eq(p)),
                req.notes.as_ref().map(|n| recruitment_sources::notes.eq(n)),
                recruitment_sources::updated_at.eq(chrono::Utc::now()),
            ))
            .get_result(&mut conn)
            .await
            .optional()
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(result.map(Into::into))
    }

    async fn delete_recruitment_source(
        &self,
        id: Uuid,
        auth_token: &crate::diesel_pool::TenantAuthToken,
    ) -> Result<u64, AppError> {
        let workspace_id = auth_token.workspace_id();
        let mut conn = self
            .diesel_pools
            .tenants
            .get_connection(&workspace_id)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        let affected = diesel::delete(recruitment_sources::table)
            .filter(recruitment_sources::id.eq(id))
            .filter(recruitment_sources::workspace_id.eq(workspace_id))
            .execute(&mut conn)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(affected as u64)
    }

    async fn get_recruitment_source_breakdown(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
    ) -> Result<Vec<RecruitmentSourceBreakdown>, AppError> {
        let workspace_id = auth_token.workspace_id();
        let mut conn = self
            .diesel_pools
            .tenants
            .get_connection(&workspace_id)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        let rows: Vec<RecruitmentSourceBreakdownRow> = diesel::sql_query(
            r#"
            SELECT
                COALESCE(rs.source_type::text, 'direct') AS source_type,
                COALESCE(rs.name, 'Direct Hire') AS source_name,
                COUNT(*) AS count
            FROM employees e
            LEFT JOIN recruitment_sources rs ON e.recruitment_source_id = rs.id
            WHERE e.workspace_id = $1 AND e.is_active = TRUE
            GROUP BY rs.source_type, rs.name
            ORDER BY count DESC
            "#,
        )
        .bind::<UuidType, _>(workspace_id)
        .load(&mut conn)
        .await
        .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(rows
            .into_iter()
            .map(|r| RecruitmentSourceBreakdown {
                source_type: r.source_type,
                source_name: r.source_name,
                count: r.count,
            })
            .collect())
    }

    async fn get_turnover_analytics(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> Result<TurnoverAnalytics, AppError> {
        let workspace_id = auth_token.workspace_id();
        let mut conn = self
            .diesel_pools
            .tenants
            .get_connection(&workspace_id)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        // 1. Total terminated count
        let total: i64 = diesel::sql_query(
            r#"SELECT COUNT(*) FROM employees e
               WHERE e.workspace_id = $1 AND e.is_active = FALSE
                 AND e.termination_date IS NOT NULL
                 AND e.termination_date >= $2 AND e.termination_date <= $3"#,
        )
        .bind::<UuidType, _>(workspace_id)
        .bind::<Date, _>(start_date)
        .bind::<Date, _>(end_date)
        .get_result::<CountRow>(&mut conn)
        .await
        .map(|r| r.count)
        .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        // 2. Tenure distribution
        let tenure_rows: Vec<TenureRow> = diesel::sql_query(
            r#"
            SELECT bucket, COUNT(*) AS count FROM (
                SELECT CASE
                    WHEN e.termination_date - e.hire_date < 90 THEN '< 3 months'
                    WHEN e.termination_date - e.hire_date < 180 THEN '3-6 months'
                    WHEN e.termination_date - e.hire_date < 365 THEN '6-12 months'
                    WHEN e.termination_date - e.hire_date < 730 THEN '1-2 years'
                    WHEN e.termination_date - e.hire_date < 1825 THEN '2-5 years'
                    ELSE '> 5 years'
                END AS bucket
                FROM employees e
                WHERE e.workspace_id = $1 AND e.is_active = FALSE
                  AND e.termination_date IS NOT NULL
                  AND e.termination_date >= $2 AND e.termination_date <= $3
            ) sub
            GROUP BY bucket
            ORDER BY CASE bucket
                WHEN '< 3 months' THEN 1
                WHEN '3-6 months' THEN 2
                WHEN '6-12 months' THEN 3
                WHEN '1-2 years' THEN 4
                WHEN '2-5 years' THEN 5
                WHEN '> 5 years' THEN 6
                ELSE 7
            END
            "#,
        )
        .bind::<UuidType, _>(workspace_id)
        .bind::<Date, _>(start_date)
        .bind::<Date, _>(end_date)
        .load(&mut conn)
        .await
        .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        // 3. By recruitment source
        let source_rows: Vec<RecruitmentSourceBreakdownRow> = diesel::sql_query(
            r#"
            SELECT
                COALESCE(rs.source_type::text, 'direct') AS source_type,
                COALESCE(rs.name, 'Direct Hire') AS source_name,
                COUNT(*) AS count
            FROM employees e
            LEFT JOIN recruitment_sources rs ON e.recruitment_source_id = rs.id
            WHERE e.workspace_id = $1 AND e.is_active = FALSE
              AND e.termination_date IS NOT NULL
              AND e.termination_date >= $2 AND e.termination_date <= $3
            GROUP BY rs.source_type, rs.name
            ORDER BY count DESC
            "#,
        )
        .bind::<UuidType, _>(workspace_id)
        .bind::<Date, _>(start_date)
        .bind::<Date, _>(end_date)
        .load(&mut conn)
        .await
        .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        // 4. By department (latest department assignment before termination)
        let dept_rows: Vec<DeptRow> = diesel::sql_query(
            r#"
            SELECT
                COALESCE(d.name, 'Unassigned') AS department_name,
                COUNT(*) AS count
            FROM employees e
            LEFT JOIN LATERAL (
                SELECT edh.department_id
                FROM employee_department_history edh
                WHERE edh.employee_id = e.id
                ORDER BY edh.effective_date DESC
                LIMIT 1
            ) latest_dept ON TRUE
            LEFT JOIN departments d ON d.id = latest_dept.department_id
            WHERE e.workspace_id = $1 AND e.is_active = FALSE
              AND e.termination_date IS NOT NULL
              AND e.termination_date >= $2 AND e.termination_date <= $3
            GROUP BY d.name
            ORDER BY count DESC
            "#,
        )
        .bind::<UuidType, _>(workspace_id)
        .bind::<Date, _>(start_date)
        .bind::<Date, _>(end_date)
        .load(&mut conn)
        .await
        .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(TurnoverAnalytics {
            tenure_distribution: tenure_rows
                .into_iter()
                .map(|r| TurnoverTenureBucket {
                    bucket: r.bucket,
                    count: r.count,
                })
                .collect(),
            by_source: source_rows
                .into_iter()
                .map(|r| TurnoverBySource {
                    source_type: r.source_type,
                    source_name: r.source_name,
                    count: r.count,
                })
                .collect(),
            by_department: dept_rows
                .into_iter()
                .map(|r| TurnoverByDepartment {
                    department_name: r.department_name,
                    count: r.count,
                })
                .collect(),
            total_terminated: total,
            period_start: start_date.to_string(),
            period_end: end_date.to_string(),
        })
    }

    async fn get_managed_departments(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        user_id: Uuid,
    ) -> Result<Vec<Uuid>, AppError> {
        let workspace_id = auth_token.workspace_id();
        use crate::schema::employee_department_history;

        let mut conn = self
            .diesel_pools
            .tenants
            .get_connection(&workspace_id)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        let result: Vec<Uuid> = employees::table
            .inner_join(employee_department_history::table)
            .select(employee_department_history::department_id)
            .filter(employees::workspace_id.eq(workspace_id))
            .filter(employees::user_id.eq(user_id))
            .distinct()
            .load(&mut conn)
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
        let mut conn = self
            .diesel_pools
            .tenants
            .get_connection(&workspace_id)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        let result: Option<Uuid> = employees::table
            .select(employees::id)
            .filter(employees::workspace_id.eq(workspace_id))
            .filter(employees::user_id.eq(user_id))
            .filter(employees::is_active.eq(true))
            .order(employees::updated_at.desc())
            .first(&mut conn)
            .await
            .optional()
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(result)
    }

    // --- ESS (Employee Self Service) ---

    async fn get_employee_profile(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        employee_id: Uuid,
    ) -> Result<Option<EmployeeProfile>, AppError> {
        let workspace_id = auth_token.workspace_id();
        let mut conn = self
            .diesel_pools
            .tenants
            .get_connection(&workspace_id)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        let result: Option<EmployeeProfileEntity> = employee_profiles::table
            .inner_join(employees::table)
            .select(EmployeeProfileEntity::as_select())
            .filter(employee_profiles::employee_id.eq(employee_id))
            .filter(employees::workspace_id.eq(workspace_id))
            .first(&mut conn)
            .await
            .optional()
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(result.map(Into::into))
    }

    async fn upsert_employee_profile(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        employee_id: Uuid,
        req: &UpdateEmployeeProfileRequest,
    ) -> Result<EmployeeProfile, AppError> {
        let workspace_id = auth_token.workspace_id();
        let mut conn = self
            .diesel_pools
            .tenants
            .get_connection(&workspace_id)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        let result: EmployeeProfileEntity = diesel::insert_into(employee_profiles::table)
            .values((
                employee_profiles::employee_id.eq(employee_id),
                employee_profiles::phone.eq(&req.phone),
                employee_profiles::address.eq(&req.address),
                employee_profiles::date_of_birth.eq(req.date_of_birth),
                employee_profiles::gender.eq(&req.gender),
                employee_profiles::national_id.eq(&req.national_id),
                employee_profiles::bank_name.eq(&req.bank_name),
                employee_profiles::bank_account_number.eq(&req.bank_account_number),
                employee_profiles::bank_branch.eq(&req.bank_branch),
            ))
            .on_conflict(employee_profiles::employee_id)
            .do_update()
            .set((
                employee_profiles::phone.eq(diesel::dsl::sql::<
                    diesel::sql_types::Nullable<diesel::sql_types::Text>,
                >(
                    "COALESCE(EXCLUDED.phone, employee_profiles.phone)"
                )),
                employee_profiles::address.eq(diesel::dsl::sql::<
                    diesel::sql_types::Nullable<diesel::sql_types::Text>,
                >(
                    "COALESCE(EXCLUDED.address, employee_profiles.address)",
                )),
                employee_profiles::date_of_birth.eq(diesel::dsl::sql::<
                    diesel::sql_types::Nullable<diesel::sql_types::Date>,
                >(
                    "COALESCE(EXCLUDED.date_of_birth, employee_profiles.date_of_birth)",
                )),
                employee_profiles::gender.eq(diesel::dsl::sql::<
                    diesel::sql_types::Nullable<diesel::sql_types::Text>,
                >(
                    "COALESCE(EXCLUDED.gender, employee_profiles.gender)",
                )),
                employee_profiles::national_id.eq(diesel::dsl::sql::<
                    diesel::sql_types::Nullable<diesel::sql_types::Text>,
                >(
                    "COALESCE(EXCLUDED.national_id, employee_profiles.national_id)",
                )),
                employee_profiles::bank_name.eq(diesel::dsl::sql::<
                    diesel::sql_types::Nullable<diesel::sql_types::Text>,
                >(
                    "COALESCE(EXCLUDED.bank_name, employee_profiles.bank_name)",
                )),
                employee_profiles::bank_account_number.eq(diesel::dsl::sql::<
                    diesel::sql_types::Nullable<diesel::sql_types::Text>,
                >(
                    "COALESCE(EXCLUDED.bank_account_number, employee_profiles.bank_account_number)",
                )),
                employee_profiles::bank_branch.eq(diesel::dsl::sql::<
                    diesel::sql_types::Nullable<diesel::sql_types::Text>,
                >(
                    "COALESCE(EXCLUDED.bank_branch, employee_profiles.bank_branch)",
                )),
                employee_profiles::updated_at.eq(chrono::Utc::now()),
            ))
            .get_result(&mut conn)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(result.into())
    }

    async fn get_emergency_contacts(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        employee_id: Uuid,
    ) -> Result<Vec<EmployeeEmergencyContact>, AppError> {
        let workspace_id = auth_token.workspace_id();
        let mut conn = self
            .diesel_pools
            .tenants
            .get_connection(&workspace_id)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        let results: Vec<EmployeeEmergencyContactEntity> = employee_emergency_contacts::table
            .inner_join(employees::table)
            .select(EmployeeEmergencyContactEntity::as_select())
            .filter(employee_emergency_contacts::employee_id.eq(employee_id))
            .filter(employees::workspace_id.eq(workspace_id))
            .order((
                employee_emergency_contacts::is_primary.desc(),
                employee_emergency_contacts::created_at.asc(),
            ))
            .load(&mut conn)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(results.into_iter().map(Into::into).collect())
    }

    async fn upsert_emergency_contact(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        employee_id: Uuid,
        req: &CreateEmergencyContactRequest,
    ) -> Result<EmployeeEmergencyContact, AppError> {
        let workspace_id = auth_token.workspace_id();
        let mut conn = self
            .diesel_pools
            .tenants
            .get_connection(&workspace_id)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        let is_primary = req.is_primary.unwrap_or(false);

        conn.transaction::<_, AppError, _>(|mut conn| {
            Box::pin(async move {
                if is_primary {
                    diesel::update(employee_emergency_contacts::table)
                        .filter(employee_emergency_contacts::employee_id.eq(employee_id))
                        .set(employee_emergency_contacts::is_primary.eq(false))
                        .execute(&mut conn)
                        .await
                        .map_err(|e| AppError::Internal(format!("db: {e}")))?;
                }

                let result: EmployeeEmergencyContactEntity =
                    diesel::insert_into(employee_emergency_contacts::table)
                        .values((
                            employee_emergency_contacts::id.eq(Uuid::new_v4()),
                            employee_emergency_contacts::employee_id.eq(employee_id),
                            employee_emergency_contacts::contact_name.eq(&req.contact_name),
                            employee_emergency_contacts::relationship.eq(&req.relationship),
                            employee_emergency_contacts::phone.eq(&req.phone),
                            employee_emergency_contacts::is_primary.eq(is_primary),
                        ))
                        .get_result(&mut conn)
                        .await
                        .map_err(|e| AppError::Internal(format!("db: {e}")))?;

                Ok(result.into())
            })
        })
        .await
    }

    async fn delete_emergency_contact(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        employee_id: Uuid,
        contact_id: Uuid,
    ) -> Result<u64, AppError> {
        let workspace_id = auth_token.workspace_id();
        let mut conn = self
            .diesel_pools
            .tenants
            .get_connection(&workspace_id)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        let affected = diesel::delete(employee_emergency_contacts::table)
            .filter(employee_emergency_contacts::id.eq(contact_id))
            .filter(employee_emergency_contacts::employee_id.eq(employee_id))
            .filter(diesel::dsl::exists(
                employees::table
                    .filter(employees::id.eq(employee_id))
                    .filter(employees::workspace_id.eq(workspace_id)),
            ))
            .execute(&mut conn)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(affected as u64)
    }

    async fn get_employee_documents(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        employee_id: Uuid,
    ) -> Result<Vec<EmployeeDocument>, AppError> {
        let workspace_id = auth_token.workspace_id();
        let mut conn = self
            .diesel_pools
            .tenants
            .get_connection(&workspace_id)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        let results: Vec<EmployeeDocumentEntity> = employee_documents::table
            .select(EmployeeDocumentEntity::as_select())
            .filter(employee_documents::employee_id.eq(employee_id))
            .filter(employee_documents::workspace_id.eq(workspace_id))
            .order(employee_documents::created_at.desc())
            .load(&mut conn)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(results.into_iter().map(Into::into).collect())
    }

    async fn create_employee_document(&self, doc: &EmployeeDocument) -> Result<(), AppError> {
        let workspace_id = doc.workspace_id;
        let mut conn = self
            .diesel_pools
            .tenants
            .get_connection(&workspace_id)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        let entity: EmployeeDocumentEntity = doc.clone().into();

        diesel::insert_into(employee_documents::table)
            .values(&entity)
            .execute(&mut conn)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(())
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::diesel_pool::DieselPools;

    #[tokio::test]
    async fn test_repo_initialization() {
        let test_db_url = std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgres://user:pass@localhost:5432/leeattend_test_db".to_string());
        
        let pools = DieselPools::for_test(&test_db_url);
        let _repo = DieselEmployeeRepository::new(pools);
        // DI connection test passes if it compiles
    }
}
