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
    date: NaiveDate,
    shift_id: Option<Uuid>,   // None means Day Off
    source: String,            // "roster", "fixed", "department", "none"
    shift_name: Option<String>,
    shift_symbol: Option<String>,
    start_time: Option<NaiveTime>,
    end_time: Option<NaiveTime>,
    break_duration_minutes: Option<i32>,
    break_start: Option<NaiveTime>,
    break_end: Option<NaiveTime>,
    is_break_auto_deduct: Option<bool>,
    early_punch_in_grace_period_minutes: Option<i32>,
    allow_early_ot: Option<bool>,
    break_is_overtime: Option<bool>,
    max_early_ot_minutes: Option<i32>,
    #[serde(default)]
    traces: Vec<ScheduleTrace>,
    #[serde(default)]
    is_holiday: bool,
    #[serde(default)]
    is_holiday_paid: bool,
    #[serde(default)]
    holiday_name: Option<String>,
}

#[allow(clippy::too_many_arguments)]
impl EffectiveSchedule {
    pub fn new(
        date: NaiveDate,
        shift_id: Option<Uuid>,
        source: String,
        shift_name: Option<String>,
        shift_symbol: Option<String>,
        start_time: Option<NaiveTime>,
        end_time: Option<NaiveTime>,
        break_duration_minutes: Option<i32>,
        break_start: Option<NaiveTime>,
        break_end: Option<NaiveTime>,
        is_break_auto_deduct: Option<bool>,
        early_punch_in_grace_period_minutes: Option<i32>,
        allow_early_ot: Option<bool>,
        break_is_overtime: Option<bool>,
        max_early_ot_minutes: Option<i32>,
        traces: Vec<ScheduleTrace>,
        is_holiday: bool,
        is_holiday_paid: bool,
        holiday_name: Option<String>,
    ) -> Self {
        Self {
            date,
            shift_id,
            source,
            shift_name,
            shift_symbol,
            start_time,
            end_time,
            break_duration_minutes,
            break_start,
            break_end,
            is_break_auto_deduct,
            early_punch_in_grace_period_minutes,
            allow_early_ot,
            break_is_overtime,
            max_early_ot_minutes,
            traces,
            is_holiday,
            is_holiday_paid,
            holiday_name,
        }
    }

    // Getters
    pub fn date(&self) -> NaiveDate { self.date }
    pub fn shift_id(&self) -> Option<Uuid> { self.shift_id }
    pub fn source(&self) -> &str { &self.source }
    pub fn shift_name(&self) -> Option<&str> { self.shift_name.as_deref() }
    pub fn shift_symbol(&self) -> Option<&str> { self.shift_symbol.as_deref() }
    pub fn start_time(&self) -> Option<NaiveTime> { self.start_time }
    pub fn end_time(&self) -> Option<NaiveTime> { self.end_time }
    pub fn break_duration_minutes(&self) -> Option<i32> { self.break_duration_minutes }
    pub fn break_start(&self) -> Option<NaiveTime> { self.break_start }
    pub fn break_end(&self) -> Option<NaiveTime> { self.break_end }
    pub fn is_break_auto_deduct(&self) -> Option<bool> { self.is_break_auto_deduct }
    pub fn early_punch_in_grace_period_minutes(&self) -> Option<i32> { self.early_punch_in_grace_period_minutes }
    pub fn allow_early_ot(&self) -> Option<bool> { self.allow_early_ot }
    pub fn break_is_overtime(&self) -> Option<bool> { self.break_is_overtime }
    pub fn max_early_ot_minutes(&self) -> Option<i32> { self.max_early_ot_minutes }
    pub fn traces(&self) -> &[ScheduleTrace] { &self.traces }
    pub fn is_holiday(&self) -> bool { self.is_holiday }
    pub fn is_holiday_paid(&self) -> bool { self.is_holiday_paid }
    pub fn holiday_name(&self) -> Option<&str> { self.holiday_name.as_deref() }

    /// Populate shift details after construction (used by roster repository
    /// when resolving shift data post-construction).
    pub fn set_shift_details(&mut self, shift: &crate::models::Shift) {
        self.shift_name = Some(shift.name().to_string());
        self.shift_symbol = shift.report_symbol().map(String::from);
        self.start_time = Some(shift.start_time());
        self.end_time = Some(shift.end_time());
        self.break_duration_minutes = Some(shift.break_duration_minutes());
        self.break_start = shift.break_start();
        self.break_end = shift.break_end();
        self.is_break_auto_deduct = Some(shift.is_break_auto_deduct());
        self.early_punch_in_grace_period_minutes = Some(shift.early_punch_in_grace_period_minutes());
        self.allow_early_ot = Some(shift.allow_early_ot());
        self.break_is_overtime = Some(shift.break_is_overtime());
        self.max_early_ot_minutes = shift.max_early_ot_minutes();
    }

    /// Populate shift details from a cached `RosterShift`.
    pub fn set_shift_details_from_cache(&mut self, shift: &crate::cache::RosterShift) {
        self.shift_name = Some(shift.name().to_string());
        self.shift_symbol = shift.report_symbol().map(String::from);
        self.start_time = Some(shift.start_time());
        self.end_time = Some(shift.end_time());
        self.break_duration_minutes = Some(shift.break_duration_minutes());
        self.break_start = shift.break_start();
        self.break_end = shift.break_end();
        self.is_break_auto_deduct = Some(shift.is_break_auto_deduct());
        self.early_punch_in_grace_period_minutes = Some(shift.early_punch_in_grace_period_minutes());
        self.allow_early_ot = Some(shift.allow_early_ot());
        self.break_is_overtime = Some(shift.break_is_overtime());
        self.max_early_ot_minutes = shift.max_early_ot_minutes();
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, utoipa::ToSchema)]
pub struct EmployeeSchedule {
    employment_id: Uuid,
    employee_id: Uuid,
    employee_id_number: String,
    full_name: String,
    schedules: Vec<EffectiveSchedule>,
}

impl EmployeeSchedule {
    pub fn new(
        employment_id: Uuid,
        employee_id: Uuid,
        employee_id_number: String,
        full_name: String,
        schedules: Vec<EffectiveSchedule>,
    ) -> Self {
        Self { employment_id, employee_id, employee_id_number, full_name, schedules }
    }

    pub fn employment_id(&self) -> Uuid { self.employment_id }
    pub fn employee_id(&self) -> Uuid { self.employee_id }
    pub fn employee_id_number(&self) -> &str { &self.employee_id_number }
    pub fn full_name(&self) -> &str { &self.full_name }
    pub fn schedules(&self) -> &[EffectiveSchedule] { &self.schedules }
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
        let schedule = EffectiveSchedule::new(
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            Some(Uuid::new_v4()),
            "roster".to_string(),
            Some("Morning Shift".to_string()),
            Some("M".to_string()),
            NaiveTime::from_hms_opt(8, 0, 0),
            NaiveTime::from_hms_opt(17, 0, 0),
            Some(60),
            None,
            None,
            Some(true),
            None,
            None,
            None,
            None,
            vec![],
            false,
            false,
            None,
        );

        let json = serde_json::to_string(&schedule).unwrap();
        assert!(json.contains("Morning Shift"));
        assert!(json.contains("roster"));
    }
}
