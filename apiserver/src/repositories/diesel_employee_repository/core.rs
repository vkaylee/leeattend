use crate::errors::AppError;
use crate::utils::i18n::I18nKey;
use async_trait::async_trait;
use chrono::NaiveDate;
use diesel::prelude::*;
use diesel_async::AsyncConnection;
use diesel_async::RunQueryDsl;
use uuid::Uuid;

use crate::models::diesel::employee::{
    CountRow, EmployeeEntity, EmployeeRow, UpdateDieselEmployee,
};
use crate::models::{
    Employee, EmployeeFilterOptions, EmployeePaginationArgs, UpdateEmployeeRequest,
};
use crate::repositories::employee::*;
use crate::schema::employees;
use diesel::sql_types::Uuid as UuidType;

use super::{
    build_employee_search_sql, DieselEmployeeRepository, EMPLOYEE_DEPT_JOIN, EMPLOYEE_SELECT_QUERY,
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
            "{EMPLOYEE_SELECT_QUERY} WHERE e.workspace_id = $1 AND e.employee_id_number = $2 AND e.deleted_at IS NULL"
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

        let query = format!("{EMPLOYEE_SELECT_QUERY} WHERE e.id = $1 AND e.workspace_id = $2 AND e.deleted_at IS NULL");
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

        let query = format!("{EMPLOYEE_SELECT_QUERY} WHERE e.workspace_id = $1 AND e.id = ANY($2) AND e.deleted_at IS NULL");
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
        use crate::repositories::workspace_repository::WorkspaceRepository;
        use diesel::sql_types::{Array, BigInt, Nullable, Text};

        let user_ids = if let Some(ref permission) = options.permission {
            let workspace_repo =
                crate::repositories::diesel_workspace_repository::DieselWorkspaceRepository::new(
                    self.diesel_pools.clone(),
                );
            let ids = workspace_repo
                .get_users_with_permission(workspace_id, permission)
                .await?;
            if ids.is_empty() {
                return Ok((vec![], 0));
            }
            Some(ids)
        } else {
            None
        };

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
            " AND ({count_dept_param}::uuid[] IS NULL OR (\
                SELECT department_id FROM employments emp \
                WHERE emp.employee_id = e.id AND emp.deleted_at IS NULL \
                ORDER BY emp.start_date DESC, emp.id DESC LIMIT 1 \
            ) = ANY({count_dept_param}))"
        );

        let perm_filter = if user_ids.is_some() {
            let count_perm_param = if search_sql.has_search { "$4" } else { "$3" };
            format!(" AND e.user_id = ANY({count_perm_param})")
        } else {
            String::new()
        };

        let where_clause = &search_sql.where_clause;
        let count_sql = format!(
            "SELECT COUNT(*)::bigint as count FROM employees e {where_clause}{dept_filter}{perm_filter}"
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

        let total_row: CountRow = match (search_sql.has_search, &user_ids) {
            (true, Some(ids)) => diesel::sql_query(&count_sql)
                .bind::<UuidType, _>(workspace_id)
                .bind::<Nullable<Text>, _>(count_search_param)
                .bind::<Nullable<Array<UuidType>>, _>(dept_ids)
                .bind::<Array<UuidType>, _>(ids)
                .get_result(&mut *conn)
                .await
                .map_err(|e| AppError::Internal(format!("db: {e}")))?,
            (true, None) => diesel::sql_query(&count_sql)
                .bind::<UuidType, _>(workspace_id)
                .bind::<Nullable<Text>, _>(count_search_param)
                .bind::<Nullable<Array<UuidType>>, _>(dept_ids)
                .get_result(&mut *conn)
                .await
                .map_err(|e| AppError::Internal(format!("db: {e}")))?,
            (false, Some(ids)) => diesel::sql_query(&count_sql)
                .bind::<UuidType, _>(workspace_id)
                .bind::<Nullable<Array<UuidType>>, _>(dept_ids)
                .bind::<Array<UuidType>, _>(ids)
                .get_result(&mut *conn)
                .await
                .map_err(|e| AppError::Internal(format!("db: {e}")))?,
            (false, None) => diesel::sql_query(&count_sql)
                .bind::<UuidType, _>(workspace_id)
                .bind::<Nullable<Array<UuidType>>, _>(dept_ids)
                .get_result(&mut *conn)
                .await
                .map_err(|e| AppError::Internal(format!("db: {e}")))?,
        };

        let total = total_row.count;

        let data_dept_filter = " AND ($6::uuid[] IS NULL OR (\
            SELECT department_id FROM employments emp \
            WHERE emp.employee_id = e.id AND emp.deleted_at IS NULL \
            ORDER BY emp.start_date DESC, emp.id DESC LIMIT 1 \
        ) = ANY($6))";

        let data_perm_filter = if user_ids.is_some() {
            " AND e.user_id = ANY($7)"
        } else {
            ""
        };

        let dept_join = if search_sql.needs_dept_join {
            EMPLOYEE_DEPT_JOIN
        } else {
            ""
        };

        let where_clause = &search_sql.where_clause;
        let order_clause = &search_sql.order_clause;
        let sql = format!(
            "{EMPLOYEE_SELECT_QUERY} {dept_join} {where_clause}{data_dept_filter}{data_perm_filter} {order_clause} LIMIT $3 OFFSET $4"
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

        let employee_rows: Vec<EmployeeRow> = if let Some(ref ids) = user_ids {
            diesel::sql_query(&sql)
                .bind::<UuidType, _>(workspace_id)
                .bind::<Nullable<Text>, _>(search_str_1)
                .bind::<BigInt, _>(options.limit)
                .bind::<BigInt, _>(offset)
                .bind::<Nullable<Text>, _>(search_str_2)
                .bind::<Nullable<Array<UuidType>>, _>(options.department_ids)
                .bind::<Array<UuidType>, _>(ids)
                .load(&mut *conn)
                .await
        } else {
            diesel::sql_query(&sql)
                .bind::<UuidType, _>(workspace_id)
                .bind::<Nullable<Text>, _>(search_str_1)
                .bind::<BigInt, _>(options.limit)
                .bind::<BigInt, _>(offset)
                .bind::<Nullable<Text>, _>(search_str_2)
                .bind::<Nullable<Array<UuidType>>, _>(options.department_ids)
                .load(&mut *conn)
                .await
        }
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
            "{EMPLOYEE_SELECT_QUERY} WHERE e.workspace_id = $1 AND EXISTS (SELECT 1 FROM employments em WHERE em.employee_id = e.id AND em.status = 'active' AND em.deleted_at IS NULL) AND e.deleted_at IS NULL ORDER BY e.full_name ASC"
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

        use crate::schema::employments;
        let mut query = employees::table
            .select(employees::id)
            .filter(employees::workspace_id.eq(workspace_id))
            .filter(employees::deleted_at.is_null())
            .filter(diesel::dsl::exists(
                employments::table
                    .filter(employments::employee_id.eq(employees::id))
                    .filter(employments::status.eq("active"))
                    .filter(employments::deleted_at.is_null()),
            ))
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
        use crate::schema::employments;

        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .tenants_tx
                .get_connection(&workspace_id)
                .await
                .map_err(|e| AppError::Internal(format!("db: {e}")))?,
        );

        let mut count_query = employees::table
            .filter(employees::workspace_id.eq(workspace_id))
            .filter(employees::deleted_at.is_null())
            .into_boxed();

        let status = args.status_filter.as_deref().unwrap_or("active");
        match status {
            "active" => {
                if let Some((start, end)) = args.active_during {
                    count_query = count_query.filter(diesel::dsl::exists(
                        employments::table
                            .filter(employments::employee_id.eq(employees::id))
                            .filter(employments::deleted_at.is_null())
                            .filter(employments::start_date.le(end))
                            .filter(
                                employments::status.eq("active").or(employments::status
                                    .eq("terminated")
                                    .and(employments::end_date.ge(start))),
                            ),
                    ));
                } else {
                    count_query = count_query.filter(diesel::dsl::exists(
                        employments::table
                            .filter(employments::employee_id.eq(employees::id))
                            .filter(employments::deleted_at.is_null())
                            .filter(employments::status.eq("active")),
                    ));
                }
            }
            "terminated" => {
                count_query = count_query.filter(diesel::dsl::exists(
                    employments::table
                        .filter(employments::employee_id.eq(employees::id))
                        .filter(employments::deleted_at.is_null())
                        .filter(employments::status.eq("terminated")),
                ));
            }
            "all" => {
                count_query = count_query.filter(diesel::dsl::exists(
                    employments::table
                        .filter(employments::employee_id.eq(employees::id))
                        .filter(employments::deleted_at.is_null()),
                ));
            }
            _ => {}
        }

        if let Some(dept_id) = args.department_id {
            if let Some((start, end)) = args.active_during {
                count_query = count_query.filter(diesel::dsl::exists(
                    employments::table
                        .filter(employments::employee_id.eq(employees::id))
                        .filter(employments::deleted_at.is_null())
                        .filter(employments::department_id.eq(dept_id))
                        .filter(employments::start_date.le(end))
                        .filter(
                            employments::status.eq("active").or(employments::status
                                .eq("terminated")
                                .and(employments::end_date.ge(start))),
                        ),
                ));
            } else {
                count_query = count_query.filter(diesel::dsl::exists(
                    employments::table
                        .filter(employments::employee_id.eq(employees::id))
                        .filter(employments::deleted_at.is_null())
                        .filter(employments::department_id.eq(dept_id))
                        .filter(employments::status.eq("active")),
                ));
            }
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
            .filter(employees::deleted_at.is_null())
            .into_boxed();

        match status {
            "active" => {
                if let Some((start, end)) = args.active_during {
                    data_query = data_query.filter(diesel::dsl::exists(
                        employments::table
                            .filter(employments::employee_id.eq(employees::id))
                            .filter(employments::deleted_at.is_null())
                            .filter(employments::start_date.le(end))
                            .filter(
                                employments::status.eq("active").or(employments::status
                                    .eq("terminated")
                                    .and(employments::end_date.ge(start))),
                            ),
                    ));
                } else {
                    data_query = data_query.filter(diesel::dsl::exists(
                        employments::table
                            .filter(employments::employee_id.eq(employees::id))
                            .filter(employments::deleted_at.is_null())
                            .filter(employments::status.eq("active")),
                    ));
                }
            }
            "terminated" => {
                data_query = data_query.filter(diesel::dsl::exists(
                    employments::table
                        .filter(employments::employee_id.eq(employees::id))
                        .filter(employments::deleted_at.is_null())
                        .filter(employments::status.eq("terminated")),
                ));
            }
            "all" => {
                data_query = data_query.filter(diesel::dsl::exists(
                    employments::table
                        .filter(employments::employee_id.eq(employees::id))
                        .filter(employments::deleted_at.is_null()),
                ));
            }
            _ => {}
        }

        if let Some(dept_id) = args.department_id {
            if let Some((start, end)) = args.active_during {
                data_query = data_query.filter(diesel::dsl::exists(
                    employments::table
                        .filter(employments::employee_id.eq(employees::id))
                        .filter(employments::deleted_at.is_null())
                        .filter(employments::department_id.eq(dept_id))
                        .filter(employments::start_date.le(end))
                        .filter(
                            employments::status.eq("active").or(employments::status
                                .eq("terminated")
                                .and(employments::end_date.ge(start))),
                        ),
                ));
            } else {
                data_query = data_query.filter(diesel::dsl::exists(
                    employments::table
                        .filter(employments::employee_id.eq(employees::id))
                        .filter(employments::deleted_at.is_null())
                        .filter(employments::department_id.eq(dept_id))
                        .filter(employments::status.eq("active")),
                ));
            }
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
            WITH latest_history AS (
                SELECT employee_id, department_id,
                       ROW_NUMBER() OVER (PARTITION BY employee_id ORDER BY start_date DESC) as rn
                FROM employments
                WHERE deleted_at IS NULL AND start_date <= $2 AND (end_date IS NULL OR end_date >= $2)
            )
            SELECT e.id AS employee_id,
                   lh.department_id AS department_id,
                   COALESCE(d.name, 'Unassigned') AS department_name
            FROM employees e
            LEFT JOIN latest_history lh ON lh.employee_id = e.id AND lh.rn = 1
            LEFT JOIN departments d ON d.id = lh.department_id
            WHERE EXISTS (SELECT 1 FROM employments em WHERE em.employee_id = e.id AND em.status = 'active' AND em.deleted_at IS NULL) AND e.workspace_id = $1 AND e.deleted_at IS NULL
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
        self.update_employee_transactional(id, auth_token, req)
            .await
    }

    async fn update_employee_transactional(
        &self,
        id: Uuid,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        req: &UpdateEmployeeRequest,
    ) -> Result<Option<Employee>, AppError> {
        let workspace_id = auth_token.workspace_id();
        let mut conn = self
            .diesel_pools
            .tenant_transaction_conn(auth_token)
            .await?;

        conn.transaction(|conn| {
            Box::pin(async move {
                if let Some(Some(new_user_id)) = req.user_id {
                    if req.force_transfer == Some(true) {
                        diesel::update(employees::table)
                            .filter(employees::user_id.eq(new_user_id))
                            .filter(employees::workspace_id.eq(workspace_id))
                            .filter(employees::id.ne(id))
                            .set(employees::user_id.eq(None::<Uuid>))
                            .execute(&mut *conn)
                            .await
                            .map_err(|e| AppError::Internal(format!("db: {e}")))?;
                    }
                }

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
                    updated_at: Some(chrono::Utc::now()),
                };

                let result: Option<EmployeeEntity> = diesel::update(employees::table)
                    .filter(employees::id.eq(id))
                    .filter(employees::workspace_id.eq(workspace_id))
                    .set(&update_model)
                    .get_result(&mut *conn)
                    .await
                    .optional()
                    .map_err(|e| AppError::Internal(format!("db: {e}")))?;

                if req.recruitment_source_id.is_some() {
                    diesel::update(employees::table)
                        .filter(employees::id.eq(id))
                        .filter(employees::workspace_id.eq(workspace_id))
                        .set((
                            employees::recruitment_source_id.eq(req.recruitment_source_id),
                            employees::updated_at.eq(Some(chrono::Utc::now())),
                        ))
                        .execute(&mut *conn)
                        .await
                        .map_err(|e| AppError::Internal(format!("db: {e}")))?;
                }

                Ok(result.map(Into::into))
            })
        })
        .await
    }

    async fn create_employee(&self, employee: &Employee) -> Result<(), AppError> {
        let auth_token =
            crate::diesel_pool::TenantAuthToken::system_override(employee.workspace_id());
        let workspace_id = employee.workspace_id();
        let mut conn = self
            .diesel_pools
            .tenant_transaction_conn(&auth_token)
            .await?;
        use crate::schema::employers;
        let emp_id_opt: Option<Uuid> = diesel_async::RunQueryDsl::first(
            employers::table
                .filter(employers::workspace_id.eq(workspace_id))
                .select(employers::id),
            &mut conn,
        )
        .await
        .optional()
        .map_err(|e| AppError::Internal(format!("db select first employer: {e}")))?;
        let employer_id = emp_id_opt.unwrap_or_else(Uuid::new_v4);

        self.create_employee_transactional(
            &auth_token,
            employee,
            employee.created_at().date_naive(),
            None,
            employer_id,
            None,
            None,
        )
        .await
    }

    async fn create_employee_transactional(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        employee: &Employee,
        hire_date: chrono::NaiveDate,
        recruitment_source_id: Option<Uuid>,
        employer_id: Uuid,
        primary_location_id: Option<Uuid>,
        department_id: Option<Uuid>,
    ) -> Result<(), AppError> {
        let workspace_id = auth_token.workspace_id();
        if employee.workspace_id() != workspace_id {
            return Err(AppError::forbidden(I18nKey::Forbidden));
        }

        let mut conn = self
            .diesel_pools
            .tenant_transaction_conn(auth_token)
            .await?;

        let mut new_entity: EmployeeEntity = employee.clone().into();
        new_entity.recruitment_source_id = recruitment_source_id;

        conn.transaction(|conn| {
            Box::pin(async move {
                diesel::insert_into(crate::schema::employees::table)
                    .values(&new_entity)
                    .execute(&mut *conn)
                    .await
                    .map_err(|e| AppError::Internal(format!("db employee insert: {e}")))?;

                use crate::schema::employee_contracts;
                use crate::schema::employments;

                let employment_id = Uuid::new_v4();
                let new_emp = crate::models::diesel::employment::EmploymentEntity {
                    id: employment_id,
                    workspace_id,
                    employee_id: employee.id(),
                    employer_id,
                    primary_location_id,
                    department_id,
                    status: "active".to_string(),
                    start_date: hire_date,
                    end_date: None,
                    created_at: chrono::Utc::now(),
                    updated_at: chrono::Utc::now(),
                    deleted_at: None,
                    job_title: None,
                    employment_type: "full_time".to_string(),
                    manager_employment_id: None,
                    notes: None,
                };

                diesel::insert_into(employments::table)
                    .values(&new_emp)
                    .execute(&mut *conn)
                    .await
                    .map_err(|e| AppError::Internal(format!("db insert employment: {e}")))?;

                let default_contract =
                    crate::models::diesel::employee_contract::EmployeeContractEntity {
                        id: Uuid::new_v4(),
                        workspace_id,
                        contract_number: None,
                        contract_type: crate::models::enums::ContractType::Indefinite,
                        status: crate::models::enums::ContractStatus::Active,
                        start_date: hire_date,
                        end_date: None,
                        document_id: None,
                        notes: Some(
                            "Hệ thống tự động khởi tạo thỏa thuận làm việc khi onboard".to_string(),
                        ),
                        created_at: chrono::Utc::now(),
                        updated_at: chrono::Utc::now(),
                        deleted_at: None,
                        employment_id,
                    };

                diesel::insert_into(employee_contracts::table)
                    .values(&default_contract)
                    .execute(&mut *conn)
                    .await
                    .map_err(|e| AppError::Internal(format!("db contract insert: {e}")))?;

                Ok::<(), AppError>(())
            })
        })
        .await
    }

    async fn create_employee_skip_duplicates(&self, employee: &Employee) -> Result<bool, AppError> {
        let workspace_id = employee.workspace_id();
        use crate::schema::employees;

        let auth_token = crate::diesel_pool::TenantAuthToken::system_override(workspace_id);
        if self
            .get_employee_by_id_number(employee.employee_id_number(), &auth_token)
            .await?
            .is_some()
        {
            return Ok(false);
        }

        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .tenants_tx
                .get_connection(&workspace_id)
                .await
                .map_err(|e| AppError::Internal(format!("db: {e}")))?,
        );

        let new_entity: EmployeeEntity = employee.clone().into();

        conn.transaction(|conn| {
            Box::pin(async move {
                diesel::insert_into(employees::table)
                    .values(&new_entity)
                    .execute(&mut *conn)
                    .await
                    .map_err(|e| AppError::Internal(format!("db employee insert: {e}")))?;

                use crate::schema::employers;
                let emp_id_opt: Option<Uuid> = diesel_async::RunQueryDsl::first(
                    employers::table
                        .filter(employers::workspace_id.eq(workspace_id))
                        .select(employers::id),
                    &mut *conn,
                )
                .await
                .optional()
                .map_err(|e| AppError::Internal(format!("db select first employer: {e}")))?;
                let emp_id = emp_id_opt.unwrap_or_else(Uuid::new_v4);

                use crate::schema::employee_contracts;
                use crate::schema::employments;

                let employment_id = Uuid::new_v4();
                let new_emp = crate::models::diesel::employment::EmploymentEntity {
                    id: employment_id,
                    workspace_id,
                    employee_id: employee.id(),
                    employer_id: emp_id,
                    primary_location_id: None,
                    department_id: None,
                    status: "active".to_string(),
                    start_date: employee.created_at().date_naive(),
                    end_date: None,
                    created_at: chrono::Utc::now(),
                    updated_at: chrono::Utc::now(),
                    deleted_at: None,
                    job_title: None,
                    employment_type: "full_time".to_string(),
                    manager_employment_id: None,
                    notes: None,
                };

                diesel::insert_into(employments::table)
                    .values(&new_emp)
                    .execute(&mut *conn)
                    .await
                    .map_err(|e| AppError::Internal(format!("db insert employment: {e}")))?;

                let default_contract =
                    crate::models::diesel::employee_contract::EmployeeContractEntity {
                        id: Uuid::new_v4(),
                        workspace_id,
                        contract_number: None,
                        contract_type: crate::models::enums::ContractType::Indefinite,
                        status: crate::models::enums::ContractStatus::Active,
                        start_date: employee.created_at().date_naive(),
                        end_date: None,
                        document_id: None,
                        notes: Some(
                            "Hệ thống tự động khởi tạo thỏa thuận làm việc khi onboard".to_string(),
                        ),
                        created_at: chrono::Utc::now(),
                        updated_at: chrono::Utc::now(),
                        deleted_at: None,
                        employment_id,
                    };

                diesel::insert_into(employee_contracts::table)
                    .values(&default_contract)
                    .execute(&mut *conn)
                    .await
                    .map_err(|e| AppError::Internal(format!("db contract insert: {e}")))?;

                Ok::<(), AppError>(())
            })
        })
        .await?;

        Ok(true)
    }

    async fn rehire_employee(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        employee_id: Uuid,
        hire_date: chrono::NaiveDate,
        employer_id: Uuid,
        primary_location_id: Option<Uuid>,
        department_id: Option<Uuid>,
    ) -> Result<Option<Employee>, AppError> {
        let workspace_id = auth_token.workspace_id();
        let mut conn = self
            .diesel_pools
            .tenant_transaction_conn(auth_token)
            .await?;

        conn.transaction(|conn| {
            Box::pin(async move {
                let employee_exists: bool = employees::table
                    .filter(employees::id.eq(employee_id))
                    .filter(employees::workspace_id.eq(workspace_id))
                    .select(diesel::dsl::sql::<diesel::sql_types::Bool>("1"))
                    .first::<bool>(&mut *conn)
                    .await
                    .optional()
                    .map_err(|e| AppError::Internal(format!("db select employee: {e}")))?
                    .is_some();

                if !employee_exists {
                    return Ok::<Option<Employee>, AppError>(None);
                }

                use crate::schema::employments;

                let previous_employments = diesel_async::RunQueryDsl::load::<crate::models::diesel::employment::EmploymentEntity>(
                    employments::table
                        .filter(employments::employee_id.eq(employee_id))
                        .filter(employments::deleted_at.is_null()),
                    &mut *conn,
                )
                .await
                .map_err(|e| AppError::Internal(format!("db select previous employments: {e}")))?;

                for emp in &previous_employments {
                    if emp.status == "active" {
                        return Err(AppError::Validation("error.employee_already_active".into()));
                    }
                    if let Some(end_date) = emp.end_date {
                        if hire_date <= end_date {
                            return Err(AppError::Validation("error.rehire_date_invalid".into()));
                        }
                    }
                }

                let employment_id = Uuid::new_v4();
                let new_emp = crate::models::diesel::employment::EmploymentEntity {
                    id: employment_id,
                    workspace_id,
                    employee_id,
                    employer_id,
                    primary_location_id,
                    department_id,
                    status: "active".to_string(),
                    start_date: hire_date,
                    end_date: None,
                    job_title: None,
                    employment_type: "full_time".to_string(),
                    manager_employment_id: None,
                    notes: None,
                    created_at: chrono::Utc::now(),
                    updated_at: chrono::Utc::now(),
                    deleted_at: None,
                };

                diesel::insert_into(employments::table)
                    .values(&new_emp)
                    .execute(&mut *conn)
                    .await
                    .map_err(|e| AppError::Internal(format!("db insert employment: {e}")))?;

                let query = format!("{EMPLOYEE_SELECT_QUERY} WHERE e.id = $1 AND e.workspace_id = $2 AND e.deleted_at IS NULL");
                let row: Option<EmployeeRow> = diesel::sql_query(query)
                    .bind::<diesel::sql_types::Uuid, _>(employee_id)
                    .bind::<diesel::sql_types::Uuid, _>(workspace_id)
                    .get_result(&mut *conn)
                    .await
                    .optional()
                    .map_err(|e| AppError::Internal(format!("db select employee row: {e}")))?;

                Ok(row.map(Into::into))
            })
        })
        .await
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

        let affected = diesel::update(employees::table)
            .filter(employees::id.eq(id))
            .filter(employees::workspace_id.eq(workspace_id))
            .set(employees::deleted_at.eq(Some(chrono::Utc::now())))
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
            .filter(employees::deleted_at.is_null())
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
            .filter(employees::deleted_at.is_null())
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

        // 1. Get unique input numbers (case-insensitive deduplication)
        let mut unique_emps = std::collections::HashMap::new();
        for emp in new_employees {
            let key = emp.employee_id_number().to_lowercase();
            unique_emps.entry(key).or_insert(emp);
        }

        let numbers: Vec<String> = unique_emps.keys().cloned().collect();

        // 2. Find which ones already exist in the DB
        let existing = self
            .get_employee_ids_by_numbers(auth_token, &numbers)
            .await?;

        // 3. Filter out those that already exist
        let mut to_insert = Vec::new();
        for (key, emp) in &unique_emps {
            if !existing.contains_key(key) {
                let entity: EmployeeEntity = (*emp).clone().into();
                to_insert.push(entity);
            }
        }

        tracing::debug!("bulk_create_employees_skip_duplicates: workspace_id={}, new_employees_len={}, unique_emps_keys={:?}, existing={:?}, to_insert_len={}", workspace_id, new_employees.len(), unique_emps.keys().collect::<Vec<_>>(), existing, to_insert.len());

        // 4. Perform chunked insertion inside a transaction
        conn.transaction(|conn| {
            Box::pin(async move {
                if !to_insert.is_empty() {
                    for chunk in to_insert.chunks(1000) {
                        tracing::debug!("inserting chunk of size {}", chunk.len());
                        let res = diesel::insert_into(employees::table)
                            .values(chunk)
                            .execute(&mut *conn)
                            .await;
                        tracing::debug!("insert res={:?}", res);
                        res.map_err(|e| AppError::Internal(format!("db: {e}")))?;
                    }
                }

                use crate::schema::employers;
                use crate::schema::work_locations;

                let default_employer_id: Option<Uuid> = diesel_async::RunQueryDsl::first(
                    employers::table
                        .filter(employers::workspace_id.eq(workspace_id))
                        .select(employers::id),
                    &mut *conn,
                )
                .await
                .optional()
                .map_err(|e| AppError::Internal(format!("db select first employer: {e}")))?;
                let default_employer_id = default_employer_id.unwrap_or_else(Uuid::new_v4);

                let hq_id: Option<Uuid> = diesel_async::RunQueryDsl::first(
                    work_locations::table
                        .filter(work_locations::workspace_id.eq(workspace_id))
                        .filter(work_locations::is_headquarters.eq(true))
                        .select(work_locations::id),
                    &mut *conn,
                )
                .await
                .optional()
                .unwrap_or(None);

                // Insert employments & contracts for each inserted employee
                let mut employments_to_insert = Vec::new();
                let mut contracts_to_insert = Vec::new();
                for emp in unique_emps.values() {
                    if !existing.contains_key(&emp.employee_id_number().to_lowercase()) {
                        let employment_id = Uuid::new_v4();
                        employments_to_insert.push(
                            crate::models::diesel::employment::EmploymentEntity {
                                id: employment_id,
                                workspace_id,
                                employee_id: emp.id(),
                                employer_id: default_employer_id,
                                primary_location_id: hq_id,
                                department_id: None,
                                status: "active".to_string(),
                                start_date: emp.created_at().date_naive(),
                                end_date: None,
                                created_at: chrono::Utc::now(),
                                updated_at: chrono::Utc::now(),
                                deleted_at: None,
                                job_title: None,
                                employment_type: "full_time".to_string(),
                                manager_employment_id: None,
                                notes: None,
                            },
                        );
                        contracts_to_insert.push(
                            crate::models::diesel::employee_contract::EmployeeContractEntity {
                                id: Uuid::new_v4(),
                                workspace_id,
                                contract_number: None,
                                contract_type: crate::models::enums::ContractType::Indefinite,
                                status: crate::models::enums::ContractStatus::Active,
                                start_date: emp.created_at().date_naive(),
                                end_date: None,
                                document_id: None,
                                notes: Some(
                                    "Hệ thống tự động khởi tạo thỏa thuận làm việc khi import"
                                        .to_string(),
                                ),
                                created_at: chrono::Utc::now(),
                                updated_at: chrono::Utc::now(),
                                deleted_at: None,
                                employment_id,
                            },
                        );
                    }
                }

                if !employments_to_insert.is_empty() {
                    use crate::schema::employments;
                    for chunk in employments_to_insert.chunks(1000) {
                        diesel::insert_into(employments::table)
                            .values(chunk)
                            .execute(&mut *conn)
                            .await
                            .map_err(|e| {
                                AppError::Internal(format!("db employments insert: {e}"))
                            })?;
                    }
                }

                if !contracts_to_insert.is_empty() {
                    for chunk in contracts_to_insert.chunks(1000) {
                        diesel::insert_into(crate::schema::employee_contracts::table)
                            .values(chunk)
                            .execute(&mut *conn)
                            .await
                            .map_err(|e| AppError::Internal(format!("db contracts insert: {e}")))?;
                    }
                }

                Ok::<(), AppError>(())
            })
        })
        .await?;

        Ok(())
    }

    async fn get_managed_departments(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        user_id: Uuid,
    ) -> Result<Vec<Uuid>, AppError> {
        let workspace_id = auth_token.workspace_id();
        use crate::schema::employments;

        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .tenants_tx
                .get_connection(&workspace_id)
                .await
                .map_err(|e| AppError::Internal(format!("db: {e}")))?,
        );

        let result: Vec<Option<Uuid>> = employees::table
            .inner_join(employments::table)
            .select(employments::department_id)
            .filter(employees::workspace_id.eq(workspace_id))
            .filter(employees::deleted_at.is_null())
            .filter(employments::deleted_at.is_null())
            .filter(employees::user_id.eq(user_id))
            .distinct()
            .load(&mut *conn)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(result.into_iter().flatten().collect())
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

        use crate::schema::employments;
        let result: Option<Uuid> = employees::table
            .select(employees::id)
            .filter(employees::workspace_id.eq(workspace_id))
            .filter(employees::deleted_at.is_null())
            .filter(employees::user_id.eq(user_id))
            .filter(diesel::dsl::exists(
                employments::table
                    .filter(employments::employee_id.eq(employees::id))
                    .filter(employments::status.eq("active"))
                    .filter(employments::deleted_at.is_null()),
            ))
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

        let query = r"
            SELECT wl.timezone as timezone
            FROM employments emp
            JOIN work_locations wl ON wl.id = emp.primary_location_id
            WHERE (emp.employee_id = $1 OR emp.id = $1) AND emp.workspace_id = $2
            ORDER BY CASE WHEN emp.status = 'active' THEN 0 ELSE 1 END, emp.start_date DESC
            LIMIT 1
        ";

        let row: TzRow = diesel::sql_query(query)
            .bind::<diesel::sql_types::Uuid, _>(employee_id)
            .bind::<diesel::sql_types::Uuid, _>(workspace_id)
            .get_result(&mut *conn)
            .await
            .map_err(|e| match e {
                diesel::result::Error::NotFound => AppError::not_found(I18nKey::EmployeeNotFound),
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

        let query = r"
            SELECT COALESCE(
                (SELECT host_wl.timezone
                 FROM employee_assignments ea
                 JOIN work_locations host_wl ON host_wl.id = ea.host_location_id
                 WHERE ea.employment_id = emp.id AND ea.status = 'active'
                   AND ea.start_date <= $3
                   AND (ea.end_date IS NULL OR ea.end_date >= $3)
                 ORDER BY ea.created_at DESC LIMIT 1),
                primary_wl.timezone
            ) as timezone
            FROM employments emp
            JOIN work_locations primary_wl ON primary_wl.id = emp.primary_location_id
            WHERE (emp.employee_id = $1 OR emp.id = $1) AND emp.workspace_id = $2
            ORDER BY CASE WHEN emp.status = 'active' THEN 0 ELSE 1 END, emp.start_date DESC
            LIMIT 1
        ";

        let row: TzRow = diesel::sql_query(query)
            .bind::<diesel::sql_types::Uuid, _>(employee_id)
            .bind::<diesel::sql_types::Uuid, _>(workspace_id)
            .bind::<diesel::sql_types::Date, _>(tentative_date)
            .get_result(&mut *conn)
            .await
            .map_err(|e| match e {
                diesel::result::Error::NotFound => AppError::not_found(I18nKey::EmployeeNotFound),
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
