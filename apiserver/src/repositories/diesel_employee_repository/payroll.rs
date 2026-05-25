#![allow(unused_imports)]
use crate::errors::AppError;
use async_trait::async_trait;
use chrono::NaiveDate;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use diesel_async::AsyncConnection;
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

use super::{DieselEmployeeRepository, EMPLOYEE_SELECT_QUERY, EMPLOYEE_DEPT_JOIN, EmployeeSearchSql, build_employee_search_sql};

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

        let results: Vec<Entity> = diesel::sql_query(r#"SELECT epc.id, epc.employee_id, epc.std_working_days, epc.hours_per_day, epc.effective_date, epc.created_at
               FROM employee_payroll_configs epc
               JOIN employees e ON e.id = epc.employee_id
               WHERE epc.employee_id = $1 AND e.workspace_id = $2
               ORDER BY epc.effective_date DESC"#,
        )
        .bind::<diesel::sql_types::Uuid, _>(employee_id)
        .bind::<diesel::sql_types::Uuid, _>(workspace_id)
        .load(&mut *conn)
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

}
