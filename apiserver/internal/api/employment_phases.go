package api

import (
	"context"
	"encoding/json"
	"errors"
	"net/http"
	"strings"
	"time"

	"github.com/google/uuid"
	"github.com/vkaylee/leeattend/apiserver/internal/apperror"
	httpinfra "github.com/vkaylee/leeattend/apiserver/internal/infra/http"
)

var (
	ErrEmploymentPhaseDefinitionNotFound = errors.New("employment phase definition: not found")
	ErrEmployeePhaseNotFound             = errors.New("employee phase: not found")
)

type EmploymentPhaseDefinition struct {
	ID           uuid.UUID       `json:"id"`
	Name         string          `json:"name"`
	Description  *string         `json:"description"`
	PhaseType    string          `json:"phase_type"`
	RuleConfig   json.RawMessage `json:"rule_config"`
	IsActive     bool            `json:"is_active"`
	DisplayOrder int             `json:"display_order"`
	CreatedAt    string          `json:"created_at"`
	UpdatedAt    string          `json:"updated_at"`
}

type EmployeePhase struct {
	ID                uuid.UUID       `json:"id"`
	EmploymentID      uuid.UUID       `json:"employment_id"`
	PhaseDefinitionID uuid.UUID       `json:"phase_definition_id"`
	StartDate         string          `json:"start_date"`
	EndDate           string          `json:"end_date"`
	Status            string          `json:"status"`
	OverrideConfig    json.RawMessage `json:"override_config"`
	EvaluationNotes   *string         `json:"evaluation_notes"`
	EvaluatedBy       *uuid.UUID      `json:"evaluated_by"`
	CompletedAt       *string         `json:"completed_at"`
	CreatedAt         string          `json:"created_at"`
	UpdatedAt         string          `json:"updated_at"`
}

type EmploymentPhaseAssignment struct {
	EmployeePhase
	EmployeeID     uuid.UUID `json:"employee_id"`
	EmployeeName   string    `json:"employee_name"`
	EmployeeCode   string    `json:"employee_code"`
	DepartmentName *string   `json:"department_name,omitempty"`
	PhaseName      string    `json:"phase_name"`
	PhaseType      string    `json:"phase_type"`
}

type EmploymentPhaseAssignmentListResponse struct {
	Data  []EmploymentPhaseAssignment `json:"data"`
	Total int64                       `json:"total"`
	Page  int                         `json:"page"`
	Limit int                         `json:"limit"`
}

type CreateEmploymentPhaseDefinitionRequest struct {
	Name         string          `json:"name"`
	Description  *string         `json:"description"`
	PhaseType    string          `json:"phase_type"`
	RuleConfig   json.RawMessage `json:"rule_config"`
	DisplayOrder *int            `json:"display_order"`
}

type AssignEmployeePhaseRequest struct {
	PhaseDefinitionID uuid.UUID       `json:"phase_definition_id"`
	StartDate         string          `json:"start_date"`
	EndDate           string          `json:"end_date"`
	OverrideConfig    json.RawMessage `json:"override_config"`
}

type UpdateEmployeePhaseStatusRequest struct {
	Status          string  `json:"status"`
	EvaluationNotes *string `json:"evaluation_notes"`
}

type EmploymentPhaseRepository interface {
	CreateDefinition(context.Context, EmploymentPhaseDefinition) (EmploymentPhaseDefinition, error)
	ListDefinitions(context.Context) ([]EmploymentPhaseDefinition, error)
	ListOrganizationPhases(context.Context, int, int, string, string, *uuid.UUID) (EmploymentPhaseAssignmentListResponse, error)
	AssignPhase(context.Context, uuid.UUID, AssignEmployeePhaseRequest) (EmployeePhase, error)
	UpdatePhaseStatus(context.Context, uuid.UUID, uuid.UUID, UpdateEmployeePhaseStatusRequest, uuid.UUID) (EmployeePhase, error)
	ListPhases(context.Context, uuid.UUID) ([]EmployeePhase, error)
	ResolveActiveEmployment(context.Context, uuid.UUID) (uuid.UUID, error)
}

type EmploymentPhaseHandler struct {
	Repository EmploymentPhaseRepository
	Authorizer Authorizer
}

