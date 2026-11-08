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

type employeeSliceAuthorizer struct {
	perms map[string]bool
}

func (a employeeSliceAuthorizer) HasPermission(_ context.Context, _ uuid.UUID, permission string) bool {
	return a.perms[permission]
}
func (a employeeSliceAuthorizer) HasScopedPermission(ctx context.Context, userID uuid.UUID, permission string, _ *uuid.UUID) bool {
	return a.HasPermission(ctx, userID, permission)
}
func (employeeSliceAuthorizer) IsMember(context.Context, uuid.UUID) (bool, error) { return true, nil }
func (employeeSliceAuthorizer) EffectivePermissions(context.Context, uuid.UUID) ([]string, []string, error) {
	return nil, nil, nil
}
func (employeeSliceAuthorizer) GetUsersWithPermission(context.Context, string) ([]uuid.UUID, error) {
	return nil, nil
}

type mockEmployeeRepo struct{}

func (mockEmployeeRepo) CreateEmployee(_ context.Context, req CreateEmployeeRequest) (EmployeeRecord, error) {
	return EmployeeRecord{ID: uuid.New(), FullName: req.FullName, EmployeeIDNumber: req.EmployeeIDNumber}, nil
}
func (mockEmployeeRepo) ListEmployees(_ context.Context, page, limit int, _ string) (EmployeeListResponse, error) {
	return EmployeeListResponse{Data: []EmployeeRecord{{ID: uuid.New(), FullName: "Emp"}}, Total: 1, Page: page, Limit: limit}, nil
}
func (mockEmployeeRepo) GetEmployee(_ context.Context, empID uuid.UUID) (EmployeeRecord, error) {
	return EmployeeRecord{ID: empID, FullName: "Emp"}, nil
}
func (mockEmployeeRepo) UpdateEmployee(_ context.Context, empID uuid.UUID, req UpdateEmployeeRequest) (EmployeeRecord, error) {
	name := "Emp"
	if req.FullName != nil {
		name = *req.FullName
	}
	return EmployeeRecord{ID: empID, FullName: name}, nil
}
func (mockEmployeeRepo) SoftDeleteEmployee(context.Context, uuid.UUID) error { return nil }
func (mockEmployeeRepo) RehireEmployee(_ context.Context, empID uuid.UUID, _ RehireEmployeeRequest) (EmployeeRecord, error) {
	return EmployeeRecord{ID: empID, FullName: "Rehired"}, nil
}

func TestEmployeeHandler_ListRequiresPermission(t *testing.T) {
	h := EmployeeHandler{Repository: mockEmployeeRepo{}, Authorizer: employeeSliceAuthorizer{perms: map[string]bool{}}}
	req := httptest.NewRequest(http.MethodGet, "/api/v1/employees", nil)
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), uuid.New()))
	rec := httptest.NewRecorder()

	h.List(rec, req)
	if rec.Code != http.StatusForbidden {
		t.Fatalf("expected 403, got %d", rec.Code)
	}
}

func TestEmployeeHandler_DeleteSuccess(t *testing.T) {
	empID := uuid.New()
	h := EmployeeHandler{Repository: mockEmployeeRepo{}, Authorizer: employeeSliceAuthorizer{perms: map[string]bool{"employees:delete": true}}}
	req := httptest.NewRequest(http.MethodDelete, "/api/v1/employees/"+empID.String(), nil)
	req.SetPathValue("id", empID.String())
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), uuid.New()))
	rec := httptest.NewRecorder()

	h.Delete(rec, req)
	if rec.Code != http.StatusNoContent {
		t.Fatalf("expected 204, got %d", rec.Code)
	}
}

func TestEmployeeHandler_CreateFailsOnEmptyFields(t *testing.T) {
	h := EmployeeHandler{Repository: mockEmployeeRepo{}, Authorizer: employeeSliceAuthorizer{perms: map[string]bool{"employees:create": true}}}
	req := httptest.NewRequest(http.MethodPost, "/api/v1/employees", bytes.NewBufferString(`{}`))
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), uuid.New()))
	rec := httptest.NewRecorder()

	h.Create(rec, req)
	if rec.Code != http.StatusBadRequest {
		t.Fatalf("expected 400, got %d", rec.Code)
	}
}

func TestEmployeeHandler_Get(t *testing.T) {
	empID := uuid.New()
	h := EmployeeHandler{Repository: mockEmployeeRepo{}, Authorizer: employeeSliceAuthorizer{perms: map[string]bool{"employees:read": true}}}
	req := httptest.NewRequest(http.MethodGet, "/api/v1/employees/"+empID.String(), nil)
	req.SetPathValue("id", empID.String())
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), uuid.New()))
	rec := httptest.NewRecorder()
	h.Get(rec, req)
	if rec.Code != http.StatusOK {
		t.Fatalf("expected 200, got %d: %s", rec.Code, rec.Body.String())
	}
}

func TestEmployeeHandler_Update(t *testing.T) {
	empID := uuid.New()
	h := EmployeeHandler{Repository: mockEmployeeRepo{}, Authorizer: employeeSliceAuthorizer{perms: map[string]bool{"employees:update": true}}}
	req := httptest.NewRequest(http.MethodPut, "/api/v1/employees/"+empID.String(), bytes.NewBufferString(`{"full_name":"Updated Name"}`))
	req.SetPathValue("id", empID.String())
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), uuid.New()))
	rec := httptest.NewRecorder()
	h.Update(rec, req)
	if rec.Code != http.StatusOK {
		t.Fatalf("expected 200, got %d: %s", rec.Code, rec.Body.String())
	}
}

func TestEmployeeHandler_UpdateForbidden(t *testing.T) {
	empID := uuid.New()
	h := EmployeeHandler{Repository: mockEmployeeRepo{}, Authorizer: employeeSliceAuthorizer{perms: map[string]bool{}}}
	req := httptest.NewRequest(http.MethodPut, "/api/v1/employees/"+empID.String(), bytes.NewBufferString(`{"full_name":"Updated Name"}`))
	req.SetPathValue("id", empID.String())
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), uuid.New()))
	rec := httptest.NewRecorder()
	h.Update(rec, req)
	if rec.Code != http.StatusForbidden {
		t.Fatalf("expected 403, got %d", rec.Code)
	}
}
