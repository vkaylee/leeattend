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
func (employeeSliceAuthorizer) GetManagedDepartmentIDs(context.Context, uuid.UUID) ([]uuid.UUID, error) {
	return nil, nil
}
func (employeeSliceAuthorizer) IsGlobalManager(context.Context, uuid.UUID) bool { return false }

type mockEmployeeRepo struct{}

func (mockEmployeeRepo) CreateEmployee(_ context.Context, req CreateEmployeeRequest) (EmployeeRecord, error) {
	return EmployeeRecord{ID: uuid.New(), FullName: req.FullName, EmployeeIDNumber: req.EmployeeIDNumber}, nil
}

type captureCreateRepo struct {
	mockEmployeeRepo
	request CreateEmployeeRequest
}

func (m *captureCreateRepo) CreateEmployee(_ context.Context, request CreateEmployeeRequest) (EmployeeRecord, error) {
	m.request = request
	return EmployeeRecord{ID: uuid.New(), FullName: request.FullName, EmployeeIDNumber: request.EmployeeIDNumber}, nil
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

type captureTenureCreator struct {
	request CreateEmployeeTenureRequest
}

func (m *captureTenureCreator) CreateEmployeeTenure(_ context.Context, empID uuid.UUID, request CreateEmployeeTenureRequest) (EmployeeTenureCreated, error) {
	m.request = request
	code := "EMP-REUSED"
	if request.EmployeeNumber != nil {
		code = *request.EmployeeNumber
	}
	return EmployeeTenureCreated{
		TenureID:     uuid.New(),
		EmployeeID:   empID,
		EmployeeCode: code,
		HireType:     "rehire",
		StartDate:    request.StartDate,
		EmployerID:   request.EmployerID,
		Status:       MatchStatusActive,
	}, nil
}

func TestEmployeeHandler_CreatePreservesOptionalUserID(t *testing.T) {
	repo := &captureCreateRepo{}
	userID := uuid.New()
	h := EmployeeHandler{Repository: repo, Authorizer: employeeSliceAuthorizer{perms: map[string]bool{"employees:create": true}}}
	req := httptest.NewRequest(http.MethodPost, "/api/v1/employees", bytes.NewBufferString(`{"full_name":"New Emp","employee_id_number":"EMP-100","hire_date":"2025-01-01","user_id":"`+userID.String()+`"}`))
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), uuid.New()))
	rec := httptest.NewRecorder()

	h.Create(rec, req)
	if rec.Code != http.StatusCreated {
		t.Fatalf("expected 201, got %d: %s", rec.Code, rec.Body.String())
	}
	if repo.request.UserID == nil || *repo.request.UserID != userID {
		t.Fatalf("create user_id = %v, want %s", repo.request.UserID, userID)
	}
}

func TestEmployeeHandler_CreateTenureAcceptsOptionalEmployeeNumber(t *testing.T) {
	creator := &captureTenureCreator{}
	employeeID := uuid.New()
	h := EmployeeHandler{TenureCreator: creator, Authorizer: employeeSliceAuthorizer{perms: map[string]bool{"employees:rehire": true}}}
	req := httptest.NewRequest(http.MethodPost, "/api/v1/employees/"+employeeID.String()+"/tenures", bytes.NewBufferString(`{"start_date":"2026-02-01","employee_number":"1541","employer_id":"11111111-1111-1111-1111-111111111111","prior_service_credit_days":90}`))
	req.SetPathValue("employee_id", employeeID.String())
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), uuid.New()))
	rec := httptest.NewRecorder()

	h.CreateTenure(rec, req)
	if rec.Code != http.StatusCreated {
		t.Fatalf("expected 201, got %d: %s", rec.Code, rec.Body.String())
	}
	if creator.request.EmployeeNumber == nil || *creator.request.EmployeeNumber != "1541" {
		t.Fatalf("rehire employee number = %v, want 1541", creator.request.EmployeeNumber)
	}
	if creator.request.PriorServiceCreditDays == nil || *creator.request.PriorServiceCreditDays != 90 {
		t.Fatalf("prior_service_credit_days = %v, want 90", creator.request.PriorServiceCreditDays)
	}
}

func TestEmployeeHandler_CreateTenureAllowsOmittedEmployeeNumber(t *testing.T) {
	creator := &captureTenureCreator{}
	employeeID := uuid.New()
	h := EmployeeHandler{TenureCreator: creator, Authorizer: employeeSliceAuthorizer{perms: map[string]bool{"employees:rehire": true}}}
	req := httptest.NewRequest(http.MethodPost, "/api/v1/employees/"+employeeID.String()+"/tenures", bytes.NewBufferString(`{"start_date":"2026-02-01","employer_id":"11111111-1111-1111-1111-111111111111"}`))
	req.SetPathValue("employee_id", employeeID.String())
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), uuid.New()))
	rec := httptest.NewRecorder()

	h.CreateTenure(rec, req)
	if rec.Code != http.StatusCreated {
		t.Fatalf("expected 201, got %d: %s", rec.Code, rec.Body.String())
	}
	if creator.request.EmployeeNumber != nil {
		t.Fatalf("rehire employee number = %v, want omitted", creator.request.EmployeeNumber)
	}
}

func TestEmployeeHandler_CreateTenureRequiresEmployer(t *testing.T) {
	h := EmployeeHandler{TenureCreator: &captureTenureCreator{}, Authorizer: employeeSliceAuthorizer{perms: map[string]bool{"employees:rehire": true}}}
	employeeID := uuid.NewString()
	req := httptest.NewRequest(http.MethodPost, "/api/v1/employees/"+employeeID+"/tenures", bytes.NewBufferString(`{"start_date":"2026-02-01"}`))
	req.SetPathValue("employee_id", employeeID)
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), uuid.New()))
	rec := httptest.NewRecorder()

	h.CreateTenure(rec, req)
	if rec.Code != http.StatusBadRequest {
		t.Fatalf("expected 400, got %d: %s", rec.Code, rec.Body.String())
	}
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
