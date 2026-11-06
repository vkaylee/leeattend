package api

import (
	"bytes"
	"context"
	"encoding/json"
	"net/http"
	"net/http/httptest"
	"testing"

	"github.com/google/uuid"
	httpinfra "github.com/vkaylee/leeattend/apiserver/internal/infra/http"
)

type mockAuthorizer struct {
	perms map[string]bool
}

func (m mockAuthorizer) HasPermission(_ context.Context, _, _ uuid.UUID, perm string) bool {
	return m.perms[perm]
}

type mockWorkspaceRepo struct{}

func (m mockWorkspaceRepo) GetWorkspace(_ context.Context, id uuid.UUID) (WorkspaceRecord, error) {
	return WorkspaceRecord{ID: id, Name: "Mock WS"}, nil
}
func (m mockWorkspaceRepo) UpdateWorkspace(_ context.Context, _ uuid.UUID, _ WorkspacePatch) (WorkspaceRecord, error) {
	return WorkspaceRecord{}, nil
}
func (m mockWorkspaceRepo) CreateRole(_ context.Context, _ uuid.UUID, _ CreateRoleRequest) error {
	return nil
}
func (m mockWorkspaceRepo) ListFeatureFlags(_ context.Context, _ uuid.UUID) ([]FeatureFlagRecord, error) {
	return nil, nil
}
func (m mockWorkspaceRepo) CreateFeatureFlag(_ context.Context, wsID uuid.UUID, req FeatureFlagRequest) (FeatureFlagRecord, error) {
	return FeatureFlagRecord{WorkspaceID: wsID, FlagKey: req.FlagKey, Enabled: req.Enabled}, nil
}
func (m mockWorkspaceRepo) UpdateFeatureFlag(_ context.Context, _ uuid.UUID, id uuid.UUID, p FeatureFlagPatch) (FeatureFlagRecord, error) {
	record := FeatureFlagRecord{ID: id}
	if p.Enabled != nil {
		record.Enabled = *p.Enabled
	}
	return record, nil
}
func (m mockWorkspaceRepo) GetFeatureFlag(_ context.Context, _ uuid.UUID, key string) (FeatureFlagRecord, error) {
	return FeatureFlagRecord{FlagKey: key}, nil
}
func (m mockWorkspaceRepo) GetFeatureFlagByID(_ context.Context, _ uuid.UUID, id uuid.UUID) (FeatureFlagRecord, error) {
	return FeatureFlagRecord{ID: id}, nil
}
func (m mockWorkspaceRepo) DeleteFeatureFlag(_ context.Context, _ uuid.UUID, _ uuid.UUID) (bool, error) {
	return true, nil
}
func (m mockWorkspaceRepo) UpsertFeatureFlag(_ context.Context, wsID uuid.UUID, req FeatureFlagRequest) (FeatureFlagRecord, error) {
	return FeatureFlagRecord{WorkspaceID: wsID, FlagKey: req.FlagKey, Enabled: req.Enabled}, nil
}
func (m mockWorkspaceRepo) PutConfig(_ context.Context, _ uuid.UUID, _ ConfigEntryRequest, _ uuid.UUID) error {
	return nil
}
func (m mockWorkspaceRepo) ListConfigs(_ context.Context, wsID uuid.UUID) ([]WorkspaceConfigEntry, error) {
	return []WorkspaceConfigEntry{{ID: uuid.New(), WorkspaceID: wsID, ConfigKey: "max_overtime_hours"}}, nil
}
func (m mockWorkspaceRepo) GetConfig(_ context.Context, _ uuid.UUID, key string) (ResolvedConfig, error) {
	text := "stored"
	return ResolvedConfig{Key: key, Value: ConfigValue{Text: &text}, ScopeType: "workspace"}, nil
}
func (m mockWorkspaceRepo) DeleteConfig(_ context.Context, _ uuid.UUID, _ uuid.UUID) error {
	return nil
}

func TestPathUUID_PrefersResolvedWorkspaceContext(t *testing.T) {
	resolvedID := uuid.New()
	pathID := uuid.New()
	req := httptest.NewRequest(http.MethodGet, "/api/v1/workspaces/"+pathID.String(), nil)
	req.SetPathValue("ws_id", pathID.String())
	req = req.WithContext(httpinfra.WithWorkspaceIDForTest(req.Context(), resolvedID))

	got, ok := pathUUID(req, "ws_id")
	if !ok || got != resolvedID {
		t.Fatalf("expected resolved workspace %s, got %s (ok=%v)", resolvedID, got, ok)
	}
}

