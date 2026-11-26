package api

import (
	"context"
	"encoding/json"
	"errors"
	"fmt"
	"net/http"
	"strconv"
	"time"

	"github.com/google/uuid"
	"github.com/vkaylee/leeattend/apiserver/internal/apperror"
	httpinfra "github.com/vkaylee/leeattend/apiserver/internal/infra/http"
)

type AssignmentStatus string

const (
	AssignmentStatusActive    AssignmentStatus = "active"
	AssignmentStatusCompleted AssignmentStatus = "completed"
	AssignmentStatusCancelled AssignmentStatus = "cancelled"
)

func (s AssignmentStatus) valid() bool {
	return s == AssignmentStatusActive || s == AssignmentStatusCompleted || s == AssignmentStatusCancelled
}

type EmployeeAssignment struct {
	ID             uuid.UUID        `json:"id"`
	HomeLocationID uuid.UUID        `json:"home_location_id"`
	HostLocationID uuid.UUID        `json:"host_location_id"`
	StartDate      string           `json:"start_date"`
	EndDate        *string          `json:"end_date"`
	Reason         *string          `json:"reason"`
	Status         AssignmentStatus `json:"status"`
	CreatedAt      string           `json:"created_at"`
	UpdatedAt      string           `json:"updated_at"`
	EmploymentID   uuid.UUID        `json:"employment_id"`
}

type EmployeeAssignmentWithEmployee struct {
	EmployeeAssignment
	EmployeeID       uuid.UUID `json:"employee_id"`
	EmployeeName     string    `json:"employee_name"`
	EmployeeCode     string    `json:"employee_code"`
	HomeLocationName string    `json:"home_location_name"`
	HostLocationName string    `json:"host_location_name"`
}

type CreateEmployeeAssignmentRequest struct {
	HostLocationID uuid.UUID `json:"host_location_id"`
	StartDate      string    `json:"start_date"`
	EndDate        *string   `json:"end_date"`
	Reason         *string   `json:"reason"`
}

// UpdateEmployeeAssignmentRequest preserves omitted nullable fields and clears them on explicit null.
type UpdateEmployeeAssignmentRequest struct {
	EndDate        **string          `json:"end_date"`
	Status         *AssignmentStatus `json:"status"`
	Reason         **string          `json:"reason"`
	EndDatePresent bool              `json:"-"`
	ReasonPresent  bool              `json:"-"`
}

func (r *UpdateEmployeeAssignmentRequest) UnmarshalJSON(data []byte) error {
	type plain UpdateEmployeeAssignmentRequest
	var value plain
	if err := json.Unmarshal(data, &value); err != nil {
		return err
	}
	var fields map[string]json.RawMessage
	if err := json.Unmarshal(data, &fields); err != nil {
		return err
	}
	for key := range fields {
		if key != "end_date" && key != "status" && key != "reason" {
			return fmt.Errorf("json: unknown field %q", key)
		}
	}
	*r = UpdateEmployeeAssignmentRequest(value)
	_, r.EndDatePresent = fields["end_date"]
	_, r.ReasonPresent = fields["reason"]
	return nil
}

type AssignmentRuleConditions struct {
	DepartmentID      *uuid.UUID `json:"department_id"`
	EmployeeIDNumbers []string   `json:"employee_id_numbers"`
	Tags              []string   `json:"tags"`
	LocationID        *uuid.UUID `json:"location_id"`
	EmployerID        *uuid.UUID `json:"employer_id"`
}

type AssignmentRule struct {
	ID         uuid.UUID                `json:"id"`
	PatternID  uuid.UUID                `json:"pattern_id"`
	Name       string                   `json:"name"`
	Priority   int                      `json:"priority"`
	Conditions AssignmentRuleConditions `json:"conditions"`
	IsActive   bool                     `json:"is_active"`
	CreatedAt  string                   `json:"created_at"`
	UpdatedAt  string                   `json:"updated_at"`
	StartDate  *string                  `json:"start_date"`
	EndDate    *string                  `json:"end_date"`
}

