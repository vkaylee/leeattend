use crate::diesel_pool::DieselPools;
use crate::errors::AppError;
use crate::models::diesel::leave::*;
use crate::models::{EmployeeLeaveBalance, LeaveBalance, WorkspaceLeaveBalance};
use crate::repositories::leave_balance_repository::LeaveBalanceRepository;

use crate::schema::*;
use async_trait::async_trait;
use bigdecimal::BigDecimal;
use chrono::Datelike;
use diesel::prelude::*;
use diesel_async::{AsyncConnection, RunQueryDsl};
use std::str::FromStr;
use uuid::Uuid;

pub struct DieselLeaveBalanceRepository {
    pub diesel_pools: DieselPools,
}

impl DieselLeaveBalanceRepository {
    pub fn new(diesel_pools: DieselPools) -> Self {
        Self { diesel_pools }
    }
}

#[allow(clippy::too_many_lines)]
#[async_trait]
impl LeaveBalanceRepository for DieselLeaveBalanceRepository {
    async fn get_employee_balances(
        &self,
        workspace_id: Uuid,
        employee_id: Uuid,
        year: i32,
    ) -> Result<Vec<EmployeeLeaveBalance>, AppError> {
        let mut conn = self
            .diesel_pools
            .tenants
            .get_pool(&workspace_id)
            .await
            .map_err(|e| AppError::Internal(format!("db pool error: {e}")))?
            .get()
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        // Using raw SQL for the complex CTE report to ensure compatibility
        let sql = r#"
            WITH leave_usage AS (
                SELECT
                    lr.leave_type_id,
                    SUM(
                        (SELECT COUNT(*)
                         FROM generate_series(lr.start_date, lr.end_date, '1 day'::interval) d
                         WHERE EXTRACT(DOW FROM d) NOT IN (0, 6))
                    )::INT as used_days
                FROM leave_requests lr
                JOIN employees e_usage ON lr.employee_id = e_usage.id
                WHERE lr.employee_id = $1
                  AND e_usage.workspace_id = $3
                  AND lr.status IN ('approved', 'pre_approved')
                  AND EXTRACT(YEAR FROM lr.start_date)::INT = $2
                GROUP BY lr.leave_type_id
            )
            SELECT
                lb.leave_type_id as "leave_type_id",
                lt.code as "leave_type_code",
                lt.name as "leave_type_name",
                lt.is_paid as "is_paid",
                lb.entitlement as "entitlement",
                lb.accrued_amount::FLOAT8 as "accrued_amount",
                lb.carried_over as "carried_over",
                COALESCE(lu.used_days, 0)::INT as "used",
                (
                    lb.entitlement + lb.accrued_amount + lb.carried_over
                    - COALESCE(lu.used_days, 0)
                )::FLOAT8 as "remaining"
            FROM leave_balances lb
            JOIN leave_types lt ON lb.leave_type_id = lt.id
            JOIN employees e ON lb.employee_id = e.id
            LEFT JOIN leave_usage lu ON lb.leave_type_id = lu.leave_type_id
            WHERE lb.employee_id = $1 AND lb.year = $2 AND e.workspace_id = $3
        "#;

        let balances = diesel::sql_query(sql)
            .bind::<diesel::sql_types::Uuid, _>(employee_id)
            .bind::<diesel::sql_types::Int4, _>(year)
            .bind::<diesel::sql_types::Uuid, _>(workspace_id)
            .load::<crate::models::diesel::leave::EmployeeLeaveBalanceRaw>(&mut conn)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(balances.into_iter().map(Into::into).collect())
    }

