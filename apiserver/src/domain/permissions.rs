// Role-based permission system for LeeAttend
//
//   MemberRole → HashSet<Permission>
//   Higher roles inherit all permissions of lower roles.
//
//   Hierarchy: Owner > Admin > HrManager | ShiftManager > TeamLead > Employee | Member


/// Granular permissions for every action in the system.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Permission {
    // ── Workspace ──────────────────────────────────────
    WorkspaceUpdate,
    WorkspaceDelete,
    WorkspaceSettingsRead,
    WorkspaceSettingsWrite,

    // ── Members ────────────────────────────────────────
    MembersRead,
    MembersInvite,
    MembersRolesManage,
    MembersRemove,

    // ── Employees ──────────────────────────────────────
    EmployeesRead,
    EmployeesCreate,
    EmployeesUpdate,
    EmployeesDelete,
    EmployeesRehire,
    EmployeesUpload,

    // ── Departments ────────────────────────────────────
    DepartmentsRead,
    DepartmentsCreate,
    DepartmentsUpdate,
    DepartmentsDelete,
    DepartmentsTransfer,

    // ── Attendance ─────────────────────────────────────
    AttendanceLogsRead,
    AttendanceLogsImport,
    AttendanceSummariesRead,
    AttendanceAdjustmentsSubmit,
    AttendanceAdjustmentsApprove,
    AttendanceReprocess,
    AttendanceStagingManage,

    // ── Overtime ───────────────────────────────────────
    OvertimeClaim,
    OvertimeApprove,
    OvertimePoliciesManage,
    OvertimePaperUpload,

    // ── Leave ──────────────────────────────────────────
    LeaveRequestsRead,
    LeaveRequestsSubmit,
    LeaveRequestsApprove,
    LeaveBalancesRead,
    LeaveBalancesAdjust,
    LeaveTypesManage,
    LeavePoliciesManage,
    LeaveAccrualRun,

    // ── Shifts ─────────────────────────────────────────
    ShiftsRead,
    ShiftsCreate,
    ShiftsUpdate,
    ShiftsAssign,
    ShiftSwapsManage,
    ShiftSwapsApprove,

    // ── Working Patterns ───────────────────────────────
    PatternsRead,
    PatternsCreate,
    PatternsAssign,
    PatternsDelete,

    // ── Rosters ────────────────────────────────────────
    RostersRead,
    RostersManage,

    // ── Assignment Rules ───────────────────────────────
    AssignmentRulesManage,

    // ── Payroll ────────────────────────────────────────
    PayrollRead,
    PayrollCalculate,
    PayrollSettingsManage,
    PayrollComponentsManage,
    PayrollIncentivesManage,

    // ── Holidays ───────────────────────────────────────
    HolidaysRead,
    HolidaysManage,

    // ── Reports ────────────────────────────────────────
    ReportsRead,
    ReportsExport,

    // ── Dashboard ──────────────────────────────────────
    DashboardRead,
    DashboardPreferences,

    // ── Agents & Devices ───────────────────────────────
    AgentsRead,
    AgentsManage,
    DevicesRead,
    DevicesManage,

    // ── Recruitment ────────────────────────────────────
    RecruitmentRead,
    RecruitmentManage,

    // ── Feature Flags ─────────────────────────────────
    FeatureFlagsRead,
    FeatureFlagsManage,

    // ── Infrastructure ─────────────────────────────────
    InfraMonitor,

    // ── ESS (Self-Service) ─────────────────────────────
    EssProfile,
    EssAttendance,
    EssClock,
    EssLeave,
    EssPayslips,
    EssShiftSwaps,
}

