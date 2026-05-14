use chrono::{DateTime, NaiveDate, NaiveTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::models::attendance::ScheduleTrace;
use crate::models::enums::AssignmentHistoryType;

#[derive(Debug, Serialize, Deserialize, Clone, sqlx::FromRow)]
pub struct WorkingPattern {
    pub id: Uuid,
    pub workspace_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone, sqlx::FromRow)]
pub struct WorkingPatternDetail {
    pub id: Uuid,
    pub pattern_id: Uuid,
    pub day_of_week: i32,
    pub shift_id: Option<Uuid>,
}

#[derive(Debug, Serialize, Deserialize, Clone, sqlx::FromRow)]
pub struct EmployeeWorkingPattern {
    pub id: Uuid,
    pub workspace_id: Uuid,
    pub employee_id: Uuid,
    pub pattern_id: Uuid,
    pub start_date: NaiveDate,
    pub end_date: Option<NaiveDate>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct WorkingPatternDetailRequest {
    pub day_of_week: i32,
    pub shift_id: Option<Uuid>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct CreateWorkingPatternRequest {
    pub name: String,
    pub description: Option<String>,
    pub details: Vec<WorkingPatternDetailRequest>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct AssignPatternRequest {
    pub pattern_id: Uuid,
    pub start_date: NaiveDate,
    pub end_date: Option<NaiveDate>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct BulkAssignPatternRequest {
    pub employee_ids: Vec<Uuid>,
    pub pattern_id: Uuid,
    pub start_date: NaiveDate,
    pub end_date: Option<NaiveDate>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BulkAssignResponse {
    pub success_count: usize,
    pub failed_employees: Vec<Uuid>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EffectiveSchedule {
    pub date: NaiveDate,
    pub shift_id: Option<Uuid>, // None means Day Off
    pub source: String,         // "roster", "fixed", "department", "none"
    pub shift_name: Option<String>,
    pub shift_symbol: Option<String>,
    pub start_time: Option<NaiveTime>,
    pub end_time: Option<NaiveTime>,
    pub break_duration_minutes: Option<i32>,
    pub break_start: Option<NaiveTime>,
    pub break_end: Option<NaiveTime>,
    pub is_break_auto_deduct: Option<bool>,
    pub early_punch_in_grace_period_minutes: Option<i32>,
    pub allow_early_ot: Option<bool>,
    #[serde(default)]
    pub traces: Vec<ScheduleTrace>,
    #[serde(default)]
    pub is_holiday: bool,
    #[serde(default)]
    pub holiday_name: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EmployeeSchedule {
    pub employee_id: Uuid,
    pub employee_id_number: String,
    pub full_name: String,
    pub schedules: Vec<EffectiveSchedule>,
}

#[derive(Debug, Serialize, Deserialize, Clone, diesel::QueryableByName, sqlx::FromRow)]
pub struct AssignmentHistory {
    #[diesel(sql_type = diesel::sql_types::Uuid)]
    pub id: Uuid,
    #[diesel(sql_type = diesel::sql_types::Uuid)]
    pub employee_id: Uuid,
    #[diesel(sql_type = diesel::sql_types::Text)]
    pub employee_name: String,
    #[diesel(sql_type = crate::schema::sql_types::AssignmentHistoryType)]
    pub assignment_type: AssignmentHistoryType,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
    pub shift_name: Option<String>,
    #[diesel(sql_type = diesel::sql_types::Text)]
    pub date_range: String,
    #[diesel(sql_type = diesel::sql_types::Timestamptz)]
    pub created_at: DateTime<Utc>,
}
