package auth

import (
	"github.com/google/uuid"
	"github.com/vkaylee/leeattend/apiserver/internal/apperror"
	"github.com/vkaylee/leeattend/apiserver/internal/domain/ids"
)

// Permission represents a granular action permission in the system.
// Canonical permission variants remain after edge-agent controls were retired.
type Permission string

const (
	// Organization settings
	PermissionOrganizationSettingsRead  Permission = "organization:settings:read"
	PermissionOrganizationSettingsWrite Permission = "organization:settings:write"

	// Members and RBAC administration
	PermissionMembersRead              Permission = "members:read"
	PermissionMembersInvite            Permission = "members:invite"
	PermissionMembersRolesManage       Permission = "members:roles:manage" // Deprecated: use roles:manage or members:roles:assign.
	PermissionMembersRolesAssign       Permission = "members:roles:assign"
	PermissionMembersPermissionsManage Permission = "members:permissions:manage"
	PermissionMembersRemove            Permission = "members:remove"
	PermissionMembersEmployeeLink      Permission = "members:employee:link"
	PermissionRolesManage              Permission = "roles:manage"

	// Employees
	PermissionEmployeesRead   Permission = "employees:read"
	PermissionEmployeesCreate Permission = "employees:create"
	PermissionEmployeesUpdate Permission = "employees:update"
	PermissionEmployeesDelete Permission = "employees:delete"
	PermissionEmployeesRehire Permission = "employees:rehire"
	PermissionEmployeesUpload Permission = "employees:upload"

	// Departments
	PermissionDepartmentsRead     Permission = "departments:read"
	PermissionDepartmentsCreate   Permission = "departments:create"
	PermissionDepartmentsUpdate   Permission = "departments:update"
	PermissionDepartmentsDelete   Permission = "departments:delete"
	PermissionDepartmentsTransfer Permission = "departments:transfer"

	// Attendance
	PermissionAttendanceLogsRead                Permission = "attendance:logs:read"
	PermissionAttendanceLogsImport              Permission = "attendance:logs:import"
	PermissionAttendanceSummariesRead           Permission = "attendance:summaries:read"
	PermissionAttendanceAdjustmentsSubmit       Permission = "attendance:adjustments:submit"
	PermissionAttendanceAdjustmentsApprove      Permission = "attendance:adjustments:approve"
	PermissionAttendanceAdjustmentsSubmitBehalf Permission = "attendance:adjustments:submit_on_behalf"
	PermissionAttendanceAdjustmentsManageBehalf Permission = "attendance:adjustments:manage_behalf"
	PermissionAttendanceReprocess               Permission = "attendance:reprocess"
	PermissionAttendanceStagingManage           Permission = "attendance:staging:manage"
	PermissionAttendanceClosureRead             Permission = "attendance:closure:read"
	PermissionAttendanceClosureClose            Permission = "attendance:closure:close"
	PermissionAttendanceClosureReopen           Permission = "attendance:closure:reopen"

	// Overtime
	PermissionOvertimeClaim               Permission = "overtime:claim"
	PermissionOvertimeApprove             Permission = "overtime:approve"
	PermissionOvertimeClaimOnBehalf       Permission = "overtime:claim_on_behalf"
	PermissionOvertimePoliciesManage      Permission = "overtime:policies:manage"
	PermissionOvertimePaperUpload         Permission = "overtime:paper:upload"
	PermissionOvertimeRequestCreate       Permission = "overtime:request:create"
	PermissionOvertimeRequestCreateBehalf Permission = "overtime:request:create_on_behalf"
	PermissionOvertimeRequestRead         Permission = "overtime:request:read"
	PermissionOvertimeApproveWorkflow     Permission = "overtime:approve:workflow"
	PermissionOvertimeQuotaRead           Permission = "overtime:quota:read"

	// Approval Workflows
	PermissionApprovalWorkflowsRead   Permission = "approval_workflows:read"
	PermissionApprovalWorkflowsManage Permission = "approval_workflows:manage"

	// Leave
	PermissionLeaveRequestsRead           Permission = "leave:requests:read"
	PermissionLeaveRequestsSubmit         Permission = "leave:requests:submit"
	PermissionLeaveRequestsSubmitOnBehalf Permission = "leave:requests:submit_on_behalf"
	PermissionLeaveRequestsApprove        Permission = "leave:requests:approve"
	PermissionLeaveBalancesRead           Permission = "leave:balances:read"
	PermissionLeaveBalancesAdjust         Permission = "leave:balances:adjust"
	PermissionLeaveTypesManage            Permission = "leave:types:manage"
	PermissionLeavePoliciesManage         Permission = "leave:policies:manage"
	PermissionLeaveAccrualRun             Permission = "leave:accrual:run"

	// Shifts
	PermissionShiftsRead        Permission = "shifts:read"
	PermissionShiftsCreate      Permission = "shifts:create"
	PermissionShiftsUpdate      Permission = "shifts:update"
	PermissionShiftsAssign      Permission = "shifts:assign"
	PermissionShiftSwapsManage  Permission = "shift_swaps:manage"
	PermissionShiftSwapsApprove Permission = "shift_swaps:approve"

	// Working Patterns
	PermissionPatternsRead   Permission = "patterns:read"
	PermissionPatternsCreate Permission = "patterns:create"
	PermissionPatternsAssign Permission = "patterns:assign"
	PermissionPatternsDelete Permission = "patterns:delete"

	// Rosters
	PermissionRostersRead   Permission = "rosters:read"
	PermissionRostersManage Permission = "rosters:manage"

	// Schedule lifecycle
	PermissionSchedulesRead            Permission = "schedules:read"
	PermissionSchedulesPreview         Permission = "schedules:preview"
	PermissionSchedulesPublishOffice   Permission = "schedules:publish:office"
	PermissionSchedulesPublishRotating Permission = "schedules:publish:rotating"
	PermissionSchedulesAmend           Permission = "schedules:amend"
	PermissionSchedulesEvidenceExport  Permission = "schedules:evidence:export"
	PermissionSchedulesHistoryRead     Permission = "schedules:history:read"
	PermissionSchedulesActivate        Permission = "schedules:activate"

	// Assignment Rules
	PermissionAssignmentRulesManage Permission = "assignment_rules:manage"

	// Payroll
	PermissionPayrollRead             Permission = "payroll:read"
	PermissionPayrollCalculate        Permission = "payroll:calculate"
	PermissionPayrollSettingsManage   Permission = "payroll:settings:manage"
	PermissionPayrollComponentsManage Permission = "payroll:components:manage"
	PermissionPayrollIncentivesManage Permission = "payroll:incentives:manage"
	PermissionPayrollInputsManage     Permission = "payroll:inputs:manage"

	// Holidays
	PermissionHolidaysRead   Permission = "holidays:read"
	PermissionHolidaysManage Permission = "holidays:manage"

	// Reports
	PermissionReportsRead   Permission = "reports:read"
	PermissionReportsExport Permission = "reports:export"

	// Dashboard
	PermissionDashboardRead                Permission = "dashboard:read"
	PermissionDashboardPreferences         Permission = "dashboard:preferences"
	PermissionDashboardEmployeeAbsenceRead Permission = "dashboard:employee_absence:read"

	// Employment Phases
	PermissionEmploymentPhasesRead   Permission = "employment_phases:read"
	PermissionEmploymentPhasesManage Permission = "employment_phases:manage"

	// Attendance Policy Assignments
	PermissionAttendancePolicyAssignmentsRead   Permission = "attendance_policy_assignments:read"
	PermissionAttendancePolicyAssignmentsManage Permission = "attendance_policy_assignments:manage"

	// Recruitment
	PermissionRecruitmentRead   Permission = "recruitment:read"
	PermissionRecruitmentManage Permission = "recruitment:manage"

	// Feature Flags
	PermissionFeatureFlagsRead   Permission = "feature_flags:read"
	PermissionFeatureFlagsManage Permission = "feature_flags:manage"

	// Security & Audit
	PermissionAuditLogsRead          Permission = "audit_logs:read"
	PermissionAuditLogsExport        Permission = "audit_logs:export"
	PermissionSecuritySettingsManage Permission = "security:settings:manage"

	// Bulk & Emergency
	PermissionEmployeesBulkEdit       Permission = "employees:bulk_edit"
	PermissionAttendanceForceClockOut Permission = "attendance:force_clock_out"

	// Documents
	PermissionDocumentsMedicalRead  Permission = "documents:medical:read"
	PermissionDocumentsContractRead Permission = "documents:contract:read"
	PermissionDocumentsManage       Permission = "documents:manage"

	// Files
	PermissionFilesRead   Permission = "files:read"
	PermissionFilesManage Permission = "files:manage"

	// Advanced Payroll & Approvals
	PermissionPayrollApprove       Permission = "payroll:approve"
	PermissionPayrollDisburse      Permission = "payroll:disburse"
	PermissionPayrollSensitiveRead Permission = "payroll:sensitive:read"
	PermissionLeaveApproveL1       Permission = "leave:approve:l1"
	PermissionLeaveApproveL2       Permission = "leave:approve:l2"

	// Infrastructure
	PermissionInfraMonitor Permission = "infra:monitor"

	// ESS (Self-Service)
	PermissionEssProfile    Permission = "ess:profile"
	PermissionEssAttendance Permission = "ess:attendance"
	PermissionEssClock      Permission = "ess:clock"
	PermissionEssLeave      Permission = "ess:leave"
	PermissionEssPayslips   Permission = "ess:payslips"
	PermissionEssShiftSwaps Permission = "ess:shift_swaps"
)

