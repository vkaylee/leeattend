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

type mockEmployeeDeptRepo struct {
	history  []EmployeeDepartmentHistory
	deptID   *uuid.UUID
	notFound bool
}

func (m mockEmployeeDeptRepo) GetEmployeeDepartmentHistory(_ context.Context, _ uuid.UUID) ([]EmployeeDepartmentHistory, error) {
	if m.notFound {
		return nil, ErrEmployeeNotFound
	}
	return m.history, nil
}

func (m mockEmployeeDeptRepo) TransferEmployee(_ context.Context, empID uuid.UUID, req TransferEmployeeRequest) (EmployeeDepartmentHistory, error) {
	if m.notFound {
		return EmployeeDepartmentHistory{}, ErrDepartmentNotFound
	}
	return EmployeeDepartmentHistory{
		ID:            uuid.New(),
		EmployeeID:    empID,
		DepartmentID:  req.DepartmentID,
		EffectiveDate: req.EffectiveDate,
		Notes:         req.Notes,
		CreatedAt:     "2026-03-01T00:00:00Z",
	}, nil
}

func (m mockEmployeeDeptRepo) CurrentEmployeeDepartment(_ context.Context, _ uuid.UUID) (*uuid.UUID, error) {
	return m.deptID, nil
}

func TestEmployeeDepartments_ListAndTransfer(t *testing.T) {
	empID, deptID, userID := uuid.New(), uuid.New(), uuid.New()
	h := EmployeeDepartmentHandler{
		Repository: mockEmployeeDeptRepo{
			history: []EmployeeDepartmentHistory{
				{ID: uuid.New(), EmployeeID: empID, DepartmentID: deptID, EffectiveDate: "2026-01-01", CreatedAt: "2026-01-01T00:00:00Z"},
			},
			deptID: &deptID,
		},
		Authorizer: employeeSliceAuthorizer{perms: map[string]bool{"departments:transfer": true}},
	}

	// 1. List History -> 200
	req := httptest.NewRequest(http.MethodGet, "/api/v1/employees/"+empID.String()+"/departments", nil)
	req.SetPathValue("id", empID.String())
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), userID))
	rec := httptest.NewRecorder()
	h.List(rec, req)
	if rec.Code != http.StatusOK {
		t.Fatalf("expected 200, got %d", rec.Code)
	}

	// 2. Transfer Success -> 200
	newDeptID := uuid.New()
	body := `{"department_id":"` + newDeptID.String() + `","effective_date":"2026-04-01","notes":"Promotion transfer"}`
	transferReq := httptest.NewRequest(http.MethodPost, "/api/v1/employees/"+empID.String()+"/departments", bytes.NewBufferString(body))
	transferReq.SetPathValue("id", empID.String())
	transferReq = transferReq.WithContext(httpinfra.WithUserIDForTest(transferReq.Context(), userID))
	recTransfer := httptest.NewRecorder()
	h.Transfer(recTransfer, transferReq)
	if recTransfer.Code != http.StatusOK {
		t.Fatalf("expected 200, got %d: %s", recTransfer.Code, recTransfer.Body.String())
	}

	// 3. Transfer Forbidden -> 403
	hForbidden := EmployeeDepartmentHandler{
		Repository: mockEmployeeDeptRepo{deptID: &deptID},
		Authorizer: employeeSliceAuthorizer{perms: map[string]bool{}},
	}
	recForb := httptest.NewRecorder()
	hForbidden.Transfer(recForb, transferReq)
	if recForb.Code != http.StatusForbidden {
		t.Fatalf("expected 403, got %d", recForb.Code)
	}

	// 4. Transfer Invalid Date -> 400
	badReq := httptest.NewRequest(http.MethodPost, "/api/v1/employees/"+empID.String()+"/departments", bytes.NewBufferString(`{"department_id":"`+newDeptID.String()+`","effective_date":"invalid"}`))
	badReq.SetPathValue("id", empID.String())
	badReq = badReq.WithContext(httpinfra.WithUserIDForTest(badReq.Context(), userID))
	recBad := httptest.NewRecorder()
	h.Transfer(recBad, badReq)
	if recBad.Code != http.StatusBadRequest {
		t.Fatalf("expected 400 for invalid date, got %d", recBad.Code)
	}
}
