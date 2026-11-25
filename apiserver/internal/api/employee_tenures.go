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

const (
	TenureCreatedAuditAction = "employees.tenure.created"
	TenureClosedAuditAction  = "employees.tenure.closed"
)

type CreateEmployeeTenureRequest struct {
	StartDate              string     `json:"start_date"`
	EmployerID             uuid.UUID  `json:"employer_id"`
	EmployeeNumber         *string    `json:"employee_number,omitempty"`
	PrimaryLocationID      *uuid.UUID `json:"primary_location_id,omitempty"`
	DepartmentID           *uuid.UUID `json:"department_id,omitempty"`
	PriorServiceCreditDays *int       `json:"prior_service_credit_days,omitempty"`
	Notes                  *string    `json:"notes,omitempty"`
	ExpectedEmployeeCode   *string    `json:"expected_employee_code,omitempty"`
	ExpectedStatus         *string    `json:"expected_status,omitempty"`
}

type EmployeeTenureCreated struct {
	TenureID               uuid.UUID `json:"tenure_id"`
	EmployeeID             uuid.UUID `json:"employee_id"`
	EmployeeCode           string    `json:"employee_code"`
	HireType               string    `json:"hire_type"`
	PriorServiceCreditDays int       `json:"prior_service_credit_days"`
	StartDate              string    `json:"start_date"`
	RelationshipID         uuid.UUID `json:"relationship_id"`
	EmploymentID           uuid.UUID `json:"employment_id"`
	EmployerID             uuid.UUID `json:"employer_id"`
	Status                 string    `json:"status"`
}

type EmployeeTenureRelationshipSummary struct {
	ID                uuid.UUID `json:"id"`
	EmployerID        uuid.UUID `json:"employer_id"`
	EmployerName      string    `json:"employer_name"`
	Status            string    `json:"status"`
	StartDate         string    `json:"start_date"`
	EndDate           *string   `json:"end_date,omitempty"`
	TerminationReason *string   `json:"termination_reason,omitempty"`
}

type EmployeeTenureRecord struct {
	ID                     uuid.UUID                           `json:"id"`
	EmployeeID             uuid.UUID                           `json:"employee_id"`
	EmployeeNumber         string                              `json:"employee_number"`
	UserID                 *uuid.UUID                          `json:"user_id,omitempty"`
	HireType               string                              `json:"hire_type"`
	PriorServiceCreditDays int                                 `json:"prior_service_credit_days"`
	Status                 string                              `json:"status"`
	StartDate              string                              `json:"start_date"`
	EndDate                *string                             `json:"end_date,omitempty"`
	TerminationReason      *string                             `json:"termination_reason,omitempty"`
	RehireEligible         bool                                `json:"rehire_eligible"`
	Notes                  *string                             `json:"notes,omitempty"`
	CreatedAt              string                              `json:"created_at"`
	UpdatedAt              string                              `json:"updated_at"`
	Relationships          []EmployeeTenureRelationshipSummary `json:"relationships"`
}

type CloseEmployeeTenureRequest struct {
	EndDate           string  `json:"end_date"`
	TerminationReason *string `json:"termination_reason,omitempty"`
	RehireEligible    *bool   `json:"rehire_eligible,omitempty"`
	Notes             *string `json:"notes,omitempty"`
}

type EmployeeTenureClosed struct {
	TenureID            uuid.UUID `json:"tenure_id"`
	EmployeeID          uuid.UUID `json:"employee_id"`
	EndDate             string    `json:"end_date"`
	ClosedRelationships int       `json:"closed_relationships"`
	ClosedSegments      int       `json:"closed_segments"`
	SessionsRevoked     bool      `json:"sessions_revoked"`
}

type EmployeeTenureCreator interface {
	CreateEmployeeTenure(context.Context, uuid.UUID, CreateEmployeeTenureRequest) (EmployeeTenureCreated, error)
}

type EmployeeTenureManager interface {
	ListEmployeeTenures(context.Context, uuid.UUID) ([]EmployeeTenureRecord, error)
	CloseEmployeeTenure(context.Context, uuid.UUID, uuid.UUID, CloseEmployeeTenureRequest) (EmployeeTenureClosed, error)
}

