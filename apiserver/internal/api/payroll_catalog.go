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
	"github.com/vkaylee/leeattend/apiserver/internal/domain/ids"
)

type CreateSalaryComponentRequest struct {
	Name string `json:"name"`
	Code string `json:"code"`
}

type EmploymentSalaryComponentResponse struct {
	ID                    uuid.UUID       `json:"id"`
	EmploymentID          uuid.UUID       `json:"employment_id"`
	ComponentDefinitionID uuid.UUID       `json:"component_definition_id"`
	Amount                decimal.Decimal `json:"amount"`
	EffectiveDate         string          `json:"effective_date"`
	Name                  string          `json:"name"`
	Code                  string          `json:"code"`
}

type AssignSalaryComponentRequest struct {
	ComponentDefinitionID uuid.UUID       `json:"component_definition_id"`
	Amount                decimal.Decimal `json:"amount"`
	EffectiveDate         string          `json:"effective_date"`
}

type IncentiveTier struct {
	ID           uuid.UUID       `json:"id"`
	PolicyID     uuid.UUID       `json:"policy_id"`
	Priority     int             `json:"priority"`
	PayoutAmount decimal.Decimal `json:"payout_amount"`
	RuleConfig   json.RawMessage `json:"rule_config"`
}

type CreateIncentiveTierRequest struct {
	Priority     int             `json:"priority"`
	PayoutAmount decimal.Decimal `json:"payout_amount"`
	RuleConfig   json.RawMessage `json:"rule_config"`
}

type PayrollIncentivePolicy struct {
	ID            uuid.UUID       `json:"id"`
	Name          string          `json:"name"`
	BaseAmount    decimal.Decimal `json:"base_amount"`
	PayoutType    string          `json:"payout_type"`
	IsActive      bool            `json:"is_active"`
	CreatedAt     string          `json:"created_at"`
	EffectiveFrom string          `json:"effective_from"`
	EffectiveTo   *string         `json:"effective_to"`
}

type PayrollIncentivePolicyResponse struct {
	PayrollIncentivePolicy
	Tiers []IncentiveTier `json:"tiers"`
}

type CreatePayrollIncentivePolicyRequest struct {
	Name          string                       `json:"name"`
	BaseAmount    decimal.Decimal              `json:"base_amount"`
	PayoutType    string                       `json:"payout_type"`
	EffectiveFrom string                       `json:"effective_from"`
	Tiers         []CreateIncentiveTierRequest `json:"tiers"`
}
type CreatePayrollRuleDefinitionRequest struct {
	Name              string          `json:"name"`
	Description       *string         `json:"description,omitempty"`
	Code              string          `json:"code"`
	Category          string          `json:"category"`
	AppliesTo         string          `json:"applies_to"`
	CalculationMethod string          `json:"calculation_method"`
	RuleConfig        json.RawMessage `json:"rule_config"`
	IsMandatory       *bool           `json:"is_mandatory,omitempty"`
	DisplayOrder      *int            `json:"display_order,omitempty"`
}

type PayrollRuleDefinition struct {
	ID                uuid.UUID       `json:"id"`
	Name              string          `json:"name"`
	Description       *string         `json:"description"`
	Code              string          `json:"code"`
	Category          string          `json:"category"`
	AppliesTo         string          `json:"applies_to"`
	CalculationMethod string          `json:"calculation_method"`
	RuleConfig        json.RawMessage `json:"rule_config"`
	IsMandatory       bool            `json:"is_mandatory"`
	IsActive          bool            `json:"is_active"`
	DisplayOrder      int             `json:"display_order"`
	CreatedAt         string          `json:"created_at"`
	UpdatedAt         string          `json:"updated_at"`
}

type CreatePayrollRuleTierRequest struct {
	TierOrder   int              `json:"tier_order"`
	MinAmount   decimal.Decimal  `json:"min_amount"`
	MaxAmount   *decimal.Decimal `json:"max_amount,omitempty"`
	Rate        decimal.Decimal  `json:"rate"`
	FixedAmount *decimal.Decimal `json:"fixed_amount,omitempty"`
}

