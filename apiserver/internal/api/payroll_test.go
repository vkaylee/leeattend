package api

import (
	"bytes"
	"context"
	"encoding/json"
	"errors"
	"fmt"
	"net/http"
	"net/http/httptest"
	"strings"
	"testing"
	"time"

	"github.com/google/uuid"
	"github.com/jackc/pgx/v5"
	"github.com/shopspring/decimal"
	"github.com/vkaylee/leeattend/apiserver/internal/audit"
	"github.com/vkaylee/leeattend/apiserver/internal/domain/ids"
	payrolldomain "github.com/vkaylee/leeattend/apiserver/internal/domain/payroll"
	httpinfra "github.com/vkaylee/leeattend/apiserver/internal/infra/http"
)

type mockPayrollRepository struct {
	settingsNotFound  bool
	listPayslipsErr   error
	listComponentsErr error
}

type payrollAuditRecorder struct {
	events []audit.Event
}

func (r *payrollAuditRecorder) Emit(_ context.Context, event audit.Event) error {
	r.events = append(r.events, event)
	return nil
}

func (mockPayrollRepository) CreatePayrollInputBatch(_ context.Context, req CreatePayrollInputRequest, _ ids.UserID) (PayrollInputBatch, error) {
	return PayrollInputBatch{ID: uuid.New(), PeriodYear: req.PeriodYear, PeriodMonth: req.PeriodMonth, Source: req.Source, IdempotencyKey: req.IdempotencyKey, Status: "submitted", InputCount: len(req.Inputs)}, nil
}

func (mockPayrollRepository) ApprovePayrollInputBatch(_ context.Context, batchID ids.PayrollInputBatchID, actorID ids.UserID) (PayrollInputBatch, error) {
	batchUUID, actorUUID := uuid.UUID(batchID), uuid.UUID(actorID)
	return PayrollInputBatch{ID: batchUUID, Status: "approved", ApprovedBy: &actorUUID}, nil
}

func (mockPayrollRepository) LockPayrollInputBatch(_ context.Context, batchID ids.PayrollInputBatchID, actorID ids.UserID) (PayrollInputBatch, error) {
	batchUUID, actorUUID := uuid.UUID(batchID), uuid.UUID(actorID)
	return PayrollInputBatch{ID: batchUUID, Status: "locked", LockedBy: &actorUUID}, nil
}

func (mockPayrollRepository) ListPayrollInputs(context.Context, ids.PayrollInputBatchID) ([]PayrollInput, error) {
	return []PayrollInput{}, nil
}

func (mockPayrollRepository) CorrectPayrollInput(_ context.Context, inputID ids.PayrollInputID, req CorrectPayrollInputRequest, _ ids.UserID) (PayrollInput, error) {
	return PayrollInput{ID: uuid.UUID(inputID), Value: req.Value, Unit: req.Unit, Status: "corrected"}, nil
}

func (mockPayrollRepository) ListPayrollCalculationRuns(context.Context, ids.EmploymentID, int, int) ([]PayrollCalculationRun, error) {
	return []PayrollCalculationRun{}, nil
}

func (mockPayrollRepository) ComparePayrollImpact(context.Context, ids.EmploymentID, int, int, uuid.UUID, uuid.UUID) (PayrollImpactResponse, error) {
	return PayrollImpactResponse{Computed: true}, nil
}

func (mockPayrollRepository) ComparePayrollImpactBatch(_ context.Context, req PayrollImpactBatchRequest) (PayrollImpactBatchResponse, error) {
	return PayrollImpactBatchResponse{Total: len(req.EmploymentIDs)}, nil
}

func (mockPayrollRepository) ListPayrollFormulaRuleVersions(context.Context, uuid.UUID) ([]PayrollFormulaRuleVersion, error) {
	return []PayrollFormulaRuleVersion{}, nil
}

func (mockPayrollRepository) ListPayrollFormulaRules(context.Context) ([]PayrollFormulaRule, error) {
	return []PayrollFormulaRule{}, nil
}

func (mockPayrollRepository) RetirePayrollFormulaRule(_ context.Context, ruleID ids.PayrollFormulaRuleID, _ ids.UserID) (PayrollFormulaRule, error) {
	return PayrollFormulaRule{ID: uuid.UUID(ruleID), Status: "retired"}, nil
}

func (mockPayrollRepository) CreatePayrollFormulaRule(_ context.Context, req CreatePayrollFormulaRuleRequest, _ ids.UserID) (PayrollFormulaRule, error) {
	return PayrollFormulaRule{ID: uuid.New(), Code: req.Code, Name: req.Name, ComponentCode: req.ComponentCode, Version: 1, Expression: req.Expression, EffectiveFrom: req.EffectiveFrom, Status: "draft"}, nil
}

func (mockPayrollRepository) CreatePayrollFormulaRuleVersion(_ context.Context, ruleID ids.PayrollFormulaRuleID, req CreatePayrollFormulaRuleVersionRequest, actorID ids.UserID) (PayrollFormulaRuleVersion, error) {
	return PayrollFormulaRuleVersion{RuleID: uuid.UUID(ruleID), VersionID: uuid.New(), Version: 2, Expression: req.Expression, EffectiveFrom: req.EffectiveFrom, Status: "draft", CreatedBy: uuid.UUID(actorID)}, nil
}

func (mockPayrollRepository) PublishPayrollFormulaRule(_ context.Context, ruleID ids.PayrollFormulaRuleID, _ ids.UserID) (PayrollFormulaRule, error) {
	return PayrollFormulaRule{ID: uuid.UUID(ruleID), Version: 1, Status: "published"}, nil
}

func (m mockPayrollRepository) ListPayslips(context.Context, time.Time, time.Time) ([]PayslipRecord, error) {
	if m.listPayslipsErr != nil {
		return nil, m.listPayslipsErr
	}
	return []PayslipRecord{{ID: uuid.New(), NetPayAmount: decimal.RequireFromString("1234.50")}}, nil
}

func (mockPayrollRepository) ListEmployeePayslips(context.Context, ids.EmployeeID) ([]PayslipRecord, error) {
	return []PayslipRecord{{ID: uuid.New(), NetPayAmount: decimal.RequireFromString("1234.50")}}, nil
}

