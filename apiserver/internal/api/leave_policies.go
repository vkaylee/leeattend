package api

import (
	"context"
	"encoding/json"
	"net/http"
	"strings"
	"time"

	"github.com/google/uuid"
	"github.com/shopspring/decimal"
	"github.com/vkaylee/leeattend/apiserver/internal/apperror"
	httpinfra "github.com/vkaylee/leeattend/apiserver/internal/infra/http"
)

type LeaveTypeRecord struct {
	ID           uuid.UUID       `json:"id"`
	Code         string          `json:"code"`
	Name         string          `json:"name"`
	IsPaid       bool            `json:"is_paid"`
	PolicyConfig json.RawMessage `json:"policy_config"`
	CreatedAt    string          `json:"created_at"`
}

type CreateLeaveTypeRequest struct {
	Code         string          `json:"code"`
	Name         string          `json:"name"`
	IsPaid       *bool           `json:"is_paid"`
	PolicyConfig json.RawMessage `json:"policy_config"`
}

type UpdateLeaveTypeConfigRequest struct {
	PolicyConfig json.RawMessage `json:"policy_config"`
}

type SeniorityBonus struct {
	Months      int             `json:"months"`
	BonusAmount decimal.Decimal `json:"bonus_amount"`
}
type LeavePolicyConfig struct {
	Frequency            string           `json:"frequency"`
	AccrualAmount        decimal.Decimal  `json:"accrual_amount"`
	AccrualTiming        string           `json:"accrual_timing"`
	SeniorityBonus       []SeniorityBonus `json:"seniority_bonus"`
	CarryOverLimit       *int             `json:"carry_over_limit"`
	MaxBalance           *decimal.Decimal `json:"max_balance"`
	ResetMonth           *int             `json:"reset_month"`
	MinWorkDaysThreshold *decimal.Decimal `json:"min_work_days_threshold"`
}

type LeavePolicyDefinition struct {
	ID          uuid.UUID         `json:"id"`
	Name        string            `json:"name"`
	Description *string           `json:"description"`
	LeaveTypeID uuid.UUID         `json:"leave_type_id"`
	RuleConfig  LeavePolicyConfig `json:"rule_config"`
	CreatedAt   string            `json:"created_at"`
	UpdatedAt   string            `json:"updated_at"`
}
type CreateLeavePolicyRequest struct {
	Name        string            `json:"name"`
	Description *string           `json:"description"`
	LeaveTypeID uuid.UUID         `json:"leave_type_id"`
	RuleConfig  LeavePolicyConfig `json:"rule_config"`
}
type UpdateLeavePolicyRequest struct {
	Name        *string            `json:"name"`
	Description *string            `json:"description"`
	LeaveTypeID *uuid.UUID         `json:"leave_type_id"`
	RuleConfig  *LeavePolicyConfig `json:"rule_config"`
}
type AssignLeavePolicyRequest struct {
	AssigneeType  string     `json:"assignee_type"`
	AssigneeID    *uuid.UUID `json:"assignee_id"`
	PolicyID      uuid.UUID  `json:"policy_id"`
	EffectiveFrom string     `json:"effective_from"`
	EffectiveTo   *string    `json:"effective_to"`
}
type LeavePolicyAssignment struct {
	ID            uuid.UUID  `json:"id"`
	AssigneeType  string     `json:"assignee_type"`
	AssigneeID    *uuid.UUID `json:"assignee_id"`
	PolicyID      uuid.UUID  `json:"policy_id"`
	EffectiveFrom string     `json:"effective_from"`
	EffectiveTo   *string    `json:"effective_to"`
	CreatedAt     string     `json:"created_at"`
}
type LeavePolicyAssignmentDetail struct {
	LeavePolicyAssignment
	PolicyName    string `json:"policy_name"`
	LeaveTypeName string `json:"leave_type_name"`
}

