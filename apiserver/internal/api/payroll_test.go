package api

import (
	"bytes"
	"context"
	"encoding/json"
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
func (mockPayrollRepository) GetPayrollSettings(_ context.Context, wsID uuid.UUID) (*PayrollSettings, error) {
	if wsID == uuid.MustParse("00000000-0000-0000-0000-000000000099") {
		return nil, ErrPayrollSettingsNotFound
	}
	return &PayrollSettings{
		WorkspaceID:        wsID,
		StdWorkingDayMode:  "fixed",
		FixedStdDays:       decimal.RequireFromString("26"),
		WorkingDaysPerWeek: 5,
		HoursPerDay:        decimal.RequireFromString("8"),
		EffectiveDate:      "2026-01-01",
		CreatedAt:          "2026-01-01T00:00:00Z",
		UpdatedAt:          "2026-01-01T00:00:00Z",
	}, nil
}
func (mockPayrollRepository) GetPayrollSettingsHistory(_ context.Context, wsID uuid.UUID) ([]PayrollSettings, error) {
	return []PayrollSettings{
		{
			WorkspaceID:        wsID,
			StdWorkingDayMode:  "fixed",
			FixedStdDays:       decimal.RequireFromString("26"),
			WorkingDaysPerWeek: 5,
			HoursPerDay:        decimal.RequireFromString("8"),
			EffectiveDate:      "2026-01-01",
			CreatedAt:          "2026-01-01T00:00:00Z",
			UpdatedAt:          "2026-01-01T00:00:00Z",
		},
	}, nil
}
func (mockPayrollRepository) UpdatePayrollSettings(_ context.Context, wsID uuid.UUID, req UpdatePayrollSettingsRequest) (PayrollSettings, error) {
	mode := "fixed"
	if req.StdWorkingDayMode != nil {
		mode = *req.StdWorkingDayMode
	}
	fixedDays := decimal.RequireFromString("26")
	if req.FixedStdDays != nil {
		fixedDays = *req.FixedStdDays
	}
	effDate := "2026-01-01"
	if req.EffectiveDate != nil {
		effDate = *req.EffectiveDate
	}
	return PayrollSettings{
		WorkspaceID:        wsID,
		StdWorkingDayMode:  mode,
		FixedStdDays:       fixedDays,
		WorkingDaysPerWeek: 5,
		HoursPerDay:        decimal.RequireFromString("8"),
		EffectiveDate:      effDate,
		CreatedAt:          "2026-01-01T00:00:00Z",
		UpdatedAt:          "2026-01-01T00:00:00Z",
	}, nil
}
func (mockPayrollRepository) GetEmployeePayrollConfigs(_ context.Context, wsID, employeeID uuid.UUID) ([]EmployeePayrollConfig, error) {
	if employeeID == uuid.MustParse("00000000-0000-0000-0000-000000000044") {
		return nil, ErrEmployeeNotFound
	}
	if employeeID == uuid.MustParse("00000000-0000-0000-0000-000000000040") {
		return nil, ErrNoActiveEmployment
	}
	return []EmployeePayrollConfig{
		{
			ID:             uuid.New(),
			EmploymentID:   uuid.New(),
			StdWorkingDays: decimal.RequireFromString("24"),
			HoursPerDay:    decimal.RequireFromString("8"),
			EffectiveDate:  "2026-01-01",
			CreatedAt:      "2026-01-01T00:00:00Z",
		},
	}, nil
}
func (mockPayrollRepository) SetEmployeePayrollConfig(_ context.Context, wsID, employeeID uuid.UUID, req SetEmployeePayrollConfigRequest, _ uuid.UUID) (EmployeePayrollConfig, error) {
	if employeeID == uuid.MustParse("00000000-0000-0000-0000-000000000044") {
		return EmployeePayrollConfig{}, ErrEmployeeNotFound
	}
	if employeeID == uuid.MustParse("00000000-0000-0000-0000-000000000040") {
		return EmployeePayrollConfig{}, ErrNoActiveEmployment
	}
	return EmployeePayrollConfig{
		ID:             uuid.New(),
		EmploymentID:   uuid.New(),
		StdWorkingDays: req.StdWorkingDays,
		HoursPerDay:    req.HoursPerDay,
		EffectiveDate:  req.EffectiveDate,
		CreatedAt:      "2026-01-01T00:00:00Z",
	}, nil
}
func (mockPayrollRepository) GetIncentivePolicies(_ context.Context, wsID uuid.UUID) ([]PayrollIncentivePolicyResponse, error) {
	return []PayrollIncentivePolicyResponse{
		{
			PayrollIncentivePolicy: PayrollIncentivePolicy{
				ID:            uuid.MustParse("22222222-2222-2222-2222-222222222222"),
				WorkspaceID:   wsID,
				Name:          "Sales Commission",
				BaseAmount:    decimal.RequireFromString("1000.00"),
				PayoutType:    "fixed",
				IsActive:      true,
				CreatedAt:     "2026-01-01T00:00:00Z",
				EffectiveFrom: "2026-01-01",
			},
			Tiers: []IncentiveTier{
				{
					ID:           uuid.New(),
					PolicyID:     uuid.MustParse("22222222-2222-2222-2222-222222222222"),
					Priority:     1,
					PayoutAmount: decimal.RequireFromString("500.00"),
					RuleConfig:   json.RawMessage(`{}`),
				},
			},
		},
	}, nil
}
func (mockPayrollRepository) CreateIncentivePolicy(_ context.Context, wsID uuid.UUID, req CreatePayrollIncentivePolicyRequest) (PayrollIncentivePolicy, error) {
	return PayrollIncentivePolicy{
		ID:            uuid.New(),
		WorkspaceID:   wsID,
		Name:          req.Name,
		BaseAmount:    req.BaseAmount,
		PayoutType:    req.PayoutType,
		IsActive:      true,
		CreatedAt:     "2026-01-01T00:00:00Z",
		EffectiveFrom: req.EffectiveFrom,
	}, nil
}
func (mockPayrollRepository) DeleteIncentivePolicy(_ context.Context, wsID, policyID uuid.UUID) error {
	if policyID == uuid.MustParse("00000000-0000-0000-0000-000000000099") {
		return ErrIncentivePolicyNotFound
	}
	return nil
}
func (mockPayrollRepository) CloneIncentivePolicy(_ context.Context, wsID, policyID uuid.UUID) (PayrollIncentivePolicy, error) {
	if policyID == uuid.MustParse("00000000-0000-0000-0000-000000000099") {
		return PayrollIncentivePolicy{}, ErrIncentivePolicyNotFound
	}
	return PayrollIncentivePolicy{
		ID:            uuid.New(),
		WorkspaceID:   wsID,
		Name:          "Sales Commission (Copy)",
		BaseAmount:    decimal.RequireFromString("1000.00"),
		PayoutType:    "fixed",
		IsActive:      false,
		CreatedAt:     "2026-01-01T00:00:00Z",
		EffectiveFrom: "2026-01-01",
	}, nil
}
func (mockPayrollRepository) ListPayrollRules(_ context.Context, wsID uuid.UUID) ([]PayrollRuleDefinition, error) {
	return []PayrollRuleDefinition{
		{
			ID:                uuid.MustParse("33333333-3333-3333-3333-333333333333"),
			WorkspaceID:       wsID,
			Name:              "Social Insurance",
			Code:              "BHXH",
			Category:          "statutory_deduction",
			AppliesTo:         "employee",
			CalculationMethod: "percentage",
			RuleConfig:        json.RawMessage(`{"rate":"8"}`),
			IsMandatory:       true,
			IsActive:          true,
			DisplayOrder:      1,
			CreatedAt:         "2026-01-01T00:00:00Z",
			UpdatedAt:         "2026-01-01T00:00:00Z",
		},
	}, nil
}

func (mockPayrollRepository) CreatePayrollRule(_ context.Context, wsID uuid.UUID, req CreatePayrollRuleDefinitionRequest) (PayrollRuleDefinition, error) {
	if req.Code == "CONFLICT" {
		return PayrollRuleDefinition{}, ErrConflict
	}
	return PayrollRuleDefinition{
		ID:                uuid.New(),
		WorkspaceID:       wsID,
		Name:              req.Name,
		Description:       req.Description,
		Code:              req.Code,
		Category:          req.Category,
		AppliesTo:         req.AppliesTo,
		CalculationMethod: req.CalculationMethod,
		RuleConfig:        req.RuleConfig,
		IsMandatory:       req.IsMandatory != nil && *req.IsMandatory,
		IsActive:          true,
		DisplayOrder:      0,
		CreatedAt:         "2026-01-01T00:00:00Z",
		UpdatedAt:         "2026-01-01T00:00:00Z",
	}, nil
}

func (mockPayrollRepository) ListPayrollRuleTiers(_ context.Context, wsID, ruleID uuid.UUID) ([]PayrollRuleTier, error) {
	if ruleID == uuid.MustParse("00000000-0000-0000-0000-000000000099") {
		return nil, ErrRuleNotFound
	}
	return []PayrollRuleTier{
		{
			ID:               uuid.New(),
			RuleDefinitionID: ruleID,
			TierOrder:        1,
			MinAmount:        decimal.Zero,
			Rate:             decimal.RequireFromString("5"),
			FixedAmount:      decimal.Zero,
			CreatedAt:        "2026-01-01T00:00:00Z",
		},
	}, nil
}

func (mockPayrollRepository) CreatePayrollRuleTier(_ context.Context, wsID, ruleID uuid.UUID, req CreatePayrollRuleTierRequest) (PayrollRuleTier, error) {
	if ruleID == uuid.MustParse("00000000-0000-0000-0000-000000000099") {
		return PayrollRuleTier{}, ErrRuleNotFound
	}
	fixed := decimal.Zero
	if req.FixedAmount != nil {
		fixed = *req.FixedAmount
	}
	return PayrollRuleTier{
		ID:               uuid.New(),
		RuleDefinitionID: ruleID,
		TierOrder:        req.TierOrder,
		MinAmount:        req.MinAmount,
		MaxAmount:        req.MaxAmount,
		Rate:             req.Rate,
		FixedAmount:      fixed,
		CreatedAt:        "2026-01-01T00:00:00Z",
	}, nil
}

func (mockPayrollRepository) DeletePayrollRuleTier(_ context.Context, wsID, ruleID, tierID uuid.UUID) error {
	if tierID == uuid.MustParse("00000000-0000-0000-0000-000000000099") {
		return ErrTierNotFound
	}
	return nil
}

func (mockPayrollRepository) ListPayrollRuleAssignments(_ context.Context, wsID, ruleID uuid.UUID) ([]PayrollRuleAssignment, error) {
	if ruleID == uuid.MustParse("00000000-0000-0000-0000-000000000099") {
		return nil, ErrRuleNotFound
	}
	return []PayrollRuleAssignment{
		{
			ID:               uuid.New(),
			WorkspaceID:      wsID,
			RuleDefinitionID: ruleID,
			AssigneeType:     "workspace",
			EffectiveFrom:    "2026-01-01",
			CreatedAt:        "2026-01-01T00:00:00Z",
		},
	}, nil
}

func (mockPayrollRepository) AssignPayrollRule(_ context.Context, wsID, ruleID uuid.UUID, req AssignPayrollRuleRequest) (PayrollRuleAssignment, error) {
	if ruleID == uuid.MustParse("00000000-0000-0000-0000-000000000099") {
		return PayrollRuleAssignment{}, ErrRuleNotFound
	}
	if req.AssigneeType == "workspace" && req.EffectiveFrom == "2026-09-09" {
		return PayrollRuleAssignment{}, ErrAssignmentConflict
	}
	return PayrollRuleAssignment{
		ID:               uuid.New(),
		WorkspaceID:      wsID,
		RuleDefinitionID: ruleID,
		AssigneeType:     req.AssigneeType,
		AssigneeID:       req.AssigneeID,
		EffectiveFrom:    req.EffectiveFrom,
		EffectiveTo:      req.EffectiveTo,
		OverrideConfig:   req.OverrideConfig,
		CreatedAt:        "2026-01-01T00:00:00Z",
	}, nil
}

func (mockPayrollRepository) DeletePayrollRuleAssignment(_ context.Context, wsID, ruleID, assignmentID uuid.UUID) error {
	if assignmentID == uuid.MustParse("00000000-0000-0000-0000-000000000099") {
		return ErrAssignmentNotFound
	}
	return nil
}

func (mockPayrollRepository) CalculateEmployeePayroll(_ context.Context, wsID, employeeID uuid.UUID, year, month int) (FullPayslipResponse, error) {
	if employeeID == uuid.MustParse("00000000-0000-0000-0000-000000000044") {
		return FullPayslipResponse{}, ErrEmployeeNotFound
	}
	if employeeID == uuid.MustParse("00000000-0000-0000-0000-000000000040") {
		return FullPayslipResponse{}, ErrNoActiveEmployment
	}
	payslipID := uuid.New()
	employmentID := uuid.New()
	return FullPayslipResponse{
		PayslipRecord: PayslipRecord{
			ID:                payslipID,
			WorkspaceID:       wsID,
			EmploymentID:      employmentID,
			PeriodStart:       "2026-01-01",
			PeriodEnd:         "2026-01-31",
			BasePayAmount:     decimal.RequireFromString("10000000.00"),
			OvertimePayAmount: decimal.RequireFromString("500000.00"),
			DeductionsAmount:  decimal.RequireFromString("1050000.00"),
			NetPayAmount:      decimal.RequireFromString("9450000.00"),
			Status:            "draft",
			StandardUnits:     176,
			OvertimeUnits:     10,
			CreatedAt:         "2026-01-31T00:00:00Z",
			UpdatedAt:         "2026-01-31T00:00:00Z",
		},
		Segments: []PayslipSegmentResponse{
			{
				ID:                 uuid.New(),
				PayslipID:          payslipID,
				StartDate:          "2026-01-01",
				EndDate:            "2026-01-31",
				AppliedBaseSalary:  decimal.RequireFromString("10000000.00"),
				AppliedStdDays:     decimal.RequireFromString("22"),
				AppliedHoursPerDay: decimal.RequireFromString("8"),
				ActualWorkDays:     decimal.RequireFromString("22"),
				PaidWorkDays:       decimal.RequireFromString("22"),
				Amount:             decimal.RequireFromString("10000000.00"),
				Reason:             "Standard",
			},
		},
		Incentives: []IncentiveEvaluationResult{},
		RuleEvaluations: []PayslipRuleEvaluationRecord{
			{
				ID:        uuid.New(),
				PayslipID: payslipID,
				RuleCode:  "BHXH",
				Amount:    decimal.RequireFromString("800000.00"),
				CreatedAt: "2026-01-31T00:00:00Z",
			},
		},
		SalaryComponents: []SalaryComponentWithDefinition{},
	}, nil
}

func (mockPayrollRepository) CalculateBatchPayroll(_ context.Context, wsID uuid.UUID, req BatchCalculatePayrollRequest) (BatchCalculatePayrollResponse, error) {
	results := make([]BatchCalculateResult, 0, len(req.EmploymentIDs))
	for _, id := range req.EmploymentIDs {
		if id == uuid.MustParse("00000000-0000-0000-0000-000000000099") {
			errMsg := "employment not found in workspace"
			results = append(results, BatchCalculateResult{
				EmploymentID: id,
				EmployeeID:   uuid.Nil,
				Status:       "error",
				ErrorMessage: &errMsg,
			})
			continue
		}
		results = append(results, BatchCalculateResult{
			EmploymentID: id,
			EmployeeID:   uuid.New(),
			Status:       "success",
			Payslip: &PayslipRecord{
				ID:           uuid.New(),
				WorkspaceID:  wsID,
				EmploymentID: id,
				PeriodStart:  "2026-01-01",
				PeriodEnd:    "2026-01-31",
				NetPayAmount: decimal.RequireFromString("9450000.00"),
				Status:       "draft",
			},
		})
	}
	return BatchCalculatePayrollResponse{Results: results}, nil
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

func TestPayrollSettingsEndpoints(t *testing.T) {
	handler := PayrollHandler{Repository: mockPayrollRepository{}, Authorizer: payrollAuthorizer{isMember: true}}
	wsID := uuid.New()

	// 1. GET /payroll-settings (200 OK)
	req := httptest.NewRequest(http.MethodGet, "/api/v1/workspaces/"+wsID.String()+"/payroll-settings", nil)
	req.SetPathValue("ws_id", wsID.String())
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), uuid.New()))
	rec := httptest.NewRecorder()
	handler.GetPayrollSettings(rec, req)
	if rec.Code != http.StatusOK {
		t.Fatalf("expected 200, got %d: %s", rec.Code, rec.Body.String())
	}
	if !contains(rec.Body.String(), `"std_working_day_mode":"fixed"`) || !contains(rec.Body.String(), `"fixed_std_days":"26"`) {
		t.Fatalf("unexpected settings body: %s", rec.Body.String())
	}

	// 1b. GET /payroll-settings (404 Not Found)
	missingWS := uuid.MustParse("00000000-0000-0000-0000-000000000099")
	req = httptest.NewRequest(http.MethodGet, "/api/v1/workspaces/"+missingWS.String()+"/payroll-settings", nil)
	req.SetPathValue("ws_id", missingWS.String())
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), uuid.New()))
	rec = httptest.NewRecorder()
	handler.GetPayrollSettings(rec, req)
	if rec.Code != http.StatusNotFound {
		t.Fatalf("expected 404, got %d: %s", rec.Code, rec.Body.String())
	}

	// 1c. GET /payroll-settings forbidden (403)
	forbiddenHandler := PayrollHandler{Repository: mockPayrollRepository{}, Authorizer: payrollAuthorizer{allowedPerms: map[string]bool{}, isMember: false}}
	req = httptest.NewRequest(http.MethodGet, "/api/v1/workspaces/"+wsID.String()+"/payroll-settings", nil)
	req.SetPathValue("ws_id", wsID.String())
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), uuid.New()))
	rec = httptest.NewRecorder()
	forbiddenHandler.GetPayrollSettings(rec, req)
	if rec.Code != http.StatusForbidden {
		t.Fatalf("expected 403, got %d: %s", rec.Code, rec.Body.String())
	}

	// 2. PATCH /payroll-settings (200 OK)
	patchBody := `{"std_working_day_mode":"custom","fixed_std_days":"25","effective_date":"2026-06-01"}`
	req = httptest.NewRequest(http.MethodPatch, "/api/v1/workspaces/"+wsID.String()+"/payroll-settings", bytes.NewBufferString(patchBody))
	req.SetPathValue("ws_id", wsID.String())
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), uuid.New()))
	rec = httptest.NewRecorder()
	handler.UpdatePayrollSettings(rec, req)
	if rec.Code != http.StatusOK {
		t.Fatalf("expected 200, got %d: %s", rec.Code, rec.Body.String())
	}
	if !contains(rec.Body.String(), `"std_working_day_mode":"custom"`) || !contains(rec.Body.String(), `"fixed_std_days":"25"`) {
		t.Fatalf("unexpected patch response: %s", rec.Body.String())
	}

	// 2b. PATCH /payroll-settings invalid date (400)
	badPatchBody := `{"effective_date":"not-a-date"}`
	req = httptest.NewRequest(http.MethodPatch, "/api/v1/workspaces/"+wsID.String()+"/payroll-settings", bytes.NewBufferString(badPatchBody))
	req.SetPathValue("ws_id", wsID.String())
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), uuid.New()))
	rec = httptest.NewRecorder()
	handler.UpdatePayrollSettings(rec, req)
	if rec.Code != http.StatusBadRequest {
		t.Fatalf("expected 400 for bad date, got %d: %s", rec.Code, rec.Body.String())
	}

	// 3. GET /payroll-settings/history (200 OK)
	req = httptest.NewRequest(http.MethodGet, "/api/v1/workspaces/"+wsID.String()+"/payroll-settings/history", nil)
	req.SetPathValue("ws_id", wsID.String())
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), uuid.New()))
	rec = httptest.NewRecorder()
	handler.GetPayrollSettingsHistory(rec, req)
	if rec.Code != http.StatusOK {
		t.Fatalf("expected 200, got %d: %s", rec.Code, rec.Body.String())
	}
	if !contains(rec.Body.String(), `"effective_date":"2026-01-01"`) {
		t.Fatalf("expected history entries, got: %s", rec.Body.String())
	}
}

