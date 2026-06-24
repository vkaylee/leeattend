use crate::errors::AppError;
use crate::models::{EmployeeLeaveBalance, LeaveBalance, WorkspaceLeaveBalance};
use async_trait::async_trait;
use uuid::Uuid;

// ---------------------------------------------------------------------------
// Trait: LeaveBalanceRepository — DIP abstraction
// ---------------------------------------------------------------------------
#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait LeaveBalanceRepository: Send + Sync {
    async fn get_employee_balances(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        employee_id: Uuid,
        year: i32,
    ) -> Result<Vec<EmployeeLeaveBalance>, AppError>;

    async fn get_employee_balances_batch(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        employee_ids: &[Uuid],
        year: i32,
    ) -> Result<std::collections::HashMap<Uuid, Vec<EmployeeLeaveBalance>>, AppError>;
    async fn get_workspace_balances(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        year: i32,
        limit: i64,
        offset: i64,
        search: Option<String>,
    ) -> Result<(Vec<WorkspaceLeaveBalance>, i64), AppError>;
    async fn upsert_leave_balance(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        balance: &LeaveBalance,
    ) -> Result<(), AppError>;
    async fn adjust_balance(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        employee_id: Uuid,
        leave_type_id: Uuid,
        amount: rust_decimal::Decimal,
        transaction_date: chrono::NaiveDate,
        reason: &str,
    ) -> Result<(), AppError>;
}
