package api

import (
	"bytes"
	"context"
	"net/http"
	"net/http/httptest"
	"testing"

	"github.com/google/uuid"
	httpinfra "github.com/vkaylee/leeattend/apiserver/internal/infra/http"
)

type mockAuthorizer struct {
	admin bool
	perms map[string]bool
}

func (m mockAuthorizer) IsSystemAdmin(_ context.Context, _ uuid.UUID) bool { return m.admin }
func (m mockAuthorizer) HasPermission(_ context.Context, _, _ uuid.UUID, perm string) bool {
	return m.perms[perm]
}

type mockWorkspaceRepo struct{}

func (m mockWorkspaceRepo) CreateWorkspace(_ context.Context, req CreateWorkspaceRequest, _ uuid.UUID) (WorkspaceRecord, error) {
	return WorkspaceRecord{Name: req.Name}, nil
}
func (m mockWorkspaceRepo) UpdateWorkspace(_ context.Context, _ uuid.UUID, _ WorkspacePatch) (WorkspaceRecord, error) {
	return WorkspaceRecord{}, nil
}
func (m mockWorkspaceRepo) CreateRole(_ context.Context, _ uuid.UUID, _ CreateRoleRequest) error {
	return nil
}
func (m mockWorkspaceRepo) UpdateFeatureFlag(_ context.Context, _ uuid.UUID, id uuid.UUID, p FeatureFlagPatch) (FeatureFlagRecord, error) {
	return FeatureFlagRecord{ID: id, Enabled: *p.Enabled}, nil
}
func (m mockWorkspaceRepo) GetFeatureFlag(_ context.Context, _ uuid.UUID, _ string) (FeatureFlagRecord, error) {
	return FeatureFlagRecord{}, nil
}
func (m mockWorkspaceRepo) PutConfig(_ context.Context, _ uuid.UUID, _ ConfigEntryRequest, _ uuid.UUID) error {
	return nil
}

func TestWorkspaceHandler_CreateForbiddenForRegularUser(t *testing.T) {
	h := WorkspaceHandler{Repository: mockWorkspaceRepo{}, Authorizer: mockAuthorizer{admin: false}}
	req := httptest.NewRequest(http.MethodPost, "/api/v1/workspaces", bytes.NewBufferString(`{"name":"Acme","database_url":"postgres://..."}`))
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), uuid.New()))
	rec := httptest.NewRecorder()

	h.Create(rec, req)
	if rec.Code != http.StatusForbidden {
		t.Fatalf("expected 403, got %d", rec.Code)
	}
}

func TestWorkspaceHandler_PatchEmptyPayloadFailsValidation(t *testing.T) {
	wsID, userID := uuid.New(), uuid.New()
	h := WorkspaceHandler{Repository: mockWorkspaceRepo{}, Authorizer: mockAuthorizer{perms: map[string]bool{"workspace:settings:write": true}}}
	req := httptest.NewRequest(http.MethodPatch, "/api/v1/workspaces/"+wsID.String(), bytes.NewBufferString(`{}`))
	req.SetPathValue("ws_id", wsID.String())
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), userID))
	rec := httptest.NewRecorder()

	h.Patch(rec, req)
	if rec.Code != http.StatusBadRequest {
		t.Fatalf("expected 400 for empty patch, got %d", rec.Code)
	}
}
