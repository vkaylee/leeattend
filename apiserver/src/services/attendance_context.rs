//! Pre-fetched data required by [`AttendanceMatcher`].
//!
//! Contains only the fields required for shift resolution, entry-to-shift
//! matching, and daily unit calculation. Has **zero dependency** on
//! `payroll_service` — constructed via [`AttendanceMatcherContext::from_payroll`].

use chrono::NaiveDate;
use chrono_tz::Tz;
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

use crate::cache::RosterData;
use crate::models::employee_assignment::EmployeeAssignment;
use crate::models::employment::Employment;
use crate::models::{
    ConditionTypeDefinition, EmployeeDepartmentHistory, Holiday, TimeEntry, WorkingCondition,
};
use crate::repositories::overtime_policy_repository::WorkspaceOvertimeData;

// ---------------------------------------------------------------------------
// AttendanceMatcherContext
// ---------------------------------------------------------------------------

/// Pre-fetched data required for attendance matching.
///
/// Constructed via [`AttendanceMatcherContext::from_payroll`] from a
/// `PayrollContext`. There is exactly one production call site
/// (`payslip_calc.rs`).
pub struct AttendanceMatcherContext {
    employee_id: Uuid,
    tz: Tz,

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
// Constructor
// ---------------------------------------------------------------------------

impl AttendanceMatcherContext {
    /// Build from a `PayrollContext`, extracting only the fields needed
    /// by the attendance matcher.
    ///
    /// This is the **single production constructor**. The payroll context
    /// carries a superset of this data; we clone only the Arc/Vec fields
    /// that the matcher reads.
    pub fn from_payroll(pc: &crate::services::payroll_service::PayrollContext) -> Self {
        Self {
            employee_id: pc.employee_id,
            tz: pc.tz,
            daily_entries_map: pc.daily_entries_map.clone(),
            holidays_map: Arc::clone(&pc.holidays_map),
            roster_data: Arc::clone(&pc.roster_data),
            emp_id_number: pc.emp_id_number.clone(),
            emp_tags: pc.emp_tags.clone(),
            payroll_roster_map: pc.payroll_roster_map.clone(),
            overtime_data: Arc::clone(&pc.overtime_data),
            active_assignments: pc.active_assignments.clone(),
            primary_location_id: pc.primary_location_id,
            all_dept_history: pc.all_dept_history.clone(),
            active_employments: pc.active_employments.clone(),
            working_conditions: pc.working_conditions.clone(),
            condition_type_catalog: Arc::clone(&pc.condition_type_catalog),
        }
    }
}

// ---------------------------------------------------------------------------
// Accessor methods
// ---------------------------------------------------------------------------

impl AttendanceMatcherContext {
    // ── Identity ─────────────────────────────────────────────────

    pub fn employee_id(&self) -> Uuid {
        self.employee_id
    }

    pub fn tz(&self) -> Tz {
        self.tz
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
// Test-only builder
// ---------------------------------------------------------------------------
//
// Production code uses `from_payroll` exclusively.  The builder exists
// solely so that tests (including cross-module tests in
// `attendance_matcher/tests_mod.rs`) can construct partial contexts
// without assembling a full `PayrollContext`.

#[cfg(test)]
#[must_use]
pub struct AttendanceMatcherContextBuilder {
    employee_id: Uuid,
    tz: Tz,
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

#[cfg(test)]
impl Default for AttendanceMatcherContextBuilder {
    fn default() -> Self {
        Self {
            employee_id: Uuid::nil(),
            tz: chrono_tz::UTC,
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

#[cfg(test)]
impl AttendanceMatcherContextBuilder {
    #[must_use]
    pub fn employee_id(mut self, v: Uuid) -> Self {
        self.employee_id = v;
        self
    }

    #[must_use]
    pub fn tz(mut self, v: Tz) -> Self {
        self.tz = v;
        self
    }

    #[must_use]
    pub fn daily_entries_map(mut self, v: HashMap<NaiveDate, Vec<TimeEntry>>) -> Self {
        self.daily_entries_map = v;
        self
    }

    #[must_use]
    pub fn holidays_map(mut self, v: Arc<HashMap<NaiveDate, Holiday>>) -> Self {
        self.holidays_map = v;
        self
    }

    #[must_use]
    pub fn roster_data(mut self, v: Arc<RosterData>) -> Self {
        self.roster_data = v;
        self
    }

    #[must_use]
    pub fn emp_id_number(mut self, v: String) -> Self {
        self.emp_id_number = v;
        self
    }

    #[must_use]
    pub fn emp_tags(mut self, v: Option<Vec<String>>) -> Self {
        self.emp_tags = v;
        self
    }

    #[must_use]
    pub fn payroll_roster_map(mut self, v: HashMap<(Uuid, NaiveDate), Option<Uuid>>) -> Self {
        self.payroll_roster_map = v;
        self
    }

    #[must_use]
    pub fn overtime_data(mut self, v: Arc<WorkspaceOvertimeData>) -> Self {
        self.overtime_data = v;
        self
    }

    #[must_use]
    pub fn active_assignments(mut self, v: Vec<EmployeeAssignment>) -> Self {
        self.active_assignments = v;
        self
    }

    #[must_use]
    pub fn primary_location_id(mut self, v: Uuid) -> Self {
        self.primary_location_id = v;
        self
    }

    #[must_use]
    pub fn active_employments(mut self, v: Vec<Employment>) -> Self {
        self.active_employments = v;
        self
    }

    #[must_use]
    pub fn build(self) -> AttendanceMatcherContext {
        AttendanceMatcherContext {
            employee_id: self.employee_id,
            tz: self.tz,
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

#[cfg(test)]
impl AttendanceMatcherContext {
    /// Create a test-only builder with sensible defaults.
    pub fn builder() -> AttendanceMatcherContextBuilder {
        AttendanceMatcherContextBuilder::default()
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::enums::EarlyArrivalStrategy;
    use chrono::Utc;

    #[test]
    fn test_from_payroll_preserves_all_fields() {
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
            daily_summaries: HashMap::new(),
        };

        let att = AttendanceMatcherContext::from_payroll(&pc);

        // Verify all 15 fields are correctly bridged
        assert_eq!(att.employee_id(), emp_id);
        assert_eq!(att.tz(), chrono_tz::Asia::Ho_Chi_Minh);
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

    #[test]
    fn test_builder_sets_fields() {
        let emp_id = Uuid::new_v4();
        let loc_id = Uuid::new_v4();
        let date = NaiveDate::from_ymd_opt(2026, 1, 1).unwrap();

        let mut entries = HashMap::new();
        entries.insert(date, vec![]);
        let tags = vec!["tag1".to_string(), "tag2".to_string()];

        let ctx = AttendanceMatcherContext::builder()
            .employee_id(emp_id)
            .tz(chrono_tz::Asia::Ho_Chi_Minh)
            .daily_entries_map(entries)
            .emp_id_number("EMP-001".to_string())
            .emp_tags(Some(tags))
            .primary_location_id(loc_id)
            .build();

        assert_eq!(ctx.employee_id(), emp_id);
        assert_eq!(ctx.tz(), chrono_tz::Asia::Ho_Chi_Minh);
        assert_eq!(ctx.daily_entries_map().len(), 1);
        assert!(ctx.daily_entries_map().contains_key(&date));
        assert_eq!(ctx.emp_id_number(), "EMP-001");
        assert_eq!(ctx.emp_tags().unwrap().len(), 2);
        assert_eq!(ctx.primary_location_id(), loc_id);
    }
}
