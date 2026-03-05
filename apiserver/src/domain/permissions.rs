// Role-based permission system for LeeAttend
//
//   MemberRole → HashSet<Permission>
//   Higher roles inherit all permissions of lower roles.
//
//   Hierarchy: Owner > Admin > HrManager | ShiftManager > TeamLead > Employee | Member

use std::collections::HashSet;

use crate::models::enums::MemberRole;

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
}

/// Return all permissions granted to a single role (without inheritance).
fn base_permissions(role: MemberRole) -> HashSet<Permission> {
    use Permission::*;

    match role {
        // ── Employee / Member ──────────────────────────
        // ESS self-service only
        MemberRole::Employee | MemberRole::Member => HashSet::from([
            EssProfile,
            EssAttendance,
            EssClock,
            EssLeave,
            EssPayslips,
            EssShiftSwaps,
            HolidaysRead,
            AttendanceAdjustmentsSubmit,
            OvertimeClaim,
        ]),

        // ── Team Lead ──────────────────────────────────
        // Approve actions for their scope + view data
        MemberRole::TeamLead => HashSet::from([
            EmployeesRead,
            DepartmentsRead,
            AttendanceLogsRead,
            AttendanceSummariesRead,
            AttendanceAdjustmentsApprove,
            OvertimeApprove,
            LeaveRequestsRead,
            LeaveRequestsApprove,
            LeaveBalancesRead,
            ShiftsRead,
            ShiftSwapsManage,
            ShiftSwapsApprove,
            PatternsRead,
            RostersRead,
            ReportsRead,
            DashboardRead,
            DashboardPreferences,
        ]),

        // ── Shift Manager ──────────────────────────────
        // Full shift/roster/pattern management
        MemberRole::ShiftManager => HashSet::from([
            ShiftsCreate,
            ShiftsUpdate,
            ShiftsAssign,
            PatternsCreate,
            PatternsAssign,
            PatternsDelete,
            RostersManage,
            AssignmentRulesManage,
            DevicesRead,
            WorkspaceSettingsRead,
        ]),

        // ── HR Manager ─────────────────────────────────
        // Employee lifecycle, leaves, attendance admin, payroll, reports
        MemberRole::HrManager => HashSet::from([
            EmployeesCreate,
            EmployeesUpdate,
            EmployeesDelete,
            EmployeesRehire,
            EmployeesUpload,
            DepartmentsCreate,
            DepartmentsUpdate,
            DepartmentsDelete,
            DepartmentsTransfer,
            AttendanceLogsImport,
            AttendanceReprocess,
            AttendanceStagingManage,
            OvertimePoliciesManage,
            OvertimePaperUpload,
            LeaveBalancesAdjust,
            LeaveTypesManage,
            LeavePoliciesManage,
            LeaveAccrualRun,
            PayrollRead,
            PayrollCalculate,
            PayrollSettingsManage,
            PayrollComponentsManage,
            PayrollIncentivesManage,
            HolidaysManage,
            ReportsExport,
            RecruitmentRead,
            RecruitmentManage,
            MembersRead,
            WorkspaceSettingsRead,
        ]),

        // ── Admin ──────────────────────────────────────
        // Everything except workspace deletion + ownership transfer
        MemberRole::Admin => HashSet::from([
            WorkspaceUpdate,
            WorkspaceSettingsWrite,
            MembersInvite,
            MembersRolesManage,
            MembersRemove,
            AgentsRead,
            AgentsManage,
            DevicesManage,
            InfraMonitor,
        ]),

        // ── Owner ──────────────────────────────────────
        // Workspace-level destructive actions
        MemberRole::Owner => HashSet::from([WorkspaceDelete]),
    }
}

/// Return the role hierarchy chain (the role itself + all roles it inherits from).
///
/// Hierarchy:
///   Owner   → inherits Admin
///   Admin   → inherits HrManager + ShiftManager
///   HrManager   → inherits TeamLead
///   ShiftManager → inherits TeamLead
///   TeamLead → inherits Employee
///   Employee / Member → base
fn role_hierarchy(role: MemberRole) -> Vec<MemberRole> {
    match role {
        MemberRole::Owner => vec![
            MemberRole::Owner,
            MemberRole::Admin,
            MemberRole::HrManager,
            MemberRole::ShiftManager,
            MemberRole::TeamLead,
            MemberRole::Employee,
        ],
        MemberRole::Admin => vec![
            MemberRole::Admin,
            MemberRole::HrManager,
            MemberRole::ShiftManager,
            MemberRole::TeamLead,
            MemberRole::Employee,
        ],
        MemberRole::HrManager => vec![
            MemberRole::HrManager,
            MemberRole::TeamLead,
            MemberRole::Employee,
        ],
        MemberRole::ShiftManager => vec![
            MemberRole::ShiftManager,
            MemberRole::TeamLead,
            MemberRole::Employee,
        ],
        MemberRole::TeamLead => vec![MemberRole::TeamLead, MemberRole::Employee],
        MemberRole::Employee | MemberRole::Member => vec![MemberRole::Employee],
    }
}

