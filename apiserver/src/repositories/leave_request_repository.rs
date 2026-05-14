use crate::errors::AppError;
use crate::models::{LeaveRequest, LeaveStatus};
use async_trait::async_trait;
use chrono::{NaiveDate, Utc};
use uuid::Uuid;

// ---------------------------------------------------------------------------
// Trait: LeaveRequestRepository — DIP abstraction
// ---------------------------------------------------------------------------
#[async_trait]
pub trait LeaveRequestRepository: Send + Sync {
    async fn create_leave_request(&self, request: &LeaveRequest) -> Result<(), AppError>;
    async fn get_leave_request_by_id(
        &self,
        workspace_id: Uuid,
        id: Uuid,
    ) -> Result<Option<LeaveRequest>, AppError>;
    async fn update_leave_request_status(
        &self,
        workspace_id: Uuid,
        id: Uuid,
        status: LeaveStatus,
        approved_by: Option<Uuid>,
    ) -> Result<(), AppError>;
    async fn get_employee_leave_requests(
        &self,
        workspace_id: Uuid,
        employee_id: Uuid,
    ) -> Result<Vec<LeaveRequest>, AppError>;
    async fn get_workspace_leave_requests(
        &self,
        workspace_id: Uuid,
        status: Option<LeaveStatus>,
    ) -> Result<Vec<LeaveRequest>, AppError>;
    async fn get_approved_leave_breakdown(
        &self,
        employee_id: Uuid,
        start_date: NaiveDate,
        end_date: NaiveDate,
        workspace_id: Uuid,
    ) -> Result<LeaveBreakdown, AppError>;
    /// Get the employee's current department for a given leave request.
    /// Used for scoped permission checks in approval flows.
    async fn get_employee_department_for_request(
        &self,
        request_id: Uuid,
        workspace_id: Uuid,
    ) -> Result<Option<Uuid>, AppError>;
    /// List all leave types (code, name) for a workspace.
    async fn get_leave_types(&self, workspace_id: Uuid) -> Result<Vec<(String, String)>, AppError>;

    async fn get_system_generated_holiday_leave(
        &self,
        workspace_id: Uuid,
        employee_id: Uuid,
        date: NaiveDate,
    ) -> Result<Option<(Uuid, Uuid, String)>, AppError>;

    async fn cancel_leave_request(&self, workspace_id: Uuid, id: Uuid) -> Result<(), AppError>;

    async fn get_employee_leave_history_requests(
        &self,
        workspace_id: Uuid,
        employee_id: Uuid,
        year: i32,
        leave_type_id: Option<Uuid>,
    ) -> Result<Vec<LeaveHistoryRequestItem>, AppError>;

    async fn check_leave_type_exists(
        &self,
        workspace_id: Uuid,
        leave_type_id: Uuid,
    ) -> Result<bool, AppError>;

    async fn get_leave_type_name(
        &self,
        workspace_id: Uuid,
        leave_type_id: Uuid,
    ) -> Result<Option<String>, AppError>;

    async fn get_unpaid_holiday_days_without_leave(
        &self,
        workspace_id: Uuid,
        employee_id: Uuid,
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> Result<i64, AppError>;

    async fn get_batch_leave_days(
        &self,
        employee_ids: &[Uuid],
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> Result<std::collections::HashMap<Uuid, i64>, AppError>;
}

pub struct LeaveHistoryRequestItem {
    pub id: Uuid,
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
    pub leave_type_name: String,
    pub reason: Option<String>,
    pub created_at: Option<chrono::DateTime<Utc>>,
}

pub struct LeaveBreakdown {
    pub total_days: f64,
    pub unpaid_days: f64,
    pub sick_days: f64,
}
