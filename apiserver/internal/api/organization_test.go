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

func (m mockAuthorizer) IsMember(_ context.Context, _ uuid.UUID) (bool, error) {
	return true, nil
}

func (m mockAuthorizer) HasPermission(_ context.Context, _ uuid.UUID, perm string) bool {
	return m.perms[perm]
}

func (m mockAuthorizer) EffectivePermissions(_ context.Context, _ uuid.UUID) ([]string, []string, error) {
	return nil, nil, nil
}

func (m mockAuthorizer) GetManagedDepartmentIDs(_ context.Context, _ uuid.UUID) ([]uuid.UUID, error) {
	return nil, nil
}

func (m mockAuthorizer) IsGlobalManager(_ context.Context, _ uuid.UUID) bool {
	return false
}

func (m mockAuthorizer) HasScopedPermission(_ context.Context, _ uuid.UUID, perm string, _ *uuid.UUID) bool {
	return m.perms[perm]
}

func (m mockAuthorizer) GetUsersWithPermission(_ context.Context, _ string) ([]uuid.UUID, error) {
	return nil, nil
}

type mockOrganizationRepo struct{}

func (m mockOrganizationRepo) GetOrganization(_ context.Context) (OrganizationRecord, error) {
	return OrganizationRecord{Name: "Mock Org"}, nil
}

func (m mockOrganizationRepo) UpdateOrganization(_ context.Context, _ OrganizationPatch) (OrganizationRecord, error) {
	return OrganizationRecord{Name: "Updated Org"}, nil
}

func (m mockOrganizationRepo) CreateRole(_ context.Context, _ CreateRoleRequest) error {
	return nil
}

func (m mockOrganizationRepo) ListFeatureFlags(_ context.Context) ([]FeatureFlagRecord, error) {
	return nil, nil
}

func (m mockOrganizationRepo) CreateFeatureFlag(_ context.Context, req FeatureFlagRequest) (FeatureFlagRecord, error) {
	return FeatureFlagRecord{FlagKey: req.FlagKey, Enabled: req.Enabled}, nil
}

func (m mockOrganizationRepo) UpdateFeatureFlag(_ context.Context, id uuid.UUID, p FeatureFlagPatch) (FeatureFlagRecord, error) {
	record := FeatureFlagRecord{ID: id}
	if p.Enabled != nil {
		record.Enabled = *p.Enabled
	}
	return record, nil
}

func (m mockOrganizationRepo) GetFeatureFlag(_ context.Context, key string) (FeatureFlagRecord, error) {
	return FeatureFlagRecord{FlagKey: key}, nil
}

func (m mockOrganizationRepo) GetFeatureFlagByID(_ context.Context, id uuid.UUID) (FeatureFlagRecord, error) {
	return FeatureFlagRecord{ID: id}, nil
}

func (m mockOrganizationRepo) DeleteFeatureFlag(_ context.Context, _ uuid.UUID) (bool, error) {
	return true, nil
}

func (m mockOrganizationRepo) UpsertFeatureFlag(_ context.Context, req FeatureFlagRequest) (FeatureFlagRecord, error) {
	return FeatureFlagRecord{FlagKey: req.FlagKey, Enabled: req.Enabled}, nil
}

func (m mockOrganizationRepo) PutConfig(_ context.Context, _ ConfigEntryRequest, _ uuid.UUID) error {
	return nil
}

func (m mockOrganizationRepo) ListConfigs(_ context.Context) ([]OrganizationConfigEntry, error) {
	return []OrganizationConfigEntry{{ID: uuid.New(), ConfigKey: "max_overtime_hours"}}, nil
}

func (m mockOrganizationRepo) GetConfig(_ context.Context, key string) (ResolvedConfig, error) {
	text := "stored"
	return ResolvedConfig{Key: key, Value: ConfigValue{Text: &text}, ScopeType: "organization"}, nil
}

func (m mockOrganizationRepo) DeleteConfig(_ context.Context, _ uuid.UUID) error {
	return nil
}

func TestOrganizationIDFromContextIgnoresPathValue(t *testing.T) {
	resolvedID := uuid.New()
	pathID := uuid.New()
	req := httptest.NewRequest(http.MethodGet, "/api/v1/workspaces/"+pathID.String(), nil)
	req.SetPathValue("ws_id", pathID.String())
	req = req.WithContext(httpinfra.WithOrganizationIDForTest(req.Context(), resolvedID))

	got, ok := organizationIDFromContext(req)
	if !ok || got != resolvedID {
		t.Fatalf("expected resolved organization %s, got %s (ok=%v)", resolvedID, got, ok)
	}
}

