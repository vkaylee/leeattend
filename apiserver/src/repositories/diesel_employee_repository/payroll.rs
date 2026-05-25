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
impl EmployeePayrollConfigRepository for DieselEmployeeRepository {
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

        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .tenants_tx
                .get_connection(&workspace_id)
                .await
                .map_err(|e| AppError::Internal(format!("db: {e}")))?,
        );

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
            )>(&mut *conn)
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

        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .tenants_tx
                .get_connection(&workspace_id)
                .await
                .map_err(|e| AppError::Internal(format!("db: {e}")))?,
        );

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
            )>(&mut *conn)
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
        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .tenants_tx
                .get_connection(&workspace_id)
                .await
                .map_err(|e| AppError::Internal(format!("db: {e}")))?,
        );

        use crate::schema::{employee_payroll_configs, employees};

        let results = employee_payroll_configs::table
            .inner_join(employees::table)
            .filter(employee_payroll_configs::employee_id.eq(employee_id))
            .filter(employees::workspace_id.eq(workspace_id))
            .order(employee_payroll_configs::effective_date.desc())
            .select((
                employee_payroll_configs::id,
                employee_payroll_configs::employee_id,
                employee_payroll_configs::std_working_days,
                employee_payroll_configs::hours_per_day,
                employee_payroll_configs::effective_date,
                employee_payroll_configs::created_at,
            ))
            .load::<(
                Uuid,
                Uuid,
                rust_decimal::Decimal,
                rust_decimal::Decimal,
                chrono::NaiveDate,
                Option<chrono::DateTime<chrono::Utc>>,
            )>(&mut *conn)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(results
            .into_iter()
            .map(|t| crate::models::EmployeePayrollConfig {
                id: t.0,
                employee_id: t.1,
                std_working_days: t.2,
                hours_per_day: t.3,
                effective_date: t.4,
                created_at: t.5.unwrap_or_else(chrono::Utc::now),
            })
            .collect())
    }
}