// LeaveTypeRepository manages leave type definitions.
type LeaveTypeRepository interface {
	CreateLeaveType(context.Context, CreateLeaveTypeRequest) (LeaveTypeRecord, error)
	ListLeaveTypes(context.Context) ([]LeaveTypeRecord, error)
	UpdateLeaveTypeConfig(context.Context, uuid.UUID, json.RawMessage) (LeaveTypeRecord, error)
	DeleteLeaveType(context.Context, uuid.UUID) error
	RestoreLeaveType(context.Context, uuid.UUID) (LeaveTypeRecord, error)
}

// LeavePolicyRepository manages leave policies and their assignments.
type LeavePolicyRepository interface {
	ListLeavePolicies(context.Context) ([]LeavePolicyDefinition, error)
	CreateLeavePolicy(context.Context, CreateLeavePolicyRequest) (LeavePolicyDefinition, error)
	UpdateLeavePolicy(context.Context, uuid.UUID, UpdateLeavePolicyRequest) (LeavePolicyDefinition, error)
	DeleteLeavePolicy(context.Context, uuid.UUID) error
	AssignLeavePolicy(context.Context, AssignLeavePolicyRequest) (LeavePolicyAssignment, error)
	ListLeavePolicyAssignments(context.Context) ([]LeavePolicyAssignmentDetail, error)
	DeleteLeavePolicyAssignment(context.Context, uuid.UUID) error
}

func (h *LeaveHandler) CreateType(w http.ResponseWriter, r *http.Request) {

	if !hasPermission(r, h.Authorizer, "leave:types:manage") {
		writeForbidden(w, r)
		return
	}
	req, err := DecodeJSON[CreateLeaveTypeRequest](r, 1<<20)
	if err != nil || strings.TrimSpace(req.Code) == "" || strings.TrimSpace(req.Name) == "" {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	lt, err := h.Repository.CreateLeaveType(r.Context(), req)
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusCreated, lt)
}

func (h *LeaveHandler) ListTypes(w http.ResponseWriter, r *http.Request) {
	types, err := h.Repository.ListLeaveTypes(r.Context())
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusOK, types)
}

func (h *LeaveHandler) UpdateTypeConfig(w http.ResponseWriter, r *http.Request) {

	leaveTypeID, idOK := pathUUID(r, "id")
	if !idOK || !hasPermission(r, h.Authorizer, "leave:types:manage") {
		writeForbidden(w, r)
		return
	}
	req, err := DecodeJSON[UpdateLeaveTypeConfigRequest](r, 1<<20)
	if err != nil || !validLeaveTypeConfig(req.PolicyConfig) {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	record, err := h.Repository.UpdateLeaveTypeConfig(r.Context(), leaveTypeID, req.PolicyConfig)
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusOK, record)
}

func (h *LeaveHandler) DeleteType(w http.ResponseWriter, r *http.Request) {

	leaveTypeID, idOK := pathUUID(r, "id")
	if !idOK || !hasPermission(r, h.Authorizer, "leave:types:manage") {
		writeForbidden(w, r)
		return
	}
	if err := h.Repository.DeleteLeaveType(r.Context(), leaveTypeID); err != nil {
		h.writeError(w, r, err)
		return
	}
	w.WriteHeader(http.StatusNoContent)
}

func (h *LeaveHandler) RestoreType(w http.ResponseWriter, r *http.Request) {

	leaveTypeID, idOK := pathUUID(r, "id")
	if !idOK || !hasPermission(r, h.Authorizer, "leave:types:manage") {
		writeForbidden(w, r)
		return
	}
	record, err := h.Repository.RestoreLeaveType(r.Context(), leaveTypeID)
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusOK, record)
}

func validLeaveTypeConfig(value json.RawMessage) bool {
	var raw map[string]json.RawMessage
	if len(value) == 0 || json.Unmarshal(value, &raw) != nil || raw == nil {
		return false
	}
	for _, key := range []string{"requires_document", "allow_negative_balance", "approval_required"} {
		if item, ok := raw[key]; ok {
			var v bool
			if json.Unmarshal(item, &v) != nil {
				return false
			}
		}
	}
	if item, ok := raw["negative_balance_limit"]; ok && string(item) != "null" {
		var v float64
		if json.Unmarshal(item, &v) != nil {
			return false
		}
	}
	return true
}

