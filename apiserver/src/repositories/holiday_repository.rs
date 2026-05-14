use async_trait::async_trait;
use chrono::NaiveDate;
use std::collections::HashMap;
use uuid::Uuid;

use crate::errors::AppError;
use crate::models::Holiday;

/// Result of processing holiday leave deductions for employees.
#[derive(Debug, Clone, serde::Serialize)]
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

#[async_trait]
pub trait HolidayRepository: Send + Sync {
    async fn list(&self, workspace_id: Uuid, year: Option<i32>) -> Result<Vec<Holiday>, AppError>;
    async fn get_by_id(&self, workspace_id: Uuid, id: Uuid) -> Result<Option<Holiday>, AppError>;
    async fn create_holiday(
        &self,
        workspace_id: Uuid,
        req: &crate::models::CreateHolidayRequest,
    ) -> Result<Holiday, AppError>;
    async fn update_holiday(
        &self,
        workspace_id: Uuid,
        id: Uuid,
        req: &crate::models::UpdateHolidayRequest,
    ) -> Result<Option<Holiday>, AppError>;
    async fn delete(&self, workspace_id: Uuid, id: Uuid) -> Result<bool, AppError>;
    async fn get_holidays_in_range(
        &self,
        workspace_id: Uuid,
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> Result<HashMap<NaiveDate, Holiday>, AppError>;
    async fn is_holiday(
        &self,
        workspace_id: Uuid,
        date: NaiveDate,
    ) -> Result<Option<Holiday>, AppError>;
}