    async fn get_workspace_balances(
        &self,
        workspace_id: Uuid,
        year: i32,
        limit: i64,
        offset: i64,
        search: Option<String>,
    ) -> Result<(Vec<WorkspaceLeaveBalance>, i64), AppError> {
        let mut conn = self
            .diesel_pools
            .tenants
            .get_pool(&workspace_id)
            .await
            .map_err(|e| AppError::Internal(format!("db pool error: {e}")))?
            .get()
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        let search_term = format!("%{}%", search.unwrap_or_default());

        // Count
        let count_sql = r#"
            SELECT COUNT(*)
            FROM employees e
            CROSS JOIN leave_types lt
            LEFT JOIN leave_balances lb ON e.id = lb.employee_id AND lt.id = lb.leave_type_id AND lb.year = $2
            WHERE e.workspace_id = $1 AND e.is_active = TRUE AND lt.workspace_id = $1
              AND ($3 = '%%' OR e.full_name ILIKE $3 OR e.employee_id_number ILIKE $3 OR lt.name ILIKE $3 OR lt.code ILIKE $3)
              AND (
                lb.employee_id IS NOT NULL
                OR EXISTS (
                  SELECT 1 FROM leave_policy_assignments a
                  JOIN leave_policy_definitions d ON a.policy_id = d.id
                  WHERE d.leave_type_id = lt.id AND a.workspace_id = $1
                    AND a.effective_from <= NOW()
                    AND (a.effective_to IS NULL OR a.effective_to > NOW())
                    AND (
                      (a.assignee_type = 'employee' AND a.assignee_id = e.id)
                      OR (a.assignee_type = 'workspace')
                      OR (a.assignee_type = 'department' AND a.assignee_id = (
                        SELECT department_id FROM employee_department_history
                        WHERE employee_id = e.id ORDER BY effective_date DESC LIMIT 1
                      ))
                    )
                )
              )
        "#;

        let total: i64 = diesel::sql_query(count_sql)
            .bind::<diesel::sql_types::Uuid, _>(workspace_id)
            .bind::<diesel::sql_types::Int4, _>(year)
            .bind::<diesel::sql_types::Text, _>(&search_term)
            .get_result::<crate::models::diesel::BigIntCount>(&mut conn)
            .await
            .map_err(|e| AppError::Internal(format!("db count: {e}")))?
            .count;

        // Data
        let sql = r#"
            WITH leave_usage AS (
                SELECT
                    lr.employee_id,
                    lr.leave_type_id,
                    SUM(
                        (SELECT COUNT(*)
                         FROM generate_series(lr.start_date, lr.end_date, '1 day'::interval) d
                         WHERE EXTRACT(DOW FROM d) NOT IN (0, 6))
                    )::INT as used_days
                FROM leave_requests lr
                WHERE lr.status IN ('approved', 'pre_approved')
                  AND EXTRACT(YEAR FROM lr.start_date)::INT = $2
                GROUP BY lr.employee_id, lr.leave_type_id
            ),
            best_policy AS (
                SELECT DISTINCT ON (lt_id, emp_id)
                    a.assignee_id as emp_id_raw,
                    d.leave_type_id as lt_id,
                    e_inner.id as emp_id,
                    d.name as policy_name,
                    a.assignee_type as assignment_type
                FROM leave_policy_assignments a
                JOIN leave_policy_definitions d ON a.policy_id = d.id
                CROSS JOIN employees e_inner
                WHERE a.workspace_id = $1
                  AND e_inner.workspace_id = $1
                  AND e_inner.is_active = TRUE
                  AND (
                    (a.assignee_type = 'employee' AND a.assignee_id = e_inner.id) OR
                    (a.assignee_type = 'department' AND a.assignee_id = (
                        SELECT department_id FROM employee_department_history
                        WHERE employee_id = e_inner.id ORDER BY effective_date DESC LIMIT 1
                    )) OR
                    (a.assignee_type = 'workspace')
                  )
                  AND a.effective_from <= NOW()
                  AND (a.effective_to IS NULL OR a.effective_to > NOW())
                ORDER BY lt_id, emp_id,
                  CASE a.assignee_type
                    WHEN 'employee' THEN 1
                    WHEN 'department' THEN 2
                    WHEN 'workspace' THEN 3
                  END ASC
            )
            SELECT
                e.id as employee_id,
                e.full_name as full_name,
                e.employee_id_number as employee_id_number,
                lt.id as leave_type_id,
                lt.code as leave_type_code,
                lt.name as leave_type_name,
                lt.is_paid as is_paid,
                COALESCE(lb.entitlement, 0) as entitlement,
                COALESCE(lb.accrued_amount::FLOAT8, 0.0) as accrued_amount,
                COALESCE(lb.carried_over, 0) as carried_over,
                COALESCE(lu.used_days, 0) as used,
                (
                    COALESCE(lb.entitlement, 0) + COALESCE(lb.accrued_amount, 0) + COALESCE(lb.carried_over, 0)
                    - COALESCE(lu.used_days, 0)
                )::FLOAT8 as remaining,
                bp.policy_name as policy_name,
                bp.assignment_type as assignment_type
            FROM employees e
            CROSS JOIN leave_types lt
            LEFT JOIN leave_balances lb ON e.id = lb.employee_id AND lt.id = lb.leave_type_id AND lb.year = $2
            LEFT JOIN leave_usage lu ON e.id = lu.employee_id AND lt.id = lu.leave_type_id
            LEFT JOIN best_policy bp ON e.id = bp.emp_id AND lt.id = bp.lt_id
            WHERE e.workspace_id = $1 AND e.is_active = TRUE AND lt.workspace_id = $1
              AND ($3 = '%%' OR e.full_name ILIKE $3 OR e.employee_id_number ILIKE $3 OR lt.name ILIKE $3 OR lt.code ILIKE $3)
              AND (lb.employee_id IS NOT NULL OR bp.emp_id IS NOT NULL)
            ORDER BY e.full_name ASC, lt.name ASC
            LIMIT $4 OFFSET $5
        "#;

        let data = diesel::sql_query(sql)
            .bind::<diesel::sql_types::Uuid, _>(workspace_id)
            .bind::<diesel::sql_types::Int4, _>(year)
            .bind::<diesel::sql_types::Text, _>(&search_term)
            .bind::<diesel::sql_types::BigInt, _>(limit)
            .bind::<diesel::sql_types::BigInt, _>(offset)
            .load::<crate::models::diesel::leave::WorkspaceLeaveBalanceRaw>(&mut conn)
            .await
            .map_err(|e| AppError::Internal(format!("db data: {e}")))?;

        Ok((data.into_iter().map(Into::into).collect(), total))
    }

