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

type mockEmployeeRepo struct{}

func (m mockEmployeeRepo) CreateEmployee(_ context.Context, wsID uuid.UUID, req CreateEmployeeRequest) (EmployeeRecord, error) {
	return EmployeeRecord{ID: uuid.New(), WorkspaceID: wsID, FullName: req.FullName, EmployeeIDNumber: req.EmployeeIDNumber}, nil
}
func (m mockEmployeeRepo) ListEmployees(_ context.Context, wsID uuid.UUID, page, limit int, _ string) (EmployeeListResponse, error) {
	return EmployeeListResponse{Data: []EmployeeRecord{{ID: uuid.New(), WorkspaceID: wsID, FullName: "Emp"}}, Total: 1, Page: page, Limit: limit}, nil
}
func (m mockEmployeeRepo) GetEmployee(_ context.Context, wsID, empID uuid.UUID) (EmployeeRecord, error) {
	return EmployeeRecord{ID: empID, WorkspaceID: wsID, FullName: "Emp"}, nil
}
func (m mockEmployeeRepo) UpdateEmployee(_ context.Context, wsID, empID uuid.UUID, req UpdateEmployeeRequest) (EmployeeRecord, error) {
	name := "Emp"
	if req.FullName != nil {
		name = *req.FullName
	}
	return EmployeeRecord{ID: empID, WorkspaceID: wsID, FullName: name}, nil
}
func (m mockEmployeeRepo) SoftDeleteEmployee(_ context.Context, _, _ uuid.UUID) error { return nil }
func (m mockEmployeeRepo) RehireEmployee(_ context.Context, wsID uuid.UUID, empID uuid.UUID, _ RehireEmployeeRequest) (EmployeeRecord, error) {
	return EmployeeRecord{ID: empID, WorkspaceID: wsID, FullName: "Rehired"}, nil
}

func TestEmployeeHandler_ListRequiresPermission(t *testing.T) {
	wsID := uuid.New()
	h := EmployeeHandler{Repository: mockEmployeeRepo{}, Authorizer: mockAuthorizer{perms: map[string]bool{}}}
	req := httptest.NewRequest(http.MethodGet, "/api/v1/workspaces/"+wsID.String()+"/employees", nil)
	req.SetPathValue("ws_id", wsID.String())
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), uuid.New()))
	rec := httptest.NewRecorder()

	h.List(rec, req)
	if rec.Code != http.StatusForbidden {
		t.Fatalf("expected 403, got %d", rec.Code)
	}
}

func TestEmployeeHandler_DeleteSuccess(t *testing.T) {
	wsID, empID := uuid.New(), uuid.New()
	h := EmployeeHandler{Repository: mockEmployeeRepo{}, Authorizer: mockAuthorizer{perms: map[string]bool{"employees:delete": true}}}
	req := httptest.NewRequest(http.MethodDelete, "/api/v1/workspaces/"+wsID.String()+"/employees/"+empID.String(), nil)
	req.SetPathValue("ws_id", wsID.String())
	req.SetPathValue("id", empID.String())
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), uuid.New()))
	rec := httptest.NewRecorder()

	h.Delete(rec, req)
	if rec.Code != http.StatusNoContent {
		t.Fatalf("expected 204, got %d", rec.Code)
	}
}

func TestEmployeeHandler_CreateFailsOnEmptyFields(t *testing.T) {
	wsID := uuid.New()
	h := EmployeeHandler{Repository: mockEmployeeRepo{}, Authorizer: mockAuthorizer{perms: map[string]bool{"employees:create": true}}}
	req := httptest.NewRequest(http.MethodPost, "/api/v1/workspaces/"+wsID.String()+"/employees", bytes.NewBufferString(`{}`))
	req.SetPathValue("ws_id", wsID.String())
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), uuid.New()))
	rec := httptest.NewRecorder()

	h.Create(rec, req)
	if rec.Code != http.StatusBadRequest {
		t.Fatalf("expected 400, got %d", rec.Code)
	}
}

func TestEmployeeHandler_Get(t *testing.T) {
	wsID, empID := uuid.New(), uuid.New()
	h := EmployeeHandler{Repository: mockEmployeeRepo{}, Authorizer: mockAuthorizer{perms: map[string]bool{"employees:read": true}}}
	req := httptest.NewRequest(http.MethodGet, "/api/v1/workspaces/"+wsID.String()+"/employees/"+empID.String(), nil)
	req.SetPathValue("ws_id", wsID.String())
	req.SetPathValue("id", empID.String())
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), uuid.New()))
	rec := httptest.NewRecorder()
	h.Get(rec, req)
	if rec.Code != http.StatusOK {
		t.Fatalf("expected 200, got %d: %s", rec.Code, rec.Body.String())
	}
}

func TestEmployeeHandler_Update(t *testing.T) {
	wsID, empID := uuid.New(), uuid.New()
	h := EmployeeHandler{Repository: mockEmployeeRepo{}, Authorizer: mockAuthorizer{perms: map[string]bool{"employees:update": true}}}
	req := httptest.NewRequest(http.MethodPut, "/api/v1/workspaces/"+wsID.String()+"/employees/"+empID.String(), bytes.NewBufferString(`{"full_name":"Updated Name"}`))
	req.SetPathValue("ws_id", wsID.String())
	req.SetPathValue("id", empID.String())
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), uuid.New()))
	rec := httptest.NewRecorder()
	h.Update(rec, req)
	if rec.Code != http.StatusOK {
		t.Fatalf("expected 200, got %d: %s", rec.Code, rec.Body.String())
	}
}

func TestEmployeeHandler_UpdateForbidden(t *testing.T) {
	wsID, empID := uuid.New(), uuid.New()
	h := EmployeeHandler{Repository: mockEmployeeRepo{}, Authorizer: mockAuthorizer{perms: map[string]bool{}}}
	req := httptest.NewRequest(http.MethodPut, "/api/v1/workspaces/"+wsID.String()+"/employees/"+empID.String(), bytes.NewBufferString(`{"full_name":"Updated Name"}`))
	req.SetPathValue("ws_id", wsID.String())
	req.SetPathValue("id", empID.String())
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), uuid.New()))
	rec := httptest.NewRecorder()
	h.Update(rec, req)
	if rec.Code != http.StatusForbidden {
		t.Fatalf("expected 403, got %d", rec.Code)
	}
}
