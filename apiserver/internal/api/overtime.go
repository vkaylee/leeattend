package api

import (
	"context"
	"encoding/json"
	"errors"
	"net/http"
	"strings"
	"time"

	"github.com/google/uuid"
	"github.com/jackc/pgx/v5"
	"github.com/shopspring/decimal"
	"github.com/vkaylee/leeattend/apiserver/internal/apperror"
	httpinfra "github.com/vkaylee/leeattend/apiserver/internal/infra/http"
)

const (
	PermissionOvertimeClaim               = "overtime:claim"
	PermissionOvertimeApprove             = "overtime:approve"
	PermissionOvertimeClaimOnBehalf       = "overtime:claim_on_behalf"
	PermissionOvertimePoliciesManage      = "overtime:policies:manage"
	PermissionOvertimeRequestCreate       = "overtime:request:create"
	PermissionOvertimeRequestCreateBehalf = "overtime:request:create_on_behalf"
	PermissionOvertimeRequestRead         = "overtime:request:read"
	OvertimeRequestBehalfAssignmentType   = "overtime_request"
	OvertimeClaimBehalfAssignmentType     = "overtime_claim"
)

// ErrOvertimeValidation marks overtime payload validation failures surfaced by repositories.
var ErrOvertimeValidation = errors.New("overtime: validation failed")

type OvertimePolicy struct {
	ID          uuid.UUID `json:"id"`
	WorkspaceID uuid.UUID `json:"workspace_id"`
	Name        string    `json:"name"`
	Description *string   `json:"description"`
	CreatedAt   string    `json:"created_at"`
	UpdatedAt   string    `json:"updated_at"`
}

type OvertimeRule struct {
	ID         uuid.UUID       `json:"id"`
	PolicyID   uuid.UUID       `json:"policy_id"`
	Priority   int             `json:"priority"`
	Multiplier decimal.Decimal `json:"multiplier"`
	TimeStart  *string         `json:"time_start"`
	TimeEnd    *string         `json:"time_end"`
	DaysOfWeek int             `json:"days_of_week"`
	IsHoliday  bool            `json:"is_holiday"`
	CreatedAt  string          `json:"created_at"`
	UpdatedAt  string          `json:"updated_at"`
}

type OvertimePolicyListResponse struct {
	Data  []OvertimePolicy `json:"data"`
	Total int64            `json:"total"`
	Page  int              `json:"page"`
	Limit int              `json:"limit"`
}

type CreateOvertimePolicyRequest struct {
	Name        string  `json:"name"`
	Description *string `json:"description"`
}

type CreateOvertimeRuleRequest struct {
	Priority   int             `json:"priority"`
	Multiplier decimal.Decimal `json:"multiplier"`
	TimeStart  *string         `json:"time_start"`
	TimeEnd    *string         `json:"time_end"`
	DaysOfWeek int             `json:"days_of_week"`
	IsHoliday  bool            `json:"is_holiday"`
}

type AssignOvertimePolicyRequest struct {
	AssigneeType  string     `json:"assignee_type"`
	AssigneeID    *uuid.UUID `json:"assignee_id"`
	PolicyID      uuid.UUID  `json:"policy_id"`
	EffectiveFrom string     `json:"effective_from"`
}

type OvertimePolicyDetail struct {
	Policy OvertimePolicy `json:"policy"`
	Rules  []OvertimeRule `json:"rules"`
}

type OvertimeRequest struct {
	ID                uuid.UUID       `json:"id"`
	WorkspaceID       uuid.UUID       `json:"workspace_id"`
	RequestedByUserID uuid.UUID       `json:"requested_by_user_id"`
	DepartmentID      *uuid.UUID      `json:"department_id"`
	PlannedDate       string          `json:"planned_date"`
	PlannedStartTime  *string         `json:"planned_start_time"`
	PlannedEndTime    *string         `json:"planned_end_time"`
	PlannedMinutes    int             `json:"planned_minutes"`
	Reason            string          `json:"reason"`
	Status            string          `json:"status"`
	WorkflowID        *uuid.UUID      `json:"workflow_id"`
	CurrentStep       int             `json:"current_step"`
	QuotaWarning      json.RawMessage `json:"quota_warning,omitempty"`
	ReconciledAt      *string         `json:"reconciled_at,omitempty"`
	ReconciledBy      *uuid.UUID      `json:"reconciled_by,omitempty"`
	AttachmentFileIDs []uuid.UUID     `json:"attachment_file_ids,omitempty"`
	CreatedAt         string          `json:"created_at"`
	UpdatedAt         string          `json:"updated_at"`
}

