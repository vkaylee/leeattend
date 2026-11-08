package api

import (
	"context"
	"encoding/json"
	"errors"
	"fmt"
	"net/http"
	"strings"
	"time"

	"github.com/google/uuid"
	"github.com/jackc/pgx/v5"
	"github.com/vkaylee/leeattend/apiserver/internal/apperror"
	httpinfra "github.com/vkaylee/leeattend/apiserver/internal/infra/http"
)

var (
	ErrContractNotFound  = errors.New("contract: not found")
	ErrContractOverlap   = errors.New("contract: overlapping active contract")
	ErrEmploymentOverlap = errors.New("employment: overlapping employment")
)

const (
	ContractTypeProbation    = "probation"
	ContractTypeTraining     = "training"
	ContractTypeFixedTerm    = "fixed_term"
	ContractTypeIndefinite   = "indefinite"
	ContractStatusDraft      = "draft"
	ContractStatusActive     = "active"
	ContractStatusExpired    = "expired"
	ContractStatusTerminated = "terminated"
)

type EmployeeContract struct {
	ID             uuid.UUID  `json:"id"`
	EmploymentID   uuid.UUID  `json:"employment_id"`
	ContractNumber *string    `json:"contract_number"`
	ContractType   string     `json:"contract_type"`
	Status         string     `json:"status"`
	StartDate      string     `json:"start_date"`
	EndDate        *string    `json:"end_date"`
	DocumentID     *uuid.UUID `json:"document_id"`
	Notes          *string    `json:"notes"`
	CreatedAt      string     `json:"created_at"`
	UpdatedAt      string     `json:"updated_at"`
}

type CreateContractRequest struct {
	EmploymentID         uuid.UUID  `json:"employment_id"`
	ContractNumber       *string    `json:"contract_number"`
	ContractType         string     `json:"contract_type"`
	StartDate            string     `json:"start_date"`
	EndDate              *string    `json:"end_date"`
	DocumentID           *uuid.UUID `json:"document_id"`
	Notes                *string    `json:"notes"`
	AssignPhaseID        *uuid.UUID `json:"assign_phase_id"`
	AutoCloseOverlapping *bool      `json:"auto_close_overlapping"`
}

// UpdateContractRequest keeps omitted fields unchanged. Nullable fields use
// ** values so explicit JSON null clears their database value.
type UpdateContractRequest struct {
	ContractNumber    *string     `json:"contract_number"`
	ContractType      *string     `json:"contract_type"`
	Status            *string     `json:"status"`
	StartDate         *string     `json:"start_date"`
	EndDate           **string    `json:"end_date"`
	DocumentID        **uuid.UUID `json:"document_id"`
	Notes             **string    `json:"notes"`
	EndDatePresent    bool        `json:"-"`
	DocumentIDPresent bool        `json:"-"`
	NotesPresent      bool        `json:"-"`
}

func (r *UpdateContractRequest) UnmarshalJSON(data []byte) error {
	type plain UpdateContractRequest
	var value plain
	if err := json.Unmarshal(data, &value); err != nil {
		return err
	}
	var fields map[string]json.RawMessage
	if err := json.Unmarshal(data, &fields); err != nil {
		return err
	}
	allowed := map[string]bool{"contract_number": true, "contract_type": true, "status": true, "start_date": true, "end_date": true, "document_id": true, "notes": true}
	for key := range fields {
		if !allowed[key] {
			return fmt.Errorf("json: unknown field %q", key)
		}
	}
	*r = UpdateContractRequest(value)
	_, r.EndDatePresent = fields["end_date"]
	_, r.DocumentIDPresent = fields["document_id"]
	_, r.NotesPresent = fields["notes"]
	return nil
}

type EmployeeContractWithEmployee struct {
	EmployeeContract
	EmployeeName string    `json:"employee_name"`
	EmployeeCode string    `json:"employee_code"`
	EmployeeID   uuid.UUID `json:"employee_id"`
}