func TestEmployeePayrollConfigEndpoints(t *testing.T) {
	handler := PayrollHandler{Repository: mockPayrollRepository{}, Authorizer: payrollAuthorizer{isMember: true}}
	wsID := uuid.New()
	validEmpID := uuid.New()
	missingEmpID := uuid.MustParse("00000000-0000-0000-0000-000000000044")
	noActiveEmpID := uuid.MustParse("00000000-0000-0000-0000-000000000040")

	// 4. GET /employees/{id}/payroll-config (200 OK)
	req := httptest.NewRequest(http.MethodGet, "/api/v1/workspaces/"+wsID.String()+"/employees/"+validEmpID.String()+"/payroll-config", nil)
	req.SetPathValue("ws_id", wsID.String())
	req.SetPathValue("id", validEmpID.String())
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), uuid.New()))
	rec := httptest.NewRecorder()
	handler.GetEmployeePayrollConfig(rec, req)
	if rec.Code != http.StatusOK {
		t.Fatalf("expected 200, got %d: %s", rec.Code, rec.Body.String())
	}
	if !contains(rec.Body.String(), `"std_working_days":"24"`) {
		t.Fatalf("unexpected employee config body: %s", rec.Body.String())
	}

	// 4b. GET /employees/{id}/payroll-config employee not found (404)
	req = httptest.NewRequest(http.MethodGet, "/api/v1/workspaces/"+wsID.String()+"/employees/"+missingEmpID.String()+"/payroll-config", nil)
	req.SetPathValue("ws_id", wsID.String())
	req.SetPathValue("id", missingEmpID.String())
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), uuid.New()))
	rec = httptest.NewRecorder()
	handler.GetEmployeePayrollConfig(rec, req)
	if rec.Code != http.StatusNotFound {
		t.Fatalf("expected 404, got %d: %s", rec.Code, rec.Body.String())
	}

	// 4c. GET /employees/{id}/payroll-config no active employment (400)
	req = httptest.NewRequest(http.MethodGet, "/api/v1/workspaces/"+wsID.String()+"/employees/"+noActiveEmpID.String()+"/payroll-config", nil)
	req.SetPathValue("ws_id", wsID.String())
	req.SetPathValue("id", noActiveEmpID.String())
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), uuid.New()))
	rec = httptest.NewRecorder()
	handler.GetEmployeePayrollConfig(rec, req)
	if rec.Code != http.StatusBadRequest {
		t.Fatalf("expected 400, got %d: %s", rec.Code, rec.Body.String())
	}

	// 5. POST /employees/{id}/payroll-config (201 Created)
	createBody := `{"std_working_days":"22","hours_per_day":"8","effective_date":"2026-06-01"}`
	req = httptest.NewRequest(http.MethodPost, "/api/v1/workspaces/"+wsID.String()+"/employees/"+validEmpID.String()+"/payroll-config", bytes.NewBufferString(createBody))
	req.SetPathValue("ws_id", wsID.String())
	req.SetPathValue("id", validEmpID.String())
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), uuid.New()))
	rec = httptest.NewRecorder()
	handler.SetEmployeePayrollConfig(rec, req)
	if rec.Code != http.StatusCreated {
		t.Fatalf("expected 201, got %d: %s", rec.Code, rec.Body.String())
	}
	if !contains(rec.Body.String(), `"std_working_days":"22"`) {
		t.Fatalf("unexpected set config response: %s", rec.Body.String())
	}

	// 5b. POST /employees/{id}/payroll-config bad date (400)
	badBody := `{"std_working_days":"22","hours_per_day":"8","effective_date":"invalid"}`
	req = httptest.NewRequest(http.MethodPost, "/api/v1/workspaces/"+wsID.String()+"/employees/"+validEmpID.String()+"/payroll-config", bytes.NewBufferString(badBody))
	req.SetPathValue("ws_id", wsID.String())
	req.SetPathValue("id", validEmpID.String())
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), uuid.New()))
	rec = httptest.NewRecorder()
	handler.SetEmployeePayrollConfig(rec, req)
	if rec.Code != http.StatusBadRequest {
		t.Fatalf("expected 400, got %d: %s", rec.Code, rec.Body.String())
	}

	// 5c. POST /employees/{id}/payroll-config employee not found (404)
	req = httptest.NewRequest(http.MethodPost, "/api/v1/workspaces/"+wsID.String()+"/employees/"+missingEmpID.String()+"/payroll-config", bytes.NewBufferString(createBody))
	req.SetPathValue("ws_id", wsID.String())
	req.SetPathValue("id", missingEmpID.String())
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), uuid.New()))
	rec = httptest.NewRecorder()
	handler.SetEmployeePayrollConfig(rec, req)
	if rec.Code != http.StatusNotFound {
		t.Fatalf("expected 404, got %d: %s", rec.Code, rec.Body.String())
	}

	// 5d. POST /employees/{id}/payroll-config no active employment (400)
	req = httptest.NewRequest(http.MethodPost, "/api/v1/workspaces/"+wsID.String()+"/employees/"+noActiveEmpID.String()+"/payroll-config", bytes.NewBufferString(createBody))
	req.SetPathValue("ws_id", wsID.String())
	req.SetPathValue("id", noActiveEmpID.String())
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), uuid.New()))
	rec = httptest.NewRecorder()
	handler.SetEmployeePayrollConfig(rec, req)
	if rec.Code != http.StatusBadRequest {
		t.Fatalf("expected 400, got %d: %s", rec.Code, rec.Body.String())
	}
}

