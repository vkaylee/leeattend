package api

import (
	"context"
	"errors"
	"net/http"
	"strings"
	"time"

	"github.com/google/uuid"
	"github.com/jackc/pgx/v5"
	"github.com/vkaylee/leeattend/apiserver/internal/apperror"
	httpinfra "github.com/vkaylee/leeattend/apiserver/internal/infra/http"
)

const (
	PermissionWorkspaceSettingsRead    = "workspace:settings:read"
	PermissionWorkspaceSettingsWrite   = "workspace:settings:write"
	PermissionAttendanceManageBehalf   = "attendance:adjustments:manage_behalf"
	AttendanceAdjustmentAssignmentType = "attendance_adjustment"
	AttendanceTargetUser               = "USER"
	AttendanceTargetGroup              = "GROUP"
	DefaultGracePeriodMinutes          = 15
)

// AttendanceRules is the typed attendance policy rules payload.
type AttendanceRules struct {
	WorkUnitMinutes      int    `json:"work_unit_minutes"`
	RoundingPolicy       string `json:"rounding_policy"`
	EarlyArrivalStrategy string `json:"early_arrival_strategy"`
	GracePeriodMinutes   *int   `json:"grace_period_minutes"`
}

func DefaultAttendanceRules() AttendanceRules {
	return AttendanceRules{WorkUnitMinutes: 30, RoundingPolicy: "floor", EarlyArrivalStrategy: "block"}
}

// UpdateAttendanceRulesRequest partially updates workspace default rules.
type UpdateAttendanceRulesRequest struct {
	WorkUnitMinutes      *int    `json:"work_unit_minutes"`
	RoundingPolicy       *string `json:"rounding_policy"`
	ValidFrom            *string `json:"valid_from"`
	EarlyArrivalStrategy *string `json:"early_arrival_strategy"`
	GracePeriodMinutes   *int    `json:"grace_period_minutes"`
}

type AttendancePolicyVersionResponse struct {
	ID                   string  `json:"id"`
	WorkUnitMinutes      int     `json:"work_unit_minutes"`
	RoundingPolicy       string  `json:"rounding_policy"`
	EarlyArrivalStrategy string  `json:"early_arrival_strategy"`
	GracePeriodMinutes   int     `json:"grace_period_minutes"`
	ValidFrom            string  `json:"valid_from"`
	ValidTo              *string `json:"valid_to"`
	CreatedBy            *string `json:"created_by"`
	CreatedAt            string  `json:"created_at"`
}

type PolicyAssignmentResponse struct {
	ID         string          `json:"id"`
	TargetType string          `json:"target_type"`
	TargetID   string          `json:"target_id"`
	ValidFrom  string          `json:"valid_from"`
	ValidTo    *string         `json:"valid_to"`
	Rules      AttendanceRules `json:"rules"`
}

type UpsertPolicyAssignmentRequest struct {
	TargetType           string    `json:"target_type"`
	TargetID             uuid.UUID `json:"target_id"`
	WorkUnitMinutes      int       `json:"work_unit_minutes"`
	RoundingPolicy       string    `json:"rounding_policy"`
	EarlyArrivalStrategy string    `json:"early_arrival_strategy"`
	GracePeriodMinutes   *int      `json:"grace_period_minutes,omitempty"`
	ValidFrom            *string   `json:"valid_from,omitempty"`
}

// AttendancePolicyRepository persists policy rules and versioned assignments.
type AttendancePolicyRepository interface {
	GetDefaultPolicyRules(context.Context, uuid.UUID) (AttendanceRules, error)
	GetDefaultPolicyVersions(context.Context, uuid.UUID) ([]AttendancePolicyVersionResponse, error)
	UpdateDefaultPolicyRules(context.Context, uuid.UUID, uuid.UUID, UpdateAttendanceRulesRequest) (AttendancePolicyVersionResponse, error)
	ListPolicyAssignments(context.Context, uuid.UUID) ([]PolicyAssignmentResponse, error)
	UpsertPolicyAssignment(context.Context, uuid.UUID, uuid.UUID, UpsertPolicyAssignmentRequest) (PolicyAssignmentResponse, error)
	DeletePolicyAssignment(context.Context, uuid.UUID, uuid.UUID) error
}

type CreateBehalfAssignmentRequest struct {
	AssigneeUserID     uuid.UUID  `json:"assignee_user_id"`
	TargetEmployeeID   *uuid.UUID `json:"target_employee_id,omitempty"`
	TargetDepartmentID *uuid.UUID `json:"target_department_id,omitempty"`
	EmployerID         *uuid.UUID `json:"employer_id,omitempty"`
}

type BehalfAssignment struct {
	ID                     uuid.UUID  `json:"id"`
	WorkspaceID            uuid.UUID  `json:"workspace_id"`
	AssigneeUserID         uuid.UUID  `json:"assignee_user_id"`
	TargetEmployeeID       *uuid.UUID `json:"target_employee_id,omitempty"`
	TargetDepartmentID     *uuid.UUID `json:"target_department_id,omitempty"`
	AssignmentType         string     `json:"assignment_type"`
	CreatedAt              string     `json:"created_at"`
	UpdatedAt              string     `json:"updated_at"`
	EmployerID             *uuid.UUID `json:"employer_id,omitempty"`
	TargetEmployeeName     *string    `json:"target_employee_name,omitempty"`
	TargetEmployeeIDNumber *string    `json:"target_employee_id_number,omitempty"`
	TargetDepartmentName   *string    `json:"target_department_name,omitempty"`
}