type Employment struct {
	ID                  uuid.UUID                  `json:"id"`
	EmployeeID          uuid.UUID                  `json:"employee_id"`
	EmployerID          uuid.UUID                  `json:"employer_id"`
	PrimaryLocationID   *uuid.UUID                 `json:"primary_location_id"`
	DepartmentID        *uuid.UUID                 `json:"department_id"`
	Status              string                     `json:"status"`
	StartDate           string                     `json:"start_date"`
	EndDate             *string                    `json:"end_date"`
	JobTitle            *string                    `json:"job_title"`
	EmploymentType      string                     `json:"employment_type"`
	ManagerEmploymentID *uuid.UUID                 `json:"manager_employment_id"`
	CreatedAt           string                     `json:"created_at"`
	UpdatedAt           string                     `json:"updated_at"`
	TerminationReason   *string                    `json:"termination_reason,omitempty"`
	Notes               *string                    `json:"notes,omitempty"`
	CustomFields        []CustomFieldValueResponse `json:"custom_fields,omitempty"`
}

type CreateEmploymentRequest struct {
	EmployeeID          uuid.UUID  `json:"employee_id"`
	EmployerID          uuid.UUID  `json:"employer_id"`
	PrimaryLocationID   *uuid.UUID `json:"primary_location_id"`
	DepartmentID        *uuid.UUID `json:"department_id"`
	Status              string     `json:"status"`
	StartDate           string     `json:"start_date"`
	EndDate             *string    `json:"end_date"`
	JobTitle            *string    `json:"job_title"`
	EmploymentType      string     `json:"employment_type"`
	ManagerEmploymentID *uuid.UUID `json:"manager_employment_id"`
}

type UpdateEmploymentRequest struct {
	EmployerID                 *uuid.UUID  `json:"employer_id"`
	PrimaryLocationID          **uuid.UUID `json:"primary_location_id"`
	DepartmentID               **uuid.UUID `json:"department_id"`
	Status                     *string     `json:"status"`
	StartDate                  *string     `json:"start_date"`
	EndDate                    **string    `json:"end_date"`
	JobTitle                   *string     `json:"job_title"`
	EmploymentType             *string     `json:"employment_type"`
	ManagerEmploymentID        **uuid.UUID `json:"manager_employment_id"`
	TerminationReason          *string     `json:"termination_reason"`
	Notes                      *string     `json:"notes"`
	PrimaryLocationIDPresent   bool        `json:"-"`
	DepartmentIDPresent        bool        `json:"-"`
	EndDatePresent             bool        `json:"-"`
	ManagerEmploymentIDPresent bool        `json:"-"`
}

func (r *UpdateEmploymentRequest) UnmarshalJSON(data []byte) error {
	type plain UpdateEmploymentRequest
	var value plain
	if err := json.Unmarshal(data, &value); err != nil {
		return err
	}
	var fields map[string]json.RawMessage
	if err := json.Unmarshal(data, &fields); err != nil {
		return err
	}
	allowed := map[string]bool{"employer_id": true, "primary_location_id": true, "department_id": true, "status": true, "start_date": true, "end_date": true, "job_title": true, "employment_type": true, "manager_employment_id": true, "termination_reason": true, "notes": true}
	for key := range fields {
		if !allowed[key] {
			return fmt.Errorf("json: unknown field %q", key)
		}
	}
	*r = UpdateEmploymentRequest(value)
	_, r.PrimaryLocationIDPresent = fields["primary_location_id"]
	_, r.DepartmentIDPresent = fields["department_id"]
	_, r.EndDatePresent = fields["end_date"]
	_, r.ManagerEmploymentIDPresent = fields["manager_employment_id"]
	return nil
}

