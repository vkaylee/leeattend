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
	"github.com/vkaylee/leeattend/apiserver/internal/audit"
	"github.com/vkaylee/leeattend/apiserver/internal/domain/ids"
	payrolldomain "github.com/vkaylee/leeattend/apiserver/internal/domain/payroll"
	"github.com/vkaylee/leeattend/apiserver/internal/domain/payroll/formula"
	httpinfra "github.com/vkaylee/leeattend/apiserver/internal/infra/http"
)

var (
	ErrPayrollValidation       = errors.New("payroll: validation failed")
	ErrComponentNotFound       = errors.New("payroll: component not found")
	ErrComponentConflict       = errors.New("payroll: component already exists")
	ErrPayrollSettingsNotFound = errors.New("payroll: settings not found")
	ErrIncentivePolicyNotFound = errors.New("payroll: incentive policy not found")
	ErrRuleNotFound            = errors.New("payroll: rule not found")
	ErrTierNotFound            = errors.New("payroll: tier not found")
	ErrAssignmentNotFound      = errors.New("payroll: assignment not found")
	ErrAssignmentConflict      = errors.New("payroll: assignment conflict")
	ErrPayrollPeriodNotFound   = errors.New("payroll: period not found")
	ErrPayrollPeriodConflict   = errors.New("payroll: period conflict")
	ErrPayrollPeriodValidation = errors.New("payroll: period validation failed")
	ErrPayslipConflict         = errors.New("payroll: payslip conflict")
	ErrPayrollInputConflict    = errors.New("payroll: input batch conflict")
	ErrPayrollInputNotFound    = errors.New("payroll: input batch not found")
	ErrPayrollImpactNotFound   = errors.New("payroll: impact run not found")
	ErrPayrollImpactValidation = errors.New("payroll: impact runs are invalid")
)

