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
    fn from_pool(pool: PgPool) -> Self where Self: Sized;
    /// Access the underlying connection pool.
    fn pool(&self) -> &PgPool;
}

pub mod agent_repository;
pub mod assignment_rule_repository;
pub mod attendance_adjustment_repository;
pub mod department_repository;
pub mod employee_repository;
pub use employee_repository::{EmployeeRepository, SqlxEmployeeRepository};
pub mod holiday_repository;
pub mod import_job_repository;
pub mod leave_balance_repository;
pub mod leave_request_repository;
pub mod payroll_repository;
pub use payroll_repository::{PayrollRepository, SqlxPayrollRepository};
pub mod raw_attendance_log_repository;
pub mod roster_repository;
pub mod shift_repository;
pub mod shift_swap_repository;
pub mod time_entry_repository;
pub use time_entry_repository::{TimeEntryRepository, SqlxTimeEntryRepository};
pub mod user_repository;
pub mod working_pattern_repository;
pub mod workspace_repository;

pub use agent_repository::{AgentRepository, SqlxAgentRepository};
pub use workspace_repository::{SqlxWorkspaceRepository, WorkspaceRepository};
pub mod attendance_log_error_repository;
pub mod leave_policy_repository;
pub mod overtime_policy_repository;
pub mod staging_log_repository;
pub mod audit_log_repository;
pub use audit_log_repository::{AuditLogRepository, SqlxAuditLogRepository};
