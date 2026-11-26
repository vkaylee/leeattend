package api

import (
	"context"
	"errors"
	"net/http"
	"strings"

	"github.com/google/uuid"
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

type DepartmentRepository interface {
	ListDepartments(ctx context.Context, page, limit int) (DepartmentListResponse, error)
	CreateDepartment(ctx context.Context, req CreateDepartmentRequest) (DepartmentRecord, error)
	GetDepartmentByID(ctx context.Context, deptID uuid.UUID) (DepartmentRecord, error)
	UpdateDepartment(ctx context.Context, deptID uuid.UUID, req UpdateDepartmentRequest) (DepartmentRecord, error)
	DeleteDepartment(ctx context.Context, deptID uuid.UUID) error
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

func (h *DepartmentHandler) writeError(w http.ResponseWriter, r *http.Request, err error) {
	writeMappedError(w, r, err, func(err error) *apperror.AppError {
		switch {
		case errors.Is(err, ErrDepartmentNotFound), errors.Is(err, ErrNotFound):
			return apperror.NotFound(apperror.KeyNotFound)
		case errors.Is(err, ErrDepartmentConflict):
			return apperror.Conflict(apperror.KeyConflict)
		default:
			return nil
		}
	})
}