func (m mockPayrollRepository) ListSalaryComponents(context.Context) ([]SalaryComponentDefinition, error) {
	if m.listComponentsErr != nil {
		return nil, m.listComponentsErr
	}
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

func (mockPayrollRepository) CreateSalaryComponent(_ context.Context, req CreateSalaryComponentRequest) (SalaryComponentDefinition, error) {
	if req.Code == "CONFLICT" {
		return SalaryComponentDefinition{}, ErrComponentConflict
	}
	return SalaryComponentDefinition{
		ID:        uuid.New(),
		Name:      req.Name,
		Code:      req.Code,
		CreatedAt: "2026-01-01T00:00:00Z",
		UpdatedAt: "2026-01-01T00:00:00Z",
	}, nil
}

func (mockPayrollRepository) ListSalaryManagementRows(_ context.Context, query SalaryManagementQuery) (SalaryManagementResponse, error) {
	return SalaryManagementResponse{Data: []SalaryManagementRow{}, Page: query.Page, Limit: query.Limit}, nil
}

func (mockPayrollRepository) ListEmploymentSalaryComponents(_ context.Context, employmentID ids.EmploymentID) ([]EmploymentSalaryComponentResponse, error) {
	return []EmploymentSalaryComponentResponse{
		{
			ID:                    uuid.New(),
			EmploymentID:          uuid.UUID(employmentID),
			ComponentDefinitionID: uuid.MustParse("11111111-1111-1111-1111-111111111111"),
			Amount:                decimal.RequireFromString("5000000.00"),
			EffectiveDate:         "2026-01-01",
			Name:                  "Base Allowance",
			Code:                  "BASE_ALLOWANCE",
		},
	}, nil
}

func (mockPayrollRepository) AssignEmploymentSalaryComponent(_ context.Context, employmentID ids.EmploymentID, req AssignSalaryComponentRequest) (EmployeeSalaryComponent, error) {
	if req.ComponentDefinitionID == uuid.MustParse("00000000-0000-0000-0000-000000000001") {
		return EmployeeSalaryComponent{}, ErrComponentNotFound
	}
	return EmployeeSalaryComponent{
		ID:                    uuid.New(),
		EmploymentID:          uuid.UUID(employmentID),
		ComponentDefinitionID: req.ComponentDefinitionID,
		Amount:                req.Amount,
		EffectiveDate:         req.EffectiveDate,
		CreatedAt:             "2026-01-01T00:00:00Z",
		UpdatedAt:             "2026-01-01T00:00:00Z",
	}, nil
}

func (m mockPayrollRepository) GetPayrollSettings(context.Context) (*PayrollSettings, error) {
	if m.settingsNotFound {
		return nil, ErrPayrollSettingsNotFound
	}
	return &PayrollSettings{
		StdWorkingDayMode:  "fixed",
		FixedStdDays:       decimal.RequireFromString("26"),
		WorkingDaysPerWeek: 5,
		HoursPerDay:        decimal.RequireFromString("8"),
		EffectiveDate:      "2026-01-01",
		CreatedAt:          "2026-01-01T00:00:00Z",
		UpdatedAt:          "2026-01-01T00:00:00Z",
	}, nil
}

func (mockPayrollRepository) GetPayrollSettingsHistory(context.Context) ([]PayrollSettings, error) {
	return []PayrollSettings{
		{
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

func (mockPayrollRepository) UpdatePayrollSettings(_ context.Context, req UpdatePayrollSettingsRequest) (PayrollSettings, error) {
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
		StdWorkingDayMode:  mode,
		FixedStdDays:       fixedDays,
		WorkingDaysPerWeek: 5,
		HoursPerDay:        decimal.RequireFromString("8"),
		EffectiveDate:      effDate,
		CreatedAt:          "2026-01-01T00:00:00Z",
		UpdatedAt:          "2026-01-01T00:00:00Z",
	}, nil
}

func (mockPayrollRepository) GetEmployeePayrollConfigs(_ context.Context, employeeID ids.EmployeeID) ([]EmployeePayrollConfig, error) {
	if uuid.UUID(employeeID) == uuid.MustParse("00000000-0000-0000-0000-000000000044") {
		return nil, ErrEmployeeNotFound
	}
	if uuid.UUID(employeeID) == uuid.MustParse("00000000-0000-0000-0000-000000000040") {
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

func (mockPayrollRepository) SetEmployeePayrollConfig(_ context.Context, employeeID ids.EmployeeID, req SetEmployeePayrollConfigRequest, _ ids.UserID) (EmployeePayrollConfig, error) {
	if uuid.UUID(employeeID) == uuid.MustParse("00000000-0000-0000-0000-000000000044") {
		return EmployeePayrollConfig{}, ErrEmployeeNotFound
	}
	if uuid.UUID(employeeID) == uuid.MustParse("00000000-0000-0000-0000-000000000040") {
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

func (mockPayrollRepository) GetIncentivePolicies(context.Context) ([]PayrollIncentivePolicyResponse, error) {
	return []PayrollIncentivePolicyResponse{
		{
			PayrollIncentivePolicy: PayrollIncentivePolicy{
				ID:            uuid.MustParse("22222222-2222-2222-2222-222222222222"),
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

func (mockPayrollRepository) CreateIncentivePolicy(_ context.Context, req CreatePayrollIncentivePolicyRequest) (PayrollIncentivePolicy, error) {
	return PayrollIncentivePolicy{
		ID:            uuid.New(),
		Name:          req.Name,
		BaseAmount:    req.BaseAmount,
		PayoutType:    req.PayoutType,
		IsActive:      true,
		CreatedAt:     "2026-01-01T00:00:00Z",
		EffectiveFrom: req.EffectiveFrom,
	}, nil
}

func (mockPayrollRepository) DeleteIncentivePolicy(_ context.Context, policyID ids.PayrollIncentivePolicyID) error {
	if uuid.UUID(policyID) == uuid.MustParse("00000000-0000-0000-0000-000000000099") {
		return ErrIncentivePolicyNotFound
	}
	return nil
}

func (mockPayrollRepository) CloneIncentivePolicy(_ context.Context, policyID ids.PayrollIncentivePolicyID) (PayrollIncentivePolicy, error) {
	if uuid.UUID(policyID) == uuid.MustParse("00000000-0000-0000-0000-000000000099") {
		return PayrollIncentivePolicy{}, ErrIncentivePolicyNotFound
	}
	return PayrollIncentivePolicy{
		ID:            uuid.New(),
		Name:          "Sales Commission (Copy)",
		BaseAmount:    decimal.RequireFromString("1000.00"),
		PayoutType:    "fixed",
		IsActive:      false,
		CreatedAt:     "2026-01-01T00:00:00Z",
		EffectiveFrom: "2026-01-01",
	}, nil
}

func (mockPayrollRepository) ListPayrollRules(context.Context) ([]PayrollRuleDefinition, error) {
	return []PayrollRuleDefinition{
		{
			ID:                uuid.MustParse("33333333-3333-3333-3333-333333333333"),
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

func (mockPayrollRepository) CreatePayrollRule(_ context.Context, req CreatePayrollRuleDefinitionRequest) (PayrollRuleDefinition, error) {
	if req.Code == "CONFLICT" {
		return PayrollRuleDefinition{}, ErrConflict
	}
	return PayrollRuleDefinition{
		ID:                uuid.New(),
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

func (mockPayrollRepository) ListPayrollRuleTiers(_ context.Context, ruleID ids.PayrollRuleID) ([]PayrollRuleTier, error) {
	if uuid.UUID(ruleID) == uuid.MustParse("00000000-0000-0000-0000-000000000099") {
		return nil, ErrRuleNotFound
	}
	return []PayrollRuleTier{
		{
			ID:               uuid.New(),
			RuleDefinitionID: uuid.UUID(ruleID),
			TierOrder:        1,
			MinAmount:        decimal.Zero,
			Rate:             decimal.RequireFromString("5"),
			FixedAmount:      decimal.Zero,
			CreatedAt:        "2026-01-01T00:00:00Z",
		},
	}, nil
}

func (mockPayrollRepository) CreatePayrollRuleTier(_ context.Context, ruleID ids.PayrollRuleID, req CreatePayrollRuleTierRequest) (PayrollRuleTier, error) {
	if uuid.UUID(ruleID) == uuid.MustParse("00000000-0000-0000-0000-000000000099") {
		return PayrollRuleTier{}, ErrRuleNotFound
	}
	fixed := decimal.Zero
	if req.FixedAmount != nil {
		fixed = *req.FixedAmount
	}
	return PayrollRuleTier{
		ID:               uuid.New(),
		RuleDefinitionID: uuid.UUID(ruleID),
		TierOrder:        req.TierOrder,
		MinAmount:        req.MinAmount,
		MaxAmount:        req.MaxAmount,
		Rate:             req.Rate,
		FixedAmount:      fixed,
		CreatedAt:        "2026-01-01T00:00:00Z",
	}, nil
}

func (mockPayrollRepository) DeletePayrollRuleTier(_ context.Context, _ ids.PayrollRuleID, tierID ids.PayrollRuleTierID) error {
	if uuid.UUID(tierID) == uuid.MustParse("00000000-0000-0000-0000-000000000099") {
		return ErrTierNotFound
	}
	return nil
}

func (mockPayrollRepository) ListPayrollRuleAssignments(_ context.Context, ruleID ids.PayrollRuleID) ([]PayrollRuleAssignment, error) {
	if uuid.UUID(ruleID) == uuid.MustParse("00000000-0000-0000-0000-000000000099") {
		return nil, ErrRuleNotFound
	}
	return []PayrollRuleAssignment{
		{
			ID:               uuid.New(),
			RuleDefinitionID: uuid.UUID(ruleID),
			AssigneeType:     "organization",
			EffectiveFrom:    "2026-01-01",
			CreatedAt:        "2026-01-01T00:00:00Z",
		},
	}, nil
}

func (mockPayrollRepository) AssignPayrollRule(_ context.Context, ruleID ids.PayrollRuleID, req AssignPayrollRuleRequest) (PayrollRuleAssignment, error) {
	if uuid.UUID(ruleID) == uuid.MustParse("00000000-0000-0000-0000-000000000099") {
		return PayrollRuleAssignment{}, ErrRuleNotFound
	}
	if req.AssigneeType == "organization" && req.EffectiveFrom == "2026-09-09" {
		return PayrollRuleAssignment{}, ErrAssignmentConflict
	}
	return PayrollRuleAssignment{
		ID:               uuid.New(),
		RuleDefinitionID: uuid.UUID(ruleID),
		AssigneeType:     req.AssigneeType,
		AssigneeID:       req.AssigneeID,
		EffectiveFrom:    req.EffectiveFrom,
		EffectiveTo:      req.EffectiveTo,
		OverrideConfig:   req.OverrideConfig,
		CreatedAt:        "2026-01-01T00:00:00Z",
	}, nil
}

func (mockPayrollRepository) DeletePayrollRuleAssignment(_ context.Context, _ ids.PayrollRuleID, assignmentID ids.PayrollRuleAssignmentID) error {
	if uuid.UUID(assignmentID) == uuid.MustParse("00000000-0000-0000-0000-000000000099") {
		return ErrAssignmentNotFound
	}
	return nil
}

func (mockPayrollRepository) CalculateEmployeePayroll(_ context.Context, employeeID ids.EmployeeID, _, _ int, _ ids.UserID) (FullPayslipResponse, error) {
	if uuid.UUID(employeeID) == uuid.MustParse("00000000-0000-0000-0000-000000000044") {
		return FullPayslipResponse{}, ErrEmployeeNotFound
	}
	if uuid.UUID(employeeID) == uuid.MustParse("00000000-0000-0000-0000-000000000040") {
		return FullPayslipResponse{}, ErrNoActiveEmployment
	}
	payslipID := uuid.New()
	employmentID := uuid.New()
	return FullPayslipResponse{
		PayslipRecord: PayslipRecord{
			ID:                payslipID,
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

func (mockPayrollRepository) GetPayrollPeriod(_ context.Context, employerID ids.EmployerID, year, month int) (PayrollPeriod, error) {
	return PayrollPeriod{ID: uuid.New(), EmployerID: uuid.UUID(employerID), Year: year, Month: month, Status: "open", Version: 1}, nil
}

func (mockPayrollRepository) LockPayrollPeriod(_ context.Context, input payrolldomain.PayrollPeriodMutation) (PayrollPeriod, error) {
	return PayrollPeriod{ID: uuid.New(), EmployerID: uuid.UUID(input.EmployerID), Year: input.Year, Month: input.Month, Status: "locked", Version: input.ExpectedVersion + 1}, nil
}

func (mockPayrollRepository) ReopenPayrollPeriod(_ context.Context, input payrolldomain.PayrollPeriodMutation) (PayrollPeriod, error) {
	return PayrollPeriod{ID: uuid.New(), EmployerID: uuid.UUID(input.EmployerID), Year: input.Year, Month: input.Month, Status: "open", Version: input.ExpectedVersion + 1, ReopenReason: &input.Reason}, nil
}

func (mockPayrollRepository) SimulateEmployeePayroll(_ context.Context, employeeID ids.EmployeeID, _, _ int, _ ids.UserID) (FullPayslipResponse, error) {
	return FullPayslipResponse{PayslipRecord: PayslipRecord{EmploymentID: uuid.UUID(employeeID)}}, nil
}

func (mockPayrollRepository) SimulateEmployeePayrollWithPolicy(_ context.Context, employeeID ids.EmployeeID, _, _ int, _ uuid.UUID, _ ids.UserID) (FullPayslipResponse, error) {
	return FullPayslipResponse{PayslipRecord: PayslipRecord{EmploymentID: uuid.UUID(employeeID)}}, nil
}

func (mockPayrollRepository) CalculateBatchPayroll(_ context.Context, req BatchCalculatePayrollRequest, _ ids.UserID) (BatchCalculatePayrollResponse, error) {
	results := make([]BatchCalculateResult, 0, len(req.EmploymentIDs))
	for _, id := range req.EmploymentIDs {
		if id == uuid.MustParse("00000000-0000-0000-0000-000000000099") {
			errMsg := "employment not found"
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

func (a payrollAuthorizer) IsMember(_ context.Context, _ uuid.UUID) (bool, error) {
	return a.isMember, nil
}

func (a payrollAuthorizer) HasPermission(_ context.Context, _ uuid.UUID, perm string) bool {
	if a.allowedPerms == nil {
		return true
	}
	return a.allowedPerms[perm]
}

func (a payrollAuthorizer) EffectivePermissions(_ context.Context, _ uuid.UUID) ([]string, []string, error) {
	return nil, nil, nil
}

func (a payrollAuthorizer) GetManagedDepartmentIDs(_ context.Context, _ uuid.UUID) ([]uuid.UUID, error) {
	return nil, nil
}

func (a payrollAuthorizer) IsGlobalManager(_ context.Context, _ uuid.UUID) bool {
	return false
}

func (a payrollAuthorizer) HasScopedPermission(_ context.Context, _ uuid.UUID, perm string, _ *uuid.UUID) bool {
	if a.allowedPerms == nil {
		return true
	}
	return a.allowedPerms[perm]
}

func (a payrollAuthorizer) GetUsersWithPermission(_ context.Context, _ string) ([]uuid.UUID, error) {
	return nil, nil
}

func contains(value, fragment string) bool {
	for index := 0; index+len(fragment) <= len(value); index++ {
		if value[index:index+len(fragment)] == fragment {
			return true
		}
	}
	return false
}

func TestPayrollHandlersMapRepositoryFailures(t *testing.T) {
	actor := uuid.New()
	for _, test := range []struct {
		name string
		err  error
		want int
	}{
		{name: "not found", err: pgx.ErrNoRows, want: http.StatusNotFound},
		{name: "internal", err: errors.New("repository unavailable"), want: http.StatusInternalServerError},
	} {
		t.Run(test.name, func(t *testing.T) {
			handler := PayrollHandler{
				Repository: mockPayrollRepository{listPayslipsErr: test.err},
				Authorizer: payrollAuthorizer{allowedPerms: map[string]bool{"payroll:read": true}},
			}
			req := httptest.NewRequest(http.MethodGet, "/api/v1/payroll/payslips?year=2026&month=1", nil)
			req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), actor))
			rec := httptest.NewRecorder()
			handler.ListPayslips(rec, req)
			if rec.Code != test.want {
				t.Fatalf("status = %d, want %d: %s", rec.Code, test.want, rec.Body.String())
			}
		})
	}
}

func TestPayrollHandlersRejectMissingUserBeforeRepository(t *testing.T) {
	handler := PayrollHandler{
		Repository: mockPayrollRepository{listPayslipsErr: errors.New("repository should not be called")},
		Authorizer: payrollAuthorizer{allowedPerms: map[string]bool{"payroll:read": true}},
	}
	rec := httptest.NewRecorder()
	handler.ListPayslips(rec, httptest.NewRequest(http.MethodGet, "/api/v1/payroll/payslips?year=2026&month=1", nil))
	if rec.Code != http.StatusForbidden {
		t.Fatalf("status = %d, want 403", rec.Code)
	}
}

func TestPayrollSimulatePayrollAcceptsDraftPolicyVersion(t *testing.T) {
	handler := PayrollHandler{Repository: mockPayrollRepository{}, Authorizer: payrollAuthorizer{allowedPerms: map[string]bool{"payroll:calculate": true}}}
	employeeID := uuid.New()
	policyVersionID := uuid.New()
	req := httptest.NewRequest(http.MethodPost, "/api/v1/employees/"+employeeID.String()+"/simulate-payroll?year=2026&month=6&policy_version_id="+policyVersionID.String(), nil)
	req.SetPathValue("id", employeeID.String())
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), uuid.New()))
	rec := httptest.NewRecorder()
	handler.SimulatePayroll(rec, req)
	if rec.Code != http.StatusOK {
		t.Fatalf("expected 200, got %d: %s", rec.Code, rec.Body.String())
	}
}

func TestPayrollSimulatePayrollRejectsMalformedDraftPolicyVersion(t *testing.T) {
	handler := PayrollHandler{Repository: mockPayrollRepository{}, Authorizer: payrollAuthorizer{allowedPerms: map[string]bool{"payroll:calculate": true}}}
	employeeID := uuid.New()
	req := httptest.NewRequest(http.MethodPost, "/api/v1/employees/"+employeeID.String()+"/simulate-payroll?year=2026&month=6&policy_version_id=bad", nil)
	req.SetPathValue("id", employeeID.String())
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), uuid.New()))
	rec := httptest.NewRecorder()
	handler.SimulatePayroll(rec, req)
	if rec.Code != http.StatusBadRequest {
		t.Fatalf("expected 400, got %d", rec.Code)
	}
}

func TestPayrollListPayslipsValidatesPeriod(t *testing.T) {
	handler := PayrollHandler{Repository: mockPayrollRepository{}, Authorizer: payrollAuthorizer{}}
	req := httptest.NewRequest(http.MethodGet, "/api/v1/payroll/payslips?year=2026&month=13", nil)
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), uuid.New()))
	rec := httptest.NewRecorder()
	handler.ListPayslips(rec, req)
	if rec.Code != http.StatusBadRequest {
		t.Fatalf("expected 400, got %d", rec.Code)
	}
}

func TestPayrollListEmployeeReturnsDecimalAmount(t *testing.T) {
	handler := PayrollHandler{Repository: mockPayrollRepository{}, Authorizer: payrollAuthorizer{}}
	employeeID := uuid.New()
	req := httptest.NewRequest(http.MethodGet, "/api/v1/employees/"+employeeID.String()+"/payslips", nil)
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

func TestPayrollSalaryComponentsEndpoints(t *testing.T) {
	handler := PayrollHandler{Repository: mockPayrollRepository{}, Authorizer: payrollAuthorizer{}}

	// 1. GET /api/v1/payroll/salary-components
	req := httptest.NewRequest(http.MethodGet, "/api/v1/payroll/salary-components", nil)
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), uuid.New()))
	rec := httptest.NewRecorder()
	handler.ListSalaryComponents(rec, req)
	if rec.Code != http.StatusOK {
		t.Fatalf("expected 200, got %d: %s", rec.Code, rec.Body.String())
	}
	if !contains(rec.Body.String(), "BASE_ALLOWANCE") {
		t.Fatalf("expected component in body: %s", rec.Body.String())
	}

	// 2. POST /api/v1/payroll/salary-components (201 Created)
	body := `{"name":"Transport Allowance","code":"TRANSPORT"}`
	req = httptest.NewRequest(http.MethodPost, "/api/v1/payroll/salary-components", bytes.NewBufferString(body))
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
	req = httptest.NewRequest(http.MethodPost, "/api/v1/payroll/salary-components", bytes.NewBufferString(`{"name":"","code":"TEST"}`))
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
	req = httptest.NewRequest(http.MethodPost, "/api/v1/payroll/salary-components", bytes.NewBufferString(body))
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), uuid.New()))
	rec = httptest.NewRecorder()
	noPermHandler.CreateSalaryComponent(rec, req)
	if rec.Code != http.StatusForbidden {
		t.Fatalf("expected 403 without manage perm, got %d", rec.Code)
	}
}

func TestPayrollEmploymentSalaryComponentsEndpoints(t *testing.T) {
	handler := PayrollHandler{Repository: mockPayrollRepository{}, Authorizer: payrollAuthorizer{}}
	empID := uuid.New()

	// 3. GET /api/v1/employments/{id}/salary-components
	req := httptest.NewRequest(http.MethodGet, "/api/v1/employments/"+empID.String()+"/salary-components", nil)
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

	// 4. POST /api/v1/employments/{id}/salary-components (201 Created)
	compID := uuid.MustParse("11111111-1111-1111-1111-111111111111")
	assignBody := `{"component_definition_id":"` + compID.String() + `","amount":"6500000.00","effective_date":"2026-02-01"}`
	req = httptest.NewRequest(http.MethodPost, "/api/v1/employments/"+empID.String()+"/salary-components", bytes.NewBufferString(assignBody))
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
	req = httptest.NewRequest(http.MethodPost, "/api/v1/employments/"+empID.String()+"/salary-components", bytes.NewBufferString(invalidDateBody))
	req.SetPathValue("id", empID.String())
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), uuid.New()))
	rec = httptest.NewRecorder()
	handler.AssignEmploymentSalaryComponent(rec, req)
	if rec.Code != http.StatusBadRequest {
		t.Fatalf("expected 400 for bad date format, got %d", rec.Code)
	}
}

func TestPayrollSettingsEndpoints(t *testing.T) {
	handler := PayrollHandler{Repository: mockPayrollRepository{}, Authorizer: payrollAuthorizer{isMember: true}}

	// 1. GET /payroll-settings (200 OK)
	req := httptest.NewRequest(http.MethodGet, "/api/v1/payroll-settings", nil)
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
	notFoundHandler := PayrollHandler{Repository: mockPayrollRepository{settingsNotFound: true}, Authorizer: payrollAuthorizer{isMember: true}}
	req = httptest.NewRequest(http.MethodGet, "/api/v1/payroll-settings", nil)
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), uuid.New()))
	rec = httptest.NewRecorder()
	notFoundHandler.GetPayrollSettings(rec, req)
	if rec.Code != http.StatusNotFound {
		t.Fatalf("expected 404, got %d: %s", rec.Code, rec.Body.String())
	}

	// 1c. GET /payroll-settings forbidden (403)
	forbiddenHandler := PayrollHandler{Repository: mockPayrollRepository{}, Authorizer: payrollAuthorizer{allowedPerms: map[string]bool{}, isMember: false}}
	req = httptest.NewRequest(http.MethodGet, "/api/v1/payroll-settings", nil)
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), uuid.New()))
	rec = httptest.NewRecorder()
	forbiddenHandler.GetPayrollSettings(rec, req)
	if rec.Code != http.StatusForbidden {
		t.Fatalf("expected 403, got %d: %s", rec.Code, rec.Body.String())
	}

	// 1d. PATCH /payroll-settings requires payroll:manage even for members.
	memberOnlyHandler := PayrollHandler{Repository: mockPayrollRepository{}, Authorizer: payrollAuthorizer{isMember: true, allowedPerms: map[string]bool{"payroll:manage": false}}}
	req = httptest.NewRequest(http.MethodPatch, "/api/v1/payroll-settings", bytes.NewBufferString(`{"fixed_std_days":"25","effective_date":"2026-06-01"}`))
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), uuid.New()))
	rec = httptest.NewRecorder()
	memberOnlyHandler.UpdatePayrollSettings(rec, req)
	if rec.Code != http.StatusForbidden {
		t.Fatalf("expected 403 for member without payroll:manage, got %d", rec.Code)
	}

	// 2. PATCH /payroll-settings (200 OK)
	patchBody := `{"std_working_day_mode":"custom","fixed_std_days":"25","effective_date":"2026-06-01"}`
	req = httptest.NewRequest(http.MethodPatch, "/api/v1/payroll-settings", bytes.NewBufferString(patchBody))
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
	req = httptest.NewRequest(http.MethodPatch, "/api/v1/payroll-settings", bytes.NewBufferString(badPatchBody))
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), uuid.New()))
	rec = httptest.NewRecorder()
	handler.UpdatePayrollSettings(rec, req)
	if rec.Code != http.StatusBadRequest {
		t.Fatalf("expected 400 for bad date, got %d: %s", rec.Code, rec.Body.String())
	}

	// 3. GET /payroll-settings/history (200 OK)
	req = httptest.NewRequest(http.MethodGet, "/api/v1/payroll-settings/history", nil)
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
	validEmpID := uuid.New()
	missingEmpID := uuid.MustParse("00000000-0000-0000-0000-000000000044")
	noActiveEmpID := uuid.MustParse("00000000-0000-0000-0000-000000000040")

	// 4. GET /employees/{id}/payroll-config (200 OK)
	req := httptest.NewRequest(http.MethodGet, "/api/v1/employees/"+validEmpID.String()+"/payroll-config", nil)
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
	req = httptest.NewRequest(http.MethodGet, "/api/v1/employees/"+missingEmpID.String()+"/payroll-config", nil)
	req.SetPathValue("id", missingEmpID.String())
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), uuid.New()))
	rec = httptest.NewRecorder()
	handler.GetEmployeePayrollConfig(rec, req)
	if rec.Code != http.StatusNotFound {
		t.Fatalf("expected 404, got %d: %s", rec.Code, rec.Body.String())
	}

	// 4c. GET /employees/{id}/payroll-config no active employment (400)
	req = httptest.NewRequest(http.MethodGet, "/api/v1/employees/"+noActiveEmpID.String()+"/payroll-config", nil)
	req.SetPathValue("id", noActiveEmpID.String())
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), uuid.New()))
	rec = httptest.NewRecorder()
	handler.GetEmployeePayrollConfig(rec, req)
	if rec.Code != http.StatusBadRequest {
		t.Fatalf("expected 400, got %d: %s", rec.Code, rec.Body.String())
	}

	// 4d. POST /employees/{id}/payroll-config requires payroll:manage even for members.
	memberOnlyHandler := PayrollHandler{Repository: mockPayrollRepository{}, Authorizer: payrollAuthorizer{isMember: true, allowedPerms: map[string]bool{"payroll:manage": false}}}
	createBody := `{"std_working_days":"22","hours_per_day":"8","effective_date":"2026-06-01"}`
	req = httptest.NewRequest(http.MethodPost, "/api/v1/employees/"+validEmpID.String()+"/payroll-config", bytes.NewBufferString(createBody))
	req.SetPathValue("id", validEmpID.String())
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), uuid.New()))
	rec = httptest.NewRecorder()
	memberOnlyHandler.SetEmployeePayrollConfig(rec, req)
	if rec.Code != http.StatusForbidden {
		t.Fatalf("expected 403 for member without payroll:manage, got %d", rec.Code)
	}

	// 5. POST /employees/{id}/payroll-config (201 Created)
	req = httptest.NewRequest(http.MethodPost, "/api/v1/employees/"+validEmpID.String()+"/payroll-config", bytes.NewBufferString(createBody))
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
	req = httptest.NewRequest(http.MethodPost, "/api/v1/employees/"+validEmpID.String()+"/payroll-config", bytes.NewBufferString(badBody))
	req.SetPathValue("id", validEmpID.String())
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), uuid.New()))
	rec = httptest.NewRecorder()
	handler.SetEmployeePayrollConfig(rec, req)
	if rec.Code != http.StatusBadRequest {
		t.Fatalf("expected 400, got %d: %s", rec.Code, rec.Body.String())
	}

	// 5c. POST /employees/{id}/payroll-config employee not found (404)
	req = httptest.NewRequest(http.MethodPost, "/api/v1/employees/"+missingEmpID.String()+"/payroll-config", bytes.NewBufferString(createBody))
	req.SetPathValue("id", missingEmpID.String())
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), uuid.New()))
	rec = httptest.NewRecorder()
	handler.SetEmployeePayrollConfig(rec, req)
	if rec.Code != http.StatusNotFound {
		t.Fatalf("expected 404, got %d: %s", rec.Code, rec.Body.String())
	}

	// 5d. POST /employees/{id}/payroll-config no active employment (400)
	req = httptest.NewRequest(http.MethodPost, "/api/v1/employees/"+noActiveEmpID.String()+"/payroll-config", bytes.NewBufferString(createBody))
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
	validPolicyID := uuid.MustParse("22222222-2222-2222-2222-222222222222")
	missingPolicyID := uuid.MustParse("00000000-0000-0000-0000-000000000099")

	// 6. GET /payroll/incentive-policies (200 OK)
	req := httptest.NewRequest(http.MethodGet, "/api/v1/payroll/incentive-policies", nil)
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
	req = httptest.NewRequest(http.MethodPost, "/api/v1/payroll/incentive-policies", bytes.NewBufferString(createBody))
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
	req = httptest.NewRequest(http.MethodPost, "/api/v1/payroll/incentive-policies", bytes.NewBufferString(badBody))
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), uuid.New()))
	rec = httptest.NewRecorder()
	handler.CreateIncentivePolicy(rec, req)
	if rec.Code != http.StatusBadRequest {
		t.Fatalf("expected 400 for empty name, got %d: %s", rec.Code, rec.Body.String())
	}

	// 8. DELETE /payroll/incentive-policies/{policy_id} (204 No Content)
	req = httptest.NewRequest(http.MethodDelete, "/api/v1/payroll/incentive-policies/"+validPolicyID.String(), nil)
	req.SetPathValue("policy_id", validPolicyID.String())
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), uuid.New()))
	rec = httptest.NewRecorder()
	handler.DeleteIncentivePolicy(rec, req)
	if rec.Code != http.StatusNoContent {
		t.Fatalf("expected 204, got %d: %s", rec.Code, rec.Body.String())
	}

	// 9. POST /payroll/incentive-policies/{policy_id}/clone (201 Created)
	req = httptest.NewRequest(http.MethodPost, "/api/v1/payroll/incentive-policies/"+validPolicyID.String()+"/clone", nil)
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
	req = httptest.NewRequest(http.MethodPost, "/api/v1/payroll/incentive-policies/"+missingPolicyID.String()+"/clone", nil)
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

	// 1. GET /payroll-rules (200 OK)
	req := httptest.NewRequest(http.MethodGet, "/api/v1/payroll-rules", nil)
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
	req = httptest.NewRequest(http.MethodGet, "/api/v1/payroll-rules", nil)
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), uuid.New()))
	rec = httptest.NewRecorder()
	forbiddenHandler.ListPayrollRules(rec, req)
	if rec.Code != http.StatusForbidden {
		t.Fatalf("expected 403, got %d", rec.Code)
	}

	// 2. POST /payroll-rules (201 Created)
	body := `{"name":"Health Insurance","code":"BHYT","category":"statutory_deduction","applies_to":"employee","calculation_method":"percentage","rule_config":{"rate":"1.5"}}`
	req = httptest.NewRequest(http.MethodPost, "/api/v1/payroll-rules", bytes.NewBufferString(body))
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), uuid.New()))
	rec = httptest.NewRecorder()
	handler.CreatePayrollRule(rec, req)
	if rec.Code != http.StatusCreated {
		t.Fatalf("expected 201, got %d: %s", rec.Code, rec.Body.String())
	}

	// 2b. POST /payroll-rules validation error (400)
	req = httptest.NewRequest(http.MethodPost, "/api/v1/payroll-rules", bytes.NewBufferString(`{"name":""}`))
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), uuid.New()))
	rec = httptest.NewRecorder()
	handler.CreatePayrollRule(rec, req)
	if rec.Code != http.StatusBadRequest {
		t.Fatalf("expected 400, got %d", rec.Code)
	}

	// 2c. POST /payroll-rules conflict (409)
	req = httptest.NewRequest(http.MethodPost, "/api/v1/payroll-rules", bytes.NewBufferString(`{"name":"Dup","code":"CONFLICT"}`))
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), uuid.New()))
	rec = httptest.NewRecorder()
	handler.CreatePayrollRule(rec, req)
	if rec.Code != http.StatusConflict {
		t.Fatalf("expected 409, got %d", rec.Code)
	}
}