func TestIncentivePoliciesEndpoints(t *testing.T) {
	handler := PayrollHandler{Repository: mockPayrollRepository{}, Authorizer: payrollAuthorizer{isMember: true}}
	wsID := uuid.New()
	validPolicyID := uuid.MustParse("22222222-2222-2222-2222-222222222222")
	missingPolicyID := uuid.MustParse("00000000-0000-0000-0000-000000000099")

	// 6. GET /payroll/incentive-policies (200 OK)
	req := httptest.NewRequest(http.MethodGet, "/api/v1/workspaces/"+wsID.String()+"/payroll/incentive-policies", nil)
	req.SetPathValue("ws_id", wsID.String())
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), uuid.New()))
	rec := httptest.NewRecorder()
	handler.GetIncentivePolicies(rec, req)
	if rec.Code != http.StatusOK {
		t.Fatalf("expected 200, got %d: %s", rec.Code, rec.Body.String())
	}
	if !contains(rec.Body.String(), "Sales Commission") || !contains(rec.Body.String(), "tiers") {
		t.Fatalf("unexpected incentive policies body: %s", rec.Body.String())
	}

	// 7. POST /payroll/incentive-policies (201 Created)
	createBody := `{"name":"Project Bonus","base_amount":"2000.00","payout_type":"fixed","effective_from":"2026-06-01","tiers":[{"priority":1,"payout_amount":"1000.00","rule_config":{}}]}`
	req = httptest.NewRequest(http.MethodPost, "/api/v1/workspaces/"+wsID.String()+"/payroll/incentive-policies", bytes.NewBufferString(createBody))
	req.SetPathValue("ws_id", wsID.String())
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), uuid.New()))
	rec = httptest.NewRecorder()
	handler.CreateIncentivePolicy(rec, req)
	if rec.Code != http.StatusCreated {
		t.Fatalf("expected 201, got %d: %s", rec.Code, rec.Body.String())
	}
	if !contains(rec.Body.String(), "Project Bonus") {
		t.Fatalf("unexpected create response: %s", rec.Body.String())
	}

	// 7b. POST /payroll/incentive-policies invalid (400)
	badBody := `{"name":"","base_amount":"2000.00","effective_from":"2026-06-01"}`
	req = httptest.NewRequest(http.MethodPost, "/api/v1/workspaces/"+wsID.String()+"/payroll/incentive-policies", bytes.NewBufferString(badBody))
	req.SetPathValue("ws_id", wsID.String())
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), uuid.New()))
	rec = httptest.NewRecorder()
	handler.CreateIncentivePolicy(rec, req)
	if rec.Code != http.StatusBadRequest {
		t.Fatalf("expected 400 for empty name, got %d: %s", rec.Code, rec.Body.String())
	}

	// 8. DELETE /payroll/incentive-policies/{policy_id} (204 No Content)
	req = httptest.NewRequest(http.MethodDelete, "/api/v1/workspaces/"+wsID.String()+"/payroll/incentive-policies/"+validPolicyID.String(), nil)
	req.SetPathValue("ws_id", wsID.String())
	req.SetPathValue("policy_id", validPolicyID.String())
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), uuid.New()))
	rec = httptest.NewRecorder()
	handler.DeleteIncentivePolicy(rec, req)
	if rec.Code != http.StatusNoContent {
		t.Fatalf("expected 204, got %d: %s", rec.Code, rec.Body.String())
	}

	// 9. POST /payroll/incentive-policies/{policy_id}/clone (201 Created)
	req = httptest.NewRequest(http.MethodPost, "/api/v1/workspaces/"+wsID.String()+"/payroll/incentive-policies/"+validPolicyID.String()+"/clone", nil)
	req.SetPathValue("ws_id", wsID.String())
	req.SetPathValue("policy_id", validPolicyID.String())
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), uuid.New()))
	rec = httptest.NewRecorder()
	handler.CloneIncentivePolicy(rec, req)
	if rec.Code != http.StatusCreated {
		t.Fatalf("expected 201, got %d: %s", rec.Code, rec.Body.String())
	}
	if !contains(rec.Body.String(), "Sales Commission (Copy)") || !contains(rec.Body.String(), `"is_active":false`) {
		t.Fatalf("unexpected clone response: %s", rec.Body.String())
	}

	// 9b. POST /payroll/incentive-policies/{policy_id}/clone not found (404)
	req = httptest.NewRequest(http.MethodPost, "/api/v1/workspaces/"+wsID.String()+"/payroll/incentive-policies/"+missingPolicyID.String()+"/clone", nil)
	req.SetPathValue("ws_id", wsID.String())
	req.SetPathValue("policy_id", missingPolicyID.String())
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), uuid.New()))
	rec = httptest.NewRecorder()
	handler.CloneIncentivePolicy(rec, req)
	if rec.Code != http.StatusNotFound {
		t.Fatalf("expected 404, got %d: %s", rec.Code, rec.Body.String())
	}
}
func TestPayrollRulesEndpoints(t *testing.T) {
	handler := PayrollHandler{Repository: mockPayrollRepository{}, Authorizer: payrollAuthorizer{isMember: true}}
	wsID := uuid.New()

	// 1. GET /payroll-rules (200 OK)
	req := httptest.NewRequest(http.MethodGet, "/api/v1/workspaces/"+wsID.String()+"/payroll-rules", nil)
	req.SetPathValue("ws_id", wsID.String())
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), uuid.New()))
	rec := httptest.NewRecorder()
	handler.ListPayrollRules(rec, req)
	if rec.Code != http.StatusOK {
		t.Fatalf("expected 200, got %d: %s", rec.Code, rec.Body.String())
	}
	if !contains(rec.Body.String(), "Social Insurance") || !contains(rec.Body.String(), "BHXH") {
		t.Fatalf("unexpected list rules response: %s", rec.Body.String())
	}

	// 1b. GET /payroll-rules forbidden (403)
	forbiddenHandler := PayrollHandler{Repository: mockPayrollRepository{}, Authorizer: payrollAuthorizer{isMember: false, allowedPerms: map[string]bool{}}}
	req = httptest.NewRequest(http.MethodGet, "/api/v1/workspaces/"+wsID.String()+"/payroll-rules", nil)
	req.SetPathValue("ws_id", wsID.String())
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), uuid.New()))
	rec = httptest.NewRecorder()
	forbiddenHandler.ListPayrollRules(rec, req)
	if rec.Code != http.StatusForbidden {
		t.Fatalf("expected 403, got %d", rec.Code)
	}

	// 2. POST /payroll-rules (201 Created)
	body := `{"name":"Health Insurance","code":"BHYT","category":"statutory_deduction","applies_to":"employee","calculation_method":"percentage","rule_config":{"rate":"1.5"}}`
	req = httptest.NewRequest(http.MethodPost, "/api/v1/workspaces/"+wsID.String()+"/payroll-rules", bytes.NewBufferString(body))
	req.SetPathValue("ws_id", wsID.String())
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), uuid.New()))
	rec = httptest.NewRecorder()
	handler.CreatePayrollRule(rec, req)
	if rec.Code != http.StatusCreated {
		t.Fatalf("expected 201, got %d: %s", rec.Code, rec.Body.String())
	}

	// 2b. POST /payroll-rules validation error (400)
	req = httptest.NewRequest(http.MethodPost, "/api/v1/workspaces/"+wsID.String()+"/payroll-rules", bytes.NewBufferString(`{"name":""}`))
	req.SetPathValue("ws_id", wsID.String())
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), uuid.New()))
	rec = httptest.NewRecorder()
	handler.CreatePayrollRule(rec, req)
	if rec.Code != http.StatusBadRequest {
		t.Fatalf("expected 400, got %d", rec.Code)
	}

	// 2c. POST /payroll-rules conflict (409)
	req = httptest.NewRequest(http.MethodPost, "/api/v1/workspaces/"+wsID.String()+"/payroll-rules", bytes.NewBufferString(`{"name":"Dup","code":"CONFLICT"}`))
	req.SetPathValue("ws_id", wsID.String())
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), uuid.New()))
	rec = httptest.NewRecorder()
	handler.CreatePayrollRule(rec, req)
	if rec.Code != http.StatusConflict {
		t.Fatalf("expected 409, got %d", rec.Code)
	}
}