func (h EmploymentPhaseHandler) allowed(r *http.Request, permission string) bool {
	uid, ok := httpinfra.UserIDFromContext(r.Context())
	return ok && h.Authorizer != nil && h.Authorizer.HasPermission(r.Context(), uid, permission)
}
func (h EmploymentPhaseHandler) deny(w http.ResponseWriter, r *http.Request) {
	apperror.WriteError(w, r, apperror.Forbidden(apperror.KeyForbidden))
}
func (h EmploymentPhaseHandler) err(w http.ResponseWriter, r *http.Request, err error) {
	switch {
	case errors.Is(err, ErrEmployeeNotFound), errors.Is(err, ErrEmployeePhaseNotFound), errors.Is(err, ErrEmploymentPhaseDefinitionNotFound):
		apperror.WriteError(w, r, apperror.NotFound(apperror.KeyNotFound))
	case errors.Is(err, ErrNoActiveEmployment):
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
	default:
		apperror.WriteError(w, r, apperror.Internal(err))
	}
}
func validPhaseDate(s string) bool { _, err := time.Parse("2006-01-02", s); return err == nil }

func validPhaseDateRange(startDate, endDate string) bool {
	start, startErr := time.Parse("2006-01-02", startDate)
	end, endErr := time.Parse("2006-01-02", endDate)
	return startErr == nil && endErr == nil && !end.Before(start)
}