impl Permission {
    /// Try to parse a permission from its string representation (e.g., "employees:read").
    pub fn from_str_permission(s: &str) -> Option<Self> {
        match s {
            "workspace:update" => Some(Self::WorkspaceUpdate),
            "workspace:delete" => Some(Self::WorkspaceDelete),
            "workspace:settings:read" => Some(Self::WorkspaceSettingsRead),
            "workspace:settings:write" => Some(Self::WorkspaceSettingsWrite),
            "members:read" => Some(Self::MembersRead),
            "members:invite" => Some(Self::MembersInvite),
            "members:roles:manage" => Some(Self::MembersRolesManage),
            "members:remove" => Some(Self::MembersRemove),
            "employees:read" => Some(Self::EmployeesRead),
            "employees:create" => Some(Self::EmployeesCreate),
            "employees:update" => Some(Self::EmployeesUpdate),
            "employees:delete" => Some(Self::EmployeesDelete),
            "employees:rehire" => Some(Self::EmployeesRehire),
            "employees:upload" => Some(Self::EmployeesUpload),
            "departments:read" => Some(Self::DepartmentsRead),
            "departments:create" => Some(Self::DepartmentsCreate),
            "departments:update" => Some(Self::DepartmentsUpdate),
            "departments:delete" => Some(Self::DepartmentsDelete),
            "departments:transfer" => Some(Self::DepartmentsTransfer),
            "attendance:logs:read" => Some(Self::AttendanceLogsRead),
            "attendance:logs:import" => Some(Self::AttendanceLogsImport),
            "attendance:summaries:read" => Some(Self::AttendanceSummariesRead),
            "attendance:adjustments:submit" => Some(Self::AttendanceAdjustmentsSubmit),
            "attendance:adjustments:approve" => Some(Self::AttendanceAdjustmentsApprove),
            "attendance:reprocess" => Some(Self::AttendanceReprocess),
            "attendance:staging:manage" => Some(Self::AttendanceStagingManage),
            "overtime:claim" => Some(Self::OvertimeClaim),
            "overtime:approve" => Some(Self::OvertimeApprove),
            "overtime:policies:manage" => Some(Self::OvertimePoliciesManage),
            "overtime:paper:upload" => Some(Self::OvertimePaperUpload),
            "leave:requests:read" => Some(Self::LeaveRequestsRead),
            "leave:requests:submit" => Some(Self::LeaveRequestsSubmit),
            "leave:requests:approve" => Some(Self::LeaveRequestsApprove),
            "leave:balances:read" => Some(Self::LeaveBalancesRead),
            "leave:balances:adjust" => Some(Self::LeaveBalancesAdjust),
            "leave:types:manage" => Some(Self::LeaveTypesManage),
            "leave:policies:manage" => Some(Self::LeavePoliciesManage),
            "leave:accrual:run" => Some(Self::LeaveAccrualRun),
            "shifts:read" => Some(Self::ShiftsRead),
            "shifts:create" => Some(Self::ShiftsCreate),
            "shifts:update" => Some(Self::ShiftsUpdate),
            "shifts:assign" => Some(Self::ShiftsAssign),
            "shift_swaps:manage" => Some(Self::ShiftSwapsManage),
            "shift_swaps:approve" => Some(Self::ShiftSwapsApprove),
            "patterns:read" => Some(Self::PatternsRead),
            "patterns:create" => Some(Self::PatternsCreate),
            "patterns:assign" => Some(Self::PatternsAssign),
            "patterns:delete" => Some(Self::PatternsDelete),
            "rosters:read" => Some(Self::RostersRead),
            "rosters:manage" => Some(Self::RostersManage),
            "assignment_rules:manage" => Some(Self::AssignmentRulesManage),
            "payroll:read" => Some(Self::PayrollRead),
            "payroll:calculate" => Some(Self::PayrollCalculate),
            "payroll:settings:manage" => Some(Self::PayrollSettingsManage),
            "payroll:components:manage" => Some(Self::PayrollComponentsManage),
            "payroll:incentives:manage" => Some(Self::PayrollIncentivesManage),
            "holidays:read" => Some(Self::HolidaysRead),
            "holidays:manage" => Some(Self::HolidaysManage),
            "reports:read" => Some(Self::ReportsRead),
            "reports:export" => Some(Self::ReportsExport),
            "dashboard:read" => Some(Self::DashboardRead),
            "dashboard:preferences" => Some(Self::DashboardPreferences),
            "agents:read" => Some(Self::AgentsRead),
            "agents:manage" => Some(Self::AgentsManage),
            "devices:read" => Some(Self::DevicesRead),
            "devices:manage" => Some(Self::DevicesManage),
            "recruitment:read" => Some(Self::RecruitmentRead),
            "recruitment:manage" => Some(Self::RecruitmentManage),
            "feature_flags:read" => Some(Self::FeatureFlagsRead),
            "feature_flags:manage" => Some(Self::FeatureFlagsManage),
            "infra:monitor" => Some(Self::InfraMonitor),
            "ess:profile" => Some(Self::EssProfile),
            "ess:attendance" => Some(Self::EssAttendance),
            "ess:clock" => Some(Self::EssClock),
            "ess:leave" => Some(Self::EssLeave),
            "ess:payslips" => Some(Self::EssPayslips),
            "ess:shift_swaps" => Some(Self::EssShiftSwaps),
            _ => None,
        }
    }

