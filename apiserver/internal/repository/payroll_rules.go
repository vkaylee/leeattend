package repository

import (
	"context"
	"encoding/json"
	"github.com/google/uuid"
	"github.com/shopspring/decimal"
	"github.com/vkaylee/leeattend/apiserver/internal/api"
	"github.com/vkaylee/leeattend/apiserver/internal/domain/ids"
	"strings"
	"time"
)

func (r PayrollRepository) ListPayrollRules(ctx context.Context) ([]api.PayrollRuleDefinition, error) {
	db, err := requireDB(r.DB, "payroll repository")
	if err != nil {
		return nil, err
	}
	rows, err := db.Query(ctx, `SELECT id, name, description, code, category, applies_to, calculation_method, rule_config, is_mandatory, is_active, display_order, created_at, updated_at
FROM payroll_rule_definitions
ORDER BY display_order ASC, created_at DESC`)
	if err != nil {
		return nil, err
	}
	defer rows.Close()

	result := make([]api.PayrollRuleDefinition, 0)
	for rows.Next() {
		var d api.PayrollRuleDefinition
		var desc *string
		var cfg []byte
		var created, updated time.Time
		if err := rows.Scan(&d.ID, &d.Name, &desc, &d.Code, &d.Category, &d.AppliesTo, &d.CalculationMethod, &cfg, &d.IsMandatory, &d.IsActive, &d.DisplayOrder, &created, &updated); err != nil {
			return nil, err
		}
		d.Description = desc
		d.RuleConfig = cfg
		d.CreatedAt = created.UTC().Format(time.RFC3339)
		d.UpdatedAt = updated.UTC().Format(time.RFC3339)
		result = append(result, d)
	}
	return result, rows.Err()
}

func (r PayrollRepository) CreatePayrollRule(ctx context.Context, req api.CreatePayrollRuleDefinitionRequest) (api.PayrollRuleDefinition, error) {
	db, err := requireDB(r.DB, "payroll repository")
	if err != nil {
		return api.PayrollRuleDefinition{}, err
	}

	cat := req.Category
	if cat == "" {
		cat = "deduction"
	}
	applies := req.AppliesTo
	if applies == "" {
		applies = "employee"
	}
	calcMethod := req.CalculationMethod
	if calcMethod == "" {
		calcMethod = "percentage"
	}
	cfg := req.RuleConfig
	if len(cfg) == 0 {
		cfg = []byte("{}")
	}
	isMandatory := false
	if req.IsMandatory != nil {
		isMandatory = *req.IsMandatory
	}
	displayOrder := 0
	if req.DisplayOrder != nil {
		displayOrder = *req.DisplayOrder
	}

	ruleID := uuid.New()
	var created, updated time.Time
	err = db.QueryRow(ctx, `INSERT INTO payroll_rule_definitions (id, name, description, code, category, applies_to, calculation_method, rule_config, is_mandatory, is_active, display_order, created_at, updated_at)
VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, true, $10, NOW(), NOW())
RETURNING created_at, updated_at`, ruleID, strings.TrimSpace(req.Name), req.Description, strings.TrimSpace(req.Code), cat, applies, calcMethod, cfg, isMandatory, displayOrder).Scan(&created, &updated)
	if err != nil {
		if isConflictError(err) {
			return api.PayrollRuleDefinition{}, api.ErrConflict
		}
		return api.PayrollRuleDefinition{}, err
	}

	return api.PayrollRuleDefinition{
		ID:                ruleID,
		Name:              strings.TrimSpace(req.Name),
		Description:       req.Description,
		Code:              strings.TrimSpace(req.Code),
		Category:          cat,
		AppliesTo:         applies,
		CalculationMethod: calcMethod,
		RuleConfig:        cfg,
		IsMandatory:       isMandatory,
		IsActive:          true,
		DisplayOrder:      displayOrder,
		CreatedAt:         created.UTC().Format(time.RFC3339),
		UpdatedAt:         updated.UTC().Format(time.RFC3339),
	}, nil
}