func (h EmploymentPhaseHandler) CreateDefinition(w http.ResponseWriter, r *http.Request) {
	if !h.allowed(r, "employment_phases:manage") {
		h.deny(w, r)
		return
	}
	var req CreateEmploymentPhaseDefinitionRequest
	if json.NewDecoder(r.Body).Decode(&req) != nil || strings.TrimSpace(req.Name) == "" || req.PhaseType == "" || len(req.RuleConfig) == 0 || !json.Valid(req.RuleConfig) {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	display := 0
	if req.DisplayOrder != nil {
		display = *req.DisplayOrder
	}
	now := time.Now().UTC().Format(time.RFC3339)
	def := EmploymentPhaseDefinition{ID: uuid.New(), Name: req.Name, Description: req.Description, PhaseType: req.PhaseType, RuleConfig: req.RuleConfig, IsActive: true, DisplayOrder: display, CreatedAt: now, UpdatedAt: now}
	if h.Repository == nil {
		apperror.WriteError(w, r, apperror.Internal(errors.New("repository unavailable")))
		return
	}
	out, err := h.Repository.CreateDefinition(r.Context(), def)
	if err != nil {
		h.err(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusCreated, out)
}
func (h EmploymentPhaseHandler) ListDefinitions(w http.ResponseWriter, r *http.Request) {
	if !h.allowed(r, "employment_phases:read") {
		h.deny(w, r)
		return
	}
	if h.Repository == nil {
		apperror.WriteJSON(w, http.StatusOK, []EmploymentPhaseDefinition{})
		return
	}
	out, err := h.Repository.ListDefinitions(r.Context())
	if err != nil {
		h.err(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusOK, out)
}
func (h EmploymentPhaseHandler) ListOrganization(w http.ResponseWriter, r *http.Request) {
	if !h.allowed(r, "employment_phases:read") {
		h.deny(w, r)
		return
	}
	pagination, err := ParsePagination(r.URL.Query())
	if err != nil {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	status := strings.TrimSpace(r.URL.Query().Get("status"))
	query := strings.TrimSpace(r.URL.Query().Get("q"))
	var employeeID *uuid.UUID
	if rawEmployeeID := strings.TrimSpace(r.URL.Query().Get("employee_id")); rawEmployeeID != "" {
		parsedEmployeeID, parseErr := uuid.Parse(rawEmployeeID)
		if parseErr != nil {
			apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
			return
		}
		employeeID = &parsedEmployeeID
	}
	if h.Repository == nil {
		apperror.WriteJSON(w, http.StatusOK, EmploymentPhaseAssignmentListResponse{Data: []EmploymentPhaseAssignment{}, Page: pagination.Page, Limit: pagination.Limit})
		return
	}
	result, err := h.Repository.ListOrganizationPhases(r.Context(), pagination.Page, pagination.Limit, query, status, employeeID)
	if err != nil {
		h.err(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusOK, result)
}

func (h EmploymentPhaseHandler) assign(w http.ResponseWriter, r *http.Request, employmentID uuid.UUID) {
	var req AssignEmployeePhaseRequest
	if json.NewDecoder(r.Body).Decode(&req) != nil || req.PhaseDefinitionID == uuid.Nil || !validPhaseDateRange(req.StartDate, req.EndDate) || (len(req.OverrideConfig) > 0 && !json.Valid(req.OverrideConfig)) {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	out, err := h.Repository.AssignPhase(r.Context(), employmentID, req)
	if err != nil {
		h.err(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusOK, out)
}
func (h EmploymentPhaseHandler) AssignEmployee(w http.ResponseWriter, r *http.Request) {
	empID, ok := pathUUID(r, "user_id")
	if !ok || !h.allowed(r, "employment_phases:manage") {
		h.deny(w, r)
		return
	}
	employmentID, err := h.Repository.ResolveActiveEmployment(r.Context(), empID)
	if err != nil {
		h.err(w, r, err)
		return
	}
	h.assign(w, r, employmentID)
}
func (h EmploymentPhaseHandler) ListEmployee(w http.ResponseWriter, r *http.Request) {
	empID, ok := pathUUID(r, "user_id")
	if !ok || !h.allowed(r, "employment_phases:read") {
		h.deny(w, r)
		return
	}
	employmentID, err := h.Repository.ResolveActiveEmployment(r.Context(), empID)
	if err != nil {
		h.err(w, r, err)
		return
	}
	h.list(w, r, employmentID)
}
func (h EmploymentPhaseHandler) UpdateEmployeeStatus(w http.ResponseWriter, r *http.Request) {
	empID, ok := pathUUID(r, "user_id")
	phaseID, ok2 := pathUUID(r, "phase_id")
	if !ok || !ok2 || !h.allowed(r, "employment_phases:manage") {
		h.deny(w, r)
		return
	}
	employmentID, err := h.Repository.ResolveActiveEmployment(r.Context(), empID)
	if err != nil {
		h.err(w, r, err)
		return
	}
	h.updateStatus(w, r, employmentID, phaseID)
}
func (h EmploymentPhaseHandler) AssignEmployment(w http.ResponseWriter, r *http.Request) {
	employmentID, ok := pathUUID(r, "employment_id")
	if !ok || !h.allowed(r, "employment_phases:manage") {
		h.deny(w, r)
		return
	}
	h.assign(w, r, employmentID)
}
func (h EmploymentPhaseHandler) ListEmployment(w http.ResponseWriter, r *http.Request) {
	employmentID, ok := pathUUID(r, "employment_id")
	if !ok || !h.allowed(r, "employment_phases:read") {
		h.deny(w, r)
		return
	}
	h.list(w, r, employmentID)
}
func (h EmploymentPhaseHandler) UpdateEmploymentStatus(w http.ResponseWriter, r *http.Request) {
	employmentID, ok := pathUUID(r, "employment_id")
	phaseID, ok2 := pathUUID(r, "phase_id")
	if !ok || !ok2 || !h.allowed(r, "employment_phases:manage") {
		h.deny(w, r)
		return
	}
	h.updateStatus(w, r, employmentID, phaseID)
}
func (h EmploymentPhaseHandler) list(w http.ResponseWriter, r *http.Request, employmentID uuid.UUID) {
	if h.Repository == nil {
		apperror.WriteJSON(w, http.StatusOK, []EmployeePhase{})
		return
	}
	out, err := h.Repository.ListPhases(r.Context(), employmentID)
	if err != nil {
		h.err(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusOK, out)
}
func (h EmploymentPhaseHandler) updateStatus(w http.ResponseWriter, r *http.Request, employmentID, phaseID uuid.UUID) {
	var req UpdateEmployeePhaseStatusRequest
	if json.NewDecoder(r.Body).Decode(&req) != nil || (req.Status != "active" && req.Status != "completed" && req.Status != "terminated") {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	uid, _ := httpinfra.UserIDFromContext(r.Context())
	out, err := h.Repository.UpdatePhaseStatus(r.Context(), employmentID, phaseID, req, uid)
	if err != nil {
		h.err(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusOK, out)
}
