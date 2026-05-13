use crate::errors::AppError;
use async_trait::async_trait;
use chrono::NaiveDate;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use rust_decimal::Decimal;
use uuid::Uuid;

use crate::models::diesel::employee::{EmployeeEntity, UpdateDieselEmployee};
use crate::models::{
    Employee, EmployeeFilterOptions, EmployeePaginationArgs, EmployeePayrollConfig,
    RecruitmentSource, RecruitmentSourceBreakdown, TurnoverAnalytics, UpdateEmployeeRequest,
    WorkingCondition,
};
use crate::repositories::employee::{self, EmployeeBasicInfo, EmployeeRepository};
use crate::schema::employees;

pub struct DieselEmployeeRepository {
    pub sqlx_pool: sqlx::PgPool,
    pub diesel_pools: crate::diesel_pool::DieselPools,
}

impl DieselEmployeeRepository {
    pub fn new(sqlx_pool: sqlx::PgPool, diesel_pools: crate::diesel_pool::DieselPools) -> Self {
        Self {
            sqlx_pool,
            diesel_pools,
        }
    }
}

#[async_trait]
impl EmployeeRepository for DieselEmployeeRepository {
    async fn find_by_number(
        &self,
        workspace_id: Uuid,
        employee_id_number: &str,
    ) -> Result<Option<Employee>, AppError> {
        let mut conn = self
            .diesel_pools
            .api
            .get()
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        let normalized_id = crate::utils::normalize_employee_id(employee_id_number);
        let emp: Option<EmployeeEntity> = employees::table
            .filter(employees::workspace_id.eq(workspace_id))
            .filter(employees::employee_id_number.eq(normalized_id))
            .first(&mut conn)
            .await
            .optional()
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(emp.map(Into::into))
    }

    async fn get_employee_by_id(
        &self,
        id: Uuid,
        workspace_id: Uuid,
    ) -> Result<Option<Employee>, AppError> {
        let mut conn = self
            .diesel_pools
            .api
            .get()
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        let emp: Option<EmployeeEntity> = employees::table
            .filter(employees::workspace_id.eq(workspace_id))
            .filter(employees::id.eq(id))
            .first(&mut conn)
            .await
            .optional()
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(emp.map(Into::into))
    }

    async fn get_employee_by_id_number(
        &self,
        id_number: &str,
        workspace_id: Uuid,
    ) -> Result<Option<Employee>, AppError> {
        self.find_by_number(workspace_id, id_number).await
    }

