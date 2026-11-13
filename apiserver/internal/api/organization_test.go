package api

import (
	"bytes"
	"context"
	"encoding/json"
	"net/http"
	"net/http/httptest"
	"testing"

	"github.com/google/uuid"
	"github.com/vkaylee/leeattend/apiserver/internal/audit"
	httpinfra "github.com/vkaylee/leeattend/apiserver/internal/infra/http"
)

type mockAuthorizer struct {
	perms         map[string]bool
	globalManager bool
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
	return m.globalManager
}

func (m mockAuthorizer) HasScopedPermission(_ context.Context, _ uuid.UUID, perm string, _ *uuid.UUID) bool {
	return m.perms[perm]
}

func (m mockAuthorizer) GetUsersWithPermission(_ context.Context, _ string) ([]uuid.UUID, error) {
	return nil, nil
}

type organizationAuditRecorder struct {
	events []audit.Event
}

func (r *organizationAuditRecorder) Emit(_ context.Context, event audit.Event) error {
	r.events = append(r.events, event)
	return nil
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

func (m mockOrganizationRepo) GetConfig(_ context.Context, key string, _ ConfigReadRequest) (ResolvedConfig, error) {
	text := "stored"
	return ResolvedConfig{Key: key, Value: ConfigValue{Text: &text}, ScopeType: "organization"}, nil
}

func (m mockOrganizationRepo) DeleteConfig(_ context.Context, _ uuid.UUID) error {
	return nil
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
	req = httptest.NewRequest(http.MethodPost, "/api/v1/feature-flags", bytes.NewBufferString(`{"flag_key":"ff_dark_mode","enabled":true}`))
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), userID))
	rec = httptest.NewRecorder()
	h.CreateFeatureFlag(rec, req)
	if rec.Code != http.StatusCreated {
		t.Fatalf("expected 201 for create, got %d", rec.Code)
	}

	// 3. Create invalid key fails
	req = httptest.NewRequest(http.MethodPost, "/api/v1/feature-flags", bytes.NewBufferString(`{"flag_key":"dark_mode","enabled":true}`))
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
	req = httptest.NewRequest(http.MethodPut, "/api/v1/feature-flags", bytes.NewBufferString(`{"flag_key":"ff_beta_payroll","enabled":true}`))
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

func TestOrganizationHandler_FeatureFlagValidation(t *testing.T) {
	userID := uuid.New()
	h := OrganizationHandler{
		Repository: mockOrganizationRepo{},
		Authorizer: mockAuthorizer{perms: map[string]bool{"feature_flags:manage": true}},
	}
	cases := []string{"dark_mode", "ff_dark-mode", "ff_dark_mode_", "ff__dark_mode"}
	for _, key := range cases {
		t.Run(key, func(t *testing.T) {
			req := httptest.NewRequest(http.MethodPost, "/api/v1/feature-flags", bytes.NewBufferString(`{"flag_key":"`+key+`","enabled":true}`))
			req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), userID))
			rec := httptest.NewRecorder()
			h.CreateFeatureFlag(rec, req)
			if rec.Code != http.StatusBadRequest {
				t.Fatalf("expected 400 for invalid key %q, got %d", key, rec.Code)
			}
		})
	}
}

func TestOrganizationHandler_FeatureFlagAcceptsValidKeyAndMetadata(t *testing.T) {
	userID := uuid.New()
	h := OrganizationHandler{
		Repository: mockOrganizationRepo{},
		Authorizer: mockAuthorizer{perms: map[string]bool{"feature_flags:manage": true}},
	}
	req := httptest.NewRequest(http.MethodPost, "/api/v1/feature-flags", bytes.NewBufferString(`{"flag_key":"ff_dark_mode","enabled":true,"metadata":{"owner":"platform"}}`))
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), userID))
	rec := httptest.NewRecorder()
	h.CreateFeatureFlag(rec, req)
	if rec.Code != http.StatusCreated {
		t.Fatalf("expected 201 for valid flag, got %d", rec.Code)
	}
}

func TestOrganizationHandler_FeatureFlagMetadataValidation(t *testing.T) {
	userID := uuid.New()
	h := OrganizationHandler{
		Repository: mockOrganizationRepo{},
		Authorizer: mockAuthorizer{perms: map[string]bool{"feature_flags:manage": true}},
	}
	for _, payload := range []string{
		`{"flag_key":"ff_dark_mode","metadata":[]}`,
		`{"flag_key":"ff_dark_mode","metadata":"invalid"}`,
	} {
		req := httptest.NewRequest(http.MethodPost, "/api/v1/feature-flags", bytes.NewBufferString(payload))
		req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), userID))
		rec := httptest.NewRecorder()
		h.CreateFeatureFlag(rec, req)
		if rec.Code != http.StatusBadRequest {
			t.Fatalf("expected 400 for invalid metadata, got %d", rec.Code)
		}
	}
}

