use async_trait::async_trait;
use chrono::NaiveDate;
use std::collections::HashMap;
use uuid::Uuid;

use crate::errors::AppError;
use crate::models::Holiday;

/// Result of processing holiday leave deductions for employees.
#[derive(Debug, Clone, serde::Serialize, utoipa::ToSchema)]
pub enum HolidayLeaveResult {
    /// Holiday is paid — no deduction needed.
    Paid,
    /// Leave was auto-deducted from employee balance.
    LeaveDeducted {
        employee_id: Uuid,
        leave_request_id: Uuid,
        leave_type_name: String,
    },
    /// Employee has no remaining leave — unpaid day.
    Unpaid { employee_id: Uuid },
}

/// A single holiday leave deduction log entry:
/// `(employee_id, leave_type_id, amount, date, notes)`.
pub type HolidayDeductionLogEntry = (Uuid, Uuid, rust_decimal::Decimal, NaiveDate, String);

#[async_trait]
#[cfg_attr(test, mockall::automock)]
pub trait HolidayRepository: Send + Sync {
    async fn list(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        year: Option<i32>,
        location_id: Option<Uuid>,
        page: i64,
        limit: i64,
    ) -> Result<(Vec<Holiday>, i64), AppError>;
    async fn get_by_id(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        id: Uuid,
    ) -> Result<Option<Holiday>, AppError>;
    async fn create_holiday(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        req: &crate::models::CreateHolidayRequest,
    ) -> Result<Holiday, AppError>;
    async fn update_holiday(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        id: Uuid,
        req: &crate::models::UpdateHolidayRequest,
    ) -> Result<Option<Holiday>, AppError>;
    async fn delete(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        id: Uuid,
    ) -> Result<bool, AppError>;
    async fn get_holidays_in_range(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> Result<HashMap<NaiveDate, Holiday>, AppError>;
    async fn is_holiday(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        date: NaiveDate,
        location_id: Option<Uuid>,
    ) -> Result<Option<Holiday>, AppError>;

    /// Executes a bulk insertion of leave requests, shift rosters, and leave accrual logs
    /// inside a single database transaction to process holiday deductions.
    async fn apply_holiday_leave_deductions(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        requests: &[crate::models::LeaveRequest],
        rosters: &[crate::models::ShiftRoster],
        logs: &[HolidayDeductionLogEntry],
    ) -> Result<(), AppError>;
}
