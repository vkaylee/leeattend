package api

import (
	"context"
	"encoding/json"
	"errors"
	"net/http"
	"strconv"
	"strings"
	"time"

	"github.com/google/uuid"
	"github.com/jackc/pgx/v5"
	"github.com/shopspring/decimal"
	"github.com/vkaylee/leeattend/apiserver/internal/apperror"
	httpinfra "github.com/vkaylee/leeattend/apiserver/internal/infra/http"
)

var (
	ErrPayrollValidation       = errors.New("payroll: validation failed")
	ErrInvalidBaseGroup        = errors.New("payroll: invalid base group")
	ErrComponentNotFound       = errors.New("payroll: component not found")
	ErrComponentConflict       = errors.New("payroll: component already exists")
	ErrPayrollSettingsNotFound = errors.New("payroll: settings not found")
	ErrIncentivePolicyNotFound = errors.New("payroll: incentive policy not found")
	ErrRuleNotFound            = errors.New("payroll: rule not found")
	ErrTierNotFound            = errors.New("payroll: tier not found")
	ErrAssignmentNotFound      = errors.New("payroll: assignment not found")
	ErrAssignmentConflict      = errors.New("payroll: assignment conflict")
)

type CreateSalaryComponentRequest struct {
	Name         string       `json:"name"`
	Code         string       `json:"code"`
	BaseGroupIDs *[]uuid.UUID `json:"base_group_ids,omitempty"`
}

type EmploymentSalaryComponentResponse struct {
	ID                    uuid.UUID       `json:"id"`
	EmploymentID          uuid.UUID       `json:"employment_id"`
	ComponentDefinitionID uuid.UUID       `json:"component_definition_id"`
	Amount                decimal.Decimal `json:"amount"`
	EffectiveDate         string          `json:"effective_date"`
	Name                  string          `json:"name"`
	Code                  string          `json:"code"`
}

type AssignSalaryComponentRequest struct {
	ComponentDefinitionID uuid.UUID       `json:"component_definition_id"`
	Amount                decimal.Decimal `json:"amount"`
	EffectiveDate         string          `json:"effective_date"`
}

type SalaryBaseGroup struct {
	ID          uuid.UUID  `json:"id"`
	WorkspaceID *uuid.UUID `json:"workspace_id"`
	Name        string     `json:"name"`
	Slug        string     `json:"slug"`
	Description *string    `json:"description"`
	IsSystem    bool       `json:"is_system"`
	CreatedAt   string     `json:"created_at"`
	UpdatedAt   string     `json:"updated_at"`
}

type SalaryBaseGroupMember struct {
	ComponentDefinitionID uuid.UUID `json:"component_definition_id"`
	BaseGroupID           uuid.UUID `json:"base_group_id"`
	CreatedAt             string    `json:"created_at"`
}

type SalaryBasesResponse struct {
	Groups  []SalaryBaseGroup       `json:"groups"`
	Members []SalaryBaseGroupMember `json:"members"`
}

type CreateSalaryBaseRequest struct {
	Name        string  `json:"name"`
	Slug        string  `json:"slug"`
	Description *string `json:"description,omitempty"`
}

type AssignSalaryBaseMemberRequest struct {
	ComponentDefinitionID uuid.UUID `json:"component_definition_id"`
}

type PayrollQuery struct {
	Year  int `json:"year"`
	Month int `json:"month"`
}
type PayslipRecord struct {
	ID                       uuid.UUID       `json:"id"`
	WorkspaceID              uuid.UUID       `json:"workspace_id"`
	EmploymentID             uuid.UUID       `json:"employment_id"`
	PeriodStart              string          `json:"period_start"`
	PeriodEnd                string          `json:"period_end"`
	BasePayAmount            decimal.Decimal `json:"base_pay_amount"`
	OvertimePayAmount        decimal.Decimal `json:"overtime_pay_amount"`
	DeductionsAmount         decimal.Decimal `json:"deductions_amount"`
	NetPayAmount             decimal.Decimal `json:"net_pay_amount"`
	Status                   string          `json:"status"`
	StandardUnits            int             `json:"standard_units"`
	OvertimeUnits            int             `json:"overtime_units"`
	HolidayWorkPremiumAmount decimal.Decimal `json:"holiday_work_premium_amount"`
	CreatedAt                string          `json:"created_at"`
	UpdatedAt                string          `json:"updated_at"`
}
type PayrollSettings struct {
	WorkspaceID        uuid.UUID        `json:"workspace_id"`
	StdWorkingDayMode  string           `json:"std_working_day_mode"`
	FixedStdDays       decimal.Decimal  `json:"fixed_std_days"`
	WorkingDaysPerWeek int              `json:"working_days_per_week"`
	HoursPerDay        decimal.Decimal  `json:"hours_per_day"`
	MaxOtHoursPerDay   *decimal.Decimal `json:"max_ot_hours_per_day"`
	MaxOtHoursPerMonth *decimal.Decimal `json:"max_ot_hours_per_month"`
	EffectiveDate      string           `json:"effective_date"`
	CreatedAt          string           `json:"created_at"`
	UpdatedAt          string           `json:"updated_at"`
}

