package api

import (
	"context"
	"net/http"
	"strconv"
	"strings"
	"time"

	"github.com/google/uuid"
	"github.com/shopspring/decimal"
	"github.com/vkaylee/leeattend/apiserver/internal/apperror"
	"github.com/vkaylee/leeattend/apiserver/internal/audit"
	"github.com/vkaylee/leeattend/apiserver/internal/domain/ids"
	"github.com/vkaylee/leeattend/apiserver/internal/domain/payroll/formula"
	httpinfra "github.com/vkaylee/leeattend/apiserver/internal/infra/http"
)

type PreviewPayrollFormulaRequest struct {
	Expression string                     `json:"expression"`
	Values     map[string]decimal.Decimal `json:"values"`
}

type PreviewPayrollFormulaResponse struct {
	Value string `json:"value"`
}

type CreatePayrollFormulaRuleRequest struct {
	Code          string `json:"code"`
	Name          string `json:"name"`
	ComponentCode string `json:"component_code"`
	Expression    string `json:"expression"`
	EffectiveFrom string `json:"effective_from"`
}

type CreatePayrollFormulaRuleVersionRequest struct {
	Expression    string `json:"expression"`
	EffectiveFrom string `json:"effective_from"`
}

type PayrollFormulaRule struct {
	ID            uuid.UUID `json:"id"`
	Code          string    `json:"code"`
	Name          string    `json:"name"`
	ComponentCode string    `json:"component_code"`
	VersionID     uuid.UUID `json:"version_id"`
	Version       int       `json:"version"`
	Expression    string    `json:"expression"`
	EffectiveFrom string    `json:"effective_from"`
	Status        string    `json:"status"`
}

type PayrollFormulaRuleVersion struct {
	VersionID     uuid.UUID  `json:"version_id"`
	RuleID        uuid.UUID  `json:"rule_id"`
	Version       int        `json:"version"`
	Expression    string     `json:"expression"`
	EffectiveFrom string     `json:"effective_from"`
	Status        string     `json:"status"`
	CreatedBy     uuid.UUID  `json:"created_by"`
	PublishedBy   *uuid.UUID `json:"published_by,omitempty"`
	PublishedAt   *string    `json:"published_at,omitempty"`
	CreatedAt     string     `json:"created_at"`
}

// PayrollFormulaRepository manages versioned payroll formula rules.
type PayrollFormulaRepository interface {
	ListPayrollFormulaRules(context.Context) ([]PayrollFormulaRule, error)
	ListPayrollFormulaRuleVersions(context.Context, uuid.UUID) ([]PayrollFormulaRuleVersion, error)
	CreatePayrollFormulaRule(context.Context, CreatePayrollFormulaRuleRequest, ids.UserID) (PayrollFormulaRule, error)
	CreatePayrollFormulaRuleVersion(context.Context, ids.PayrollFormulaRuleID, CreatePayrollFormulaRuleVersionRequest, ids.UserID) (PayrollFormulaRuleVersion, error)
	PublishPayrollFormulaRule(context.Context, ids.PayrollFormulaRuleID, ids.UserID) (PayrollFormulaRule, error)
	RetirePayrollFormulaRule(context.Context, ids.PayrollFormulaRuleID, ids.UserID) (PayrollFormulaRule, error)
}