var allCanonicalPermissions = []Permission{
	PermissionOrganizationSettingsRead,
	PermissionOrganizationSettingsWrite,
	PermissionMembersRead,
	PermissionMembersInvite,
	PermissionMembersRolesManage,
	PermissionMembersRolesAssign,
	PermissionMembersPermissionsManage,
	PermissionMembersRemove,
	PermissionMembersEmployeeLink,
	PermissionRolesManage,
	PermissionEmployeesRead,
	PermissionEmployeesCreate,
	PermissionEmployeesUpdate,
	PermissionEmployeesDelete,
	PermissionEmployeesRehire,
	PermissionEmployeesUpload,
	PermissionDepartmentsRead,
	PermissionDepartmentsCreate,
	PermissionDepartmentsUpdate,
	PermissionDepartmentsDelete,
	PermissionDepartmentsTransfer,
	PermissionAttendanceLogsRead,
	PermissionAttendanceLogsImport,
	PermissionAttendanceSummariesRead,
	PermissionAttendanceAdjustmentsSubmit,
	PermissionAttendanceAdjustmentsApprove,
	PermissionAttendanceAdjustmentsSubmitBehalf,
	PermissionAttendanceAdjustmentsManageBehalf,
	PermissionAttendanceReprocess,
	PermissionAttendanceStagingManage,
	PermissionAttendanceClosureRead,
	PermissionAttendanceClosureClose,
	PermissionAttendanceClosureReopen,
	PermissionOvertimeClaim,
	PermissionOvertimeApprove,
	PermissionOvertimeClaimOnBehalf,
	PermissionOvertimePoliciesManage,
	PermissionOvertimePaperUpload,
	PermissionOvertimeRequestCreate,
	PermissionOvertimeRequestCreateBehalf,
	PermissionOvertimeRequestRead,
	PermissionOvertimeApproveWorkflow,
	PermissionOvertimeQuotaRead,
	PermissionApprovalWorkflowsRead,
	PermissionApprovalWorkflowsManage,
	PermissionLeaveRequestsRead,
	PermissionLeaveRequestsSubmit,
	PermissionLeaveRequestsSubmitOnBehalf,
	PermissionLeaveRequestsApprove,
	PermissionLeaveBalancesRead,
	PermissionLeaveBalancesAdjust,
	PermissionLeaveTypesManage,
	PermissionLeavePoliciesManage,
	PermissionLeaveAccrualRun,
	PermissionShiftsRead,
	PermissionShiftsCreate,
	PermissionShiftsUpdate,
	PermissionShiftsAssign,
	PermissionShiftSwapsManage,
	PermissionShiftSwapsApprove,
	PermissionPatternsRead,
	PermissionPatternsCreate,
	PermissionPatternsAssign,
	PermissionPatternsDelete,
	PermissionRostersRead,
	PermissionRostersManage,
	PermissionSchedulesRead,
	PermissionSchedulesPreview,
	PermissionSchedulesPublishOffice,
	PermissionSchedulesPublishRotating,
	PermissionSchedulesAmend,
	PermissionSchedulesEvidenceExport,
	PermissionSchedulesHistoryRead,
	PermissionSchedulesActivate,
	PermissionAssignmentRulesManage,
	PermissionPayrollRead,
	PermissionPayrollCalculate,
	PermissionPayrollSettingsManage,
	PermissionPayrollComponentsManage,
	PermissionPayrollIncentivesManage,
	PermissionPayrollInputsManage,
	PermissionHolidaysRead,
	PermissionHolidaysManage,
	PermissionReportsRead,
	PermissionReportsExport,
	PermissionDashboardRead,
	PermissionDashboardPreferences,
	PermissionDashboardEmployeeAbsenceRead,
	PermissionEmploymentPhasesRead,
	PermissionEmploymentPhasesManage,
	PermissionAttendancePolicyAssignmentsRead,
	PermissionAttendancePolicyAssignmentsManage,
	PermissionRecruitmentRead,
	PermissionRecruitmentManage,
	PermissionFeatureFlagsRead,
	PermissionFeatureFlagsManage,
	PermissionAuditLogsRead,
	PermissionAuditLogsExport,
	PermissionSecuritySettingsManage,
	PermissionEmployeesBulkEdit,
	PermissionAttendanceForceClockOut,
	PermissionDocumentsMedicalRead,
	PermissionDocumentsContractRead,
	PermissionDocumentsManage,
	PermissionFilesRead,
	PermissionFilesManage,
	PermissionPayrollApprove,
	PermissionPayrollDisburse,
	PermissionPayrollSensitiveRead,
	PermissionLeaveApproveL1,
	PermissionLeaveApproveL2,
	PermissionInfraMonitor,
	PermissionEssProfile,
	PermissionEssAttendance,
	PermissionEssClock,
	PermissionEssLeave,
	PermissionEssPayslips,
	PermissionEssShiftSwaps,
}

