package api

import (
	"bytes"
	"context"
	"encoding/json"
	"net/http"
	"net/http/httptest"
	"testing"

	"github.com/google/uuid"
	"github.com/vkaylee/leeattend/apiserver/internal/domain/ids"
	httpinfra "github.com/vkaylee/leeattend/apiserver/internal/infra/http"
)

type mockPreferencesRepo struct {
	pref  DashboardPreferences
	err   error
	req   UpdateDashboardPreferences
	users map[uuid.UUID]DashboardPreferences
}

func (m *mockPreferencesRepo) GetDashboardPreferences(ctx context.Context, u ids.UserID) (DashboardPreferences, error) {
	if m.err != nil {
		return DashboardPreferences{}, m.err
	}
	if p, ok := m.users[u.UUID()]; ok {
		return p, nil
	}
	res := m.pref
	res.UserID = u.UUID()
	return res, nil
}
func (m *mockPreferencesRepo) UpdateDashboardPreferences(ctx context.Context, u ids.UserID, r UpdateDashboardPreferences) (DashboardPreferences, error) {
	m.req = r
	if m.err != nil {
		return DashboardPreferences{}, m.err
	}
	res := m.pref
	res.UserID = u.UUID()
	if r.ActiveView != nil {
		res.ActiveView = *r.ActiveView
	}
	if r.VisibleWidgets != nil {
		res.VisibleWidgets = *r.VisibleWidgets
	}
	if r.WidgetOrder != nil {
		res.WidgetOrder = *r.WidgetOrder
	}
	if r.CollapsedSections != nil {
		res.CollapsedSections = *r.CollapsedSections
	}
	m.users = map[uuid.UUID]DashboardPreferences{u.UUID(): res}
	return res, nil
}

type prefAuthz struct {
	allowed bool
}

func (a prefAuthz) IsMember(context.Context, uuid.UUID) (bool, error) {
	return a.allowed, nil
}
func (a prefAuthz) HasPermission(_ context.Context, _ uuid.UUID, p string) bool {
	return a.allowed && p == "dashboard:preferences"
}
func (prefAuthz) EffectivePermissions(context.Context, uuid.UUID) ([]string, []string, error) {
	return nil, nil, nil
}
func (prefAuthz) GetManagedDepartmentIDs(context.Context, uuid.UUID) ([]uuid.UUID, error) {
	return nil, nil
}
func (prefAuthz) IsGlobalManager(context.Context, uuid.UUID) bool {
	return false
}
func (a prefAuthz) HasScopedPermission(ctx context.Context, u uuid.UUID, p string, _ *uuid.UUID) bool {
	return a.HasPermission(ctx, u, p)
}
func (prefAuthz) GetUsersWithPermission(context.Context, string) ([]uuid.UUID, error) {
	return nil, nil
}

func TestDashboardPreferencesHandler_Get(t *testing.T) {
	u := uuid.New()
	repo := &mockPreferencesRepo{
		pref: DashboardPreferences{
			ID:                uuid.New(),
			ActiveView:        "overview",
			VisibleWidgets:    json.RawMessage(`{"kpi_cards":true}`),
			WidgetOrder:       json.RawMessage(`["kpi_cards"]`),
			CollapsedSections: json.RawMessage(`{"summary":false}`),
			WindowLayout:      json.RawMessage(`{"app-employees":{"x":100,"y":100}}`),
		},
	}
	h := DashboardPreferencesHandler{Repository: repo, Authorizer: prefAuthz{allowed: true}}
	req := httptest.NewRequest(http.MethodGet, "/api/v1/dashboard/preferences", nil)
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), u))
	rec := httptest.NewRecorder()
	h.Get(rec, req)
	if rec.Code != http.StatusOK {
		t.Fatalf("expected 200, got %d: %s", rec.Code, rec.Body.String())
	}
	var res DashboardPreferences
	if err := json.Unmarshal(rec.Body.Bytes(), &res); err != nil {
		t.Fatalf("decode: %v", err)
	}
	if res.ActiveView != "overview" || res.UserID != u {
		t.Fatalf("unexpected res: %+v", res)
	}
	var fields map[string]json.RawMessage
	if err := json.Unmarshal(rec.Body.Bytes(), &fields); err != nil {
		t.Fatalf("decode fields: %v", err)
	}
	if _, ok := fields["workspace_id"]; ok {
		t.Fatal("workspace_id must not be exposed")
	}
}

