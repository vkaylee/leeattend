package api

import (
	"bytes"
	"context"
	"encoding/json"
	"net/http"
	"net/http/httptest"
	"testing"
	"time"

	"github.com/google/uuid"
	"github.com/vkaylee/leeattend/apiserver/internal/apperror"
	httpinfra "github.com/vkaylee/leeattend/apiserver/internal/infra/http"
)

type mockRBACRepo struct {
	members                []MemberDetail
	roles                  []Role
	extraRoles             []MemberExtraRole
	permissions            []MemberExtraPermission
	rolePerms              map[uuid.UUID][]string
	userEmails             map[string]uuid.UUID
	roleIsMutableFn        func(role uuid.UUID) (bool, error)
	deleteRoleFn           func(role uuid.UUID) (int64, error)
	updateRoleFn           func(role uuid.UUID, req UpdateRoleRequest) (Role, error)
	linkMemberEmployeeFn   func(actorID, userID, employeeID uuid.UUID) error
	unlinkMemberEmployeeFn func(actorID, userID uuid.UUID) error
}

func (m *mockRBACRepo) ListMembers(_ context.Context, _, _ int) ([]MemberDetail, int64, error) {
	return m.members, int64(len(m.members)), nil
}
func (m *mockRBACRepo) ListExtraRoles(_ context.Context) ([]MemberExtraRole, error) {
	return m.extraRoles, nil
}
func (m *mockRBACRepo) FindUserByEmail(_ context.Context, email string) (uuid.UUID, error) {
	if id, ok := m.userEmails[email]; ok {
		return id, nil
	}
	return uuid.Nil, ErrRBACUserNotFound
}
func (m *mockRBACRepo) AddMember(_ context.Context, user, role uuid.UUID) error {
	m.members = append(m.members, MemberDetail{UserID: user, RoleID: role, Email: "user@test.com"})
	return nil
}
func (m *mockRBACRepo) FindPrimaryRole(_ context.Context, user uuid.UUID) (*uuid.UUID, error) {
	for _, x := range m.members {
		if x.UserID == user {
			return &x.RoleID, nil
		}
	}
	return nil, nil
}
func (m *mockRBACRepo) UpdateMemberRole(_ context.Context, user, role uuid.UUID) error {
	for i, x := range m.members {
		if x.UserID == user {
			m.members[i].RoleID = role
			return nil
		}
	}
	return ErrRBACNotFound
}
func (m *mockRBACRepo) RemoveMember(_ context.Context, user uuid.UUID) error {
	for i, x := range m.members {
		if x.UserID == user {
			m.members = append(m.members[:i], m.members[i+1:]...)
			return nil
		}
	}
	return ErrRBACNotFound
}
func (m *mockRBACRepo) ListMemberRoles(_ context.Context, user uuid.UUID) (*Role, []MemberExtraRole, error) {
	return nil, m.extraRoles, nil
}
func (m *mockRBACRepo) AssignExtraRole(_ context.Context, user, role uuid.UUID, exp time.Time) (MemberExtraRole, error) {
	x := MemberExtraRole{ID: uuid.New(), UserID: user, RoleID: role, ExpiresAt: exp}
	m.extraRoles = append(m.extraRoles, x)
	return x, nil
}
func (m *mockRBACRepo) RevokeExtraRole(_ context.Context, roleID uuid.UUID) (int64, error) {
	for i, x := range m.extraRoles {
		if x.ID == roleID {
			m.extraRoles = append(m.extraRoles[:i], m.extraRoles[i+1:]...)
			return 1, nil
		}
	}
	return 0, nil
}
func (m *mockRBACRepo) ListMemberPermissions(_ context.Context, _ uuid.UUID) ([]MemberExtraPermission, error) {
	return m.permissions, nil
}
func (m *mockRBACRepo) AssignExtraPermission(_ context.Context, user uuid.UUID, p string, exp time.Time, by uuid.UUID) (MemberExtraPermission, error) {
	x := MemberExtraPermission{ID: uuid.New(), UserID: user, Permission: p, ExpiresAt: exp, AssignedBy: &by}
	m.permissions = append(m.permissions, x)
	return x, nil
}
func (m *mockRBACRepo) LinkMemberEmployee(_ context.Context, actorID, userID, employeeID uuid.UUID) error {
	if m.linkMemberEmployeeFn != nil {
		return m.linkMemberEmployeeFn(actorID, userID, employeeID)
	}
	return nil
}
func (m *mockRBACRepo) UnlinkMemberEmployee(_ context.Context, actorID, userID uuid.UUID) error {
	if m.unlinkMemberEmployeeFn != nil {
		return m.unlinkMemberEmployeeFn(actorID, userID)
	}
	return nil
}
func (m *mockRBACRepo) RevokeExtraPermission(_ context.Context, permID uuid.UUID) (int64, error) {
	for i, x := range m.permissions {
		if x.ID == permID {
			m.permissions = append(m.permissions[:i], m.permissions[i+1:]...)
			return 1, nil
		}
	}
	return 0, nil
}
func (m *mockRBACRepo) ListRoles(_ context.Context, _, _ int) ([]Role, int64, error) {
	return m.roles, int64(len(m.roles)), nil
}
func (m *mockRBACRepo) CreateRole(_ context.Context, req CreateRoleRequest) (Role, error) {
	r := Role{ID: uuid.New(), Name: req.Name, ScopeType: req.ScopeType}
	m.roles = append(m.roles, r)
	return r, nil
}
func (m *mockRBACRepo) UpdateRole(_ context.Context, role uuid.UUID, req UpdateRoleRequest) (Role, error) {
	if m.updateRoleFn != nil {
		return m.updateRoleFn(role, req)
	}
	return Role{ID: role, Name: "updated"}, nil
}
func (m *mockRBACRepo) DeleteRole(_ context.Context, role uuid.UUID) (int64, error) {
	if m.deleteRoleFn != nil {
		return m.deleteRoleFn(role)
	}
	return 1, nil
}
func (m *mockRBACRepo) RolePermissions(_ context.Context, role uuid.UUID) ([]string, error) {
	return m.rolePerms[role], nil
}
func (m *mockRBACRepo) RoleIsMutable(_ context.Context, role uuid.UUID) (bool, error) {
	if m.roleIsMutableFn != nil {
		return m.roleIsMutableFn(role)
	}
	return true, nil
}
func (m *mockRBACRepo) SetRolePermissions(_ context.Context, role uuid.UUID, perms []string) error {
	if m.rolePerms == nil {
		m.rolePerms = map[uuid.UUID][]string{}
	}
	m.rolePerms[role] = perms
	return nil
}