    async fn upsert_leave_balance(
        &self,
        workspace_id: Uuid,
        balance: &LeaveBalance,
    ) -> Result<(), AppError> {
        let mut conn = self
            .diesel_pools
            .tenants
            .get_pool(&workspace_id)
            .await
            .map_err(|e| AppError::Internal(format!("db pool error: {e}")))?
            .get()
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        let entity = LeaveBalanceEntity {
            id: balance.id,
            employee_id: balance.employee_id,
            year: balance.year,
            entitlement: balance.entitlement,
            carried_over: balance.carried_over,
            created_at: Some(balance.created_at),
            updated_at: Some(balance.updated_at),
            accrued_amount: BigDecimal::from_str(&balance.accrued_amount.to_string())
                .unwrap_or_else(|_| BigDecimal::from(0)),
            leave_type_id: balance.leave_type_id,
        };

        diesel::insert_into(leave_balances::table)
            .values(&entity)
            .on_conflict((
                leave_balances::employee_id,
                leave_balances::leave_type_id,
                leave_balances::year,
            ))
            .do_update()
            .set((
                leave_balances::entitlement.eq(entity.entitlement),
                leave_balances::carried_over.eq(entity.carried_over),
                leave_balances::updated_at.eq(diesel::dsl::now),
            ))
            .execute(&mut conn)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(())
    }

    async fn adjust_balance(
        &self,
        workspace_id: Uuid,
        employee_id: Uuid,
        leave_type_id: Uuid,
        amount: rust_decimal::Decimal,
        transaction_date: chrono::NaiveDate,
        reason: &str,
    ) -> Result<(), AppError> {
        let mut conn = self
            .diesel_pools
            .tenants
            .get_pool(&workspace_id)
            .await
            .map_err(|e| AppError::Internal(format!("db pool error: {e}")))?
            .get()
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        let amount_bd = BigDecimal::from_str(&amount.to_string())
            .map_err(|e| AppError::Internal(format!("BigDecimal conversion failed: {e}")))?;
        let year = transaction_date.year();

        conn.transaction(|mut conn| {
            Box::pin(async move {
                // 1. Log adjustment
                diesel::insert_into(leave_accrual_logs::table)
                    .values((
                        leave_accrual_logs::workspace_id.eq(workspace_id),
                        leave_accrual_logs::employee_id.eq(employee_id),
                        leave_accrual_logs::accrual_date.eq(transaction_date),
                        leave_accrual_logs::amount.eq(&amount_bd),
                        leave_accrual_logs::leave_type_id.eq(leave_type_id),
                        leave_accrual_logs::notes.eq(reason),
                    ))
                    .execute(&mut conn)
                    .await?;

                // 2. Update balance
                diesel::insert_into(leave_balances::table)
                    .values((
                        leave_balances::id.eq(Uuid::new_v4()),
                        leave_balances::employee_id.eq(employee_id),
                        leave_balances::leave_type_id.eq(leave_type_id),
                        leave_balances::year.eq(year),
                        leave_balances::entitlement.eq(0),
                        leave_balances::accrued_amount.eq(&amount_bd),
                        leave_balances::carried_over.eq(0),
                    ))
                    .on_conflict((
                        leave_balances::employee_id,
                        leave_balances::leave_type_id,
                        leave_balances::year,
                    ))
                    .do_update()
                    .set((
                        leave_balances::accrued_amount
                            .eq(leave_balances::accrued_amount + &amount_bd),
                        leave_balances::updated_at.eq(diesel::dsl::now),
                    ))
                    .execute(&mut conn)
                    .await?;

                Ok::<_, diesel::result::Error>(())
            })
        })
        .await
        .map_err(|e| AppError::Internal(format!("db transaction: {e}")))?;

        Ok(())
    }
}
