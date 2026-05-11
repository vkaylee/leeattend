use sqlx::PgPool;

/// Enforces that every repository implementation is constructed from a `PgPool`.
///
/// All repository traits (e.g. `EmployeeRepository`, `TimeEntryRepository`)
/// extend `PoolBacked` as a supertrait. This means any new concrete repo
/// struct **must** implement `from_pool()` or the compiler will reject it.
///
/// # Example
/// ```ignore
/// impl PoolBacked for SqlxNewRepository {
///     fn from_pool(pool: PgPool) -> Self { Self { pool } }
///     fn pool(&self) -> &PgPool { &self.pool }
/// }
/// ```
pub trait PoolBacked: Send + Sync {
    /// Construct this repository from a database pool.
    fn from_pool(pool: PgPool) -> Self
    where
        Self: Sized;
    /// Access the underlying connection pool.
    fn pool(&self) -> &PgPool;
}

pub mod agent_repository;
pub mod assignment_rule_repository;
pub mod attendance_adjustment_repository;
pub mod department_repository;
pub mod diesel_department_repository;
pub mod employee;
pub use employee::{EmployeeRepository, SqlxEmployeeRepository};
pub mod feature_flag_repository;
pub mod holiday_repository;
pub mod import_job_repository;
pub mod leave_balance_repository;
pub mod leave_request_repository;
pub mod payroll_repository;
pub use payroll_repository::{PayrollRepository, SqlxPayrollRepository};
pub mod raw_attendance_log_repository;
pub mod roster;
pub use roster::{RosterRepository, SqlxRosterRepository};
pub mod shift_repository;
pub mod shift_swap_repository;
pub mod time_entry_repository;
pub use time_entry_repository::{SqlxTimeEntryRepository, TimeEntryRepository};
pub mod employee_contract_repository;
pub mod employment_phase_repository;
pub mod payroll_rule_repository;
pub mod salary_base_repository;
pub mod user_repository;
pub mod working_pattern_repository;
pub mod workspace_repository;

pub use agent_repository::{AgentRepository, SqlxAgentRepository};
pub use employee_contract_repository::*;
pub use employment_phase_repository::EmploymentPhaseRepository;
pub use payroll_rule_repository::PayrollRuleRepository;
pub use user_repository::UserRepository;
pub use workspace_repository::{SqlxWorkspaceRepository, WorkspaceRepository};
pub mod attendance_log_error_repository;
pub mod audit_log_repository;
pub mod leave_accrual_log_repository;
pub mod leave_policy_repository;
pub mod overtime_policy_repository;
pub mod staging_log_repository;
pub use audit_log_repository::AuditLogRepository;
pub mod attendance_policy_repository;
pub use attendance_policy_repository::{
    AttendancePolicyRepository, SqlxAttendancePolicyRepository,
};
pub mod workspace_stats_repository;
pub use workspace_stats_repository::{SqlxWorkspaceStatsRepository, WorkspaceStatsRepository};
pub mod diesel_employee_contract_repository;
pub mod diesel_employee_repository;
pub use diesel_employee_contract_repository::DieselEmployeeContractRepository;
pub use diesel_employee_repository::DieselEmployeeRepository;
pub mod diesel_agent_repository;
pub use diesel_agent_repository::DieselAgentRepository;
pub mod diesel_audit_log_repository;
pub use diesel_audit_log_repository::DieselAuditLogRepository;
pub mod diesel_attendance_policy_repository;
pub use diesel_attendance_policy_repository::DieselAttendancePolicyRepository;
pub mod diesel_time_entry_repository;
pub use diesel_time_entry_repository::DieselTimeEntryRepository;
pub mod diesel_payroll_repository;
pub use diesel_payroll_repository::DieselPayrollRepository;
pub mod diesel_leave_request_repository;
pub use diesel_leave_request_repository::DieselLeaveRequestRepository;
pub mod diesel_leave_balance_repository;
pub use diesel_leave_balance_repository::DieselLeaveBalanceRepository;
pub mod diesel_leave_policy_repository;
pub use diesel_leave_policy_repository::DieselLeavePolicyRepository;
pub mod diesel_leave_accrual_log_repository;
pub use diesel_leave_accrual_log_repository::DieselLeaveAccrualLogRepository;
pub mod diesel_attendance_adjustment_repository;
pub use diesel_attendance_adjustment_repository::DieselAttendanceAdjustmentRepository;
pub mod diesel_raw_attendance_log_repository;
pub use diesel_raw_attendance_log_repository::DieselRawAttendanceLogRepository;
pub mod diesel_staging_log_repository;
pub use diesel_staging_log_repository::DieselStagingLogRepository;
pub mod diesel_attendance_log_error_repository;
pub use diesel_attendance_log_error_repository::DieselAttendanceLogErrorRepository;
pub mod diesel_payroll_rule_repository;
pub use diesel_payroll_rule_repository::DieselPayrollRuleRepository;
pub mod diesel_user_repository;
pub use diesel_user_repository::DieselUserRepository;
pub mod diesel_workspace_repository;
pub use diesel_workspace_repository::DieselWorkspaceRepository;
pub mod session_repository;
pub use session_repository::SessionRepository;
pub mod diesel_session_repository;
pub use diesel_session_repository::DieselSessionRepository;
pub mod diesel_roster_repository;
pub use diesel_roster_repository::DieselRosterRepository;
pub mod diesel_shift_repository;
pub use diesel_shift_repository::DieselShiftRepository;
pub mod diesel_shift_swap_repository;
pub use diesel_shift_swap_repository::DieselShiftSwapRepository;
pub mod diesel_holiday_repository;
pub use diesel_holiday_repository::DieselHolidayRepository;
pub mod diesel_feature_flag_repository;
pub use diesel_feature_flag_repository::DieselFeatureFlagRepository;
pub mod diesel_assignment_rule_repository;
pub use diesel_assignment_rule_repository::DieselAssignmentRuleRepository;