func TestOrganizationIDFromContextRequiresTrustedContext(t *testing.T) {
	pathID := uuid.New()
	req := httptest.NewRequest(http.MethodGet, "/api/v1/workspaces/"+pathID.String(), nil)
	req.SetPathValue("ws_id", pathID.String())

	if got, ok := organizationIDFromContext(req); ok || got != uuid.Nil {
		t.Fatalf("expected missing organization context, got %s (ok=%v)", got, ok)
	}
}

func TestOrganizationHandler_PatchEmptyPayloadFailsValidation(t *testing.T) {
	userID := uuid.New()
	h := OrganizationHandler{
		Repository: mockOrganizationRepo{},
		Authorizer: mockAuthorizer{perms: map[string]bool{"organization:settings:write": true}},
	}
	req := httptest.NewRequest(http.MethodPatch, "/api/v1", bytes.NewBufferString(`{}`))
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), userID))
	rec := httptest.NewRecorder()

	h.Patch(rec, req)
	if rec.Code != http.StatusBadRequest {
		t.Fatalf("expected 400 for empty patch, got %d", rec.Code)
	}
}

func TestOrganizationHandler_PatchRequiresPermission(t *testing.T) {
	userID := uuid.New()
	h := OrganizationHandler{
		Repository: mockOrganizationRepo{},
		Authorizer: mockAuthorizer{perms: map[string]bool{}},
	}
	req := httptest.NewRequest(http.MethodPatch, "/api/v1", bytes.NewBufferString(`{"name":"New Name"}`))
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), userID))
	rec := httptest.NewRecorder()

	h.Patch(rec, req)
	if rec.Code != http.StatusForbidden {
		t.Fatalf("expected 403 for patch without permission, got %d", rec.Code)
	}
}

func TestOrganizationHandler_FeatureFlagsCRUD(t *testing.T) {
	userID, flagID := uuid.New(), uuid.New()
	h := OrganizationHandler{
		Repository: mockOrganizationRepo{},
		Authorizer: mockAuthorizer{perms: map[string]bool{
			"feature_flags:read":   true,
			"feature_flags:manage": true,
		}},
	}

	// 1. List
	req := httptest.NewRequest(http.MethodGet, "/api/v1/feature-flags", nil)
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), userID))
	rec := httptest.NewRecorder()
	h.ListFeatureFlags(rec, req)
	if rec.Code != http.StatusOK {
		t.Fatalf("expected 200 for list, got %d", rec.Code)
	}

	// 2. Create
	req = httptest.NewRequest(http.MethodPost, "/api/v1/feature-flags", bytes.NewBufferString(`{"flag_key":"dark_mode","enabled":true}`))
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), userID))
	rec = httptest.NewRecorder()
	h.CreateFeatureFlag(rec, req)
	if rec.Code != http.StatusCreated {
		t.Fatalf("expected 201 for create, got %d", rec.Code)
	}

	// 3. Create invalid key fails
	req = httptest.NewRequest(http.MethodPost, "/api/v1/feature-flags", bytes.NewBufferString(`{"flag_key":"","enabled":true}`))
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), userID))
	rec = httptest.NewRecorder()
	h.CreateFeatureFlag(rec, req)
	if rec.Code != http.StatusBadRequest {
		t.Fatalf("expected 400 for empty key, got %d", rec.Code)
	}

	// 4. Get by ID
	req = httptest.NewRequest(http.MethodGet, "/api/v1/feature-flags/"+flagID.String(), nil)
	req.SetPathValue("id", flagID.String())
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), userID))
	rec = httptest.NewRecorder()
	h.GetFeatureFlagByID(rec, req)
	if rec.Code != http.StatusOK {
		t.Fatalf("expected 200 for get by id, got %d", rec.Code)
	}

	// 5. Update
	req = httptest.NewRequest(http.MethodPatch, "/api/v1/feature-flags/"+flagID.String(), bytes.NewBufferString(`{"enabled":false}`))
	req.SetPathValue("id", flagID.String())
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), userID))
	rec = httptest.NewRecorder()
	h.UpdateFeatureFlag(rec, req)
	if rec.Code != http.StatusOK {
		t.Fatalf("expected 200 for update, got %d", rec.Code)
	}

	// 6. Upsert
	req = httptest.NewRequest(http.MethodPut, "/api/v1/feature-flags", bytes.NewBufferString(`{"flag_key":"beta_payroll","enabled":true}`))
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), userID))
	rec = httptest.NewRecorder()
	h.UpsertFeatureFlag(rec, req)
	if rec.Code != http.StatusOK {
		t.Fatalf("expected 200 for upsert, got %d", rec.Code)
	}

	// 7. Delete
	req = httptest.NewRequest(http.MethodDelete, "/api/v1/feature-flags/"+flagID.String(), nil)
	req.SetPathValue("id", flagID.String())
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), userID))
	rec = httptest.NewRecorder()
	h.DeleteFeatureFlag(rec, req)
	if rec.Code != http.StatusNoContent {
		t.Fatalf("expected 204 for delete, got %d", rec.Code)
	}

	// 8. Check
	req = httptest.NewRequest(http.MethodGet, "/api/v1/feature-flags/check/dark_mode", nil)
	req.SetPathValue("key", "dark_mode")
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), userID))
	rec = httptest.NewRecorder()
	h.CheckFeatureFlag(rec, req)
	if rec.Code != http.StatusOK {
		t.Fatalf("expected 200 for check, got %d", rec.Code)
	}
}