func TestPayrollRuleTiersEndpoints(t *testing.T) {
	handler := PayrollHandler{Repository: mockPayrollRepository{}, Authorizer: payrollAuthorizer{isMember: true}}
	ruleID := uuid.New()
	missingRuleID := uuid.MustParse("00000000-0000-0000-0000-000000000099")

	// 3. GET /payroll-rules/{rule_id}/tiers (200 OK)
	req := httptest.NewRequest(http.MethodGet, "/api/v1/payroll-rules/"+ruleID.String()+"/tiers", nil)
	req.SetPathValue("rule_id", ruleID.String())
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), uuid.New()))
	rec := httptest.NewRecorder()
	handler.ListPayrollRuleTiers(rec, req)
	if rec.Code != http.StatusOK {
		t.Fatalf("expected 200, got %d: %s", rec.Code, rec.Body.String())
	}

	// 3b. GET /payroll-rules/{rule_id}/tiers not found (404)
	req = httptest.NewRequest(http.MethodGet, "/api/v1/payroll-rules/"+missingRuleID.String()+"/tiers", nil)
	req.SetPathValue("rule_id", missingRuleID.String())
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), uuid.New()))
	rec = httptest.NewRecorder()
	handler.ListPayrollRuleTiers(rec, req)
	if rec.Code != http.StatusNotFound {
		t.Fatalf("expected 404, got %d", rec.Code)
	}

	// 4. POST /payroll-rules/{rule_id}/tiers (201 Created)
	body := `{"tier_order":1,"min_amount":0,"max_amount":5000000,"rate":5,"fixed_amount":0}`
	req = httptest.NewRequest(http.MethodPost, "/api/v1/payroll-rules/"+ruleID.String()+"/tiers", bytes.NewBufferString(body))
	req.SetPathValue("rule_id", ruleID.String())
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), uuid.New()))
	rec = httptest.NewRecorder()
	handler.CreatePayrollRuleTier(rec, req)
	if rec.Code != http.StatusCreated {
		t.Fatalf("expected 201, got %d: %s", rec.Code, rec.Body.String())
	}

	// 4b. POST /payroll-rules/{rule_id}/tiers validation: max_amount <= min_amount (400)
	body = `{"tier_order":1,"min_amount":5000000,"max_amount":5000000,"rate":5}`
	req = httptest.NewRequest(http.MethodPost, "/api/v1/payroll-rules/"+ruleID.String()+"/tiers", bytes.NewBufferString(body))
	req.SetPathValue("rule_id", ruleID.String())
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), uuid.New()))
	rec = httptest.NewRecorder()
	handler.CreatePayrollRuleTier(rec, req)
	if rec.Code != http.StatusBadRequest {
		t.Fatalf("expected 400, got %d", rec.Code)
	}

	// 5. DELETE /payroll-rules/{rule_id}/tiers/{tier_id} (200 OK)
	tierID := uuid.New()
	req = httptest.NewRequest(http.MethodDelete, "/api/v1/payroll-rules/"+ruleID.String()+"/tiers/"+tierID.String(), nil)
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
	req = httptest.NewRequest(http.MethodDelete, "/api/v1/payroll-rules/"+ruleID.String()+"/tiers/"+missingTierID.String(), nil)
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
	ruleID := uuid.New()
	missingRuleID := uuid.MustParse("00000000-0000-0000-0000-000000000099")

	// 6. GET /payroll-rules/{rule_id}/assignments (200 OK)
	req := httptest.NewRequest(http.MethodGet, "/api/v1/payroll-rules/"+ruleID.String()+"/assignments", nil)
	req.SetPathValue("rule_id", ruleID.String())
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), uuid.New()))
	rec := httptest.NewRecorder()
	handler.ListPayrollRuleAssignments(rec, req)
	if rec.Code != http.StatusOK {
		t.Fatalf("expected 200, got %d: %s", rec.Code, rec.Body.String())
	}

	// 6b. GET /payroll-rules/{rule_id}/assignments not found (404)
	req = httptest.NewRequest(http.MethodGet, "/api/v1/payroll-rules/"+missingRuleID.String()+"/assignments", nil)
	req.SetPathValue("rule_id", missingRuleID.String())
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), uuid.New()))
	rec = httptest.NewRecorder()
	handler.ListPayrollRuleAssignments(rec, req)
	if rec.Code != http.StatusNotFound {
		t.Fatalf("expected 404, got %d", rec.Code)
	}

	// 7. POST /payroll-rules/{rule_id}/assignments for organization (201 Created)
	body := `{"assignee_type":"organization","effective_from":"2026-01-01"}`
	req = httptest.NewRequest(http.MethodPost, "/api/v1/payroll-rules/"+ruleID.String()+"/assignments", bytes.NewBufferString(body))
	req.SetPathValue("rule_id", ruleID.String())
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), uuid.New()))
	rec = httptest.NewRecorder()
	handler.AssignPayrollRule(rec, req)
	if rec.Code != http.StatusCreated {
		t.Fatalf("expected 201 for organization assignee_type, got %d: %s", rec.Code, rec.Body.String())
	}

	// 7a. POST /payroll-rules/{rule_id}/assignments for department (201 Created)
	targetDeptID := uuid.New()
	body = `{"assignee_type":"department","assignee_id":"` + targetDeptID.String() + `","effective_from":"2026-01-01"}`
	req = httptest.NewRequest(http.MethodPost, "/api/v1/payroll-rules/"+ruleID.String()+"/assignments", bytes.NewBufferString(body))
	req.SetPathValue("rule_id", ruleID.String())
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), uuid.New()))
	rec = httptest.NewRecorder()
	handler.AssignPayrollRule(rec, req)
	if rec.Code != http.StatusCreated {
		t.Fatalf("expected 201 for department assignee_type, got %d: %s", rec.Code, rec.Body.String())
	}

	// 7b. POST /payroll-rules/{rule_id}/assignments for employee (201 Created)
	targetEmpID := uuid.New()
	body = `{"assignee_type":"employee","assignee_id":"` + targetEmpID.String() + `","effective_from":"2026-01-01"}`
	req = httptest.NewRequest(http.MethodPost, "/api/v1/payroll-rules/"+ruleID.String()+"/assignments", bytes.NewBufferString(body))
	req.SetPathValue("rule_id", ruleID.String())
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), uuid.New()))
	rec = httptest.NewRecorder()
	handler.AssignPayrollRule(rec, req)
	if rec.Code != http.StatusCreated {
		t.Fatalf("expected 201 for employee assignee_type, got %d: %s", rec.Code, rec.Body.String())
	}

	// 7c. POST department without assignee_id fails (400)
	body = `{"assignee_type":"department","effective_from":"2026-01-01"}`
	req = httptest.NewRequest(http.MethodPost, "/api/v1/payroll-rules/"+ruleID.String()+"/assignments", bytes.NewBufferString(body))
	req.SetPathValue("rule_id", ruleID.String())
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), uuid.New()))
	rec = httptest.NewRecorder()
	handler.AssignPayrollRule(rec, req)
	if rec.Code != http.StatusBadRequest {
		t.Fatalf("expected 400 for department without assignee_id, got %d: %s", rec.Code, rec.Body.String())
	}

	// 7d. POST legacy workspace assignee_type rejected (400)
	body = `{"assignee_type":"workspace","effective_from":"2026-01-01"}`
	req = httptest.NewRequest(http.MethodPost, "/api/v1/payroll-rules/"+ruleID.String()+"/assignments", bytes.NewBufferString(body))
	req.SetPathValue("rule_id", ruleID.String())
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), uuid.New()))
	rec = httptest.NewRecorder()
	handler.AssignPayrollRule(rec, req)
	if rec.Code != http.StatusBadRequest {
		t.Fatalf("expected 400 for legacy workspace assignee_type, got %d: %s", rec.Code, rec.Body.String())
	}

	// 7e. POST user assignee_type rejected (400)
	targetUserID := uuid.New()
	body = `{"assignee_type":"user","assignee_id":"` + targetUserID.String() + `","effective_from":"2026-01-01"}`
	req = httptest.NewRequest(http.MethodPost, "/api/v1/payroll-rules/"+ruleID.String()+"/assignments", bytes.NewBufferString(body))
	req.SetPathValue("rule_id", ruleID.String())
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), uuid.New()))
	rec = httptest.NewRecorder()
	handler.AssignPayrollRule(rec, req)
	if rec.Code != http.StatusBadRequest {
		t.Fatalf("expected 400 for user assignee_type, got %d: %s", rec.Code, rec.Body.String())
	}

	// 7f. POST unknown assignee_type (400)
	body = `{"assignee_type":"unknown","effective_from":"2026-01-01"}`
	req = httptest.NewRequest(http.MethodPost, "/api/v1/payroll-rules/"+ruleID.String()+"/assignments", bytes.NewBufferString(body))
	req.SetPathValue("rule_id", ruleID.String())
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), uuid.New()))
	rec = httptest.NewRecorder()
	handler.AssignPayrollRule(rec, req)
	if rec.Code != http.StatusBadRequest {
		t.Fatalf("expected 400, got %d", rec.Code)
	}

	// 7g. POST conflict (409)
	body = `{"assignee_type":"organization","effective_from":"2026-09-09"}`
	req = httptest.NewRequest(http.MethodPost, "/api/v1/payroll-rules/"+ruleID.String()+"/assignments", bytes.NewBufferString(body))
	req.SetPathValue("rule_id", ruleID.String())
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), uuid.New()))
	rec = httptest.NewRecorder()
	handler.AssignPayrollRule(rec, req)
	if rec.Code != http.StatusConflict {
		t.Fatalf("expected 409, got %d", rec.Code)
	}

	// 8. DELETE /payroll-rules/{rule_id}/assignments/{assignment_id} (200 OK)
	assignID := uuid.New()
	req = httptest.NewRequest(http.MethodDelete, "/api/v1/payroll-rules/"+ruleID.String()+"/assignments/"+assignID.String(), nil)
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
	req = httptest.NewRequest(http.MethodDelete, "/api/v1/payroll-rules/"+ruleID.String()+"/assignments/"+missingAssignID.String(), nil)
	req.SetPathValue("rule_id", ruleID.String())
	req.SetPathValue("assignment_id", missingAssignID.String())
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), uuid.New()))
	rec = httptest.NewRecorder()
	handler.DeletePayrollRuleAssignment(rec, req)
	if rec.Code != http.StatusNotFound {
		t.Fatalf("expected 404, got %d", rec.Code)
	}
}

