package api

import (
	"context"
	"errors"
	"net/http"
	"strconv"
	"strings"
	"time"

	"github.com/google/uuid"
	"github.com/vkaylee/leeattend/apiserver/internal/apperror"
	"github.com/vkaylee/leeattend/apiserver/internal/audit"
	"github.com/vkaylee/leeattend/apiserver/internal/domain/identity"
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
	TenureID           *uuid.UUID `json:"tenure_id,omitempty"`
	TenureStatus       *string    `json:"tenure_status,omitempty"`
	HireType           *string    `json:"hire_type,omitempty"`
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
	FullName             string     `json:"full_name"`
	EmployeeIDNumber     string     `json:"employee_id_number"`
	HireDate             string     `json:"hire_date"`
	UserID               *uuid.UUID `json:"user_id"`
	PrimaryLocationID    *uuid.UUID `json:"primary_location_id"`
	EmployerID           *uuid.UUID `json:"employer_id"`
	DepartmentID         *uuid.UUID `json:"department_id"`
	Tags                 []string   `json:"tags"`
	Phone                *string    `json:"phone,omitempty"`
	DateOfBirth          *string    `json:"date_of_birth,omitempty"`
	NationalID           *string    `json:"national_id,omitempty"`
	PhoneLookupHash      *string    `json:"-"`
	NationalIDLookupHash *string    `json:"-"`
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
}

type EmployeeHandler struct {
	Repository      EmployeeRepository
	TenureCreator   EmployeeTenureCreator
	TenureManager   EmployeeTenureManager
	IdentityMatcher EmployeeIdentityMatcher
	IdentityHasher  identity.Hasher
	Authorizer      Authorizer
	Audit           AuditEmitter
	Metrics         *EmployeeIdentityMetrics
}

func (h *EmployeeHandler) Create(w http.ResponseWriter, r *http.Request) {
	startedAt := time.Now()
	actorID, ok := httpinfra.UserIDFromContext(r.Context())
	if !ok || actorID == uuid.Nil {
		apperror.WriteError(w, r, apperror.Unauthorized(apperror.KeyAuthenticationRequired))
		return
	}
	if h.Authorizer == nil || !h.Authorizer.HasPermission(r.Context(), actorID, "employees:create") {
		writeForbidden(w, r)
		return
	}
	request, err := DecodeJSON[CreateEmployeeRequest](r, 1<<20)
	if err != nil {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	prepared, err := prepareCreateEmployeeRequest(request, h.IdentityHasher)
	if err != nil {
		writeMatchBuildError(w, r, err)
		return
	}
	employee, err := h.Repository.CreateEmployee(r.Context(), prepared)
	h.Metrics.ObserveWrite(r.Context(), "create", err, time.Since(startedAt))
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	if h.Audit != nil {
		_ = h.Audit.Emit(r.Context(), audit.Event{
			Name:     "employees.created",
			ActorID:  actorID.String(),
			Resource: "employees",
			Outcome:  "created",
			Details: map[string]string{
				"has_employer": strconv.FormatBool(prepared.EmployerID != nil),
				"has_identity": strconv.FormatBool(prepared.PhoneLookupHash != nil || prepared.NationalIDLookupHash != nil),
			},
			At: time.Now().UTC(),
		})
	}
	apperror.WriteJSON(w, http.StatusCreated, employee)
}

func prepareCreateEmployeeRequest(req CreateEmployeeRequest, hasher identity.Hasher) (CreateEmployeeRequest, error) {
	fullName := strings.TrimSpace(req.FullName)
	code := strings.TrimSpace(req.EmployeeIDNumber)
	if fullName == "" || len(fullName) > maxMatchFullNameBytes || containsControlChar(fullName) {
		return CreateEmployeeRequest{}, ErrInvalidInput
	}
	if code == "" || len(code) > 50 || containsControlChar(code) {
		return CreateEmployeeRequest{}, ErrInvalidInput
	}
	hireDate, err := identity.NormalizeDOB(req.HireDate)
	if err != nil || hireDate == "" {
		return CreateEmployeeRequest{}, ErrInvalidInput
	}
	req.FullName = fullName
	req.EmployeeIDNumber = code
	req.HireDate = hireDate
	return normalizeCreateEmployeeIdentity(req, hasher)
}

func normalizeCreateEmployeeIdentity(req CreateEmployeeRequest, hasher identity.Hasher) (CreateEmployeeRequest, error) {
	var err error
	if req.DateOfBirth, err = normalizeOptionalDOB(req.DateOfBirth); err != nil {
		return CreateEmployeeRequest{}, err
	}
	if req.Phone, req.PhoneLookupHash, err = normalizeOptionalPhone(req.Phone, hasher); err != nil {
		return CreateEmployeeRequest{}, err
	}
	if req.NationalID, req.NationalIDLookupHash, err = normalizeOptionalNationalID(req.NationalID, hasher); err != nil {
		return CreateEmployeeRequest{}, err
	}
	return req, nil
}

func normalizeOptionalDOB(raw *string) (*string, error) {
	if raw == nil {
		return nil, nil
	}
	dob, err := identity.NormalizeDOB(*raw)
	if err != nil {
		return nil, ErrInvalidInput
	}
	if dob == "" {
		return nil, nil
	}
	return &dob, nil
}

func normalizeOptionalPhone(raw *string, hasher identity.Hasher) (*string, *string, error) {
	if raw == nil {
		return nil, nil, nil
	}
	normalized, err := identity.NormalizePhone(*raw)
	if err != nil {
		return nil, nil, ErrInvalidInput
	}
	if normalized == "" {
		return nil, nil, nil
	}
	hash, _, err := hasher.HashPhone(normalized)
	if err != nil {
		return nil, nil, err
	}
	return &normalized, &hash, nil
}

func normalizeOptionalNationalID(raw *string, hasher identity.Hasher) (*string, *string, error) {
	if raw == nil {
		return nil, nil, nil
	}
	trimmed := strings.TrimSpace(*raw)
	if len(trimmed) > maxMatchNationalIDBytes || containsControlChar(trimmed) {
		return nil, nil, ErrInvalidInput
	}
	if identity.NormalizeNationalID(trimmed) == "" {
		return nil, nil, nil
	}
	hash, _, err := hasher.HashNationalID(trimmed)
	if err != nil {
		return nil, nil, err
	}
	return &trimmed, &hash, nil
}

func (h *EmployeeHandler) List(w http.ResponseWriter, r *http.Request) {
	if !hasPermission(r, h.Authorizer, "employees:read") {
		writeForbidden(w, r)
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
	if !employeeOK || !hasPermission(r, h.Authorizer, "employees:read") {
		writeForbidden(w, r)
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
	if !employeeOK || !hasPermission(r, h.Authorizer, "employees:update") {
		writeForbidden(w, r)
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
	if !employeeOK || !hasPermission(r, h.Authorizer, "employees:delete") {
		writeForbidden(w, r)
		return
	}
	if err := h.Repository.SoftDeleteEmployee(r.Context(), employeeID); err != nil {
		h.writeError(w, r, err)
		return
	}
	w.WriteHeader(http.StatusNoContent)
}

func (h *EmployeeHandler) writeError(w http.ResponseWriter, r *http.Request, err error) {
	if errors.Is(err, ErrInvalidInput) {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	if errors.Is(err, ErrConflict) {
		apperror.WriteError(w, r, apperror.Conflict(apperror.KeyConflict))
		return
	}
	if errors.Is(err, ErrNotFound) {
		apperror.WriteError(w, r, apperror.NotFound(apperror.KeyNotFound))
		return
	}
	apperror.WriteError(w, r, apperror.Internal(err))
}