func (h *LeaveHandler) ListPolicies(w http.ResponseWriter, r *http.Request) {

	_, userOK := httpinfra.UserIDFromContext(r.Context())
	if !userOK {
		writeForbidden(w, r)
		return
	}
	data, err := h.Repository.ListLeavePolicies(r.Context())
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusOK, data)
}

func (h *LeaveHandler) CreatePolicy(w http.ResponseWriter, r *http.Request) {

	if !hasPermission(r, h.Authorizer, "leave:policies:manage") {
		writeForbidden(w, r)
		return
	}
	req, err := DecodeJSON[CreateLeavePolicyRequest](r, 1<<20)
	if err != nil || strings.TrimSpace(req.Name) == "" || req.LeaveTypeID == uuid.Nil {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	data, err := h.Repository.CreateLeavePolicy(r.Context(), req)
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusCreated, data)
}

func (h *LeaveHandler) UpdatePolicy(w http.ResponseWriter, r *http.Request) {

	id, idOK := pathUUID(r, "policy_id")
	if !idOK || !hasPermission(r, h.Authorizer, "leave:policies:manage") {
		writeForbidden(w, r)
		return
	}
	req, err := DecodeJSON[UpdateLeavePolicyRequest](r, 1<<20)
	if err != nil {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	data, err := h.Repository.UpdateLeavePolicy(r.Context(), id, req)
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusOK, data)
}

func (h *LeaveHandler) DeletePolicy(w http.ResponseWriter, r *http.Request) {

	id, idOK := pathUUID(r, "policy_id")
	if !idOK || !hasPermission(r, h.Authorizer, "leave:policies:manage") {
		writeForbidden(w, r)
		return
	}
	if err := h.Repository.DeleteLeavePolicy(r.Context(), id); err != nil {
		h.writeError(w, r, err)
		return
	}
	w.WriteHeader(http.StatusNoContent)
}

func validAssignLeavePolicyRequest(req AssignLeavePolicyRequest) bool {
	if req.PolicyID == uuid.Nil {
		return false
	}
	switch req.AssigneeType {
	case "organization":
		if req.AssigneeID != nil {
			return false
		}
	case "department", "employee", "user":
		if req.AssigneeID == nil {
			return false
		}
	default:
		return false
	}
	_, err := time.Parse("2006-01-02", req.EffectiveFrom)
	return err == nil
}

func (h *LeaveHandler) AssignPolicy(w http.ResponseWriter, r *http.Request) {
	if !hasPermission(r, h.Authorizer, "leave:policies:manage") {
		writeForbidden(w, r)
		return
	}
	req, err := DecodeJSON[AssignLeavePolicyRequest](r, 1<<20)
	if err != nil || !validAssignLeavePolicyRequest(req) {
		writeValidation(w, r)
		return
	}
	data, err := h.Repository.AssignLeavePolicy(r.Context(), req)
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusCreated, data)
}

func (h *LeaveHandler) ListPolicyAssignments(w http.ResponseWriter, r *http.Request) {

	if !hasPermission(r, h.Authorizer, "leave:policies:manage") {
		writeForbidden(w, r)
		return
	}
	data, err := h.Repository.ListLeavePolicyAssignments(r.Context())
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusOK, data)
}

func (h *LeaveHandler) DeletePolicyAssignment(w http.ResponseWriter, r *http.Request) {

	id, idOK := pathUUID(r, "assignment_id")
	if !idOK || !hasPermission(r, h.Authorizer, "leave:policies:manage") {
		writeForbidden(w, r)
		return
	}
	if err := h.Repository.DeleteLeavePolicyAssignment(r.Context(), id); err != nil {
		h.writeError(w, r, err)
		return
	}
	w.WriteHeader(http.StatusNoContent)
}
