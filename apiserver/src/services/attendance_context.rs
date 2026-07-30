//! Standalone attendance computation context.
//!
//! Extracted from `PayrollContext` to decouple attendance calculation
//! from payroll/financial data. Contains only the fields required by
//! `AttendanceMatcher`, `AttendanceCalculator`, and related services.
//!
//! Has **zero dependency** on `payroll_service` — the conversion from
//! `PayrollContext` lives in `payroll_service::context` via `From`.

use chrono::NaiveDate;
use chrono_tz::Tz;
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

use crate::cache::RosterData;
use crate::errors::AppError;
use crate::models::attendance::AttendanceRules;
use crate::models::employee_assignment::EmployeeAssignment;
use crate::models::employment::Employment;
use crate::models::enums::EarlyArrivalStrategy;
use crate::models::{
    ConditionTypeDefinition, EmployeeDepartmentHistory, Holiday, TimeEntry, WorkingCondition,
};
use crate::repositories::overtime_policy_repository::WorkspaceOvertimeData;
use crate::utils::i18n::I18nKey;

// ---------------------------------------------------------------------------
// AttendanceContext
// ---------------------------------------------------------------------------

/// Pre-fetched data required for attendance computation.
///
/// Use [`AttendanceContext::builder()`] to construct — the builder provides
/// sensible defaults for all collection fields (empty maps, empty vecs).
///
/// The `From<&PayrollContext>` bridge in `payroll_service::context` also
/// delegates to the builder.
pub struct AttendanceContext {
    employee_id: Uuid,
    tz: Tz,
    start_date: NaiveDate,
    end_date: NaiveDate,

    // ── Attendance-only ──────────────────────────────────────────
    daily_entries_map: HashMap<NaiveDate, Vec<TimeEntry>>,
    holidays_map: Arc<HashMap<NaiveDate, Holiday>>,
    roster_data: Arc<RosterData>,
    emp_id_number: String,
    emp_tags: Option<Vec<String>>,
    payroll_roster_map: HashMap<(Uuid, NaiveDate), Option<Uuid>>,
    overtime_data: Arc<WorkspaceOvertimeData>,
    active_assignments: Vec<EmployeeAssignment>,
    primary_location_id: Uuid,

    // ── Shared with payroll (cheap to clone per-employee) ────────
    all_dept_history: Vec<EmployeeDepartmentHistory>,
    active_employments: Vec<Employment>,
    working_conditions: Vec<WorkingCondition>,
    condition_type_catalog: Arc<HashMap<String, ConditionTypeDefinition>>,
}

// ---------------------------------------------------------------------------
// Builder
// ---------------------------------------------------------------------------

/// Fluent builder for [`AttendanceContext`].
///
/// All collection fields default to empty. Call `.build()` to produce
/// an `AttendanceContext`.
///
/// # Example
///
/// ```ignore
/// let ctx = AttendanceContext::builder()
///     .employee_id(id)
///     .tz(tz)
///     .start_date(start)
///     .end_date(end)
///     .emp_id_number("EMP001".into())
///     .daily_entries_map(entries)
///     .build();
/// ```
#[must_use]
pub struct AttendanceContextBuilder {
    employee_id: Uuid,
    tz: Tz,
    start_date: NaiveDate,
    end_date: NaiveDate,
    daily_entries_map: HashMap<NaiveDate, Vec<TimeEntry>>,
    holidays_map: Arc<HashMap<NaiveDate, Holiday>>,
    roster_data: Arc<RosterData>,
    emp_id_number: String,
    emp_tags: Option<Vec<String>>,
    payroll_roster_map: HashMap<(Uuid, NaiveDate), Option<Uuid>>,
    overtime_data: Arc<WorkspaceOvertimeData>,
    active_assignments: Vec<EmployeeAssignment>,
    primary_location_id: Uuid,
    all_dept_history: Vec<EmployeeDepartmentHistory>,
    active_employments: Vec<Employment>,
    working_conditions: Vec<WorkingCondition>,
    condition_type_catalog: Arc<HashMap<String, ConditionTypeDefinition>>,
}

