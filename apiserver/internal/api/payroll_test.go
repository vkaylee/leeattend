package api

import (
	"bytes"
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
func (mockPayrollRepository) ListSalaryComponents(context.Context, uuid.UUID) ([]SalaryComponentDefinition, error) {
	return []SalaryComponentDefinition{
		{
			ID:        uuid.MustParse("11111111-1111-1111-1111-111111111111"),
			Name:      "Base Allowance",
			Code:      "BASE_ALLOWANCE",
			CreatedAt: "2026-01-01T00:00:00Z",
			UpdatedAt: "2026-01-01T00:00:00Z",
		},
	}, nil
}
func (mockPayrollRepository) CreateSalaryComponent(_ context.Context, wsID uuid.UUID, req CreateSalaryComponentRequest) (SalaryComponentDefinition, error) {
	if req.Code == "CONFLICT" {
		return SalaryComponentDefinition{}, ErrComponentConflict
	}
	if req.BaseGroupIDs != nil && len(*req.BaseGroupIDs) > 0 && (*req.BaseGroupIDs)[0] == uuid.Nil {
		return SalaryComponentDefinition{}, ErrInvalidBaseGroup
	}
	return SalaryComponentDefinition{
		ID:          uuid.New(),
		WorkspaceID: wsID,
		Name:        req.Name,
		Code:        req.Code,
		CreatedAt:   "2026-01-01T00:00:00Z",
		UpdatedAt:   "2026-01-01T00:00:00Z",
	}, nil
}
func (mockPayrollRepository) ListEmploymentSalaryComponents(_ context.Context, wsID, employmentID uuid.UUID) ([]EmploymentSalaryComponentResponse, error) {
	return []EmploymentSalaryComponentResponse{
		{
			ID:                    uuid.New(),
			EmploymentID:          employmentID,
			ComponentDefinitionID: uuid.MustParse("11111111-1111-1111-1111-111111111111"),
			Amount:                decimal.RequireFromString("5000000.00"),
			EffectiveDate:         "2026-01-01",
			Name:                  "Base Allowance",
			Code:                  "BASE_ALLOWANCE",
		},
	}, nil
}
func (mockPayrollRepository) AssignEmploymentSalaryComponent(_ context.Context, wsID, employmentID uuid.UUID, req AssignSalaryComponentRequest) (EmployeeSalaryComponent, error) {
	if req.ComponentDefinitionID == uuid.MustParse("00000000-0000-0000-0000-000000000001") {
		return EmployeeSalaryComponent{}, ErrComponentNotFound
	}
	return EmployeeSalaryComponent{
		ID:                    uuid.New(),
		EmploymentID:          employmentID,
		ComponentDefinitionID: req.ComponentDefinitionID,
		Amount:                req.Amount,
		EffectiveDate:         req.EffectiveDate,
		CreatedAt:             "2026-01-01T00:00:00Z",
		UpdatedAt:             "2026-01-01T00:00:00Z",
	}, nil
}
func (mockPayrollRepository) ListSalaryBases(_ context.Context, wsID uuid.UUID) (SalaryBasesResponse, error) {
	sysID := uuid.MustParse("b0000000-0000-0000-0000-000000000001")
	return SalaryBasesResponse{
		Groups: []SalaryBaseGroup{
			{
				ID:        sysID,
				Name:      "Gross Salary",
				Slug:      "gross",
				IsSystem:  true,
				CreatedAt: "2026-01-01T00:00:00Z",
				UpdatedAt: "2026-01-01T00:00:00Z",
			},
		},
		Members: []SalaryBaseGroupMember{
			{
				ComponentDefinitionID: uuid.MustParse("11111111-1111-1111-1111-111111111111"),
				BaseGroupID:           sysID,
				CreatedAt:             "2026-01-01T00:00:00Z",
			},
		},
	}, nil
}
func (mockPayrollRepository) CreateSalaryBase(_ context.Context, wsID uuid.UUID, req CreateSalaryBaseRequest) (SalaryBaseGroup, error) {
	if req.Slug == "conflict" {
		return SalaryBaseGroup{}, ErrConflict
	}
	return SalaryBaseGroup{
		ID:          uuid.New(),
		WorkspaceID: &wsID,
		Name:        req.Name,
		Slug:        req.Slug,
		Description: req.Description,
		IsSystem:    false,
		CreatedAt:   "2026-01-01T00:00:00Z",
		UpdatedAt:   "2026-01-01T00:00:00Z",
	}, nil
}
func (mockPayrollRepository) AssignBaseMember(_ context.Context, wsID, baseID, componentID uuid.UUID) error {
	sysID := uuid.MustParse("b0000000-0000-0000-0000-000000000001")
	if baseID == sysID {
		return ErrInvalidBaseGroup
	}
	return nil
}
func (mockPayrollRepository) UnassignBaseMember(_ context.Context, wsID, baseID, componentID uuid.UUID) error {
	return nil
}

type payrollAuthorizer struct {
	allowedPerms map[string]bool
	isMember     bool
}

func (payrollAuthorizer) IsSystemAdmin(context.Context, uuid.UUID) bool { return false }
func (a payrollAuthorizer) HasPermission(_ context.Context, _ uuid.UUID, _ uuid.UUID, perm string) bool {
	if a.allowedPerms == nil {
		return true
	}
	return a.allowedPerms[perm]
}
func (a payrollAuthorizer) IsMember(_ context.Context, _ uuid.UUID, _ uuid.UUID) (bool, error) {
	return a.isMember, nil
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

func TestPayrollSalaryComponentsEndpoints(t *testing.T) {
	handler := PayrollHandler{Repository: mockPayrollRepository{}, Authorizer: payrollAuthorizer{}}
	wsID := uuid.New()

	// 1. GET /api/v1/workspaces/{ws_id}/salary-components
	req := httptest.NewRequest(http.MethodGet, "/api/v1/workspaces/"+wsID.String()+"/salary-components", nil)
	req.SetPathValue("ws_id", wsID.String())
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), uuid.New()))
	rec := httptest.NewRecorder()
	handler.ListSalaryComponents(rec, req)
	if rec.Code != http.StatusOK {
		t.Fatalf("expected 200, got %d: %s", rec.Code, rec.Body.String())
	}
	if !contains(rec.Body.String(), "BASE_ALLOWANCE") {
		t.Fatalf("expected component in body: %s", rec.Body.String())
	}

	// 2. POST /api/v1/workspaces/{ws_id}/salary-components (201 Created)
	body := `{"name":"Transport Allowance","code":"TRANSPORT"}`
	req = httptest.NewRequest(http.MethodPost, "/api/v1/workspaces/"+wsID.String()+"/salary-components", bytes.NewBufferString(body))
	req.SetPathValue("ws_id", wsID.String())
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), uuid.New()))
	rec = httptest.NewRecorder()
	handler.CreateSalaryComponent(rec, req)
	if rec.Code != http.StatusCreated {
		t.Fatalf("expected 201, got %d: %s", rec.Code, rec.Body.String())
	}
	if !contains(rec.Body.String(), "TRANSPORT") {
		t.Fatalf("expected created code in response: %s", rec.Body.String())
	}

	// Validation: empty name
	req = httptest.NewRequest(http.MethodPost, "/api/v1/workspaces/"+wsID.String()+"/salary-components", bytes.NewBufferString(`{"name":"","code":"TEST"}`))
	req.SetPathValue("ws_id", wsID.String())
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), uuid.New()))
	rec = httptest.NewRecorder()
	handler.CreateSalaryComponent(rec, req)
	if rec.Code != http.StatusBadRequest {
		t.Fatalf("expected 400 for empty name, got %d", rec.Code)
	}

	// Forbidden: manage permission check
	noPermHandler := PayrollHandler{
		Repository: mockPayrollRepository{},
		Authorizer: payrollAuthorizer{allowedPerms: map[string]bool{"payroll:read": true}, isMember: true},
	}
	req = httptest.NewRequest(http.MethodPost, "/api/v1/workspaces/"+wsID.String()+"/salary-components", bytes.NewBufferString(body))
	req.SetPathValue("ws_id", wsID.String())
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), uuid.New()))
	rec = httptest.NewRecorder()
	noPermHandler.CreateSalaryComponent(rec, req)
	if rec.Code != http.StatusForbidden {
		t.Fatalf("expected 403 without manage perm, got %d", rec.Code)
	}
}

