use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct AttendanceTrendPoint {
    pub date: NaiveDate,
    pub present: i64,
    pub late: i64,
    pub absent: i64,
    pub on_leave: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct WorkspaceStats {
    pub total_employees: i64,
    pub present_today: i64,
    pub pending_leave_requests: i64,
    pub estimated_monthly_payroll: rust_decimal::Decimal,
    pub late_today: i64,
    pub absent_today: i64,
    pub on_leave_today: i64,
    pub early_departure_today: i64,
    pub avg_work_hours_today: f64,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct EmployeeSearchQuery {
    pub q: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct PaginationParams {
    pub page: Option<i64>,
    pub limit: Option<i64>,
    pub q: Option<String>,
    pub year: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct GenericResponse {
    pub message: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PaginatedResponse<T> {
    pub data: Vec<T>,
    pub total: usize,
    pub page: i64,
    pub limit: i64,
}