type ContractRepository interface {
	ListContracts(context.Context) ([]EmployeeContractWithEmployee, error)
	ListEmployeeContracts(context.Context, uuid.UUID) ([]EmployeeContract, error)
	CreateContract(context.Context, CreateContractRequest) (EmployeeContract, error)
	GetContract(context.Context, uuid.UUID) (EmployeeContract, error)
	UpdateContract(context.Context, uuid.UUID, UpdateContractRequest) (EmployeeContract, error)
	SoftDeleteContract(context.Context, uuid.UUID) error
}

type EmploymentRepository interface {
	ListEmployments(context.Context, uuid.UUID) ([]Employment, error)
	CreateEmployment(context.Context, CreateEmploymentRequest) (Employment, error)
	GetEmployment(context.Context, uuid.UUID) (Employment, error)
	UpdateEmployment(context.Context, uuid.UUID, UpdateEmploymentRequest) (Employment, error)
	SoftDeleteEmployment(context.Context, uuid.UUID) error
}
type ContractEmploymentHandler struct {
	Contracts    ContractRepository
	Employments  EmploymentRepository
	CustomFields CustomFieldRepository
	Authorizer   Authorizer
}

func validContractType(v string) bool {
	switch v {
	case ContractTypeProbation, ContractTypeTraining, ContractTypeFixedTerm, ContractTypeIndefinite:
		return true
	}
	return false
}
func validContractStatus(v string) bool {
	switch v {
	case ContractStatusDraft, ContractStatusActive, ContractStatusExpired, ContractStatusTerminated:
		return true
	}
	return false
}
func validContractEmploymentDate(v string) bool {
	_, err := time.Parse("2006-01-02", v)
	return err == nil
}
func dateBefore(a, b string) bool { return a < b }

func (h ContractEmploymentHandler) member(r *http.Request) bool {
	uid, ok := httpinfra.UserIDFromContext(r.Context())
	if !ok || h.Authorizer == nil {
		return false
	}
	member, err := h.Authorizer.IsMember(r.Context(), uid)
	return err == nil && member
}
func (h ContractEmploymentHandler) auth(w http.ResponseWriter, r *http.Request) bool {
	if h.member(r) {
		return true
	}
	if _, ok := httpinfra.UserIDFromContext(r.Context()); !ok {
		apperror.WriteError(w, r, apperror.Unauthorized(apperror.KeyAuthenticationRequired))
	} else {
		apperror.WriteError(w, r, apperror.Forbidden(apperror.KeyNotOrganizationMember))
	}
	return false
}
func (h ContractEmploymentHandler) error(w http.ResponseWriter, r *http.Request, err error) {
	switch {
	case errors.Is(err, ErrContractNotFound):
		apperror.WriteError(w, r, apperror.NotFound("error.contract_not_found"))
	case errors.Is(err, ErrEmploymentNotFound):
		apperror.WriteError(w, r, apperror.NotFound(apperror.KeyEmploymentNotFound))
	case errors.Is(err, ErrContractOverlap), errors.Is(err, ErrEmploymentOverlap):
		apperror.WriteError(w, r, apperror.Conflict(apperror.KeyConflict))
	case errors.Is(err, pgx.ErrNoRows):
		apperror.WriteError(w, r, apperror.NotFound(apperror.KeyNotFound))
	default:
		apperror.WriteError(w, r, apperror.Internal(err))
	}
}
func (h ContractEmploymentHandler) path(w http.ResponseWriter, r *http.Request, names ...string) ([]uuid.UUID, bool) {
	if !h.auth(w, r) {
		return nil, false
	}
	ids := make([]uuid.UUID, len(names))
	for i, name := range names {
		id, ok := pathUUID(r, name)
		if !ok {
			apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
			return nil, false
		}
		ids[i] = id
	}
	return ids, true
}

