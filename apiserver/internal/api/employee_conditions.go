package api

import (
	"context"
	"errors"
	"net/http"
	"time"

	"github.com/google/uuid"
	"github.com/jackc/pgx/v5"
	"github.com/vkaylee/leeattend/apiserver/internal/apperror"
	httpinfra "github.com/vkaylee/leeattend/apiserver/internal/infra/http"
)

var (
	ErrEmployeeConditionNotFound = errors.New("employee condition: not found")
	ErrEmployeeNotFound          = errors.New("employee: not found")
)

type EmployeeCondition struct {
	ID              uuid.UUID  `json:"id"`
	EmploymentID    uuid.UUID  `json:"employment_id"`
	Type            string     `json:"type_"`
	ConditionTypeID *uuid.UUID `json:"condition_type_id"`
	MinuteAllowance int        `json:"minute_allowance"`
	IsPaid          bool       `json:"is_paid"`
	StartDate       string     `json:"start_date"`
	EndDate         *string    `json:"end_date"`
}

type EmployeeConditionWithEmployee struct {
	EmployeeCondition
	EmployeeID   uuid.UUID `json:"employee_id"`
	EmployeeName string    `json:"employee_name"`
	EmployeeCode string    `json:"employee_code"`
}

type CreateEmployeeConditionRequest struct {
	Type            string     `json:"type_"`
	ConditionTypeID *uuid.UUID `json:"condition_type_id"`
	MinuteAllowance int        `json:"minute_allowance"`
	IsPaid          bool       `json:"is_paid"`
	StartDate       string     `json:"start_date"`
	EndDate         *string    `json:"end_date"`
}

type EmployeeConditionListResponse struct {
	Data      []EmployeeCondition `json:"data"`
	Total     int64               `json:"total"`
	Page      int                 `json:"page"`
	Limit     int                 `json:"limit"`
	Truncated *bool               `json:"truncated"`
}

type EmployeeConditionRepository interface {
	ListWorkspaceConditions(context.Context, uuid.UUID) ([]EmployeeConditionWithEmployee, error)
	ListEmployeeConditions(context.Context, uuid.UUID, uuid.UUID, int, int) (EmployeeConditionListResponse, error)
	CreateEmployeeCondition(context.Context, uuid.UUID, uuid.UUID, CreateEmployeeConditionRequest) (EmployeeCondition, error)
	DeleteEmployeeCondition(context.Context, uuid.UUID, uuid.UUID, uuid.UUID) error
	CurrentEmployeeDepartment(context.Context, uuid.UUID, uuid.UUID) (*uuid.UUID, error)
}

// ScopedWorkspaceAuthorizer is optional for compatibility with existing test and
// plugin authorizers. Production WorkspaceAuthorizer implements department scope.
type ScopedWorkspaceAuthorizer interface {
	HasScopedPermission(context.Context, uuid.UUID, uuid.UUID, string, *uuid.UUID) bool
}

type EmployeeConditionHandler struct {
	Repository EmployeeConditionRepository
	Authorizer WorkspaceAuthorizer
}

func (h EmployeeConditionHandler) ListWorkspaceConditions(w http.ResponseWriter, r *http.Request) {
	wsID, ok := pathUUID(r, "ws_id")
	if !ok {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	userID, ok := httpinfra.UserIDFromContext(r.Context())
	if !ok {
		apperror.WriteError(w, r, apperror.Unauthorized(apperror.KeyAuthenticationRequired))
		return
	}
	if h.Authorizer != nil && !h.Authorizer.HasPermission(r.Context(), userID, wsID, "employees:read") {
		h.forbidden(w, r)
		return
	}
	items, err := h.Repository.ListWorkspaceConditions(r.Context(), wsID)
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusOK, items)
}

