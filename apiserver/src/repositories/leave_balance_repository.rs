use crate::errors::AppError;
use crate::models::{EmployeeLeaveBalance, LeaveBalance, WorkspaceLeaveBalance};
use async_trait::async_trait;
use uuid::Uuid;

// ---------------------------------------------------------------------------
// Trait: LeaveBalanceRepository — DIP abstraction
// ---------------------------------------------------------------------------
#[async_trait]
pub trait LeaveBalanceRepository: Send + Sync {
    async fn get_employee_balances(
        &self,
        workspace_id: Uuid,
        employee_id: Uuid,
        year: i32,
    ) -> Result<Vec<EmployeeLeaveBalance>, AppError>;
    async fn get_workspace_balances(
        &self,
        workspace_id: Uuid,
        year: i32,
        limit: i64,
        offset: i64,
        search: Option<String>,
    ) -> Result<(Vec<WorkspaceLeaveBalance>, i64), AppError>;
    async fn upsert_leave_balance(&self, balance: &LeaveBalance) -> Result<(), AppError>;
    async fn adjust_balance(
        &self,
        workspace_id: Uuid,
        employee_id: Uuid,
        leave_type_id: Uuid,
        amount: rust_decimal::Decimal,
        transaction_date: chrono::NaiveDate,
        reason: &str,
    ) -> Result<(), AppError>;
}