type rbacAuthorizer struct {
	perms  map[string]bool
	member bool
}

func (a rbacAuthorizer) IsMember(context.Context, uuid.UUID) (bool, error) {
	return a.member, nil
}
func (a rbacAuthorizer) HasPermission(_ context.Context, _ uuid.UUID, p string) bool {
	return a.perms[p]
}
func (a rbacAuthorizer) EffectivePermissions(context.Context, uuid.UUID) ([]string, []string, error) {
	return nil, nil, nil
}
func (a rbacAuthorizer) GetManagedDepartmentIDs(context.Context, uuid.UUID) ([]uuid.UUID, error) {
	return nil, nil
}
func (a rbacAuthorizer) IsGlobalManager(context.Context, uuid.UUID) bool { return false }
func (a rbacAuthorizer) HasScopedPermission(context.Context, uuid.UUID, string, *uuid.UUID) bool {
	return false
}
func (a rbacAuthorizer) GetUsersWithPermission(context.Context, string) ([]uuid.UUID, error) {
	return nil, nil
}

func TestOrganizationRBAC_Permissions(t *testing.T) {
	actorID, targetUser := uuid.New(), uuid.New()
	roleID := uuid.New()
	repo := &mockRBACRepo{
		userEmails: map[string]uuid.UUID{"new@test.com": targetUser},
	}
	authz := rbacAuthorizer{perms: map[string]bool{}, member: true}
	h := OrganizationRBACHandler{Repository: repo, Authorizer: authz}

	// 1. AddMember without members:invite -> 403 Forbidden
	reqBody := `{"email":"new@test.com","role_id":"` + roleID.String() + `"}`
	req := httptest.NewRequest(http.MethodPost, "/api/v1/members", bytes.NewBufferString(reqBody))
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), actorID))
	rec := httptest.NewRecorder()
	h.AddMember(rec, req)
	if rec.Code != http.StatusForbidden {
		t.Fatalf("expected 403, got %d", rec.Code)
	}

	// 2. AddMember with members:invite -> 201 Created
	authz.perms["members:invite"] = true
	rec = httptest.NewRecorder()
	h.AddMember(rec, req)
	if rec.Code != http.StatusCreated {
		t.Fatalf("expected 201, got %d", rec.Code)
	}

	// 3. AssignExtraPermission with unknown permission string -> 400 Validation
	authz.perms["members:roles:manage"] = true
	future := time.Now().Add(24 * time.Hour).Format(time.RFC3339)
	reqBody = `{"permission":"invalid:unknown:perm","expires_at":"` + future + `"}`
	req = httptest.NewRequest(http.MethodPost, "/api/v1/members/"+targetUser.String()+"/permissions", bytes.NewBufferString(reqBody))
	req.SetPathValue("user_id", targetUser.String())
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), actorID))
	rec = httptest.NewRecorder()
	h.AssignExtraPermission(rec, req)
	if rec.Code != http.StatusBadRequest {
		t.Fatalf("expected 400 for invalid perm, got %d", rec.Code)
	}

	// 4. AssignExtraPermission with past expiration -> 400 Validation
	past := time.Now().Add(-24 * time.Hour).Format(time.RFC3339)
	reqBody = `{"permission":"members:read","expires_at":"` + past + `"}`
	req = httptest.NewRequest(http.MethodPost, "/api/v1/members/"+targetUser.String()+"/permissions", bytes.NewBufferString(reqBody))
	req.SetPathValue("user_id", targetUser.String())
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), actorID))
	rec = httptest.NewRecorder()
	h.AssignExtraPermission(rec, req)
	if rec.Code != http.StatusBadRequest {
		t.Fatalf("expected 400 for past expiration, got %d", rec.Code)
	}
}

