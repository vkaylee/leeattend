use crate::errors::AppError;
use crate::models::diesel::attendance_exception::AttendanceExceptionEntity;
use async_trait::async_trait;
use chrono::NaiveDate;
use uuid::Uuid;

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait AttendanceExceptionRepository: Send + Sync {
    /// Fetch the exception for an employee on a given date.
    /// Returns None when no exception covers this date.
    /// At most one row per (employee_id, date) — enforced by EXCLUDE constraint.
    async fn get_exception_for_date(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        employee_id: Uuid,
        date: NaiveDate,
    ) -> Result<Option<AttendanceExceptionEntity>, AppError>;
}