type CreateAssignmentRuleRequest struct {
	PatternID  uuid.UUID                `json:"pattern_id"`
	Name       string                   `json:"name"`
	Priority   int                      `json:"priority"`
	Conditions AssignmentRuleConditions `json:"conditions"`
	StartDate  *string                  `json:"start_date"`
	EndDate    *string                  `json:"end_date"`
}

type UpdateAssignmentRuleRequest struct {
	PatternID            *uuid.UUID                `json:"pattern_id"`
	Name                 *string                   `json:"name"`
	Priority             *int                      `json:"priority"`
	Conditions           *AssignmentRuleConditions `json:"conditions"`
	IsActive             *bool                     `json:"is_active"`
	MaxConcurrentWorkers *int                      `json:"max_concurrent_workers"`
	StartDate            **string                  `json:"start_date"`
	EndDate              **string                  `json:"end_date"`
	StartDatePresent     bool                      `json:"-"`
	EndDatePresent       bool                      `json:"-"`
}

func (r *UpdateAssignmentRuleRequest) UnmarshalJSON(data []byte) error {
	type plain UpdateAssignmentRuleRequest
	var value plain
	if err := json.Unmarshal(data, &value); err != nil {
		return err
	}
	var fields map[string]json.RawMessage
	if err := json.Unmarshal(data, &fields); err != nil {
		return err
	}
	allowed := map[string]bool{"pattern_id": true, "name": true, "priority": true, "conditions": true, "is_active": true, "max_concurrent_workers": true, "start_date": true, "end_date": true}
	for key := range fields {
		if !allowed[key] {
			return fmt.Errorf("json: unknown field %q", key)
		}
	}
	*r = UpdateAssignmentRuleRequest(value)
	_, r.StartDatePresent = fields["start_date"]
	_, r.EndDatePresent = fields["end_date"]
	return nil
}

type AssignmentHistory struct {
	ID             uuid.UUID `json:"id"`
	EmployeeID     uuid.UUID `json:"employee_id"`
	EmployeeName   string    `json:"employee_name"`
	AssignmentType string    `json:"assignment_type"`
	ShiftName      *string   `json:"shift_name"`
	DateRange      string    `json:"date_range"`
	CreatedAt      string    `json:"created_at"`
}

type PaginatedAssignmentHistory struct {
	Data      []AssignmentHistory `json:"data"`
	Total     int64               `json:"total"`
	Page      int                 `json:"page"`
	Limit     int                 `json:"limit"`
	Truncated *bool               `json:"truncated"`
}

var (
	ErrEmployeeAssignmentNotFound = errors.New("employee assignment: not found")
	ErrHomeHostLocationSame       = errors.New("employee assignment: home and host locations are the same")
	ErrAssignmentRuleNotFound     = errors.New("assignment rule: not found")
)

type ActiveEmployment struct {
	ID                uuid.UUID
	PrimaryLocationID *uuid.UUID
}

type EmployeeAssignmentRepository interface {
	ListAssignments(context.Context) ([]EmployeeAssignmentWithEmployee, error)
	ListEmployeeAssignments(context.Context, uuid.UUID) ([]EmployeeAssignment, error)
	GetEmployeeAssignment(context.Context, uuid.UUID, uuid.UUID) (EmployeeAssignment, error)
	GetActiveEmployment(context.Context, uuid.UUID, string) (ActiveEmployment, error)
	CreateEmployeeAssignment(context.Context, uuid.UUID, uuid.UUID, CreateEmployeeAssignmentRequest) (EmployeeAssignment, error)
	UpdateEmployeeAssignment(context.Context, uuid.UUID, uuid.UUID, UpdateEmployeeAssignmentRequest) (EmployeeAssignment, error)
	DeleteEmployeeAssignment(context.Context, uuid.UUID, uuid.UUID) error
}

type AssignmentRuleRepository interface {
	ListAssignmentRules(context.Context) ([]AssignmentRule, error)
	CreateAssignmentRule(context.Context, CreateAssignmentRuleRequest) (AssignmentRule, error)
	UpdateAssignmentRule(context.Context, uuid.UUID, UpdateAssignmentRuleRequest) (AssignmentRule, error)
	DeleteAssignmentRule(context.Context, uuid.UUID) error
}