type CreateSalaryComponentRequest struct {
	Name string `json:"name"`
	Code string `json:"code"`
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

type PayrollQuery struct {
	Year  int `json:"year"`
	Month int `json:"month"`
}
type PayslipRecord struct {
	ID                       uuid.UUID       `json:"id"`
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
	RuleDefinitionID uuid.UUID        `json:"rule_definition_id"`
	AssigneeType     string           `json:"assignee_type"`
	AssigneeID       *uuid.UUID       `json:"assignee_id"`
	EffectiveFrom    string           `json:"effective_from"`
	EffectiveTo      *string          `json:"effective_to"`
	OverrideConfig   *json.RawMessage `json:"override_config"`
	CreatedAt        string           `json:"created_at"`
}

type PayslipSegmentResponse struct {
	ID                   uuid.UUID       `json:"id"`
	PayslipID            uuid.UUID       `json:"payslip_id"`
	StartDate            string          `json:"start_date"`
	EndDate              string          `json:"end_date"`
	DepartmentID         *uuid.UUID      `json:"department_id"`
	AppliedBaseSalary    decimal.Decimal `json:"applied_base_salary"`
	AppliedStdDays       decimal.Decimal `json:"applied_std_days"`
	AppliedHoursPerDay   decimal.Decimal `json:"applied_hours_per_day"`
	ActualWorkDays       decimal.Decimal `json:"actual_work_days"`
	PaidWorkDays         decimal.Decimal `json:"paid_work_days"`
	Amount               decimal.Decimal `json:"amount"`
	Reason               string          `json:"reason"`
	StdDaysSource        *string         `json:"std_days_source,omitempty"`
	LaborPolicyVersionID *uuid.UUID      `json:"labor_policy_version_id,omitempty"`
}

type IncentiveEvaluationResult struct {
	PolicyName     string          `json:"policy_name"`
	Amount         decimal.Decimal `json:"amount"`
	StandardAmount decimal.Decimal `json:"standard_amount"`
	TierID         *uuid.UUID      `json:"tier_id"`
}

type PayrollCalculationTrace struct {
	Step       string `json:"step"`
	Rule       string `json:"rule"`
	Expression string `json:"expression"`
	Result     string `json:"result"`
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
	CalculationTrace []PayrollCalculationTrace       `json:"calculation_trace,omitempty"`
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

type CreatePayrollInputRequest struct {
	PeriodYear     int            `json:"period_year"`
	PeriodMonth    int            `json:"period_month"`
	Source         string         `json:"source"`
	IdempotencyKey string         `json:"idempotency_key"`
	Inputs         []PayrollInput `json:"inputs"`
}

type PayrollInput struct {
	ID                uuid.UUID       `json:"id,omitempty"`
	EmploymentID      uuid.UUID       `json:"employment_id"`
	MetricCode        string          `json:"metric_code"`
	Value             decimal.Decimal `json:"value"`
	Unit              string          `json:"unit"`
	Status            string          `json:"status,omitempty"`
	ValidationMessage *string         `json:"validation_message,omitempty"`
	SourceReference   *string         `json:"source_reference,omitempty"`
}

type CorrectPayrollInputRequest struct {
	Value  decimal.Decimal `json:"value"`
	Unit   string          `json:"unit"`
	Reason string          `json:"reason"`
}

type PayrollInputBatch struct {
	ID             uuid.UUID  `json:"id"`
	PeriodYear     int        `json:"period_year"`
	PeriodMonth    int        `json:"period_month"`
	Source         string     `json:"source"`
	IdempotencyKey string     `json:"idempotency_key"`
	Status         string     `json:"status"`
	InputCount     int        `json:"input_count"`
	SubmittedBy    uuid.UUID  `json:"submitted_by"`
	ApprovedBy     *uuid.UUID `json:"approved_by,omitempty"`
	LockedBy       *uuid.UUID `json:"locked_by,omitempty"`
	CreatedAt      string     `json:"created_at"`
}

type UpdatePayrollInputBatchRequest struct {
	Version int64 `json:"version"`
}

type PreviewPayrollFormulaRequest struct {
	Expression string                     `json:"expression"`
	Values     map[string]decimal.Decimal `json:"values"`
}

type PreviewPayrollFormulaResponse struct {
	Value string `json:"value"`
}

type CreatePayrollFormulaRuleRequest struct {
	Code          string `json:"code"`
	Name          string `json:"name"`
	ComponentCode string `json:"component_code"`
	Expression    string `json:"expression"`
	EffectiveFrom string `json:"effective_from"`
}

type CreatePayrollFormulaRuleVersionRequest struct {
	Expression    string `json:"expression"`
	EffectiveFrom string `json:"effective_from"`
}

type PayrollFormulaRule struct {
	ID            uuid.UUID `json:"id"`
	Code          string    `json:"code"`
	Name          string    `json:"name"`
	ComponentCode string    `json:"component_code"`
	VersionID     uuid.UUID `json:"version_id"`
	Version       int       `json:"version"`
	Expression    string    `json:"expression"`
	EffectiveFrom string    `json:"effective_from"`
	Status        string    `json:"status"`
}

type PayrollFormulaRuleVersion struct {
	VersionID     uuid.UUID  `json:"version_id"`
	RuleID        uuid.UUID  `json:"rule_id"`
	Version       int        `json:"version"`
	Expression    string     `json:"expression"`
	EffectiveFrom string     `json:"effective_from"`
	Status        string     `json:"status"`
	CreatedBy     uuid.UUID  `json:"created_by"`
	PublishedBy   *uuid.UUID `json:"published_by,omitempty"`
	PublishedAt   *string    `json:"published_at,omitempty"`
	CreatedAt     string     `json:"created_at"`
}

type PayrollImpactBatchRequest struct {
	EmploymentIDs []uuid.UUID `json:"employment_ids"`
	Year          int         `json:"year"`
	Month         int         `json:"month"`
	BeforeRunID   uuid.UUID   `json:"before_run_id"`
	AfterRunID    uuid.UUID   `json:"after_run_id"`
}

type PayrollImpactBatchItem struct {
	EmploymentID uuid.UUID              `json:"employment_id"`
	Computed     bool                   `json:"computed"`
	Stale        bool                   `json:"stale"`
	StaleReasons []string               `json:"stale_reasons,omitempty"`
	Reason       string                 `json:"reason,omitempty"`
	Delta        *PayrollImpactResponse `json:"delta,omitempty"`
}

type PayrollImpactBatchResponse struct {
	Total        int                      `json:"total"`
	Computed     int                      `json:"computed"`
	Affected     int                      `json:"affected"`
	Uncomputable int                      `json:"uncomputable"`
	Items        []PayrollImpactBatchItem `json:"items"`
}

type PayrollImpactResponse struct {
	EmploymentID       uuid.UUID       `json:"employment_id"`
	PeriodYear         int             `json:"period_year"`
	PeriodMonth        int             `json:"period_month"`
	BeforeRunID        uuid.UUID       `json:"before_run_id"`
	AfterRunID         uuid.UUID       `json:"after_run_id"`
	Computed           bool            `json:"computed"`
	Stale              bool            `json:"stale"`
	StaleReasons       []string        `json:"stale_reasons,omitempty"`
	Reason             string          `json:"reason,omitempty"`
	BasePayDelta       decimal.Decimal `json:"base_pay_delta"`
	OvertimePayDelta   decimal.Decimal `json:"overtime_pay_delta"`
	DeductionsDelta    decimal.Decimal `json:"deductions_delta"`
	NetPayDelta        decimal.Decimal `json:"net_pay_delta"`
	StandardUnitsDelta int32           `json:"standard_units_delta"`
	OvertimeUnitsDelta int32           `json:"overtime_units_delta"`
}

type PayrollCalculationRun struct {
	ID               uuid.UUID       `json:"id"`
	EmploymentID     uuid.UUID       `json:"employment_id"`
	PayslipID        *uuid.UUID      `json:"payslip_id,omitempty"`
	PeriodYear       int             `json:"period_year"`
	PeriodMonth      int             `json:"period_month"`
	FormulaVersionID *uuid.UUID      `json:"formula_version_id,omitempty"`
	EngineVersion    string          `json:"engine_version"`
	InputSnapshot    json.RawMessage `json:"input_snapshot"`
	OutputSnapshot   json.RawMessage `json:"output_snapshot"`
	CalculationTrace json.RawMessage `json:"calculation_trace"`
	CreatedBy        *uuid.UUID      `json:"created_by,omitempty"`
	CreatedAt        string          `json:"created_at"`
}

type PayrollPeriod struct {
	ID           uuid.UUID  `json:"id"`
	EmployerID   uuid.UUID  `json:"employer_id"`
	Year         int        `json:"year"`
	Month        int        `json:"month"`
	Status       string     `json:"status"`
	Version      int64      `json:"version"`
	LockedAt     *string    `json:"locked_at,omitempty"`
	LockedBy     *uuid.UUID `json:"locked_by,omitempty"`
	ReopenedAt   *string    `json:"reopened_at,omitempty"`
	ReopenedBy   *uuid.UUID `json:"reopened_by,omitempty"`
	ReopenReason *string    `json:"reopen_reason,omitempty"`
}

type LockPayrollPeriodRequest struct {
	Version int64 `json:"version"`
}

type ReopenPayrollPeriodRequest struct {
	Version int64  `json:"version"`
	Reason  string `json:"reason"`
}

type PayrollRepository interface {
	ListPayslips(context.Context, time.Time, time.Time) ([]PayslipRecord, error)
	ListSalaryManagementRows(context.Context, SalaryManagementQuery) (SalaryManagementResponse, error)
	ListEmployeePayslips(context.Context, ids.EmployeeID) ([]PayslipRecord, error)
	ListSalaryComponents(context.Context) ([]SalaryComponentDefinition, error)
	CreateSalaryComponent(context.Context, CreateSalaryComponentRequest) (SalaryComponentDefinition, error)
	ListEmploymentSalaryComponents(context.Context, ids.EmploymentID) ([]EmploymentSalaryComponentResponse, error)
	AssignEmploymentSalaryComponent(context.Context, ids.EmploymentID, AssignSalaryComponentRequest) (EmployeeSalaryComponent, error)
	GetPayrollSettings(context.Context) (*PayrollSettings, error)
	GetPayrollSettingsHistory(context.Context) ([]PayrollSettings, error)
	UpdatePayrollSettings(context.Context, UpdatePayrollSettingsRequest) (PayrollSettings, error)
	GetEmployeePayrollConfigs(context.Context, ids.EmployeeID) ([]EmployeePayrollConfig, error)
	SetEmployeePayrollConfig(context.Context, ids.EmployeeID, SetEmployeePayrollConfigRequest, ids.UserID) (EmployeePayrollConfig, error)
	GetIncentivePolicies(context.Context) ([]PayrollIncentivePolicyResponse, error)
	CreateIncentivePolicy(context.Context, CreatePayrollIncentivePolicyRequest) (PayrollIncentivePolicy, error)
	DeleteIncentivePolicy(context.Context, ids.PayrollIncentivePolicyID) error
	CloneIncentivePolicy(context.Context, ids.PayrollIncentivePolicyID) (PayrollIncentivePolicy, error)
	ListPayrollRules(context.Context) ([]PayrollRuleDefinition, error)
	CreatePayrollRule(context.Context, CreatePayrollRuleDefinitionRequest) (PayrollRuleDefinition, error)
	ListPayrollRuleTiers(context.Context, ids.PayrollRuleID) ([]PayrollRuleTier, error)
	CreatePayrollRuleTier(context.Context, ids.PayrollRuleID, CreatePayrollRuleTierRequest) (PayrollRuleTier, error)
	DeletePayrollRuleTier(context.Context, ids.PayrollRuleID, ids.PayrollRuleTierID) error
	ListPayrollRuleAssignments(context.Context, ids.PayrollRuleID) ([]PayrollRuleAssignment, error)
	AssignPayrollRule(context.Context, ids.PayrollRuleID, AssignPayrollRuleRequest) (PayrollRuleAssignment, error)
	DeletePayrollRuleAssignment(context.Context, ids.PayrollRuleID, ids.PayrollRuleAssignmentID) error
	CalculateEmployeePayroll(context.Context, ids.EmployeeID, int, int, ids.UserID) (FullPayslipResponse, error)
	SimulateEmployeePayroll(context.Context, ids.EmployeeID, int, int, ids.UserID) (FullPayslipResponse, error)
	SimulateEmployeePayrollWithPolicy(context.Context, ids.EmployeeID, int, int, uuid.UUID, ids.UserID) (FullPayslipResponse, error)
	CalculateBatchPayroll(context.Context, BatchCalculatePayrollRequest, ids.UserID) (BatchCalculatePayrollResponse, error)
	CreatePayrollInputBatch(context.Context, CreatePayrollInputRequest, ids.UserID) (PayrollInputBatch, error)
	ApprovePayrollInputBatch(context.Context, ids.PayrollInputBatchID, ids.UserID) (PayrollInputBatch, error)
	LockPayrollInputBatch(context.Context, ids.PayrollInputBatchID, ids.UserID) (PayrollInputBatch, error)
	ListPayrollInputs(context.Context, ids.PayrollInputBatchID) ([]PayrollInput, error)
	CorrectPayrollInput(context.Context, ids.PayrollInputID, CorrectPayrollInputRequest, ids.UserID) (PayrollInput, error)
	ListPayrollFormulaRules(context.Context) ([]PayrollFormulaRule, error)
	ListPayrollFormulaRuleVersions(context.Context, uuid.UUID) ([]PayrollFormulaRuleVersion, error)
	CreatePayrollFormulaRule(context.Context, CreatePayrollFormulaRuleRequest, ids.UserID) (PayrollFormulaRule, error)
	CreatePayrollFormulaRuleVersion(context.Context, ids.PayrollFormulaRuleID, CreatePayrollFormulaRuleVersionRequest, ids.UserID) (PayrollFormulaRuleVersion, error)
	PublishPayrollFormulaRule(context.Context, ids.PayrollFormulaRuleID, ids.UserID) (PayrollFormulaRule, error)
	RetirePayrollFormulaRule(context.Context, ids.PayrollFormulaRuleID, ids.UserID) (PayrollFormulaRule, error)
	ListPayrollCalculationRuns(context.Context, ids.EmploymentID, int, int) ([]PayrollCalculationRun, error)
	ComparePayrollImpact(context.Context, ids.EmploymentID, int, int, uuid.UUID, uuid.UUID) (PayrollImpactResponse, error)
	ComparePayrollImpactBatch(context.Context, PayrollImpactBatchRequest) (PayrollImpactBatchResponse, error)
	GetPayrollPeriod(context.Context, ids.EmployerID, int, int) (PayrollPeriod, error)
	LockPayrollPeriod(context.Context, payrolldomain.PayrollPeriodMutation) (PayrollPeriod, error)
	ReopenPayrollPeriod(context.Context, payrolldomain.PayrollPeriodMutation) (PayrollPeriod, error)
}
type PayrollHandler struct {
	Repository PayrollRepository
	Authorizer Authorizer
	Audit      PayrollAudit
}

func (h *PayrollHandler) ListPayslips(w http.ResponseWriter, r *http.Request) {

	if !h.allowed(r, "payroll:read") {
		h.forbidden(w, r)
		return
	}
	start, end, err := parsePayrollPeriod(r)
	if err != nil {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	rows, err := h.Repository.ListPayslips(r.Context(), start, end)
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusOK, rows)
}
func (h *PayrollHandler) ListEmployee(w http.ResponseWriter, r *http.Request) {

	empID, empOK := pathUUID(r, "employee_id")
	if !empOK || !h.allowed(r, "payroll:read") {
		h.forbidden(w, r)
		return
	}
	rows, err := h.Repository.ListEmployeePayslips(r.Context(), ids.EmployeeID(empID))
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusOK, rows)
}
func (h *PayrollHandler) ListSalaryComponents(w http.ResponseWriter, r *http.Request) {

	if !h.allowedRead(r) {
		h.forbidden(w, r)
		return
	}
	rows, err := h.Repository.ListSalaryComponents(r.Context())
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	if rows == nil {
		rows = []SalaryComponentDefinition{}
	}
	apperror.WriteJSON(w, http.StatusOK, rows)
}

func (h *PayrollHandler) CreateSalaryComponent(w http.ResponseWriter, r *http.Request) {

	if !h.allowedManage(r) {
		h.forbidden(w, r)
		return
	}
	req, err := DecodeJSON[CreateSalaryComponentRequest](r, 1<<20)
	if err != nil || strings.TrimSpace(req.Name) == "" || strings.TrimSpace(req.Code) == "" {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	comp, err := h.Repository.CreateSalaryComponent(r.Context(), req)
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusCreated, comp)
}

func (h *PayrollHandler) ListEmploymentSalaryComponents(w http.ResponseWriter, r *http.Request) {

	empID, empOK := employmentIDFromRequest(r)
	if !empOK || !h.allowedRead(r) {
		h.forbidden(w, r)
		return
	}
	comps, err := h.Repository.ListEmploymentSalaryComponents(r.Context(), ids.EmploymentID(empID))
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	if comps == nil {
		comps = []EmploymentSalaryComponentResponse{}
	}
	apperror.WriteJSON(w, http.StatusOK, comps)
}

func (h *PayrollHandler) AssignEmploymentSalaryComponent(w http.ResponseWriter, r *http.Request) {

	empID, empOK := employmentIDFromRequest(r)
	if !empOK || !h.allowedManage(r) {
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
	comp, err := h.Repository.AssignEmploymentSalaryComponent(r.Context(), ids.EmploymentID(empID), req)
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusCreated, comp)
}

func (h *PayrollHandler) GetPayrollSettings(w http.ResponseWriter, r *http.Request) {

	if !h.allowedMember(r) {
		h.forbidden(w, r)
		return
	}
	settings, err := h.Repository.GetPayrollSettings(r.Context())
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusOK, settings)
}