func TestOrganizationHandler_EffectiveFeatureFlagsRequiresAuthentication(t *testing.T) {
	h := OrganizationHandler{Repository: mockOrganizationRepo{}, Authorizer: mockAuthorizer{}}
	req := httptest.NewRequest(http.MethodGet, "/api/v1/feature-flags/effective", nil)
	rec := httptest.NewRecorder()
	h.ListEffectiveFeatureFlags(rec, req)
	if rec.Code != http.StatusUnauthorized {
		t.Fatalf("expected 401 without authentication, got %d", rec.Code)
	}
}

func TestOrganizationHandler_EffectiveFeatureFlagsReturnsRuntimeState(t *testing.T) {
	userID := uuid.New()
	h := OrganizationHandler{Repository: mockOrganizationRepo{}, Authorizer: mockAuthorizer{}}
	req := httptest.NewRequest(http.MethodGet, "/api/v1/feature-flags/effective", nil)
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), userID))
	rec := httptest.NewRecorder()
	h.ListEffectiveFeatureFlags(rec, req)
	if rec.Code != http.StatusOK {
		t.Fatalf("expected 200 for effective flags, got %d", rec.Code)
	}
	var flags []FeatureFlagCheck
	if err := json.Unmarshal(rec.Body.Bytes(), &flags); err != nil {
		t.Fatalf("decode effective flags: %v", err)
	}
	if flags == nil {
		t.Fatal("expected non-nil effective flags list")
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

func TestOrganizationHandler_PutConfigRejectsLegacyScope(t *testing.T) {
	userID := uuid.New()
	h := OrganizationHandler{
		Repository: mockOrganizationRepo{},
		Authorizer: mockAuthorizer{perms: map[string]bool{"organization:settings:write": true}},
	}
	req := httptest.NewRequest(http.MethodPut, "/api/v1/configs", bytes.NewBufferString(`{"scope_type":"workspace","config_key":"theme_mode","effective_from":"2026-01-01","value":{"Text":"dark"}}`))
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), userID))
	rec := httptest.NewRecorder()
	h.PutConfig(rec, req)
	if rec.Code != http.StatusBadRequest {
		t.Fatalf("expected 400 for legacy workspace scope, got %d", rec.Code)
	}
}

func TestOrganizationHandler_PutLifecycleConfigEmitsAudit(t *testing.T) {
	userID, employerID := uuid.New(), uuid.New()
	auditRecorder := &organizationAuditRecorder{}
	h := OrganizationHandler{
		Repository: mockOrganizationRepo{},
		Authorizer: mockAuthorizer{perms: map[string]bool{"organization:settings:write": true}},
		Audit:      auditRecorder,
	}
	body := bytes.NewBufferString(`{"scope_type":"employer","scope_id":"` + employerID.String() + `","config_key":"schedule.lifecycle.enabled","value":{"Bool":true},"effective_from":"2026-11-13"}`)
	req := httptest.NewRequest(http.MethodPut, "/api/v1/configs", body)
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), userID))
	rec := httptest.NewRecorder()
	h.PutConfig(rec, req)
	if rec.Code != http.StatusForbidden {
		t.Fatalf("expected non-global manager to be forbidden, got %d: %s", rec.Code, rec.Body.String())
	}
	if len(auditRecorder.events) != 0 {
		t.Fatalf("forbidden config write must not emit audit, got %+v", auditRecorder.events)
	}
	h.Authorizer = mockAuthorizer{perms: map[string]bool{"organization:settings:write": true}, globalManager: true}
	req = httptest.NewRequest(http.MethodPut, "/api/v1/configs", bytes.NewBufferString(`{"scope_type":"employer","scope_id":"`+employerID.String()+`","config_key":"schedule.lifecycle.enabled","value":{"Bool":true},"effective_from":"2026-11-13"}`))
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), userID))
	rec = httptest.NewRecorder()
	h.PutConfig(rec, req)
	if rec.Code != http.StatusOK {
		t.Fatalf("expected global manager write 200, got %d: %s", rec.Code, rec.Body.String())
	}
	if len(auditRecorder.events) != 1 || auditRecorder.events[0].Name != "schedule_lifecycle.activation.updated" {
		t.Fatalf("expected lifecycle activation audit, got %+v", auditRecorder.events)
	}
}

func TestParseConfigReadRequest(t *testing.T) {
	employerID := uuid.New()
	valid := httptest.NewRequest(http.MethodGet, "/api/v1/configs/schedule.lifecycle.enabled?scope_type=employer&scope_id="+employerID.String()+"&as_of=2026-11-14", nil)
	request, err := parseConfigReadRequest(valid)
	if err != nil || request.ScopeType != "employer" || request.ScopeID == nil || *request.ScopeID != employerID || request.AsOf.Format("2006-01-02") != "2026-11-14" {
		t.Fatalf("unexpected parsed config request: %+v err=%v", request, err)
	}
	for _, target := range []string{
		"/api/v1/configs/key?scope_type=employer",
		"/api/v1/configs/key?scope_type=organization&scope_id=" + employerID.String(),
		"/api/v1/configs/key?scope_type=employer&scope_id=bad",
		"/api/v1/configs/key?scope_type=organization&as_of=bad",
	} {
		if _, err := parseConfigReadRequest(httptest.NewRequest(http.MethodGet, target, nil)); err == nil {
			t.Fatalf("expected invalid config read request for %s", target)
		}
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