// ListTenures handles GET /api/v1/employees/{employee_id}/tenures.
func (h *EmployeeHandler) ListTenures(w http.ResponseWriter, r *http.Request) {
	if !h.allowed(r, "employees:read") {
		h.forbidden(w, r)
		return
	}
	employeeID, ok := pathUUID(r, "employee_id")
	if !ok {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	if h.TenureManager == nil {
		apperror.WriteError(w, r, apperror.ServiceUnavailable(apperror.KeyServiceUnavailable))
		return
	}
	tenures, err := h.TenureManager.ListEmployeeTenures(r.Context(), employeeID)
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	if tenures == nil {
		tenures = []EmployeeTenureRecord{}
	}
	apperror.WriteJSON(w, http.StatusOK, tenures)
}

// CreateTenure handles POST /api/v1/employees/{employee_id}/tenures.
// It is the unified rehire endpoint: preserves person identity and atomically creates
// a new non-overlapping tenure (reusing the previous number or issuing a new permanently-owned
// number), employment relationship, and initial employment segment.
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
				"has_department":  strconv.FormatBool(prepared.DepartmentID != nil),
				"has_location":    strconv.FormatBool(prepared.PrimaryLocationID != nil),
				"has_custom_code": strconv.FormatBool(prepared.EmployeeNumber != nil),
			},
			At: time.Now().UTC(),
		})
	}
	apperror.WriteJSON(w, http.StatusCreated, created)
}

// CloseTenure handles POST /api/v1/employees/{employee_id}/tenures/{tenure_id}/close.
func (h *EmployeeHandler) CloseTenure(w http.ResponseWriter, r *http.Request) {
	actorID, employeeID, tenureID, req, ok := h.parseCloseTenureRequest(w, r)
	if !ok {
		return
	}
	if h.TenureManager == nil {
		apperror.WriteError(w, r, apperror.ServiceUnavailable(apperror.KeyServiceUnavailable))
		return
	}
	closed, err := h.TenureManager.CloseEmployeeTenure(r.Context(), employeeID, tenureID, req)
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	h.emitTenureClosedAudit(r.Context(), actorID, closed)
	apperror.WriteJSON(w, http.StatusOK, closed)
}

func (h *EmployeeHandler) parseCloseTenureRequest(w http.ResponseWriter, r *http.Request) (uuid.UUID, uuid.UUID, uuid.UUID, CloseEmployeeTenureRequest, bool) {
	actorID, ok := httpinfra.UserIDFromContext(r.Context())
	if !ok || actorID == uuid.Nil {
		apperror.WriteError(w, r, apperror.Unauthorized(apperror.KeyAuthenticationRequired))
		return uuid.Nil, uuid.Nil, uuid.Nil, CloseEmployeeTenureRequest{}, false
	}
	if !h.allowed(r, "employees:update") {
		h.forbidden(w, r)
		return uuid.Nil, uuid.Nil, uuid.Nil, CloseEmployeeTenureRequest{}, false
	}
	employeeID, empOK := pathUUID(r, "employee_id")
	tenureID, tenureOK := pathUUID(r, "tenure_id")
	if !empOK || !tenureOK {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return uuid.Nil, uuid.Nil, uuid.Nil, CloseEmployeeTenureRequest{}, false
	}
	req, err := DecodeJSON[CloseEmployeeTenureRequest](r, 1<<20)
	if err != nil {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return uuid.Nil, uuid.Nil, uuid.Nil, CloseEmployeeTenureRequest{}, false
	}
	endDate, err := identity.NormalizeDOB(req.EndDate)
	if err != nil || endDate == "" {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return uuid.Nil, uuid.Nil, uuid.Nil, CloseEmployeeTenureRequest{}, false
	}
	req.EndDate = endDate
	return actorID, employeeID, tenureID, req, true
}

func (h *EmployeeHandler) emitTenureClosedAudit(ctx context.Context, actorID uuid.UUID, closed EmployeeTenureClosed) {
	if h.Audit == nil {
		return
	}
	_ = h.Audit.Emit(ctx, audit.Event{
		Name:     TenureClosedAuditAction,
		ActorID:  actorID.String(),
		Resource: "employees",
		Outcome:  "closed",
		Details: map[string]string{
			"closed_relationships": strconv.Itoa(closed.ClosedRelationships),
			"closed_segments":      strconv.Itoa(closed.ClosedSegments),
			"sessions_revoked":     strconv.FormatBool(closed.SessionsRevoked),
		},
		At: time.Now().UTC(),
	})
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
	if req.PriorServiceCreditDays != nil && *req.PriorServiceCreditDays < 0 {
		return CreateEmployeeTenureRequest{}, ErrInvalidInput
	}
	if req.EmployeeNumber != nil {
		trimmed := strings.TrimSpace(*req.EmployeeNumber)
		if trimmed == "" || len(trimmed) > 50 || containsControlChar(trimmed) {
			return CreateEmployeeTenureRequest{}, ErrInvalidInput
		}
		req.EmployeeNumber = &trimmed
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
