package api

import (
	"context"
	"net/http"
	"net/http/httptest"
	"testing"

	"github.com/google/uuid"
	httpinfra "github.com/vkaylee/leeattend/apiserver/internal/infra/http"
)

type systemHealthAuthzMock struct {
	allowed bool
}

func (m systemHealthAuthzMock) HasPermission(context.Context, uuid.UUID, string) bool {
	return m.allowed
}
func (systemHealthAuthzMock) HasScopedPermission(context.Context, uuid.UUID, string, *uuid.UUID) bool {
	return false
}
func (systemHealthAuthzMock) IsMember(context.Context, uuid.UUID) (bool, error) { return false, nil }
func (systemHealthAuthzMock) EffectivePermissions(context.Context, uuid.UUID) ([]string, []string, error) {
	return nil, nil, nil
}
func (systemHealthAuthzMock) GetUsersWithPermission(context.Context, string) ([]uuid.UUID, error) {
	return nil, nil
}
func (systemHealthAuthzMock) GetManagedDepartmentIDs(context.Context, uuid.UUID) ([]uuid.UUID, error) {
	return nil, nil
}
func (m systemHealthAuthzMock) IsGlobalManager(context.Context, uuid.UUID) bool {
	return m.allowed
}

func systemHealthRequest(userID uuid.UUID) *http.Request {
	req := httptest.NewRequest(http.MethodGet, "/api/v1/system/health", nil)
	return req.WithContext(httpinfra.WithUserIDForTest(req.Context(), userID))
}

func TestSystemHealthHandlerRequiresInfraPermission(t *testing.T) {
	userID := uuid.New()
	h := SystemHealthHandler{Authorizer: systemHealthAuthzMock{allowed: true}}
	rec := httptest.NewRecorder()
	h.Get(rec, systemHealthRequest(userID))
	if rec.Code != http.StatusOK {
		t.Fatalf("expected 200, got %d: %s", rec.Code, rec.Body.String())
	}
	if rec.Body.String() != `{"status":"ok","viewer_user_id":"`+userID.String()+`"}`+"\n" {
		t.Fatalf("unexpected response: %s", rec.Body.String())
	}
}

func TestSystemHealthHandlerRejectsWithoutInfraPermission(t *testing.T) {
	h := SystemHealthHandler{Authorizer: systemHealthAuthzMock{allowed: false}}
	rec := httptest.NewRecorder()
	h.Get(rec, systemHealthRequest(uuid.New()))
	if rec.Code != http.StatusForbidden {
		t.Fatalf("expected 403, got %d", rec.Code)
	}
}

func TestSystemHealthHandlerRequiresContext(t *testing.T) {
	h := SystemHealthHandler{Authorizer: systemHealthAuthzMock{allowed: true}}
	req := httptest.NewRequest(http.MethodGet, "/api/v1/system/health", nil)
	rec := httptest.NewRecorder()
	h.Get(rec, req)
	if rec.Code != http.StatusUnauthorized {
		t.Fatalf("expected 401, got %d", rec.Code)
	}
}