type PayrollRuleTier struct {
	ID               uuid.UUID        `json:"id"`
	RuleDefinitionID uuid.UUID        `json:"rule_definition_id"`
	TierOrder        int              `json:"tier_order"`
	MinAmount        decimal.Decimal  `json:"min_amount"`
	MaxAmount        *decimal.Decimal `json:"max_amount"`
	Rate             decimal.Decimal  `json:"rate"`
	FixedAmount      decimal.Decimal  `json:"fixed_amount"`
	CreatedAt        string           `json:"created_at"`
}

type AssignPayrollRuleRequest struct {
	AssigneeType   string           `json:"assignee_type"`
	AssigneeID     *uuid.UUID       `json:"assignee_id,omitempty"`
	EffectiveFrom  string           `json:"effective_from"`
	EffectiveTo    *string          `json:"effective_to,omitempty"`
	OverrideConfig *json.RawMessage `json:"override_config,omitempty"`
}

type PayrollRuleAssignment struct {
	ID               uuid.UUID        `json:"id"`
	RuleDefinitionID uuid.UUID        `json:"rule_definition_id"`
	AssigneeType     string           `json:"assignee_type"`
	AssigneeID       *uuid.UUID       `json:"assignee_id"`
	EffectiveFrom    string           `json:"effective_from"`
	EffectiveTo      *string          `json:"effective_to"`
	OverrideConfig   *json.RawMessage `json:"override_config"`
	CreatedAt        string           `json:"created_at"`
}

// PayrollCatalogRepository manages salary components, incentive policies, and payroll rules.
type PayrollCatalogRepository interface {
	ListSalaryComponents(context.Context) ([]SalaryComponentDefinition, error)
	CreateSalaryComponent(context.Context, CreateSalaryComponentRequest) (SalaryComponentDefinition, error)
	ListEmploymentSalaryComponents(context.Context, ids.EmploymentID) ([]EmploymentSalaryComponentResponse, error)
	AssignEmploymentSalaryComponent(context.Context, ids.EmploymentID, AssignSalaryComponentRequest) (EmployeeSalaryComponent, error)
	GetIncentivePolicies(context.Context) ([]PayrollIncentivePolicyResponse, error)
	CreateIncentivePolicy(context.Context, CreatePayrollIncentivePolicyRequest) (PayrollIncentivePolicy, error)
	DeleteIncentivePolicy(context.Context, ids.PayrollIncentivePolicyID) error
	CloneIncentivePolicy(context.Context, ids.PayrollIncentivePolicyID) (PayrollIncentivePolicy, error)
	ListPayrollRules(context.Context) ([]PayrollRuleDefinition, error)
	CreatePayrollRule(context.Context, CreatePayrollRuleDefinitionRequest) (PayrollRuleDefinition, error)
	ListPayrollRuleTiers(context.Context, ids.PayrollRuleID) ([]PayrollRuleTier, error)
	CreatePayrollRuleTier(context.Context, ids.PayrollRuleID, CreatePayrollRuleTierRequest) (PayrollRuleTier, error)
	DeletePayrollRuleTier(context.Context, ids.PayrollRuleID, ids.PayrollRuleTierID) error
	ListPayrollRuleAssignments(context.Context, ids.PayrollRuleID) ([]PayrollRuleAssignment, error)
	AssignPayrollRule(context.Context, ids.PayrollRuleID, AssignPayrollRuleRequest) (PayrollRuleAssignment, error)
	DeletePayrollRuleAssignment(context.Context, ids.PayrollRuleID, ids.PayrollRuleAssignmentID) error
}

func (h *PayrollHandler) ListSalaryComponents(w http.ResponseWriter, r *http.Request) {

	if !h.allowedRead(r) {
		writeForbidden(w, r)
		return
	}
	rows, err := h.Repository.ListSalaryComponents(r.Context())
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	if rows == nil {
		rows = []SalaryComponentDefinition{}
	}
	apperror.WriteJSON(w, http.StatusOK, rows)
}

func (h *PayrollHandler) CreateSalaryComponent(w http.ResponseWriter, r *http.Request) {

	if !h.allowedManage(r) {
		writeForbidden(w, r)
		return
	}
	req, err := DecodeJSON[CreateSalaryComponentRequest](r, 1<<20)
	if err != nil || strings.TrimSpace(req.Name) == "" || strings.TrimSpace(req.Code) == "" {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	comp, err := h.Repository.CreateSalaryComponent(r.Context(), req)
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusCreated, comp)
}