func TestPayrollInputAuditEventsAvoidSensitiveValues(t *testing.T) {
	actorID := uuid.New()
	employmentID := uuid.New()
	auditRecorder := &payrollAuditRecorder{}
	handler := PayrollHandler{
		Repository: mockPayrollRepository{},
		Authorizer: payrollAuthorizer{allowedPerms: map[string]bool{"payroll:inputs:manage": true, "payroll:manage": true}},
		Audit:      auditRecorder,
	}
	body := `{"period_year":2026,"period_month":10,"source":"production","idempotency_key":"batch-1","inputs":[{"employment_id":"` + employmentID.String() + `","metric_code":"production.output","value":"950","unit":"product","source_reference":"internal-source"}]}`
	createReq := httptest.NewRequest(http.MethodPost, "/api/v1/payroll/inputs/import", bytes.NewBufferString(body))
	createReq = createReq.WithContext(httpinfra.WithUserIDForTest(createReq.Context(), actorID))
	createRec := httptest.NewRecorder()
	handler.CreatePayrollInputBatch(createRec, createReq)
	if createRec.Code != http.StatusCreated {
		t.Fatalf("create status = %d, want 201", createRec.Code)
	}

	inputID := uuid.New()
	correctReq := httptest.NewRequest(http.MethodPost, "/api/v1/payroll/inputs/"+inputID.String()+"/correct", bytes.NewBufferString(`{"value":"1000","unit":"product","reason":"source correction"}`))
	correctReq.SetPathValue("input_id", inputID.String())
	correctReq = correctReq.WithContext(httpinfra.WithUserIDForTest(correctReq.Context(), actorID))
	correctRec := httptest.NewRecorder()
	handler.CorrectPayrollInput(correctRec, correctReq)
	if correctRec.Code != http.StatusOK {
		t.Fatalf("correct status = %d, want 200", correctRec.Code)
	}

	if len(auditRecorder.events) != 2 {
		t.Fatalf("audit event count = %d, want 2", len(auditRecorder.events))
	}
	if auditRecorder.events[0].Name != "payroll.input.submitted" || auditRecorder.events[1].Name != "payroll.input.corrected" {
		t.Fatalf("unexpected input events: %+v", auditRecorder.events)
	}
	for _, event := range auditRecorder.events {
		if event.ActorID != actorID.String() {
			t.Fatalf("unexpected actor: %+v", event)
		}
		details := fmt.Sprint(event.Details)
		if strings.Contains(details, "950") || strings.Contains(details, "1000") || strings.Contains(details, "source correction") {
			t.Fatalf("sensitive input value leaked into audit details: %+v", event)
		}
	}
}