func (r PayrollRepository) ListPayrollRuleTiers(ctx context.Context, ruleID ids.PayrollRuleID) ([]api.PayrollRuleTier, error) {
	db, err := requireDB(r.DB, "payroll repository")
	if err != nil {
		return nil, err
	}
	var exists bool
	if err := db.QueryRow(ctx, `SELECT EXISTS (SELECT 1 FROM payroll_rule_definitions WHERE id = $1)`, uuid.UUID(ruleID)).Scan(&exists); err != nil {
		return nil, err
	}
	if !exists {
		return nil, api.ErrRuleNotFound
	}

	rows, err := db.Query(ctx, `SELECT id, rule_definition_id, tier_order, min_amount::text, max_amount::text, rate::text, fixed_amount::text, created_at
FROM payroll_rule_tiers
WHERE rule_definition_id = $1
ORDER BY tier_order ASC`, uuid.UUID(ruleID))
	if err != nil {
		return nil, err
	}
	defer rows.Close()

	result := make([]api.PayrollRuleTier, 0)
	for rows.Next() {
		var t api.PayrollRuleTier
		var minStr, rateStr, fixedStr string
		var maxStr *string
		var created time.Time
		if err := rows.Scan(&t.ID, &t.RuleDefinitionID, &t.TierOrder, &minStr, &maxStr, &rateStr, &fixedStr, &created); err != nil {
			return nil, err
		}
		t.MinAmount, _ = decimal.NewFromString(minStr)
		if maxStr != nil {
			maxDec, _ := decimal.NewFromString(*maxStr)
			t.MaxAmount = &maxDec
		}
		t.Rate, _ = decimal.NewFromString(rateStr)
		t.FixedAmount, _ = decimal.NewFromString(fixedStr)
		t.CreatedAt = created.UTC().Format(time.RFC3339)
		result = append(result, t)
	}
	return result, rows.Err()
}

func (r PayrollRepository) CreatePayrollRuleTier(ctx context.Context, ruleID ids.PayrollRuleID, req api.CreatePayrollRuleTierRequest) (api.PayrollRuleTier, error) {
	db, err := requireDB(r.DB, "payroll repository")
	if err != nil {
		return api.PayrollRuleTier{}, err
	}
	var exists bool
	if err := db.QueryRow(ctx, `SELECT EXISTS (SELECT 1 FROM payroll_rule_definitions WHERE id = $1)`, uuid.UUID(ruleID)).Scan(&exists); err != nil {
		return api.PayrollRuleTier{}, err
	}
	if !exists {
		return api.PayrollRuleTier{}, api.ErrRuleNotFound
	}
	if req.MaxAmount != nil && req.MaxAmount.LessThanOrEqual(req.MinAmount) {
		return api.PayrollRuleTier{}, api.ErrPayrollValidation
	}

	fixed := decimal.Zero
	if req.FixedAmount != nil {
		fixed = *req.FixedAmount
	}

	tierID := uuid.New()
	var created time.Time
	var maxParam any
	if req.MaxAmount != nil {
		maxParam = req.MaxAmount.String()
	}
	err = db.QueryRow(ctx, `INSERT INTO payroll_rule_tiers (id, rule_definition_id, tier_order, min_amount, max_amount, rate, fixed_amount, created_at)
VALUES ($1, $2, $3, $4, $5, $6, $7, NOW())
RETURNING created_at`, uuid.UUID(tierID), uuid.UUID(ruleID), req.TierOrder, req.MinAmount.String(), maxParam, req.Rate.String(), fixed.String()).Scan(&created)
	if err != nil {
		if isConflictError(err) {
			return api.PayrollRuleTier{}, api.ErrConflict
		}
		return api.PayrollRuleTier{}, err
	}

	return api.PayrollRuleTier{
		ID:               tierID,
		RuleDefinitionID: uuid.UUID(ruleID),
		TierOrder:        req.TierOrder,
		MinAmount:        req.MinAmount,
		MaxAmount:        req.MaxAmount,
		Rate:             req.Rate,
		FixedAmount:      fixed,
		CreatedAt:        created.UTC().Format(time.RFC3339),
	}, nil
}

func (r PayrollRepository) DeletePayrollRuleTier(ctx context.Context, ruleID ids.PayrollRuleID, tierID ids.PayrollRuleTierID) error {
	db, err := requireDB(r.DB, "payroll repository")
	if err != nil {
		return err
	}
	_, err = db.Exec(ctx, `DELETE FROM payroll_rule_tiers WHERE id = $1 AND rule_definition_id = $2`, uuid.UUID(tierID), uuid.UUID(ruleID))
	return err
}

