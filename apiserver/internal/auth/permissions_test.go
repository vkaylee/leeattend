package auth

import (
	"errors"
	"net/http"
	"testing"

	"github.com/google/uuid"
	"github.com/vkaylee/leeattend/apiserver/internal/apperror"
)

func TestCanonicalPermissionsCatalog(t *testing.T) {
	all := AllPermissions()

	// Canonical permissions exclude retired edge-agent and device controls.
	if len(all) != 99 {
		t.Fatalf("expected exactly 99 permissions, got %d", len(all))
	}

	// 2. Check no duplicates
	seen := make(map[Permission]bool, len(all))
	for _, p := range all {
		if seen[p] {
			t.Fatalf("duplicate permission detected: %s", p)
		}
		seen[p] = true

		// 3. Roundtrip parsing
		parsed, ok := ParsePermission(string(p))
		if !ok || parsed != p {
			t.Errorf("failed to parse permission string %s", p)
		}
	}

	// 4. Invalid permission rejection
	if _, ok := ParsePermission("non_existent:permission"); ok {
		t.Error("expected non_existent:permission to be rejected")
	}
}

func TestMemberContextDefaultDeny(t *testing.T) {
	// 1. Nil context MUST default-deny
	var nilCtx *MemberContext
	if nilCtx.HasPermission(PermissionEmployeesRead) {
		t.Error("nil context must return false for HasPermission")
	}
	err := nilCtx.RequirePermission(PermissionEmployeesRead)
	assertForbidden(t, err)

	// 2. Context with empty permissions MUST default-deny
	emptyCtx := &MemberContext{
		UserID:      uuid.New(),
		Permissions: map[Permission]bool{},
	}
	if emptyCtx.HasPermission(PermissionEmployeesRead) {
		t.Error("empty permissions map must return false for HasPermission")
	}
	assertForbidden(t, emptyCtx.RequirePermission(PermissionEmployeesRead))

	// 3. Context with specific permissions MUST deny unassigned permissions
	memberCtx := &MemberContext{
		UserID: uuid.New(),
		Permissions: map[Permission]bool{
			PermissionEmployeesRead: true,
		},
	}
	if !memberCtx.HasPermission(PermissionEmployeesRead) {
		t.Error("expected granted permission to be true")
	}
	if err := memberCtx.RequirePermission(PermissionEmployeesRead); err != nil {
		t.Errorf("expected no error for granted permission, got %v", err)
	}

	if memberCtx.HasPermission(PermissionEmployeesDelete) {
		t.Error("unassigned permission must return false")
	}
	assertForbidden(t, memberCtx.RequirePermission(PermissionEmployeesDelete))
}

func TestMemberContextScopedPermissions(t *testing.T) {
	deptA := uuid.New()
	deptB := uuid.New()

	// 1. Global scope (ManagedDepartmentIDs == nil) allows any department
	adminCtx := &MemberContext{
		UserID:               uuid.New(),
		Permissions:          map[Permission]bool{PermissionShiftsAssign: true},
		ManagedDepartmentIDs: nil, // global scope
	}
	if err := adminCtx.RequireScopedPermission(PermissionShiftsAssign, &deptA); err != nil {
		t.Errorf("global scope should allow deptA: %v", err)
	}
	if err := adminCtx.RequireScopedPermission(PermissionShiftsAssign, nil); err != nil {
		t.Errorf("global scope should allow nil dept: %v", err)
	}

	// 2. Department-scoped context restricts access
	teamLeadCtx := &MemberContext{
		UserID:      uuid.New(),
		Permissions: map[Permission]bool{PermissionShiftsAssign: true},
		ManagedDepartmentIDs: map[uuid.UUID]bool{
			deptA: true,
		},
	}

	// Target in managed department -> Allowed
	if err := teamLeadCtx.RequireScopedPermission(PermissionShiftsAssign, &deptA); err != nil {
		t.Errorf("scoped check should allow managed deptA: %v", err)
	}

	// Target outside managed department -> Denied (403)
	assertForbidden(t, teamLeadCtx.RequireScopedPermission(PermissionShiftsAssign, &deptB))

	// Nil target for scoped role -> Denied (403)
	assertForbidden(t, teamLeadCtx.RequireScopedPermission(PermissionShiftsAssign, nil))

	// Unassigned permission for scoped role -> Denied (403)
	assertForbidden(t, teamLeadCtx.RequireScopedPermission(PermissionPayrollRead, &deptA))
}

func TestMemberContextRequireAnyScopedPermission(t *testing.T) {
	deptA := uuid.New()
	ctx := &MemberContext{
		UserID: uuid.New(),
		Permissions: map[Permission]bool{
			PermissionOvertimeClaimOnBehalf: true,
		},
		ManagedDepartmentIDs: map[uuid.UUID]bool{deptA: true},
	}

	// Has one of the permissions
	err := ctx.RequireAnyScopedPermission([]Permission{
		PermissionOvertimeApprove,
		PermissionOvertimeClaimOnBehalf,
	}, &deptA)
	if err != nil {
		t.Errorf("expected access allowed when holding one of the permissions: %v", err)
	}

	// Has none of the permissions
	errNone := ctx.RequireAnyScopedPermission([]Permission{
		PermissionOvertimeApprove,
		PermissionOvertimePaperUpload,
	}, &deptA)
	assertForbidden(t, errNone)
}

func TestSystemContextDefaultDeny(t *testing.T) {
	var nilCtx *SystemContext
	assertForbidden(t, nilCtx.RequireRole(SystemRolePlatformAdmin))

	userCtx := &SystemContext{
		UserID: uuid.New(),
		Roles:  []SystemRoleType{SystemRoleSupport},
	}

	if !userCtx.HasRole(SystemRoleSupport) {
		t.Error("expected user to have Support role")
	}
	if userCtx.HasRole(SystemRolePlatformAdmin) {
		t.Error("user should not have PlatformAdmin role")
	}

	if err := userCtx.RequireRole(SystemRoleSupport); err != nil {
		t.Errorf("expected success for granted role: %v", err)
	}
	assertForbidden(t, userCtx.RequireRole(SystemRolePlatformAdmin))

	// RequireAnyRole
	if err := userCtx.RequireAnyRole(SystemRolePlatformAdmin, SystemRoleSupport); err != nil {
		t.Errorf("expected success when holding one of the roles: %v", err)
	}
	assertForbidden(t, userCtx.RequireAnyRole(SystemRolePlatformAdmin))
}

func assertForbidden(t *testing.T, err error) {
	t.Helper()
	if err == nil {
		t.Fatal("expected 403 Forbidden error, got nil")
	}
	var appErr *apperror.AppError
	if !errors.As(err, &appErr) {
		t.Fatalf("expected AppError, got %T: %v", err, err)
	}
	if appErr.Status != http.StatusForbidden {
		t.Errorf("expected status 403 Forbidden, got %d", appErr.Status)
	}
	if appErr.Code != apperror.CodeForbidden {
		t.Errorf("expected code FORBIDDEN, got %s", appErr.Code)
	}
}
