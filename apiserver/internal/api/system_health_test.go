package api

import (
	"context"
	"errors"
	"net/http"
	"net/http/httptest"
	"testing"

	"github.com/google/uuid"
	httpinfra "github.com/vkaylee/leeattend/apiserver/internal/infra/http"
)

type systemHealthRepoMock struct {
	roles []string
	err   error
}

func (m systemHealthRepoMock) GetSystemRolesForUser(context.Context, uuid.UUID) ([]string, error) {
	return m.roles, m.err
}

func systemHealthRequest(userID uuid.UUID) *http.Request {
	req := httptest.NewRequest(http.MethodGet, "/api/v1/system/health", nil)
	return req.WithContext(httpinfra.WithUserIDForTest(req.Context(), userID))
}

func TestSystemHealthHandlerPlatformAdmin(t *testing.T) {
	userID := uuid.New()
	h := SystemHealthHandler{Roles: systemHealthRepoMock{roles: []string{"platform_admin", "unknown_role"}}}
	rec := httptest.NewRecorder()
	h.Get(rec, systemHealthRequest(userID))
	if rec.Code != http.StatusOK {
		t.Fatalf("expected 200, got %d: %s", rec.Code, rec.Body.String())
	}
	want := `{"status":"ok","viewer_user_id":"` + userID.String() + `","viewer_roles":["platform_admin"]}`
	if rec.Body.String() != want+"\n" {
		t.Fatalf("unexpected response: %s", rec.Body.String())
	}
}

func TestSystemHealthHandlerSupport(t *testing.T) {
	h := SystemHealthHandler{Roles: systemHealthRepoMock{roles: []string{"support"}}}
	rec := httptest.NewRecorder()
	h.Get(rec, systemHealthRequest(uuid.New()))
	if rec.Code != http.StatusOK {
		t.Fatalf("expected support role 200, got %d", rec.Code)
	}
}

func TestSystemHealthHandlerRejectsBillingAndEmptyRoles(t *testing.T) {
	for _, roles := range [][]string{{"billing_admin"}, {}} {
		h := SystemHealthHandler{Roles: systemHealthRepoMock{roles: roles}}
		rec := httptest.NewRecorder()
		h.Get(rec, systemHealthRequest(uuid.New()))
		if rec.Code != http.StatusForbidden {
			t.Fatalf("expected 403 for roles %v, got %d", roles, rec.Code)
		}
	}
}

func TestSystemHealthHandlerRepositoryError(t *testing.T) {
	h := SystemHealthHandler{Roles: systemHealthRepoMock{err: errors.New("role query failed")}}
	rec := httptest.NewRecorder()
	h.Get(rec, systemHealthRequest(uuid.New()))
	if rec.Code != http.StatusInternalServerError {
		t.Fatalf("expected 500, got %d", rec.Code)
	}
}

func TestSystemHealthHandlerRequiresUserContext(t *testing.T) {
	h := SystemHealthHandler{Roles: systemHealthRepoMock{roles: []string{"platform_admin"}}}
	req := httptest.NewRequest(http.MethodGet, "/api/v1/system/health", nil)
	rec := httptest.NewRecorder()
	h.Get(rec, req)
	if rec.Code != http.StatusUnauthorized {
		t.Fatalf("expected 401, got %d", rec.Code)
	}
}