func TestPayrollInputLifecycleEmitsAuditEvents(t *testing.T) {
	actorID := uuid.New()
	batchID := uuid.New()
	auditRecorder := &payrollAuditRecorder{}
	authorizer := payrollAuthorizer{allowedPerms: map[string]bool{"payroll:inputs:manage": true, "payroll:approve": true, "payroll:manage": true}}
	handler := PayrollHandler{Repository: mockPayrollRepository{}, Authorizer: authorizer, Audit: auditRecorder}

	request := func(path string, action func(http.ResponseWriter, *http.Request)) {
		req := httptest.NewRequest(http.MethodPost, path, nil)
		req.SetPathValue("batch_id", batchID.String())
		req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), actorID))
		rec := httptest.NewRecorder()
		action(rec, req)
		if rec.Code != http.StatusOK {
			t.Fatalf("%s status = %d, want 200", path, rec.Code)
		}
	}

	request("/api/v1/payroll/inputs/"+batchID.String()+"/approve", handler.ApprovePayrollInputBatch)
	request("/api/v1/payroll/inputs/"+batchID.String()+"/lock", handler.LockPayrollInputBatch)

	if len(auditRecorder.events) != 2 {
		t.Fatalf("audit event count = %d, want 2", len(auditRecorder.events))
	}
	if auditRecorder.events[0].Name != "payroll.input.approved" || auditRecorder.events[1].Name != "payroll.input.locked" {
		t.Fatalf("unexpected lifecycle events: %+v", auditRecorder.events)
	}
	for _, event := range auditRecorder.events {
		if event.ActorID != actorID.String() || event.Resource != batchID.String() {
			t.Fatalf("unexpected audit context: %+v", event)
		}
	}
}