func (h ContractEmploymentHandler) ListContracts(w http.ResponseWriter, r *http.Request) {
	if !h.auth(w, r) {
		return
	}
	result, err := h.Contracts.ListContracts(r.Context())
	if err != nil {
		h.error(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusOK, result)
}
func (h ContractEmploymentHandler) ListEmployeeContracts(w http.ResponseWriter, r *http.Request) {
	ids, ok := h.path(w, r, "employee_id")
	if !ok {
		return
	}
	result, err := h.Contracts.ListEmployeeContracts(r.Context(), ids[0])
	if err != nil {
		h.error(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusOK, result)
}
func (h ContractEmploymentHandler) CreateContract(w http.ResponseWriter, r *http.Request) {
	if _, ok := h.path(w, r, "employee_id"); !ok {
		return
	}
	req, err := DecodeJSON[CreateContractRequest](r, 1<<20)
	if err != nil || req.EmploymentID == uuid.Nil || !validContractType(req.ContractType) || !validContractEmploymentDate(req.StartDate) || (req.EndDate != nil && !validContractEmploymentDate(*req.EndDate)) || (req.EndDate != nil && dateBefore(*req.EndDate, req.StartDate)) {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	result, err := h.Contracts.CreateContract(r.Context(), req)
	if err != nil {
		h.error(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusCreated, result)
}
func (h ContractEmploymentHandler) GetContract(w http.ResponseWriter, r *http.Request) {
	ids, ok := h.path(w, r, "employee_id", "contract_id")
	if !ok {
		return
	}
	result, err := h.Contracts.GetContract(r.Context(), ids[1])
	if err != nil {
		h.error(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusOK, result)
}
func (h ContractEmploymentHandler) UpdateContract(w http.ResponseWriter, r *http.Request) {
	ids, ok := h.path(w, r, "employee_id", "contract_id")
	if !ok {
		return
	}
	req, err := DecodeJSON[UpdateContractRequest](r, 1<<20)
	if err != nil || (req.ContractType != nil && !validContractType(*req.ContractType)) || (req.Status != nil && !validContractStatus(*req.Status)) || (req.StartDate != nil && !validContractEmploymentDate(*req.StartDate)) || (req.EndDatePresent && req.EndDate != nil && *req.EndDate != nil && !validContractEmploymentDate(**req.EndDate)) {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	result, err := h.Contracts.UpdateContract(r.Context(), ids[1], req)
	if err != nil {
		h.error(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusOK, result)
}
func (h ContractEmploymentHandler) DeleteContract(w http.ResponseWriter, r *http.Request) {
	ids, ok := h.path(w, r, "employee_id", "contract_id")
	if !ok {
		return
	}
	if err := h.Contracts.SoftDeleteContract(r.Context(), ids[1]); err != nil {
		h.error(w, r, err)
		return
	}
	w.WriteHeader(http.StatusNoContent)
}

func (h ContractEmploymentHandler) addCustomFields(ctx context.Context, emp Employment) (Employment, error) {
	if h.CustomFields == nil {
		return emp, nil
	}
	values, err := h.CustomFields.GetValues(ctx, emp.ID)
	if err != nil {
		return emp, err
	}
	emp.CustomFields = values
	return emp, nil
}
func (h ContractEmploymentHandler) ListEmployments(w http.ResponseWriter, r *http.Request) {
	ids, ok := h.path(w, r, "employee_id")
	if !ok {
		return
	}
	list, err := h.Employments.ListEmployments(r.Context(), ids[0])
	if err != nil {
		h.error(w, r, err)
		return
	}
	for i := range list {
		list[i], err = h.addCustomFields(r.Context(), list[i])
		if err != nil {
			h.error(w, r, err)
			return
		}
	}
	if list == nil {
		list = []Employment{}
	}
	apperror.WriteJSON(w, http.StatusOK, list)
}
func (h ContractEmploymentHandler) CreateEmployment(w http.ResponseWriter, r *http.Request) {
	if _, ok := h.path(w, r, "employee_id"); !ok {
		return
	}
	req, err := DecodeJSON[CreateEmploymentRequest](r, 1<<20)
	if err != nil || req.EmployeeID == uuid.Nil || req.EmployerID == uuid.Nil || strings.TrimSpace(req.Status) == "" || strings.TrimSpace(req.EmploymentType) == "" || !validContractEmploymentDate(req.StartDate) || (req.EndDate != nil && (!validContractEmploymentDate(*req.EndDate) || dateBefore(*req.EndDate, req.StartDate))) {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	existing, err := h.Employments.ListEmployments(r.Context(), req.EmployeeID)
	if err != nil {
		h.error(w, r, err)
		return
	}
	for _, item := range existing {
		if item.EmployerID == req.EmployerID && rangesOverlap(req.StartDate, req.EndDate, item.StartDate, item.EndDate) {
			h.error(w, r, ErrEmploymentOverlap)
			return
		}
	}
	result, err := h.Employments.CreateEmployment(r.Context(), req)
	if err != nil {
		h.error(w, r, err)
		return
	}
	result, err = h.addCustomFields(r.Context(), result)
	if err != nil {
		h.error(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusCreated, result)
}
func (h ContractEmploymentHandler) GetEmployment(w http.ResponseWriter, r *http.Request) {
	ids, ok := h.path(w, r, "employee_id", "employment_id")
	if !ok {
		return
	}
	result, err := h.Employments.GetEmployment(r.Context(), ids[1])
	if err != nil {
		h.error(w, r, err)
		return
	}
	result, err = h.addCustomFields(r.Context(), result)
	if err != nil {
		h.error(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusOK, result)
}
func (h ContractEmploymentHandler) UpdateEmployment(w http.ResponseWriter, r *http.Request) {
	ids, ok := h.path(w, r, "employee_id", "employment_id")
	if !ok {
		return
	}
	req, err := DecodeJSON[UpdateEmploymentRequest](r, 1<<20)
	if err != nil || (req.StartDate != nil && !validContractEmploymentDate(*req.StartDate)) || (req.EndDatePresent && req.EndDate != nil && *req.EndDate != nil && !validContractEmploymentDate(**req.EndDate)) {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	current, err := h.Employments.GetEmployment(r.Context(), ids[1])
	if err != nil {
		h.error(w, r, err)
		return
	}
	targetEmployer := current.EmployerID
	if req.EmployerID != nil {
		targetEmployer = *req.EmployerID
	}
	targetStart := current.StartDate
	if req.StartDate != nil {
		targetStart = *req.StartDate
	}
	targetEnd := current.EndDate
	if req.EndDatePresent {
		targetEnd = nil
		if req.EndDate != nil {
			targetEnd = *req.EndDate
		}
	}
	existing, err := h.Employments.ListEmployments(r.Context(), current.EmployeeID)
	if err != nil {
		h.error(w, r, err)
		return
	}
	for _, item := range existing {
		if item.ID != current.ID && item.EmployerID == targetEmployer && rangesOverlap(targetStart, targetEnd, item.StartDate, item.EndDate) {
			h.error(w, r, ErrEmploymentOverlap)
			return
		}
	}
	result, err := h.Employments.UpdateEmployment(r.Context(), ids[1], req)
	if err != nil {
		h.error(w, r, err)
		return
	}
	result, err = h.addCustomFields(r.Context(), result)
	if err != nil {
		h.error(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusOK, result)
}
func (h ContractEmploymentHandler) DeleteEmployment(w http.ResponseWriter, r *http.Request) {
	ids, ok := h.path(w, r, "employee_id", "employment_id")
	if !ok {
		return
	}
	if err := h.Employments.SoftDeleteEmployment(r.Context(), ids[1]); err != nil {
		h.error(w, r, err)
		return
	}
	w.WriteHeader(http.StatusNoContent)
}

func rangesOverlap(s1 string, e1 *string, s2 string, e2 *string) bool {
	if e1 != nil && *e1 < s2 || e2 != nil && *e2 < s1 {
		return false
	}
	return true
}