func TestPayrollRuleTiersEndpoints(t *testing.T) {
	handler := PayrollHandler{Repository: mockPayrollRepository{}, Authorizer: payrollAuthorizer{isMember: true}}
	wsID := uuid.New()
	ruleID := uuid.New()
	missingRuleID := uuid.MustParse("00000000-0000-0000-0000-000000000099")

	// 3. GET /payroll-rules/{rule_id}/tiers (200 OK)
	req := httptest.NewRequest(http.MethodGet, "/api/v1/workspaces/"+wsID.String()+"/payroll-rules/"+ruleID.String()+"/tiers", nil)
	req.SetPathValue("ws_id", wsID.String())
	req.SetPathValue("rule_id", ruleID.String())
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), uuid.New()))
	rec := httptest.NewRecorder()
	handler.ListPayrollRuleTiers(rec, req)
	if rec.Code != http.StatusOK {
		t.Fatalf("expected 200, got %d: %s", rec.Code, rec.Body.String())
	}

	// 3b. GET /payroll-rules/{rule_id}/tiers not found (404)
	req = httptest.NewRequest(http.MethodGet, "/api/v1/workspaces/"+wsID.String()+"/payroll-rules/"+missingRuleID.String()+"/tiers", nil)
	req.SetPathValue("ws_id", wsID.String())
	req.SetPathValue("rule_id", missingRuleID.String())
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), uuid.New()))
	rec = httptest.NewRecorder()
	handler.ListPayrollRuleTiers(rec, req)
	if rec.Code != http.StatusNotFound {
		t.Fatalf("expected 404, got %d", rec.Code)
	}

	// 4. POST /payroll-rules/{rule_id}/tiers (201 Created)
	body := `{"tier_order":1,"min_amount":0,"max_amount":5000000,"rate":5,"fixed_amount":0}`
	req = httptest.NewRequest(http.MethodPost, "/api/v1/workspaces/"+wsID.String()+"/payroll-rules/"+ruleID.String()+"/tiers", bytes.NewBufferString(body))
	req.SetPathValue("ws_id", wsID.String())
	req.SetPathValue("rule_id", ruleID.String())
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), uuid.New()))
	rec = httptest.NewRecorder()
	handler.CreatePayrollRuleTier(rec, req)
	if rec.Code != http.StatusCreated {
		t.Fatalf("expected 201, got %d: %s", rec.Code, rec.Body.String())
	}

	// 4b. POST /payroll-rules/{rule_id}/tiers validation: max_amount <= min_amount (400)
	body = `{"tier_order":1,"min_amount":5000000,"max_amount":5000000,"rate":5}`
	req = httptest.NewRequest(http.MethodPost, "/api/v1/workspaces/"+wsID.String()+"/payroll-rules/"+ruleID.String()+"/tiers", bytes.NewBufferString(body))
	req.SetPathValue("ws_id", wsID.String())
	req.SetPathValue("rule_id", ruleID.String())
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), uuid.New()))
	rec = httptest.NewRecorder()
	handler.CreatePayrollRuleTier(rec, req)
	if rec.Code != http.StatusBadRequest {
		t.Fatalf("expected 400, got %d", rec.Code)
	}

	// 5. DELETE /payroll-rules/{rule_id}/tiers/{tier_id} (200 OK)
	tierID := uuid.New()
	req = httptest.NewRequest(http.MethodDelete, "/api/v1/workspaces/"+wsID.String()+"/payroll-rules/"+ruleID.String()+"/tiers/"+tierID.String(), nil)
	req.SetPathValue("ws_id", wsID.String())
	req.SetPathValue("rule_id", ruleID.String())
	req.SetPathValue("tier_id", tierID.String())
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), uuid.New()))
	rec = httptest.NewRecorder()
	handler.DeletePayrollRuleTier(rec, req)
	if rec.Code != http.StatusOK {
		t.Fatalf("expected 200, got %d: %s", rec.Code, rec.Body.String())
	}

	// 5b. DELETE tier not found (404)
	missingTierID := uuid.MustParse("00000000-0000-0000-0000-000000000099")
	req = httptest.NewRequest(http.MethodDelete, "/api/v1/workspaces/"+wsID.String()+"/payroll-rules/"+ruleID.String()+"/tiers/"+missingTierID.String(), nil)
	req.SetPathValue("ws_id", wsID.String())
	req.SetPathValue("rule_id", ruleID.String())
	req.SetPathValue("tier_id", missingTierID.String())
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), uuid.New()))
	rec = httptest.NewRecorder()
	handler.DeletePayrollRuleTier(rec, req)
	if rec.Code != http.StatusNotFound {
		t.Fatalf("expected 404, got %d", rec.Code)
	}
}

