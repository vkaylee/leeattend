package api

import (
	"context"
	"encoding/json"
	"net/http"
	"strconv"
	"strings"
	"time"

	"github.com/google/uuid"
	"github.com/shopspring/decimal"
	"github.com/vkaylee/leeattend/apiserver/internal/apperror"
	"github.com/vkaylee/leeattend/apiserver/internal/domain/ids"
	payrolldomain "github.com/vkaylee/leeattend/apiserver/internal/domain/payroll"
	httpinfra "github.com/vkaylee/leeattend/apiserver/internal/infra/http"
)

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

type PayslipSegmentResponse struct {
	ID                  uuid.UUID       `json:"id"`
	PayslipID           uuid.UUID       `json:"payslip_id"`
	StartDate           string          `json:"start_date"`
	EndDate             string          `json:"end_date"`
	DepartmentID        *uuid.UUID      `json:"department_id"`
	AppliedBaseSalary   decimal.Decimal `json:"applied_base_salary"`
	AppliedStdDays      decimal.Decimal `json:"applied_std_days"`
	AppliedHoursPerDay  decimal.Decimal `json:"applied_hours_per_day"`
	ActualWorkDays      decimal.Decimal `json:"actual_work_days"`
	PaidWorkDays        decimal.Decimal `json:"paid_work_days"`
	Amount              decimal.Decimal `json:"amount"`
	Reason              string          `json:"reason"`
	StdDaysSource       *string         `json:"std_days_source,omitempty"`
	WorkPolicyVersionID *uuid.UUID      `json:"work_policy_version_id,omitempty"`
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

type PayrollImpactBatchRequest struct {
	EmploymentIDs []uuid.UUID `json:"employment_ids"`
	Year          int         `json:"year"`
	Month         int         `json:"month"`
	BeforeRunID   uuid.UUID   `json:"before_run_id"`
	AfterRunID    uuid.UUID   `json:"after_run_id"`
}

type PayrollImpactBatchItem struct {
	EmploymentID     uuid.UUID              `json:"employment_id"`
	InputFingerprint string                 `json:"input_fingerprint,omitempty"`
	Computed         bool                   `json:"computed"`
	Stale            bool                   `json:"stale"`
	StaleReasons     []string               `json:"stale_reasons,omitempty"`
	Reason           string                 `json:"reason,omitempty"`
	Delta            *PayrollImpactResponse `json:"delta,omitempty"`
}

type PayrollImpactBatchResponse struct {
	Total             int                      `json:"total"`
	Computed          int                      `json:"computed"`
	Affected          int                      `json:"affected"`
	Uncomputable      int                      `json:"uncomputable"`
	EvaluatedAt       string                   `json:"evaluated_at"`
	ExpiresAt         string                   `json:"expires_at"`
	StaleAfterSeconds int                      `json:"stale_after_seconds"`
	Items             []PayrollImpactBatchItem `json:"items"`
}

type PayrollImpactResponse struct {
	EmploymentID       uuid.UUID       `json:"employment_id"`
	InputFingerprint   string          `json:"input_fingerprint,omitempty"`
	EvaluatedAt        string          `json:"evaluated_at"`
	ExpiresAt          string          `json:"expires_at"`
	StaleAfterSeconds  int             `json:"stale_after_seconds"`
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

type WorkPolicyBatchSimulationRequest struct {
	Year                      int         `json:"year"`
	Month                     int         `json:"month"`
	EmploymentRelationshipIDs []uuid.UUID `json:"employment_relationship_ids"`
}

type WorkPolicyBatchSimulationItem struct {
	EmploymentRelationshipID uuid.UUID                 `json:"employment_relationship_id"`
	EmploymentID             *uuid.UUID                `json:"employment_id,omitempty"`
	Status                   string                    `json:"status"` // "computed" | "uncomputable"
	Affected                 bool                      `json:"affected"`
	Reason                   string                    `json:"reason,omitempty"`
	InputFingerprint         string                    `json:"input_fingerprint,omitempty"`
	BaselineGross            *decimal.Decimal          `json:"baseline_gross,omitempty"`
	CandidateGross           *decimal.Decimal          `json:"candidate_gross,omitempty"`
	GrossDelta               *decimal.Decimal          `json:"gross_delta,omitempty"`
	BaselineNet              *decimal.Decimal          `json:"baseline_net,omitempty"`
	CandidateNet             *decimal.Decimal          `json:"candidate_net,omitempty"`
	NetDelta                 *decimal.Decimal          `json:"net_delta,omitempty"`
	BaselineOTPay            *decimal.Decimal          `json:"baseline_ot_pay,omitempty"`
	CandidateOTPay           *decimal.Decimal          `json:"candidate_ot_pay,omitempty"`
	OTPayDelta               *decimal.Decimal          `json:"ot_pay_delta,omitempty"`
	Trace                    []PayrollCalculationTrace `json:"trace,omitempty"`
}

type WorkPolicyBatchSimulationResponse struct {
	GroupID           uuid.UUID                       `json:"group_id"`
	VersionID         uuid.UUID                       `json:"version_id"`
	PeriodYear        int                             `json:"period_year"`
	PeriodMonth       int                             `json:"period_month"`
	Total             int                             `json:"total"`
	Computed          int                             `json:"computed"`
	Affected          int                             `json:"affected"`
	Uncomputable      int                             `json:"uncomputable"`
	InputFingerprint  string                          `json:"input_fingerprint"`
	EvaluatedAt       string                          `json:"evaluated_at"`
	ExpiresAt         string                          `json:"expires_at"`
	StaleAfterSeconds int                             `json:"stale_after_seconds"`
	Items             []WorkPolicyBatchSimulationItem `json:"items"`
}

// PayrollRunRepository manages payslips, calculation runs, simulations, impact diffs, and periods.
type PayrollRunRepository interface {
	ListPayslips(context.Context, time.Time, time.Time) ([]PayslipRecord, error)
	ListSalaryManagementRows(context.Context, SalaryManagementQuery) (SalaryManagementResponse, error)
	ListEmployeePayslips(context.Context, ids.EmployeeID) ([]PayslipRecord, error)
	CalculateEmployeePayroll(context.Context, ids.EmployeeID, int, int, ids.UserID) (FullPayslipResponse, error)
	SimulateEmployeePayroll(context.Context, ids.EmployeeID, int, int, ids.UserID) (FullPayslipResponse, error)
	SimulateEmployeePayrollWithPolicy(context.Context, ids.EmployeeID, int, int, uuid.UUID, ids.UserID) (FullPayslipResponse, error)
	SimulateWorkPolicyVersionBatch(context.Context, uuid.UUID, uuid.UUID, WorkPolicyBatchSimulationRequest) (WorkPolicyBatchSimulationResponse, error)
	CalculateBatchPayroll(context.Context, BatchCalculatePayrollRequest, ids.UserID) (BatchCalculatePayrollResponse, error)
	ListPayrollCalculationRuns(context.Context, ids.EmploymentID, int, int, int) ([]PayrollCalculationRun, error)
	ComparePayrollImpact(context.Context, ids.EmploymentID, int, int, uuid.UUID, uuid.UUID) (PayrollImpactResponse, error)
	ComparePayrollImpactBatch(context.Context, PayrollImpactBatchRequest) (PayrollImpactBatchResponse, error)
	GetPayrollPeriod(context.Context, ids.EmployerID, int, int) (PayrollPeriod, error)
	LockPayrollPeriod(context.Context, payrolldomain.PayrollPeriodMutation) (PayrollPeriod, error)
	ReopenPayrollPeriod(context.Context, payrolldomain.PayrollPeriodMutation) (PayrollPeriod, error)
}

func (h *PayrollHandler) ListPayslips(w http.ResponseWriter, r *http.Request) {

	if !hasPermission(r, h.Authorizer, "payroll:read") {
		writeForbidden(w, r)
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
	if !empOK || !hasPermission(r, h.Authorizer, "payroll:read") {
		writeForbidden(w, r)
		return
	}
	rows, err := h.Repository.ListEmployeePayslips(r.Context(), ids.EmployeeID(empID))
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusOK, rows)
}

func (h *PayrollHandler) GetPayrollPeriod(w http.ResponseWriter, r *http.Request) {
	if !hasPermission(r, h.Authorizer, "payroll:read") {
		writeForbidden(w, r)
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
	if !hasPermission(r, h.Authorizer, "payroll:periods:lock") {
		writeForbidden(w, r)
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
	actorID, _ := httpinfra.UserIDValueFromContext(r.Context())
	period, err := h.Repository.LockPayrollPeriod(r.Context(), payrolldomain.PayrollPeriodMutation{
		EmployerID: ids.EmployerID(employerID), Year: year, Month: month, ExpectedVersion: request.Version, ActorID: actorID,
	})
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusOK, period)
}

func (h *PayrollHandler) ReopenPayrollPeriod(w http.ResponseWriter, r *http.Request) {
	if !hasPermission(r, h.Authorizer, "payroll:periods:reopen") {
		writeForbidden(w, r)
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
	actorID, _ := httpinfra.UserIDValueFromContext(r.Context())
	period, err := h.Repository.ReopenPayrollPeriod(r.Context(), payrolldomain.PayrollPeriodMutation{
		EmployerID: ids.EmployerID(employerID), Year: year, Month: month, ExpectedVersion: request.Version,
		Reason: strings.TrimSpace(request.Reason), ActorID: actorID,
	})
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusOK, period)
}

func isValidPayrollPeriod(year, month int) bool {
	return year >= 2000 && year <= 2100 && month >= 1 && month <= 12
}

func parsePayrollYearMonth(r *http.Request) (int, int, bool) {
	year, yearErr := strconv.Atoi(r.URL.Query().Get("year"))
	month, monthErr := strconv.Atoi(r.URL.Query().Get("month"))
	if yearErr != nil || monthErr != nil || !isValidPayrollPeriod(year, month) {
		return 0, 0, false
	}
	return year, month, true
}

func parseImpactRunPair(r *http.Request) (uuid.UUID, uuid.UUID, bool) {
	beforeRunID, beforeErr := uuid.Parse(r.URL.Query().Get("before_run_id"))
	afterRunID, afterErr := uuid.Parse(r.URL.Query().Get("after_run_id"))
	if beforeErr != nil || afterErr != nil || beforeRunID == uuid.Nil || afterRunID == uuid.Nil || beforeRunID == afterRunID {
		return uuid.Nil, uuid.Nil, false
	}
	return beforeRunID, afterRunID, true
}

func validPayrollImpactBatchRequest(req PayrollImpactBatchRequest) bool {
	if len(req.EmploymentIDs) == 0 || len(req.EmploymentIDs) > 100 || hasDuplicateUUIDs(req.EmploymentIDs) {
		return false
	}
	if !isValidPayrollPeriod(req.Year, req.Month) {
		return false
	}
	return req.BeforeRunID != uuid.Nil && req.AfterRunID != uuid.Nil && req.BeforeRunID != req.AfterRunID
}

func parseStrictLimitParam(raw string, def, maxVal int) (int, bool) {
	trimmed := strings.TrimSpace(raw)
	if trimmed == "" {
		return def, true
	}
	parsed, err := strconv.Atoi(trimmed)
	if err != nil || parsed < 1 || parsed > maxVal {
		return 0, false
	}
	return parsed, true
}

func (h *PayrollHandler) CalculatePayroll(w http.ResponseWriter, r *http.Request) {
	empID, eOK := pathUUID(r, "id")
	if !eOK || !h.allowedCalculate(r) {
		writeForbidden(w, r)
		return
	}
	year, month, ok := parsePayrollYearMonth(r)
	if !ok {
		writeValidation(w, r)
		return
	}
	actorID, actorOK := httpinfra.UserIDValueFromContext(r.Context())
	if !actorOK {
		writeForbidden(w, r)
		return
	}
	resp, err := h.Repository.CalculateEmployeePayroll(r.Context(), ids.EmployeeID(empID), year, month, actorID)
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusOK, resp)
}

func (h *PayrollHandler) ComparePayrollImpact(w http.ResponseWriter, r *http.Request) {
	if !hasPermission(r, h.Authorizer, "payroll:sensitive:read") {
		writeForbidden(w, r)
		return
	}
	employmentID, ok := pathUUID(r, "id")
	if !ok {
		writeValidation(w, r)
		return
	}
	year, month, ymOK := parsePayrollYearMonth(r)
	beforeRunID, afterRunID, runsOK := parseImpactRunPair(r)
	if !ymOK || !runsOK {
		writeValidation(w, r)
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
	if !hasPermission(r, h.Authorizer, "payroll:sensitive:read") {
		writeForbidden(w, r)
		return
	}
	req, err := DecodeJSON[PayrollImpactBatchRequest](r, 1<<20)
	if err != nil || !validPayrollImpactBatchRequest(req) {
		writeValidation(w, r)
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
	if !hasPermission(r, h.Authorizer, "payroll:read") {
		writeForbidden(w, r)
		return
	}
	employmentID, ok := pathUUID(r, "id")
	if !ok {
		writeValidation(w, r)
		return
	}
	year, month, ymOK := parsePayrollYearMonth(r)
	limit, limitOK := parseStrictLimitParam(r.URL.Query().Get("limit"), 50, 100)
	if !ymOK || !limitOK {
		writeValidation(w, r)
		return
	}
	runs, err := h.Repository.ListPayrollCalculationRuns(r.Context(), ids.EmploymentID(employmentID), year, month, limit)
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	if runs == nil {
		runs = []PayrollCalculationRun{}
	}
	apperror.WriteJSON(w, http.StatusOK, runs)
}

func (h *PayrollHandler) SimulatePayroll(w http.ResponseWriter, r *http.Request) {
	empID, ok := pathUUID(r, "id")
	if !ok || !h.allowedCalculate(r) {
		writeForbidden(w, r)
		return
	}
	year, month, ymOK := parsePayrollYearMonth(r)
	if !ymOK {
		writeValidation(w, r)
		return
	}
	actorID, ok := httpinfra.UserIDValueFromContext(r.Context())
	if !ok {
		writeForbidden(w, r)
		return
	}
	var result FullPayslipResponse
	var err error
	if rawPolicyVersionID := strings.TrimSpace(r.URL.Query().Get("policy_version_id")); rawPolicyVersionID != "" {
		policyVersionID, parseErr := uuid.Parse(rawPolicyVersionID)
		if parseErr != nil {
			writeValidation(w, r)
			return
		}
		result, err = h.Repository.SimulateEmployeePayrollWithPolicy(r.Context(), ids.EmployeeID(empID), year, month, policyVersionID, actorID)
	} else {
		result, err = h.Repository.SimulateEmployeePayroll(r.Context(), ids.EmployeeID(empID), year, month, actorID)
	}
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusOK, result)
}

func normalizeBatchCalculateRequest(req *BatchCalculatePayrollRequest) bool {
	if len(req.EmploymentIDs) == 0 && len(req.EmployeeIDs) > 0 {
		req.EmploymentIDs = req.EmployeeIDs
	}
	if len(req.EmploymentIDs) == 0 || len(req.EmploymentIDs) > 500 || hasDuplicateUUIDs(req.EmploymentIDs) {
		return false
	}
	return isValidPayrollPeriod(req.Year, req.Month)
}

func (h *PayrollHandler) CalculateBatchPayroll(w http.ResponseWriter, r *http.Request) {
	if !h.allowedCalculate(r) {
		writeForbidden(w, r)
		return
	}
	req, err := DecodeJSON[BatchCalculatePayrollRequest](r, 1<<20)
	if err != nil || !normalizeBatchCalculateRequest(&req) {
		writeValidation(w, r)
		return
	}
	actorID, actorOK := httpinfra.UserIDValueFromContext(r.Context())
	if !actorOK {
		writeForbidden(w, r)
		return
	}
	batchCtx, cancel := context.WithTimeout(r.Context(), 30*time.Second)
	defer cancel()
	resp, err := h.Repository.CalculateBatchPayroll(batchCtx, req, actorID)
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusOK, resp)
}

func validateWorkPolicyBatchSimRequest(req WorkPolicyBatchSimulationRequest) bool {
	if req.Year < 2000 || req.Year > 2100 || req.Month < 1 || req.Month > 12 {
		return false
	}
	if len(req.EmploymentRelationshipIDs) == 0 || len(req.EmploymentRelationshipIDs) > 100 || hasDuplicateUUIDs(req.EmploymentRelationshipIDs) {
		return false
	}
	for _, relID := range req.EmploymentRelationshipIDs {
		if relID == uuid.Nil {
			return false
		}
	}
	return true
}

func (h *PayrollHandler) SimulateWorkPolicyVersionBatch(w http.ResponseWriter, r *http.Request) {
	if !hasPermission(r, h.Authorizer, "payroll:sensitive:read") {
		writeForbidden(w, r)
		return
	}
	groupID, groupOK := pathUUID(r, "group_id")
	versionID, versionOK := pathUUID(r, "version_id")
	if !groupOK || !versionOK {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	req, err := DecodeJSON[WorkPolicyBatchSimulationRequest](r, 1<<20)
	if err != nil || !validateWorkPolicyBatchSimRequest(req) {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	simCtx, cancel := context.WithTimeout(r.Context(), 30*time.Second)
	defer cancel()
	resp, err := h.Repository.SimulateWorkPolicyVersionBatch(simCtx, groupID, versionID, req)
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusOK, resp)
}