type OvertimeRequestEmployee struct {
	ID                uuid.UUID  `json:"id"`
	OvertimeRequestID uuid.UUID  `json:"overtime_request_id"`
	EmploymentID      uuid.UUID  `json:"employment_id"`
	ConsentStatus     string     `json:"consent_status"`
	ConsentAt         *string    `json:"consent_at,omitempty"`
	DeclineReason     *string    `json:"decline_reason,omitempty"`
	ActualOTMinutes   *int       `json:"actual_ot_minutes,omitempty"`
	MatchedSummaryID  *uuid.UUID `json:"matched_summary_id,omitempty"`
	CreatedAt         string     `json:"created_at"`
}

type OvertimeRequestDetail struct {
	Request   OvertimeRequest           `json:"request"`
	Employees []OvertimeRequestEmployee `json:"employees"`
}

type OvertimeRequestListResponse struct {
	Data  []OvertimeRequest `json:"data"`
	Total int64             `json:"total"`
	Page  int               `json:"page"`
	Limit int               `json:"limit"`
}

type CreateOvertimeRequest struct {
	DepartmentID      *uuid.UUID  `json:"department_id"`
	PlannedDate       string      `json:"planned_date"`
	PlannedStartTime  *string     `json:"planned_start_time"`
	PlannedEndTime    *string     `json:"planned_end_time"`
	PlannedMinutes    int         `json:"planned_minutes"`
	Reason            string      `json:"reason"`
	Employees         []uuid.UUID `json:"employees"`
	AttachmentFileIDs []uuid.UUID `json:"attachment_file_ids"`
}

type EmployeeConsentRequest struct {
	Status        string  `json:"status"`
	DeclineReason *string `json:"decline_reason"`
}

type ClaimOTRequest struct {
	Date           string  `json:"date"`
	ClaimedMinutes int     `json:"claimed_minutes"`
	Reason         *string `json:"reason"`
}

type ApproveOTRequest struct {
	Date            string  `json:"date"`
	ApprovedMinutes int     `json:"approved_minutes"`
	ManagerNote     *string `json:"manager_note"`
	Status          string  `json:"status"`
}

type OvertimeRepository interface {
	ListOvertimePolicies(context.Context, uuid.UUID, int, int) (OvertimePolicyListResponse, error)
	CreateOvertimePolicy(context.Context, uuid.UUID, CreateOvertimePolicyRequest) (OvertimePolicy, error)
	GetOvertimePolicy(context.Context, uuid.UUID, uuid.UUID) (OvertimePolicyDetail, error)
	AddOvertimeRule(context.Context, uuid.UUID, uuid.UUID, CreateOvertimeRuleRequest) (OvertimeRule, error)
	AssignOvertimePolicy(context.Context, uuid.UUID, AssignOvertimePolicyRequest) error
	ListOvertimeRequests(context.Context, uuid.UUID, int, int) (OvertimeRequestListResponse, error)
	CreateOvertimeRequest(context.Context, uuid.UUID, uuid.UUID, CreateOvertimeRequest) (OvertimeRequest, error)
	GetOvertimeRequest(context.Context, uuid.UUID, uuid.UUID) (OvertimeRequestDetail, error)
	SubmitOvertimeRequest(context.Context, uuid.UUID, uuid.UUID) error
	ConsentOvertimeRequest(context.Context, uuid.UUID, uuid.UUID, uuid.UUID, EmployeeConsentRequest) error
	GetEmployee(context.Context, uuid.UUID, uuid.UUID) (EmployeeRecord, error)
	EmployeeDepartmentID(context.Context, uuid.UUID, uuid.UUID) (*uuid.UUID, error)
	ClaimOvertime(context.Context, uuid.UUID, uuid.UUID, ClaimOTRequest, uuid.UUID) error
	ApproveOvertime(context.Context, uuid.UUID, uuid.UUID, ApproveOTRequest, uuid.UUID) error
}

