package api

import (
	"bytes"
	"context"
	"net/http"
	"net/http/httptest"
	"testing"

	"github.com/google/uuid"
	"github.com/shopspring/decimal"
	httpinfra "github.com/vkaylee/leeattend/apiserver/internal/infra/http"
)

type mockDepartmentRepo struct {
	notFound bool
}

func (m mockDepartmentRepo) ListDepartments(_ context.Context, wsID uuid.UUID, page, limit int) (DepartmentListResponse, error) {
	return DepartmentListResponse{
		Data: []DepartmentRecord{
			{ID: uuid.New(), WorkspaceID: wsID, Name: "Engineering", ActiveEmployeeCount: 3},
		},
		Total: 1,
		Page:  page,
		Limit: limit,
	}, nil
}

func (m mockDepartmentRepo) CreateDepartment(_ context.Context, wsID uuid.UUID, req CreateDepartmentRequest) (DepartmentRecord, error) {
	return DepartmentRecord{
		ID:          uuid.New(),
		WorkspaceID: wsID,
		Name:        req.Name,
		Description: req.Description,
		ManagerID:   req.ManagerID,
	}, nil
}

func (m mockDepartmentRepo) GetDepartmentByID(_ context.Context, wsID, deptID uuid.UUID) (DepartmentRecord, error) {
	if m.notFound {
		return DepartmentRecord{}, ErrDepartmentNotFound
	}
	return DepartmentRecord{ID: deptID, WorkspaceID: wsID, Name: "Engineering"}, nil
}

func (m mockDepartmentRepo) UpdateDepartment(_ context.Context, wsID, deptID uuid.UUID, req UpdateDepartmentRequest) (DepartmentRecord, error) {
	if m.notFound {
		return DepartmentRecord{}, ErrDepartmentNotFound
	}
	name := "Engineering"
	if req.Name != nil {
		name = *req.Name
	}
	return DepartmentRecord{ID: deptID, WorkspaceID: wsID, Name: name}, nil
}

func (m mockDepartmentRepo) DeleteDepartment(_ context.Context, _, _ uuid.UUID) error {
	if m.notFound {
		return ErrDepartmentNotFound
	}
	return nil
}

func (m mockDepartmentRepo) GetDepartmentPayrollConfigs(_ context.Context, _, deptID uuid.UUID) ([]DepartmentPayrollConfig, error) {
	if m.notFound {
		return []DepartmentPayrollConfig{}, nil
	}
	return []DepartmentPayrollConfig{
		{
			ID:             uuid.New(),
			DepartmentID:   deptID,
			StdWorkingDays: decimal.NewFromFloat(22),
			HoursPerDay:    decimal.NewFromFloat(8),
			EffectiveDate:  "2026-01-01",
		},
	}, nil
}

func (m mockDepartmentRepo) SetDepartmentPayrollConfig(_ context.Context, _, deptID uuid.UUID, req SetDepartmentPayrollConfigRequest, _ uuid.UUID) (DepartmentPayrollConfig, error) {
	if m.notFound {
		return DepartmentPayrollConfig{}, ErrDepartmentNotFound
	}
	return DepartmentPayrollConfig{
		ID:             uuid.New(),
		DepartmentID:   deptID,
		StdWorkingDays: req.StdWorkingDays,
		HoursPerDay:    req.HoursPerDay,
		EffectiveDate:  req.EffectiveDate,
	}, nil
}

func TestDepartmentHandler_List(t *testing.T) {
	wsID, userID := uuid.New(), uuid.New()
	h := DepartmentHandler{Repository: mockDepartmentRepo{}, Authorizer: mockAuthorizer{perms: map[string]bool{"departments:read": true}}}
	req := httptest.NewRequest(http.MethodGet, "/api/v1/workspaces/"+wsID.String()+"/departments", nil)
	req = req.WithContext(httpinfra.WithOrganizationIDForTest(req.Context(), wsID))
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), userID))
	rec := httptest.NewRecorder()

	h.List(rec, req)
	if rec.Code != http.StatusOK {
		t.Fatalf("expected 200, got %d", rec.Code)
	}
}