type UpdatePayrollSettingsRequest struct {
	StdWorkingDayMode  *string           `json:"std_working_day_mode,omitempty"`
	FixedStdDays       *decimal.Decimal  `json:"fixed_std_days,omitempty"`
	WorkingDaysPerWeek *int              `json:"working_days_per_week,omitempty"`
	HoursPerDay        *decimal.Decimal  `json:"hours_per_day,omitempty"`
	MaxOtHoursPerDay   **decimal.Decimal `json:"max_ot_hours_per_day,omitempty"`
	MaxOtHoursPerMonth **decimal.Decimal `json:"max_ot_hours_per_month,omitempty"`
	EffectiveDate      *string           `json:"effective_date,omitempty"`
}

type EmployeePayrollConfig struct {
	ID             uuid.UUID       `json:"id"`
	EmploymentID   uuid.UUID       `json:"employment_id"`
	StdWorkingDays decimal.Decimal `json:"std_working_days"`
	HoursPerDay    decimal.Decimal `json:"hours_per_day"`
	EffectiveDate  string          `json:"effective_date"`
	CreatedAt      string          `json:"created_at"`
}

type SetEmployeePayrollConfigRequest struct {
	StdWorkingDays decimal.Decimal `json:"std_working_days"`
	HoursPerDay    decimal.Decimal `json:"hours_per_day"`
	EffectiveDate  string          `json:"effective_date"`
}

type IncentiveTier struct {
	ID           uuid.UUID       `json:"id"`
	PolicyID     uuid.UUID       `json:"policy_id"`
	Priority     int             `json:"priority"`
	PayoutAmount decimal.Decimal `json:"payout_amount"`
	RuleConfig   json.RawMessage `json:"rule_config"`
}

type CreateIncentiveTierRequest struct {
	Priority     int             `json:"priority"`
	PayoutAmount decimal.Decimal `json:"payout_amount"`
	RuleConfig   json.RawMessage `json:"rule_config"`
}

type PayrollIncentivePolicy struct {
	ID            uuid.UUID       `json:"id"`
	WorkspaceID   uuid.UUID       `json:"workspace_id"`
	Name          string          `json:"name"`
	BaseAmount    decimal.Decimal `json:"base_amount"`
	PayoutType    string          `json:"payout_type"`
	IsActive      bool            `json:"is_active"`
	CreatedAt     string          `json:"created_at"`
	EffectiveFrom string          `json:"effective_from"`
	EffectiveTo   *string         `json:"effective_to"`
}

type PayrollIncentivePolicyResponse struct {
	PayrollIncentivePolicy
	Tiers []IncentiveTier `json:"tiers"`
}

type CreatePayrollIncentivePolicyRequest struct {
	Name          string                       `json:"name"`
	BaseAmount    decimal.Decimal              `json:"base_amount"`
	PayoutType    string                       `json:"payout_type"`
	EffectiveFrom string                       `json:"effective_from"`
	Tiers         []CreateIncentiveTierRequest `json:"tiers"`
}
type CreatePayrollRuleDefinitionRequest struct {
	Name              string          `json:"name"`
	Description       *string         `json:"description,omitempty"`
	Code              string          `json:"code"`
	Category          string          `json:"category"`
	AppliesTo         string          `json:"applies_to"`
	CalculationMethod string          `json:"calculation_method"`
	RuleConfig        json.RawMessage `json:"rule_config"`
	IsMandatory       *bool           `json:"is_mandatory,omitempty"`
	DisplayOrder      *int            `json:"display_order,omitempty"`
}

type PayrollRuleDefinition struct {
	ID                uuid.UUID       `json:"id"`
	WorkspaceID       uuid.UUID       `json:"workspace_id"`
	Name              string          `json:"name"`
	Description       *string         `json:"description"`
	Code              string          `json:"code"`
	Category          string          `json:"category"`
	AppliesTo         string          `json:"applies_to"`
	CalculationMethod string          `json:"calculation_method"`
	RuleConfig        json.RawMessage `json:"rule_config"`
	IsMandatory       bool            `json:"is_mandatory"`
	IsActive          bool            `json:"is_active"`
	DisplayOrder      int             `json:"display_order"`
	CreatedAt         string          `json:"created_at"`
	UpdatedAt         string          `json:"updated_at"`
}

type CreatePayrollRuleTierRequest struct {
	TierOrder   int              `json:"tier_order"`
	MinAmount   decimal.Decimal  `json:"min_amount"`
	MaxAmount   *decimal.Decimal `json:"max_amount,omitempty"`
	Rate        decimal.Decimal  `json:"rate"`
	FixedAmount *decimal.Decimal `json:"fixed_amount,omitempty"`
}

type PayrollRuleTier struct {
	ID               uuid.UUID        `json:"id"`
	RuleDefinitionID uuid.UUID        `json:"rule_definition_id"`
	TierOrder        int              `json:"tier_order"`
	MinAmount        decimal.Decimal  `json:"min_amount"`
	MaxAmount        *decimal.Decimal `json:"max_amount"`
	Rate             decimal.Decimal  `json:"rate"`
	FixedAmount      decimal.Decimal  `json:"fixed_amount"`
	CreatedAt        string           `json:"created_at"`
}

