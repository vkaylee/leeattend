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
            LeaveRequestsSubmit,
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
            FeatureFlagsRead,
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
            FeatureFlagsManage,
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

    // =====================================================================
    // Additional RBAC tests
    // =====================================================================

    #[test]
    fn all_permission_strings_are_parseable() {
        // Every string returned by all_permission_strings() must roundtrip
        for perm_str in Permission::all_permission_strings() {
            assert!(
                Permission::from_str_permission(perm_str).is_some(),
                "Permission string '{}' could not be parsed",
                perm_str
            );
        }
    }

    #[test]
    fn all_permissions_have_str_roundtrip() {
        // Every variant must roundtrip: to_str_permission → from_str_permission
        for perm_str in Permission::all_permission_strings() {
            let perm = Permission::from_str_permission(perm_str).unwrap();
            let back = perm.to_str_permission();
            assert_eq!(back, perm_str, "Roundtrip failed for {perm_str}");
        }
    }

    #[test]
    fn higher_roles_have_more_permissions() {
        let employee_count = role_permissions(MemberRole::Employee).len();
        let team_lead_count = role_permissions(MemberRole::TeamLead).len();
        let hr_count = role_permissions(MemberRole::HrManager).len();
        let admin_count = role_permissions(MemberRole::Admin).len();
        let owner_count = role_permissions(MemberRole::Owner).len();

        assert!(team_lead_count > employee_count, "TeamLead should have more perms than Employee");
        assert!(hr_count > team_lead_count, "HrManager should have more perms than TeamLead");
        assert!(admin_count > hr_count, "Admin should have more perms than HrManager");
        assert!(owner_count > admin_count, "Owner should have more perms than Admin");
    }

    #[test]
    fn owner_is_superset_of_all_roles() {
        let owner_perms = role_permissions(MemberRole::Owner);
        for role in [
            MemberRole::Admin, MemberRole::HrManager,
            MemberRole::ShiftManager, MemberRole::TeamLead,
            MemberRole::Employee, MemberRole::Member,
        ] {
            let role_perms = role_permissions(role);
            assert!(
                role_perms.is_subset(&owner_perms),
                "{:?} permissions should be subset of Owner",
                role
            );
        }
    }

    #[test]
    fn effective_permissions_combines_multiple_roles() {
        let roles = vec![MemberRole::HrManager, MemberRole::ShiftManager];
        let combined = effective_permissions(&roles);
        let hr_perms = role_permissions(MemberRole::HrManager);
        let shift_perms = role_permissions(MemberRole::ShiftManager);

        assert!(hr_perms.is_subset(&combined));
        assert!(shift_perms.is_subset(&combined));
    }

    #[test]
    fn has_permission_with_empty_roles() {
        assert!(!has_permission(&[], Permission::EssProfile));
        assert!(!has_permission(&[], Permission::WorkspaceDelete));
    }

    #[test]
    fn shift_manager_inherits_team_lead() {
        let sm_perms = role_permissions(MemberRole::ShiftManager);
        // ShiftManager → TeamLead → Employee
        assert!(sm_perms.contains(&Permission::OvertimeApprove)); // from TeamLead
        assert!(sm_perms.contains(&Permission::EssProfile)); // from Employee
        assert!(sm_perms.contains(&Permission::ShiftsCreate)); // own
    }

    #[test]
    fn hr_manager_inherits_team_lead() {
        let hr_perms = role_permissions(MemberRole::HrManager);
        assert!(hr_perms.contains(&Permission::LeaveRequestsApprove)); // from TeamLead
        assert!(hr_perms.contains(&Permission::EssProfile)); // from Employee
        assert!(hr_perms.contains(&Permission::PayrollCalculate)); // own
    }

    #[test]
    fn admin_inherits_both_managers() {
        let admin_perms = role_permissions(MemberRole::Admin);
        // From HrManager
        assert!(admin_perms.contains(&Permission::PayrollCalculate));
        assert!(admin_perms.contains(&Permission::EmployeesCreate));
        // From ShiftManager
        assert!(admin_perms.contains(&Permission::ShiftsCreate));
        assert!(admin_perms.contains(&Permission::RostersManage));
        // Own
        assert!(admin_perms.contains(&Permission::AgentsManage));
        assert!(admin_perms.contains(&Permission::InfraMonitor));
    }

    #[test]
    fn employee_cannot_approve_or_manage() {
        let perms = role_permissions(MemberRole::Employee);
        assert!(!perms.contains(&Permission::AttendanceAdjustmentsApprove));
        assert!(!perms.contains(&Permission::LeaveRequestsApprove));
        assert!(!perms.contains(&Permission::PayrollCalculate));
        assert!(!perms.contains(&Permission::AgentsManage));
        assert!(!perms.contains(&Permission::WorkspaceDelete));
    }

    #[test]
    fn from_str_permission_unknown_returns_none() {
        assert_eq!(Permission::from_str_permission(""), None);
        assert_eq!(Permission::from_str_permission("workspace:fly"), None);
        assert_eq!(Permission::from_str_permission("unknown:permission:string"), None);
    }
}