type AssignmentHistoryRepository interface {
	ListAssignmentHistory(context.Context, int, int) ([]AssignmentHistory, int64, error)
}

type AssignmentHandler struct {
	Assignments EmployeeAssignmentRepository
	Rules       AssignmentRuleRepository
	History     AssignmentHistoryRepository
	Employees   EmployeeRepository
	Authorizer  Authorizer
}

func (h *AssignmentHandler) member(w http.ResponseWriter, r *http.Request) bool {
	userID, ok := httpinfra.UserIDFromContext(r.Context())
	if !ok {
		apperror.WriteError(w, r, apperror.Unauthorized(apperror.KeyAuthenticationRequired))
		return false
	}
	if h.Authorizer == nil {
		apperror.WriteError(w, r, apperror.Forbidden(apperror.KeyNotOrganizationMember))
		return false
	}
	member, err := h.Authorizer.IsMember(r.Context(), userID)
	if err != nil {
		apperror.WriteError(w, r, apperror.Internal(err))
		return false
	}
	if !member {
		apperror.WriteError(w, r, apperror.Forbidden(apperror.KeyNotOrganizationMember))
		return false
	}
	return true
}

func (h *AssignmentHandler) permission(w http.ResponseWriter, r *http.Request, permission string) bool {
	userID, ok := httpinfra.UserIDFromContext(r.Context())
	if !ok {
		apperror.WriteError(w, r, apperror.Unauthorized(apperror.KeyAuthenticationRequired))
		return false
	}
	if h.Authorizer == nil || !h.Authorizer.HasPermission(r.Context(), userID, permission) {
		apperror.WriteError(w, r, apperror.Forbidden(apperror.KeyForbidden))
		return false
	}
	return true
}

func assignmentIDs(r *http.Request, names ...string) ([]uuid.UUID, bool) {
	ids := make([]uuid.UUID, len(names))
	for i, name := range names {
		var ok bool
		ids[i], ok = pathUUID(r, name)
		if !ok {
			return nil, false
		}
	}
	return ids, true
}

func validAssignmentDate(value string) bool {
	_, err := time.Parse("2006-01-02", value)
	return err == nil
}

func validUpdateEmployeeAssignmentRequest(req UpdateEmployeeAssignmentRequest, err error) bool {
	return err == nil && validUpdateEmployeeAssignmentFields(req) && validAssignmentReason(req)
}

func validUpdateAssignmentRuleRequest(req UpdateAssignmentRuleRequest, err error) bool {
	return err == nil && validAssignmentRuleDates(req)
}

func validAssignmentRuleDates(req UpdateAssignmentRuleRequest) bool {
	return (!req.StartDatePresent || req.StartDate == nil || *req.StartDate == nil || validAssignmentDate(**req.StartDate)) &&
		(!req.EndDatePresent || req.EndDate == nil || *req.EndDate == nil || validAssignmentDate(**req.EndDate))
}

func validUpdateEmployeeAssignmentFields(req UpdateEmployeeAssignmentRequest) bool {
	return (req.Status == nil || req.Status.valid()) && (!req.EndDatePresent || req.EndDate == nil || *req.EndDate == nil || validAssignmentDate(**req.EndDate))
}

func validAssignmentReason(req UpdateEmployeeAssignmentRequest) bool {
	return !req.ReasonPresent || req.Reason == nil || *req.Reason == nil || len(**req.Reason) <= 1<<20
}

func validCreateEmployeeAssignmentRequest(req CreateEmployeeAssignmentRequest) bool {
	return req.HostLocationID != uuid.Nil && validAssignmentDate(req.StartDate) && (req.EndDate == nil || validAssignmentDate(*req.EndDate))
}