func TestDepartmentHandler_Create(t *testing.T) {
	wsID, userID := uuid.New(), uuid.New()
	h := DepartmentHandler{
		Repository: mockDepartmentRepo{},
		Authorizer: mockAuthorizer{perms: map[string]bool{"departments:create": true}},
	}

	// Success
	req := httptest.NewRequest(http.MethodPost, "/api/v1/workspaces/"+wsID.String()+"/departments", bytes.NewBufferString(`{"name":"Finance","description":"Finance team"}`))
	req = req.WithContext(httpinfra.WithOrganizationIDForTest(req.Context(), wsID))
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), userID))
	rec := httptest.NewRecorder()
	h.Create(rec, req)
	if rec.Code != http.StatusCreated {
		t.Fatalf("expected 201, got %d", rec.Code)
	}

	// Forbidden
	hNoAuth := DepartmentHandler{Repository: mockDepartmentRepo{}, Authorizer: mockAuthorizer{perms: map[string]bool{}}}
	req = httptest.NewRequest(http.MethodPost, "/api/v1/workspaces/"+wsID.String()+"/departments", bytes.NewBufferString(`{"name":"Finance"}`))
	req = req.WithContext(httpinfra.WithOrganizationIDForTest(req.Context(), wsID))
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), userID))
	rec = httptest.NewRecorder()
	hNoAuth.Create(rec, req)
	if rec.Code != http.StatusForbidden {
		t.Fatalf("expected 403, got %d", rec.Code)
	}

	// Empty name
	req = httptest.NewRequest(http.MethodPost, "/api/v1/workspaces/"+wsID.String()+"/departments", bytes.NewBufferString(`{"name":""}`))
	req = req.WithContext(httpinfra.WithOrganizationIDForTest(req.Context(), wsID))
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), userID))
	rec = httptest.NewRecorder()
	h.Create(rec, req)
	if rec.Code != http.StatusBadRequest {
		t.Fatalf("expected 400 for empty name, got %d", rec.Code)
	}
}

func TestDepartmentHandler_UpdateAndDelete(t *testing.T) {
	wsID, deptID, userID := uuid.New(), uuid.New(), uuid.New()
	h := DepartmentHandler{
		Repository: mockDepartmentRepo{},
		Authorizer: mockAuthorizer{perms: map[string]bool{
			"departments:update": true,
			"departments:delete": true,
		}},
	}

	// Update
	req := httptest.NewRequest(http.MethodPatch, "/api/v1/workspaces/"+wsID.String()+"/departments/"+deptID.String(), bytes.NewBufferString(`{"name":"R&D"}`))
	req = req.WithContext(httpinfra.WithOrganizationIDForTest(req.Context(), wsID))
	req.SetPathValue("id", deptID.String())
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), userID))
	rec := httptest.NewRecorder()
	h.Update(rec, req)
	if rec.Code != http.StatusOK {
		t.Fatalf("expected 200, got %d", rec.Code)
	}

	// Delete
	req = httptest.NewRequest(http.MethodDelete, "/api/v1/workspaces/"+wsID.String()+"/departments/"+deptID.String(), nil)
	req = req.WithContext(httpinfra.WithOrganizationIDForTest(req.Context(), wsID))
	req.SetPathValue("id", deptID.String())
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), userID))
	rec = httptest.NewRecorder()
	h.Delete(rec, req)
	if rec.Code != http.StatusNoContent {
		t.Fatalf("expected 204, got %d", rec.Code)
	}
}

func TestDepartmentHandler_PayrollConfig(t *testing.T) {
	wsID, deptID, userID := uuid.New(), uuid.New(), uuid.New()
	h := DepartmentHandler{Repository: mockDepartmentRepo{}}

	// Get
	req := httptest.NewRequest(http.MethodGet, "/api/v1/workspaces/"+wsID.String()+"/departments/"+deptID.String()+"/payroll-config", nil)
	req = req.WithContext(httpinfra.WithOrganizationIDForTest(req.Context(), wsID))
	req.SetPathValue("id", deptID.String())
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), userID))
	rec := httptest.NewRecorder()
	h.GetPayrollConfig(rec, req)
	if rec.Code != http.StatusOK {
		t.Fatalf("expected 200, got %d", rec.Code)
	}

	// Set
	req = httptest.NewRequest(http.MethodPost, "/api/v1/workspaces/"+wsID.String()+"/departments/"+deptID.String()+"/payroll-config", bytes.NewBufferString(`{"std_working_days":"22.00","hours_per_day":"8.00","effective_date":"2026-01-01"}`))
	req = req.WithContext(httpinfra.WithOrganizationIDForTest(req.Context(), wsID))
	req.SetPathValue("id", deptID.String())
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), userID))
	rec = httptest.NewRecorder()
	h.SetPayrollConfig(rec, req)
	if rec.Code != http.StatusCreated {
		t.Fatalf("expected 201, got %d", rec.Code)
	}

	// Set on non-existent department returns 404
	hNotFound := DepartmentHandler{Repository: mockDepartmentRepo{notFound: true}}
	req = httptest.NewRequest(http.MethodPost, "/api/v1/workspaces/"+wsID.String()+"/departments/"+deptID.String()+"/payroll-config", bytes.NewBufferString(`{"std_working_days":"22.00","hours_per_day":"8.00","effective_date":"2026-01-01"}`))
	req = req.WithContext(httpinfra.WithOrganizationIDForTest(req.Context(), wsID))
	req.SetPathValue("id", deptID.String())
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), userID))
	rec = httptest.NewRecorder()
	hNotFound.SetPayrollConfig(rec, req)
	if rec.Code != http.StatusNotFound {
		t.Fatalf("expected 404 for missing department, got %d", rec.Code)
	}
}