    /// Convert this permission to its canonical string representation.
    pub fn to_str_permission(&self) -> &'static str {
        match self {
            Self::WorkspaceUpdate => "workspace:update",
            Self::WorkspaceDelete => "workspace:delete",
            Self::WorkspaceSettingsRead => "workspace:settings:read",
            Self::WorkspaceSettingsWrite => "workspace:settings:write",
            Self::MembersRead => "members:read",
            Self::MembersInvite => "members:invite",
            Self::MembersRolesManage => "members:roles:manage",
            Self::MembersRemove => "members:remove",
            Self::EmployeesRead => "employees:read",
            Self::EmployeesCreate => "employees:create",
            Self::EmployeesUpdate => "employees:update",
            Self::EmployeesDelete => "employees:delete",
            Self::EmployeesRehire => "employees:rehire",
            Self::EmployeesUpload => "employees:upload",
            Self::DepartmentsRead => "departments:read",
            Self::DepartmentsCreate => "departments:create",
            Self::DepartmentsUpdate => "departments:update",
            Self::DepartmentsDelete => "departments:delete",
            Self::DepartmentsTransfer => "departments:transfer",
            Self::AttendanceLogsRead => "attendance:logs:read",
            Self::AttendanceLogsImport => "attendance:logs:import",
            Self::AttendanceSummariesRead => "attendance:summaries:read",
            Self::AttendanceAdjustmentsSubmit => "attendance:adjustments:submit",
            Self::AttendanceAdjustmentsApprove => "attendance:adjustments:approve",
            Self::AttendanceReprocess => "attendance:reprocess",
            Self::AttendanceStagingManage => "attendance:staging:manage",
            Self::OvertimeClaim => "overtime:claim",
            Self::OvertimeApprove => "overtime:approve",
            Self::OvertimePoliciesManage => "overtime:policies:manage",
            Self::OvertimePaperUpload => "overtime:paper:upload",
            Self::LeaveRequestsRead => "leave:requests:read",
            Self::LeaveRequestsSubmit => "leave:requests:submit",
            Self::LeaveRequestsApprove => "leave:requests:approve",
            Self::LeaveBalancesRead => "leave:balances:read",
            Self::LeaveBalancesAdjust => "leave:balances:adjust",
            Self::LeaveTypesManage => "leave:types:manage",
            Self::LeavePoliciesManage => "leave:policies:manage",
            Self::LeaveAccrualRun => "leave:accrual:run",
            Self::ShiftsRead => "shifts:read",
            Self::ShiftsCreate => "shifts:create",
            Self::ShiftsUpdate => "shifts:update",
            Self::ShiftsAssign => "shifts:assign",
            Self::ShiftSwapsManage => "shift_swaps:manage",
            Self::ShiftSwapsApprove => "shift_swaps:approve",
            Self::PatternsRead => "patterns:read",
            Self::PatternsCreate => "patterns:create",
            Self::PatternsAssign => "patterns:assign",
            Self::PatternsDelete => "patterns:delete",
            Self::RostersRead => "rosters:read",
            Self::RostersManage => "rosters:manage",
            Self::AssignmentRulesManage => "assignment_rules:manage",
            Self::PayrollRead => "payroll:read",
            Self::PayrollCalculate => "payroll:calculate",
            Self::PayrollSettingsManage => "payroll:settings:manage",
            Self::PayrollComponentsManage => "payroll:components:manage",
            Self::PayrollIncentivesManage => "payroll:incentives:manage",
            Self::HolidaysRead => "holidays:read",
            Self::HolidaysManage => "holidays:manage",
            Self::ReportsRead => "reports:read",
            Self::ReportsExport => "reports:export",
            Self::DashboardRead => "dashboard:read",
            Self::DashboardPreferences => "dashboard:preferences",
            Self::AgentsRead => "agents:read",
            Self::AgentsManage => "agents:manage",
            Self::DevicesRead => "devices:read",
            Self::DevicesManage => "devices:manage",
            Self::RecruitmentRead => "recruitment:read",
            Self::RecruitmentManage => "recruitment:manage",
            Self::FeatureFlagsRead => "feature_flags:read",
            Self::FeatureFlagsManage => "feature_flags:manage",
            Self::InfraMonitor => "infra:monitor",
            Self::EssProfile => "ess:profile",
            Self::EssAttendance => "ess:attendance",
            Self::EssClock => "ess:clock",
            Self::EssLeave => "ess:leave",
            Self::EssPayslips => "ess:payslips",
            Self::EssShiftSwaps => "ess:shift_swaps",
        }
    }

    /// Return all available permission strings (for API listing).
    pub fn all_permission_strings() -> Vec<&'static str> {
        vec![
            "workspace:update", "workspace:delete", "workspace:settings:read", "workspace:settings:write",
            "members:read", "members:invite", "members:roles:manage", "members:remove",
            "employees:read", "employees:create", "employees:update", "employees:delete", "employees:rehire", "employees:upload",
            "departments:read", "departments:create", "departments:update", "departments:delete", "departments:transfer",
            "attendance:logs:read", "attendance:logs:import", "attendance:summaries:read",
            "attendance:adjustments:submit", "attendance:adjustments:approve", "attendance:reprocess", "attendance:staging:manage",
            "overtime:claim", "overtime:approve", "overtime:policies:manage", "overtime:paper:upload",
            "leave:requests:read", "leave:requests:submit", "leave:requests:approve", "leave:balances:read", "leave:balances:adjust",
            "leave:types:manage", "leave:policies:manage", "leave:accrual:run",
            "shifts:read", "shifts:create", "shifts:update", "shifts:assign",
            "shift_swaps:manage", "shift_swaps:approve",
            "patterns:read", "patterns:create", "patterns:assign", "patterns:delete",
            "rosters:read", "rosters:manage", "assignment_rules:manage",
            "payroll:read", "payroll:calculate", "payroll:settings:manage", "payroll:components:manage", "payroll:incentives:manage",
            "holidays:read", "holidays:manage",
            "reports:read", "reports:export",
            "dashboard:read", "dashboard:preferences",
            "agents:read", "agents:manage", "devices:read", "devices:manage",
            "recruitment:read", "recruitment:manage",
            "infra:monitor",
            "ess:profile", "ess:attendance", "ess:clock", "ess:leave", "ess:payslips", "ess:shift_swaps",
        ]
    }
}