type BehalfAssignmentRepository interface {
	CreateBehalfAssignment(context.Context, uuid.UUID, CreateBehalfAssignmentRequest, string) (BehalfAssignment, error)
	ListBehalfAssignments(context.Context, uuid.UUID, string) ([]BehalfAssignment, error)
	DeleteBehalfAssignment(context.Context, uuid.UUID, uuid.UUID) error
	ListMyBehalfTargets(context.Context, uuid.UUID, uuid.UUID, string) ([]EmployeeRecord, error)
}

type AttendancePolicyHandler struct {
	PolicyRepository AttendancePolicyRepository
	BehalfRepository BehalfAssignmentRepository
	Authorizer       AttendanceAuthorizer
}

func (h AttendancePolicyHandler) allowed(r *http.Request, workspaceID uuid.UUID, permission string) bool {
	userID, ok := httpinfra.UserIDFromContext(r.Context())
	return ok && h.Authorizer != nil && h.Authorizer.HasPermission(r.Context(), userID, workspaceID, permission)
}

func (h AttendancePolicyHandler) forbidden(w http.ResponseWriter, r *http.Request) {
	apperror.WriteError(w, r, apperror.Forbidden(apperror.KeyForbidden))
}

func (h AttendancePolicyHandler) fail(w http.ResponseWriter, r *http.Request, err error) {
	if errors.Is(err, pgx.ErrNoRows) {
		apperror.WriteError(w, r, apperror.NotFound(apperror.KeyNotFound))
		return
	}
	var appErr *apperror.AppError
	if errors.As(err, &appErr) {
		apperror.WriteError(w, r, appErr)
		return
	}
	apperror.WriteError(w, r, apperror.Internal(err))
}

func validRules(workUnit int, rounding, early string) bool {
	if workUnit <= 0 || (rounding != "floor" && rounding != "round" && rounding != "ceil" && rounding != "none") {
		return false
	}
	return early == "allow" || early == "block" || early == "clamp"
}

func validPolicyTarget(target string) bool {
	return target == AttendanceTargetUser || target == AttendanceTargetGroup
}

