package api

import (
	"bytes"
	"context"
	"encoding/json"
	"errors"
	"net/http"
	"net/http/httptest"
	"testing"
	"time"

	"github.com/google/uuid"
	"github.com/vkaylee/leeattend/apiserver/internal/apperror"
	"github.com/vkaylee/leeattend/apiserver/internal/infra/crypto"
	httpinfra "github.com/vkaylee/leeattend/apiserver/internal/infra/http"
)

type mockRBACRepo struct {
	members         []WorkspaceMemberDetail
	roles           []WorkspaceRole
	extraRoles      []MemberExtraRole
	permissions     []MemberExtraPermission
	rolePerms       map[uuid.UUID][]string
	storageConfig   *StorageConfigRecord
	userEmails      map[string]uuid.UUID
	roleIsMutableFn func(ws, role uuid.UUID) (bool, error)
	deleteRoleFn    func(ws, role uuid.UUID) (int64, error)
	updateRoleFn    func(ws, role uuid.UUID, req UpdateWorkspaceRoleRequest) (WorkspaceRole, error)
}

func (m *mockRBACRepo) ListMembers(_ context.Context, _ uuid.UUID, _, _ int) ([]WorkspaceMemberDetail, int64, error) {
	return m.members, int64(len(m.members)), nil
}
func (m *mockRBACRepo) ListExtraRoles(_ context.Context, _ uuid.UUID) ([]MemberExtraRole, error) {
	return m.extraRoles, nil
}
func (m *mockRBACRepo) FindUserByEmail(_ context.Context, email string) (uuid.UUID, error) {
	if id, ok := m.userEmails[email]; ok {
		return id, nil
	}
	return uuid.Nil, ErrRBACUserNotFound
}
func (m *mockRBACRepo) AddMember(_ context.Context, ws, user, role uuid.UUID) error {
	m.members = append(m.members, WorkspaceMemberDetail{WorkspaceID: ws, UserID: user, RoleID: role, Email: "user@test.com"})
	return nil
}
func (m *mockRBACRepo) FindPrimaryRole(_ context.Context, _, user uuid.UUID) (*uuid.UUID, error) {
	for _, x := range m.members {
		if x.UserID == user {
			return &x.RoleID, nil
		}
	}
	return nil, nil
}
func (m *mockRBACRepo) UpdateMemberRole(_ context.Context, _, user, role uuid.UUID) error {
	for i, x := range m.members {
		if x.UserID == user {
			m.members[i].RoleID = role
			return nil
		}
	}
	return ErrRBACNotFound
}
func (m *mockRBACRepo) RemoveMember(_ context.Context, _, user uuid.UUID) error {
	for i, x := range m.members {
		if x.UserID == user {
			m.members = append(m.members[:i], m.members[i+1:]...)
			return nil
		}
	}
	return ErrRBACNotFound
}
func (m *mockRBACRepo) ListMemberRoles(_ context.Context, _, user uuid.UUID) (*WorkspaceRole, []MemberExtraRole, error) {
	return nil, m.extraRoles, nil
}
func (m *mockRBACRepo) AssignExtraRole(_ context.Context, ws, user, role uuid.UUID, exp time.Time) (MemberExtraRole, error) {
	x := MemberExtraRole{ID: uuid.New(), WorkspaceID: ws, UserID: user, RoleID: role, ExpiresAt: exp}
	m.extraRoles = append(m.extraRoles, x)
	return x, nil
}
func (m *mockRBACRepo) RevokeExtraRole(_ context.Context, _, roleID uuid.UUID) (int64, error) {
	for i, x := range m.extraRoles {
		if x.ID == roleID {
			m.extraRoles = append(m.extraRoles[:i], m.extraRoles[i+1:]...)
			return 1, nil
		}
	}
	return 0, nil
}
func (m *mockRBACRepo) ListMemberPermissions(_ context.Context, _, _ uuid.UUID) ([]MemberExtraPermission, error) {
	return m.permissions, nil
}
func (m *mockRBACRepo) AssignExtraPermission(_ context.Context, ws, user uuid.UUID, p string, exp time.Time, by uuid.UUID) (MemberExtraPermission, error) {
	x := MemberExtraPermission{ID: uuid.New(), WorkspaceID: ws, UserID: user, Permission: p, ExpiresAt: exp, AssignedBy: &by}
	m.permissions = append(m.permissions, x)
	return x, nil
}
func (m *mockRBACRepo) RevokeExtraPermission(_ context.Context, _, permID uuid.UUID) (int64, error) {
	for i, x := range m.permissions {
		if x.ID == permID {
			m.permissions = append(m.permissions[:i], m.permissions[i+1:]...)
			return 1, nil
		}
	}
	return 0, nil
}
func (m *mockRBACRepo) ListRoles(_ context.Context, _ uuid.UUID, _, _ int) ([]WorkspaceRole, int64, error) {
	return m.roles, int64(len(m.roles)), nil
}
func (m *mockRBACRepo) CreateRole(_ context.Context, ws uuid.UUID, req CreateWorkspaceRoleRequest) (WorkspaceRole, error) {
	r := WorkspaceRole{ID: uuid.New(), WorkspaceID: &ws, Name: req.Name, ScopeType: req.ScopeType}
	m.roles = append(m.roles, r)
	return r, nil
}
func (m *mockRBACRepo) UpdateRole(_ context.Context, ws, role uuid.UUID, req UpdateWorkspaceRoleRequest) (WorkspaceRole, error) {
	if m.updateRoleFn != nil {
		return m.updateRoleFn(ws, role, req)
	}
	return WorkspaceRole{ID: role, WorkspaceID: &ws, Name: "updated"}, nil
}
func (m *mockRBACRepo) DeleteRole(_ context.Context, ws, role uuid.UUID) (int64, error) {
	if m.deleteRoleFn != nil {
		return m.deleteRoleFn(ws, role)
	}
	return 1, nil
}
func (m *mockRBACRepo) RolePermissions(_ context.Context, _, role uuid.UUID) ([]string, error) {
	return m.rolePerms[role], nil
}
func (m *mockRBACRepo) RoleIsMutable(_ context.Context, ws, role uuid.UUID) (bool, error) {
	if m.roleIsMutableFn != nil {
		return m.roleIsMutableFn(ws, role)
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
func (m *mockRBACRepo) GetStorageConfig(_ context.Context, _ uuid.UUID) (*StorageConfigRecord, error) {
	return m.storageConfig, nil
}
func (m *mockRBACRepo) UpsertStorageConfig(_ context.Context, _ uuid.UUID, rec StorageConfigRecord) (*StorageConfigRecord, error) {
	m.storageConfig = &rec
	return &rec, nil
}
func (m *mockRBACRepo) DeleteStorageConfig(_ context.Context, _ uuid.UUID) error {
	m.storageConfig = nil
	return nil
}

type rbacAuthorizer struct {
	perms map[string]bool
}

func (a rbacAuthorizer) IsSystemAdmin(_ context.Context, _ uuid.UUID) bool { return false }
func (a rbacAuthorizer) HasPermission(_ context.Context, _, _ uuid.UUID, p string) bool {
	return a.perms[p]
}

func newTestCrypto() *crypto.Service {
	key := bytes.Repeat([]byte("k"), 32)
	svc, err := crypto.NewService("v1", map[string][]byte{"v1": key})
	if err != nil {
		panic(err)
	}
	return svc
}

func TestWorkspaceRBAC_Permissions(t *testing.T) {
	wsID, actorID, targetUser := uuid.New(), uuid.New(), uuid.New()
	roleID := uuid.New()
	repo := &mockRBACRepo{
		userEmails: map[string]uuid.UUID{"new@test.com": targetUser},
	}
	authz := rbacAuthorizer{perms: map[string]bool{}}
	h := WorkspaceRBACHandler{Repository: repo, Authorizer: authz}

	// 1. AddMember without members:invite -> 403 Forbidden
	reqBody := `{"email":"new@test.com","role_id":"` + roleID.String() + `"}`
	req := httptest.NewRequest(http.MethodPost, "/api/v1/workspaces/"+wsID.String()+"/members", bytes.NewBufferString(reqBody))
	req.SetPathValue("ws_id", wsID.String())
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
	req = httptest.NewRequest(http.MethodPost, "/api/v1/workspaces/"+wsID.String()+"/members/"+targetUser.String()+"/permissions", bytes.NewBufferString(reqBody))
	req.SetPathValue("ws_id", wsID.String())
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
	req = httptest.NewRequest(http.MethodPost, "/api/v1/workspaces/"+wsID.String()+"/members/"+targetUser.String()+"/permissions", bytes.NewBufferString(reqBody))
	req.SetPathValue("ws_id", wsID.String())
	req.SetPathValue("user_id", targetUser.String())
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), actorID))
	rec = httptest.NewRecorder()
	h.AssignExtraPermission(rec, req)
	if rec.Code != http.StatusBadRequest {
		t.Fatalf("expected 400 for past expiration, got %d", rec.Code)
	}
}