func (h *AssignmentHandler) validateEmployee(w http.ResponseWriter, r *http.Request, employeeID uuid.UUID) bool {
	if h.Employees == nil {
		return true
	}
	if _, err := h.Employees.GetEmployee(r.Context(), employeeID); err == nil {
		return true
	} else if errors.Is(err, ErrNotFound) {
		apperror.WriteError(w, r, apperror.NotFound(apperror.KeyEmployeeNotFound))
	} else {
		h.assignmentError(w, r, err)
	}
	return false
}

func (h *AssignmentHandler) assignmentError(w http.ResponseWriter, r *http.Request, err error) {
	if mapped := mapLifecycleGuardError(err); mapped != nil {
		apperror.WriteError(w, r, mapped)
		return
	}
	switch {
	case errors.Is(err, ErrEmployeeAssignmentNotFound):
		apperror.WriteError(w, r, apperror.NotFound("error.employee_assignment_not_found"))
	case errors.Is(err, ErrAssignmentRuleNotFound):
		apperror.WriteError(w, r, apperror.NotFound(apperror.KeyNotFound))
	case errors.Is(err, ErrNoActiveEmployment):
		apperror.WriteError(w, r, apperror.Validation("error.no_active_employment"))
	case errors.Is(err, ErrHomeHostLocationSame):
		apperror.WriteError(w, r, apperror.Validation("error.home_host_location_same"))
	case errors.Is(err, ErrConflict):
		apperror.WriteError(w, r, apperror.Conflict(apperror.KeyConflict))
	case errors.Is(err, ErrNotFound):
		apperror.WriteError(w, r, apperror.NotFound(apperror.KeyEmployeeNotFound))
	default:
		apperror.WriteError(w, r, apperror.Internal(err))
	}
}