func (h *PayrollHandler) ListEmploymentSalaryComponents(w http.ResponseWriter, r *http.Request) {

	empID, empOK := employmentIDFromRequest(r)
	if !empOK || !h.allowedRead(r) {
		writeForbidden(w, r)
		return
	}
	comps, err := h.Repository.ListEmploymentSalaryComponents(r.Context(), ids.EmploymentID(empID))
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	if comps == nil {
		comps = []EmploymentSalaryComponentResponse{}
	}
	apperror.WriteJSON(w, http.StatusOK, comps)
}

func (h *PayrollHandler) AssignEmploymentSalaryComponent(w http.ResponseWriter, r *http.Request) {

	empID, empOK := employmentIDFromRequest(r)
	if !empOK || !h.allowedManage(r) {
		writeForbidden(w, r)
		return
	}
	req, err := DecodeJSON[AssignSalaryComponentRequest](r, 1<<20)
	if err != nil || req.ComponentDefinitionID == uuid.Nil {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	if _, err := time.Parse("2006-01-02", req.EffectiveDate); err != nil {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	comp, err := h.Repository.AssignEmploymentSalaryComponent(r.Context(), ids.EmploymentID(empID), req)
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusCreated, comp)
}

func (h *PayrollHandler) GetIncentivePolicies(w http.ResponseWriter, r *http.Request) {

	if !h.allowedMember(r) {
		writeForbidden(w, r)
		return
	}
	policies, err := h.Repository.GetIncentivePolicies(r.Context())
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	if policies == nil {
		policies = []PayrollIncentivePolicyResponse{}
	}
	apperror.WriteJSON(w, http.StatusOK, policies)
}

func (h *PayrollHandler) CreateIncentivePolicy(w http.ResponseWriter, r *http.Request) {

	if !h.allowedMember(r) {
		writeForbidden(w, r)
		return
	}
	req, err := DecodeJSON[CreatePayrollIncentivePolicyRequest](r, 1<<20)
	if err != nil || strings.TrimSpace(req.Name) == "" || req.EffectiveFrom == "" {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	if _, err := time.Parse("2006-01-02", req.EffectiveFrom); err != nil {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	if req.PayoutType == "" {
		req.PayoutType = "fixed"
	}
	policy, err := h.Repository.CreateIncentivePolicy(r.Context(), req)
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusCreated, policy)
}

func (h *PayrollHandler) DeleteIncentivePolicy(w http.ResponseWriter, r *http.Request) {

	policyID, pOK := pathUUID(r, "policy_id")
	if !pOK || !h.allowedMember(r) {
		writeForbidden(w, r)
		return
	}
	if err := h.Repository.DeleteIncentivePolicy(r.Context(), ids.PayrollIncentivePolicyID(policyID)); err != nil {
		h.writeError(w, r, err)
		return
	}
	w.WriteHeader(http.StatusNoContent)
}

func (h *PayrollHandler) CloneIncentivePolicy(w http.ResponseWriter, r *http.Request) {

	policyID, pOK := pathUUID(r, "policy_id")
	if !pOK || !h.allowedMember(r) {
		writeForbidden(w, r)
		return
	}
	policy, err := h.Repository.CloneIncentivePolicy(r.Context(), ids.PayrollIncentivePolicyID(policyID))
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusCreated, policy)
}

func (h *PayrollHandler) ListPayrollRules(w http.ResponseWriter, r *http.Request) {

	if !h.allowedMember(r) {
		writeForbidden(w, r)
		return
	}
	rules, err := h.Repository.ListPayrollRules(r.Context())
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	if rules == nil {
		rules = []PayrollRuleDefinition{}
	}
	apperror.WriteJSON(w, http.StatusOK, rules)
}

func (h *PayrollHandler) CreatePayrollRule(w http.ResponseWriter, r *http.Request) {

	if !h.allowedMember(r) {
		writeForbidden(w, r)
		return
	}
	req, err := DecodeJSON[CreatePayrollRuleDefinitionRequest](r, 1<<20)
	if err != nil || strings.TrimSpace(req.Name) == "" || strings.TrimSpace(req.Code) == "" {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	rule, err := h.Repository.CreatePayrollRule(r.Context(), req)
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusCreated, rule)
}

func (h *PayrollHandler) ListPayrollRuleTiers(w http.ResponseWriter, r *http.Request) {

	ruleID, rOK := pathUUID(r, "rule_id")
	if !rOK || !h.allowedMember(r) {
		writeForbidden(w, r)
		return
	}
	tiers, err := h.Repository.ListPayrollRuleTiers(r.Context(), ids.PayrollRuleID(ruleID))
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	if tiers == nil {
		tiers = []PayrollRuleTier{}
	}
	apperror.WriteJSON(w, http.StatusOK, tiers)
}

func (h *PayrollHandler) CreatePayrollRuleTier(w http.ResponseWriter, r *http.Request) {

	ruleID, rOK := pathUUID(r, "rule_id")
	if !rOK || !h.allowedMember(r) {
		writeForbidden(w, r)
		return
	}
	req, err := DecodeJSON[CreatePayrollRuleTierRequest](r, 1<<20)
	if err != nil || req.TierOrder < 0 || req.MinAmount.IsNegative() {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	if req.MaxAmount != nil && req.MaxAmount.LessThanOrEqual(req.MinAmount) {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	tier, err := h.Repository.CreatePayrollRuleTier(r.Context(), ids.PayrollRuleID(ruleID), req)
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusCreated, tier)
}

func (h *PayrollHandler) DeletePayrollRuleTier(w http.ResponseWriter, r *http.Request) {

	ruleID, rOK := pathUUID(r, "rule_id")
	tierID, tOK := pathUUID(r, "tier_id")
	if !rOK || !tOK || !h.allowedMember(r) {
		writeForbidden(w, r)
		return
	}
	if err := h.Repository.DeletePayrollRuleTier(r.Context(), ids.PayrollRuleID(ruleID), ids.PayrollRuleTierID(tierID)); err != nil {
		h.writeError(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusOK, map[string]bool{"success": true})
}

func (h *PayrollHandler) ListPayrollRuleAssignments(w http.ResponseWriter, r *http.Request) {

	ruleID, rOK := pathUUID(r, "rule_id")
	if !rOK || !h.allowedMember(r) {
		writeForbidden(w, r)
		return
	}
	assignments, err := h.Repository.ListPayrollRuleAssignments(r.Context(), ids.PayrollRuleID(ruleID))
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	if assignments == nil {
		assignments = []PayrollRuleAssignment{}
	}
	apperror.WriteJSON(w, http.StatusOK, assignments)
}

func (h *PayrollHandler) AssignPayrollRule(w http.ResponseWriter, r *http.Request) {

	ruleID, rOK := pathUUID(r, "rule_id")
	if !rOK || !h.allowedMember(r) {
		writeForbidden(w, r)
		return
	}
	req, err := DecodeJSON[AssignPayrollRuleRequest](r, 1<<20)
	if err != nil || (req.AssigneeType != "organization" && req.AssigneeType != "department" && req.AssigneeType != "employee") {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	if req.AssigneeType != "organization" && req.AssigneeID == nil {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	if req.AssigneeType == "organization" {
		req.AssigneeID = nil
	}
	effFrom, err := time.Parse("2006-01-02", req.EffectiveFrom)
	if err != nil {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	if req.EffectiveTo != nil {
		effTo, err := time.Parse("2006-01-02", *req.EffectiveTo)
		if err != nil || effTo.Before(effFrom) {
			apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
			return
		}
	}
	assignment, err := h.Repository.AssignPayrollRule(r.Context(), ids.PayrollRuleID(ruleID), req)
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusCreated, assignment)
}

func (h *PayrollHandler) DeletePayrollRuleAssignment(w http.ResponseWriter, r *http.Request) {

	ruleID, rOK := pathUUID(r, "rule_id")
	assignmentID, aOK := pathUUID(r, "assignment_id")
	if !rOK || !aOK || !h.allowedMember(r) {
		writeForbidden(w, r)
		return
	}
	if err := h.Repository.DeletePayrollRuleAssignment(r.Context(), ids.PayrollRuleID(ruleID), ids.PayrollRuleAssignmentID(assignmentID)); err != nil {
		h.writeError(w, r, err)
		return
	}
	apperror.WriteJSON(w, http.StatusOK, map[string]bool{"success": true})
}