func TestWorkspaceRBAC_RejectSelfRemoval(t *testing.T) {
	wsID, actorID, otherID := uuid.New(), uuid.New(), uuid.New()
	repo := &mockRBACRepo{
		members: []WorkspaceMemberDetail{
			{WorkspaceID: wsID, UserID: actorID},
			{WorkspaceID: wsID, UserID: otherID},
		},
	}
	authz := rbacAuthorizer{perms: map[string]bool{"members:remove": true}}
	h := WorkspaceRBACHandler{Repository: repo, Authorizer: authz}

	// 1. Attempt self removal -> 400 Validation (error.cannot_remove_self)
	req := httptest.NewRequest(http.MethodDelete, "/api/v1/workspaces/"+wsID.String()+"/members/"+actorID.String(), nil)
	req.SetPathValue("ws_id", wsID.String())
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
	req = httptest.NewRequest(http.MethodDelete, "/api/v1/workspaces/"+wsID.String()+"/members/"+otherID.String(), nil)
	req.SetPathValue("ws_id", wsID.String())
	req.SetPathValue("user_id", otherID.String())
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), actorID))
	rec = httptest.NewRecorder()
	h.RemoveMember(rec, req)

	if rec.Code != http.StatusNoContent {
		t.Fatalf("expected 204 for other removal, got %d", rec.Code)
	}
}