func (h *AssignmentHandler) ListAssignments(w http.ResponseWriter, r *http.Request) {
	if !h.member(w, r) || !h.permission(w, r, "employees:read") {
		return
	}
	items, err := h.Assignments.ListAssignments(r.Context())
	if err != nil {
		h.assignmentError(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusOK, items)
}

func (h *AssignmentHandler) ListEmployeeAssignments(w http.ResponseWriter, r *http.Request) {
	ids, ok := assignmentIDs(r, "employee_id")
	if !ok || !h.member(w, r) {
		return
	}
	items, err := h.Assignments.ListEmployeeAssignments(r.Context(), ids[0])
	if err != nil {
		h.assignmentError(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusOK, items)
}

func (h *AssignmentHandler) GetEmployeeAssignment(w http.ResponseWriter, r *http.Request) {
	ids, ok := assignmentIDs(r, "employee_id", "assignment_id")
	if !ok || !h.member(w, r) {
		return
	}
	item, err := h.Assignments.GetEmployeeAssignment(r.Context(), ids[0], ids[1])
	if err != nil {
		h.assignmentError(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusOK, item)
}

func (h *AssignmentHandler) CreateEmployeeAssignment(w http.ResponseWriter, r *http.Request) {
	ids, ok := assignmentIDs(r, "employee_id")
	if !ok || !h.member(w, r) {
		return
	}
	req, err := DecodeJSON[CreateEmployeeAssignmentRequest](r, 1<<20)
	if err != nil || !validCreateEmployeeAssignmentRequest(req) {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	if !h.validateEmployee(w, r, ids[0]) {
		return
	}
	active, err := h.Assignments.GetActiveEmployment(r.Context(), ids[0], time.Now().UTC().Format("2006-01-02"))
	if err != nil {
		h.assignmentError(w, r, err)
		return
	}
	if active.PrimaryLocationID == nil {
		apperror.WriteError(w, r, apperror.ValidationDetails("Employee active employment has no primary location assigned"))
		return
	}
	if *active.PrimaryLocationID == req.HostLocationID {
		h.assignmentError(w, r, ErrHomeHostLocationSame)
		return
	}
	item, err := h.Assignments.CreateEmployeeAssignment(r.Context(), active.ID, *active.PrimaryLocationID, req)
	if err != nil {
		h.assignmentError(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusOK, item)
}

func (h *AssignmentHandler) UpdateEmployeeAssignment(w http.ResponseWriter, r *http.Request) {
	ids, ok := assignmentIDs(r, "employee_id", "assignment_id")
	if !ok || !h.member(w, r) {
		return
	}
	req, err := DecodeJSON[UpdateEmployeeAssignmentRequest](r, 1<<20)
	if !validUpdateEmployeeAssignmentRequest(req, err) {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	item, err := h.Assignments.UpdateEmployeeAssignment(r.Context(), ids[0], ids[1], req)
	if err != nil {
		h.assignmentError(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusOK, item)
}

func (h *AssignmentHandler) DeleteEmployeeAssignment(w http.ResponseWriter, r *http.Request) {
	ids, ok := assignmentIDs(r, "employee_id", "assignment_id")
	if !ok || !h.member(w, r) {
		return
	}
	if err := h.Assignments.DeleteEmployeeAssignment(r.Context(), ids[0], ids[1]); err != nil {
		h.assignmentError(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusOK, nil)
}

func (h *AssignmentHandler) ListAssignmentRules(w http.ResponseWriter, r *http.Request) {
	if !h.permission(w, r, "assignment_rules:manage") {
		return
	}
	items, err := h.Rules.ListAssignmentRules(r.Context())
	if err != nil {
		h.assignmentError(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusOK, items)
}

func (h *AssignmentHandler) CreateAssignmentRule(w http.ResponseWriter, r *http.Request) {
	if !h.permission(w, r, "assignment_rules:manage") {
		return
	}
	req, err := DecodeJSON[CreateAssignmentRuleRequest](r, 1<<20)
	if err != nil || req.PatternID == uuid.Nil || req.Name == "" || (req.StartDate != nil && !validAssignmentDate(*req.StartDate)) || (req.EndDate != nil && !validAssignmentDate(*req.EndDate)) {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	item, err := h.Rules.CreateAssignmentRule(r.Context(), req)
	if err != nil {
		h.assignmentError(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusCreated, item)
}

func (h *AssignmentHandler) UpdateAssignmentRule(w http.ResponseWriter, r *http.Request) {
	ids, ok := assignmentIDs(r, "rule_id")
	if !ok || !h.permission(w, r, "assignment_rules:manage") {
		return
	}
	req, err := DecodeJSON[UpdateAssignmentRuleRequest](r, 1<<20)
	if !validUpdateAssignmentRuleRequest(req, err) {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	item, err := h.Rules.UpdateAssignmentRule(r.Context(), ids[0], req)
	if err != nil {
		h.assignmentError(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusOK, item)
}

func (h *AssignmentHandler) DeleteAssignmentRule(w http.ResponseWriter, r *http.Request) {
	ids, ok := assignmentIDs(r, "rule_id")
	if !ok || !h.permission(w, r, "assignment_rules:manage") {
		return
	}
	if err := h.Rules.DeleteAssignmentRule(r.Context(), ids[0]); err != nil {
		h.assignmentError(w, r, err)
		return
	}
	w.WriteHeader(http.StatusNoContent)
}

func (h *AssignmentHandler) AssignmentHistory(w http.ResponseWriter, r *http.Request) {
	if !h.member(w, r) {
		return
	}
	page, limit, err := assignmentPagination(r)
	if err != nil {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	items, total, err := h.History.ListAssignmentHistory(r.Context(), page, limit)
	if err != nil {
		h.assignmentError(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusOK, PaginatedAssignmentHistory{Data: items, Total: total, Page: page, Limit: limit})
}

func assignmentPagination(r *http.Request) (int, int, error) {
	page, limit := 1, 50
	var err error
	if raw := r.URL.Query().Get("page"); raw != "" {
		page, err = strconv.Atoi(raw)
		if err != nil {
			return 0, 0, err
		}
		if page < 1 {
			page = 1
		}
	}
	if raw := r.URL.Query().Get("limit"); raw != "" {
		limit, err = strconv.Atoi(raw)
		if err != nil {
			return 0, 0, err
		}
		if limit < 1 {
			limit = 1
		} else if limit > 3000 {
			limit = 3000
		}
	}
	return page, limit, nil
}
