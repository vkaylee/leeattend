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
	"github.com/vkaylee/leeattend/apiserver/internal/apperror"
	authz "github.com/vkaylee/leeattend/apiserver/internal/auth"
	httpinfra "github.com/vkaylee/leeattend/apiserver/internal/infra/http"
)

var (
	ErrContractNotFound               = errors.New("contract: not found")
	ErrContractOverlap                = errors.New("contract: overlapping active contract")
	ErrEmploymentOverlap              = errors.New("employment: overlapping employment")
	ErrEmploymentRelationshipNotFound = errors.New("employment relationship: not found")
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

type EmploymentRelationship struct {
	ID                uuid.UUID `json:"id"`
	EmployeeID        uuid.UUID `json:"employee_id"`
	EmployerID        uuid.UUID `json:"employer_id"`
	TenureID          uuid.UUID `json:"tenure_id"`
	Status            string    `json:"status"`
	StartDate         string    `json:"start_date"`
	EndDate           *string   `json:"end_date"`
	TerminationReason *string   `json:"termination_reason,omitempty"`
	Notes             *string   `json:"notes,omitempty"`
	CreatedAt         string    `json:"created_at"`
	UpdatedAt         string    `json:"updated_at"`
}

type EmploymentRelationshipListResponse struct {
	Data  []EmploymentRelationship `json:"data"`
	Total int64                    `json:"total"`
	Page  int                      `json:"page"`
	Limit int                      `json:"limit"`
}

type CreateEmploymentRelationshipRequest struct {
	EmployerID          uuid.UUID  `json:"employer_id"`
	TenureID            *uuid.UUID `json:"tenure_id,omitempty"`
	Status              string     `json:"status"`
	StartDate           string     `json:"start_date"`
	EndDate             *string    `json:"end_date"`
	TerminationReason   *string    `json:"termination_reason"`
	Notes               *string    `json:"notes"`
	PrimaryLocationID   *uuid.UUID `json:"primary_location_id"`
	DepartmentID        *uuid.UUID `json:"department_id"`
	JobTitle            *string    `json:"job_title"`
	EmploymentType      string     `json:"employment_type"`
	ManagerEmploymentID *uuid.UUID `json:"manager_employment_id"`
}

type EmploymentRelationshipCreated struct {
	Relationship EmploymentRelationship `json:"relationship"`
	Segment      Employment             `json:"segment"`
}

type CreateEmploymentRequest struct {
	EmployeeID          uuid.UUID  `json:"employee_id"`
	EmployerID          uuid.UUID  `json:"employer_id"`
	RelationshipID      *uuid.UUID `json:"relationship_id,omitempty"`
	PrimaryLocationID   *uuid.UUID `json:"primary_location_id"`
	DepartmentID        *uuid.UUID `json:"department_id"`
	Status              string     `json:"status"`
	StartDate           string     `json:"start_date"`
	EndDate             *string    `json:"end_date"`
	JobTitle            *string    `json:"job_title"`
	EmploymentType      string     `json:"employment_type"`
	ManagerEmploymentID *uuid.UUID `json:"manager_employment_id"`
}

type TerminateEmploymentRequest struct {
	TerminationDate   string  `json:"termination_date"`
	TerminationReason *string `json:"termination_reason,omitempty"`
	RehireEligible    *bool   `json:"rehire_eligible,omitempty"`
	Notes             *string `json:"notes,omitempty"`
}

type TerminateEmploymentResult struct {
	Employment         Employment `json:"employment"`
	RelationshipClosed bool       `json:"relationship_closed"`
	TenureClosed       bool       `json:"tenure_closed"`
	SessionsRevoked    bool       `json:"sessions_revoked"`
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
	ListEmploymentRelationships(context.Context, uuid.UUID, int, int, *uuid.UUID, *time.Time) (EmploymentRelationshipListResponse, error)
	GetEmploymentRelationship(context.Context, uuid.UUID) (EmploymentRelationship, error)
	CreateEmploymentRelationship(context.Context, uuid.UUID, CreateEmploymentRelationshipRequest) (EmploymentRelationshipCreated, error)
	CreateEmployment(context.Context, CreateEmploymentRequest) (Employment, error)
	GetEmployment(context.Context, uuid.UUID) (Employment, error)
	UpdateEmployment(context.Context, uuid.UUID, UpdateEmploymentRequest) (Employment, error)
	TerminateEmployment(context.Context, uuid.UUID, uuid.UUID, TerminateEmploymentRequest) (TerminateEmploymentResult, error)
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

func (h *ContractEmploymentHandler) member(r *http.Request) bool {
	uid, ok := httpinfra.UserIDFromContext(r.Context())
	if !ok || h.Authorizer == nil {
		return false
	}
	member, err := h.Authorizer.IsMember(r.Context(), uid)
	return err == nil && member
}
func (h *ContractEmploymentHandler) auth(w http.ResponseWriter, r *http.Request) bool {
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

func (h *ContractEmploymentHandler) permission(w http.ResponseWriter, r *http.Request, required string) bool {
	userID, ok := httpinfra.UserIDFromContext(r.Context())
	if !ok {
		apperror.WriteError(w, r, apperror.Unauthorized(apperror.KeyAuthenticationRequired))
		return false
	}
	if h.Authorizer == nil || !h.Authorizer.HasPermission(r.Context(), userID, required) {
		apperror.WriteError(w, r, apperror.Forbidden(apperror.KeyForbidden))
		return false
	}
	return true
}

func (h *ContractEmploymentHandler) canReadEmployee(w http.ResponseWriter, r *http.Request) bool {
	return h.permission(w, r, string(authz.PermissionEmployeesRead))
}

func (h *ContractEmploymentHandler) canManageEmployee(w http.ResponseWriter, r *http.Request) bool {
	return h.permission(w, r, string(authz.PermissionEmployeesUpdate))
}

func (h *ContractEmploymentHandler) canCreateEmployee(w http.ResponseWriter, r *http.Request) bool {
	return h.permission(w, r, string(authz.PermissionEmployeesCreate))
}

func (h *ContractEmploymentHandler) canDeleteEmployee(w http.ResponseWriter, r *http.Request) bool {
	return h.permission(w, r, string(authz.PermissionEmployeesDelete))
}

func (h *ContractEmploymentHandler) canReadContract(w http.ResponseWriter, r *http.Request) bool {
	return h.permission(w, r, string(authz.PermissionDocumentsContractRead))
}

func (h *ContractEmploymentHandler) canManageContract(w http.ResponseWriter, r *http.Request) bool {
	return h.permission(w, r, string(authz.PermissionDocumentsManage))
}

func (h *ContractEmploymentHandler) employeePermission(w http.ResponseWriter, r *http.Request, manage bool) bool {
	if manage {
		return h.canManageEmployee(w, r)
	}
	return h.canReadEmployee(w, r)
}
func (h *ContractEmploymentHandler) error(w http.ResponseWriter, r *http.Request, err error) {
	switch {
	case errors.Is(err, ErrContractNotFound):
		apperror.WriteError(w, r, apperror.NotFound("error.contract_not_found"))
	case errors.Is(err, ErrEmploymentNotFound), errors.Is(err, ErrEmploymentRelationshipNotFound):
		apperror.WriteError(w, r, apperror.NotFound(apperror.KeyEmploymentNotFound))
	case errors.Is(err, ErrEmployeeNotFound), errors.Is(err, ErrEmployerNotFound):
		apperror.WriteError(w, r, apperror.NotFound(apperror.KeyNotFound))
	case errors.Is(err, ErrContractOverlap), errors.Is(err, ErrEmploymentOverlap), errors.Is(err, ErrConflict):
		apperror.WriteError(w, r, apperror.Conflict(apperror.KeyConflict))
	case errors.Is(err, ErrInvalidInput):
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
	case errors.Is(err, ErrNotFound):
		apperror.WriteError(w, r, apperror.NotFound(apperror.KeyNotFound))
	default:
		apperror.WriteError(w, r, apperror.Internal(err))
	}
}
func (h *ContractEmploymentHandler) path(w http.ResponseWriter, r *http.Request, names ...string) ([]uuid.UUID, bool) {
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

func (h *ContractEmploymentHandler) ListContracts(w http.ResponseWriter, r *http.Request) {
	if !h.auth(w, r) || !h.canReadContract(w, r) {
		return
	}
	result, err := h.Contracts.ListContracts(r.Context())
	if err != nil {
		h.error(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusOK, result)
}
func (h *ContractEmploymentHandler) ListEmployeeContracts(w http.ResponseWriter, r *http.Request) {
	if !h.canReadContract(w, r) {
		return
	}
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
func (h *ContractEmploymentHandler) contractForEmployee(ctx context.Context, employeeID, contractID uuid.UUID) (EmployeeContract, error) {
	contract, err := h.Contracts.GetContract(ctx, contractID)
	if err != nil {
		return EmployeeContract{}, err
	}
	employment, err := h.Employments.GetEmployment(ctx, contract.EmploymentID)
	if err != nil {
		if errors.Is(err, ErrEmploymentNotFound) {
			return EmployeeContract{}, ErrContractNotFound
		}
		return EmployeeContract{}, err
	}
	if employment.EmployeeID != employeeID {
		return EmployeeContract{}, ErrContractNotFound
	}
	return contract, nil
}
func (h *ContractEmploymentHandler) CreateContract(w http.ResponseWriter, r *http.Request) {
	if !h.canManageContract(w, r) {
		return
	}
	ids, ok := h.path(w, r, "employee_id")
	if !ok {
		return
	}
	req, err := DecodeJSON[CreateContractRequest](r, 1<<20)
	if err != nil || req.EmploymentID == uuid.Nil || !validContractType(req.ContractType) || !validContractEmploymentDate(req.StartDate) || (req.EndDate != nil && !validContractEmploymentDate(*req.EndDate)) || (req.EndDate != nil && dateBefore(*req.EndDate, req.StartDate)) {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	employment, err := h.Employments.GetEmployment(r.Context(), req.EmploymentID)
	if err != nil || employment.EmployeeID != ids[0] {
		if err != nil {
			h.error(w, r, err)
		} else {
			h.error(w, r, ErrContractNotFound)
		}
		return
	}
	result, err := h.Contracts.CreateContract(r.Context(), req)
	if err != nil {
		h.error(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusCreated, result)
}
func (h *ContractEmploymentHandler) GetContract(w http.ResponseWriter, r *http.Request) {
	if !h.canReadContract(w, r) {
		return
	}
	ids, ok := h.path(w, r, "employee_id", "contract_id")
	if !ok {
		return
	}
	result, err := h.contractForEmployee(r.Context(), ids[0], ids[1])
	if err != nil {
		h.error(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusOK, result)
}
func (h *ContractEmploymentHandler) UpdateContract(w http.ResponseWriter, r *http.Request) {
	if !h.canManageContract(w, r) {
		return
	}
	ids, ok := h.path(w, r, "employee_id", "contract_id")
	if !ok {
		return
	}
	if _, err := h.contractForEmployee(r.Context(), ids[0], ids[1]); err != nil {
		h.error(w, r, err)
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
func (h *ContractEmploymentHandler) DeleteContract(w http.ResponseWriter, r *http.Request) {
	if !h.canManageContract(w, r) {
		return
	}
	ids, ok := h.path(w, r, "employee_id", "contract_id")
	if !ok {
		return
	}
	if _, err := h.contractForEmployee(r.Context(), ids[0], ids[1]); err != nil {
		h.error(w, r, err)
		return
	}
	if err := h.Contracts.SoftDeleteContract(r.Context(), ids[1]); err != nil {
		h.error(w, r, err)
		return
	}
	w.WriteHeader(http.StatusNoContent)
}

func (h *ContractEmploymentHandler) addCustomFields(ctx context.Context, emp Employment) (Employment, error) {
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
func (h *ContractEmploymentHandler) CreateEmploymentRelationship(w http.ResponseWriter, r *http.Request) {
	if !h.canCreateEmployee(w, r) {
		return
	}
	ids, ok := h.path(w, r, "employee_id")
	if !ok {
		return
	}
	req, err := DecodeJSON[CreateEmploymentRelationshipRequest](r, 1<<20)
	if err != nil || req.EmployerID == uuid.Nil || strings.TrimSpace(req.Status) == "" || strings.TrimSpace(req.EmploymentType) == "" || !validContractEmploymentDate(req.StartDate) || (req.EndDate != nil && (!validContractEmploymentDate(*req.EndDate) || dateBefore(*req.EndDate, req.StartDate))) {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	result, err := h.Employments.CreateEmploymentRelationship(r.Context(), ids[0], req)
	if err != nil {
		h.error(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusCreated, result)
}

func (h *ContractEmploymentHandler) ListEmploymentRelationships(w http.ResponseWriter, r *http.Request) {
	if !h.employeePermission(w, r, false) {
		return
	}
	ids, ok := h.path(w, r, "employee_id")
	if !ok {
		return
	}
	pagination, err := ParsePagination(r.URL.Query())
	if err != nil {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	var employerID *uuid.UUID
	if raw := r.URL.Query().Get("employer_id"); raw != "" {
		value, parseErr := uuid.Parse(raw)
		if parseErr != nil {
			apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
			return
		}
		employerID = &value
	}
	var asOf *time.Time
	if raw := r.URL.Query().Get("as_of"); raw != "" {
		value, parseErr := time.Parse("2006-01-02", raw)
		if parseErr != nil {
			apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
			return
		}
		asOf = &value
	}
	result, err := h.Employments.ListEmploymentRelationships(r.Context(), ids[0], pagination.Page, pagination.Limit, employerID, asOf)
	if err != nil {
		h.error(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusOK, result)
}

func (h *ContractEmploymentHandler) GetEmploymentRelationship(w http.ResponseWriter, r *http.Request) {
	if !h.employeePermission(w, r, false) {
		return
	}
	ids, ok := h.path(w, r, "employee_id", "relationship_id")
	if !ok {
		return
	}
	relationship, err := h.Employments.GetEmploymentRelationship(r.Context(), ids[1])
	if err != nil {
		h.error(w, r, err)
		return
	}
	if relationship.EmployeeID != ids[0] {
		h.error(w, r, ErrEmploymentNotFound)
		return
	}
	apperror.WriteJSON(w, http.StatusOK, relationship)
}

func (h *ContractEmploymentHandler) ListEmployments(w http.ResponseWriter, r *http.Request) {
	if !h.employeePermission(w, r, false) {
		return
	}
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
func (h *ContractEmploymentHandler) CreateEmployment(w http.ResponseWriter, r *http.Request) {
	if !h.canCreateEmployee(w, r) {
		return
	}
	ids, ok := h.path(w, r, "employee_id")
	if !ok {
		return
	}
	req, err := DecodeJSON[CreateEmploymentRequest](r, 1<<20)
	if err != nil || req.EmployeeID == uuid.Nil || req.EmployerID == uuid.Nil || strings.TrimSpace(req.Status) == "" || strings.TrimSpace(req.EmploymentType) == "" || !validContractEmploymentDate(req.StartDate) || (req.EndDate != nil && (!validContractEmploymentDate(*req.EndDate) || dateBefore(*req.EndDate, req.StartDate))) {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	if req.EmployeeID != ids[0] {
		h.error(w, r, ErrEmploymentNotFound)
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
func (h *ContractEmploymentHandler) employmentForEmployee(ctx context.Context, employeeID, employmentID uuid.UUID) (Employment, error) {
	employment, err := h.Employments.GetEmployment(ctx, employmentID)
	if err != nil {
		return Employment{}, err
	}
	if employment.EmployeeID != employeeID {
		return Employment{}, ErrEmploymentNotFound
	}
	return employment, nil
}
func (h *ContractEmploymentHandler) GetEmployment(w http.ResponseWriter, r *http.Request) {
	if !h.employeePermission(w, r, false) {
		return
	}
	ids, ok := h.path(w, r, "employee_id", "employment_id")
	if !ok {
		return
	}
	result, err := h.employmentForEmployee(r.Context(), ids[0], ids[1])
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
func (h *ContractEmploymentHandler) UpdateEmployment(w http.ResponseWriter, r *http.Request) {
	if !h.employeePermission(w, r, true) {
		return
	}
	ids, ok := h.path(w, r, "employee_id", "employment_id")
	if !ok {
		return
	}
	req, err := DecodeJSON[UpdateEmploymentRequest](r, 1<<20)
	if err != nil || (req.StartDate != nil && !validContractEmploymentDate(*req.StartDate)) || (req.EndDatePresent && req.EndDate != nil && *req.EndDate != nil && !validContractEmploymentDate(**req.EndDate)) {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	if req.Status != nil && strings.EqualFold(strings.TrimSpace(*req.Status), "terminated") {
		apperror.WriteError(w, r, apperror.Conflict(apperror.KeyConflict))
		return
	}
	current, err := h.employmentForEmployee(r.Context(), ids[0], ids[1])
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
func (h *ContractEmploymentHandler) TerminateEmployment(w http.ResponseWriter, r *http.Request) {
	if !h.employeePermission(w, r, true) {
		return
	}
	ids, ok := h.path(w, r, "employee_id", "employment_id")
	if !ok {
		return
	}
	req, err := DecodeJSON[TerminateEmploymentRequest](r, 1<<20)
	if err != nil || !validContractEmploymentDate(req.TerminationDate) {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	result, err := h.Employments.TerminateEmployment(r.Context(), ids[0], ids[1], req)
	if err != nil {
		h.error(w, r, err)
		return
	}
	result.Employment, err = h.addCustomFields(r.Context(), result.Employment)
	if err != nil {
		h.error(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusOK, result)
}

func (h *ContractEmploymentHandler) DeleteEmployment(w http.ResponseWriter, r *http.Request) {
	if !h.canDeleteEmployee(w, r) {
		return
	}
	ids, ok := h.path(w, r, "employee_id", "employment_id")
	if !ok {
		return
	}
	if _, err := h.employmentForEmployee(r.Context(), ids[0], ids[1]); err != nil {
		h.error(w, r, err)
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
