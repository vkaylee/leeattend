package api

import (
	"context"
	"net/http"
	"net/http/httptest"
	"testing"
	"time"

	"github.com/google/uuid"
	"github.com/shopspring/decimal"
	httpinfra "github.com/vkaylee/leeattend/apiserver/internal/infra/http"
)

type mockPayrollRepository struct{}

func (mockPayrollRepository) ListWorkspacePayslips(context.Context, uuid.UUID, time.Time, time.Time) ([]PayslipRecord, error) {
	return []PayslipRecord{{ID: uuid.New(), NetPayAmount: decimal.RequireFromString("1234.50")}}, nil
}
func (mockPayrollRepository) ListEmployeePayslips(context.Context, uuid.UUID, uuid.UUID) ([]PayslipRecord, error) {
	return []PayslipRecord{{ID: uuid.New(), NetPayAmount: decimal.RequireFromString("1234.50")}}, nil
}

type payrollAuthorizer struct{}

func (payrollAuthorizer) IsSystemAdmin(context.Context, uuid.UUID) bool { return false }
func (payrollAuthorizer) HasPermission(context.Context, uuid.UUID, uuid.UUID, string) bool {
	return true
}

func TestPayrollListWorkspaceValidatesPeriod(t *testing.T) {
	handler := PayrollHandler{Repository: mockPayrollRepository{}, Authorizer: payrollAuthorizer{}}
	wsID := uuid.New()
	req := httptest.NewRequest(http.MethodGet, "/api/v1/workspaces/"+wsID.String()+"/payroll/payslips?year=2026&month=13", nil)
	req.SetPathValue("ws_id", wsID.String())
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), uuid.New()))
	rec := httptest.NewRecorder()
	handler.ListWorkspace(rec, req)
	if rec.Code != http.StatusBadRequest {
		t.Fatalf("expected 400, got %d", rec.Code)
	}
}

func TestPayrollListEmployeeReturnsDecimalAmount(t *testing.T) {
	handler := PayrollHandler{Repository: mockPayrollRepository{}, Authorizer: payrollAuthorizer{}}
	wsID, employeeID := uuid.New(), uuid.New()
	req := httptest.NewRequest(http.MethodGet, "/api/v1/workspaces/"+wsID.String()+"/employees/"+employeeID.String()+"/payslips", nil)
	req.SetPathValue("ws_id", wsID.String())
	req.SetPathValue("employee_id", employeeID.String())
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), uuid.New()))
	rec := httptest.NewRecorder()
	handler.ListEmployee(rec, req)
	if rec.Code != http.StatusOK {
		t.Fatalf("expected 200, got %d: %s", rec.Code, rec.Body.String())
	}
	if !contains(rec.Body.String(), "1234.5") {
		t.Fatalf("decimal amount missing: %s", rec.Body.String())
	}
}

func contains(value, fragment string) bool {
	for index := 0; index+len(fragment) <= len(value); index++ {
		if value[index:index+len(fragment)] == fragment {
			return true
		}
	}
	return false
}