func TestOrganizationRBAC_MemberEmployeeLinkLifecycle(t *testing.T) {
	actorID, userID, employeeID := uuid.New(), uuid.New(), uuid.New()
	linked := false
	repo := &mockRBACRepo{
		linkMemberEmployeeFn: func(gotActor, gotUser, gotEmployee uuid.UUID) error {
			if gotActor != actorID || gotUser != userID || gotEmployee != employeeID {
				t.Fatalf("unexpected link args: %s %s %s", gotActor, gotUser, gotEmployee)
			}
			linked = true
			return nil
		},
		unlinkMemberEmployeeFn: func(gotActor, gotUser uuid.UUID) error {
			if gotActor != actorID || gotUser != userID {
				t.Fatalf("unexpected unlink args: %s %s", gotActor, gotUser)
			}
			linked = false
			return nil
		},
	}
	authz := rbacAuthorizer{perms: map[string]bool{"members:employee:link": true}, member: true}
	h := OrganizationRBACHandler{Repository: repo, Authorizer: authz}

	req := httptest.NewRequest(http.MethodPut, "/api/v1/members/"+userID.String()+"/employee", bytes.NewBufferString(`{"employee_id":"`+employeeID.String()+`"}`))
	req.SetPathValue("user_id", userID.String())
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), actorID))
	rec := httptest.NewRecorder()
	h.LinkMemberEmployee(rec, req)
	if rec.Code != http.StatusNoContent || !linked {
		t.Fatalf("expected link 204 and linked state, got %d linked=%t", rec.Code, linked)
	}

	req = httptest.NewRequest(http.MethodDelete, "/api/v1/members/"+userID.String()+"/employee", nil)
	req.SetPathValue("user_id", userID.String())
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), actorID))
	rec = httptest.NewRecorder()
	h.UnlinkMemberEmployee(rec, req)
	if rec.Code != http.StatusNoContent || linked {
		t.Fatalf("expected unlink 204 and unlinked state, got %d linked=%t", rec.Code, linked)
	}
}

