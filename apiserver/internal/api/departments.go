package api

import (
	"context"
	"errors"
	"net/http"
	"strings"

	"github.com/google/uuid"
	"github.com/jackc/pgx/v5"
	"github.com/shopspring/decimal"
	"github.com/vkaylee/leeattend/apiserver/internal/apperror"
	"github.com/vkaylee/leeattend/apiserver/internal/auth"
	httpinfra "github.com/vkaylee/leeattend/apiserver/internal/infra/http"
)

var (
	ErrDepartmentNotFound = errors.New("department: not found")
	ErrDepartmentConflict = errors.New("department: name already exists")
)

type DepartmentRecord struct {
	ID                  uuid.UUID  `json:"id"`
	WorkspaceID         uuid.UUID  `json:"workspace_id"`
	Name                string     `json:"name"`
	Description         *string    `json:"description"`
	ManagerID           *uuid.UUID `json:"manager_id"`
	CreatedAt           string     `json:"created_at"`
	UpdatedAt           string     `json:"updated_at"`
	ActiveEmployeeCount int64      `json:"active_employee_count"`
}

type DepartmentListResponse struct {
	Data  []DepartmentRecord `json:"data"`
	Total int64              `json:"total"`
	Page  int                `json:"page"`
	Limit int                `json:"limit"`
}

type CreateDepartmentRequest struct {
	Name        string     `json:"name"`
	Description *string    `json:"description"`
	ManagerID   *uuid.UUID `json:"manager_id"`
}

type UpdateDepartmentRequest struct {
	Name        *string    `json:"name"`
	Description *string    `json:"description"`
	ManagerID   *uuid.UUID `json:"manager_id"`
}

type DepartmentPayrollConfig struct {
	ID             uuid.UUID       `json:"id"`
	DepartmentID   uuid.UUID       `json:"department_id"`
	StdWorkingDays decimal.Decimal `json:"std_working_days"`
	HoursPerDay    decimal.Decimal `json:"hours_per_day"`
	EffectiveDate  string          `json:"effective_date"`
	CreatedAt      string          `json:"created_at"`
}

type SetDepartmentPayrollConfigRequest struct {
	StdWorkingDays decimal.Decimal `json:"std_working_days"`
	HoursPerDay    decimal.Decimal `json:"hours_per_day"`
	EffectiveDate  string          `json:"effective_date"`
}

type DepartmentRepository interface {
	ListDepartments(ctx context.Context, workspaceID uuid.UUID, page, limit int) (DepartmentListResponse, error)
	CreateDepartment(ctx context.Context, workspaceID uuid.UUID, req CreateDepartmentRequest) (DepartmentRecord, error)
	GetDepartmentByID(ctx context.Context, workspaceID, deptID uuid.UUID) (DepartmentRecord, error)
	UpdateDepartment(ctx context.Context, workspaceID, deptID uuid.UUID, req UpdateDepartmentRequest) (DepartmentRecord, error)
	DeleteDepartment(ctx context.Context, workspaceID, deptID uuid.UUID) error
	GetDepartmentPayrollConfigs(ctx context.Context, workspaceID, deptID uuid.UUID) ([]DepartmentPayrollConfig, error)
	SetDepartmentPayrollConfig(ctx context.Context, workspaceID, deptID uuid.UUID, req SetDepartmentPayrollConfigRequest, actorID uuid.UUID) (DepartmentPayrollConfig, error)
}

type DepartmentHandler struct {
	Repository DepartmentRepository
	Authorizer WorkspaceAuthorizer
}