var permissionSet = func() map[Permission]bool {
	m := make(map[Permission]bool, len(allCanonicalPermissions))
	for _, p := range allCanonicalPermissions {
		m[p] = true
	}
	return m
}()

// AllPermissions returns all canonical system permissions.
func AllPermissions() []Permission {
	out := make([]Permission, len(allCanonicalPermissions))
	copy(out, allCanonicalPermissions)
	return out
}

// ParsePermission validates whether the string is a recognized system permission.
func ParsePermission(s string) (Permission, bool) {
	p := Permission(s)
	if permissionSet[p] {
		return p, true
	}
	return "", false
}

// Organization-level authorization context types.
type OrganizationContext = MemberContext

// MemberContext models an authenticated caller's organization membership.
// ponytail: permissions stored as map[Permission]bool for O(1) checks. Upgrade to bitset if memory critical for high-concurrency pools.
type MemberContext struct {
	UserID               ids.UserID
	UserName             string
	UserEmail            string
	Roles                []uuid.UUID
	Permissions          map[Permission]bool
	ManagedDepartmentIDs map[uuid.UUID]bool // nil = global scope (admin/owner); non-nil = department scope (team lead)
}

func (m *MemberContext) HasRole(roleID uuid.UUID) bool {
	if m == nil {
		return false
	}
	for _, r := range m.Roles {
		if r == roleID {
			return true
		}
	}
	return false
}