func (r PayrollRepository) ListPayrollRuleAssignments(ctx context.Context, ruleID ids.PayrollRuleID) ([]api.PayrollRuleAssignment, error) {
	db, err := requireDB(r.DB, "payroll repository")
	if err != nil {
		return nil, err
	}
	var exists bool
	if err := db.QueryRow(ctx, `SELECT EXISTS (SELECT 1 FROM payroll_rule_definitions WHERE id = $1)`, uuid.UUID(ruleID)).Scan(&exists); err != nil {
		return nil, err
	}
	if !exists {
		return nil, api.ErrRuleNotFound
	}

	rows, err := db.Query(ctx, `SELECT id, rule_definition_id, assignee_type, assignee_id, effective_from::text, effective_to::text, override_config, created_at
FROM payroll_rule_assignments
WHERE rule_definition_id = $1
ORDER BY created_at DESC`, uuid.UUID(ruleID))
	if err != nil {
		return nil, err
	}
	defer rows.Close()

	result := make([]api.PayrollRuleAssignment, 0)
	for rows.Next() {
		var a api.PayrollRuleAssignment
		var assigneeID *uuid.UUID
		var effTo *string
		var overrideCfg []byte
		var created time.Time
		if err := rows.Scan(&a.ID, &a.RuleDefinitionID, &a.AssigneeType, &assigneeID, &a.EffectiveFrom, &effTo, &overrideCfg, &created); err != nil {
			return nil, err
		}
		a.AssigneeID = assigneeID
		a.EffectiveTo = effTo
		if len(overrideCfg) > 0 {
			raw := json.RawMessage(overrideCfg)
			a.OverrideConfig = &raw
		}
		a.CreatedAt = created.UTC().Format(time.RFC3339)
		result = append(result, a)
	}
	return result, rows.Err()
}

func (r PayrollRepository) AssignPayrollRule(ctx context.Context, ruleID ids.PayrollRuleID, req api.AssignPayrollRuleRequest) (api.PayrollRuleAssignment, error) {
	db, err := requireDB(r.DB, "payroll repository")
	if err != nil {
		return api.PayrollRuleAssignment{}, err
	}
	var exists bool
	if err := db.QueryRow(ctx, `SELECT EXISTS (SELECT 1 FROM payroll_rule_definitions WHERE id = $1)`, uuid.UUID(ruleID)).Scan(&exists); err != nil {
		return api.PayrollRuleAssignment{}, err
	}
	if !exists {
		return api.PayrollRuleAssignment{}, api.ErrRuleNotFound
	}

	// Organization-wide assignments are implied by the singleton organization,
	// so they carry no target row identifier.
	assigneeID := req.AssigneeID
	if req.AssigneeType == "organization" {
		assigneeID = nil
	}

	assignID := uuid.New()
	var created time.Time
	var overrideParam any
	if req.OverrideConfig != nil && len(*req.OverrideConfig) > 0 {
		overrideParam = []byte(*req.OverrideConfig)
	}
	err = db.QueryRow(ctx, `INSERT INTO payroll_rule_assignments (id, rule_definition_id, assignee_type, assignee_id, effective_from, effective_to, override_config, created_at)
VALUES ($1, $2, $3, $4, $5::date, $6::date, $7, NOW())
RETURNING created_at`, assignID, uuid.UUID(ruleID), req.AssigneeType, assigneeID, req.EffectiveFrom, req.EffectiveTo, overrideParam).Scan(&created)
	if err != nil {
		if isConflictError(err) {
			return api.PayrollRuleAssignment{}, api.ErrConflict
		}
		return api.PayrollRuleAssignment{}, err
	}

	return api.PayrollRuleAssignment{
		ID:               assignID,
		RuleDefinitionID: uuid.UUID(ruleID),
		AssigneeType:     req.AssigneeType,
		AssigneeID:       assigneeID,
		EffectiveFrom:    req.EffectiveFrom,
		EffectiveTo:      req.EffectiveTo,
		OverrideConfig:   req.OverrideConfig,
		CreatedAt:        created.UTC().Format(time.RFC3339),
	}, nil
}

func (r PayrollRepository) DeletePayrollRuleAssignment(ctx context.Context, ruleID ids.PayrollRuleID, assignmentID ids.PayrollRuleAssignmentID) error {
	db, err := requireDB(r.DB, "payroll repository")
	if err != nil {
		return err
	}
	_, err = db.Exec(ctx, `DELETE FROM payroll_rule_assignments WHERE id = $1 AND rule_definition_id = $2`, uuid.UUID(assignmentID), uuid.UUID(ruleID))
	return err
}