impl Default for AttendanceContextBuilder {
    fn default() -> Self {
        Self {
            employee_id: Uuid::nil(),
            tz: chrono_tz::UTC,
            start_date: NaiveDate::from_ymd_opt(2000, 1, 1).unwrap_or_default(),
            end_date: NaiveDate::from_ymd_opt(2000, 1, 1).unwrap_or_default(),
            daily_entries_map: HashMap::new(),
            holidays_map: Arc::new(HashMap::new()),
            roster_data: Arc::new(RosterData::default()),
            emp_id_number: String::new(),
            emp_tags: None,
            payroll_roster_map: HashMap::new(),
            overtime_data: Arc::new(WorkspaceOvertimeData::default()),
            active_assignments: Vec::new(),
            primary_location_id: Uuid::nil(),
            all_dept_history: Vec::new(),
            active_employments: Vec::new(),
            working_conditions: Vec::new(),
            condition_type_catalog: Arc::new(HashMap::new()),
        }
    }
}

#[allow(clippy::return_self_not_must_use)]
impl AttendanceContextBuilder {
    pub fn employee_id(mut self, v: Uuid) -> Self {
        self.employee_id = v;
        self
    }

    pub fn tz(mut self, v: Tz) -> Self {
        self.tz = v;
        self
    }

    pub fn start_date(mut self, v: NaiveDate) -> Self {
        self.start_date = v;
        self
    }

    pub fn end_date(mut self, v: NaiveDate) -> Self {
        self.end_date = v;
        self
    }

    pub fn daily_entries_map(mut self, v: HashMap<NaiveDate, Vec<TimeEntry>>) -> Self {
        self.daily_entries_map = v;
        self
    }

    pub fn holidays_map(mut self, v: Arc<HashMap<NaiveDate, Holiday>>) -> Self {
        self.holidays_map = v;
        self
    }

    pub fn roster_data(mut self, v: Arc<RosterData>) -> Self {
        self.roster_data = v;
        self
    }

    pub fn emp_id_number(mut self, v: String) -> Self {
        self.emp_id_number = v;
        self
    }

    pub fn emp_tags(mut self, v: Option<Vec<String>>) -> Self {
        self.emp_tags = v;
        self
    }

    pub fn payroll_roster_map(mut self, v: HashMap<(Uuid, NaiveDate), Option<Uuid>>) -> Self {
        self.payroll_roster_map = v;
        self
    }

    pub fn overtime_data(mut self, v: Arc<WorkspaceOvertimeData>) -> Self {
        self.overtime_data = v;
        self
    }

    pub fn active_assignments(mut self, v: Vec<EmployeeAssignment>) -> Self {
        self.active_assignments = v;
        self
    }

    pub fn primary_location_id(mut self, v: Uuid) -> Self {
        self.primary_location_id = v;
        self
    }

    pub fn all_dept_history(mut self, v: Vec<EmployeeDepartmentHistory>) -> Self {
        self.all_dept_history = v;
        self
    }

    pub fn active_employments(mut self, v: Vec<Employment>) -> Self {
        self.active_employments = v;
        self
    }

    pub fn working_conditions(mut self, v: Vec<WorkingCondition>) -> Self {
        self.working_conditions = v;
        self
    }

    pub fn condition_type_catalog(
        mut self,
        v: Arc<HashMap<String, ConditionTypeDefinition>>,
    ) -> Self {
        self.condition_type_catalog = v;
        self
    }