func (h *PayrollHandler) UpdatePayrollSettings(w http.ResponseWriter, r *http.Request) {

	if !h.allowedManage(r) {
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
	settings, err := h.Repository.UpdatePayrollSettings(r.Context(), req)
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusOK, settings)
}

func (h *PayrollHandler) GetPayrollSettingsHistory(w http.ResponseWriter, r *http.Request) {

	if !h.allowedMember(r) {
		h.forbidden(w, r)
		return
	}
	history, err := h.Repository.GetPayrollSettingsHistory(r.Context())
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	if history == nil {
		history = []PayrollSettings{}
	}
	apperror.WriteJSON(w, http.StatusOK, history)
}

func (h *PayrollHandler) GetEmployeePayrollConfig(w http.ResponseWriter, r *http.Request) {

	empID, empOK := pathUUID(r, "id")
	if !empOK || !h.allowedMember(r) {
		h.forbidden(w, r)
		return
	}
	configs, err := h.Repository.GetEmployeePayrollConfigs(r.Context(), ids.EmployeeID(empID))
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	if configs == nil {
		configs = []EmployeePayrollConfig{}
	}
	apperror.WriteJSON(w, http.StatusOK, configs)
}

func (h *PayrollHandler) SetEmployeePayrollConfig(w http.ResponseWriter, r *http.Request) {

	empID, empOK := pathUUID(r, "id")
	if !empOK || !h.allowedManage(r) {
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
	config, err := h.Repository.SetEmployeePayrollConfig(r.Context(), ids.EmployeeID(empID), req, ids.UserID(userID))
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusCreated, config)
}

