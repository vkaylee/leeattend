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
pub struct DepartmentAttendancePoint {
    pub department_id: uuid::Uuid,
    pub department_name: String,
    pub on_time: i64,
    pub late: i64,
    pub absent: i64,
    pub on_leave: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct UnexcusedAbsencesPoint {
    pub department_id: uuid::Uuid,
    pub department_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub employee_id: Option<uuid::Uuid>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub employee_name: Option<String>,
    pub unexcused_count: i64,
    pub consecutive_count: i64,
    pub total_absent_count: i64,
    pub total_leave_count: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct OvertimeTrendPoint {
    pub date: NaiveDate,
    pub system_ot_hours: f64,
    pub approved_ot_hours: f64,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct PeakTimeSlot {
    pub time_slot: String,
    pub check_in: i64,
    pub check_out: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct LeaveTypeBreakdown {
    pub leave_type: String,
    pub count: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct LatenessByDay {
    pub day: String,
    pub late_count: i64,
    pub avg_late_minutes: f64,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct EmployeeSearchQuery {
    pub q: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema, utoipa::IntoParams)]
pub struct PaginationParams {
    pub page: Option<i64>,
    pub limit: Option<i64>,
    pub q: Option<String>,
    pub year: Option<i32>,
    pub sort_by: Option<String>,
    pub sort_dir: Option<String>,
    pub department_id: Option<String>,
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
