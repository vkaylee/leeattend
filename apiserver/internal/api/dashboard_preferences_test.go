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

type mockPreferencesRepo struct {
	pref DashboardPreferences
	err  error
	req  UpdateDashboardPreferences
}

func (m *mockPreferencesRepo) GetDashboardPreferences(ctx context.Context, u, w uuid.UUID) (DashboardPreferences, error) {
	if m.err != nil {
		return DashboardPreferences{}, m.err
	}
	res := m.pref
	res.UserID = u
	res.WorkspaceID = w
	return res, nil
}
func (m *mockPreferencesRepo) UpdateDashboardPreferences(ctx context.Context, u, w uuid.UUID, r UpdateDashboardPreferences) (DashboardPreferences, error) {
	m.req = r
	if m.err != nil {
		return DashboardPreferences{}, m.err
	}
	res := m.pref
	res.UserID = u
	res.WorkspaceID = w
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
	return res, nil
}

type prefAuthz struct {
	allowed bool
}

func (a prefAuthz) HasPermission(ctx context.Context, u, w uuid.UUID, p string) bool {
	return a.allowed && p == "dashboard:preferences"
}

func (prefAuthz) IsSystemAdmin(context.Context, uuid.UUID) bool {
	return false
}

func TestDashboardPreferencesHandler_Get(t *testing.T) {
	u := uuid.New()
	w := uuid.New()
	repo := &mockPreferencesRepo{
		pref: DashboardPreferences{
			ID:                uuid.New(),
			ActiveView:        "overview",
			VisibleWidgets:    json.RawMessage(`{"kpi_cards":true}`),
			WidgetOrder:       json.RawMessage(`["kpi_cards"]`),
			CollapsedSections: json.RawMessage(`{"summary":false}`),
		},
	}
	h := DashboardPreferencesHandler{
		Repository: repo,
		Authorizer: prefAuthz{allowed: true},
	}

	req := httptest.NewRequest(http.MethodGet, "/api/v1/dashboard/preferences", nil)
	ctx := httpinfra.WithUserIDForTest(req.Context(), u)
	ctx = httpinfra.WithWorkspaceIDForTest(ctx, w)
	req = req.WithContext(ctx)

	rec := httptest.NewRecorder()
	h.Get(rec, req)
	if rec.Code != http.StatusOK {
		t.Fatalf("expected 200, got %d: %s", rec.Code, rec.Body.String())
	}
	var res DashboardPreferences
	if err := json.Unmarshal(rec.Body.Bytes(), &res); err != nil {
		t.Fatalf("decode: %v", err)
	}
	if res.ActiveView != "overview" || res.UserID != u || res.WorkspaceID != w {
		t.Fatalf("unexpected res: %+v", res)
	}
}

func TestDashboardPreferencesHandler_Put(t *testing.T) {
	u := uuid.New()
	w := uuid.New()
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

	body := `{"active_view":"attendance","visible_widgets":{"kpi_cards":false},"widget_order":["kpi_cards"],"collapsed_sections":{"charts":true}}`
	req := httptest.NewRequest(http.MethodPut, "/api/v1/dashboard/preferences", bytes.NewBufferString(body))
	ctx := httpinfra.WithUserIDForTest(req.Context(), u)
	ctx = httpinfra.WithWorkspaceIDForTest(ctx, w)
	req = req.WithContext(ctx)

	rec := httptest.NewRecorder()
	h.Put(rec, req)
	if rec.Code != http.StatusOK {
		t.Fatalf("expected 200, got %d: %s", rec.Code, rec.Body.String())
	}
	if repo.req.ActiveView == nil || *repo.req.ActiveView != "attendance" {
		t.Fatalf("expected updated active_view: %+v", repo.req)
	}
}

func TestDashboardPreferencesHandler_Forbidden(t *testing.T) {
	u := uuid.New()
	w := uuid.New()
	h := DashboardPreferencesHandler{
		Repository: &mockPreferencesRepo{},
		Authorizer: prefAuthz{allowed: false},
	}
	req := httptest.NewRequest(http.MethodGet, "/api/v1/dashboard/preferences", nil)
	ctx := httpinfra.WithUserIDForTest(req.Context(), u)
	ctx = httpinfra.WithWorkspaceIDForTest(ctx, w)
	req = req.WithContext(ctx)

	rec := httptest.NewRecorder()
	h.Get(rec, req)
	if rec.Code != http.StatusForbidden {
		t.Fatalf("expected 403, got %d", rec.Code)
	}
}

func TestDashboardPreferencesHandler_Validation(t *testing.T) {
	u := uuid.New()
	w := uuid.New()
	h := DashboardPreferencesHandler{
		Repository: &mockPreferencesRepo{},
		Authorizer: prefAuthz{allowed: true},
	}
	body := `{"active_view":"invalid_view"}`
	req := httptest.NewRequest(http.MethodPut, "/api/v1/dashboard/preferences", bytes.NewBufferString(body))
	ctx := httpinfra.WithUserIDForTest(req.Context(), u)
	ctx = httpinfra.WithWorkspaceIDForTest(ctx, w)
	req = req.WithContext(ctx)

	rec := httptest.NewRecorder()
	h.Put(rec, req)
	if rec.Code != http.StatusBadRequest {
		t.Fatalf("expected 400 for invalid active view, got %d", rec.Code)
	}
}