func TestPayrollEmploymentSalaryComponentsEndpoints(t *testing.T) {
	handler := PayrollHandler{Repository: mockPayrollRepository{}, Authorizer: payrollAuthorizer{}}
	wsID := uuid.New()
	empID := uuid.New()

	// 3. GET /api/v1/workspaces/{ws_id}/employments/{id}/salary-components
	req := httptest.NewRequest(http.MethodGet, "/api/v1/workspaces/"+wsID.String()+"/employments/"+empID.String()+"/salary-components", nil)
	req.SetPathValue("ws_id", wsID.String())
	req.SetPathValue("id", empID.String())
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), uuid.New()))
	rec := httptest.NewRecorder()
	handler.ListEmploymentSalaryComponents(rec, req)
	if rec.Code != http.StatusOK {
		t.Fatalf("expected 200, got %d: %s", rec.Code, rec.Body.String())
	}
	if !contains(rec.Body.String(), "5000000") {
		t.Fatalf("expected decimal amount in response: %s", rec.Body.String())
	}

	// 4. POST /api/v1/workspaces/{ws_id}/employments/{id}/salary-components (201 Created)
	compID := uuid.MustParse("11111111-1111-1111-1111-111111111111")
	assignBody := `{"component_definition_id":"` + compID.String() + `","amount":"6500000.00","effective_date":"2026-02-01"}`
	req = httptest.NewRequest(http.MethodPost, "/api/v1/workspaces/"+wsID.String()+"/employments/"+empID.String()+"/salary-components", bytes.NewBufferString(assignBody))
	req.SetPathValue("ws_id", wsID.String())
	req.SetPathValue("id", empID.String())
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), uuid.New()))
	rec = httptest.NewRecorder()
	handler.AssignEmploymentSalaryComponent(rec, req)
	if rec.Code != http.StatusCreated {
		t.Fatalf("expected 201, got %d: %s", rec.Code, rec.Body.String())
	}
	if !contains(rec.Body.String(), "6500000") {
		t.Fatalf("expected assigned amount: %s", rec.Body.String())
	}

	// Validation: invalid date
	invalidDateBody := `{"component_definition_id":"` + compID.String() + `","amount":"6500000.00","effective_date":"2026/02/01"}`
	req = httptest.NewRequest(http.MethodPost, "/api/v1/workspaces/"+wsID.String()+"/employments/"+empID.String()+"/salary-components", bytes.NewBufferString(invalidDateBody))
	req.SetPathValue("ws_id", wsID.String())
	req.SetPathValue("id", empID.String())
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), uuid.New()))
	rec = httptest.NewRecorder()
	handler.AssignEmploymentSalaryComponent(rec, req)
	if rec.Code != http.StatusBadRequest {
		t.Fatalf("expected 400 for bad date format, got %d", rec.Code)
	}
}