func TestPayrollRuleAssignmentsEndpoints(t *testing.T) {
	handler := PayrollHandler{Repository: mockPayrollRepository{}, Authorizer: payrollAuthorizer{isMember: true}}
	wsID := uuid.New()
	ruleID := uuid.New()
	missingRuleID := uuid.MustParse("00000000-0000-0000-0000-000000000099")

	// 6. GET /payroll-rules/{rule_id}/assignments (200 OK)
	req := httptest.NewRequest(http.MethodGet, "/api/v1/workspaces/"+wsID.String()+"/payroll-rules/"+ruleID.String()+"/assignments", nil)
	req.SetPathValue("ws_id", wsID.String())
	req.SetPathValue("rule_id", ruleID.String())
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), uuid.New()))
	rec := httptest.NewRecorder()
	handler.ListPayrollRuleAssignments(rec, req)
	if rec.Code != http.StatusOK {
		t.Fatalf("expected 200, got %d: %s", rec.Code, rec.Body.String())
	}

	// 6b. GET /payroll-rules/{rule_id}/assignments not found (404)
	req = httptest.NewRequest(http.MethodGet, "/api/v1/workspaces/"+wsID.String()+"/payroll-rules/"+missingRuleID.String()+"/assignments", nil)
	req.SetPathValue("ws_id", wsID.String())
	req.SetPathValue("rule_id", missingRuleID.String())
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), uuid.New()))
	rec = httptest.NewRecorder()
	handler.ListPayrollRuleAssignments(rec, req)
	if rec.Code != http.StatusNotFound {
		t.Fatalf("expected 404, got %d", rec.Code)
	}

	// 7. POST /payroll-rules/{rule_id}/assignments (201 Created)
	body := `{"assignee_type":"workspace","effective_from":"2026-01-01"}`
	req = httptest.NewRequest(http.MethodPost, "/api/v1/workspaces/"+wsID.String()+"/payroll-rules/"+ruleID.String()+"/assignments", bytes.NewBufferString(body))
	req.SetPathValue("ws_id", wsID.String())
	req.SetPathValue("rule_id", ruleID.String())
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), uuid.New()))
	rec = httptest.NewRecorder()
	handler.AssignPayrollRule(rec, req)
	if rec.Code != http.StatusCreated {
		t.Fatalf("expected 201, got %d: %s", rec.Code, rec.Body.String())
	}
	// 7a. POST /payroll-rules/{rule_id}/assignments with user assignee_type (201 Created)
	targetUserID := uuid.New()
	body = `{"assignee_type":"user","assignee_id":"` + targetUserID.String() + `","effective_from":"2026-01-01"}`
	req = httptest.NewRequest(http.MethodPost, "/api/v1/workspaces/"+wsID.String()+"/payroll-rules/"+ruleID.String()+"/assignments", bytes.NewBufferString(body))
	req.SetPathValue("ws_id", wsID.String())
	req.SetPathValue("rule_id", ruleID.String())
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), uuid.New()))
	rec = httptest.NewRecorder()
	handler.AssignPayrollRule(rec, req)
	if rec.Code != http.StatusCreated {
		t.Fatalf("expected 201 for user assignee_type, got %d: %s", rec.Code, rec.Body.String())
	}

	// 7b. POST /payroll-rules/{rule_id}/assignments validation error: invalid assignee_type (400)
	body = `{"assignee_type":"unknown","effective_from":"2026-01-01"}`
	req = httptest.NewRequest(http.MethodPost, "/api/v1/workspaces/"+wsID.String()+"/payroll-rules/"+ruleID.String()+"/assignments", bytes.NewBufferString(body))
	req.SetPathValue("ws_id", wsID.String())
	req.SetPathValue("rule_id", ruleID.String())
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), uuid.New()))
	rec = httptest.NewRecorder()
	handler.AssignPayrollRule(rec, req)
	if rec.Code != http.StatusBadRequest {
		t.Fatalf("expected 400, got %d", rec.Code)
	}

	// 7c. POST /payroll-rules/{rule_id}/assignments conflict (409)
	body = `{"assignee_type":"workspace","effective_from":"2026-09-09"}`
	req = httptest.NewRequest(http.MethodPost, "/api/v1/workspaces/"+wsID.String()+"/payroll-rules/"+ruleID.String()+"/assignments", bytes.NewBufferString(body))
	req.SetPathValue("ws_id", wsID.String())
	req.SetPathValue("rule_id", ruleID.String())
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), uuid.New()))
	rec = httptest.NewRecorder()
	handler.AssignPayrollRule(rec, req)
	if rec.Code != http.StatusConflict {
		t.Fatalf("expected 409, got %d", rec.Code)
	}

	// 8. DELETE /payroll-rules/{rule_id}/assignments/{assignment_id} (200 OK)
	assignID := uuid.New()
	req = httptest.NewRequest(http.MethodDelete, "/api/v1/workspaces/"+wsID.String()+"/payroll-rules/"+ruleID.String()+"/assignments/"+assignID.String(), nil)
	req.SetPathValue("ws_id", wsID.String())
	req.SetPathValue("rule_id", ruleID.String())
	req.SetPathValue("assignment_id", assignID.String())
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), uuid.New()))
	rec = httptest.NewRecorder()
	handler.DeletePayrollRuleAssignment(rec, req)
	if rec.Code != http.StatusOK {
		t.Fatalf("expected 200, got %d: %s", rec.Code, rec.Body.String())
	}

	// 8b. DELETE assignment not found (404)
	missingAssignID := uuid.MustParse("00000000-0000-0000-0000-000000000099")
	req = httptest.NewRequest(http.MethodDelete, "/api/v1/workspaces/"+wsID.String()+"/payroll-rules/"+ruleID.String()+"/assignments/"+missingAssignID.String(), nil)
	req.SetPathValue("ws_id", wsID.String())
	req.SetPathValue("rule_id", ruleID.String())
	req.SetPathValue("assignment_id", missingAssignID.String())
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), uuid.New()))
	rec = httptest.NewRecorder()
	handler.DeletePayrollRuleAssignment(rec, req)
	if rec.Code != http.StatusNotFound {
		t.Fatalf("expected 404, got %d", rec.Code)
	}
}