    async fn get_all_employees(
        &self,
        workspace_id: Uuid,
        options: EmployeeFilterOptions,
    ) -> Result<(Vec<Employee>, i64), AppError> {
        use crate::models::diesel::employee::{CountRow, EmployeeRow};
        use diesel::sql_types::{Array, BigInt, Nullable, Text, Uuid as UuidType};

        let mut conn = self
            .diesel_pools
            .api
            .get()
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        let search_sql = employee::build_employee_search_sql(
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

        let count_sql = format!(
            "SELECT COUNT(*)::bigint as count FROM employees e {}{}",
            search_sql.where_clause, dept_filter
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
            employee::EMPLOYEE_DEPT_JOIN
        } else {
            ""
        };

        let sql = format!(
            "{} {} {}{} {} LIMIT $3 OFFSET $4",
            employee::EMPLOYEE_SELECT_QUERY,
            dept_join,
            search_sql.where_clause,
            data_dept_filter,
            search_sql.order_clause
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

    async fn get_active_employees(&self, workspace_id: Uuid) -> Result<Vec<Employee>, AppError> {
        let mut conn = self
            .diesel_pools
            .api
            .get()
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        let emps: Vec<EmployeeEntity> = employees::table
            .filter(employees::workspace_id.eq(workspace_id))
            .filter(employees::is_active.eq(true))
            .order(employees::full_name.asc())
            .load(&mut conn)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(emps.into_iter().map(Into::into).collect())
    }

    async fn get_active_employee_ids(
        &self,
        workspace_id: Uuid,
        filter_ids: Option<&Vec<Uuid>>,
    ) -> Result<Vec<Uuid>, AppError> {
        let mut conn = self
            .diesel_pools
            .api
            .get()
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
        workspace_id: Uuid,
        args: EmployeePaginationArgs<'_>,
    ) -> Result<(Vec<EmployeeBasicInfo>, i64), AppError> {
        use crate::schema::employee_department_history;

        let mut conn = self
            .diesel_pools
            .api
            .get()
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

        if let Some(ids) = args.filter_ids.clone() {
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

    async fn update_employee(
        &self,
        id: Uuid,
        workspace_id: Uuid,
        req: &UpdateEmployeeRequest,
    ) -> Result<Option<Employee>, AppError> {
        let mut conn = self
            .diesel_pools
            .api
            .get()
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
        workspace_id: Uuid,
        employee_id: Uuid,
        hire_date: chrono::NaiveDate,
    ) -> Result<Option<Employee>, AppError> {
        let mut conn = self
            .diesel_pools
            .api
            .get()
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

    async fn delete_employee(&self, id: Uuid, workspace_id: Uuid) -> Result<u64, AppError> {
        let mut conn = self
            .diesel_pools
            .api
            .get()
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
        workspace_id: Uuid,
        employee_id: Uuid,
    ) -> Result<Vec<WorkingCondition>, AppError> {
        employee::get_working_conditions(&self.sqlx_pool, workspace_id, employee_id)
            .await
            .map_err(Into::into)
    }

    async fn create_working_condition(&self, condition: &WorkingCondition) -> Result<(), AppError> {
        employee::create_working_condition(&self.sqlx_pool, condition)
            .await
            .map_err(Into::into)
    }

    async fn delete_working_condition(
        &self,
        id: Uuid,
        employee_id: Uuid,
    ) -> Result<bool, AppError> {
        employee::delete_working_condition(&self.sqlx_pool, id, employee_id)
            .await
            .map_err(Into::into)
    }

    async fn get_existing_employee_ids_by_numbers(
        &self,
        workspace_id: Uuid,
        id_numbers: &[String],
    ) -> Result<Vec<String>, AppError> {
        employee::get_existing_employee_ids_by_numbers(&self.sqlx_pool, workspace_id, id_numbers)
            .await
            .map_err(Into::into)
    }

    async fn upsert_employee_payroll_config(
        &self,
        employee_id: Uuid,
        std_working_days: Decimal,
        hours_per_day: Decimal,
        effective_date: NaiveDate,
    ) -> Result<EmployeePayrollConfig, AppError> {
        employee::upsert_employee_payroll_config(
            &self.sqlx_pool,
            employee_id,
            std_working_days,
            hours_per_day,
            effective_date,
        )
        .await
        .map_err(Into::into)
    }

    async fn get_employee_payroll_config_at(
        &self,
        employee_id: Uuid,
        date: NaiveDate,
    ) -> Result<Option<EmployeePayrollConfig>, AppError> {
        employee::get_employee_payroll_config_at(&self.sqlx_pool, employee_id, date)
            .await
            .map_err(Into::into)
    }

    async fn set_employee_recruitment_source(
        &self,
        workspace_id: Uuid,
        employee_id: Uuid,
        recruitment_source_id: Option<Uuid>,
    ) -> Result<(), AppError> {
        employee::set_employee_recruitment_source(
            &self.sqlx_pool,
            workspace_id,
            employee_id,
            recruitment_source_id,
        )
        .await
        .map_err(Into::into)
    }

    async fn create_recruitment_source(&self, source: &RecruitmentSource) -> Result<(), AppError> {
        employee::create_recruitment_source(&self.sqlx_pool, source)
            .await
            .map_err(Into::into)
    }

    async fn get_recruitment_sources(
        &self,
        workspace_id: Uuid,
    ) -> Result<Vec<RecruitmentSource>, AppError> {
        employee::get_recruitment_sources(&self.sqlx_pool, workspace_id)
            .await
            .map_err(Into::into)
    }

    async fn get_recruitment_source_by_id(
        &self,
        id: Uuid,
        workspace_id: Uuid,
    ) -> Result<Option<RecruitmentSource>, AppError> {
        employee::get_recruitment_source_by_id(&self.sqlx_pool, id, workspace_id)
            .await
            .map_err(Into::into)
    }

    async fn update_recruitment_source(
        &self,
        id: Uuid,
        workspace_id: Uuid,
        req: &crate::models::UpdateRecruitmentSourceRequest,
    ) -> Result<Option<RecruitmentSource>, AppError> {
        employee::update_recruitment_source(&self.sqlx_pool, id, workspace_id, req)
            .await
            .map_err(Into::into)
    }

    async fn delete_recruitment_source(
        &self,
        id: Uuid,
        workspace_id: Uuid,
    ) -> Result<u64, AppError> {
        employee::delete_recruitment_source(&self.sqlx_pool, id, workspace_id)
            .await
            .map_err(Into::into)
    }

    async fn get_recruitment_source_breakdown(
        &self,
        workspace_id: Uuid,
    ) -> Result<Vec<RecruitmentSourceBreakdown>, AppError> {
        employee::get_recruitment_source_breakdown(&self.sqlx_pool, workspace_id)
            .await
            .map_err(Into::into)
    }

    async fn get_turnover_analytics(
        &self,
        workspace_id: Uuid,
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> Result<TurnoverAnalytics, AppError> {
        employee::get_turnover_analytics(&self.sqlx_pool, workspace_id, start_date, end_date)
            .await
            .map_err(Into::into)
    }

    async fn get_managed_departments(
        &self,
        workspace_id: Uuid,
        user_id: Uuid,
    ) -> Result<Vec<Uuid>, AppError> {
        // Fallback to SqlxEmployeeRepository implementation
        let repo =
            crate::repositories::employee::SqlxEmployeeRepository::new(self.sqlx_pool.clone());
        repo.get_managed_departments(workspace_id, user_id).await
    }

    async fn get_employee_id_by_user_id(
        &self,
        workspace_id: Uuid,
        user_id: Uuid,
    ) -> Result<Option<Uuid>, AppError> {
        let repo =
            crate::repositories::employee::SqlxEmployeeRepository::new(self.sqlx_pool.clone());
        repo.get_employee_id_by_user_id(workspace_id, user_id).await
    }
}

impl crate::repositories::PoolBacked for DieselEmployeeRepository {
    #[allow(clippy::panic)]
    #[allow(clippy::panic)]
    #[allow(clippy::panic)]
    fn from_pool(_pool: sqlx::PgPool) -> Self {
        panic!("DieselEmployeeRepository requires diesel_pools to be manually injected via new()");
    }

    fn pool(&self) -> &sqlx::PgPool {
        &self.sqlx_pool
    }
}