func TestPayrollInputImportValidatesAndAuthorizes(t *testing.T) {
	employmentID := uuid.New()
	body := `{"period_year":2026,"period_month":10,"source":"production","idempotency_key":"batch-1","inputs":[{"employment_id":"` + employmentID.String() + `","metric_code":"production.output","value":"950","unit":"product"}]}`
	request := func(authorizer payrollAuthorizer, payload string) *httptest.ResponseRecorder {
		req := httptest.NewRequest(http.MethodPost, "/api/v1/payroll/inputs/import", bytes.NewBufferString(payload))
		req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), uuid.New()))
		rec := httptest.NewRecorder()
		(&PayrollHandler{Repository: mockPayrollRepository{}, Authorizer: authorizer}).CreatePayrollInputBatch(rec, req)
		return rec
	}
	if got := request(payrollAuthorizer{allowedPerms: map[string]bool{}}, body).Code; got != http.StatusForbidden {
		t.Fatalf("forbidden status = %d, want 403", got)
	}
	if got := request(payrollAuthorizer{}, `{"period_year":2026,"period_month":10,"source":"production","idempotency_key":"batch-1","inputs":[]}`).Code; got != http.StatusBadRequest {
		t.Fatalf("invalid status = %d, want 400", got)
	}
	if got := request(payrollAuthorizer{}, body).Code; got != http.StatusCreated {
		t.Fatalf("valid status = %d, want 201", got)
	}
}