func TestPayrollCalculationEndpoints(t *testing.T) {
	calcAuthorizer := payrollAuthorizer{
		isMember:     true,
		allowedPerms: map[string]bool{"payroll:calculate": true},
	}
	handler := PayrollHandler{Repository: mockPayrollRepository{}, Authorizer: calcAuthorizer}
	wsID := uuid.New()
	empID := uuid.New()

	// 9. POST /employees/{id}/calculate-payroll (200 OK)
	req := httptest.NewRequest(http.MethodPost, "/api/v1/workspaces/"+wsID.String()+"/employees/"+empID.String()+"/calculate-payroll?year=2026&month=1", nil)
	req.SetPathValue("ws_id", wsID.String())
	req.SetPathValue("id", empID.String())
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), uuid.New()))
	rec := httptest.NewRecorder()
	handler.CalculatePayroll(rec, req)
	if rec.Code != http.StatusOK {
		t.Fatalf("expected 200, got %d: %s", rec.Code, rec.Body.String())
	}
	if !contains(rec.Body.String(), "10000000") || !contains(rec.Body.String(), "BHXH") {
		t.Fatalf("unexpected calculate response: %s", rec.Body.String())
	}

	// 9b. POST calculate-payroll forbidden without permission (403)
	noPermHandler := PayrollHandler{Repository: mockPayrollRepository{}, Authorizer: payrollAuthorizer{isMember: true, allowedPerms: map[string]bool{}}}
	req = httptest.NewRequest(http.MethodPost, "/api/v1/workspaces/"+wsID.String()+"/employees/"+empID.String()+"/calculate-payroll?year=2026&month=1", nil)
	req.SetPathValue("ws_id", wsID.String())
	req.SetPathValue("id", empID.String())
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), uuid.New()))
	rec = httptest.NewRecorder()
	noPermHandler.CalculatePayroll(rec, req)
	if rec.Code != http.StatusForbidden {
		t.Fatalf("expected 403, got %d", rec.Code)
	}

	// 9c. POST calculate-payroll invalid month (400)
	req = httptest.NewRequest(http.MethodPost, "/api/v1/workspaces/"+wsID.String()+"/employees/"+empID.String()+"/calculate-payroll?year=2026&month=13", nil)
	req.SetPathValue("ws_id", wsID.String())
	req.SetPathValue("id", empID.String())
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), uuid.New()))
	rec = httptest.NewRecorder()
	handler.CalculatePayroll(rec, req)
	if rec.Code != http.StatusBadRequest {
		t.Fatalf("expected 400, got %d", rec.Code)
	}

	// 9d. POST calculate-payroll missing employee (404)
	missingEmpID := uuid.MustParse("00000000-0000-0000-0000-000000000044")
	req = httptest.NewRequest(http.MethodPost, "/api/v1/workspaces/"+wsID.String()+"/employees/"+missingEmpID.String()+"/calculate-payroll?year=2026&month=1", nil)
	req.SetPathValue("ws_id", wsID.String())
	req.SetPathValue("id", missingEmpID.String())
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), uuid.New()))
	rec = httptest.NewRecorder()
	handler.CalculatePayroll(rec, req)
	if rec.Code != http.StatusNotFound {
		t.Fatalf("expected 404, got %d", rec.Code)
	}

	// 9e. POST calculate-payroll no active employment (400)
	noActiveEmpID := uuid.MustParse("00000000-0000-0000-0000-000000000040")
	req = httptest.NewRequest(http.MethodPost, "/api/v1/workspaces/"+wsID.String()+"/employees/"+noActiveEmpID.String()+"/calculate-payroll?year=2026&month=1", nil)
	req.SetPathValue("ws_id", wsID.String())
	req.SetPathValue("id", noActiveEmpID.String())
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), uuid.New()))
	rec = httptest.NewRecorder()
	handler.CalculatePayroll(rec, req)
	if rec.Code != http.StatusBadRequest {
		t.Fatalf("expected 400, got %d", rec.Code)
	}

	// 10. POST /payroll/calculate-batch (200 OK)
	validEmp1 := uuid.New()
	missingEmp := uuid.MustParse("00000000-0000-0000-0000-000000000099")
	batchBody := `{"employment_ids":["` + validEmp1.String() + `","` + missingEmp.String() + `"],"year":2026,"month":1}`
	req = httptest.NewRequest(http.MethodPost, "/api/v1/workspaces/"+wsID.String()+"/payroll/calculate-batch", bytes.NewBufferString(batchBody))
	req.SetPathValue("ws_id", wsID.String())
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), uuid.New()))
	rec = httptest.NewRecorder()
	handler.CalculateBatchPayroll(rec, req)
	if rec.Code != http.StatusOK {
		t.Fatalf("expected 200, got %d: %s", rec.Code, rec.Body.String())
	}
	var batchResp BatchCalculatePayrollResponse
	if err := json.Unmarshal(rec.Body.Bytes(), &batchResp); err != nil {
		t.Fatalf("failed to decode batch response: %v", err)
	}
	if len(batchResp.Results) != 2 {
		t.Fatalf("expected 2 results, got %d", len(batchResp.Results))
	}
	if batchResp.Results[0].Status != "success" || batchResp.Results[1].Status != "error" {
		t.Fatalf("unexpected statuses: %v, %v", batchResp.Results[0].Status, batchResp.Results[1].Status)
	}

	// 10b. POST calculate-batch empty list (400)
	req = httptest.NewRequest(http.MethodPost, "/api/v1/workspaces/"+wsID.String()+"/payroll/calculate-batch", bytes.NewBufferString(`{"employment_ids":[],"year":2026,"month":1}`))
	req.SetPathValue("ws_id", wsID.String())
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), uuid.New()))
	rec = httptest.NewRecorder()
	handler.CalculateBatchPayroll(rec, req)
	if rec.Code != http.StatusBadRequest {
		t.Fatalf("expected 400, got %d", rec.Code)
	}

	// 10c. POST calculate-batch forbidden without permission (403)
	req = httptest.NewRequest(http.MethodPost, "/api/v1/workspaces/"+wsID.String()+"/payroll/calculate-batch", bytes.NewBufferString(batchBody))
	req.SetPathValue("ws_id", wsID.String())
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), uuid.New()))
	rec = httptest.NewRecorder()
	noPermHandler.CalculateBatchPayroll(rec, req)
	if rec.Code != http.StatusForbidden {
		t.Fatalf("expected 403, got %d", rec.Code)
	}
}
