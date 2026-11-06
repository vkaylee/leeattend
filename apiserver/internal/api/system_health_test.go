package api

import (
	"context"
	"net/http"
	"net/http/httptest"
	"testing"

	"github.com/google/uuid"
	"github.com/vkaylee/leeattend/apiserver/internal/auth"
	httpinfra "github.com/vkaylee/leeattend/apiserver/internal/infra/http"
)

type systemHealthAuthzMock struct {
	allowed bool
}

func (m systemHealthAuthzMock) HasPermission(context.Context, uuid.UUID, uuid.UUID, string) bool {
	return m.allowed
}

func systemHealthRequest(userID, workspaceID uuid.UUID) *http.Request {
	req := httptest.NewRequest(http.MethodGet, "/api/v1/system/health", nil)
	ctx := httpinfra.WithUserIDForTest(req.Context(), userID)
	return req.WithContext(httpinfra.WithWorkspaceIDForTest(ctx, workspaceID))
}

func TestSystemHealthHandlerRequiresInfraPermission(t *testing.T) {
	userID, workspaceID := uuid.New(), uuid.New()
	h := SystemHealthHandler{Authorizer: systemHealthAuthzMock{allowed: true}}
	rec := httptest.NewRecorder()
	h.Get(rec, systemHealthRequest(userID, workspaceID))
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
	h.Get(rec, systemHealthRequest(uuid.New(), uuid.New()))
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

var _ = auth.PermissionInfraMonitor