func TestPayrollFormulaLifecycleEmitsAuditEvents(t *testing.T) {
	actorID := uuid.New()
	ruleID := uuid.New()
	auditRecorder := &payrollAuditRecorder{}
	handler := PayrollHandler{
		Repository: mockPayrollRepository{},
		Authorizer: payrollAuthorizer{allowedPerms: map[string]bool{"payroll:manage": true}},
		Audit:      auditRecorder,
	}

	createReq := httptest.NewRequest(http.MethodPost, "/api/v1/payroll/formulas", bytes.NewBufferString(`{"code":"kpi.bonus","name":"KPI Bonus","component_code":"KPI_BONUS","expression":"actual * rate","effective_from":"2026-10-01"}`))
	createReq = createReq.WithContext(httpinfra.WithUserIDForTest(createReq.Context(), actorID))
	createRec := httptest.NewRecorder()
	handler.CreatePayrollFormulaRule(createRec, createReq)
	if createRec.Code != http.StatusCreated {
		t.Fatalf("create status = %d, want 201", createRec.Code)
	}

	publishReq := httptest.NewRequest(http.MethodPost, "/api/v1/payroll/formulas/"+ruleID.String()+"/publish", nil)
	publishReq.SetPathValue("rule_id", ruleID.String())
	publishReq = publishReq.WithContext(httpinfra.WithUserIDForTest(publishReq.Context(), actorID))
	publishRec := httptest.NewRecorder()
	handler.PublishPayrollFormulaRule(publishRec, publishReq)
	if publishRec.Code != http.StatusOK {
		t.Fatalf("publish status = %d, want 200", publishRec.Code)
	}

	retireReq := httptest.NewRequest(http.MethodPost, "/api/v1/payroll/formulas/"+ruleID.String()+"/retire", nil)
	retireReq.SetPathValue("rule_id", ruleID.String())
	retireReq = retireReq.WithContext(httpinfra.WithUserIDForTest(retireReq.Context(), actorID))
	retireRec := httptest.NewRecorder()
	handler.RetirePayrollFormulaRule(retireRec, retireReq)
	if retireRec.Code != http.StatusOK {
		t.Fatalf("retire status = %d, want 200", retireRec.Code)
	}

	want := []string{"payroll.formula.created", "payroll.formula.published", "payroll.formula.retired"}
	if len(auditRecorder.events) != len(want) {
		t.Fatalf("audit event count = %d, want %d", len(auditRecorder.events), len(want))
	}
	for index, event := range auditRecorder.events {
		if event.Name != want[index] || event.ActorID != actorID.String() {
			t.Fatalf("unexpected formula audit event: %+v", event)
		}
	}
}

func TestPayrollFormulaVersionEndpointValidatesAndAudits(t *testing.T) {
	actorID := uuid.New()
	ruleID := uuid.New()
	auditRecorder := &payrollAuditRecorder{}
	handler := PayrollHandler{
		Repository: mockPayrollRepository{},
		Authorizer: payrollAuthorizer{allowedPerms: map[string]bool{"payroll:manage": true}},
		Audit:      auditRecorder,
	}
	request := httptest.NewRequest(http.MethodPost, "/api/v1/payroll/formulas/"+ruleID.String()+"/versions", bytes.NewBufferString(`{"expression":"actual * rate","effective_from":"2026-11-01"}`))
	request.SetPathValue("rule_id", ruleID.String())
	request = request.WithContext(httpinfra.WithUserIDForTest(request.Context(), actorID))
	response := httptest.NewRecorder()
	handler.CreatePayrollFormulaRuleVersion(response, request)
	if response.Code != http.StatusCreated {
		t.Fatalf("create version status = %d, want 201", response.Code)
	}
	if len(auditRecorder.events) != 1 || auditRecorder.events[0].Name != "payroll.formula.version_created" {
		t.Fatalf("unexpected version audit events: %+v", auditRecorder.events)
	}

	invalid := httptest.NewRequest(http.MethodPost, "/api/v1/payroll/formulas/"+ruleID.String()+"/versions", bytes.NewBufferString(`{"expression":"DROP TABLE payroll_inputs","effective_from":"2026-11-01"}`))
	invalid.SetPathValue("rule_id", ruleID.String())
	invalid = invalid.WithContext(httpinfra.WithUserIDForTest(invalid.Context(), actorID))
	invalidResponse := httptest.NewRecorder()
	handler.CreatePayrollFormulaRuleVersion(invalidResponse, invalid)
	if invalidResponse.Code != http.StatusBadRequest {
		t.Fatalf("unsafe version status = %d, want 400", invalidResponse.Code)
	}
	if len(auditRecorder.events) != 1 {
		t.Fatalf("invalid version emitted audit event: %+v", auditRecorder.events)
	}
}

func TestPayrollFormulaRuleEndpointsRequireManage(t *testing.T) {
	req := httptest.NewRequest(http.MethodPost, "/api/v1/payroll/formulas", bytes.NewBufferString(`{"code":"kpi.bonus","name":"KPI Bonus","component_code":"KPI_BONUS","expression":"actual * rate","effective_from":"2026-10-01"}`))
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), uuid.New()))
	rec := httptest.NewRecorder()
	(&PayrollHandler{Repository: mockPayrollRepository{}, Authorizer: payrollAuthorizer{allowedPerms: map[string]bool{}}}).CreatePayrollFormulaRule(rec, req)
	if rec.Code != http.StatusForbidden {
		t.Fatalf("create forbidden status = %d, want 403", rec.Code)
	}
}

func TestPayrollFormulaPreviewRequiresManageAndEvaluates(t *testing.T) {
	request := func(permission bool, body string) *httptest.ResponseRecorder {
		req := httptest.NewRequest(http.MethodPost, "/api/v1/payroll/formulas/preview", bytes.NewBufferString(body))
		req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), uuid.New()))
		rec := httptest.NewRecorder()
		(&PayrollHandler{Repository: mockPayrollRepository{}, Authorizer: payrollAuthorizer{allowedPerms: map[string]bool{"payroll:manage": permission}}}).PreviewPayrollFormula(rec, req)
		return rec
	}
	if got := request(false, `{"expression":"1 + 1","values":{}}`).Code; got != http.StatusForbidden {
		t.Fatalf("forbidden status = %d, want 403", got)
	}
	valid := request(true, `{"expression":"min(actual / target, 1.2)","values":{"actual":"950","target":"1000"}}`)
	if valid.Code != http.StatusOK || !contains(valid.Body.String(), `"value":"0.95"`) {
		t.Fatalf("preview = %d %s, want 200 with 0.95", valid.Code, valid.Body.String())
	}
	if got := request(true, `{"expression":"DROP TABLE payroll_inputs","values":{}}`).Code; got != http.StatusBadRequest {
		t.Fatalf("unsafe status = %d, want 400", got)
	}
}

func TestPayrollInputLifecycleEndpoints(t *testing.T) {
	batchID := uuid.New()
	actorID := uuid.New()
	request := func(path, permission string) *httptest.ResponseRecorder {
		req := httptest.NewRequest(http.MethodPost, path, nil)
		req.SetPathValue("batch_id", batchID.String())
		req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), actorID))
		rec := httptest.NewRecorder()
		(&PayrollHandler{Repository: mockPayrollRepository{}, Authorizer: payrollAuthorizer{allowedPerms: map[string]bool{permission: true}}}).ApprovePayrollInputBatch(rec, req)
		return rec
	}
	if got := request("/api/v1/payroll/inputs/"+batchID.String()+"/approve", "payroll:approve").Code; got != http.StatusOK {
		t.Fatalf("approve status = %d, want 200", got)
	}
	lockedReq := httptest.NewRequest(http.MethodPost, "/api/v1/payroll/inputs/"+batchID.String()+"/lock", nil)
	lockedReq.SetPathValue("batch_id", batchID.String())
	lockedReq = lockedReq.WithContext(httpinfra.WithUserIDForTest(lockedReq.Context(), actorID))
	lockedRec := httptest.NewRecorder()
	(&PayrollHandler{Repository: mockPayrollRepository{}, Authorizer: payrollAuthorizer{allowedPerms: map[string]bool{"payroll:manage": true}}}).LockPayrollInputBatch(lockedRec, lockedReq)
	if lockedRec.Code != http.StatusOK {
		t.Fatalf("lock status = %d, want 200", lockedRec.Code)
	}
}