func TestOrganizationRBAC_MemberEmployeeLinkRequiresPermission(t *testing.T) {
	actorID, userID := uuid.New(), uuid.New()
	h := OrganizationRBACHandler{Repository: &mockRBACRepo{}, Authorizer: rbacAuthorizer{member: true}}
	req := httptest.NewRequest(http.MethodDelete, "/api/v1/members/"+userID.String()+"/employee", nil)
	req.SetPathValue("user_id", userID.String())
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), actorID))
	rec := httptest.NewRecorder()
	h.UnlinkMemberEmployee(rec, req)
	if rec.Code != http.StatusForbidden {
		t.Fatalf("expected 403 without link permission, got %d", rec.Code)
	}
}

func TestOrganizationRBAC_RejectSelfRemoval(t *testing.T) {
	actorID, otherID := uuid.New(), uuid.New()
	repo := &mockRBACRepo{
		members: []MemberDetail{
			{UserID: actorID},
			{UserID: otherID},
		},
	}
	authz := rbacAuthorizer{perms: map[string]bool{"members:remove": true}, member: true}
	h := OrganizationRBACHandler{Repository: repo, Authorizer: authz}

	// 1. Attempt self removal -> 400 Validation (error.cannot_remove_self)
	req := httptest.NewRequest(http.MethodDelete, "/api/v1/members/"+actorID.String(), nil)
	req.SetPathValue("user_id", actorID.String())
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), actorID))
	rec := httptest.NewRecorder()
	h.RemoveMember(rec, req)

	if rec.Code != http.StatusBadRequest {
		t.Fatalf("expected 400 for self removal, got %d", rec.Code)
	}
	var env apperror.WireErrorEnvelope
	if err := json.Unmarshal(rec.Body.Bytes(), &env); err != nil {
		t.Fatalf("failed unmarshaling error envelope: %v", err)
	}
	if env.Error.Code != apperror.CodeValidationError {
		t.Errorf("expected VALIDATION_ERROR, got %s", env.Error.Code)
	}

	// 2. Remove other user -> 204 No Content
	req = httptest.NewRequest(http.MethodDelete, "/api/v1/members/"+otherID.String(), nil)
	req.SetPathValue("user_id", otherID.String())
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), actorID))
	rec = httptest.NewRecorder()
	h.RemoveMember(rec, req)

	if rec.Code != http.StatusNoContent {
		t.Fatalf("expected 204 for other removal, got %d", rec.Code)
	}
}

func TestOrganizationRBAC_SystemRoleMutationProtection(t *testing.T) {
	actorID, systemRoleID := uuid.New(), uuid.New()
	repo := &mockRBACRepo{
		deleteRoleFn: func(_ uuid.UUID) (int64, error) {
			// System role deletion blocked
			return 0, nil
		},
		updateRoleFn: func(_ uuid.UUID, _ UpdateRoleRequest) (Role, error) {
			return Role{}, ErrRBACProtected
		},
		roleIsMutableFn: func(_ uuid.UUID) (bool, error) {
			return false, nil
		},
	}
	authz := rbacAuthorizer{perms: map[string]bool{"members:roles:manage": true}, member: true}
	h := OrganizationRBACHandler{Repository: repo, Authorizer: authz}

	// 1. Update system role -> 404 Not Found (role not found or is system role)
	req := httptest.NewRequest(http.MethodPut, "/api/v1/roles/"+systemRoleID.String(), bytes.NewBufferString(`{"name":"NewName"}`))
	req.SetPathValue("role_id", systemRoleID.String())
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), actorID))
	rec := httptest.NewRecorder()
	h.UpdateRole(rec, req)
	if rec.Code != http.StatusNotFound {
		t.Fatalf("expected 404 for system role update, got %d", rec.Code)
	}

	// 2. Delete system role -> 404 Not Found
	req = httptest.NewRequest(http.MethodDelete, "/api/v1/roles/"+systemRoleID.String(), nil)
	req.SetPathValue("role_id", systemRoleID.String())
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), actorID))
	rec = httptest.NewRecorder()
	h.DeleteRole(rec, req)
	if rec.Code != http.StatusNotFound {
		t.Fatalf("expected 404 for system role delete, got %d", rec.Code)
	}

	// 3. Set system role permissions -> 404 Not Found
	req = httptest.NewRequest(http.MethodPut, "/api/v1/roles/"+systemRoleID.String()+"/permissions", bytes.NewBufferString(`{"permissions":["members:read"]}`))
	req.SetPathValue("role_id", systemRoleID.String())
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), actorID))
	rec = httptest.NewRecorder()
	h.SetRolePermissions(rec, req)
	if rec.Code != http.StatusNotFound {
		t.Fatalf("expected 404 for system role permissions update, got %d", rec.Code)
	}
}