func (h *PayrollHandler) ListPayrollFormulaRuleVersions(w http.ResponseWriter, r *http.Request) {
	if !hasPermission(r, h.Authorizer, "payroll:read") {
		writeForbidden(w, r)
		return
	}
	ruleID, ok := pathUUID(r, "rule_id")
	if !ok {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	versions, err := h.Repository.ListPayrollFormulaRuleVersions(r.Context(), ruleID)
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	if versions == nil {
		versions = []PayrollFormulaRuleVersion{}
	}
	apperror.WriteJSON(w, http.StatusOK, versions)
}

func (h *PayrollHandler) ListPayrollFormulaRules(w http.ResponseWriter, r *http.Request) {
	if !hasPermission(r, h.Authorizer, "payroll:read") {
		writeForbidden(w, r)
		return
	}
	rules, err := h.Repository.ListPayrollFormulaRules(r.Context())
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	if rules == nil {
		rules = []PayrollFormulaRule{}
	}
	apperror.WriteJSON(w, http.StatusOK, rules)
}

func (h *PayrollHandler) CreatePayrollFormulaRule(w http.ResponseWriter, r *http.Request) {
	if !hasPermission(r, h.Authorizer, "payroll:manage") {
		writeForbidden(w, r)
		return
	}
	actorID, ok := httpinfra.UserIDValueFromContext(r.Context())
	if !ok {
		writeForbidden(w, r)
		return
	}
	req, err := DecodeJSON[CreatePayrollFormulaRuleRequest](r, 1<<20)
	if err != nil || strings.TrimSpace(req.Code) == "" || strings.TrimSpace(req.Name) == "" || strings.TrimSpace(req.ComponentCode) == "" || len(req.Expression) == 0 || len(req.Expression) > 4096 {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	if _, err := time.Parse("2006-01-02", req.EffectiveFrom); err != nil {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	if err := formula.Validate(req.Expression); err != nil {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	rule, err := h.Repository.CreatePayrollFormulaRule(r.Context(), req, actorID)
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	h.emitPayrollAudit(r, audit.Event{
		Name:     "payroll.formula.created",
		Resource: rule.ID.String(),
		Outcome:  "success",
		Details: map[string]string{
			"rule_id":        rule.ID.String(),
			"version_id":     rule.VersionID.String(),
			"version":        strconv.Itoa(rule.Version),
			"effective_from": rule.EffectiveFrom,
		},
	})
	apperror.WriteJSON(w, http.StatusCreated, rule)
}

func (h *PayrollHandler) CreatePayrollFormulaRuleVersion(w http.ResponseWriter, r *http.Request) {
	if !hasPermission(r, h.Authorizer, "payroll:manage") {
		writeForbidden(w, r)
		return
	}
	ruleID, ok := pathUUID(r, "rule_id")
	actorID, actorOK := httpinfra.UserIDValueFromContext(r.Context())
	if !ok || !actorOK {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	req, err := DecodeJSON[CreatePayrollFormulaRuleVersionRequest](r, 1<<20)
	if err != nil || len(req.Expression) == 0 || len(req.Expression) > 4096 {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	if _, err := time.Parse("2006-01-02", req.EffectiveFrom); err != nil {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	if err := formula.Validate(req.Expression); err != nil {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	version, err := h.Repository.CreatePayrollFormulaRuleVersion(r.Context(), ids.PayrollFormulaRuleID(ruleID), req, actorID)
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	h.emitPayrollAudit(r, audit.Event{
		Name:     "payroll.formula.version_created",
		Resource: version.RuleID.String(),
		Outcome:  "success",
		Details: map[string]string{
			"rule_id":        version.RuleID.String(),
			"version_id":     version.VersionID.String(),
			"version":        strconv.Itoa(version.Version),
			"effective_from": version.EffectiveFrom,
		},
	})
	apperror.WriteJSON(w, http.StatusCreated, version)
}

func (h *PayrollHandler) RetirePayrollFormulaRule(w http.ResponseWriter, r *http.Request) {
	if !hasPermission(r, h.Authorizer, "payroll:manage") {
		writeForbidden(w, r)
		return
	}
	ruleID, ok := pathUUID(r, "rule_id")
	actorID, actorOK := httpinfra.UserIDValueFromContext(r.Context())
	if !ok || !actorOK {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	rule, err := h.Repository.RetirePayrollFormulaRule(r.Context(), ids.PayrollFormulaRuleID(ruleID), actorID)
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	h.emitPayrollAudit(r, audit.Event{
		Name:     "payroll.formula.retired",
		Resource: rule.ID.String(),
		Outcome:  "success",
		Details: map[string]string{
			"rule_id":    rule.ID.String(),
			"version_id": rule.VersionID.String(),
			"version":    strconv.Itoa(rule.Version),
		},
	})
	apperror.WriteJSON(w, http.StatusOK, rule)
}

func (h *PayrollHandler) PublishPayrollFormulaRule(w http.ResponseWriter, r *http.Request) {
	if !hasPermission(r, h.Authorizer, "payroll:manage") {
		writeForbidden(w, r)
		return
	}
	ruleID, ok := pathUUID(r, "rule_id")
	actorID, actorOK := httpinfra.UserIDValueFromContext(r.Context())
	if !ok || !actorOK {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	rule, err := h.Repository.PublishPayrollFormulaRule(r.Context(), ids.PayrollFormulaRuleID(ruleID), actorID)
	if err != nil {
		h.writeError(w, r, err)
		return
	}
	h.emitPayrollAudit(r, audit.Event{
		Name:     "payroll.formula.published",
		Resource: rule.ID.String(),
		Outcome:  "success",
		Details: map[string]string{
			"rule_id":        rule.ID.String(),
			"version_id":     rule.VersionID.String(),
			"version":        strconv.Itoa(rule.Version),
			"effective_from": rule.EffectiveFrom,
		},
	})
	apperror.WriteJSON(w, http.StatusOK, rule)
}

func (h *PayrollHandler) PreviewPayrollFormula(w http.ResponseWriter, r *http.Request) {
	if !hasPermission(r, h.Authorizer, "payroll:manage") {
		writeForbidden(w, r)
		return
	}
	req, err := DecodeJSON[PreviewPayrollFormulaRequest](r, 1<<20)
	if err != nil || len(req.Expression) == 0 || len(req.Expression) > 4096 || len(req.Values) > 100 {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	value, err := formula.Evaluate(req.Expression, req.Values)
	if err != nil {
		apperror.WriteError(w, r, apperror.Validation(apperror.KeyValidation))
		return
	}
	apperror.WriteJSON(w, http.StatusOK, PreviewPayrollFormulaResponse{Value: value.String()})
}