func TestPayrollSalaryBasesEndpoints(t *testing.T) {
	handler := PayrollHandler{Repository: mockPayrollRepository{}, Authorizer: payrollAuthorizer{}}
	wsID := uuid.New()

	// 5. GET /api/v1/workspaces/{ws_id}/salary-bases (200 OK)
	req := httptest.NewRequest(http.MethodGet, "/api/v1/workspaces/"+wsID.String()+"/salary-bases", nil)
	req.SetPathValue("ws_id", wsID.String())
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), uuid.New()))
	rec := httptest.NewRecorder()
	handler.ListSalaryBases(rec, req)
	if rec.Code != http.StatusOK {
		t.Fatalf("expected 200, got %d: %s", rec.Code, rec.Body.String())
	}
	if !contains(rec.Body.String(), "Gross Salary") || !contains(rec.Body.String(), "gross") {
		t.Fatalf("expected system base in response: %s", rec.Body.String())
	}

	// 6. POST /api/v1/workspaces/{ws_id}/salary-bases (201 Created)
	createBody := `{"name":"Custom Base","slug":"custom_base","description":"Custom salary base"}`
	req = httptest.NewRequest(http.MethodPost, "/api/v1/workspaces/"+wsID.String()+"/salary-bases", bytes.NewBufferString(createBody))
	req.SetPathValue("ws_id", wsID.String())
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), uuid.New()))
	rec = httptest.NewRecorder()
	handler.CreateSalaryBase(rec, req)
	if rec.Code != http.StatusCreated {
		t.Fatalf("expected 201, got %d: %s", rec.Code, rec.Body.String())
	}
	if !contains(rec.Body.String(), "custom_base") {
		t.Fatalf("expected custom slug in response: %s", rec.Body.String())
	}

	// 7. POST /api/v1/workspaces/{ws_id}/salary-bases/{base_id}/members (200 OK)
	customBaseID := uuid.New()
	compID := uuid.MustParse("11111111-1111-1111-1111-111111111111")
	memberBody := `{"component_definition_id":"` + compID.String() + `"}`
	req = httptest.NewRequest(http.MethodPost, "/api/v1/workspaces/"+wsID.String()+"/salary-bases/"+customBaseID.String()+"/members", bytes.NewBufferString(memberBody))
	req.SetPathValue("ws_id", wsID.String())
	req.SetPathValue("base_id", customBaseID.String())
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), uuid.New()))
	rec = httptest.NewRecorder()
	handler.AssignBaseMember(rec, req)
	if rec.Code != http.StatusOK {
		t.Fatalf("expected 200, got %d: %s", rec.Code, rec.Body.String())
	}
	if !contains(rec.Body.String(), `"success":true`) {
		t.Fatalf("expected success:true, got %s", rec.Body.String())
	}

	// 7b. System base member assign rejection (403 Forbidden)
	sysBaseID := uuid.MustParse("b0000000-0000-0000-0000-000000000001")
	req = httptest.NewRequest(http.MethodPost, "/api/v1/workspaces/"+wsID.String()+"/salary-bases/"+sysBaseID.String()+"/members", bytes.NewBufferString(memberBody))
	req.SetPathValue("ws_id", wsID.String())
	req.SetPathValue("base_id", sysBaseID.String())
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), uuid.New()))
	rec = httptest.NewRecorder()
	handler.AssignBaseMember(rec, req)
	if rec.Code != http.StatusForbidden {
		t.Fatalf("expected 403 for system base assign, got %d: %s", rec.Code, rec.Body.String())
	}

	// 8. DELETE /api/v1/workspaces/{ws_id}/salary-bases/{base_id}/members/{component_id} (200 OK)
	req = httptest.NewRequest(http.MethodDelete, "/api/v1/workspaces/"+wsID.String()+"/salary-bases/"+customBaseID.String()+"/members/"+compID.String(), nil)
	req.SetPathValue("ws_id", wsID.String())
	req.SetPathValue("base_id", customBaseID.String())
	req.SetPathValue("component_id", compID.String())
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), uuid.New()))
	rec = httptest.NewRecorder()
	handler.UnassignBaseMember(rec, req)
	if rec.Code != http.StatusOK {
		t.Fatalf("expected 200, got %d: %s", rec.Code, rec.Body.String())
	}
	if !contains(rec.Body.String(), `"success":true`) {
		t.Fatalf("expected success:true, got %s", rec.Body.String())
	}
}
