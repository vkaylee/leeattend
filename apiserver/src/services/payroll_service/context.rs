//! Payroll domain types and helper functions.
//!
//! Extracted from `payroll_service/mod.rs` to satisfy SRP.

use chrono::NaiveDate;
use chrono_tz::Tz;
use rust_decimal::Decimal;
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

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
    #[allow(dead_code)]
    pub(crate) active_contracts: Vec<crate::models::contract::EmployeeContract>,
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
// Helper functions
// ---------------------------------------------------------------------------

pub(crate) fn empty_batch_response() -> crate::models::BatchCalculatePayrollResponse {
    crate::models::BatchCalculatePayrollResponse { results: vec![] }
}

pub(crate) fn month_boundaries(year: i32, month: u32) -> (NaiveDate, NaiveDate) {
    let start_date = NaiveDate::from_ymd_opt(year, month, 1).unwrap_or_default();
    let next_month_date = if month == 12 {
        NaiveDate::from_ymd_opt(year + 1, 1, 1)
    } else {
        NaiveDate::from_ymd_opt(year, month + 1, 1)
    }
    .unwrap_or_default();
    let end_date = next_month_date.pred_opt().unwrap_or_default();
    (start_date, end_date)
}

pub(crate) fn default_attendance_rules() -> crate::models::AttendanceRules {
    crate::models::AttendanceRules {
        work_unit_minutes: 30,
        rounding_policy: "floor".to_string(),
        early_arrival_strategy: crate::models::EarlyArrivalStrategy::Block,
        grace_period_minutes: None,
    }
}

#[allow(clippy::trivially_copy_pass_by_ref)]
pub(crate) fn build_daily_entries_map(
    entries: Vec<crate::models::TimeEntry>,
    tz: &Tz,
) -> HashMap<NaiveDate, Vec<crate::models::TimeEntry>> {
    let mut map: HashMap<NaiveDate, Vec<crate::models::TimeEntry>> = HashMap::new();
    for entry in entries {
        let local_date = entry.punch_in_time().with_timezone(tz).date_naive();
        map.entry(local_date).or_default().push(entry);
    }
    map
}