/// Compute ALL permissions for a given role (including inherited ones).
pub fn role_permissions(role: MemberRole) -> HashSet<Permission> {
    let mut perms = HashSet::new();
    for r in role_hierarchy(role) {
        perms.extend(base_permissions(r));
    }
    perms
}

/// Check whether a set of roles (primary + extra) grants a specific permission.
pub fn has_permission(roles: &[MemberRole], perm: Permission) -> bool {
    roles
        .iter()
        .any(|role| role_permissions(*role).contains(&perm))
}

/// Compute the full effective permission set for a list of roles.
pub fn effective_permissions(roles: &[MemberRole]) -> HashSet<Permission> {
    let mut perms = HashSet::new();
    for role in roles {
        perms.extend(role_permissions(*role));
    }
    perms
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn owner_has_all_permissions() {
        let perms = role_permissions(MemberRole::Owner);
        assert!(perms.contains(&Permission::WorkspaceDelete));
        assert!(perms.contains(&Permission::WorkspaceUpdate));
        assert!(perms.contains(&Permission::EmployeesCreate));
        assert!(perms.contains(&Permission::ShiftsCreate));
        assert!(perms.contains(&Permission::PayrollCalculate));
        assert!(perms.contains(&Permission::EssProfile));
        assert!(perms.contains(&Permission::InfraMonitor));
    }

    #[test]
    fn admin_cannot_delete_workspace() {
        let perms = role_permissions(MemberRole::Admin);
        assert!(!perms.contains(&Permission::WorkspaceDelete));
        assert!(perms.contains(&Permission::WorkspaceUpdate));
        assert!(perms.contains(&Permission::EmployeesCreate));
        assert!(perms.contains(&Permission::AgentsManage));
    }

    #[test]
    fn hr_manager_has_employee_management() {
        let perms = role_permissions(MemberRole::HrManager);
        assert!(perms.contains(&Permission::EmployeesCreate));
        assert!(perms.contains(&Permission::PayrollCalculate));
        assert!(perms.contains(&Permission::LeaveTypesManage));
        // Should NOT have admin-only perms
        assert!(!perms.contains(&Permission::AgentsManage));
        assert!(!perms.contains(&Permission::WorkspaceUpdate));
        assert!(!perms.contains(&Permission::MembersInvite));
    }

    #[test]
    fn shift_manager_has_shift_management() {
        let perms = role_permissions(MemberRole::ShiftManager);
        assert!(perms.contains(&Permission::ShiftsCreate));
        assert!(perms.contains(&Permission::PatternsCreate));
        assert!(perms.contains(&Permission::RostersManage));
        // Should NOT have HR-only perms
        assert!(!perms.contains(&Permission::EmployeesCreate));
        assert!(!perms.contains(&Permission::PayrollCalculate));
    }

    #[test]
    fn team_lead_has_approval_permissions() {
        let perms = role_permissions(MemberRole::TeamLead);
        assert!(perms.contains(&Permission::OvertimeApprove));
        assert!(perms.contains(&Permission::LeaveRequestsApprove));
        assert!(perms.contains(&Permission::AttendanceAdjustmentsApprove));
        assert!(perms.contains(&Permission::DashboardRead));
        // Should NOT have creation perms
        assert!(!perms.contains(&Permission::EmployeesCreate));
        assert!(!perms.contains(&Permission::ShiftsCreate));
    }

    #[test]
    fn employee_has_ess_only() {
        let perms = role_permissions(MemberRole::Employee);
        assert!(perms.contains(&Permission::EssProfile));
        assert!(perms.contains(&Permission::EssLeave));
        assert!(perms.contains(&Permission::OvertimeClaim));
        // Should NOT have any management perms
        assert!(!perms.contains(&Permission::EmployeesRead));
        assert!(!perms.contains(&Permission::DashboardRead));
        assert!(!perms.contains(&Permission::ShiftsRead));
    }

    #[test]
    fn member_same_as_employee() {
        let member_perms = role_permissions(MemberRole::Member);
        let employee_perms = role_permissions(MemberRole::Employee);
        assert_eq!(member_perms, employee_perms);
    }

    #[test]
    fn has_permission_works_with_multiple_roles() {
        let roles = vec![MemberRole::Employee, MemberRole::TeamLead];
        assert!(has_permission(&roles, Permission::OvertimeApprove));
        assert!(has_permission(&roles, Permission::EssProfile));
        assert!(!has_permission(&roles, Permission::EmployeesCreate));
    }

    #[test]
    fn from_str_permission_roundtrip() {
        assert_eq!(
            Permission::from_str_permission("employees:read"),
            Some(Permission::EmployeesRead)
        );
        assert_eq!(
            Permission::from_str_permission("attendance:logs:import"),
            Some(Permission::AttendanceLogsImport)
        );
        assert_eq!(Permission::from_str_permission("invalid"), None);
    }
}