func (h EmployeeConditionHandler) List(w http.ResponseWriter, r *http.Request) {
	wsID, employeeID, ok := employeePath(r)
	if !ok {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	if _, ok := httpinfra.UserIDFromContext(r.Context()); !ok {
		apperror.WriteError(w, r, apperror.Unauthorized(apperror.KeyAuthenticationRequired))
		return
	}
	p, err := ParsePagination(r.URL.Query())
	if err != nil {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	result, err := h.Repository.ListEmployeeConditions(r.Context(), wsID, employeeID, p.Page, p.Limit)
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusOK, result)
}

func (h EmployeeConditionHandler) Create(w http.ResponseWriter, r *http.Request) {
	wsID, employeeID, ok := employeePath(r)
	if !ok {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	userID, ok := httpinfra.UserIDFromContext(r.Context())
	if !ok {
		apperror.WriteError(w, r, apperror.Unauthorized(apperror.KeyAuthenticationRequired))
		return
	}
	targetDept, err := h.Repository.CurrentEmployeeDepartment(r.Context(), wsID, employeeID)
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	if !h.allowedScoped(r.Context(), userID, wsID, "employees:update", targetDept) {
		h.forbidden(w, r)
		return
	}
	req, err := DecodeJSON[CreateEmployeeConditionRequest](r, 1<<20)
	if err != nil || req.StartDate == "" || req.Type == "" || !validConditionDate(req.StartDate) || (req.EndDate != nil && !validConditionDate(*req.EndDate)) {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	if req.EndDate != nil && *req.EndDate < req.StartDate {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	condition, err := h.Repository.CreateEmployeeCondition(r.Context(), wsID, employeeID, req)
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusCreated, condition)
}

func (h EmployeeConditionHandler) Delete(w http.ResponseWriter, r *http.Request) {
	wsID, employeeID, ok := employeePath(r)
	conditionID, conditionOK := pathUUID(r, "cond_id")
	if !ok || !conditionOK {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	userID, ok := httpinfra.UserIDFromContext(r.Context())
	if !ok {
		apperror.WriteError(w, r, apperror.Unauthorized(apperror.KeyAuthenticationRequired))
		return
	}
	targetDept, err := h.Repository.CurrentEmployeeDepartment(r.Context(), wsID, employeeID)
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	if !h.allowedScoped(r.Context(), userID, wsID, "employees:update", targetDept) {
		h.forbidden(w, r)
		return
	}
	if err := h.Repository.DeleteEmployeeCondition(r.Context(), wsID, employeeID, conditionID); err != nil {
		h.writeError(w, r, err)
		return
	}
	w.WriteHeader(http.StatusNoContent)
}

func employeePath(r *http.Request) (uuid.UUID, uuid.UUID, bool) {
	wsID, wsOK := pathUUID(r, "ws_id")
	employeeID, employeeOK := pathUUID(r, "id")
	return wsID, employeeID, wsOK && employeeOK
}

func validConditionDate(value string) bool {
	_, err := time.Parse("2006-01-02", value)
	return err == nil
}

func (h EmployeeConditionHandler) allowedScoped(ctx context.Context, userID, wsID uuid.UUID, permission string, dept *uuid.UUID) bool {
	if h.Authorizer == nil {
		return false
	}
	if scoped, ok := h.Authorizer.(ScopedWorkspaceAuthorizer); ok {
		return scoped.HasScopedPermission(ctx, userID, wsID, permission, dept)
	}
	return h.Authorizer.HasPermission(ctx, userID, wsID, permission)
}

func (h EmployeeConditionHandler) forbidden(w http.ResponseWriter, r *http.Request) {
	apperror.WriteError(w, r, apperror.Forbidden(apperror.KeyForbidden))
}

func (h EmployeeConditionHandler) writeError(w http.ResponseWriter, r *http.Request, err error) {
	switch {
	case errors.Is(err, ErrEmployeeNotFound):
		apperror.WriteError(w, r, apperror.NotFound(apperror.KeyEmployeeNotFound))
	case errors.Is(err, ErrEmployeeConditionNotFound), errors.Is(err, ErrEmploymentNotFound), errors.Is(err, pgx.ErrNoRows):
		apperror.WriteError(w, r, apperror.NotFound(apperror.KeyNotFound))
	case errors.Is(err, ErrNoActiveEmployment):
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
	default:
		apperror.WriteError(w, r, apperror.Internal(err))
	}
}
