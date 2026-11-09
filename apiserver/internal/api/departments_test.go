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

func (m mockDepartmentRepo) ListDepartments(_ context.Context, page, limit int) (DepartmentListResponse, error) {
	return DepartmentListResponse{
		Data: []DepartmentRecord{
			{ID: uuid.New(), Name: "Engineering", ActiveEmployeeCount: 3},
		},
		Total: 1,
		Page:  page,
		Limit: limit,
	}, nil
}

func (m mockDepartmentRepo) CreateDepartment(_ context.Context, req CreateDepartmentRequest) (DepartmentRecord, error) {
	return DepartmentRecord{
		ID:          uuid.New(),
		Name:        req.Name,
		Description: req.Description,
		ManagerID:   req.ManagerID,
	}, nil
}

func (m mockDepartmentRepo) GetDepartmentByID(_ context.Context, deptID uuid.UUID) (DepartmentRecord, error) {
	if m.notFound {
		return DepartmentRecord{}, ErrDepartmentNotFound
	}
	return DepartmentRecord{ID: deptID, Name: "Engineering"}, nil
}

func (m mockDepartmentRepo) UpdateDepartment(_ context.Context, deptID uuid.UUID, req UpdateDepartmentRequest) (DepartmentRecord, error) {
	if m.notFound {
		return DepartmentRecord{}, ErrDepartmentNotFound
	}
	name := "Engineering"
	if req.Name != nil {
		name = *req.Name
	}
	return DepartmentRecord{ID: deptID, Name: name}, nil
}

func (m mockDepartmentRepo) DeleteDepartment(_ context.Context, _ uuid.UUID) error {
	if m.notFound {
		return ErrDepartmentNotFound
	}
	return nil
}

