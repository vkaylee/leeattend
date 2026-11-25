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
	"github.com/vkaylee/leeattend/apiserver/internal/auth"
	"github.com/vkaylee/leeattend/apiserver/internal/domain/identity"
	httpinfra "github.com/vkaylee/leeattend/apiserver/internal/infra/http"
)

const TenureCreatedAuditAction = "employees.tenure.created"

type CreateEmployeeTenureRequest struct {
	StartDate            string     `json:"start_date"`
	EmployerID           uuid.UUID  `json:"employer_id"`
	PrimaryLocationID    *uuid.UUID `json:"primary_location_id,omitempty"`
	DepartmentID         *uuid.UUID `json:"department_id,omitempty"`
	ExpectedEmployeeCode *string    `json:"expected_employee_code,omitempty"`
	ExpectedStatus       *string    `json:"expected_status,omitempty"`
}

type EmployeeTenureCreated struct {
	TenureID       uuid.UUID `json:"tenure_id"`
	EmployeeID     uuid.UUID `json:"employee_id"`
	EmployeeCode   string    `json:"employee_code"`
	StartDate      string    `json:"start_date"`
	RelationshipID uuid.UUID `json:"relationship_id"`
	EmploymentID   uuid.UUID `json:"employment_id"`
	EmployerID     uuid.UUID `json:"employer_id"`
	Status         string    `json:"status"`
}

type EmployeeTenureCreator interface {
	CreateEmployeeTenure(context.Context, uuid.UUID, CreateEmployeeTenureRequest) (EmployeeTenureCreated, error)
}

// CreateTenure handles POST /api/v1/employees/{employee_id}/tenures.
// It explicitly reuses an existing terminated employee (preserving employee ID, identity, and code)
// by creating a new non-overlapping tenure, employment relationship, and employment atomically.
func (h *EmployeeHandler) CreateTenure(w http.ResponseWriter, r *http.Request) {
	startedAt := time.Now()
	actorID, ok := httpinfra.UserIDFromContext(r.Context())
	if !ok || actorID == uuid.Nil {
		apperror.WriteError(w, r, apperror.Unauthorized(apperror.KeyAuthenticationRequired))
		return
	}
	employeeID, employeeOK := pathUUID(r, "employee_id")
	if !employeeOK {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	req, ok := decodeCreateTenureRequest(w, r)
	if !ok {
		return
	}
	if !h.authorizeCreateTenure(w, r, actorID, req.DepartmentID) {
		return
	}
	prepared, err := validateCreateTenureRequest(req)
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	if h.TenureCreator == nil {
		apperror.WriteError(w, r, apperror.ServiceUnavailable(apperror.KeyServiceUnavailable))
		return
	}

	created, err := h.TenureCreator.CreateEmployeeTenure(r.Context(), employeeID, prepared)
	h.Metrics.ObserveWrite(r.Context(), "reuse_tenure", err, time.Since(startedAt))
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	if h.Audit != nil {
		_ = h.Audit.Emit(r.Context(), audit.Event{
			Name:     TenureCreatedAuditAction,
			ActorID:  actorID.String(),
			Resource: "employees",
			Outcome:  "created",
			Details: map[string]string{
				"has_department": strconv.FormatBool(prepared.DepartmentID != nil),
				"has_location":   strconv.FormatBool(prepared.PrimaryLocationID != nil),
			},
			At: time.Now().UTC(),
		})
	}
	apperror.WriteJSON(w, http.StatusCreated, created)
}

func (h *EmployeeHandler) authorizeCreateTenure(w http.ResponseWriter, r *http.Request, actorID uuid.UUID, departmentID *uuid.UUID) bool {
	if h.Authorizer == nil || !h.Authorizer.HasScopedPermission(r.Context(), actorID, string(auth.PermissionEmployeesRehire), departmentID) {
		h.forbidden(w, r)
		return false
	}
	return true
}

func decodeCreateTenureRequest(w http.ResponseWriter, r *http.Request) (CreateEmployeeTenureRequest, bool) {
	req, err := DecodeJSON[CreateEmployeeTenureRequest](r, 1<<20)
	if err != nil {
		var maxBytesErr *http.MaxBytesError
		if errors.As(err, &maxBytesErr) {
			apperror.WriteError(w, r, apperror.PayloadTooLarge(apperror.KeyFileTooLarge))
			return CreateEmployeeTenureRequest{}, false
		}
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return CreateEmployeeTenureRequest{}, false
	}
	return req, true
}

func validateCreateTenureRequest(req CreateEmployeeTenureRequest) (CreateEmployeeTenureRequest, error) {
	startDate, err := identity.NormalizeDOB(req.StartDate)
	if err != nil || startDate == "" || req.EmployerID == uuid.Nil {
		return CreateEmployeeTenureRequest{}, ErrInvalidInput
	}
	if req.PrimaryLocationID != nil && *req.PrimaryLocationID == uuid.Nil {
		return CreateEmployeeTenureRequest{}, ErrInvalidInput
	}
	if req.DepartmentID != nil && *req.DepartmentID == uuid.Nil {
		return CreateEmployeeTenureRequest{}, ErrInvalidInput
	}
	if err := validateExpectedTenureGuards(req.ExpectedEmployeeCode, req.ExpectedStatus); err != nil {
		return CreateEmployeeTenureRequest{}, err
	}
	req.StartDate = startDate
	return req, nil
}

func validateExpectedTenureGuards(expectedCode, expectedStatus *string) error {
	if expectedCode != nil && identity.NormalizeEmployeeCode(*expectedCode) == "" {
		return ErrInvalidInput
	}
	if expectedStatus == nil {
		return nil
	}
	switch strings.ToLower(strings.TrimSpace(*expectedStatus)) {
	case MatchStatusTerminated:
		return nil
	case MatchStatusActive:
		return ErrConflict
	default:
		return ErrInvalidInput
	}
}