func TestWorkspaceRBAC_SystemRoleMutationProtection(t *testing.T) {
	wsID, actorID, systemRoleID := uuid.New(), uuid.New(), uuid.New()
	repo := &mockRBACRepo{
		deleteRoleFn: func(_, _ uuid.UUID) (int64, error) {
			// System role deletion blocked
			return 0, nil
		},
		updateRoleFn: func(_, _ uuid.UUID, _ UpdateWorkspaceRoleRequest) (WorkspaceRole, error) {
			return WorkspaceRole{}, ErrRBACProtected
		},
		roleIsMutableFn: func(_, _ uuid.UUID) (bool, error) {
			return false, nil
		},
	}
	authz := rbacAuthorizer{perms: map[string]bool{"members:roles:manage": true}}
	h := WorkspaceRBACHandler{Repository: repo, Authorizer: authz}

	// 1. Update system role -> 404 Not Found (role not found or is system role)
	req := httptest.NewRequest(http.MethodPut, "/api/v1/workspaces/"+wsID.String()+"/roles/"+systemRoleID.String(), bytes.NewBufferString(`{"name":"NewName"}`))
	req.SetPathValue("ws_id", wsID.String())
	req.SetPathValue("role_id", systemRoleID.String())
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), actorID))
	rec := httptest.NewRecorder()
	h.UpdateRole(rec, req)
	if rec.Code != http.StatusNotFound {
		t.Fatalf("expected 404 for system role update, got %d", rec.Code)
	}

	// 2. Delete system role -> 404 Not Found
	req = httptest.NewRequest(http.MethodDelete, "/api/v1/workspaces/"+wsID.String()+"/roles/"+systemRoleID.String(), nil)
	req.SetPathValue("ws_id", wsID.String())
	req.SetPathValue("role_id", systemRoleID.String())
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), actorID))
	rec = httptest.NewRecorder()
	h.DeleteRole(rec, req)
	if rec.Code != http.StatusNotFound {
		t.Fatalf("expected 404 for system role delete, got %d", rec.Code)
	}

	// 3. Set system role permissions -> 404 Not Found
	req = httptest.NewRequest(http.MethodPut, "/api/v1/workspaces/"+wsID.String()+"/roles/"+systemRoleID.String()+"/permissions", bytes.NewBufferString(`{"permissions":["members:read"]}`))
	req.SetPathValue("ws_id", wsID.String())
	req.SetPathValue("role_id", systemRoleID.String())
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), actorID))
	rec = httptest.NewRecorder()
	h.SetRolePermissions(rec, req)
	if rec.Code != http.StatusNotFound {
		t.Fatalf("expected 404 for system role permissions update, got %d", rec.Code)
	}
}

type dummyStorageTester struct{ err error }

func (d dummyStorageTester) Test(_ context.Context, _ StorageConfigRequest) error { return d.err }