func TestPathUUID_FallsBackToPathValue(t *testing.T) {
	pathID := uuid.New()
	req := httptest.NewRequest(http.MethodGet, "/api/v1/workspaces/"+pathID.String(), nil)
	req.SetPathValue("ws_id", pathID.String())

	got, ok := pathUUID(req, "ws_id")
	if !ok || got != pathID {
		t.Fatalf("expected path workspace %s, got %s (ok=%v)", pathID, got, ok)
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

func TestWorkspaceHandler_FeatureFlagsCRUD(t *testing.T) {
	wsID, userID, flagID := uuid.New(), uuid.New(), uuid.New()
	h := WorkspaceHandler{
		Repository: mockWorkspaceRepo{},
		Authorizer: mockAuthorizer{perms: map[string]bool{
			"feature_flags:read":   true,
			"feature_flags:manage": true,
		}},
	}

	// 1. List
	req := httptest.NewRequest(http.MethodGet, "/api/v1/workspaces/"+wsID.String()+"/feature-flags", nil)
	req.SetPathValue("ws_id", wsID.String())
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), userID))
	rec := httptest.NewRecorder()
	h.ListFeatureFlags(rec, req)
	if rec.Code != http.StatusOK {
		t.Fatalf("expected 200 for list, got %d", rec.Code)
	}

	// 2. Create
	req = httptest.NewRequest(http.MethodPost, "/api/v1/workspaces/"+wsID.String()+"/feature-flags", bytes.NewBufferString(`{"flag_key":"dark_mode","enabled":true}`))
	req.SetPathValue("ws_id", wsID.String())
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), userID))
	rec = httptest.NewRecorder()
	h.CreateFeatureFlag(rec, req)
	if rec.Code != http.StatusCreated {
		t.Fatalf("expected 201 for create, got %d", rec.Code)
	}

	// 3. Create invalid key fails
	req = httptest.NewRequest(http.MethodPost, "/api/v1/workspaces/"+wsID.String()+"/feature-flags", bytes.NewBufferString(`{"flag_key":"","enabled":true}`))
	req.SetPathValue("ws_id", wsID.String())
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), userID))
	rec = httptest.NewRecorder()
	h.CreateFeatureFlag(rec, req)
	if rec.Code != http.StatusBadRequest {
		t.Fatalf("expected 400 for empty key, got %d", rec.Code)
	}

	// 4. Get by ID
	req = httptest.NewRequest(http.MethodGet, "/api/v1/workspaces/"+wsID.String()+"/feature-flags/"+flagID.String(), nil)
	req.SetPathValue("ws_id", wsID.String())
	req.SetPathValue("id", flagID.String())
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), userID))
	rec = httptest.NewRecorder()
	h.GetFeatureFlagByID(rec, req)
	if rec.Code != http.StatusOK {
		t.Fatalf("expected 200 for get by id, got %d", rec.Code)
	}

	// 5. Update
	req = httptest.NewRequest(http.MethodPatch, "/api/v1/workspaces/"+wsID.String()+"/feature-flags/"+flagID.String(), bytes.NewBufferString(`{"enabled":false}`))
	req.SetPathValue("ws_id", wsID.String())
	req.SetPathValue("id", flagID.String())
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), userID))
	rec = httptest.NewRecorder()
	h.UpdateFeatureFlag(rec, req)
	if rec.Code != http.StatusOK {
		t.Fatalf("expected 200 for update, got %d", rec.Code)
	}

	// 6. Upsert
	req = httptest.NewRequest(http.MethodPut, "/api/v1/workspaces/"+wsID.String()+"/feature-flags", bytes.NewBufferString(`{"flag_key":"beta_payroll","enabled":true}`))
	req.SetPathValue("ws_id", wsID.String())
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), userID))
	rec = httptest.NewRecorder()
	h.UpsertFeatureFlag(rec, req)
	if rec.Code != http.StatusOK {
		t.Fatalf("expected 200 for upsert, got %d", rec.Code)
	}

	// 7. Delete
	req = httptest.NewRequest(http.MethodDelete, "/api/v1/workspaces/"+wsID.String()+"/feature-flags/"+flagID.String(), nil)
	req.SetPathValue("ws_id", wsID.String())
	req.SetPathValue("id", flagID.String())
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), userID))
	rec = httptest.NewRecorder()
	h.DeleteFeatureFlag(rec, req)
	if rec.Code != http.StatusNoContent {
		t.Fatalf("expected 204 for delete, got %d", rec.Code)
	}

	// 8. Check
	req = httptest.NewRequest(http.MethodGet, "/api/v1/workspaces/"+wsID.String()+"/feature-flags/check/dark_mode", nil)
	req.SetPathValue("ws_id", wsID.String())
	req.SetPathValue("key", "dark_mode")
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), userID))
	rec = httptest.NewRecorder()
	h.CheckFeatureFlag(rec, req)
	if rec.Code != http.StatusOK {
		t.Fatalf("expected 200 for check, got %d", rec.Code)
	}
}