func (h DepartmentHandler) List(w http.ResponseWriter, r *http.Request) {
	wsID, ok := organizationIDFromContext(r)
	if !ok {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	_, userOK := httpinfra.UserIDFromContext(r.Context())
	if !userOK {
		apperror.WriteError(w, r, apperror.Unauthorized(apperror.KeyAuthenticationRequired))
		return
	}
	p, err := ParsePagination(r.URL.Query())
	if err != nil {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	res, err := h.Repository.ListDepartments(r.Context(), wsID, p.Page, p.Limit)
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusOK, res)
}

func (h DepartmentHandler) Create(w http.ResponseWriter, r *http.Request) {
	wsID, ok := organizationIDFromContext(r)
	if !ok {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	userID, userOK := httpinfra.UserIDFromContext(r.Context())
	if !userOK || h.Authorizer == nil || !h.Authorizer.HasPermission(r.Context(), userID, wsID, string(auth.PermissionDepartmentsCreate)) {
		apperror.WriteError(w, r, apperror.Forbidden(apperror.KeyForbidden))
		return
	}
	req, err := DecodeJSON[CreateDepartmentRequest](r, 1<<20)
	if err != nil || strings.TrimSpace(req.Name) == "" {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	req.Name = strings.TrimSpace(req.Name)
	dept, err := h.Repository.CreateDepartment(r.Context(), wsID, req)
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusCreated, dept)
}

func (h DepartmentHandler) Update(w http.ResponseWriter, r *http.Request) {
	wsID, ok := organizationIDFromContext(r)
	deptID, deptOK := pathUUID(r, "id")
	if !ok || !deptOK {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	userID, userOK := httpinfra.UserIDFromContext(r.Context())
	if !userOK || h.Authorizer == nil || !h.Authorizer.HasPermission(r.Context(), userID, wsID, string(auth.PermissionDepartmentsUpdate)) {
		apperror.WriteError(w, r, apperror.Forbidden(apperror.KeyForbidden))
		return
	}
	req, err := DecodeJSON[UpdateDepartmentRequest](r, 1<<20)
	if err != nil || (req.Name == nil && req.Description == nil && req.ManagerID == nil) {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	if req.Name != nil {
		trimmed := strings.TrimSpace(*req.Name)
		req.Name = &trimmed
	}
	dept, err := h.Repository.UpdateDepartment(r.Context(), wsID, deptID, req)
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusOK, dept)
}

func (h DepartmentHandler) Delete(w http.ResponseWriter, r *http.Request) {
	wsID, ok := organizationIDFromContext(r)
	deptID, deptOK := pathUUID(r, "id")
	if !ok || !deptOK {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	userID, userOK := httpinfra.UserIDFromContext(r.Context())
	if !userOK || h.Authorizer == nil || !h.Authorizer.HasPermission(r.Context(), userID, wsID, string(auth.PermissionDepartmentsDelete)) {
		apperror.WriteError(w, r, apperror.Forbidden(apperror.KeyForbidden))
		return
	}
	if err := h.Repository.DeleteDepartment(r.Context(), wsID, deptID); err != nil {
		h.writeError(w, r, err)
		return
	}
	w.WriteHeader(http.StatusNoContent)
}

func (h DepartmentHandler) GetPayrollConfig(w http.ResponseWriter, r *http.Request) {
	wsID, ok := organizationIDFromContext(r)
	deptID, deptOK := pathUUID(r, "id")
	if !ok || !deptOK {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	_, userOK := httpinfra.UserIDFromContext(r.Context())
	if !userOK {
		apperror.WriteError(w, r, apperror.Unauthorized(apperror.KeyAuthenticationRequired))
		return
	}
	configs, err := h.Repository.GetDepartmentPayrollConfigs(r.Context(), wsID, deptID)
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusOK, configs)
}

func (h DepartmentHandler) SetPayrollConfig(w http.ResponseWriter, r *http.Request) {
	wsID, ok := organizationIDFromContext(r)
	deptID, deptOK := pathUUID(r, "id")
	if !ok || !deptOK {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	userID, userOK := httpinfra.UserIDFromContext(r.Context())
	if !userOK {
		apperror.WriteError(w, r, apperror.Unauthorized(apperror.KeyAuthenticationRequired))
		return
	}
	req, err := DecodeJSON[SetDepartmentPayrollConfigRequest](r, 1<<20)
	if err != nil || req.EffectiveDate == "" {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	config, err := h.Repository.SetDepartmentPayrollConfig(r.Context(), wsID, deptID, req, userID)
	if err != nil {
		h.writePayrollError(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusCreated, config)
}

func (h DepartmentHandler) writeError(w http.ResponseWriter, r *http.Request, err error) {
	if errors.Is(err, ErrDepartmentNotFound) || errors.Is(err, pgx.ErrNoRows) {
		apperror.WriteError(w, r, apperror.NotFound(apperror.KeyNotFound))
		return
	}
	if errors.Is(err, ErrDepartmentConflict) {
		apperror.WriteError(w, r, apperror.Conflict(apperror.KeyConflict))
		return
	}
	apperror.WriteError(w, r, apperror.Internal(err))
}

func (h DepartmentHandler) writePayrollError(w http.ResponseWriter, r *http.Request, err error) {
	if errors.Is(err, ErrDepartmentNotFound) || errors.Is(err, pgx.ErrNoRows) {
		apperror.WriteError(w, r, apperror.NotFound(apperror.KeyDepartmentNotFound))
		return
	}
	apperror.WriteError(w, r, apperror.Internal(err))
}
