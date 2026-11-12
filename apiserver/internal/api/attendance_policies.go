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
	"github.com/vkaylee/leeattend/apiserver/internal/auth"
	httpinfra "github.com/vkaylee/leeattend/apiserver/internal/infra/http"
)

const (
	AttendanceAdjustmentAssignmentType = "attendance_adjustment"
	AttendanceTargetUser               = "USER"
	AttendanceTargetGroup              = "GROUP"
	AttendanceRoundingFloor            = "floor"
	AttendanceRoundingRound            = "round"
	AttendanceRoundingCeil             = "ceil"
	AttendanceRoundingNone             = "none"
	EarlyArrivalAllow                  = "allow"
	EarlyArrivalBlock                  = "block"
	EarlyArrivalClamp                  = "clamp"
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
	return AttendanceRules{WorkUnitMinutes: 30, RoundingPolicy: AttendanceRoundingFloor, EarlyArrivalStrategy: EarlyArrivalBlock}
}

// UpdateAttendanceRulesRequest partially updates organization default rules.
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
	ID             string          `json:"id"`
	TargetType     string          `json:"target_type"`
	TargetID       string          `json:"target_id"`
	TargetName     *string         `json:"target_name,omitempty"`
	TargetCode     *string         `json:"target_code,omitempty"`
	DepartmentName *string         `json:"department_name,omitempty"`
	ValidFrom      string          `json:"valid_from"`
	ValidTo        *string         `json:"valid_to"`
	Rules          AttendanceRules `json:"rules"`
}

type PolicyAssignmentListResponse struct {
	Data  []PolicyAssignmentResponse `json:"data"`
	Total int64                      `json:"total"`
	Page  int                        `json:"page"`
	Limit int                        `json:"limit"`
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
	GetDefaultPolicyRules(context.Context) (AttendanceRules, error)
	GetDefaultPolicyVersions(context.Context) ([]AttendancePolicyVersionResponse, error)
	UpdateDefaultPolicyRules(context.Context, uuid.UUID, UpdateAttendanceRulesRequest) (AttendancePolicyVersionResponse, error)
	ListPolicyAssignments(context.Context, int, int, string, string) (PolicyAssignmentListResponse, error)
	UpsertPolicyAssignment(context.Context, uuid.UUID, UpsertPolicyAssignmentRequest) (PolicyAssignmentResponse, error)
	DeletePolicyAssignment(context.Context, uuid.UUID) error
}

type CreateBehalfAssignmentRequest struct {
	AssigneeUserID     uuid.UUID  `json:"assignee_user_id"`
	TargetEmployeeID   *uuid.UUID `json:"target_employee_id,omitempty"`
	TargetDepartmentID *uuid.UUID `json:"target_department_id,omitempty"`
	EmployerID         *uuid.UUID `json:"employer_id,omitempty"`
}

type BehalfAssignment struct {
	ID                     uuid.UUID  `json:"id"`
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
	CreateBehalfAssignment(context.Context, CreateBehalfAssignmentRequest, string) (BehalfAssignment, error)
	ListBehalfAssignments(context.Context, string) ([]BehalfAssignment, error)
	DeleteBehalfAssignment(context.Context, uuid.UUID, string) error
	ListMyBehalfTargets(context.Context, uuid.UUID, string) ([]EmployeeRecord, error)
}

type AttendancePolicyHandler struct {
	PolicyRepository AttendancePolicyRepository
	BehalfRepository BehalfAssignmentRepository
	Authorizer       Authorizer
}

func (h AttendancePolicyHandler) allowed(r *http.Request, permission string) bool {
	userID, ok := httpinfra.UserIDFromContext(r.Context())
	return ok && h.Authorizer != nil && h.Authorizer.HasPermission(r.Context(), userID, permission)
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
	if workUnit <= 0 || (rounding != AttendanceRoundingFloor && rounding != AttendanceRoundingRound && rounding != AttendanceRoundingCeil && rounding != AttendanceRoundingNone) {
		return false
	}
	return early == EarlyArrivalAllow || early == EarlyArrivalBlock || early == EarlyArrivalClamp
}

func validPolicyTarget(target string) bool {
	return target == AttendanceTargetUser || target == AttendanceTargetGroup
}

