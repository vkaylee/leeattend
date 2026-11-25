package api

import (
	"context"
	"errors"
	"net/http"

	"github.com/google/uuid"
	"github.com/jackc/pgx/v5"
	"github.com/vkaylee/leeattend/apiserver/internal/apperror"
	"github.com/vkaylee/leeattend/apiserver/internal/auth"
	httpinfra "github.com/vkaylee/leeattend/apiserver/internal/infra/http"
)

const AttendanceAdjustmentAssignmentType = "attendance_adjustment"

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
	BehalfRepository BehalfAssignmentRepository
	Authorizer       Authorizer
}

func (h *AttendancePolicyHandler) allowed(r *http.Request, permission string) bool {
	userID, ok := httpinfra.UserIDFromContext(r.Context())
	return ok && h.Authorizer != nil && h.Authorizer.HasPermission(r.Context(), userID, permission)
}

func (h *AttendancePolicyHandler) forbidden(w http.ResponseWriter, r *http.Request) {
	apperror.WriteError(w, r, apperror.Forbidden(apperror.KeyForbidden))
}

func (h *AttendancePolicyHandler) fail(w http.ResponseWriter, r *http.Request, err error) {
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

func (h *AttendancePolicyHandler) ListBehalfAssignments(w http.ResponseWriter, r *http.Request) {
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

func (h *AttendancePolicyHandler) CreateBehalfAssignment(w http.ResponseWriter, r *http.Request) {
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

func (h *AttendancePolicyHandler) DeleteBehalfAssignment(w http.ResponseWriter, r *http.Request) {
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

func (h *AttendancePolicyHandler) MyBehalfTargets(w http.ResponseWriter, r *http.Request) {
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