func TestWorkspaceHandler_Get(t *testing.T) {
	wsID, userID := uuid.New(), uuid.New()
	h := WorkspaceHandler{Repository: mockWorkspaceRepo{}, Authorizer: mockAuthorizer{}}

	req := httptest.NewRequest(http.MethodGet, "/api/v1/workspaces/"+wsID.String(), nil)
	req.SetPathValue("ws_id", wsID.String())
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), userID))
	rec := httptest.NewRecorder()
	h.Get(rec, req)
	if rec.Code != http.StatusOK {
		t.Fatalf("expected 200 for get, got %d", rec.Code)
	}
	var got WorkspaceRecord
	if err := json.Unmarshal(rec.Body.Bytes(), &got); err != nil || got.ID != wsID {
		t.Fatalf("expected workspace %s, got %s (err=%v)", wsID, rec.Body.String(), err)
	}
}

func TestWorkspaceHandler_ConfigEndpoints(t *testing.T) {
	wsID, userID, entryID := uuid.New(), uuid.New(), uuid.New()
	h := WorkspaceHandler{
		Repository: mockWorkspaceRepo{},
		Authorizer: mockAuthorizer{perms: map[string]bool{
			"workspace:settings:read":  true,
			"workspace:settings:write": true,
		}},
	}

	req := httptest.NewRequest(http.MethodGet, "/api/v1/workspaces/"+wsID.String()+"/configs", nil)
	req.SetPathValue("ws_id", wsID.String())
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), userID))
	rec := httptest.NewRecorder()
	h.ListConfigs(rec, req)
	if rec.Code != http.StatusOK {
		t.Fatalf("expected 200 for list configs, got %d", rec.Code)
	}
	var entries []WorkspaceConfigEntry
	if err := json.Unmarshal(rec.Body.Bytes(), &entries); err != nil || len(entries) != 1 {
		t.Fatalf("expected one config entry, got %s (err=%v)", rec.Body.String(), err)
	}

	req = httptest.NewRequest(http.MethodGet, "/api/v1/workspaces/"+wsID.String()+"/configs/max_overtime_hours", nil)
	req.SetPathValue("ws_id", wsID.String())
	req.SetPathValue("key", "max_overtime_hours")
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), userID))
	rec = httptest.NewRecorder()
	h.GetConfig(rec, req)
	if rec.Code != http.StatusOK {
		t.Fatalf("expected 200 for get config, got %d", rec.Code)
	}
	var resolved ResolvedConfig
	if err := json.Unmarshal(rec.Body.Bytes(), &resolved); err != nil || resolved.Key != "max_overtime_hours" || resolved.Value.Text == nil || *resolved.Value.Text != "mock" {
		t.Fatalf("expected Rust mock config response, got %s (err=%v)", rec.Body.String(), err)
	}

	req = httptest.NewRequest(http.MethodDelete, "/api/v1/workspaces/"+wsID.String()+"/configs/"+entryID.String(), nil)
	req.SetPathValue("ws_id", wsID.String())
	req.SetPathValue("key", entryID.String())
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), userID))
	rec = httptest.NewRecorder()
	h.DeleteConfig(rec, req)
	if rec.Code != http.StatusOK {
		t.Fatalf("expected 200 for delete config, got %d", rec.Code)
	}
}

func TestWorkspaceHandler_ConfigRequiresPermission(t *testing.T) {
	wsID, userID := uuid.New(), uuid.New()
	h := WorkspaceHandler{Repository: mockWorkspaceRepo{}, Authorizer: mockAuthorizer{}}
	req := httptest.NewRequest(http.MethodGet, "/api/v1/workspaces/"+wsID.String()+"/configs", nil)
	req.SetPathValue("ws_id", wsID.String())
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), userID))
	rec := httptest.NewRecorder()
	h.ListConfigs(rec, req)
	if rec.Code != http.StatusForbidden {
		t.Fatalf("expected 403, got %d", rec.Code)
	}
}