    /// Consume the builder and produce an [`AttendanceContext`].
    pub fn build(self) -> AttendanceContext {
        AttendanceContext {
            employee_id: self.employee_id,
            tz: self.tz,
            start_date: self.start_date,
            end_date: self.end_date,
            daily_entries_map: self.daily_entries_map,
            holidays_map: self.holidays_map,
            roster_data: self.roster_data,
            emp_id_number: self.emp_id_number,
            emp_tags: self.emp_tags,
            payroll_roster_map: self.payroll_roster_map,
            overtime_data: self.overtime_data,
            active_assignments: self.active_assignments,
            primary_location_id: self.primary_location_id,
            all_dept_history: self.all_dept_history,
            active_employments: self.active_employments,
            working_conditions: self.working_conditions,
            condition_type_catalog: self.condition_type_catalog,
        }
    }
}

// ---------------------------------------------------------------------------
// Accessor methods
// ---------------------------------------------------------------------------

impl AttendanceContext {
    /// Create a new builder with sensible defaults.
    pub fn builder() -> AttendanceContextBuilder {
        AttendanceContextBuilder::default()
    }

    // ── Identity ─────────────────────────────────────────────────

    pub fn employee_id(&self) -> Uuid {
        self.employee_id
    }

    pub fn tz(&self) -> Tz {
        self.tz
    }

    pub fn start_date(&self) -> NaiveDate {
        self.start_date
    }

    pub fn end_date(&self) -> NaiveDate {
        self.end_date
    }

    // ── Attendance data ──────────────────────────────────────────

    pub fn daily_entries_map(&self) -> &HashMap<NaiveDate, Vec<TimeEntry>> {
        &self.daily_entries_map
    }

    pub fn holidays_map(&self) -> &Arc<HashMap<NaiveDate, Holiday>> {
        &self.holidays_map
    }

    pub fn roster_data(&self) -> &Arc<RosterData> {
        &self.roster_data
    }

    pub fn emp_id_number(&self) -> &str {
        &self.emp_id_number
    }

    pub fn emp_tags(&self) -> Option<&Vec<String>> {
        self.emp_tags.as_ref()
    }

    pub fn payroll_roster_map(&self) -> &HashMap<(Uuid, NaiveDate), Option<Uuid>> {
        &self.payroll_roster_map
    }

    pub fn overtime_data(&self) -> &Arc<WorkspaceOvertimeData> {
        &self.overtime_data
    }

    pub fn active_assignments(&self) -> &[EmployeeAssignment] {
        &self.active_assignments
    }

    pub fn primary_location_id(&self) -> Uuid {
        self.primary_location_id
    }

    // ── Shared with payroll ──────────────────────────────────────

    pub fn all_dept_history(&self) -> &[EmployeeDepartmentHistory] {
        &self.all_dept_history
    }

    pub fn active_employments(&self) -> &[Employment] {
        &self.active_employments
    }

    pub fn working_conditions(&self) -> &[WorkingCondition] {
        &self.working_conditions
    }

    pub fn condition_type_catalog(&self) -> &Arc<HashMap<String, ConditionTypeDefinition>> {
        &self.condition_type_catalog
    }
}

// ---------------------------------------------------------------------------
// Shared helpers (moved from payroll_service::context)
// ---------------------------------------------------------------------------

/// Compute the start and end `NaiveDate` for a year/month pair.
pub(crate) fn month_boundaries(year: i32, month: u32) -> Result<(NaiveDate, NaiveDate), AppError> {
    let start_date = NaiveDate::from_ymd_opt(year, month, 1)
        .ok_or_else(|| AppError::validation(I18nKey::InvalidYearMonth))?;
    let next_month_date = if month == 12 {
        NaiveDate::from_ymd_opt(year + 1, 1, 1)
    } else {
        NaiveDate::from_ymd_opt(year, month + 1, 1)
    }
    .ok_or_else(|| AppError::validation(I18nKey::InvalidYearMonth))?;
    let end_date = next_month_date
        .pred_opt()
        .ok_or_else(|| AppError::validation(I18nKey::InvalidYearMonth))?;
    Ok((start_date, end_date))
}

