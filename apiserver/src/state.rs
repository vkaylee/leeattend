use std::sync::Arc;

use crate::agent_manager::AgentManager;
use crate::db_pool::DbPools;
use crate::repositories::audit_log_repository::AuditLogRepository;
use crate::repositories::department_repository::DepartmentRepository;
use crate::repositories::holiday_repository::HolidayRepositoryTrait;
use crate::repositories::import_job_repository::ImportJobRepository;
use crate::repositories::leave_policy_repository::LeavePolicyRepository;
use crate::repositories::leave_request_repository::LeaveRequestRepository;
use crate::repositories::payroll_repository::PayrollRepository;
use crate::repositories::raw_attendance_log_repository::RawAttendanceLogRepository;
use crate::repositories::roster_repository::RosterRepository;
use crate::repositories::shift_swap_repository::ShiftSwapRepository;
use crate::repositories::staging_log_repository::StagingLogRepository;
use crate::repositories::time_entry_repository::TimeEntryRepository;
use crate::repositories::user_repository::UserRepository;
use crate::repositories::{AgentRepository, EmployeeRepository, WorkspaceRepository};
use crate::services::attendance_adjustment_service::AttendanceAdjustmentService;
use crate::services::attendance_import_service::AttendanceImportService;
use crate::services::auth_service::AuthService;
use crate::services::holiday_leave_service::HolidayLeaveService;
use crate::services::leave_accrual_service::LeaveAccrualService;
use crate::services::leave_service::LeaveService;
use crate::services::pairing_service::PairingService;
use crate::services::payroll_service::PayrollService;
use crate::services::shift_service::ShiftService;
use crate::services::shift_swap_service::ShiftSwapService;
use crate::services::workspace_service::WorkspaceService;
use crate::services::{
    AgentService, AttendanceLogService, AttendanceSummaryService, EmployeeService,
    ReportCoordinator, ReportService, TimeTrackingService,
};

pub use crate::queue::RedisPool;

// ── Domain Sub-Structs ─────────────────────────────────────────────

/// Employee management — service + repository
pub struct EmployeeState {
    pub service: Arc<EmployeeService>,
    pub repo: Arc<dyn EmployeeRepository>,
}

/// Time & attendance tracking, summaries, logs, staging, adjustments, imports
pub struct AttendanceState {
    pub time_tracking_service: Arc<TimeTrackingService>,
    pub summary_service: Arc<AttendanceSummaryService>,
    pub log_service: Arc<AttendanceLogService>,
    pub pairing_service: Arc<PairingService>,
    pub adjustment_service: Arc<AttendanceAdjustmentService>,
    pub import_service: Arc<AttendanceImportService>,
    pub time_entry_repo: Arc<dyn TimeEntryRepository>,
    pub staging_repo: Arc<dyn StagingLogRepository>,
    pub raw_log_repo: Arc<dyn RawAttendanceLogRepository>,
    pub import_repo: Arc<dyn ImportJobRepository>,
}

/// Shift management — shifts, rosters, swaps, patterns
pub struct ShiftState {
    pub service: Arc<ShiftService>,
    pub swap_service: Arc<ShiftSwapService>,
    pub roster_repo: Arc<dyn RosterRepository>,
    pub swap_repo: Arc<dyn ShiftSwapRepository>,
}

/// Leave management — requests, policies, holidays, accrual
pub struct LeaveState {
    pub service: Arc<LeaveService>,
    pub holiday_leave_service: Arc<HolidayLeaveService>,
    pub accrual_service: Arc<LeaveAccrualService>,
    pub request_repo: Arc<dyn LeaveRequestRepository>,
    pub policy_repo: Arc<dyn LeavePolicyRepository>,
    pub holiday_repo: Arc<dyn HolidayRepositoryTrait>,
}

/// Payroll — calculation, settings, repos
pub struct PayrollState {
    pub service: Arc<PayrollService>,
    pub repo: Arc<dyn PayrollRepository>,
    pub department_repo: Arc<dyn DepartmentRepository>,
}

/// Reports & async coordination
pub struct ReportState {
    pub service: Arc<ReportService>,
    pub coordinator: Option<Arc<ReportCoordinator>>,
}

/// Auth, workspace, user management
pub struct CoreState {
    pub auth_service: Arc<AuthService>,
    pub workspace_service: Arc<WorkspaceService>,
    pub workspace_repo: Arc<dyn WorkspaceRepository>,
    pub audit_repo: Arc<dyn AuditLogRepository>,
    pub user_repo: Arc<dyn UserRepository>,
    pub jwt_secret: String,
}

/// AI agent management
pub struct AgentState {
    pub manager: AgentManager,
    pub service: Arc<AgentService>,
    pub repo: Arc<dyn AgentRepository>,
}

/// Infrastructure — DB, Redis, caches
pub struct InfraState {
    pub db: DbPools,
    pub redis: Option<RedisPool>,
    pub roster_cache: Option<Arc<crate::cache::RosterCache>>,
}

// ── Composed AppState ──────────────────────────────────────────────

pub struct AppState {
    pub employees: EmployeeState,
    pub attendance: AttendanceState,
    pub shifts: ShiftState,
    pub leaves: LeaveState,
    pub payroll: PayrollState,
    pub reports: ReportState,
    pub core: CoreState,
    pub agents: AgentState,
    pub infra: InfraState,
}