func (h AttendancePolicyHandler) GetRules(w http.ResponseWriter, r *http.Request) {
	if !h.allowed(r, string(auth.PermissionOrganizationSettingsRead)) {
		h.forbidden(w, r)
		return
	}
	rules, err := h.PolicyRepository.GetDefaultPolicyRules(r.Context())
	if err != nil {
		h.fail(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusOK, rules)
}

func (h AttendancePolicyHandler) UpdateRules(w http.ResponseWriter, r *http.Request) {
	userID, userOK := httpinfra.UserIDFromContext(r.Context())
	if !userOK || !h.allowed(r, string(auth.PermissionOrganizationSettingsWrite)) {
		h.forbidden(w, r)
		return
	}
	req, err := DecodeJSON[UpdateAttendanceRulesRequest](r, 1<<20)
	if err != nil {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	if req.WorkUnitMinutes != nil && *req.WorkUnitMinutes <= 0 || req.RoundingPolicy != nil && !validRules(30, *req.RoundingPolicy, EarlyArrivalBlock) || req.EarlyArrivalStrategy != nil && !validRules(30, AttendanceRoundingFloor, *req.EarlyArrivalStrategy) || req.GracePeriodMinutes != nil && *req.GracePeriodMinutes < 0 {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	if req.ValidFrom != nil {
		if _, err := parsePolicyValidFrom(req.ValidFrom); err != nil {
			apperror.WriteError(w, r, apperror.Validation(apperror.KeyInvalidDateFormat))
			return
		}
	}
	result, err := h.PolicyRepository.UpdateDefaultPolicyRules(r.Context(), userID, req)
	if err != nil {
		h.fail(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusOK, result)
}

func (h AttendancePolicyHandler) ListRuleVersions(w http.ResponseWriter, r *http.Request) {
	if !h.allowed(r, string(auth.PermissionOrganizationSettingsRead)) {
		h.forbidden(w, r)
		return
	}
	result, err := h.PolicyRepository.GetDefaultPolicyVersions(r.Context())
	if err != nil {
		h.fail(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusOK, result)
}

func (h AttendancePolicyHandler) ListAssignments(w http.ResponseWriter, r *http.Request) {
	if !h.allowed(r, string(auth.PermissionAttendancePolicyAssignmentsRead)) {
		h.forbidden(w, r)
		return
	}
	pagination, err := ParsePagination(r.URL.Query())
	if err != nil {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	query := strings.TrimSpace(r.URL.Query().Get("q"))
	targetType := strings.TrimSpace(r.URL.Query().Get("target_type"))
	if targetType != "" && !validPolicyTarget(targetType) {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	result, err := h.PolicyRepository.ListPolicyAssignments(r.Context(), pagination.Page, pagination.Limit, query, targetType)
	if err != nil {
		h.fail(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusOK, result)
}

func (h AttendancePolicyHandler) UpsertAssignment(w http.ResponseWriter, r *http.Request) {
	userID, userOK := httpinfra.UserIDFromContext(r.Context())
	if !userOK || !h.allowed(r, string(auth.PermissionAttendancePolicyAssignmentsManage)) {
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
	result, err := h.PolicyRepository.UpsertPolicyAssignment(r.Context(), userID, req)
	if err != nil {
		h.fail(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusOK, result)
}

func (h AttendancePolicyHandler) DeleteAssignment(w http.ResponseWriter, r *http.Request) {
	assignmentID, assignmentOK := pathUUID(r, "assignment_id")
	if !assignmentOK || !h.allowed(r, string(auth.PermissionAttendancePolicyAssignmentsManage)) {
		h.forbidden(w, r)
		return
	}
	if err := h.PolicyRepository.DeletePolicyAssignment(r.Context(), assignmentID); err != nil {
		h.fail(w, r, err)
		return
	}
	w.WriteHeader(http.StatusNoContent)
}

func (h AttendancePolicyHandler) ListBehalfAssignments(w http.ResponseWriter, r *http.Request) {
	if !h.allowed(r, string(auth.PermissionAttendanceAdjustmentsManageBehalf)) {
		h.forbidden(w, r)
		return
	}
	result, err := h.BehalfRepository.ListBehalfAssignments(r.Context(), AttendanceAdjustmentAssignmentType)
	if err != nil {
		h.fail(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusOK, result)
}

func (h AttendancePolicyHandler) CreateBehalfAssignment(w http.ResponseWriter, r *http.Request) {
	if !h.allowed(r, string(auth.PermissionAttendanceAdjustmentsManageBehalf)) {
		h.forbidden(w, r)
		return
	}
	req, err := DecodeJSON[CreateBehalfAssignmentRequest](r, 1<<20)
	if err != nil || req.AssigneeUserID == uuid.Nil || (req.TargetEmployeeID == nil) == (req.TargetDepartmentID == nil) {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	result, err := h.BehalfRepository.CreateBehalfAssignment(r.Context(), req, AttendanceAdjustmentAssignmentType)
	if err != nil {
		h.fail(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusOK, result)
}

func (h AttendancePolicyHandler) DeleteBehalfAssignment(w http.ResponseWriter, r *http.Request) {
	assignmentID, assignmentOK := pathUUID(r, "id")
	if !assignmentOK || !h.allowed(r, string(auth.PermissionAttendanceAdjustmentsManageBehalf)) {
		h.forbidden(w, r)
		return
	}
	if err := h.BehalfRepository.DeleteBehalfAssignment(r.Context(), assignmentID, AttendanceAdjustmentAssignmentType); err != nil {
		h.fail(w, r, err)
		return
	}
	w.WriteHeader(http.StatusOK)
}

func (h AttendancePolicyHandler) MyBehalfTargets(w http.ResponseWriter, r *http.Request) {
	userID, userOK := httpinfra.UserIDFromContext(r.Context())
	if !userOK {
		apperror.WriteError(w, r, apperror.Unauthorized(apperror.KeyAuthenticationRequired))
		return
	}
	result, err := h.BehalfRepository.ListMyBehalfTargets(r.Context(), userID, AttendanceAdjustmentAssignmentType)
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