type OvertimeHandler struct {
	Repository       OvertimeRepository
	BehalfRepository BehalfAssignmentRepository
	Authorizer       WorkspaceAuthorizer
}

func (h OvertimeHandler) userID(r *http.Request) (uuid.UUID, bool) {
	return httpinfra.UserIDFromContext(r.Context())
}

func (h OvertimeHandler) allowed(r *http.Request, workspaceID uuid.UUID, permission string) bool {
	userID, ok := h.userID(r)
	return ok && h.Authorizer != nil && h.Authorizer.HasPermission(r.Context(), userID, workspaceID, permission)
}

func (h OvertimeHandler) scopedAllowed(r *http.Request, workspaceID uuid.UUID, permission string, departmentID *uuid.UUID) bool {
	userID, ok := h.userID(r)
	if !ok || h.Authorizer == nil || !h.Authorizer.HasPermission(r.Context(), userID, workspaceID, permission) {
		return false
	}
	scoped, ok := h.Authorizer.(interface {
		HasScopedPermission(context.Context, uuid.UUID, uuid.UUID, string, *uuid.UUID) bool
	})
	if !ok {
		return true
	}
	return scoped.HasScopedPermission(r.Context(), userID, workspaceID, permission, departmentID)
}

func (h OvertimeHandler) forbidden(w http.ResponseWriter, r *http.Request) {
	apperror.WriteError(w, r, apperror.Forbidden(apperror.KeyForbidden))
}