func TestOrganizationRBAC_LastAdminProtection(t *testing.T) {
	actorID, targetUser := uuid.New(), uuid.New()
	newRoleID := uuid.New()
	repo := &mockRBACRepo{
		updateRoleFn: nil,
	}
	authz := rbacAuthorizer{perms: map[string]bool{"members:roles:manage": true, "members:remove": true}, member: true}

	// 1. UpdateMemberRole fails with ErrRBACLastAdmin -> 409 Conflict
	repo.members = []MemberDetail{{UserID: targetUser, RoleID: uuid.New()}}
	// Override UpdateMemberRole behavior by updating member with last admin error simulation
	origRepo := repo
	_ = origRepo

	// Test with a repo that returns ErrRBACLastAdmin on UpdateMemberRole
	lastAdminRepo := &mockLastAdminRepo{mockRBACRepo: *repo}
	hLastAdmin := OrganizationRBACHandler{Repository: lastAdminRepo, Authorizer: authz}

	reqBody := `{"role_id":"` + newRoleID.String() + `"}`
	req := httptest.NewRequest(http.MethodPatch, "/api/v1/members/"+targetUser.String(), bytes.NewBufferString(reqBody))
	req.SetPathValue("user_id", targetUser.String())
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), actorID))
	rec := httptest.NewRecorder()
	hLastAdmin.UpdateMemberRole(rec, req)
	if rec.Code != http.StatusConflict {
		t.Fatalf("expected 409 Conflict on last admin role change, got %d", rec.Code)
	}

	// 2. RemoveMember fails with ErrRBACLastAdmin -> 409 Conflict
	req = httptest.NewRequest(http.MethodDelete, "/api/v1/members/"+targetUser.String(), nil)
	req.SetPathValue("user_id", targetUser.String())
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), actorID))
	rec = httptest.NewRecorder()
	hLastAdmin.RemoveMember(rec, req)
	if rec.Code != http.StatusConflict {
		t.Fatalf("expected 409 Conflict on last admin removal, got %d", rec.Code)
	}
}

type mockLastAdminRepo struct {
	mockRBACRepo
}

func (m *mockLastAdminRepo) UpdateMemberRole(_ context.Context, _, _ uuid.UUID) error {
	return ErrRBACLastAdmin
}
func (m *mockLastAdminRepo) RemoveMember(_ context.Context, _ uuid.UUID) error {
	return ErrRBACLastAdmin
}

func TestOrganizationRBAC_MemberContextRequired(t *testing.T) {
	actorID := uuid.New()
	repo := &mockRBACRepo{}
	authz := rbacAuthorizer{member: false}
	h := OrganizationRBACHandler{Repository: repo, Authorizer: authz}

	// 1. Unauthenticated -> 401
	req := httptest.NewRequest(http.MethodGet, "/api/v1/members", nil)
	rec := httptest.NewRecorder()
	h.ListMembers(rec, req)
	if rec.Code != http.StatusUnauthorized {
		t.Fatalf("expected 401 for unauthenticated member check, got %d", rec.Code)
	}

	// 2. Non-member -> 403
	req = httptest.NewRequest(http.MethodGet, "/api/v1/members", nil)
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), actorID))
	rec = httptest.NewRecorder()
	h.ListMembers(rec, req)
	if rec.Code != http.StatusForbidden {
		t.Fatalf("expected 403 for non-member, got %d", rec.Code)
	}
}