type AssignPayrollRuleRequest struct {
	AssigneeType   string           `json:"assignee_type"`
	AssigneeID     *uuid.UUID       `json:"assignee_id,omitempty"`
	EffectiveFrom  string           `json:"effective_from"`
	EffectiveTo    *string          `json:"effective_to,omitempty"`
	OverrideConfig *json.RawMessage `json:"override_config,omitempty"`
}

type PayrollRuleAssignment struct {
	ID               uuid.UUID        `json:"id"`
	WorkspaceID      uuid.UUID        `json:"workspace_id"`
	RuleDefinitionID uuid.UUID        `json:"rule_definition_id"`
	AssigneeType     string           `json:"assignee_type"`
	AssigneeID       *uuid.UUID       `json:"assignee_id"`
	EffectiveFrom    string           `json:"effective_from"`
	EffectiveTo      *string          `json:"effective_to"`
	OverrideConfig   *json.RawMessage `json:"override_config"`
	CreatedAt        string           `json:"created_at"`
}

type PayslipSegmentResponse struct {
	ID                 uuid.UUID       `json:"id"`
	PayslipID          uuid.UUID       `json:"payslip_id"`
	StartDate          string          `json:"start_date"`
	EndDate            string          `json:"end_date"`
	DepartmentID       *uuid.UUID      `json:"department_id"`
	AppliedBaseSalary  decimal.Decimal `json:"applied_base_salary"`
	AppliedStdDays     decimal.Decimal `json:"applied_std_days"`
	AppliedHoursPerDay decimal.Decimal `json:"applied_hours_per_day"`
	ActualWorkDays     decimal.Decimal `json:"actual_work_days"`
	PaidWorkDays       decimal.Decimal `json:"paid_work_days"`
	Amount             decimal.Decimal `json:"amount"`
	Reason             string          `json:"reason"`
	StdDaysSource      *string         `json:"std_days_source,omitempty"`
}

type IncentiveEvaluationResult struct {
	PolicyName     string          `json:"policy_name"`
	Amount         decimal.Decimal `json:"amount"`
	StandardAmount decimal.Decimal `json:"standard_amount"`
	TierID         *uuid.UUID      `json:"tier_id"`
}

type PayslipRuleEvaluationRecord struct {
	ID        uuid.UUID       `json:"id"`
	PayslipID uuid.UUID       `json:"payslip_id"`
	RuleCode  string          `json:"rule_code"`
	Amount    decimal.Decimal `json:"amount"`
	CreatedAt string          `json:"created_at"`
}

type FullPayslipResponse struct {
	PayslipRecord
	Segments         []PayslipSegmentResponse        `json:"segments"`
	Incentives       []IncentiveEvaluationResult     `json:"incentives"`
	RuleEvaluations  []PayslipRuleEvaluationRecord   `json:"rule_evaluations"`
	SalaryComponents []SalaryComponentWithDefinition `json:"salary_components"`
}

type BatchCalculatePayrollRequest struct {
	EmploymentIDs []uuid.UUID `json:"employment_ids"`
	EmployeeIDs   []uuid.UUID `json:"employee_ids,omitempty"`
	Year          int         `json:"year"`
	Month         int         `json:"month"`
}

type BatchCalculateResult struct {
	EmploymentID uuid.UUID      `json:"employment_id"`
	EmployeeID   uuid.UUID      `json:"employee_id"`
	Status       string         `json:"status"`
	Payslip      *PayslipRecord `json:"payslip"`
	ErrorMessage *string        `json:"error_message"`
}

type BatchCalculatePayrollResponse struct {
	Results []BatchCalculateResult `json:"results"`
}

