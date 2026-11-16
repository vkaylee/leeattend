package api

import (
	"context"
	"errors"
	"net/http"
	"strings"
	"time"

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

type DepartmentPayrollReconciliation struct {
	DepartmentID            uuid.UUID        `json:"department_id"`
	DepartmentName          string           `json:"department_name"`
	LegacyStdWorkingDays    *decimal.Decimal `json:"legacy_std_working_days,omitempty"`
	CanonicalEffectiveFrom  *string          `json:"canonical_effective_from,omitempty"`
	CanonicalStdWorkingDays *decimal.Decimal `json:"canonical_std_working_days,omitempty"`
	Status                  string           `json:"reconciliation_status"`
}

type DepartmentPayrollReconciliationResponse struct {
	Data  []DepartmentPayrollReconciliation `json:"data"`
	Total int64                             `json:"total"`
	Page  int                               `json:"page"`
	Limit int                               `json:"limit"`
}

type SetDepartmentPayrollConfigRequest struct {
	StdWorkingDays decimal.Decimal `json:"std_working_days"`
	HoursPerDay    decimal.Decimal `json:"hours_per_day"`
	EffectiveDate  string          `json:"effective_date"`
}

type DepartmentRepository interface {
	ListDepartments(ctx context.Context, page, limit int) (DepartmentListResponse, error)
	CreateDepartment(ctx context.Context, req CreateDepartmentRequest) (DepartmentRecord, error)
	GetDepartmentByID(ctx context.Context, deptID uuid.UUID) (DepartmentRecord, error)
	UpdateDepartment(ctx context.Context, deptID uuid.UUID, req UpdateDepartmentRequest) (DepartmentRecord, error)
	DeleteDepartment(ctx context.Context, deptID uuid.UUID) error
	GetDepartmentPayrollConfigs(ctx context.Context, deptID uuid.UUID) ([]DepartmentPayrollConfig, error)
	ListDepartmentPayrollReconciliation(ctx context.Context, page, limit int, status string) (DepartmentPayrollReconciliationResponse, error)
	SetDepartmentPayrollConfig(ctx context.Context, deptID uuid.UUID, req SetDepartmentPayrollConfigRequest, actorID uuid.UUID) (DepartmentPayrollConfig, error)
}

type DepartmentHandler struct {
	Repository DepartmentRepository
	Authorizer Authorizer
}

func (h *DepartmentHandler) List(w http.ResponseWriter, r *http.Request) {
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
	res, err := h.Repository.ListDepartments(r.Context(), p.Page, p.Limit)
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusOK, res)
}

func (h *DepartmentHandler) Create(w http.ResponseWriter, r *http.Request) {
	userID, userOK := httpinfra.UserIDFromContext(r.Context())
	if !userOK || h.Authorizer == nil || !h.Authorizer.HasPermission(r.Context(), userID, string(auth.PermissionDepartmentsCreate)) {
		apperror.WriteError(w, r, apperror.Forbidden(apperror.KeyForbidden))
		return
	}
	req, err := DecodeJSON[CreateDepartmentRequest](r, 1<<20)
	if err != nil || strings.TrimSpace(req.Name) == "" {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	req.Name = strings.TrimSpace(req.Name)
	dept, err := h.Repository.CreateDepartment(r.Context(), req)
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusCreated, dept)
}

func (h *DepartmentHandler) Update(w http.ResponseWriter, r *http.Request) {
	deptID, deptOK := pathUUID(r, "id")
	if !deptOK {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	userID, userOK := httpinfra.UserIDFromContext(r.Context())
	if !userOK || h.Authorizer == nil || !h.Authorizer.HasScopedPermission(r.Context(), userID, string(auth.PermissionDepartmentsUpdate), &deptID) {
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
	dept, err := h.Repository.UpdateDepartment(r.Context(), deptID, req)
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusOK, dept)
}

func (h *DepartmentHandler) Delete(w http.ResponseWriter, r *http.Request) {
	deptID, deptOK := pathUUID(r, "id")
	if !deptOK {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	userID, userOK := httpinfra.UserIDFromContext(r.Context())
	if !userOK || h.Authorizer == nil || !h.Authorizer.HasScopedPermission(r.Context(), userID, string(auth.PermissionDepartmentsDelete), &deptID) {
		apperror.WriteError(w, r, apperror.Forbidden(apperror.KeyForbidden))
		return
	}
	if err := h.Repository.DeleteDepartment(r.Context(), deptID); err != nil {
		h.writeError(w, r, err)
		return
	}
	w.WriteHeader(http.StatusNoContent)
}

func (h *DepartmentHandler) ListPayrollReconciliation(w http.ResponseWriter, r *http.Request) {
	userID, userOK := httpinfra.UserIDFromContext(r.Context())
	if !userOK || h.Authorizer == nil || !h.Authorizer.HasPermission(r.Context(), userID, string(auth.PermissionPayrollRead)) {
		apperror.WriteError(w, r, apperror.Forbidden(apperror.KeyForbidden))
		return
	}
	pagination, err := ParsePagination(r.URL.Query())
	if err != nil {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	status := strings.TrimSpace(r.URL.Query().Get("status"))
	if status != "" {
		switch status {
		case "matched", "missing_legacy_value", "missing_canonical_config", "value_mismatch":
		default:
			apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
			return
		}
	}
	result, err := h.Repository.ListDepartmentPayrollReconciliation(r.Context(), pagination.Page, pagination.Limit, status)
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusOK, result)
}

func (h *DepartmentHandler) GetPayrollConfig(w http.ResponseWriter, r *http.Request) {
	deptID, deptOK := pathUUID(r, "id")
	if !deptOK {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	_, userOK := httpinfra.UserIDFromContext(r.Context())
	if !userOK {
		apperror.WriteError(w, r, apperror.Unauthorized(apperror.KeyAuthenticationRequired))
		return
	}
	configs, err := h.Repository.GetDepartmentPayrollConfigs(r.Context(), deptID)
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusOK, configs)
}

func (h *DepartmentHandler) SetPayrollConfig(w http.ResponseWriter, r *http.Request) {
	deptID, deptOK := pathUUID(r, "id")
	if !deptOK {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	userID, userOK := httpinfra.UserIDFromContext(r.Context())
	if !userOK || h.Authorizer == nil || !h.Authorizer.HasScopedPermission(r.Context(), userID, string(auth.PermissionDepartmentsUpdate), &deptID) {
		apperror.WriteError(w, r, apperror.Forbidden(apperror.KeyForbidden))
		return
	}
	req, err := DecodeJSON[SetDepartmentPayrollConfigRequest](r, 1<<20)
	if err != nil || strings.TrimSpace(req.EffectiveDate) == "" || req.StdWorkingDays.IsNegative() || req.HoursPerDay.IsNegative() {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	if _, err := time.Parse("2006-01-02", strings.TrimSpace(req.EffectiveDate)); err != nil {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	req.EffectiveDate = strings.TrimSpace(req.EffectiveDate)
	config, err := h.Repository.SetDepartmentPayrollConfig(r.Context(), deptID, req, userID)
	if err != nil {
		h.writePayrollError(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusCreated, config)
}
func (h *DepartmentHandler) writeError(w http.ResponseWriter, r *http.Request, err error) {
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

func (h *DepartmentHandler) writePayrollError(w http.ResponseWriter, r *http.Request, err error) {
	if errors.Is(err, ErrDepartmentNotFound) || errors.Is(err, pgx.ErrNoRows) {
		apperror.WriteError(w, r, apperror.NotFound(apperror.KeyDepartmentNotFound))
		return
	}
	apperror.WriteError(w, r, apperror.Internal(err))
}