func TestDashboardPreferencesHandler_Put(t *testing.T) {
	u := uuid.New()
	repo := &mockPreferencesRepo{
		pref: DashboardPreferences{
			ID:         uuid.New(),
			ActiveView: "overview",
		},
	}
	h := DashboardPreferencesHandler{
		Repository: repo,
		Authorizer: prefAuthz{allowed: true},
	}

	body := `{"active_view":"attendance","visible_widgets":{"kpi_cards":false},"widget_order":["kpi_cards"],"collapsed_sections":{"charts":true},"window_layout":{"app-attendance":{"x":200,"y":150}}}`
	req := httptest.NewRequest(http.MethodPut, "/api/v1/dashboard/preferences", bytes.NewBufferString(body))
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), u))

	rec := httptest.NewRecorder()
	h.Put(rec, req)
	if rec.Code != http.StatusOK {
		t.Fatalf("expected 200, got %d: %s", rec.Code, rec.Body.String())
	}
	if repo.req.ActiveView == nil || *repo.req.ActiveView != "attendance" {
		t.Fatalf("expected updated active_view: %+v", repo.req)
	}
	if repo.req.WindowLayout == nil || string(*repo.req.WindowLayout) != `{"app-attendance":{"x":200,"y":150}}` {
		t.Fatalf("expected updated window_layout: %+v", repo.req)
	}
}

func TestDashboardPreferencesHandler_Unauthenticated(t *testing.T) {
	h := DashboardPreferencesHandler{
		Repository: &mockPreferencesRepo{},
		Authorizer: prefAuthz{allowed: true},
	}
	req := httptest.NewRequest(http.MethodGet, "/api/v1/dashboard/preferences", nil)
	rec := httptest.NewRecorder()
	h.Get(rec, req)
	if rec.Code != http.StatusForbidden {
		t.Fatalf("expected 403, got %d", rec.Code)
	}
}

func TestDashboardPreferencesHandler_Forbidden(t *testing.T) {
	u := uuid.New()
	h := DashboardPreferencesHandler{
		Repository: &mockPreferencesRepo{},
		Authorizer: prefAuthz{allowed: false},
	}
	req := httptest.NewRequest(http.MethodGet, "/api/v1/dashboard/preferences", nil)
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), u))

	rec := httptest.NewRecorder()
	h.Get(rec, req)
	if rec.Code != http.StatusForbidden {
		t.Fatalf("expected 403, got %d", rec.Code)
	}
}

func TestDashboardPreferencesHandler_Validation(t *testing.T) {
	u := uuid.New()
	h := DashboardPreferencesHandler{
		Repository: &mockPreferencesRepo{},
		Authorizer: prefAuthz{allowed: true},
	}
	body := `{"active_view":"invalid_view"}`
	req := httptest.NewRequest(http.MethodPut, "/api/v1/dashboard/preferences", bytes.NewBufferString(body))
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), u))

	rec := httptest.NewRecorder()
	h.Put(rec, req)
	if rec.Code != http.StatusBadRequest {
		t.Fatalf("expected 400 for invalid active view, got %d", rec.Code)
	}
}

func TestDashboardPreferencesHandler_UserIsolation(t *testing.T) {
	u1 := uuid.New()
	u2 := uuid.New()
	repo := &mockPreferencesRepo{
		users: map[uuid.UUID]DashboardPreferences{
			u1: {UserID: u1, ActiveView: "trends"},
			u2: {UserID: u2, ActiveView: "custom"},
		},
	}
	h := DashboardPreferencesHandler{
		Repository: repo,
		Authorizer: prefAuthz{allowed: true},
	}

	req1 := httptest.NewRequest(http.MethodGet, "/api/v1/dashboard/preferences", nil)
	req1 = req1.WithContext(httpinfra.WithUserIDForTest(req1.Context(), u1))
	rec1 := httptest.NewRecorder()
	h.Get(rec1, req1)
	if rec1.Code != http.StatusOK {
		t.Fatalf("expected 200, got %d: %s", rec1.Code, rec1.Body.String())
	}
	var res1 DashboardPreferences
	if err := json.Unmarshal(rec1.Body.Bytes(), &res1); err != nil {
		t.Fatalf("decode: %v", err)
	}
	if res1.UserID != u1 || res1.ActiveView != "trends" {
		t.Fatalf("user 1 unexpected res: %+v", res1)
	}

	req2 := httptest.NewRequest(http.MethodGet, "/api/v1/dashboard/preferences", nil)
	req2 = req2.WithContext(httpinfra.WithUserIDForTest(req2.Context(), u2))
	rec2 := httptest.NewRecorder()
	h.Get(rec2, req2)
	if rec2.Code != http.StatusOK {
		t.Fatalf("expected 200, got %d: %s", rec2.Code, rec2.Body.String())
	}
	var res2 DashboardPreferences
	if err := json.Unmarshal(rec2.Body.Bytes(), &res2); err != nil {
		t.Fatalf("decode: %v", err)
	}
	if res2.UserID != u2 || res2.ActiveView != "custom" {
		t.Fatalf("user 2 unexpected res: %+v", res2)
	}
}
