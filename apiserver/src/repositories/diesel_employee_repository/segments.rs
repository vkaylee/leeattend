use super::*;
use crate::errors::AppError;
use crate::models::diesel::employee::{CountRow, EmployeeRow};
use crate::models::Employee;
use crate::repositories::employee::EmployeeBasicInfo;
use crate::schema::employees;
use diesel::prelude::*;
use diesel::sql_types::Uuid as UuidType;
use diesel_async::RunQueryDsl;
use uuid::Uuid;

impl DieselEmployeeRepository {
    #[allow(clippy::too_many_lines)]
    pub(super) async fn get_all_employees_impl(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        options: crate::models::EmployeeFilterOptions,
    ) -> Result<(Vec<Employee>, i64), AppError> {
        let workspace_id = auth_token.workspace_id();
        use crate::repositories::workspace_repository::WorkspaceRepository;
        use diesel::sql_types::{Array, BigInt, Nullable, Text};

        let user_ids = if let Some(permission) = options.permission() {
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
            options.search_query(),
            options.sort_by(),
            options.sort_dir(),
        );

        let filter_sql = if let Some(ids) = options.filter_ids() {
            if ids.is_empty() {
                " AND FALSE".to_string()
            } else {
                let ids_str: Vec<String> = ids.iter().map(|id| format!("'{id}'")).collect();
                format!(" AND e.id IN ({})", ids_str.join(","))
            }
        } else {
            String::new()
        };

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

        let status_filter = match options.status() {
            Some("active") => {
                " AND EXISTS (SELECT 1 FROM employments emp WHERE emp.employee_id = e.id AND emp.status = 'active' AND emp.deleted_at IS NULL)"
            }
            Some("terminated") => {
                " AND NOT EXISTS (SELECT 1 FROM employments emp WHERE emp.employee_id = e.id AND emp.status = 'active' AND emp.deleted_at IS NULL)"
            }
            _ => "",
        };

        let where_clause = &search_sql.where_clause;
        let count_sql = format!(
            "SELECT COUNT(*)::bigint as count FROM employees e {where_clause}{status_filter}{dept_filter}{perm_filter}{filter_sql}"
        );

        let count_search_param = if search_sql.has_search {
            let raw_q = options
                .search_query()
                .unwrap_or_default()
                .trim()
                .to_string();
            Some(format!("%{raw_q}%"))
        } else {
            None
        };

        let dept_ids = options.department_ids().map(ToOwned::to_owned);

        let total_row: CountRow = match (search_sql.has_search, &user_ids) {
            (true, Some(ids)) => diesel::sql_query(&count_sql)
                .bind::<UuidType, _>(workspace_id)
                .bind::<Nullable<Text>, _>(count_search_param)
                .bind::<Nullable<Array<UuidType>>, _>(dept_ids)
                .bind::<Array<UuidType>, _>(ids)
                .get_result(&mut *conn)
                .await
                .map_err(AppError::from)?,
            (true, None) => diesel::sql_query(&count_sql)
                .bind::<UuidType, _>(workspace_id)
                .bind::<Nullable<Text>, _>(count_search_param)
                .bind::<Nullable<Array<UuidType>>, _>(dept_ids)
                .get_result(&mut *conn)
                .await
                .map_err(AppError::from)?,
            (false, Some(ids)) => diesel::sql_query(&count_sql)
                .bind::<UuidType, _>(workspace_id)
                .bind::<Nullable<Array<UuidType>>, _>(dept_ids)
                .bind::<Array<UuidType>, _>(ids)
                .get_result(&mut *conn)
                .await
                .map_err(AppError::from)?,
            (false, None) => diesel::sql_query(&count_sql)
                .bind::<UuidType, _>(workspace_id)
                .bind::<Nullable<Array<UuidType>>, _>(dept_ids)
                .get_result(&mut *conn)
                .await
                .map_err(AppError::from)?,
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
            "{EMPLOYEE_SELECT_QUERY} {dept_join} {where_clause}{status_filter}{data_dept_filter}{data_perm_filter}{filter_sql} {order_clause} LIMIT $3 OFFSET $4"
        );

        let (search_str_1, search_str_2) = if search_sql.has_search {
            let raw_q = options
                .search_query()
                .unwrap_or_default()
                .trim()
                .to_string();
            (Some(format!("%{raw_q}%")), Some(raw_q))
        } else {
            (None, None)
        };

        let offset = (options.page() - 1) * options.limit();

        let employee_rows: Vec<EmployeeRow> = if let Some(ref ids) = user_ids {
            diesel::sql_query(&sql)
                .bind::<UuidType, _>(workspace_id)
                .bind::<Nullable<Text>, _>(search_str_1)
                .bind::<BigInt, _>(options.limit())
                .bind::<BigInt, _>(offset)
                .bind::<Nullable<Text>, _>(search_str_2)
                .bind::<Nullable<Array<UuidType>>, _>(
                    options.department_ids().map(ToOwned::to_owned),
                )
                .bind::<Array<UuidType>, _>(ids)
                .load(&mut *conn)
                .await
        } else {
            diesel::sql_query(&sql)
                .bind::<UuidType, _>(workspace_id)
                .bind::<Nullable<Text>, _>(search_str_1)
                .bind::<BigInt, _>(options.limit())
                .bind::<BigInt, _>(offset)
                .bind::<Nullable<Text>, _>(search_str_2)
                .bind::<Nullable<Array<UuidType>>, _>(
                    options.department_ids().map(ToOwned::to_owned),
                )
                .load(&mut *conn)
                .await
        }
        .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        let employees: Vec<Employee> = employee_rows.into_iter().map(Into::into).collect();

        Ok((employees, total))
    }

    #[allow(clippy::too_many_lines)]
    pub(super) async fn get_paginated_employee_ids_impl(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        args: crate::models::EmployeePaginationArgs<'_>,
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

        let status = args.status_filter().unwrap_or("active");
        match status {
            "active" => {
                if let Some((start, end)) = args.active_during() {
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

        if let Some(dept_id) = args.department_id() {
            if let Some((start, end)) = args.active_during() {
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

        if let Some(ids) = args.filter_ids() {
            count_query = count_query.filter(employees::id.eq_any(ids));
        }

        let total: i64 = count_query
            .count()
            .get_result(&mut *conn)
            .await
            .map_err(AppError::from)?;

        if total == 0 {
            return Ok((vec![], 0));
        }

        let mut data_query = employees::table
            .filter(employees::workspace_id.eq(workspace_id))
            .filter(employees::deleted_at.is_null())
            .into_boxed();

        match status {
            "active" => {
                if let Some((start, end)) = args.active_during() {
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

        if let Some(dept_id) = args.department_id() {
            if let Some((start, end)) = args.active_during() {
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

        if let Some(ids) = args.filter_ids() {
            data_query = data_query.filter(employees::id.eq_any(ids.clone()));
        }

        if args.sort_desc() {
            data_query = data_query.order(employees::created_at.desc());
        } else {
            data_query = data_query.order(employees::created_at.asc());
        }

        let offset = (args.page() - 1) * args.limit();

        let result: Vec<EmployeeBasicInfo> = data_query
            .select((
                employees::id,
                employees::full_name,
                employees::employee_id_number,
            ))
            .limit(args.limit())
            .offset(offset)
            .load(&mut *conn)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok((result, total))
    }

    pub(super) async fn get_employee_departments_at_impl(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        date: chrono::NaiveDate,
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
}
