package api

import (
	"context"
	"errors"
	"net/http"

	"github.com/google/uuid"
	"github.com/jackc/pgx/v5"
	"github.com/vkaylee/leeattend/apiserver/internal/apperror"
	httpinfra "github.com/vkaylee/leeattend/apiserver/internal/infra/http"
)

type EmployeeDepartmentHistory struct {
	ID            uuid.UUID `json:"id"`
	EmployeeID    uuid.UUID `json:"employee_id"`
	DepartmentID  uuid.UUID `json:"department_id"`
	EffectiveDate string    `json:"effective_date"`
	Notes         *string   `json:"notes"`
	CreatedAt     string    `json:"created_at"`
}

type TransferEmployeeRequest struct {
	DepartmentID  uuid.UUID `json:"department_id"`
	EffectiveDate string    `json:"effective_date"`
	Notes         *string   `json:"notes"`
}

type EmployeeDepartmentRepository interface {
	GetEmployeeDepartmentHistory(context.Context, uuid.UUID) ([]EmployeeDepartmentHistory, error)
	TransferEmployee(context.Context, uuid.UUID, TransferEmployeeRequest) (EmployeeDepartmentHistory, error)
	CurrentEmployeeDepartment(context.Context, uuid.UUID) (*uuid.UUID, error)
}

type EmployeeDepartmentHandler struct {
	Repository EmployeeDepartmentRepository
	Authorizer Authorizer
}

func (h *EmployeeDepartmentHandler) List(w http.ResponseWriter, r *http.Request) {
	employeeID, ok := employeePath(r)
	if !ok {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	if _, ok := httpinfra.UserIDFromContext(r.Context()); !ok {
		apperror.WriteError(w, r, apperror.Unauthorized(apperror.KeyAuthenticationRequired))
		return
	}
	history, err := h.Repository.GetEmployeeDepartmentHistory(r.Context(), employeeID)
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusOK, history)
}

func (h *EmployeeDepartmentHandler) Transfer(w http.ResponseWriter, r *http.Request) {
	employeeID, ok := employeePath(r)
	if !ok {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	userID, ok := httpinfra.UserIDFromContext(r.Context())
	if !ok {
		apperror.WriteError(w, r, apperror.Unauthorized(apperror.KeyAuthenticationRequired))
		return
	}
	targetDept, err := h.Repository.CurrentEmployeeDepartment(r.Context(), employeeID)
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	if h.Authorizer == nil || !h.Authorizer.HasScopedPermission(r.Context(), userID, "departments:transfer", targetDept) {
		apperror.WriteError(w, r, apperror.Forbidden(apperror.KeyForbidden))
		return
	}
	req, err := DecodeJSON[TransferEmployeeRequest](r, 1<<20)
	if err != nil || req.DepartmentID == uuid.Nil || !validDate(req.EffectiveDate) {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	history, err := h.Repository.TransferEmployee(r.Context(), employeeID, req)
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusOK, history)
}

func (h *EmployeeDepartmentHandler) writeError(w http.ResponseWriter, r *http.Request, err error) {
	switch {
	case errors.Is(err, ErrEmployeeNotFound):
		apperror.WriteError(w, r, apperror.NotFound(apperror.KeyEmployeeNotFound))
	case errors.Is(err, ErrDepartmentNotFound), errors.Is(err, pgx.ErrNoRows), errors.Is(err, ErrEmploymentNotFound):
		apperror.WriteError(w, r, apperror.NotFound(apperror.KeyNotFound))
	default:
		apperror.WriteError(w, r, apperror.Internal(err))
	}
}