func (h *PayrollHandler) GetIncentivePolicies(w http.ResponseWriter, r *http.Request) {

	if !h.allowedMember(r) {
		h.forbidden(w, r)
		return
	}
	policies, err := h.Repository.GetIncentivePolicies(r.Context())
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	if policies == nil {
		policies = []PayrollIncentivePolicyResponse{}
	}
	apperror.WriteJSON(w, http.StatusOK, policies)
}

func (h *PayrollHandler) CreateIncentivePolicy(w http.ResponseWriter, r *http.Request) {

	if !h.allowedMember(r) {
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
	policy, err := h.Repository.CreateIncentivePolicy(r.Context(), req)
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusCreated, policy)
}

func (h *PayrollHandler) DeleteIncentivePolicy(w http.ResponseWriter, r *http.Request) {

	policyID, pOK := pathUUID(r, "policy_id")
	if !pOK || !h.allowedMember(r) {
		h.forbidden(w, r)
		return
	}
	if err := h.Repository.DeleteIncentivePolicy(r.Context(), ids.PayrollIncentivePolicyID(policyID)); err != nil {
		h.writeError(w, r, err)
		return
	}
	w.WriteHeader(http.StatusNoContent)
}

func (h *PayrollHandler) CloneIncentivePolicy(w http.ResponseWriter, r *http.Request) {

	policyID, pOK := pathUUID(r, "policy_id")
	if !pOK || !h.allowedMember(r) {
		h.forbidden(w, r)
		return
	}
	policy, err := h.Repository.CloneIncentivePolicy(r.Context(), ids.PayrollIncentivePolicyID(policyID))
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusCreated, policy)
}

func (h *PayrollHandler) ListPayrollRules(w http.ResponseWriter, r *http.Request) {

	if !h.allowedMember(r) {
		h.forbidden(w, r)
		return
	}
	rules, err := h.Repository.ListPayrollRules(r.Context())
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	if rules == nil {
		rules = []PayrollRuleDefinition{}
	}
	apperror.WriteJSON(w, http.StatusOK, rules)
}

func (h *PayrollHandler) CreatePayrollRule(w http.ResponseWriter, r *http.Request) {

	if !h.allowedMember(r) {
		h.forbidden(w, r)
		return
	}
	req, err := DecodeJSON[CreatePayrollRuleDefinitionRequest](r, 1<<20)
	if err != nil || strings.TrimSpace(req.Name) == "" || strings.TrimSpace(req.Code) == "" {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	rule, err := h.Repository.CreatePayrollRule(r.Context(), req)
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusCreated, rule)
}

func (h *PayrollHandler) ListPayrollRuleTiers(w http.ResponseWriter, r *http.Request) {

	ruleID, rOK := pathUUID(r, "rule_id")
	if !rOK || !h.allowedMember(r) {
		h.forbidden(w, r)
		return
	}
	tiers, err := h.Repository.ListPayrollRuleTiers(r.Context(), ids.PayrollRuleID(ruleID))
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	if tiers == nil {
		tiers = []PayrollRuleTier{}
	}
	apperror.WriteJSON(w, http.StatusOK, tiers)
}

func (h *PayrollHandler) CreatePayrollRuleTier(w http.ResponseWriter, r *http.Request) {

	ruleID, rOK := pathUUID(r, "rule_id")
	if !rOK || !h.allowedMember(r) {
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
	tier, err := h.Repository.CreatePayrollRuleTier(r.Context(), ids.PayrollRuleID(ruleID), req)
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusCreated, tier)
}