func TestWorkspaceRBAC_StorageCredentialEncryptionMasking(t *testing.T) {
	wsID, actorID := uuid.New(), uuid.New()
	cryptoSvc := newTestCrypto()
	repo := &mockRBACRepo{}
	tester := dummyStorageTester{err: nil}
	h := WorkspaceRBACHandler{
		Repository:    repo,
		Crypto:        cryptoSvc,
		StorageTester: tester,
	}

	rawAccess := "AKIAIOSFODNN7EXAMPLE"
	rawSecret := "wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY"

	// 1. PUT Storage settings
	reqBody, _ := json.Marshal(StorageConfigRequest{
		EndpointURL:      "https://s3.ap-southeast-1.amazonaws.com",
		Region:           "ap-southeast-1",
		BucketName:       "test-bucket",
		AccessKey:        rawAccess,
		SecretKey:        rawSecret,
		UsePathStyle:     true,
		PresignedTTLSecs: 900,
		MaxFileSize:      10485760,
	})
	req := httptest.NewRequest(http.MethodPut, "/api/v1/workspaces/"+wsID.String()+"/settings/storage", bytes.NewBuffer(reqBody))
	req.SetPathValue("ws_id", wsID.String())
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), actorID))
	rec := httptest.NewRecorder()
	h.PutStorage(rec, req)

	if rec.Code != http.StatusOK {
		t.Fatalf("expected 200, got %d: %s", rec.Code, rec.Body.String())
	}

	var resp StorageConfigResponse
	if err := json.Unmarshal(rec.Body.Bytes(), &resp); err != nil {
		t.Fatalf("failed decoding put storage response: %v", err)
	}
	if resp.AccessKeyMasked != "***" {
		t.Errorf("expected access_key_masked = '***', got %q", resp.AccessKeyMasked)
	}

	// Verify DB stored encrypted values, NOT plaintext
	if repo.storageConfig == nil {
		t.Fatal("storage config was not saved in repo")
	}
	if repo.storageConfig.AccessKeyEnc == rawAccess || repo.storageConfig.SecretKeyEnc == rawSecret {
		t.Fatal("credentials stored as plaintext in DB!")
	}
	decryptedAccess, err := cryptoSvc.Decrypt(repo.storageConfig.AccessKeyEnc)
	if err != nil || decryptedAccess != rawAccess {
		t.Fatalf("decryption failed or mismatched: %v, got %q", err, decryptedAccess)
	}
	decryptedSecret, err := cryptoSvc.Decrypt(repo.storageConfig.SecretKeyEnc)
	if err != nil || decryptedSecret != rawSecret {
		t.Fatalf("decryption failed or mismatched: %v, got %q", err, decryptedSecret)
	}

	// 2. GET Storage settings -> returns masked
	req = httptest.NewRequest(http.MethodGet, "/api/v1/workspaces/"+wsID.String()+"/settings/storage", nil)
	req.SetPathValue("ws_id", wsID.String())
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), actorID))
	rec = httptest.NewRecorder()
	h.GetStorage(rec, req)

	if rec.Code != http.StatusOK {
		t.Fatalf("expected 200, got %d", rec.Code)
	}
	var getResp StorageConfigResponse
	if err := json.Unmarshal(rec.Body.Bytes(), &getResp); err != nil {
		t.Fatalf("failed decoding get storage response: %v", err)
	}
	if getResp.AccessKeyMasked != "***" {
		t.Errorf("expected masked access key, got %q", getResp.AccessKeyMasked)
	}

	// 3. POST Storage test -> 200 true
	req = httptest.NewRequest(http.MethodPost, "/api/v1/workspaces/"+wsID.String()+"/settings/storage/test", bytes.NewBuffer(reqBody))
	req.SetPathValue("ws_id", wsID.String())
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), actorID))
	rec = httptest.NewRecorder()
	h.TestStorage(rec, req)

	if rec.Code != http.StatusOK {
		t.Fatalf("expected 200, got %d", rec.Code)
	}
	var testResult bool
	if err := json.Unmarshal(rec.Body.Bytes(), &testResult); err != nil || !testResult {
		t.Fatalf("expected true, got %v (err: %v)", testResult, err)
	}

	// Test storage test failure
	h.StorageTester = dummyStorageTester{err: errors.New("s3 connection timeout")}
	req = httptest.NewRequest(http.MethodPost, "/api/v1/workspaces/"+wsID.String()+"/settings/storage/test", bytes.NewBuffer(reqBody))
	req.SetPathValue("ws_id", wsID.String())
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), actorID))
	rec = httptest.NewRecorder()
	h.TestStorage(rec, req)
	if rec.Code != http.StatusInternalServerError {
		t.Fatalf("expected 500 on test failure, got %d", rec.Code)
	}

	// 4. DELETE Storage settings -> 200
	req = httptest.NewRequest(http.MethodDelete, "/api/v1/workspaces/"+wsID.String()+"/settings/storage", nil)
	req.SetPathValue("ws_id", wsID.String())
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), actorID))
	rec = httptest.NewRecorder()
	h.DeleteStorage(rec, req)

	if rec.Code != http.StatusOK {
		t.Fatalf("expected 200, got %d", rec.Code)
	}
	if repo.storageConfig != nil {
		t.Errorf("storage config was not deleted")
	}
}