type PayrollRepository interface {
	ListWorkspacePayslips(context.Context, uuid.UUID, time.Time, time.Time) ([]PayslipRecord, error)
	ListEmployeePayslips(context.Context, uuid.UUID, uuid.UUID) ([]PayslipRecord, error)
	ListSalaryComponents(context.Context, uuid.UUID) ([]SalaryComponentDefinition, error)
	CreateSalaryComponent(context.Context, uuid.UUID, CreateSalaryComponentRequest) (SalaryComponentDefinition, error)
	ListEmploymentSalaryComponents(context.Context, uuid.UUID, uuid.UUID) ([]EmploymentSalaryComponentResponse, error)
	AssignEmploymentSalaryComponent(context.Context, uuid.UUID, uuid.UUID, AssignSalaryComponentRequest) (EmployeeSalaryComponent, error)
	ListSalaryBases(context.Context, uuid.UUID) (SalaryBasesResponse, error)
	CreateSalaryBase(context.Context, uuid.UUID, CreateSalaryBaseRequest) (SalaryBaseGroup, error)
	AssignBaseMember(context.Context, uuid.UUID, uuid.UUID, uuid.UUID) error
	UnassignBaseMember(context.Context, uuid.UUID, uuid.UUID, uuid.UUID) error
	GetPayrollSettings(context.Context, uuid.UUID) (*PayrollSettings, error)
	GetPayrollSettingsHistory(context.Context, uuid.UUID) ([]PayrollSettings, error)
	UpdatePayrollSettings(context.Context, uuid.UUID, UpdatePayrollSettingsRequest) (PayrollSettings, error)
	GetEmployeePayrollConfigs(context.Context, uuid.UUID, uuid.UUID) ([]EmployeePayrollConfig, error)
	SetEmployeePayrollConfig(context.Context, uuid.UUID, uuid.UUID, SetEmployeePayrollConfigRequest, uuid.UUID) (EmployeePayrollConfig, error)
	GetIncentivePolicies(context.Context, uuid.UUID) ([]PayrollIncentivePolicyResponse, error)
	CreateIncentivePolicy(context.Context, uuid.UUID, CreatePayrollIncentivePolicyRequest) (PayrollIncentivePolicy, error)
	DeleteIncentivePolicy(context.Context, uuid.UUID, uuid.UUID) error
	CloneIncentivePolicy(context.Context, uuid.UUID, uuid.UUID) (PayrollIncentivePolicy, error)
	ListPayrollRules(context.Context, uuid.UUID) ([]PayrollRuleDefinition, error)
	CreatePayrollRule(context.Context, uuid.UUID, CreatePayrollRuleDefinitionRequest) (PayrollRuleDefinition, error)
	ListPayrollRuleTiers(context.Context, uuid.UUID, uuid.UUID) ([]PayrollRuleTier, error)
	CreatePayrollRuleTier(context.Context, uuid.UUID, uuid.UUID, CreatePayrollRuleTierRequest) (PayrollRuleTier, error)
	DeletePayrollRuleTier(context.Context, uuid.UUID, uuid.UUID, uuid.UUID) error
	ListPayrollRuleAssignments(context.Context, uuid.UUID, uuid.UUID) ([]PayrollRuleAssignment, error)
	AssignPayrollRule(context.Context, uuid.UUID, uuid.UUID, AssignPayrollRuleRequest) (PayrollRuleAssignment, error)
	DeletePayrollRuleAssignment(context.Context, uuid.UUID, uuid.UUID, uuid.UUID) error
	CalculateEmployeePayroll(context.Context, uuid.UUID, uuid.UUID, int, int) (FullPayslipResponse, error)
	CalculateBatchPayroll(context.Context, uuid.UUID, BatchCalculatePayrollRequest) (BatchCalculatePayrollResponse, error)
}
type PayrollHandler struct {
	Repository PayrollRepository
	Authorizer WorkspaceAuthorizer
}

func (h PayrollHandler) ListWorkspace(w http.ResponseWriter, r *http.Request) {
	wsID, ok := organizationIDFromContext(r)
	if !ok || !h.allowed(r, wsID, "payroll:read") {
		h.forbidden(w, r)
		return
	}
	start, end, err := parsePayrollPeriod(r)
	if err != nil {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	rows, err := h.Repository.ListWorkspacePayslips(r.Context(), wsID, start, end)
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusOK, rows)
}
func (h PayrollHandler) ListEmployee(w http.ResponseWriter, r *http.Request) {
	wsID, ok := organizationIDFromContext(r)
	empID, empOK := pathUUID(r, "employee_id")
	if !ok || !empOK || !h.allowed(r, wsID, "payroll:read") {
		h.forbidden(w, r)
		return
	}
	rows, err := h.Repository.ListEmployeePayslips(r.Context(), wsID, empID)
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusOK, rows)
}
func (h PayrollHandler) ListSalaryComponents(w http.ResponseWriter, r *http.Request) {
	wsID, ok := organizationIDFromContext(r)
	if !ok || !h.allowedRead(r, wsID) {
		h.forbidden(w, r)
		return
	}
	rows, err := h.Repository.ListSalaryComponents(r.Context(), wsID)
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	if rows == nil {
		rows = []SalaryComponentDefinition{}
	}
	apperror.WriteJSON(w, http.StatusOK, rows)
}

func (h PayrollHandler) CreateSalaryComponent(w http.ResponseWriter, r *http.Request) {
	wsID, ok := organizationIDFromContext(r)
	if !ok || !h.allowedManage(r, wsID) {
		h.forbidden(w, r)
		return
	}
	req, err := DecodeJSON[CreateSalaryComponentRequest](r, 1<<20)
	if err != nil || strings.TrimSpace(req.Name) == "" || strings.TrimSpace(req.Code) == "" {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	comp, err := h.Repository.CreateSalaryComponent(r.Context(), wsID, req)
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusCreated, comp)
}

func (h PayrollHandler) ListEmploymentSalaryComponents(w http.ResponseWriter, r *http.Request) {
	wsID, ok := organizationIDFromContext(r)
	empID, empOK := employmentIDFromRequest(r)
	if !ok || !empOK || !h.allowedRead(r, wsID) {
		h.forbidden(w, r)
		return
	}
	comps, err := h.Repository.ListEmploymentSalaryComponents(r.Context(), wsID, empID)
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	if comps == nil {
		comps = []EmploymentSalaryComponentResponse{}
	}
	apperror.WriteJSON(w, http.StatusOK, comps)
}