func TestOrganizationHandler_Get(t *testing.T) {
	userID := uuid.New()
	h := OrganizationHandler{Repository: mockOrganizationRepo{}, Authorizer: mockAuthorizer{}}

	req := httptest.NewRequest(http.MethodGet, "/api/v1", nil)
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), userID))
	rec := httptest.NewRecorder()
	h.Get(rec, req)
	if rec.Code != http.StatusOK {
		t.Fatalf("expected 200 for get, got %d", rec.Code)
	}
	var got OrganizationRecord
	if err := json.Unmarshal(rec.Body.Bytes(), &got); err != nil || got.Name != "Mock Org" {
		t.Fatalf("expected organization Mock Org, got %s (err=%v)", rec.Body.String(), err)
	}
}

func TestOrganizationHandler_GetRequiresAuth(t *testing.T) {
	h := OrganizationHandler{Repository: mockOrganizationRepo{}, Authorizer: mockAuthorizer{}}

	req := httptest.NewRequest(http.MethodGet, "/api/v1", nil)
	rec := httptest.NewRecorder()
	h.Get(rec, req)
	if rec.Code != http.StatusUnauthorized {
		t.Fatalf("expected 401 for unauthenticated get, got %d", rec.Code)
	}
}

func TestOrganizationHandler_ConfigEndpoints(t *testing.T) {
	userID, entryID := uuid.New(), uuid.New()
	h := OrganizationHandler{
		Repository: mockOrganizationRepo{},
		Authorizer: mockAuthorizer{perms: map[string]bool{
			"organization:settings:read":  true,
			"organization:settings:write": true,
		}},
	}

	req := httptest.NewRequest(http.MethodGet, "/api/v1/configs", nil)
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), userID))
	rec := httptest.NewRecorder()
	h.ListConfigs(rec, req)
	if rec.Code != http.StatusOK {
		t.Fatalf("expected 200 for list configs, got %d", rec.Code)
	}
	var entries []OrganizationConfigEntry
	if err := json.Unmarshal(rec.Body.Bytes(), &entries); err != nil || len(entries) != 1 {
		t.Fatalf("expected one config entry, got %s (err=%v)", rec.Body.String(), err)
	}

	req = httptest.NewRequest(http.MethodGet, "/api/v1/configs/max_overtime_hours", nil)
	req.SetPathValue("key", "max_overtime_hours")
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), userID))
	rec = httptest.NewRecorder()
	h.GetConfig(rec, req)
	if rec.Code != http.StatusOK {
		t.Fatalf("expected 200 for get config, got %d", rec.Code)
	}
	var resolved ResolvedConfig
	if err := json.Unmarshal(rec.Body.Bytes(), &resolved); err != nil || resolved.Key != "max_overtime_hours" || resolved.Value.Text == nil || *resolved.Value.Text != "stored" {
		t.Fatalf("expected stored config response, got %s (err=%v)", rec.Body.String(), err)
	}

	req = httptest.NewRequest(http.MethodDelete, "/api/v1/configs/"+entryID.String(), nil)
	req.SetPathValue("key", entryID.String())
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), userID))
	rec = httptest.NewRecorder()
	h.DeleteConfig(rec, req)
	if rec.Code != http.StatusOK {
		t.Fatalf("expected 200 for delete config, got %d", rec.Code)
	}
}

func TestOrganizationHandler_ConfigRequiresPermission(t *testing.T) {
	userID := uuid.New()
	h := OrganizationHandler{Repository: mockOrganizationRepo{}, Authorizer: mockAuthorizer{}}
	req := httptest.NewRequest(http.MethodGet, "/api/v1/configs", nil)
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), userID))
	rec := httptest.NewRecorder()
	h.ListConfigs(rec, req)
	if rec.Code != http.StatusForbidden {
		t.Fatalf("expected 403, got %d", rec.Code)
	}
}
