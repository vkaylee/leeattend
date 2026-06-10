use chrono::{DateTime, NaiveDate, NaiveTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::models::attendance::ScheduleTrace;
use crate::models::enums::AssignmentHistoryType;

#[derive(Debug, Serialize, Deserialize, Clone, utoipa::ToSchema)]
pub struct WorkingPattern {
    pub id: Uuid,
    pub workspace_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub pattern_type: String,
    pub cycle_length_days: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize, Clone, utoipa::ToSchema)]
pub struct WorkingPatternDetail {
    pub id: Uuid,
    pub pattern_id: Uuid,
    pub schedule_day: i32,
    pub shift_id: Option<Uuid>,
}

#[derive(Debug, Serialize, Deserialize, Clone, utoipa::ToSchema)]
pub struct EmployeeWorkingPattern {
    pub id: Uuid,
    pub workspace_id: Uuid,
    pub employment_id: Uuid,
    pub pattern_id: Uuid,
    pub start_date: NaiveDate,
    pub end_date: Option<NaiveDate>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub anchor_date: Option<NaiveDate>,
}

#[derive(Debug, Deserialize, Serialize, utoipa::ToSchema)]
pub struct WorkingPatternDetailRequest {
    pub schedule_day: i32,
    pub shift_id: Option<Uuid>,
}

#[derive(Debug, Deserialize, Serialize, utoipa::ToSchema)]
pub struct CreateWorkingPatternRequest {
    pub name: String,
    pub description: Option<String>,
    pub pattern_type: Option<String>,
    pub cycle_length_days: Option<i32>,
    pub details: Vec<WorkingPatternDetailRequest>,
}

#[derive(Debug, Deserialize, Serialize, utoipa::ToSchema)]
pub struct AssignPatternRequest {
    pub pattern_id: Uuid,
    pub start_date: NaiveDate,
    pub end_date: Option<NaiveDate>,
    pub anchor_date: Option<NaiveDate>,
}

#[derive(Debug, Deserialize, Serialize, utoipa::ToSchema)]
pub struct BulkAssignPatternRequest {
    pub employment_ids: Vec<Uuid>,
    pub pattern_id: Uuid,
    pub start_date: NaiveDate,
    pub end_date: Option<NaiveDate>,
    pub anchor_date: Option<NaiveDate>,
}

#[derive(Debug, Serialize, Deserialize, utoipa::ToSchema)]
pub struct BulkAssignResponse {
    pub success_count: usize,
    pub failed_employees: Vec<Uuid>,
}

#[derive(Debug, Serialize, Deserialize, Clone, utoipa::ToSchema)]
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
    pub break_is_overtime: Option<bool>,
    pub max_early_ot_minutes: Option<i32>,
    #[serde(default)]
    pub traces: Vec<ScheduleTrace>,
    #[serde(default)]
    pub is_holiday: bool,
    #[serde(default)]
    pub is_holiday_paid: bool,
    #[serde(default)]
    pub holiday_name: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, utoipa::ToSchema)]
pub struct EmployeeSchedule {
    pub employment_id: Uuid,
    pub employee_id: Uuid,
    pub employee_id_number: String,
    pub full_name: String,
    pub schedules: Vec<EffectiveSchedule>,
}

#[derive(Debug, Serialize, Deserialize, Clone, diesel::QueryableByName, utoipa::ToSchema)]
pub struct AssignmentHistory {
    #[diesel(sql_type = diesel::sql_types::Uuid)]
    pub id: Uuid,
    #[diesel(sql_type = diesel::sql_types::Uuid)]
    pub employee_id: Uuid,
    #[diesel(sql_type = diesel::sql_types::Text)]
    pub employee_name: String,
    #[diesel(sql_type = crate::models::enums::AssignmentHistoryTypeSql)]
    pub assignment_type: AssignmentHistoryType,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
    pub shift_name: Option<String>,
    #[diesel(sql_type = diesel::sql_types::Text)]
    pub date_range: String,
    #[diesel(sql_type = diesel::sql_types::Timestamptz)]
    pub created_at: DateTime<Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{NaiveDate, NaiveTime};

    #[test]
    fn test_working_pattern_serialization() {
        let pattern = WorkingPattern {
            id: Uuid::new_v4(),
            workspace_id: Uuid::new_v4(),
            name: "Test Pattern".to_string(),
            description: None,
            is_active: true,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            pattern_type: "weekly".to_string(),
            cycle_length_days: Some(7),
        };

        let json = serde_json::to_string(&pattern).unwrap();
        assert!(json.contains("Test Pattern"));
        assert!(json.contains("weekly"));
        assert!(json.contains("cycle_length_days"));
    }

    #[test]
    fn test_effective_schedule_serialization() {
        let schedule = EffectiveSchedule {
            date: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            shift_id: Some(Uuid::new_v4()),
            source: "roster".to_string(),
            shift_name: Some("Morning Shift".to_string()),
            shift_symbol: Some("M".to_string()),
            start_time: NaiveTime::from_hms_opt(8, 0, 0),
            end_time: NaiveTime::from_hms_opt(17, 0, 0),
            break_duration_minutes: Some(60),
            break_start: None,
            break_end: None,
            is_break_auto_deduct: Some(true),
            early_punch_in_grace_period_minutes: None,
            allow_early_ot: None,
            break_is_overtime: None,
            max_early_ot_minutes: None,
            traces: vec![],
            is_holiday: false,
            is_holiday_paid: false,
            holiday_name: None,
        };

        let json = serde_json::to_string(&schedule).unwrap();
        assert!(json.contains("Morning Shift"));
        assert!(json.contains("roster"));
    }
}
