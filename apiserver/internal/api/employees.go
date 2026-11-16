package api

import (
	"context"
	"errors"
	"net/http"
	"strings"

	"github.com/google/uuid"
	"github.com/jackc/pgx/v5"
	"github.com/vkaylee/leeattend/apiserver/internal/apperror"
	httpinfra "github.com/vkaylee/leeattend/apiserver/internal/infra/http"
)

type EmployeeRecord struct {
	ID                 uuid.UUID  `json:"id"`
	UserID             *uuid.UUID `json:"user_id"`
	FullName           string     `json:"full_name"`
	EmployeeIDNumber   string     `json:"employee_id_number"`
	Tags               []string   `json:"tags"`
	CreatedAt          string     `json:"created_at"`
	UpdatedAt          string     `json:"updated_at"`
	DepartmentName     *string    `json:"department_name"`
	ActiveEmploymentID *uuid.UUID `json:"active_employment_id"`
	EmployerID         *uuid.UUID `json:"employer_id"`
	PrimaryLocationID  *uuid.UUID `json:"primary_location_id"`
}

type UpdateEmployeeRequest struct {
	UserID               **uuid.UUID `json:"user_id"`
	FullName             *string     `json:"full_name"`
	EmployeeIDNumber     *string     `json:"employee_id_number"`
	Tags                 *[]string   `json:"tags"`
	MaxConcurrentWorkers *int        `json:"max_concurrent_workers"`
	RecruitmentSourceID  *uuid.UUID  `json:"recruitment_source_id"`
	ForceTransfer        *bool       `json:"force_transfer"`
}

type CreateEmployeeRequest struct {
	FullName          string     `json:"full_name"`
	EmployeeIDNumber  string     `json:"employee_id_number"`
	HireDate          string     `json:"hire_date"`
	PrimaryLocationID *uuid.UUID `json:"primary_location_id"`
	EmployerID        *uuid.UUID `json:"employer_id"`
	DepartmentID      *uuid.UUID `json:"department_id"`
	Tags              []string   `json:"tags"`
}
type RehireEmployeeRequest struct {
	HireDate          string     `json:"hire_date"`
	EmployeeIDNumber  *string    `json:"employee_id_number,omitempty"`
	EmployerID        *uuid.UUID `json:"employer_id"`
	PrimaryLocationID *uuid.UUID `json:"primary_location_id"`
}
type EmployeeListResponse struct {
	Data      []EmployeeRecord `json:"data"`
	Total     int64            `json:"total"`
	Page      int              `json:"page"`
	Limit     int              `json:"limit"`
	Truncated *bool            `json:"truncated"`
}

type EmployeeRepository interface {
	CreateEmployee(context.Context, CreateEmployeeRequest) (EmployeeRecord, error)
	ListEmployees(context.Context, int, int, string) (EmployeeListResponse, error)
	GetEmployee(context.Context, uuid.UUID) (EmployeeRecord, error)
	UpdateEmployee(context.Context, uuid.UUID, UpdateEmployeeRequest) (EmployeeRecord, error)
	SoftDeleteEmployee(context.Context, uuid.UUID) error
	RehireEmployee(context.Context, uuid.UUID, RehireEmployeeRequest) (EmployeeRecord, error)
}

type EmployeeHandler struct {
	Repository EmployeeRepository
	Authorizer Authorizer
}

func (h *EmployeeHandler) Create(w http.ResponseWriter, r *http.Request) {
	if !h.allowed(r, "employees:create") {
		h.forbidden(w, r)
		return
	}
	request, err := DecodeJSON[CreateEmployeeRequest](r, 1<<20)
	if err != nil || strings.TrimSpace(request.FullName) == "" || strings.TrimSpace(request.EmployeeIDNumber) == "" || request.HireDate == "" {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	employee, err := h.Repository.CreateEmployee(r.Context(), request)
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusCreated, employee)
}

func (h *EmployeeHandler) List(w http.ResponseWriter, r *http.Request) {
	if !h.allowed(r, "employees:read") {
		h.forbidden(w, r)
		return
	}
	pagination, err := ParsePagination(r.URL.Query())
	if err != nil {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	result, err := h.Repository.ListEmployees(r.Context(), pagination.Page, pagination.Limit, r.URL.Query().Get("search"))
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusOK, result)
}

func (h *EmployeeHandler) Get(w http.ResponseWriter, r *http.Request) {
	employeeID, employeeOK := pathUUID(r, "id")
	if !employeeOK || !h.allowed(r, "employees:read") {
		h.forbidden(w, r)
		return
	}
	employee, err := h.Repository.GetEmployee(r.Context(), employeeID)
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusOK, employee)
}

func (h *EmployeeHandler) Update(w http.ResponseWriter, r *http.Request) {
	employeeID, employeeOK := pathUUID(r, "id")
	if !employeeOK || !h.allowed(r, "employees:update") {
		h.forbidden(w, r)
		return
	}
	request, err := DecodeJSON[UpdateEmployeeRequest](r, 1<<20)
	if err != nil {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	employee, err := h.Repository.UpdateEmployee(r.Context(), employeeID, request)
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusOK, employee)
}

func (h *EmployeeHandler) Delete(w http.ResponseWriter, r *http.Request) {
	employeeID, employeeOK := pathUUID(r, "id")
	if !employeeOK || !h.allowed(r, "employees:delete") {
		h.forbidden(w, r)
		return
	}
	if err := h.Repository.SoftDeleteEmployee(r.Context(), employeeID); err != nil {
		h.writeError(w, r, err)
		return
	}
	w.WriteHeader(http.StatusNoContent)
}

func (h *EmployeeHandler) Rehire(w http.ResponseWriter, r *http.Request) {
	employeeID, employeeOK := pathUUID(r, "id")
	if !employeeOK || !h.allowed(r, "employees:rehire") {
		h.forbidden(w, r)
		return
	}
	request, err := DecodeJSON[RehireEmployeeRequest](r, 1<<20)
	if err != nil || request.HireDate == "" || request.EmployerID == nil {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	employee, err := h.Repository.RehireEmployee(r.Context(), employeeID, request)
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusOK, employee)
}

func (h *EmployeeHandler) allowed(r *http.Request, permission string) bool {
	userID, ok := httpinfra.UserIDFromContext(r.Context())
	return ok && h.Authorizer != nil && h.Authorizer.HasPermission(r.Context(), userID, permission)
}
func (h *EmployeeHandler) forbidden(w http.ResponseWriter, r *http.Request) {
	apperror.WriteError(w, r, apperror.Forbidden(apperror.KeyForbidden))
}
func (h *EmployeeHandler) writeError(w http.ResponseWriter, r *http.Request, err error) {
	if errors.Is(err, ErrConflict) {
		apperror.WriteError(w, r, apperror.Conflict(apperror.KeyConflict))
		return
	}
	if errors.Is(err, pgx.ErrNoRows) {
		apperror.WriteError(w, r, apperror.NotFound(apperror.KeyNotFound))
		return
	}
	apperror.WriteError(w, r, apperror.Internal(err))
}