// HasPermission enforces strict DEFAULT-DENY.
// Nil context, empty permissions map, or unassigned permission unconditionally returns false.
func (m *MemberContext) HasPermission(perm Permission) bool {
	if m == nil || m.Permissions == nil {
		return false
	}
	return m.Permissions[perm]
}

// RequirePermission rejects callers lacking the given permission with 403 Forbidden.
func (m *MemberContext) RequirePermission(perm Permission) error {
	if !m.HasPermission(perm) {
		return apperror.Forbidden(apperror.KeyForbidden)
	}
	return nil
}

// RequireScopedPermission validates the permission and enforces department boundary if scoped.
func (m *MemberContext) RequireScopedPermission(perm Permission, targetDeptID *uuid.UUID) error {
	if err := m.RequirePermission(perm); err != nil {
		return err
	}
	// Global scope (Admin/Owner)
	if m.ManagedDepartmentIDs == nil {
		return nil
	}
	// Scoped check: target department must be specified and present in caller's managed departments
	if targetDeptID == nil || !m.ManagedDepartmentIDs[*targetDeptID] {
		return apperror.Forbidden(apperror.KeyForbidden)
	}
	return nil
}

// RequireAnyScopedPermission validates caller holds at least one permission in perms, scoped to target department.
func (m *MemberContext) RequireAnyScopedPermission(perms []Permission, targetDeptID *uuid.UUID) error {
	if m == nil {
		return apperror.Forbidden(apperror.KeyForbidden)
	}
	hasPerm := false
	for _, p := range perms {
		if m.HasPermission(p) {
			hasPerm = true
			break
		}
	}
	if !hasPerm {
		return apperror.Forbidden(apperror.KeyForbidden)
	}

	// Global scope
	if m.ManagedDepartmentIDs == nil {
		return nil
	}

	// Scoped check
	if targetDeptID == nil || !m.ManagedDepartmentIDs[*targetDeptID] {
		return apperror.Forbidden(apperror.KeyForbidden)
	}
	return nil
}

// EmployeeContext models self-service employee context linked to an organization member.
type EmployeeContext struct {
	UserID     uuid.UUID
	EmployeeID uuid.UUID
	Member     *MemberContext
}
