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
    employee_documents, employee_emergency_contacts, employee_profiles, employees,
    recruitment_sources,
};
use diesel::sql_types::{Date, Uuid as UuidType};

use super::{
    build_employee_search_sql, DieselEmployeeRepository, EmployeeSearchSql, EMPLOYEE_DEPT_JOIN,
    EMPLOYEE_SELECT_QUERY,
};

#[allow(clippy::too_many_lines)]
#[async_trait]
impl EmployeeWorkingConditionRepository for DieselEmployeeRepository {
    async fn get_working_conditions(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        employee_id: Uuid,
        page: i64,
        limit: i64,
    ) -> Result<(Vec<WorkingCondition>, i64), AppError> {
        let workspace_id = auth_token.workspace_id();
        use crate::schema::{employees, working_conditions};

        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .tenants_tx
                .get_connection(&workspace_id)
                .await
                .map_err(|e| AppError::Internal(format!("db: {e}")))?,
        );

        let total: i64 = working_conditions::table
            .inner_join(employees::table)
            .filter(working_conditions::employee_id.eq(employee_id))
            .filter(employees::workspace_id.eq(workspace_id))
            .filter(employees::deleted_at.is_null())
            .count()
            .get_result(&mut *conn)
            .await
            .map_err(|e| AppError::Internal(format!("db count: {e}")))?;

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
            .filter(employees::deleted_at.is_null())
            .order(working_conditions::start_date.desc())
            .limit(limit)
            .offset((page - 1) * limit)
            .load::<(
                Uuid,
                Uuid,
                String,
                i32,
                bool,
                chrono::NaiveDate,
                Option<chrono::NaiveDate>,
            )>(&mut *conn)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        let data = result
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
            .collect();

        Ok((data, total))
    }

    async fn create_working_condition(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        condition: &WorkingCondition,
    ) -> Result<(), AppError> {
        let workspace_id = auth_token.workspace_id();
        use crate::schema::working_conditions;

        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .tenants_tx
                .get_connection(&workspace_id)
                .await
                .map_err(|e| AppError::Internal(format!("db: {e}")))?,
        );

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
            .execute(&mut *conn)
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

        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .tenants_tx
                .get_connection(&workspace_id)
                .await
                .map_err(|e| AppError::Internal(format!("db: {e}")))?,
        );

        let affected = diesel::delete(working_conditions::table)
            .filter(working_conditions::id.eq(id))
            .filter(working_conditions::employee_id.eq(employee_id))
            .execute(&mut *conn)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(affected > 0)
    }

    async fn get_working_conditions_at(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        employee_id: Uuid,
        date: NaiveDate,
    ) -> Result<Vec<WorkingCondition>, AppError> {
        let workspace_id = auth_token.workspace_id();
        use crate::schema::{employees, working_conditions};

        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .tenants_tx
                .get_connection(&workspace_id)
                .await
                .map_err(|e| AppError::Internal(format!("db: {e}")))?,
        );

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
            .filter(employees::deleted_at.is_null())
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
            )>(&mut *conn)
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

        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .tenants_tx
                .get_connection(&workspace_id)
                .await
                .map_err(|e| AppError::Internal(format!("db: {e}")))?,
        );

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
            .filter(employees::deleted_at.is_null())
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
            )>(&mut *conn)
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
}