func (h *PayrollHandler) DeletePayrollRuleTier(w http.ResponseWriter, r *http.Request) {

	ruleID, rOK := pathUUID(r, "rule_id")
	tierID, tOK := pathUUID(r, "tier_id")
	if !rOK || !tOK || !h.allowedMember(r) {
		h.forbidden(w, r)
		return
	}
	if err := h.Repository.DeletePayrollRuleTier(r.Context(), ids.PayrollRuleID(ruleID), ids.PayrollRuleTierID(tierID)); err != nil {
		h.writeError(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusOK, map[string]bool{"success": true})
}

func (h *PayrollHandler) ListPayrollRuleAssignments(w http.ResponseWriter, r *http.Request) {

	ruleID, rOK := pathUUID(r, "rule_id")
	if !rOK || !h.allowedMember(r) {
		h.forbidden(w, r)
		return
	}
	assignments, err := h.Repository.ListPayrollRuleAssignments(r.Context(), ids.PayrollRuleID(ruleID))
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	if assignments == nil {
		assignments = []PayrollRuleAssignment{}
	}
	apperror.WriteJSON(w, http.StatusOK, assignments)
}

func (h *PayrollHandler) AssignPayrollRule(w http.ResponseWriter, r *http.Request) {

	ruleID, rOK := pathUUID(r, "rule_id")
	if !rOK || !h.allowedMember(r) {
		h.forbidden(w, r)
		return
	}
	req, err := DecodeJSON[AssignPayrollRuleRequest](r, 1<<20)
	if err != nil || (req.AssigneeType != "organization" && req.AssigneeType != "department" && req.AssigneeType != "employee") {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	if req.AssigneeType != "organization" && req.AssigneeID == nil {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	if req.AssigneeType == "organization" {
		req.AssigneeID = nil
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
	assignment, err := h.Repository.AssignPayrollRule(r.Context(), ids.PayrollRuleID(ruleID), req)
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusCreated, assignment)
}

func (h *PayrollHandler) DeletePayrollRuleAssignment(w http.ResponseWriter, r *http.Request) {

	ruleID, rOK := pathUUID(r, "rule_id")
	assignmentID, aOK := pathUUID(r, "assignment_id")
	if !rOK || !aOK || !h.allowedMember(r) {
		h.forbidden(w, r)
		return
	}
	if err := h.Repository.DeletePayrollRuleAssignment(r.Context(), ids.PayrollRuleID(ruleID), ids.PayrollRuleAssignmentID(assignmentID)); err != nil {
		h.writeError(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusOK, map[string]bool{"success": true})
}

func (h *PayrollHandler) GetPayrollPeriod(w http.ResponseWriter, r *http.Request) {
	if !h.allowed(r, "payroll:read") {
		h.forbidden(w, r)
		return
	}
	employerID, ok := payrollPeriodEmployer(r)
	if !ok {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	year, month, err := payrollPeriodPath(r)
	if err != nil {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	period, err := h.Repository.GetPayrollPeriod(r.Context(), ids.EmployerID(employerID), year, month)
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusOK, period)
}

func (h *PayrollHandler) LockPayrollPeriod(w http.ResponseWriter, r *http.Request) {
	if !h.allowed(r, "payroll:periods:lock") {
		h.forbidden(w, r)
		return
	}
	employerID, ok := payrollPeriodEmployer(r)
	if !ok {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	year, month, err := payrollPeriodPath(r)
	if err != nil {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	request, err := DecodeJSON[LockPayrollPeriodRequest](r, 1<<20)
	if err != nil || request.Version < 1 {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	actorID, _ := httpinfra.UserIDFromContext(r.Context())
	period, err := h.Repository.LockPayrollPeriod(r.Context(), payrolldomain.PayrollPeriodMutation{
		EmployerID: ids.EmployerID(employerID), Year: year, Month: month, ExpectedVersion: request.Version, ActorID: ids.UserID(actorID),
	})
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusOK, period)
}

func (h *PayrollHandler) ReopenPayrollPeriod(w http.ResponseWriter, r *http.Request) {
	if !h.allowed(r, "payroll:periods:reopen") {
		h.forbidden(w, r)
		return
	}
	employerID, ok := payrollPeriodEmployer(r)
	if !ok {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	year, month, err := payrollPeriodPath(r)
	if err != nil {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	request, err := DecodeJSON[ReopenPayrollPeriodRequest](r, 1<<20)
	if err != nil || request.Version < 1 || strings.TrimSpace(request.Reason) == "" || len(request.Reason) > 500 {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	actorID, _ := httpinfra.UserIDFromContext(r.Context())
	period, err := h.Repository.ReopenPayrollPeriod(r.Context(), payrolldomain.PayrollPeriodMutation{
		EmployerID: ids.EmployerID(employerID), Year: year, Month: month, ExpectedVersion: request.Version,
		Reason: strings.TrimSpace(request.Reason), ActorID: ids.UserID(actorID),
	})
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusOK, period)
}

func (h *PayrollHandler) CalculatePayroll(w http.ResponseWriter, r *http.Request) {

	empID, eOK := pathUUID(r, "id")
	if !eOK || !h.allowedCalculate(r) {
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
	actorID, actorOK := httpinfra.UserIDFromContext(r.Context())
	if !actorOK {
		h.forbidden(w, r)
		return
	}
	resp, err := h.Repository.CalculateEmployeePayroll(r.Context(), ids.EmployeeID(empID), year, month, ids.UserID(actorID))
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusOK, resp)
}

func (h *PayrollHandler) CreatePayrollInputBatch(w http.ResponseWriter, r *http.Request) {
	if !h.allowed(r, "payroll:inputs:manage") && !h.allowed(r, "payroll:manage") {
		h.forbidden(w, r)
		return
	}
	userID, ok := httpinfra.UserIDFromContext(r.Context())
	if !ok {
		h.forbidden(w, r)
		return
	}
	req, err := DecodeJSON[CreatePayrollInputRequest](r, 4<<20)
	if err != nil || !validPayrollInputRequest(req) {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	batch, err := h.Repository.CreatePayrollInputBatch(r.Context(), req, ids.UserID(userID))
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	h.emitPayrollAudit(r, audit.Event{
		Name:     "payroll.input.submitted",
		Resource: batch.ID.String(),
		Outcome:  "success",
		Details: map[string]string{
			"batch_id":     batch.ID.String(),
			"period_year":  strconv.Itoa(batch.PeriodYear),
			"period_month": strconv.Itoa(batch.PeriodMonth),
			"input_count":  strconv.Itoa(batch.InputCount),
		},
	})
	apperror.WriteJSON(w, http.StatusCreated, batch)
}

func validPayrollInputRequest(req CreatePayrollInputRequest) bool {
	if req.PeriodYear < 2000 || req.PeriodYear > 2100 || req.PeriodMonth < 1 || req.PeriodMonth > 12 {
		return false
	}
	if strings.TrimSpace(req.Source) == "" || len(req.Source) > 100 || strings.TrimSpace(req.IdempotencyKey) == "" || len(req.IdempotencyKey) > 200 || len(req.Inputs) == 0 || len(req.Inputs) > 10000 {
		return false
	}
	seen := make(map[string]struct{}, len(req.Inputs))
	for _, input := range req.Inputs {
		if input.EmploymentID == uuid.Nil || strings.TrimSpace(input.MetricCode) == "" || len(input.MetricCode) > 100 || strings.TrimSpace(input.Unit) == "" || len(input.Unit) > 40 || input.Value.IsNegative() {
			return false
		}
		key := input.EmploymentID.String() + "|" + input.MetricCode
		if _, exists := seen[key]; exists {
			return false
		}
		seen[key] = struct{}{}
	}
	return true
}

func (h *PayrollHandler) ComparePayrollImpact(w http.ResponseWriter, r *http.Request) {
	if !h.allowed(r, "payroll:sensitive:read") {
		h.forbidden(w, r)
		return
	}
	employmentID, ok := pathUUID(r, "id")
	if !ok {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	year, err := strconv.Atoi(r.URL.Query().Get("year"))
	month, monthErr := strconv.Atoi(r.URL.Query().Get("month"))
	beforeRunID, beforeErr := uuid.Parse(r.URL.Query().Get("before_run_id"))
	afterRunID, afterErr := uuid.Parse(r.URL.Query().Get("after_run_id"))
	if err != nil || monthErr != nil || beforeErr != nil || afterErr != nil || year < 2000 || year > 2100 || month < 1 || month > 12 || beforeRunID == uuid.Nil || afterRunID == uuid.Nil || beforeRunID == afterRunID {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	result, err := h.Repository.ComparePayrollImpact(r.Context(), ids.EmploymentID(employmentID), year, month, beforeRunID, afterRunID)
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusOK, result)
}

func (h *PayrollHandler) ComparePayrollImpactBatch(w http.ResponseWriter, r *http.Request) {
	if !h.allowed(r, "payroll:sensitive:read") {
		h.forbidden(w, r)
		return
	}
	req, err := DecodeJSON[PayrollImpactBatchRequest](r, 1<<20)
	if err != nil || len(req.EmploymentIDs) == 0 || len(req.EmploymentIDs) > 100 || hasDuplicateUUIDs(req.EmploymentIDs) || req.Year < 2000 || req.Year > 2100 || req.Month < 1 || req.Month > 12 || req.BeforeRunID == uuid.Nil || req.AfterRunID == uuid.Nil || req.BeforeRunID == req.AfterRunID {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	result, err := h.Repository.ComparePayrollImpactBatch(r.Context(), req)
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusOK, result)
}

func hasDuplicateUUIDs(values []uuid.UUID) bool {
	seen := make(map[uuid.UUID]struct{}, len(values))
	for _, value := range values {
		if _, exists := seen[value]; exists {
			return true
		}
		seen[value] = struct{}{}
	}
	return false
}

func (h *PayrollHandler) ListPayrollCalculationRuns(w http.ResponseWriter, r *http.Request) {
	if !h.allowed(r, "payroll:read") {
		h.forbidden(w, r)
		return
	}
	employmentID, ok := pathUUID(r, "id")
	if !ok {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	year, err := strconv.Atoi(r.URL.Query().Get("year"))
	month, monthErr := strconv.Atoi(r.URL.Query().Get("month"))
	if err != nil || monthErr != nil || year < 2000 || year > 2100 || month < 1 || month > 12 {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	runs, err := h.Repository.ListPayrollCalculationRuns(r.Context(), ids.EmploymentID(employmentID), year, month)
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	if runs == nil {
		runs = []PayrollCalculationRun{}
	}
	apperror.WriteJSON(w, http.StatusOK, runs)
}

func (h *PayrollHandler) ListPayrollFormulaRuleVersions(w http.ResponseWriter, r *http.Request) {
	if !h.allowed(r, "payroll:read") {
		h.forbidden(w, r)
		return
	}
	ruleID, ok := pathUUID(r, "rule_id")
	if !ok {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	versions, err := h.Repository.ListPayrollFormulaRuleVersions(r.Context(), ruleID)
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	if versions == nil {
		versions = []PayrollFormulaRuleVersion{}
	}
	apperror.WriteJSON(w, http.StatusOK, versions)
}

func (h *PayrollHandler) ListPayrollFormulaRules(w http.ResponseWriter, r *http.Request) {
	if !h.allowed(r, "payroll:read") {
		h.forbidden(w, r)
		return
	}
	rules, err := h.Repository.ListPayrollFormulaRules(r.Context())
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	if rules == nil {
		rules = []PayrollFormulaRule{}
	}
	apperror.WriteJSON(w, http.StatusOK, rules)
}

func (h *PayrollHandler) CreatePayrollFormulaRule(w http.ResponseWriter, r *http.Request) {
	if !h.allowed(r, "payroll:manage") {
		h.forbidden(w, r)
		return
	}
	actorID, ok := httpinfra.UserIDFromContext(r.Context())
	if !ok {
		h.forbidden(w, r)
		return
	}
	req, err := DecodeJSON[CreatePayrollFormulaRuleRequest](r, 1<<20)
	if err != nil || strings.TrimSpace(req.Code) == "" || strings.TrimSpace(req.Name) == "" || strings.TrimSpace(req.ComponentCode) == "" || len(req.Expression) == 0 || len(req.Expression) > 4096 {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	if _, err := time.Parse("2006-01-02", req.EffectiveFrom); err != nil {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	if err := formula.Validate(req.Expression); err != nil {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	rule, err := h.Repository.CreatePayrollFormulaRule(r.Context(), req, ids.UserID(actorID))
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	h.emitPayrollAudit(r, audit.Event{
		Name:     "payroll.formula.created",
		Resource: rule.ID.String(),
		Outcome:  "success",
		Details: map[string]string{
			"rule_id":        rule.ID.String(),
			"version_id":     rule.VersionID.String(),
			"version":        strconv.Itoa(rule.Version),
			"effective_from": rule.EffectiveFrom,
		},
	})
	apperror.WriteJSON(w, http.StatusCreated, rule)
}

func (h *PayrollHandler) CreatePayrollFormulaRuleVersion(w http.ResponseWriter, r *http.Request) {
	if !h.allowed(r, "payroll:manage") {
		h.forbidden(w, r)
		return
	}
	ruleID, ok := pathUUID(r, "rule_id")
	actorID, actorOK := httpinfra.UserIDFromContext(r.Context())
	if !ok || !actorOK {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	req, err := DecodeJSON[CreatePayrollFormulaRuleVersionRequest](r, 1<<20)
	if err != nil || len(req.Expression) == 0 || len(req.Expression) > 4096 {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	if _, err := time.Parse("2006-01-02", req.EffectiveFrom); err != nil {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	if err := formula.Validate(req.Expression); err != nil {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	version, err := h.Repository.CreatePayrollFormulaRuleVersion(r.Context(), ids.PayrollFormulaRuleID(ruleID), req, ids.UserID(actorID))
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	h.emitPayrollAudit(r, audit.Event{
		Name:     "payroll.formula.version_created",
		Resource: version.RuleID.String(),
		Outcome:  "success",
		Details: map[string]string{
			"rule_id":        version.RuleID.String(),
			"version_id":     version.VersionID.String(),
			"version":        strconv.Itoa(version.Version),
			"effective_from": version.EffectiveFrom,
		},
	})
	apperror.WriteJSON(w, http.StatusCreated, version)
}

func (h *PayrollHandler) RetirePayrollFormulaRule(w http.ResponseWriter, r *http.Request) {
	if !h.allowed(r, "payroll:manage") {
		h.forbidden(w, r)
		return
	}
	ruleID, ok := pathUUID(r, "rule_id")
	actorID, actorOK := httpinfra.UserIDFromContext(r.Context())
	if !ok || !actorOK {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	rule, err := h.Repository.RetirePayrollFormulaRule(r.Context(), ids.PayrollFormulaRuleID(ruleID), ids.UserID(actorID))
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	h.emitPayrollAudit(r, audit.Event{
		Name:     "payroll.formula.retired",
		Resource: rule.ID.String(),
		Outcome:  "success",
		Details: map[string]string{
			"rule_id":    rule.ID.String(),
			"version_id": rule.VersionID.String(),
			"version":    strconv.Itoa(rule.Version),
		},
	})
	apperror.WriteJSON(w, http.StatusOK, rule)
}

func (h *PayrollHandler) PublishPayrollFormulaRule(w http.ResponseWriter, r *http.Request) {
	if !h.allowed(r, "payroll:manage") {
		h.forbidden(w, r)
		return
	}
	ruleID, ok := pathUUID(r, "rule_id")
	actorID, actorOK := httpinfra.UserIDFromContext(r.Context())
	if !ok || !actorOK {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	rule, err := h.Repository.PublishPayrollFormulaRule(r.Context(), ids.PayrollFormulaRuleID(ruleID), ids.UserID(actorID))
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	h.emitPayrollAudit(r, audit.Event{
		Name:     "payroll.formula.published",
		Resource: rule.ID.String(),
		Outcome:  "success",
		Details: map[string]string{
			"rule_id":        rule.ID.String(),
			"version_id":     rule.VersionID.String(),
			"version":        strconv.Itoa(rule.Version),
			"effective_from": rule.EffectiveFrom,
		},
	})
	apperror.WriteJSON(w, http.StatusOK, rule)
}

func (h *PayrollHandler) ListPayrollInputs(w http.ResponseWriter, r *http.Request) {
	if !h.allowed(r, "payroll:read") {
		h.forbidden(w, r)
		return
	}
	batchID, ok := pathUUID(r, "batch_id")
	if !ok {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	inputs, err := h.Repository.ListPayrollInputs(r.Context(), ids.PayrollInputBatchID(batchID))
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	if inputs == nil {
		inputs = []PayrollInput{}
	}
	apperror.WriteJSON(w, http.StatusOK, inputs)
}

func (h *PayrollHandler) CorrectPayrollInput(w http.ResponseWriter, r *http.Request) {
	if !h.allowed(r, "payroll:manage") {
		h.forbidden(w, r)
		return
	}
	inputID, ok := pathUUID(r, "input_id")
	actorID, actorOK := httpinfra.UserIDFromContext(r.Context())
	if !ok || !actorOK {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	req, err := DecodeJSON[CorrectPayrollInputRequest](r, 1<<20)
	if err != nil || req.Value.IsNegative() || strings.TrimSpace(req.Unit) == "" || len(strings.TrimSpace(req.Reason)) == 0 || len(req.Reason) > 500 {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	input, err := h.Repository.CorrectPayrollInput(r.Context(), ids.PayrollInputID(inputID), req, ids.UserID(actorID))
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	h.emitPayrollAudit(r, audit.Event{
		Name:     "payroll.input.corrected",
		Resource: input.ID.String(),
		Outcome:  "success",
		Details: map[string]string{
			"input_id":      input.ID.String(),
			"employment_id": input.EmploymentID.String(),
			"metric_code":   input.MetricCode,
			"status":        input.Status,
		},
	})
	apperror.WriteJSON(w, http.StatusOK, input)
}

func (h *PayrollHandler) PreviewPayrollFormula(w http.ResponseWriter, r *http.Request) {
	if !h.allowed(r, "payroll:manage") {
		h.forbidden(w, r)
		return
	}
	req, err := DecodeJSON[PreviewPayrollFormulaRequest](r, 1<<20)
	if err != nil || len(req.Expression) == 0 || len(req.Expression) > 4096 || len(req.Values) > 100 {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	value, err := formula.Evaluate(req.Expression, req.Values)
	if err != nil {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	apperror.WriteJSON(w, http.StatusOK, PreviewPayrollFormulaResponse{Value: value.String()})
}

func (h *PayrollHandler) ApprovePayrollInputBatch(w http.ResponseWriter, r *http.Request) {
	h.transitionPayrollInputBatch(w, r, "approve")
}

func (h *PayrollHandler) LockPayrollInputBatch(w http.ResponseWriter, r *http.Request) {
	h.transitionPayrollInputBatch(w, r, "lock")
}

func (h *PayrollHandler) transitionPayrollInputBatch(w http.ResponseWriter, r *http.Request, action string) {
	permission := "payroll:manage"
	if action == "approve" {
		permission = "payroll:approve"
	}
	if !h.allowed(r, permission) {
		h.forbidden(w, r)
		return
	}
	batchID, ok := pathUUID(r, "batch_id")
	actorID, actorOK := httpinfra.UserIDFromContext(r.Context())
	if !ok || !actorOK {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	var batch PayrollInputBatch
	var err error
	if action == "approve" {
		batch, err = h.Repository.ApprovePayrollInputBatch(r.Context(), ids.PayrollInputBatchID(batchID), ids.UserID(actorID))
	} else {
		batch, err = h.Repository.LockPayrollInputBatch(r.Context(), ids.PayrollInputBatchID(batchID), ids.UserID(actorID))
	}
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	eventName := "payroll.input.approved"
	if action == "lock" {
		eventName = "payroll.input.locked"
	}
	h.emitPayrollAudit(r, audit.Event{
		Name:     eventName,
		Resource: batch.ID.String(),
		Outcome:  "success",
		Details: map[string]string{
			"batch_id": batch.ID.String(),
			"status":   batch.Status,
		},
	})
	apperror.WriteJSON(w, http.StatusOK, batch)
}

func (h *PayrollHandler) SimulatePayroll(w http.ResponseWriter, r *http.Request) {
	empID, ok := pathUUID(r, "id")
	if !ok || !h.allowedCalculate(r) {
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
	actorID, ok := httpinfra.UserIDFromContext(r.Context())
	if !ok {
		h.forbidden(w, r)
		return
	}
	var result FullPayslipResponse
	if rawPolicyVersionID := strings.TrimSpace(r.URL.Query().Get("policy_version_id")); rawPolicyVersionID != "" {
		policyVersionID, parseErr := uuid.Parse(rawPolicyVersionID)
		if parseErr != nil {
			apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
			return
		}
		result, err = h.Repository.SimulateEmployeePayrollWithPolicy(r.Context(), ids.EmployeeID(empID), year, month, policyVersionID, ids.UserID(actorID))
	} else {
		result, err = h.Repository.SimulateEmployeePayroll(r.Context(), ids.EmployeeID(empID), year, month, ids.UserID(actorID))
	}
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusOK, result)
}

func (h *PayrollHandler) CalculateBatchPayroll(w http.ResponseWriter, r *http.Request) {

	if !h.allowedCalculate(r) {
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
	actorID, actorOK := httpinfra.UserIDFromContext(r.Context())
	if !actorOK {
		h.forbidden(w, r)
		return
	}
	resp, err := h.Repository.CalculateBatchPayroll(r.Context(), req, ids.UserID(actorID))
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusOK, resp)
}

func (h *PayrollHandler) allowedCalculate(r *http.Request) bool {
	u, ok := httpinfra.UserIDFromContext(r.Context())
	if !ok || h.Authorizer == nil {
		return false
	}
	return h.Authorizer.HasPermission(r.Context(), u, "payroll:calculate")
}

func (h *PayrollHandler) allowedRead(r *http.Request) bool {
	u, ok := httpinfra.UserIDFromContext(r.Context())
	if !ok || h.Authorizer == nil {
		return false
	}
	if h.Authorizer.HasPermission(r.Context(), u, "payroll:read") {
		return true
	}
	member, err := h.Authorizer.IsMember(r.Context(), u)
	return err == nil && member
}

func (h *PayrollHandler) allowedMember(r *http.Request) bool {
	u, ok := httpinfra.UserIDFromContext(r.Context())
	if !ok || h.Authorizer == nil {
		return false
	}
	if member, err := h.Authorizer.IsMember(r.Context(), u); err == nil && member {
		return true
	}
	return h.Authorizer.HasPermission(r.Context(), u, "payroll:read")
}

func (h *PayrollHandler) allowedManage(r *http.Request) bool {
	u, ok := httpinfra.UserIDFromContext(r.Context())
	if !ok || h.Authorizer == nil {
		return false
	}
	return h.Authorizer.HasPermission(r.Context(), u, "payroll:components:manage") || h.Authorizer.HasPermission(r.Context(), u, "payroll:manage")
}

func (h *PayrollHandler) allowed(r *http.Request, permission string) bool {
	u, ok := httpinfra.UserIDFromContext(r.Context())
	return ok && h.Authorizer != nil && h.Authorizer.HasPermission(r.Context(), u, permission)
}

func (h *PayrollHandler) emitPayrollAudit(r *http.Request, event audit.Event) {
	if h.Audit == nil {
		return
	}
	actorID, ok := httpinfra.UserIDFromContext(r.Context())
	if !ok {
		return
	}
	event.ActorID = actorID.String()
	_ = h.Audit.Emit(r.Context(), event)
}

func (h *PayrollHandler) forbidden(w http.ResponseWriter, r *http.Request) {
	apperror.WriteError(w, r, apperror.Forbidden(apperror.KeyForbidden))
}
func (h *PayrollHandler) writeError(w http.ResponseWriter, r *http.Request, err error) {
	if errors.Is(err, pgx.ErrNoRows) || errors.Is(err, ErrComponentNotFound) || errors.Is(err, ErrEmploymentNotFound) || errors.Is(err, ErrPayrollSettingsNotFound) || errors.Is(err, ErrIncentivePolicyNotFound) || errors.Is(err, ErrPayrollInputNotFound) || errors.Is(err, ErrRuleNotFound) || errors.Is(err, ErrTierNotFound) || errors.Is(err, ErrAssignmentNotFound) || errors.Is(err, ErrPayrollPeriodNotFound) || errors.Is(err, ErrEmployerNotFound) {
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
	if errors.Is(err, ErrPayrollImpactNotFound) {
		apperror.WriteError(w, r, apperror.NotFound(apperror.KeyNotFound))
		return
	}
	if errors.Is(err, ErrPayrollImpactValidation) || errors.Is(err, ErrPayrollValidation) || errors.Is(err, ErrPayrollPeriodValidation) {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	if errors.Is(err, ErrComponentConflict) || errors.Is(err, ErrConflict) || errors.Is(err, ErrAssignmentConflict) || errors.Is(err, ErrPayrollPeriodConflict) || errors.Is(err, ErrPayslipConflict) || errors.Is(err, ErrPayrollInputConflict) {
		apperror.WriteError(w, r, apperror.Conflict(apperror.KeyConflict))
		return
	}
	apperror.WriteError(w, r, apperror.Internal(err))
}