func TestPayrollImpactEndpointEnforcesSensitiveReadAndValidatesRuns(t *testing.T) {
	employmentID := uuid.New()
	request := func(permission bool, query string) *httptest.ResponseRecorder {
		req := httptest.NewRequest(http.MethodGet, "/api/v1/employments/"+employmentID.String()+"/payroll/impact?"+query, nil)
		req.SetPathValue("id", employmentID.String())
		req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), uuid.New()))
		rec := httptest.NewRecorder()
		(&PayrollHandler{Repository: mockPayrollRepository{}, Authorizer: payrollAuthorizer{allowedPerms: map[string]bool{"payroll:sensitive:read": permission}}}).ComparePayrollImpact(rec, req)
		return rec
	}
	validQuery := "year=2026&month=6&before_run_id=" + uuid.NewString() + "&after_run_id=" + uuid.NewString()
	if got := request(false, validQuery).Code; got != http.StatusForbidden {
		t.Fatalf("forbidden status = %d, want 403", got)
	}
	if got := request(true, "year=2026&month=6").Code; got != http.StatusBadRequest {
		t.Fatalf("invalid run query status = %d, want 400", got)
	}
	batchRequest := httptest.NewRequest(http.MethodPost, "/api/v1/payroll/impact/batch", bytes.NewBufferString(`{"employment_ids":["`+employmentID.String()+`","`+employmentID.String()+`"],"year":2026,"month":6,"before_run_id":"`+uuid.NewString()+`","after_run_id":"`+uuid.NewString()+`"}`))
	batchRequest = batchRequest.WithContext(httpinfra.WithUserIDForTest(batchRequest.Context(), uuid.New()))
	batchRecorder := httptest.NewRecorder()
	(&PayrollHandler{Repository: mockPayrollRepository{}, Authorizer: payrollAuthorizer{allowedPerms: map[string]bool{"payroll:sensitive:read": true}}}).ComparePayrollImpactBatch(batchRecorder, batchRequest)
	if batchRecorder.Code != http.StatusBadRequest {
		t.Fatalf("duplicate employment IDs status = %d, want 400", batchRecorder.Code)
	}
	if got := request(true, validQuery).Code; got != http.StatusOK {
		t.Fatalf("valid impact status = %d, want 200", got)
	}
}

func TestPayrollBatchRejectsDuplicateEmploymentIDs(t *testing.T) {
	userID := uuid.New()
	employmentID := uuid.New()
	handler := PayrollHandler{Repository: mockPayrollRepository{}, Authorizer: payrollAuthorizer{allowedPerms: map[string]bool{"payroll:calculate": true}}}
	req := httptest.NewRequest(http.MethodPost, "/api/v1/payroll/calculate-batch", bytes.NewBufferString(`{"employment_ids":["`+employmentID.String()+`","`+employmentID.String()+`"],"year":2026,"month":1}`))
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), userID))
	rec := httptest.NewRecorder()
	handler.CalculateBatchPayroll(rec, req)
	if rec.Code != http.StatusBadRequest {
		t.Fatalf("duplicate batch IDs status = %d, want 400", rec.Code)
	}
}

func TestPayrollImpactBatchRequiresSensitiveRead(t *testing.T) {
	employmentID := uuid.New()
	handler := PayrollHandler{Repository: mockPayrollRepository{}, Authorizer: payrollAuthorizer{allowedPerms: map[string]bool{}}}
	req := httptest.NewRequest(http.MethodPost, "/api/v1/payroll/impact/batch", bytes.NewBufferString(`{"employment_ids":["`+employmentID.String()+`"],"year":2026,"month":6,"before_run_id":"`+uuid.NewString()+`","after_run_id":"`+uuid.NewString()+`"}`))
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), uuid.New()))
	rec := httptest.NewRecorder()
	handler.ComparePayrollImpactBatch(rec, req)
	if rec.Code != http.StatusForbidden {
		t.Fatalf("aggregate impact without sensitive-read permission = %d, want 403", rec.Code)
	}
}

func TestPayrollImpactBatchRejectsMoreThanMaximumEmploymentIDs(t *testing.T) {
	employmentIDs := make([]string, 101)
	for index := range employmentIDs {
		employmentIDs[index] = uuid.NewString()
	}
	body, err := json.Marshal(map[string]any{
		"employment_ids": employmentIDs,
		"year":           2026,
		"month":          6,
		"before_run_id":  uuid.NewString(),
		"after_run_id":   uuid.NewString(),
	})
	if err != nil {
		t.Fatalf("marshal oversized impact request: %v", err)
	}
	handler := PayrollHandler{Repository: mockPayrollRepository{}, Authorizer: payrollAuthorizer{allowedPerms: map[string]bool{"payroll:sensitive:read": true}}}
	req := httptest.NewRequest(http.MethodPost, "/api/v1/payroll/impact/batch", bytes.NewReader(body))
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), uuid.New()))
	rec := httptest.NewRecorder()
	handler.ComparePayrollImpactBatch(rec, req)
	if rec.Code != http.StatusBadRequest {
		t.Fatalf("aggregate impact with 101 employment IDs = %d, want 400", rec.Code)
	}
}

func TestPayrollCalculationEndpoints(t *testing.T) {
	calcAuthorizer := payrollAuthorizer{
		isMember:     true,
		allowedPerms: map[string]bool{"payroll:calculate": true},
	}
	handler := PayrollHandler{Repository: mockPayrollRepository{}, Authorizer: calcAuthorizer}
	empID := uuid.New()

	// 9. POST /employees/{id}/calculate-payroll (200 OK)
	req := httptest.NewRequest(http.MethodPost, "/api/v1/employees/"+empID.String()+"/calculate-payroll?year=2026&month=1", nil)
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
	req = httptest.NewRequest(http.MethodPost, "/api/v1/employees/"+empID.String()+"/calculate-payroll?year=2026&month=1", nil)
	req.SetPathValue("id", empID.String())
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), uuid.New()))
	rec = httptest.NewRecorder()
	noPermHandler.CalculatePayroll(rec, req)
	if rec.Code != http.StatusForbidden {
		t.Fatalf("expected 403, got %d", rec.Code)
	}

	// 9c. POST calculate-payroll invalid month (400)
	req = httptest.NewRequest(http.MethodPost, "/api/v1/employees/"+empID.String()+"/calculate-payroll?year=2026&month=13", nil)
	req.SetPathValue("id", empID.String())
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), uuid.New()))
	rec = httptest.NewRecorder()
	handler.CalculatePayroll(rec, req)
	if rec.Code != http.StatusBadRequest {
		t.Fatalf("expected 400, got %d", rec.Code)
	}

	// 9d. POST calculate-payroll missing employee (404)
	missingEmpID := uuid.MustParse("00000000-0000-0000-0000-000000000044")
	req = httptest.NewRequest(http.MethodPost, "/api/v1/employees/"+missingEmpID.String()+"/calculate-payroll?year=2026&month=1", nil)
	req.SetPathValue("id", missingEmpID.String())
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), uuid.New()))
	rec = httptest.NewRecorder()
	handler.CalculatePayroll(rec, req)
	if rec.Code != http.StatusNotFound {
		t.Fatalf("expected 404, got %d", rec.Code)
	}

	// 9e. POST calculate-payroll no active employment (400)
	noActiveEmpID := uuid.MustParse("00000000-0000-0000-0000-000000000040")
	req = httptest.NewRequest(http.MethodPost, "/api/v1/employees/"+noActiveEmpID.String()+"/calculate-payroll?year=2026&month=1", nil)
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
	req = httptest.NewRequest(http.MethodPost, "/api/v1/payroll/calculate-batch", bytes.NewBufferString(batchBody))
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
	if batchResp.Results[0].EmploymentID != validEmp1 || batchResp.Results[1].EmploymentID != missingEmp {
		t.Fatalf("batch results must preserve request order: got %v, %v", batchResp.Results[0].EmploymentID, batchResp.Results[1].EmploymentID)
	}

	// 10b. POST calculate-batch empty list (400)
	req = httptest.NewRequest(http.MethodPost, "/api/v1/payroll/calculate-batch", bytes.NewBufferString(`{"employment_ids":[],"year":2026,"month":1}`))
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), uuid.New()))
	rec = httptest.NewRecorder()
	handler.CalculateBatchPayroll(rec, req)
	if rec.Code != http.StatusBadRequest {
		t.Fatalf("expected 400, got %d", rec.Code)
	}

	// 10c. POST calculate-batch forbidden without permission (403)
	req = httptest.NewRequest(http.MethodPost, "/api/v1/payroll/calculate-batch", bytes.NewBufferString(batchBody))
	req = req.WithContext(httpinfra.WithUserIDForTest(req.Context(), uuid.New()))
	rec = httptest.NewRecorder()
	noPermHandler.CalculateBatchPayroll(rec, req)
	if rec.Code != http.StatusForbidden {
		t.Fatalf("expected 403, got %d", rec.Code)
	}
}
