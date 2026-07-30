//! Payroll domain types and helper functions.
//!
//! Extracted from `payroll_service/mod.rs` to satisfy SRP.

use chrono::NaiveDate;
use chrono_tz::Tz;
use rust_decimal::Decimal;
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

use super::super::attendance_context::AttendanceContext;
use super::super::financial_calculator::{IncentivePolicyWithTiers, RuleAssignmentWithTiers};

// ---------------------------------------------------------------------------
// Core context types
// ---------------------------------------------------------------------------

/// All pre-fetched data required for payroll calculation.
/// Constructed once by `fetch_context()` and passed to subsequent phases.
pub struct PayrollContext {
    pub(crate) employee_id: Uuid,
    pub(crate) workspace: crate::models::Workspace,
    pub(crate) tz: Tz,
    pub(crate) start_date: NaiveDate,
    pub(crate) end_date: NaiveDate,
    pub(crate) salary_history: Vec<(
        crate::models::EmployeeSalaryComponent,
        crate::models::SalaryComponentDefinition,
    )>,
    pub(crate) all_dept_history: Vec<crate::models::EmployeeDepartmentHistory>,
    pub(crate) daily_entries_map: HashMap<NaiveDate, Vec<crate::models::TimeEntry>>,
    pub(crate) holidays_map: Arc<HashMap<NaiveDate, crate::models::Holiday>>,
    pub(crate) roster_data: Arc<crate::cache::RosterData>,
    pub(crate) emp_id_number: String,
    pub(crate) emp_tags: Option<Vec<String>>,
    pub(crate) payroll_roster_map: HashMap<(Uuid, NaiveDate), Option<Uuid>>,
    pub(crate) working_conditions: Vec<crate::models::WorkingCondition>,
    pub(crate) condition_type_catalog: Arc<HashMap<String, crate::models::ConditionTypeDefinition>>,
    pub(crate) attendance_policy: crate::models::AttendanceRules,
    pub(crate) emp_payroll_configs: Vec<crate::models::EmployeePayrollConfig>,
    pub(crate) ws_payroll_settings_history: Arc<Vec<crate::models::PayrollSettings>>,
    pub(crate) department_payroll_configs:
        HashMap<Uuid, Vec<crate::models::DepartmentPayrollConfig>>,
    pub(crate) overtime_data:
        Arc<crate::repositories::overtime_policy_repository::WorkspaceOvertimeData>,
    pub(crate) ot_base_component_ids: std::collections::HashSet<Uuid>,
    pub(crate) active_assignments: Vec<crate::models::employee_assignment::EmployeeAssignment>,
    pub(crate) active_employments: Vec<crate::models::employment::Employment>,
    pub(crate) primary_location_id: Uuid,
    pub(crate) month_end_payroll_settings: Arc<Option<crate::models::PayrollSettings>>,
    pub(crate) month_end_attendance_policy: crate::models::AttendanceRules,
    pub(crate) leave_breakdown: crate::repositories::leave_request_repository::LeaveBreakdown,
    pub(crate) active_policies_with_tiers: Arc<Vec<IncentivePolicyWithTiers>>,
    pub(crate) salary_base_groups: Arc<Vec<crate::models::SalaryBaseGroup>>,
    pub(crate) unpaid_holiday_days: i64,
    pub(crate) rule_assignments_with_tiers: Vec<RuleAssignmentWithTiers>,
}

/// Named record for a payroll segment snapshot.
///
/// Replaces the 10-element tuple that was previously used for
/// `segment_snapshots`, giving each field a meaningful name.
#[derive(Debug, Clone)]
pub struct SegmentSnapshot {
    pub(crate) seg_start: NaiveDate,
    pub(crate) seg_end: NaiveDate,
    pub(crate) department_id: Option<Uuid>,
    pub(crate) total_base: Decimal,
    pub(crate) std_working_days: Decimal,
    pub(crate) hours_per_day: Decimal,
    pub(crate) actual_worked_days: Decimal,
    pub(crate) paid_days: Decimal,
    pub(crate) amount: Decimal,
    pub(crate) source: String,
}

#[async_trait::async_trait]
pub trait RosterDataProvider: Send + Sync {
    async fn get_roster_data(
        &self,
        workspace_id: Uuid,
    ) -> Result<Arc<crate::cache::RosterData>, crate::errors::AppError>;
}

/// Concrete `RosterDataProvider` that calls `RosterRepository::get_raw_roster_data`.
/// Used in production (`app_builder`) and in integration tests.
pub struct DirectRosterDataProvider(pub Arc<dyn crate::repositories::roster::RosterRepository>);

#[async_trait::async_trait]
impl RosterDataProvider for DirectRosterDataProvider {
    async fn get_roster_data(
        &self,
        workspace_id: Uuid,
    ) -> Result<Arc<crate::cache::RosterData>, crate::errors::AppError> {
        self.0.get_raw_roster_data(workspace_id).await
    }
}

/// Immutable data shared across all chunks in a batch payroll run.
/// Fields that are identical for all employments are wrapped in `Arc`
/// so that per-employment `PayrollContext` construction only increments
/// reference counts instead of deep-cloning.
pub(crate) struct ChunkSharedData {
    pub(crate) workspace: crate::models::Workspace,
    pub(crate) tz: Tz,
    pub(crate) start_date: NaiveDate,
    pub(crate) end_date: NaiveDate,
    pub(crate) start_dt: chrono::DateTime<chrono::Utc>,
    pub(crate) end_dt: chrono::DateTime<chrono::Utc>,
    pub(crate) token: crate::diesel_pool::TenantAuthToken,
    pub(crate) condition_type_catalog: Arc<HashMap<String, crate::models::ConditionTypeDefinition>>,
    pub(crate) overtime_data:
        Arc<crate::repositories::overtime_policy_repository::WorkspaceOvertimeData>,
    pub(crate) holidays_map: Arc<HashMap<NaiveDate, crate::models::Holiday>>,
    pub(crate) roster_data: Arc<crate::cache::RosterData>,
    pub(crate) ws_payroll_settings_history: Arc<Vec<crate::models::PayrollSettings>>,
    pub(crate) month_end_payroll_settings: Arc<Option<crate::models::PayrollSettings>>,
    pub(crate) active_policies_with_tiers: Arc<Vec<IncentivePolicyWithTiers>>,
    pub(crate) salary_base_groups: Arc<Vec<crate::models::SalaryBaseGroup>>,
}

// ---------------------------------------------------------------------------
// AttendanceContext conversion — payroll → attendance bridge
// ---------------------------------------------------------------------------

impl From<&PayrollContext> for AttendanceContext {
    fn from(pc: &PayrollContext) -> Self {
        Self {
            employee_id: pc.employee_id,
            tz: pc.tz,
            start_date: pc.start_date,
            end_date: pc.end_date,
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
// Helper functions (re-exported from attendance_context)
// ---------------------------------------------------------------------------

pub(crate) use super::super::attendance_context::{
    build_daily_entries_map, default_attendance_rules, month_boundaries,
};