func (h PayrollHandler) AssignEmploymentSalaryComponent(w http.ResponseWriter, r *http.Request) {
	wsID, ok := organizationIDFromContext(r)
	empID, empOK := employmentIDFromRequest(r)
	if !ok || !empOK || !h.allowedManage(r, wsID) {
		h.forbidden(w, r)
		return
	}
	req, err := DecodeJSON[AssignSalaryComponentRequest](r, 1<<20)
	if err != nil || req.ComponentDefinitionID == uuid.Nil {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	if _, err := time.Parse("2006-01-02", req.EffectiveDate); err != nil {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	comp, err := h.Repository.AssignEmploymentSalaryComponent(r.Context(), wsID, empID, req)
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusCreated, comp)
}

func (h PayrollHandler) ListSalaryBases(w http.ResponseWriter, r *http.Request) {
	wsID, ok := organizationIDFromContext(r)
	if !ok || !h.allowedRead(r, wsID) {
		h.forbidden(w, r)
		return
	}
	bases, err := h.Repository.ListSalaryBases(r.Context(), wsID)
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	if bases.Groups == nil {
		bases.Groups = []SalaryBaseGroup{}
	}
	if bases.Members == nil {
		bases.Members = []SalaryBaseGroupMember{}
	}
	apperror.WriteJSON(w, http.StatusOK, bases)
}

func (h PayrollHandler) CreateSalaryBase(w http.ResponseWriter, r *http.Request) {
	wsID, ok := organizationIDFromContext(r)
	if !ok || !h.allowedManage(r, wsID) {
		h.forbidden(w, r)
		return
	}
	req, err := DecodeJSON[CreateSalaryBaseRequest](r, 1<<20)
	if err != nil || strings.TrimSpace(req.Name) == "" || strings.TrimSpace(req.Slug) == "" {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	base, err := h.Repository.CreateSalaryBase(r.Context(), wsID, req)
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusCreated, base)
}

func (h PayrollHandler) AssignBaseMember(w http.ResponseWriter, r *http.Request) {
	wsID, ok := organizationIDFromContext(r)
	baseID, baseOK := pathUUID(r, "base_id")
	if !ok || !baseOK || !h.allowedManage(r, wsID) {
		h.forbidden(w, r)
		return
	}
	req, err := DecodeJSON[AssignSalaryBaseMemberRequest](r, 1<<20)
	if err != nil || req.ComponentDefinitionID == uuid.Nil {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	if err := h.Repository.AssignBaseMember(r.Context(), wsID, baseID, req.ComponentDefinitionID); err != nil {
		h.writeError(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusOK, map[string]bool{"success": true})
}

func (h PayrollHandler) UnassignBaseMember(w http.ResponseWriter, r *http.Request) {
	wsID, ok := organizationIDFromContext(r)
	baseID, baseOK := pathUUID(r, "base_id")
	compID, compOK := pathUUID(r, "component_id")
	if !ok || !baseOK || !compOK || !h.allowedManage(r, wsID) {
		h.forbidden(w, r)
		return
	}
	if err := h.Repository.UnassignBaseMember(r.Context(), wsID, baseID, compID); err != nil {
		h.writeError(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusOK, map[string]bool{"success": true})
}
func (h PayrollHandler) GetPayrollSettings(w http.ResponseWriter, r *http.Request) {
	wsID, ok := organizationIDFromContext(r)
	if !ok || !h.allowedMember(r, wsID) {
		h.forbidden(w, r)
		return
	}
	settings, err := h.Repository.GetPayrollSettings(r.Context(), wsID)
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusOK, settings)
}

func (h PayrollHandler) UpdatePayrollSettings(w http.ResponseWriter, r *http.Request) {
	wsID, ok := organizationIDFromContext(r)
	if !ok || !h.allowedMember(r, wsID) {
		h.forbidden(w, r)
		return
	}
	req, err := DecodeJSON[UpdatePayrollSettingsRequest](r, 1<<20)
	if err != nil {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	if req.EffectiveDate != nil && *req.EffectiveDate != "" {
		if _, err := time.Parse("2006-01-02", *req.EffectiveDate); err != nil {
			apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
			return
		}
	}
	settings, err := h.Repository.UpdatePayrollSettings(r.Context(), wsID, req)
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusOK, settings)
}

func (h PayrollHandler) GetPayrollSettingsHistory(w http.ResponseWriter, r *http.Request) {
	wsID, ok := organizationIDFromContext(r)
	if !ok || !h.allowedMember(r, wsID) {
		h.forbidden(w, r)
		return
	}
	history, err := h.Repository.GetPayrollSettingsHistory(r.Context(), wsID)
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	if history == nil {
		history = []PayrollSettings{}
	}
	apperror.WriteJSON(w, http.StatusOK, history)
}

func (h PayrollHandler) GetEmployeePayrollConfig(w http.ResponseWriter, r *http.Request) {
	wsID, ok := organizationIDFromContext(r)
	empID, empOK := pathUUID(r, "id")
	if !ok || !empOK || !h.allowedMember(r, wsID) {
		h.forbidden(w, r)
		return
	}
	configs, err := h.Repository.GetEmployeePayrollConfigs(r.Context(), wsID, empID)
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	if configs == nil {
		configs = []EmployeePayrollConfig{}
	}
	apperror.WriteJSON(w, http.StatusOK, configs)
}

func (h PayrollHandler) SetEmployeePayrollConfig(w http.ResponseWriter, r *http.Request) {
	wsID, ok := organizationIDFromContext(r)
	empID, empOK := pathUUID(r, "id")
	if !ok || !empOK || !h.allowedMember(r, wsID) {
		h.forbidden(w, r)
		return
	}
	req, err := DecodeJSON[SetEmployeePayrollConfigRequest](r, 1<<20)
	if err != nil || req.EffectiveDate == "" {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	if _, err := time.Parse("2006-01-02", req.EffectiveDate); err != nil {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	if req.StdWorkingDays.IsNegative() || req.HoursPerDay.IsNegative() {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	userID, _ := httpinfra.UserIDFromContext(r.Context())
	config, err := h.Repository.SetEmployeePayrollConfig(r.Context(), wsID, empID, req, userID)
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusCreated, config)
}

func (h PayrollHandler) GetIncentivePolicies(w http.ResponseWriter, r *http.Request) {
	wsID, ok := organizationIDFromContext(r)
	if !ok || !h.allowedMember(r, wsID) {
		h.forbidden(w, r)
		return
	}
	policies, err := h.Repository.GetIncentivePolicies(r.Context(), wsID)
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	if policies == nil {
		policies = []PayrollIncentivePolicyResponse{}
	}
	apperror.WriteJSON(w, http.StatusOK, policies)
}

func (h PayrollHandler) CreateIncentivePolicy(w http.ResponseWriter, r *http.Request) {
	wsID, ok := organizationIDFromContext(r)
	if !ok || !h.allowedMember(r, wsID) {
		h.forbidden(w, r)
		return
	}
	req, err := DecodeJSON[CreatePayrollIncentivePolicyRequest](r, 1<<20)
	if err != nil || strings.TrimSpace(req.Name) == "" || req.EffectiveFrom == "" {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	if _, err := time.Parse("2006-01-02", req.EffectiveFrom); err != nil {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	if req.PayoutType == "" {
		req.PayoutType = "fixed"
	}
	policy, err := h.Repository.CreateIncentivePolicy(r.Context(), wsID, req)
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusCreated, policy)
}

func (h PayrollHandler) DeleteIncentivePolicy(w http.ResponseWriter, r *http.Request) {
	wsID, ok := organizationIDFromContext(r)
	policyID, pOK := pathUUID(r, "policy_id")
	if !ok || !pOK || !h.allowedMember(r, wsID) {
		h.forbidden(w, r)
		return
	}
	if err := h.Repository.DeleteIncentivePolicy(r.Context(), wsID, policyID); err != nil {
		h.writeError(w, r, err)
		return
	}
	w.WriteHeader(http.StatusNoContent)
}

func (h PayrollHandler) CloneIncentivePolicy(w http.ResponseWriter, r *http.Request) {
	wsID, ok := organizationIDFromContext(r)
	policyID, pOK := pathUUID(r, "policy_id")
	if !ok || !pOK || !h.allowedMember(r, wsID) {
		h.forbidden(w, r)
		return
	}
	policy, err := h.Repository.CloneIncentivePolicy(r.Context(), wsID, policyID)
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusCreated, policy)
}

func (h PayrollHandler) ListPayrollRules(w http.ResponseWriter, r *http.Request) {
	wsID, ok := organizationIDFromContext(r)
	if !ok || !h.allowedMember(r, wsID) {
		h.forbidden(w, r)
		return
	}
	rules, err := h.Repository.ListPayrollRules(r.Context(), wsID)
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	if rules == nil {
		rules = []PayrollRuleDefinition{}
	}
	apperror.WriteJSON(w, http.StatusOK, rules)
}

func (h PayrollHandler) CreatePayrollRule(w http.ResponseWriter, r *http.Request) {
	wsID, ok := organizationIDFromContext(r)
	if !ok || !h.allowedMember(r, wsID) {
		h.forbidden(w, r)
		return
	}
	req, err := DecodeJSON[CreatePayrollRuleDefinitionRequest](r, 1<<20)
	if err != nil || strings.TrimSpace(req.Name) == "" || strings.TrimSpace(req.Code) == "" {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	rule, err := h.Repository.CreatePayrollRule(r.Context(), wsID, req)
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusCreated, rule)
}

func (h PayrollHandler) ListPayrollRuleTiers(w http.ResponseWriter, r *http.Request) {
	wsID, ok := organizationIDFromContext(r)
	ruleID, rOK := pathUUID(r, "rule_id")
	if !ok || !rOK || !h.allowedMember(r, wsID) {
		h.forbidden(w, r)
		return
	}
	tiers, err := h.Repository.ListPayrollRuleTiers(r.Context(), wsID, ruleID)
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	if tiers == nil {
		tiers = []PayrollRuleTier{}
	}
	apperror.WriteJSON(w, http.StatusOK, tiers)
}

func (h PayrollHandler) CreatePayrollRuleTier(w http.ResponseWriter, r *http.Request) {
	wsID, ok := organizationIDFromContext(r)
	ruleID, rOK := pathUUID(r, "rule_id")
	if !ok || !rOK || !h.allowedMember(r, wsID) {
		h.forbidden(w, r)
		return
	}
	req, err := DecodeJSON[CreatePayrollRuleTierRequest](r, 1<<20)
	if err != nil || req.TierOrder < 0 || req.MinAmount.IsNegative() {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	if req.MaxAmount != nil && req.MaxAmount.LessThanOrEqual(req.MinAmount) {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	tier, err := h.Repository.CreatePayrollRuleTier(r.Context(), wsID, ruleID, req)
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusCreated, tier)
}

func (h PayrollHandler) DeletePayrollRuleTier(w http.ResponseWriter, r *http.Request) {
	wsID, ok := organizationIDFromContext(r)
	ruleID, rOK := pathUUID(r, "rule_id")
	tierID, tOK := pathUUID(r, "tier_id")
	if !ok || !rOK || !tOK || !h.allowedMember(r, wsID) {
		h.forbidden(w, r)
		return
	}
	if err := h.Repository.DeletePayrollRuleTier(r.Context(), wsID, ruleID, tierID); err != nil {
		h.writeError(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusOK, map[string]bool{"success": true})
}

func (h PayrollHandler) ListPayrollRuleAssignments(w http.ResponseWriter, r *http.Request) {
	wsID, ok := organizationIDFromContext(r)
	ruleID, rOK := pathUUID(r, "rule_id")
	if !ok || !rOK || !h.allowedMember(r, wsID) {
		h.forbidden(w, r)
		return
	}
	assignments, err := h.Repository.ListPayrollRuleAssignments(r.Context(), wsID, ruleID)
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	if assignments == nil {
		assignments = []PayrollRuleAssignment{}
	}
	apperror.WriteJSON(w, http.StatusOK, assignments)
}

func (h PayrollHandler) AssignPayrollRule(w http.ResponseWriter, r *http.Request) {
	wsID, ok := organizationIDFromContext(r)
	ruleID, rOK := pathUUID(r, "rule_id")
	if !ok || !rOK || !h.allowedMember(r, wsID) {
		h.forbidden(w, r)
		return
	}
	req, err := DecodeJSON[AssignPayrollRuleRequest](r, 1<<20)
	if err != nil || (req.AssigneeType != "workspace" && req.AssigneeType != "department" && req.AssigneeType != "employee" && req.AssigneeType != "user") {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	if req.AssigneeType != "workspace" && req.AssigneeID == nil {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	effFrom, err := time.Parse("2006-01-02", req.EffectiveFrom)
	if err != nil {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	if req.EffectiveTo != nil {
		effTo, err := time.Parse("2006-01-02", *req.EffectiveTo)
		if err != nil || effTo.Before(effFrom) {
			apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
			return
		}
	}
	assignment, err := h.Repository.AssignPayrollRule(r.Context(), wsID, ruleID, req)
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusCreated, assignment)
}

func (h PayrollHandler) DeletePayrollRuleAssignment(w http.ResponseWriter, r *http.Request) {
	wsID, ok := organizationIDFromContext(r)
	ruleID, rOK := pathUUID(r, "rule_id")
	assignmentID, aOK := pathUUID(r, "assignment_id")
	if !ok || !rOK || !aOK || !h.allowedMember(r, wsID) {
		h.forbidden(w, r)
		return
	}
	if err := h.Repository.DeletePayrollRuleAssignment(r.Context(), wsID, ruleID, assignmentID); err != nil {
		h.writeError(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusOK, map[string]bool{"success": true})
}

func (h PayrollHandler) CalculatePayroll(w http.ResponseWriter, r *http.Request) {
	wsID, ok := organizationIDFromContext(r)
	empID, eOK := pathUUID(r, "id")
	if !ok || !eOK || !h.allowedCalculate(r, wsID) {
		h.forbidden(w, r)
		return
	}
	year, err := strconv.Atoi(r.URL.Query().Get("year"))
	if err != nil {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	month, err := strconv.Atoi(r.URL.Query().Get("month"))
	if err != nil || year < 2000 || year > 2100 || month < 1 || month > 12 {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	resp, err := h.Repository.CalculateEmployeePayroll(r.Context(), wsID, empID, year, month)
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusOK, resp)
}

func (h PayrollHandler) CalculateBatchPayroll(w http.ResponseWriter, r *http.Request) {
	wsID, ok := organizationIDFromContext(r)
	if !ok || !h.allowedCalculate(r, wsID) {
		h.forbidden(w, r)
		return
	}
	req, err := DecodeJSON[BatchCalculatePayrollRequest](r, 1<<20)
	if err != nil {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	if len(req.EmploymentIDs) == 0 && len(req.EmployeeIDs) > 0 {
		req.EmploymentIDs = req.EmployeeIDs
	}
	if len(req.EmploymentIDs) == 0 || len(req.EmploymentIDs) > 500 {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	if req.Year < 2000 || req.Year > 2100 || req.Month < 1 || req.Month > 12 {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	resp, err := h.Repository.CalculateBatchPayroll(r.Context(), wsID, req)
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusOK, resp)
}

func (h PayrollHandler) allowedCalculate(r *http.Request, wsID uuid.UUID) bool {
	u, ok := httpinfra.UserIDFromContext(r.Context())
	if !ok || h.Authorizer == nil {
		return false
	}
	return h.Authorizer.HasPermission(r.Context(), u, wsID, "payroll:calculate")
}

func employmentIDFromRequest(r *http.Request) (uuid.UUID, bool) {
	if id, ok := pathUUID(r, "id"); ok {
		return id, true
	}
	return pathUUID(r, "employment_id")
}

func (h PayrollHandler) allowedRead(r *http.Request, wsID uuid.UUID) bool {
	u, ok := httpinfra.UserIDFromContext(r.Context())
	if !ok || h.Authorizer == nil {
		return false
	}
	if h.Authorizer.HasPermission(r.Context(), u, wsID, "payroll:read") {
		return true
	}
	if memAuth, ok := h.Authorizer.(interface {
		IsMember(context.Context, uuid.UUID, uuid.UUID) (bool, error)
	}); ok {
		m, err := memAuth.IsMember(r.Context(), u, wsID)
		return err == nil && m
	}
	return false
}
func (h PayrollHandler) allowedMember(r *http.Request, wsID uuid.UUID) bool {
	u, ok := httpinfra.UserIDFromContext(r.Context())
	if !ok || h.Authorizer == nil {
		return false
	}
	if memAuth, ok := h.Authorizer.(interface {
		IsMember(context.Context, uuid.UUID, uuid.UUID) (bool, error)
	}); ok {
		m, err := memAuth.IsMember(r.Context(), u, wsID)
		if err == nil && m {
			return true
		}
	}
	return h.Authorizer.HasPermission(r.Context(), u, wsID, "payroll:read")
}

func (h PayrollHandler) allowedManage(r *http.Request, wsID uuid.UUID) bool {
	u, ok := httpinfra.UserIDFromContext(r.Context())
	if !ok || h.Authorizer == nil {
		return false
	}
	return h.Authorizer.HasPermission(r.Context(), u, wsID, "payroll:components:manage")
}
func parsePayrollPeriod(r *http.Request) (time.Time, time.Time, error) {
	year, err := strconv.Atoi(r.URL.Query().Get("year"))
	if err != nil {
		return time.Time{}, time.Time{}, ErrPayrollValidation
	}
	month, err := strconv.Atoi(r.URL.Query().Get("month"))
	if err != nil || year < 2000 || year > 2100 || month < 1 || month > 12 {
		return time.Time{}, time.Time{}, ErrPayrollValidation
	}
	start := time.Date(year, time.Month(month), 1, 0, 0, 0, 0, time.UTC)
	return start, start.AddDate(0, 1, 0), nil
}
func (h PayrollHandler) allowed(r *http.Request, wsID uuid.UUID, p string) bool {
	u, ok := httpinfra.UserIDFromContext(r.Context())
	return ok && h.Authorizer != nil && h.Authorizer.HasPermission(r.Context(), u, wsID, p)
}
func (h PayrollHandler) forbidden(w http.ResponseWriter, r *http.Request) {
	apperror.WriteError(w, r, apperror.Forbidden(apperror.KeyForbidden))
}
func (h PayrollHandler) writeError(w http.ResponseWriter, r *http.Request, err error) {
	if errors.Is(err, pgx.ErrNoRows) || errors.Is(err, ErrComponentNotFound) || errors.Is(err, ErrEmploymentNotFound) || errors.Is(err, ErrPayrollSettingsNotFound) || errors.Is(err, ErrIncentivePolicyNotFound) || errors.Is(err, ErrRuleNotFound) || errors.Is(err, ErrTierNotFound) || errors.Is(err, ErrAssignmentNotFound) {
		apperror.WriteError(w, r, apperror.NotFound(apperror.KeyNotFound))
		return
	}
	if errors.Is(err, ErrEmployeeNotFound) {
		apperror.WriteError(w, r, apperror.NotFound(apperror.KeyEmployeeNotFound))
		return
	}
	if errors.Is(err, ErrNoActiveEmployment) {
		apperror.WriteError(w, r, apperror.Validation("error.no_active_employment"))
		return
	}
	if errors.Is(err, ErrPayrollValidation) {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	if errors.Is(err, ErrInvalidBaseGroup) {
		apperror.WriteError(w, r, apperror.Forbidden(apperror.KeyInvalidBaseGroup))
		return
	}
	if errors.Is(err, ErrComponentConflict) || errors.Is(err, ErrConflict) || errors.Is(err, ErrAssignmentConflict) {
		apperror.WriteError(w, r, apperror.Conflict(apperror.KeyConflict))
		return
	}
	apperror.WriteError(w, r, apperror.Internal(err))
}