/// Fallback attendance rules used when no policy is assigned.
pub(crate) fn default_attendance_rules() -> AttendanceRules {
    AttendanceRules {
        work_unit_minutes: 30,
        rounding_policy: crate::models::RoundingPolicy::Floor,
        early_arrival_strategy: EarlyArrivalStrategy::Block,
        grace_period_minutes: None,
    }
}

/// Group time entries by local calendar date.
#[allow(clippy::trivially_copy_pass_by_ref)]
pub(crate) fn build_daily_entries_map(
    entries: Vec<TimeEntry>,
    tz: &Tz,
) -> HashMap<NaiveDate, Vec<TimeEntry>> {
    let mut map: HashMap<NaiveDate, Vec<TimeEntry>> = HashMap::new();
    for entry in entries {
        let local_date = entry.punch_in_time().with_timezone(tz).date_naive();
        map.entry(local_date).or_default().push(entry);
    }
    map
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[test]
    fn test_builder_defaults() {
        let ctx = AttendanceContext::builder().build();

        assert_eq!(ctx.employee_id(), Uuid::nil());
        assert_eq!(ctx.tz(), chrono_tz::UTC);
        assert_eq!(
            ctx.start_date(),
            NaiveDate::from_ymd_opt(2000, 1, 1).unwrap()
        );
        assert_eq!(ctx.end_date(), NaiveDate::from_ymd_opt(2000, 1, 1).unwrap());
        assert!(ctx.daily_entries_map().is_empty());
        assert!(ctx.holidays_map().is_empty());
        assert!(ctx.roster_data().shifts.is_empty());
        assert!(ctx.roster_data().shift_intervals.is_empty());
        assert_eq!(ctx.emp_id_number(), "");
        assert!(ctx.emp_tags().is_none());
        assert!(ctx.payroll_roster_map().is_empty());
        assert!(ctx.overtime_data().policies.is_empty());
        assert!(ctx.overtime_data().assignments.is_empty());
        assert!(ctx.active_assignments().is_empty());
        assert_eq!(ctx.primary_location_id(), Uuid::nil());
        assert!(ctx.all_dept_history().is_empty());
        assert!(ctx.active_employments().is_empty());
        assert!(ctx.working_conditions().is_empty());
        assert!(ctx.condition_type_catalog().is_empty());
    }

    #[test]
    fn test_builder_sets_all_fields() {
        let emp_id = Uuid::new_v4();
        let loc_id = Uuid::new_v4();
        let start = NaiveDate::from_ymd_opt(2026, 1, 1).unwrap();
        let end = NaiveDate::from_ymd_opt(2026, 1, 31).unwrap();

        let mut entries = HashMap::new();
        entries.insert(start, vec![]);
        let tags = vec!["tag1".to_string(), "tag2".to_string()];

        let ctx = AttendanceContext::builder()
            .employee_id(emp_id)
            .tz(chrono_tz::Asia::Ho_Chi_Minh)
            .start_date(start)
            .end_date(end)
            .daily_entries_map(entries)
            .emp_id_number("EMP-001".to_string())
            .emp_tags(Some(tags))
            .primary_location_id(loc_id)
            .build();

        assert_eq!(ctx.employee_id(), emp_id);
        assert_eq!(ctx.tz(), chrono_tz::Asia::Ho_Chi_Minh);
        assert_eq!(ctx.start_date(), start);
        assert_eq!(ctx.end_date(), end);
        assert_eq!(ctx.daily_entries_map().len(), 1);
        assert!(ctx.daily_entries_map().contains_key(&start));
        assert_eq!(ctx.emp_id_number(), "EMP-001");
        assert_eq!(ctx.emp_tags().unwrap().len(), 2);
        assert_eq!(ctx.primary_location_id(), loc_id);
    }

    #[test]
    fn test_from_payroll_context_preserves_fields() {
        use crate::services::payroll_service::PayrollContext;

        let emp_id = Uuid::new_v4();
        let loc_id = Uuid::new_v4();
        let start = NaiveDate::from_ymd_opt(2026, 3, 1).unwrap();
        let end = NaiveDate::from_ymd_opt(2026, 3, 31).unwrap();

        let workspace = crate::models::Workspace::new(
            Uuid::new_v4(),
            "Test WS".to_string(),
            "YYYY-MM-DD".to_string(),
            "HH:mm:ss".to_string(),
            "UTC".to_string(),
            "#000".to_string(),
            "en".to_string(),
            None,
            None,
            "SplitAtMidnight".to_string(),
            rust_decimal::Decimal::new(15, 1),
            Utc::now(),
            None,
        );

        let pc = PayrollContext {
            employee_id: emp_id,
            workspace,
            tz: chrono_tz::Asia::Ho_Chi_Minh,
            start_date: start,
            end_date: end,
            salary_history: vec![],
            all_dept_history: vec![],
            daily_entries_map: HashMap::new(),
            holidays_map: Arc::new(HashMap::new()),
            roster_data: Arc::new(crate::cache::RosterData::default()),
            emp_id_number: "EMP-042".to_string(),
            emp_tags: Some(vec!["senior".to_string()]),
            payroll_roster_map: HashMap::new(),
            working_conditions: vec![],
            condition_type_catalog: Arc::new(HashMap::new()),
            attendance_policy: crate::models::AttendanceRules {
                work_unit_minutes: 30,
                rounding_policy: crate::models::RoundingPolicy::Floor,
                early_arrival_strategy: EarlyArrivalStrategy::Block,
                grace_period_minutes: None,
            },
            emp_payroll_configs: vec![],
            ws_payroll_settings_history: Arc::new(vec![]),
            department_payroll_configs: HashMap::new(),
            overtime_data: Arc::new(WorkspaceOvertimeData::default()),
            ot_base_component_ids: std::collections::HashSet::new(),
            active_assignments: vec![],
            active_employments: vec![],
            primary_location_id: loc_id,
            month_end_payroll_settings: Arc::new(None),
            month_end_attendance_policy: crate::models::AttendanceRules {
                work_unit_minutes: 30,
                rounding_policy: crate::models::RoundingPolicy::Floor,
                early_arrival_strategy: EarlyArrivalStrategy::Block,
                grace_period_minutes: None,
            },
            leave_breakdown: crate::repositories::leave_request_repository::LeaveBreakdown {
                total_days: 0.0,
                unpaid_days: 0.0,
                sick_days: 0.0,
            },
            active_policies_with_tiers: Arc::new(vec![]),
            salary_base_groups: Arc::new(vec![]),
            unpaid_holiday_days: 0,
            rule_assignments_with_tiers: vec![],
        };

        let att = AttendanceContext::from(&pc);

        // Verify all 17 fields are correctly bridged
        assert_eq!(att.employee_id(), emp_id);
        assert_eq!(att.tz(), chrono_tz::Asia::Ho_Chi_Minh);
        assert_eq!(att.start_date(), start);
        assert_eq!(att.end_date(), end);
        assert!(att.daily_entries_map().is_empty());
        assert!(att.holidays_map().is_empty());
        assert!(att.roster_data().shifts.is_empty());
        assert_eq!(att.emp_id_number(), "EMP-042");
        assert_eq!(att.emp_tags(), Some(&vec!["senior".to_string()]));
        assert!(att.payroll_roster_map().is_empty());
        assert!(att.overtime_data().policies.is_empty());
        assert!(att.active_assignments().is_empty());
        assert_eq!(att.primary_location_id(), loc_id);
        assert!(att.all_dept_history().is_empty());
        assert!(att.active_employments().is_empty());
        assert!(att.working_conditions().is_empty());
        assert!(att.condition_type_catalog().is_empty());
    }
}