func (h OvertimeHandler) fail(w http.ResponseWriter, r *http.Request, err error) {
	if errors.Is(err, pgx.ErrNoRows) {
		apperror.WriteError(w, r, apperror.NotFound(apperror.KeyNotFound))
		return
	}
	if errors.Is(err, ErrOvertimeValidation) {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	var appErr *apperror.AppError
	if errors.As(err, &appErr) {
		apperror.WriteError(w, r, appErr)
		return
	}
	apperror.WriteError(w, r, apperror.Internal(err))
}

func validOvertimeDate(raw string) bool {
	_, err := time.Parse("2006-01-02", strings.TrimSpace(raw))
	return err == nil
}

func validOvertimeClock(raw *string) bool {
	if raw == nil || strings.TrimSpace(*raw) == "" {
		return true
	}
	value := strings.TrimSpace(*raw)
	if _, err := time.Parse("15:04:05", value); err == nil {
		return true
	}
	_, err := time.Parse("15:04", value)
	return err == nil
}

func (h OvertimeHandler) ListPolicies(w http.ResponseWriter, r *http.Request) {
	workspaceID, ok := pathUUID(r, "ws_id")
	if !ok {
		h.forbidden(w, r)
		return
	}
	pagination, err := ParsePagination(r.URL.Query())
	if err != nil {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	result, err := h.Repository.ListOvertimePolicies(r.Context(), workspaceID, pagination.Page, pagination.Limit)
	if err != nil {
		h.fail(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusOK, result)
}

func (h OvertimeHandler) CreatePolicy(w http.ResponseWriter, r *http.Request) {
	workspaceID, ok := pathUUID(r, "ws_id")
	if !ok || !h.allowed(r, workspaceID, PermissionOvertimePoliciesManage) {
		h.forbidden(w, r)
		return
	}
	req, err := DecodeJSON[CreateOvertimePolicyRequest](r, 1<<20)
	if err != nil || strings.TrimSpace(req.Name) == "" {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	result, err := h.Repository.CreateOvertimePolicy(r.Context(), workspaceID, req)
	if err != nil {
		h.fail(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusCreated, result)
}

func (h OvertimeHandler) GetPolicy(w http.ResponseWriter, r *http.Request) {
	workspaceID, ok := pathUUID(r, "ws_id")
	policyID, policyOK := pathUUID(r, "policy_id")
	if !ok || !policyOK {
		h.forbidden(w, r)
		return
	}
	result, err := h.Repository.GetOvertimePolicy(r.Context(), workspaceID, policyID)
	if err != nil {
		h.fail(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusOK, result)
}

func (h OvertimeHandler) AddRule(w http.ResponseWriter, r *http.Request) {
	workspaceID, ok := pathUUID(r, "ws_id")
	policyID, policyOK := pathUUID(r, "policy_id")
	if !ok || !policyOK || !h.allowed(r, workspaceID, PermissionOvertimePoliciesManage) {
		h.forbidden(w, r)
		return
	}
	req, err := DecodeJSON[CreateOvertimeRuleRequest](r, 1<<20)
	if err != nil || !req.Multiplier.IsPositive() || !validOvertimeClock(req.TimeStart) || !validOvertimeClock(req.TimeEnd) || req.DaysOfWeek < 0 || req.DaysOfWeek > 127 {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	result, err := h.Repository.AddOvertimeRule(r.Context(), workspaceID, policyID, req)
	if err != nil {
		h.fail(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusCreated, result)
}

func (h OvertimeHandler) AssignPolicy(w http.ResponseWriter, r *http.Request) {
	workspaceID, ok := pathUUID(r, "ws_id")
	if !ok || !h.allowed(r, workspaceID, PermissionOvertimePoliciesManage) {
		h.forbidden(w, r)
		return
	}
	req, err := DecodeJSON[AssignOvertimePolicyRequest](r, 1<<20)
	if err != nil || req.PolicyID == uuid.Nil || !validOvertimeDate(req.EffectiveFrom) {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	switch req.AssigneeType {
	case "workspace":
		if req.AssigneeID != nil {
			apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
			return
		}
	case "department", "employee":
		if req.AssigneeID == nil {
			apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
			return
		}
	default:
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	if err := h.Repository.AssignOvertimePolicy(r.Context(), workspaceID, req); err != nil {
		h.fail(w, r, err)
		return
	}
	w.WriteHeader(http.StatusOK)
}

func (h OvertimeHandler) ListRequests(w http.ResponseWriter, r *http.Request) {
	workspaceID, ok := pathUUID(r, "ws_id")
	if !ok || !h.allowed(r, workspaceID, PermissionOvertimeRequestRead) {
		h.forbidden(w, r)
		return
	}
	pagination, err := ParsePagination(r.URL.Query())
	if err != nil {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	result, err := h.Repository.ListOvertimeRequests(r.Context(), workspaceID, pagination.Page, pagination.Limit)
	if err != nil {
		h.fail(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusOK, result)
}

func (h OvertimeHandler) CreateRequest(w http.ResponseWriter, r *http.Request) {
	workspaceID, ok := pathUUID(r, "ws_id")
	userID, userOK := h.userID(r)
	if !ok || !userOK || !h.allowed(r, workspaceID, PermissionOvertimeRequestCreate) {
		h.forbidden(w, r)
		return
	}
	req, err := DecodeJSON[CreateOvertimeRequest](r, 1<<20)
	if err != nil || !validOvertimeDate(req.PlannedDate) || req.PlannedMinutes <= 0 || strings.TrimSpace(req.Reason) == "" || len(req.Employees) == 0 || !validOvertimeClock(req.PlannedStartTime) || !validOvertimeClock(req.PlannedEndTime) {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	behalfAllowed := h.allowed(r, workspaceID, PermissionOvertimeRequestCreateBehalf)
	for _, employeeID := range req.Employees {
		employee, err := h.Repository.GetEmployee(r.Context(), workspaceID, employeeID)
		if err != nil {
			h.fail(w, r, err)
			return
		}
		if employee.UserID == nil || *employee.UserID != userID {
			if !behalfAllowed {
				h.forbidden(w, r)
				return
			}
		}
	}
	result, err := h.Repository.CreateOvertimeRequest(r.Context(), workspaceID, userID, req)
	if err != nil {
		h.fail(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusCreated, result)
}

func (h OvertimeHandler) GetRequest(w http.ResponseWriter, r *http.Request) {
	workspaceID, ok := pathUUID(r, "ws_id")
	requestID, requestOK := pathUUID(r, "id")
	if !ok || !requestOK || !h.allowed(r, workspaceID, PermissionOvertimeRequestRead) {
		h.forbidden(w, r)
		return
	}
	result, err := h.Repository.GetOvertimeRequest(r.Context(), workspaceID, requestID)
	if err != nil {
		h.fail(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusOK, result)
}

func (h OvertimeHandler) SubmitRequest(w http.ResponseWriter, r *http.Request) {
	workspaceID, ok := pathUUID(r, "ws_id")
	requestID, requestOK := pathUUID(r, "id")
	if !ok || !requestOK || !h.allowed(r, workspaceID, PermissionOvertimeRequestCreate) {
		h.forbidden(w, r)
		return
	}
	if err := h.Repository.SubmitOvertimeRequest(r.Context(), workspaceID, requestID); err != nil {
		h.fail(w, r, err)
		return
	}
	w.WriteHeader(http.StatusOK)
}

func (h OvertimeHandler) ConsentRequest(w http.ResponseWriter, r *http.Request) {
	workspaceID, ok := pathUUID(r, "ws_id")
	requestID, requestOK := pathUUID(r, "id")
	employeeID, employeeOK := pathUUID(r, "emp_id")
	userID, userOK := h.userID(r)
	if !ok || !requestOK || !employeeOK || !userOK {
		h.forbidden(w, r)
		return
	}
	employee, err := h.Repository.GetEmployee(r.Context(), workspaceID, employeeID)
	if err != nil {
		h.fail(w, r, err)
		return
	}
	if employee.UserID == nil || *employee.UserID != userID {
		if !h.allowed(r, workspaceID, PermissionOvertimeRequestCreateBehalf) {
			h.forbidden(w, r)
			return
		}
	}
	req, err := DecodeJSON[EmployeeConsentRequest](r, 1<<20)
	if err != nil || (req.Status != "accepted" && req.Status != "declined") {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	if err := h.Repository.ConsentOvertimeRequest(r.Context(), workspaceID, requestID, employeeID, req); err != nil {
		h.fail(w, r, err)
		return
	}
	w.WriteHeader(http.StatusOK)
}

func (h OvertimeHandler) ListRequestBehalf(w http.ResponseWriter, r *http.Request) {
	workspaceID, ok := pathUUID(r, "ws_id")
	if !ok {
		h.forbidden(w, r)
		return
	}
	pagination, err := ParsePagination(r.URL.Query())
	if err != nil {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	items, err := h.BehalfRepository.ListBehalfAssignments(r.Context(), workspaceID, OvertimeRequestBehalfAssignmentType)
	if err != nil {
		h.fail(w, r, err)
		return
	}
	total := len(items)
	start := pagination.Offset
	if start > total {
		start = total
	}
	end := start + pagination.Limit
	if end > total {
		end = total
	}
	apperror.WriteJSON(w, http.StatusOK, AttendancePage[BehalfAssignment]{Data: items[start:end], Total: int64(total), Page: pagination.Page, Limit: pagination.Limit})
}

func (h OvertimeHandler) CreateRequestBehalf(w http.ResponseWriter, r *http.Request) {
	workspaceID, ok := pathUUID(r, "ws_id")
	if !ok || !h.allowed(r, workspaceID, PermissionOvertimePoliciesManage) {
		h.forbidden(w, r)
		return
	}
	req, err := DecodeJSON[CreateBehalfAssignmentRequest](r, 1<<20)
	if err != nil || req.AssigneeUserID == uuid.Nil || (req.TargetEmployeeID == nil) == (req.TargetDepartmentID == nil) {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	result, err := h.BehalfRepository.CreateBehalfAssignment(r.Context(), workspaceID, req, OvertimeRequestBehalfAssignmentType)
	if err != nil {
		h.fail(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusOK, result)
}

func (h OvertimeHandler) RequestBehalfTargets(w http.ResponseWriter, r *http.Request) {
	workspaceID, ok := pathUUID(r, "ws_id")
	userID, userOK := h.userID(r)
	if !ok || !userOK {
		apperror.WriteError(w, r, apperror.Unauthorized(apperror.KeyAuthenticationRequired))
		return
	}
	result, err := h.BehalfRepository.ListMyBehalfTargets(r.Context(), workspaceID, userID, OvertimeRequestBehalfAssignmentType)
	if err != nil {
		h.fail(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusOK, result)
}

func (h OvertimeHandler) DeleteRequestBehalf(w http.ResponseWriter, r *http.Request) {
	workspaceID, ok := pathUUID(r, "ws_id")
	assignmentID, assignmentOK := pathUUID(r, "id")
	if !ok || !assignmentOK || !h.allowed(r, workspaceID, PermissionOvertimePoliciesManage) {
		h.forbidden(w, r)
		return
	}
	if err := h.BehalfRepository.DeleteBehalfAssignment(r.Context(), workspaceID, assignmentID); err != nil {
		h.fail(w, r, err)
		return
	}
	w.WriteHeader(http.StatusOK)
}

func (h OvertimeHandler) ClaimOT(w http.ResponseWriter, r *http.Request) {
	workspaceID, ok := pathUUID(r, "ws_id")
	employeeID, employeeOK := pathUUID(r, "employee_id")
	userID, userOK := h.userID(r)
	if !ok || !employeeOK || !userOK {
		h.forbidden(w, r)
		return
	}
	employee, err := h.Repository.GetEmployee(r.Context(), workspaceID, employeeID)
	if err != nil {
		h.fail(w, r, err)
		return
	}
	self := employee.UserID != nil && *employee.UserID == userID
	if !self && !h.allowed(r, workspaceID, PermissionOvertimeClaimOnBehalf) && !h.allowed(r, workspaceID, PermissionOvertimeApprove) {
		h.forbidden(w, r)
		return
	}
	req, err := DecodeJSON[ClaimOTRequest](r, 1<<20)
	if err != nil || !validOvertimeDate(req.Date) || req.ClaimedMinutes <= 0 {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	if err := h.Repository.ClaimOvertime(r.Context(), workspaceID, employeeID, req, userID); err != nil {
		h.fail(w, r, err)
		return
	}
	w.WriteHeader(http.StatusOK)
}

func (h OvertimeHandler) ApproveOT(w http.ResponseWriter, r *http.Request) {
	workspaceID, ok := pathUUID(r, "ws_id")
	employeeID, employeeOK := pathUUID(r, "employee_id")
	userID, userOK := h.userID(r)
	if !ok || !employeeOK || !userOK {
		h.forbidden(w, r)
		return
	}
	req, err := DecodeJSON[ApproveOTRequest](r, 1<<20)
	if err != nil || !validOvertimeDate(req.Date) || (req.Status != "approved" && req.Status != "rejected") || req.ApprovedMinutes < 0 {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	departmentID, err := h.Repository.EmployeeDepartmentID(r.Context(), workspaceID, employeeID)
	if err != nil {
		h.fail(w, r, err)
		return
	}
	if !h.scopedAllowed(r, workspaceID, PermissionOvertimeApprove, departmentID) {
		h.forbidden(w, r)
		return
	}
	if err := h.Repository.ApproveOvertime(r.Context(), workspaceID, employeeID, req, userID); err != nil {
		h.fail(w, r, err)
		return
	}
	w.WriteHeader(http.StatusOK)
}

func (h OvertimeHandler) ListClaimBehalf(w http.ResponseWriter, r *http.Request) {
	workspaceID, ok := pathUUID(r, "ws_id")
	if !ok {
		h.forbidden(w, r)
		return
	}
	items, err := h.BehalfRepository.ListBehalfAssignments(r.Context(), workspaceID, OvertimeClaimBehalfAssignmentType)
	if err != nil {
		h.fail(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusOK, items)
}

func (h OvertimeHandler) CreateClaimBehalf(w http.ResponseWriter, r *http.Request) {
	workspaceID, ok := pathUUID(r, "ws_id")
	if !ok || !h.allowed(r, workspaceID, PermissionOvertimePoliciesManage) {
		h.forbidden(w, r)
		return
	}
	req, err := DecodeJSON[CreateBehalfAssignmentRequest](r, 1<<20)
	if err != nil || req.AssigneeUserID == uuid.Nil || (req.TargetEmployeeID == nil) == (req.TargetDepartmentID == nil) {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	result, err := h.BehalfRepository.CreateBehalfAssignment(r.Context(), workspaceID, req, OvertimeClaimBehalfAssignmentType)
	if err != nil {
		h.fail(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusOK, result)
}

func (h OvertimeHandler) DeleteClaimBehalf(w http.ResponseWriter, r *http.Request) {
	workspaceID, ok := pathUUID(r, "ws_id")
	assignmentID, assignmentOK := pathUUID(r, "id")
	if !ok || !assignmentOK || !h.allowed(r, workspaceID, PermissionOvertimePoliciesManage) {
		h.forbidden(w, r)
		return
	}
	if err := h.BehalfRepository.DeleteBehalfAssignment(r.Context(), workspaceID, assignmentID); err != nil {
		h.fail(w, r, err)
		return
	}
	w.WriteHeader(http.StatusOK)
}
