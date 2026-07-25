use crate::errors::AppError;
use crate::models::{LeaveRequest, LeaveStatus};
use async_trait::async_trait;
use chrono::{NaiveDate, Utc};
use uuid::Uuid;

// ---------------------------------------------------------------------------
// Trait: LeaveRequestRepository — DIP abstraction
// ---------------------------------------------------------------------------
#[async_trait]
#[cfg_attr(test, mockall::automock)]
pub trait LeaveRequestRepository: Send + Sync {
    async fn create_leave_request(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        request: &LeaveRequest,
    ) -> Result<(), AppError>;
    async fn get_leave_request_by_id(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        id: Uuid,
    ) -> Result<Option<LeaveRequest>, AppError>;
    async fn update_leave_request_status(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        id: Uuid,
        status: LeaveStatus,
        approved_by: Option<Uuid>,
    ) -> Result<(), AppError>;
    async fn get_employee_leave_requests(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        employee_id: Uuid,
    ) -> Result<Vec<LeaveRequest>, AppError>;
    async fn get_workspace_leave_requests(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        status: Option<LeaveStatus>,
    ) -> Result<Vec<LeaveRequest>, AppError>;
    async fn get_paginated_workspace_leave_requests(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        status: Option<LeaveStatus>,
        employee_id: Option<Uuid>,
        limit: i64,
        offset: i64,
    ) -> Result<(Vec<LeaveRequest>, i64), AppError>;
    async fn get_approved_leave_breakdown(
        &self,
        employee_id: Uuid,
        start_date: NaiveDate,
        end_date: NaiveDate,
        auth_token: &crate::diesel_pool::TenantAuthToken,
    ) -> Result<LeaveBreakdown, AppError>;
    /// Get the employee's current department for a given leave request.
    /// Used for scoped permission checks in approval flows.
    async fn get_employee_department_for_request(
        &self,
        request_id: Uuid,
        auth_token: &crate::diesel_pool::TenantAuthToken,
    ) -> Result<Option<Uuid>, AppError>;
    /// List all leave types (code, name) for a workspace.
    async fn get_leave_types(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
    ) -> Result<Vec<(String, String)>, AppError>;

    async fn get_system_generated_holiday_leave(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        employee_id: Uuid,
        date: NaiveDate,
    ) -> Result<Option<(Uuid, Uuid, String)>, AppError>;

    async fn cancel_leave_request(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        id: Uuid,
    ) -> Result<(), AppError>;

    async fn bulk_cancel_leave_requests(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        ids: &[Uuid],
    ) -> Result<u64, AppError>;

    /// Create a leave request within an existing connection/transaction.
    async fn create_leave_request_tx(
        &self,
        conn: &mut diesel_async::AsyncPgConnection,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        request: &LeaveRequest,
    ) -> Result<(), AppError>;

    /// Bulk cancel leave requests within an existing connection/transaction.
    async fn bulk_cancel_leave_requests_tx(
        &self,
        conn: &mut diesel_async::AsyncPgConnection,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        ids: &[Uuid],
    ) -> Result<u64, AppError>;

    /// Update leave request status within an existing connection/transaction.
    /// Used by approve_leave_request to keep status update + side effects atomic.
    async fn update_leave_request_status_tx(
        &self,
        conn: &mut diesel_async::AsyncPgConnection,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        id: Uuid,
        status: LeaveStatus,
        approved_by: Option<Uuid>,
    ) -> Result<(), AppError>;

    async fn get_employee_leave_history_requests(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        employee_id: Uuid,
        year: i32,
        leave_type_id: Option<Uuid>,
    ) -> Result<Vec<LeaveHistoryRequestItem>, AppError>;

    async fn check_leave_type_exists(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        leave_type_id: Uuid,
    ) -> Result<bool, AppError>;

    async fn get_leave_type_name(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        leave_type_id: Uuid,
    ) -> Result<Option<String>, AppError>;

    async fn get_unpaid_holiday_days_without_leave(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        employee_id: Uuid,
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> Result<i64, AppError>;

    async fn get_batch_leave_days(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        employee_ids: &[Uuid],
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> Result<std::collections::HashMap<Uuid, i64>, AppError>;

    async fn get_approved_leave_dates(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        employee_ids: &[Uuid],
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> Result<std::collections::HashSet<(Uuid, NaiveDate)>, AppError>;

    async fn get_approved_leave_details(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        employee_ids: &[Uuid],
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> Result<std::collections::HashMap<(Uuid, NaiveDate), bool>, AppError>;

    async fn get_approved_leave_periods(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        employee_id: Uuid,
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> Result<Vec<(NaiveDate, NaiveDate)>, AppError>;

    async fn get_approved_leave_breakdowns_batch(
        &self,
        employee_ids: &[Uuid],
        start_date: NaiveDate,
        end_date: NaiveDate,
        auth_token: &crate::diesel_pool::TenantAuthToken,
    ) -> Result<std::collections::HashMap<Uuid, LeaveBreakdown>, AppError>;

    async fn get_unpaid_holiday_days_without_leave_batch(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        employee_ids: &[Uuid],
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> Result<std::collections::HashMap<Uuid, i64>, AppError>;

    async fn get_leave_requests_by_ids(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        ids: &[Uuid],
    ) -> Result<Vec<LeaveRequest>, AppError>;
}

#[derive(Clone)]
pub struct LeaveHistoryRequestItem {
    pub id: Uuid,
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
    pub leave_type_name: String,
    pub reason: Option<String>,
    pub created_at: Option<chrono::DateTime<Utc>>,
}

#[derive(Debug, Clone, Copy)]
pub struct LeaveBreakdown {
    pub total_days: f64,
    pub unpaid_days: f64,
    pub sick_days: f64,
}