func (m mockDepartmentRepo) GetDepartmentPayrollConfigs(_ context.Context, deptID uuid.UUID) ([]DepartmentPayrollConfig, error) {
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

func (m mockDepartmentRepo) SetDepartmentPayrollConfig(_ context.Context, deptID uuid.UUID, req SetDepartmentPayrollConfigRequest, _ uuid.UUID) (DepartmentPayrollConfig, error) {
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

type mockDepartmentAuthz struct {
	permissions map[string]bool
}

func (a mockDepartmentAuthz) HasPermission(_ context.Context, _ uuid.UUID, permission string) bool {
	return a.permissions[permission]
}

func (a mockDepartmentAuthz) HasScopedPermission(_ context.Context, _ uuid.UUID, permission string, _ *uuid.UUID) bool {
	return a.permissions[permission]
}

func (a mockDepartmentAuthz) IsMember(_ context.Context, _ uuid.UUID) (bool, error) {
	return true, nil
}

func (a mockDepartmentAuthz) EffectivePermissions(_ context.Context, _ uuid.UUID) ([]string, []string, error) {
	return nil, nil, nil
}

func (a mockDepartmentAuthz) GetUsersWithPermission(_ context.Context, _ string) ([]uuid.UUID, error) {
	return nil, nil
}

func (a mockDepartmentAuthz) GetManagedDepartmentIDs(_ context.Context, _ uuid.UUID) ([]uuid.UUID, error) {
	return nil, nil
}

func (a mockDepartmentAuthz) IsGlobalManager(_ context.Context, _ uuid.UUID) bool {
	return false
}

func departmentRequest(method, path, body string, userID uuid.UUID) *http.Request {
	req := httptest.NewRequest(method, path, bytes.NewBufferString(body))
	return req.WithContext(httpinfra.WithUserIDForTest(req.Context(), userID))
}

func TestDepartmentHandler_List(t *testing.T) {
	userID := uuid.New()
	h := DepartmentHandler{Repository: mockDepartmentRepo{}, Authorizer: mockDepartmentAuthz{permissions: map[string]bool{"departments:read": true}}}
	req := departmentRequest(http.MethodGet, "/api/v1/departments", "", userID)
	rec := httptest.NewRecorder()

	h.List(rec, req)
	if rec.Code != http.StatusOK {
		t.Fatalf("expected 200, got %d", rec.Code)
	}
}

func TestDepartmentHandler_Create(t *testing.T) {
	userID := uuid.New()
	h := DepartmentHandler{
		Repository: mockDepartmentRepo{},
		Authorizer: mockDepartmentAuthz{permissions: map[string]bool{"departments:create": true}},
	}

	// Success
	req := departmentRequest(http.MethodPost, "/api/v1/departments", `{"name":"Finance","description":"Finance team"}`, userID)
	rec := httptest.NewRecorder()
	h.Create(rec, req)
	if rec.Code != http.StatusCreated {
		t.Fatalf("expected 201, got %d", rec.Code)
	}

	// Forbidden
	hNoAuth := DepartmentHandler{Repository: mockDepartmentRepo{}, Authorizer: mockDepartmentAuthz{permissions: map[string]bool{}}}
	req = departmentRequest(http.MethodPost, "/api/v1/departments", `{"name":"Finance"}`, userID)
	rec = httptest.NewRecorder()
	hNoAuth.Create(rec, req)
	if rec.Code != http.StatusForbidden {
		t.Fatalf("expected 403, got %d", rec.Code)
	}

	// Empty name
	req = departmentRequest(http.MethodPost, "/api/v1/departments", `{"name":""}`, userID)
	rec = httptest.NewRecorder()
	h.Create(rec, req)
	if rec.Code != http.StatusBadRequest {
		t.Fatalf("expected 400 for empty name, got %d", rec.Code)
	}
}

func TestDepartmentHandler_UpdateAndDelete(t *testing.T) {
	deptID, userID := uuid.New(), uuid.New()
	h := DepartmentHandler{
		Repository: mockDepartmentRepo{},
		Authorizer: mockDepartmentAuthz{permissions: map[string]bool{
			"departments:update": true,
			"departments:delete": true,
		}},
	}

	// Update
	req := departmentRequest(http.MethodPatch, "/api/v1/departments/"+deptID.String(), `{"name":"R&D"}`, userID)
	req.SetPathValue("id", deptID.String())
	rec := httptest.NewRecorder()
	h.Update(rec, req)
	if rec.Code != http.StatusOK {
		t.Fatalf("expected 200, got %d", rec.Code)
	}

	// Delete
	req = departmentRequest(http.MethodDelete, "/api/v1/departments/"+deptID.String(), "", userID)
	req.SetPathValue("id", deptID.String())
	rec = httptest.NewRecorder()
	h.Delete(rec, req)
	if rec.Code != http.StatusNoContent {
		t.Fatalf("expected 204, got %d", rec.Code)
	}
}

func TestDepartmentHandler_PayrollConfig(t *testing.T) {
	deptID, userID := uuid.New(), uuid.New()
	h := DepartmentHandler{Repository: mockDepartmentRepo{}}

	// Get
	req := departmentRequest(http.MethodGet, "/api/v1/departments/"+deptID.String()+"/payroll-config", "", userID)
	req.SetPathValue("id", deptID.String())
	rec := httptest.NewRecorder()
	h.GetPayrollConfig(rec, req)
	if rec.Code != http.StatusOK {
		t.Fatalf("expected 200, got %d", rec.Code)
	}

	// Set
	body := `{"std_working_days":"22.00","hours_per_day":"8.00","effective_date":"2026-01-01"}`
	req = departmentRequest(http.MethodPost, "/api/v1/departments/"+deptID.String()+"/payroll-config", body, userID)
	req.SetPathValue("id", deptID.String())
	rec = httptest.NewRecorder()
	h.SetPayrollConfig(rec, req)
	if rec.Code != http.StatusCreated {
		t.Fatalf("expected 201, got %d", rec.Code)
	}

	// Set on non-existent department returns 404
	hNotFound := DepartmentHandler{Repository: mockDepartmentRepo{notFound: true}}
	req = departmentRequest(http.MethodPost, "/api/v1/departments/"+deptID.String()+"/payroll-config", body, userID)
	req.SetPathValue("id", deptID.String())
	rec = httptest.NewRecorder()
	hNotFound.SetPayrollConfig(rec, req)
	if rec.Code != http.StatusNotFound {
		t.Fatalf("expected 404 for missing department, got %d", rec.Code)
	}
}