func (h AttendancePolicyHandler) GetRules(w http.ResponseWriter, r *http.Request) {
	workspaceID, ok := pathUUID(r, "ws_id")
	if !ok || !h.allowed(r, workspaceID, PermissionWorkspaceSettingsRead) {
		h.forbidden(w, r)
		return
	}
	rules, err := h.PolicyRepository.GetDefaultPolicyRules(r.Context(), workspaceID)
	if err != nil {
		h.fail(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusOK, rules)
}

func (h AttendancePolicyHandler) UpdateRules(w http.ResponseWriter, r *http.Request) {
	workspaceID, ok := pathUUID(r, "ws_id")
	userID, userOK := httpinfra.UserIDFromContext(r.Context())
	if !ok || !userOK || !h.allowed(r, workspaceID, PermissionWorkspaceSettingsWrite) {
		h.forbidden(w, r)
		return
	}
	req, err := DecodeJSON[UpdateAttendanceRulesRequest](r, 1<<20)
	if err != nil {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	if req.WorkUnitMinutes != nil && *req.WorkUnitMinutes <= 0 || req.RoundingPolicy != nil && !validRules(30, *req.RoundingPolicy, "block") || req.EarlyArrivalStrategy != nil && !validRules(30, "floor", *req.EarlyArrivalStrategy) || req.GracePeriodMinutes != nil && *req.GracePeriodMinutes < 0 {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	if req.ValidFrom != nil {
		if _, err := parsePolicyValidFrom(req.ValidFrom); err != nil {
			apperror.WriteError(w, r, apperror.Validation(apperror.KeyInvalidDateFormat))
			return
		}
	}
	result, err := h.PolicyRepository.UpdateDefaultPolicyRules(r.Context(), workspaceID, userID, req)
	if err != nil {
		h.fail(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusOK, result)
}

func (h AttendancePolicyHandler) ListRuleVersions(w http.ResponseWriter, r *http.Request) {
	workspaceID, ok := pathUUID(r, "ws_id")
	if !ok || !h.allowed(r, workspaceID, PermissionWorkspaceSettingsRead) {
		h.forbidden(w, r)
		return
	}
	result, err := h.PolicyRepository.GetDefaultPolicyVersions(r.Context(), workspaceID)
	if err != nil {
		h.fail(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusOK, result)
}

func (h AttendancePolicyHandler) ListAssignments(w http.ResponseWriter, r *http.Request) {
	workspaceID, ok := pathUUID(r, "ws_id")
	if !ok || !h.allowed(r, workspaceID, PermissionWorkspaceSettingsRead) {
		h.forbidden(w, r)
		return
	}
	result, err := h.PolicyRepository.ListPolicyAssignments(r.Context(), workspaceID)
	if err != nil {
		h.fail(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusOK, result)
}

func (h AttendancePolicyHandler) UpsertAssignment(w http.ResponseWriter, r *http.Request) {
	workspaceID, ok := pathUUID(r, "ws_id")
	userID, userOK := httpinfra.UserIDFromContext(r.Context())
	if !ok || !userOK || !h.allowed(r, workspaceID, PermissionWorkspaceSettingsWrite) {
		h.forbidden(w, r)
		return
	}
	req, err := DecodeJSON[UpsertPolicyAssignmentRequest](r, 1<<20)
	if err != nil || req.TargetID == uuid.Nil || !validPolicyTarget(req.TargetType) || !validRules(req.WorkUnitMinutes, req.RoundingPolicy, req.EarlyArrivalStrategy) || req.GracePeriodMinutes != nil && *req.GracePeriodMinutes < 0 {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	if req.ValidFrom != nil {
		if _, err := parsePolicyValidFrom(req.ValidFrom); err != nil {
			apperror.WriteError(w, r, apperror.Validation(apperror.KeyInvalidDateFormat))
			return
		}
	}
	result, err := h.PolicyRepository.UpsertPolicyAssignment(r.Context(), workspaceID, userID, req)
	if err != nil {
		h.fail(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusOK, result)
}

func (h AttendancePolicyHandler) DeleteAssignment(w http.ResponseWriter, r *http.Request) {
	workspaceID, ok := pathUUID(r, "ws_id")
	assignmentID, assignmentOK := pathUUID(r, "assignment_id")
	if !ok || !assignmentOK || !h.allowed(r, workspaceID, PermissionWorkspaceSettingsWrite) {
		h.forbidden(w, r)
		return
	}
	if err := h.PolicyRepository.DeletePolicyAssignment(r.Context(), workspaceID, assignmentID); err != nil {
		h.fail(w, r, err)
		return
	}
	w.WriteHeader(http.StatusNoContent)
}

func (h AttendancePolicyHandler) ListBehalfAssignments(w http.ResponseWriter, r *http.Request) {
	workspaceID, ok := pathUUID(r, "ws_id")
	if !ok || !h.allowed(r, workspaceID, PermissionAttendanceManageBehalf) {
		h.forbidden(w, r)
		return
	}
	result, err := h.BehalfRepository.ListBehalfAssignments(r.Context(), workspaceID, AttendanceAdjustmentAssignmentType)
	if err != nil {
		h.fail(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusOK, result)
}

func (h AttendancePolicyHandler) CreateBehalfAssignment(w http.ResponseWriter, r *http.Request) {
	workspaceID, ok := pathUUID(r, "ws_id")
	if !ok || !h.allowed(r, workspaceID, PermissionAttendanceManageBehalf) {
		h.forbidden(w, r)
		return
	}
	req, err := DecodeJSON[CreateBehalfAssignmentRequest](r, 1<<20)
	if err != nil || req.AssigneeUserID == uuid.Nil || (req.TargetEmployeeID == nil) == (req.TargetDepartmentID == nil) {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	result, err := h.BehalfRepository.CreateBehalfAssignment(r.Context(), workspaceID, req, AttendanceAdjustmentAssignmentType)
	if err != nil {
		h.fail(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusOK, result)
}

func (h AttendancePolicyHandler) DeleteBehalfAssignment(w http.ResponseWriter, r *http.Request) {
	workspaceID, ok := pathUUID(r, "ws_id")
	assignmentID, assignmentOK := pathUUID(r, "id")
	if !ok || !assignmentOK || !h.allowed(r, workspaceID, PermissionAttendanceManageBehalf) {
		h.forbidden(w, r)
		return
	}
	if err := h.BehalfRepository.DeleteBehalfAssignment(r.Context(), workspaceID, assignmentID); err != nil {
		h.fail(w, r, err)
		return
	}
	w.WriteHeader(http.StatusOK)
}

func (h AttendancePolicyHandler) MyBehalfTargets(w http.ResponseWriter, r *http.Request) {
	workspaceID, ok := pathUUID(r, "ws_id")
	userID, userOK := httpinfra.UserIDFromContext(r.Context())
	if !ok || !userOK {
		apperror.WriteError(w, r, apperror.Unauthorized(apperror.KeyAuthenticationRequired))
		return
	}
	result, err := h.BehalfRepository.ListMyBehalfTargets(r.Context(), workspaceID, userID, AttendanceAdjustmentAssignmentType)
	if err != nil {
		h.fail(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusOK, result)
}

func parsePolicyValidFrom(raw *string) (time.Time, error) {
	if raw == nil || strings.TrimSpace(*raw) == "" {
		return time.Now().UTC(), nil
	}
	if value, err := time.Parse(time.RFC3339, strings.TrimSpace(*raw)); err == nil {
		return value.UTC(), nil
	}
	value, err := time.Parse("2006-01-02", strings.TrimSpace(*raw))
	if err != nil {
		return time.Time{}, err
	}
	return value.UTC(), nil
}
