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

    // ── Security & Audit ──────────────────────────────
    AuditLogsRead,
    AuditLogsExport,
    SecuritySettingsManage,

    // ── Bulk & Emergency ──────────────────────────────
    EmployeesBulkEdit,
    AttendanceForceClockOut,

    // ── Documents ─────────────────────────────────────
    DocumentsMedicalRead,
    DocumentsContractRead,

    // ── Advanced Payroll & Approvals ──────────────────
    PayrollApprove,
    PayrollDisburse,
    PayrollSensitiveRead,
    LeaveApproveL1,
    LeaveApproveL2,

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
            "audit_logs:read" => Some(Self::AuditLogsRead),
            "audit_logs:export" => Some(Self::AuditLogsExport),
            "security:settings:manage" => Some(Self::SecuritySettingsManage),
            "employees:bulk_edit" => Some(Self::EmployeesBulkEdit),
            "attendance:force_clock_out" => Some(Self::AttendanceForceClockOut),
            "documents:medical:read" => Some(Self::DocumentsMedicalRead),
            "documents:contract:read" => Some(Self::DocumentsContractRead),
            "payroll:approve" => Some(Self::PayrollApprove),
            "payroll:disburse" => Some(Self::PayrollDisburse),
            "payroll:sensitive:read" => Some(Self::PayrollSensitiveRead),
            "leave:approve:l1" => Some(Self::LeaveApproveL1),
            "leave:approve:l2" => Some(Self::LeaveApproveL2),
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
            Self::AuditLogsRead => "audit_logs:read",
            Self::AuditLogsExport => "audit_logs:export",
            Self::SecuritySettingsManage => "security:settings:manage",
            Self::EmployeesBulkEdit => "employees:bulk_edit",
            Self::AttendanceForceClockOut => "attendance:force_clock_out",
            Self::DocumentsMedicalRead => "documents:medical:read",
            Self::DocumentsContractRead => "documents:contract:read",
            Self::PayrollApprove => "payroll:approve",
            Self::PayrollDisburse => "payroll:disburse",
            Self::PayrollSensitiveRead => "payroll:sensitive:read",
            Self::LeaveApproveL1 => "leave:approve:l1",
            Self::LeaveApproveL2 => "leave:approve:l2",
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
            "workspace:update",
            "workspace:delete",
            "workspace:settings:read",
            "workspace:settings:write",
            "members:read",
            "members:invite",
            "members:roles:manage",
            "members:remove",
            "employees:read",
            "employees:create",
            "employees:update",
            "employees:delete",
            "employees:rehire",
            "employees:upload",
            "departments:read",
            "departments:create",
            "departments:update",
            "departments:delete",
            "departments:transfer",
            "attendance:logs:read",
            "attendance:logs:import",
            "attendance:summaries:read",
            "attendance:adjustments:submit",
            "attendance:adjustments:approve",
            "attendance:reprocess",
            "attendance:staging:manage",
            "overtime:claim",
            "overtime:approve",
            "overtime:policies:manage",
            "overtime:paper:upload",
            "leave:requests:read",
            "leave:requests:submit",
            "leave:requests:approve",
            "leave:balances:read",
            "leave:balances:adjust",
            "leave:types:manage",
            "leave:policies:manage",
            "leave:accrual:run",
            "shifts:read",
            "shifts:create",
            "shifts:update",
            "shifts:assign",
            "shift_swaps:manage",
            "shift_swaps:approve",
            "patterns:read",
            "patterns:create",
            "patterns:assign",
            "patterns:delete",
            "rosters:read",
            "rosters:manage",
            "assignment_rules:manage",
            "payroll:read",
            "payroll:calculate",
            "payroll:settings:manage",
            "payroll:components:manage",
            "payroll:incentives:manage",
            "holidays:read",
            "holidays:manage",
            "reports:read",
            "reports:export",
            "dashboard:read",
            "dashboard:preferences",
            "agents:read",
            "agents:manage",
            "devices:read",
            "devices:manage",
            "recruitment:read",
            "recruitment:manage",
            "feature_flags:read",
            "feature_flags:manage",
            "audit_logs:read",
            "audit_logs:export",
            "security:settings:manage",
            "employees:bulk_edit",
            "attendance:force_clock_out",
            "documents:medical:read",
            "documents:contract:read",
            "payroll:approve",
            "payroll:disburse",
            "payroll:sensitive:read",
            "leave:approve:l1",
            "leave:approve:l2",
            "infra:monitor",
            "ess:profile",
            "ess:attendance",
            "ess:clock",
            "ess:leave",
            "ess:payslips",
            "ess:shift_swaps",
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    /// Lock the total permission count at 89.
    /// If you add a new Permission variant, update this number,
    /// `all_permission_strings()`, AND the frontend PermissionSync.test.ts.
    #[test]
    fn test_total_permission_count_is_89() {
        let all = Permission::all_permission_strings();
        assert_eq!(
            all.len(),
            89,
            "Permission count changed! Update this test, the UI permissionGroups, AND PermissionSync.test.ts. Current count: {}",
            all.len()
        );
    }

    /// Every string in all_permission_strings() must roundtrip through from_str → to_str.
    #[test]
    fn test_all_permission_strings_roundtrip() {
        for s in Permission::all_permission_strings() {
            let parsed = Permission::from_str_permission(s);
            assert!(
                parsed.is_some(),
                "from_str_permission failed for '{s}' — missing match arm in from_str_permission()"
            );
            let back = parsed.unwrap().to_str_permission();
            assert_eq!(
                s, back,
                "Roundtrip mismatch: from_str('{s}') → to_str = '{back}'"
            );
        }
    }

    /// No duplicate strings in all_permission_strings().
    #[test]
    fn test_no_duplicate_permission_strings() {
        let all = Permission::all_permission_strings();
        let unique: HashSet<&str> = all.iter().copied().collect();
        assert_eq!(
            all.len(),
            unique.len(),
            "Duplicate permission strings detected! Count: {} vs unique: {}",
            all.len(),
            unique.len()
        );
    }

    /// Every Permission enum variant is covered by all_permission_strings().
    /// We verify this indirectly: to_str_permission for each variant must appear in the vec.
    #[test]
    fn test_all_enum_variants_covered() {
        let all_set: HashSet<&str> = Permission::all_permission_strings().into_iter().collect();

        // Exhaustive list of all enum variants
        let all_variants = vec![
            Permission::WorkspaceUpdate,
            Permission::WorkspaceDelete,
            Permission::WorkspaceSettingsRead,
            Permission::WorkspaceSettingsWrite,
            Permission::MembersRead,
            Permission::MembersInvite,
            Permission::MembersRolesManage,
            Permission::MembersRemove,
            Permission::EmployeesRead,
            Permission::EmployeesCreate,
            Permission::EmployeesUpdate,
            Permission::EmployeesDelete,
            Permission::EmployeesRehire,
            Permission::EmployeesUpload,
            Permission::DepartmentsRead,
            Permission::DepartmentsCreate,
            Permission::DepartmentsUpdate,
            Permission::DepartmentsDelete,
            Permission::DepartmentsTransfer,
            Permission::AttendanceLogsRead,
            Permission::AttendanceLogsImport,
            Permission::AttendanceSummariesRead,
            Permission::AttendanceAdjustmentsSubmit,
            Permission::AttendanceAdjustmentsApprove,
            Permission::AttendanceReprocess,
            Permission::AttendanceStagingManage,
            Permission::OvertimeClaim,
            Permission::OvertimeApprove,
            Permission::OvertimePoliciesManage,
            Permission::OvertimePaperUpload,
            Permission::LeaveRequestsRead,
            Permission::LeaveRequestsSubmit,
            Permission::LeaveRequestsApprove,
            Permission::LeaveBalancesRead,
            Permission::LeaveBalancesAdjust,
            Permission::LeaveTypesManage,
            Permission::LeavePoliciesManage,
            Permission::LeaveAccrualRun,
            Permission::ShiftsRead,
            Permission::ShiftsCreate,
            Permission::ShiftsUpdate,
            Permission::ShiftsAssign,
            Permission::ShiftSwapsManage,
            Permission::ShiftSwapsApprove,
            Permission::PatternsRead,
            Permission::PatternsCreate,
            Permission::PatternsAssign,
            Permission::PatternsDelete,
            Permission::RostersRead,
            Permission::RostersManage,
            Permission::AssignmentRulesManage,
            Permission::PayrollRead,
            Permission::PayrollCalculate,
            Permission::PayrollSettingsManage,
            Permission::PayrollComponentsManage,
            Permission::PayrollIncentivesManage,
            Permission::HolidaysRead,
            Permission::HolidaysManage,
            Permission::ReportsRead,
            Permission::ReportsExport,
            Permission::DashboardRead,
            Permission::DashboardPreferences,
            Permission::AgentsRead,
            Permission::AgentsManage,
            Permission::DevicesRead,
            Permission::DevicesManage,
            Permission::RecruitmentRead,
            Permission::RecruitmentManage,
            Permission::FeatureFlagsRead,
            Permission::FeatureFlagsManage,
            Permission::AuditLogsRead,
            Permission::AuditLogsExport,
            Permission::SecuritySettingsManage,
            Permission::EmployeesBulkEdit,
            Permission::AttendanceForceClockOut,
            Permission::DocumentsMedicalRead,
            Permission::DocumentsContractRead,
            Permission::PayrollApprove,
            Permission::PayrollDisburse,
            Permission::PayrollSensitiveRead,
            Permission::LeaveApproveL1,
            Permission::LeaveApproveL2,
            Permission::InfraMonitor,
            Permission::EssProfile,
            Permission::EssAttendance,
            Permission::EssClock,
            Permission::EssLeave,
            Permission::EssPayslips,
            Permission::EssShiftSwaps,
        ];

        assert_eq!(
            all_variants.len(),
            89,
            "Variant list in test is stale — expected 89, got {}",
            all_variants.len()
        );

        for variant in &all_variants {
            let s = variant.to_str_permission();
            assert!(
                all_set.contains(s),
                "Enum variant {:?} ('{s}') is missing from all_permission_strings()",
                variant
            );
        }
    }

    /// All permission strings follow colon-separated lowercase format.
    #[test]
    fn test_permission_string_format() {
        for s in Permission::all_permission_strings() {
            assert!(!s.is_empty(), "Empty permission string found");
            assert!(
                s.chars().all(|c| c.is_ascii_lowercase() || c == ':' || c == '_' || c.is_ascii_digit()),
                "Permission string '{s}' contains invalid characters (expected lowercase, digits, colons, underscores)"
            );
            assert!(
                !s.starts_with(':') && !s.ends_with(':'),
                "Permission string '{s}' must not start or end with ':'"
            );
            assert!(
                !s.contains("::"),
                "Permission string '{s}' must not contain consecutive colons"
            );
        }
    }
}
