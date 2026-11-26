package repository

import (
	"context"
	"errors"
	"github.com/google/uuid"
	"github.com/jackc/pgx/v5"
	"github.com/vkaylee/leeattend/apiserver/internal/api"
	"github.com/vkaylee/leeattend/apiserver/internal/domain/ids"
	"github.com/vkaylee/leeattend/apiserver/internal/service"
	"strings"
	"time"
)

func (r PayrollRepository) ListPayrollFormulaRuleVersions(ctx context.Context, ruleID uuid.UUID) ([]api.PayrollFormulaRuleVersion, error) {
	db, err := requireDB(r.DB, "payroll repository")
	if err != nil {
		return nil, err
	}
	rows, err := db.Query(ctx, `SELECT v.id, v.rule_id, v.version_number, v.expression, v.effective_from::text, v.status, v.created_by, v.published_by, v.published_at, v.created_at
FROM payroll_formula_rule_versions v WHERE v.rule_id = $1 ORDER BY v.version_number DESC`, ruleID)
	if err != nil {
		return nil, err
	}
	defer rows.Close()
	result := make([]api.PayrollFormulaRuleVersion, 0)
	for rows.Next() {
		var version api.PayrollFormulaRuleVersion
		var publishedAt, createdAt time.Time
		if err := rows.Scan(&version.VersionID, &version.RuleID, &version.Version, &version.Expression, &version.EffectiveFrom, &version.Status, &version.CreatedBy, &version.PublishedBy, &publishedAt, &createdAt); err != nil {
			return nil, err
		}
		if !publishedAt.IsZero() {
			value := publishedAt.UTC().Format(time.RFC3339)
			version.PublishedAt = &value
		}
		version.CreatedAt = createdAt.UTC().Format(time.RFC3339)
		result = append(result, version)
	}
	return result, rows.Err()
}

func (r PayrollRepository) ListPayrollFormulaRules(ctx context.Context) ([]api.PayrollFormulaRule, error) {
	db, err := requireDB(r.DB, "payroll repository")
	if err != nil {
		return nil, err
	}
	rows, err := db.Query(ctx, `SELECT r.id, r.code, r.name, r.component_code, v.id, v.version_number, v.expression, v.effective_from::text, v.status
FROM payroll_formula_rules r
JOIN LATERAL (SELECT id, version_number, expression, effective_from, status FROM payroll_formula_rule_versions WHERE rule_id = r.id ORDER BY version_number DESC LIMIT 1) v ON true
ORDER BY r.code ASC`)
	if err != nil {
		return nil, err
	}
	defer rows.Close()
	result := make([]api.PayrollFormulaRule, 0)
	for rows.Next() {
		var rule api.PayrollFormulaRule
		if err := rows.Scan(&rule.ID, &rule.Code, &rule.Name, &rule.ComponentCode, &rule.VersionID, &rule.Version, &rule.Expression, &rule.EffectiveFrom, &rule.Status); err != nil {
			return nil, err
		}
		result = append(result, rule)
	}
	return result, rows.Err()
}

func (r PayrollRepository) CreatePayrollFormulaRule(ctx context.Context, req api.CreatePayrollFormulaRuleRequest, actorID ids.UserID) (api.PayrollFormulaRule, error) {
	db, err := requireDB(r.DB, "payroll repository")
	if err != nil {
		return api.PayrollFormulaRule{}, err
	}
	beginner, ok := db.(service.TxBeginner)
	if !ok {
		return api.PayrollFormulaRule{}, errors.New("payroll repository: database does not support transactions")
	}
	var result api.PayrollFormulaRule
	err = service.WithTx(ctx, beginner, func(ctx context.Context, tx pgx.Tx) error {
		ruleID := uuid.New()
		if _, err := tx.Exec(ctx, `INSERT INTO payroll_formula_rules (id, code, name, component_code, created_by) VALUES ($1, $2, $3, $4, $5)`, ruleID, strings.TrimSpace(req.Code), strings.TrimSpace(req.Name), strings.TrimSpace(req.ComponentCode), uuid.UUID(actorID)); err != nil {
			if isConflictError(err) {
				return api.ErrConflict
			}
			return err
		}
		if err := tx.QueryRow(ctx, `INSERT INTO payroll_formula_rule_versions (rule_id, version_number, expression, effective_from, status, created_by) VALUES ($1, 1, $2, $3::date, 'draft', $4) RETURNING $5::uuid, $6::text, $7::text, $8::text, id, version_number, expression, effective_from::text, status`, ruleID, req.Expression, req.EffectiveFrom, uuid.UUID(actorID), ruleID, req.Code, req.Name, req.ComponentCode).Scan(
			&result.ID, &result.Code, &result.Name, &result.ComponentCode, &result.VersionID, &result.Version, &result.Expression, &result.EffectiveFrom, &result.Status); err != nil {
			return err
		}
		return nil
	})
	if err != nil {
		return api.PayrollFormulaRule{}, err
	}
	return result, nil
}

func (r PayrollRepository) CreatePayrollFormulaRuleVersion(ctx context.Context, ruleID ids.PayrollFormulaRuleID, req api.CreatePayrollFormulaRuleVersionRequest, actorID ids.UserID) (api.PayrollFormulaRuleVersion, error) {
	db, err := requireDB(r.DB, "payroll repository")
	if err != nil {
		return api.PayrollFormulaRuleVersion{}, err
	}
	beginner, ok := db.(service.TxBeginner)
	if !ok {
		return api.PayrollFormulaRuleVersion{}, errors.New("payroll repository: database does not support transactions")
	}
	var result api.PayrollFormulaRuleVersion
	err = service.WithTx(ctx, beginner, func(ctx context.Context, tx pgx.Tx) error {
		var retired bool
		if err := tx.QueryRow(ctx, `SELECT retired_at IS NOT NULL FROM payroll_formula_rules WHERE id = $1 FOR UPDATE`, uuid.UUID(ruleID)).Scan(&retired); err != nil {
			if errors.Is(err, pgx.ErrNoRows) {
				return api.ErrRuleNotFound
			}
			return err
		}
		if retired {
			return api.ErrConflict
		}
		var created time.Time
		err := tx.QueryRow(ctx, `
			INSERT INTO payroll_formula_rule_versions (rule_id, version_number, expression, effective_from, status, created_by)
			SELECT $1, COALESCE(MAX(version_number), 0) + 1, $2, $3::date, 'draft', $4
			FROM payroll_formula_rule_versions
			WHERE rule_id = $1
			RETURNING id, rule_id, version_number, expression, effective_from::text, status, created_by, created_at`, uuid.UUID(ruleID), req.Expression, req.EffectiveFrom, uuid.UUID(actorID)).Scan(
			&result.VersionID, &result.RuleID, &result.Version, &result.Expression, &result.EffectiveFrom, &result.Status, &result.CreatedBy, &created)
		if err != nil {
			if isConflictError(err) {
				return api.ErrConflict
			}
			return err
		}
		result.CreatedAt = created.UTC().Format(time.RFC3339)
		return nil
	})
	if err != nil {
		return api.PayrollFormulaRuleVersion{}, err
	}
	return result, nil
}

func (r PayrollRepository) RetirePayrollFormulaRule(ctx context.Context, ruleID ids.PayrollFormulaRuleID, actorID ids.UserID) (api.PayrollFormulaRule, error) {
	db, err := requireDB(r.DB, "payroll repository")
	if err != nil {
		return api.PayrollFormulaRule{}, err
	}
	beginner, ok := db.(service.TxBeginner)
	if !ok {
		return api.PayrollFormulaRule{}, errors.New("payroll repository: database does not support transactions")
	}
	var result api.PayrollFormulaRule
	err = service.WithTx(ctx, beginner, func(ctx context.Context, tx pgx.Tx) error {
		if err := tx.QueryRow(ctx, `UPDATE payroll_formula_rule_versions v
		SET status = 'retired', retired_at = CURRENT_TIMESTAMP
		FROM payroll_formula_rules r
		WHERE v.rule_id = r.id AND r.id = $1 AND v.status = 'published'
		RETURNING r.id, r.code, r.name, r.component_code, v.id, v.version_number, v.expression, v.effective_from::text, v.status`, uuid.UUID(ruleID)).Scan(
			&result.ID, &result.Code, &result.Name, &result.ComponentCode, &result.VersionID, &result.Version, &result.Expression, &result.EffectiveFrom, &result.Status); err != nil {
			if errors.Is(err, pgx.ErrNoRows) {
				return api.ErrRuleNotFound
			}
			return err
		}
		_, err := tx.Exec(ctx, `UPDATE payroll_formula_rules SET retired_at = CURRENT_TIMESTAMP WHERE id = $1`, uuid.UUID(ruleID))
		return err
	})
	return result, err
}

func (r PayrollRepository) PublishPayrollFormulaRule(ctx context.Context, ruleID ids.PayrollFormulaRuleID, actorID ids.UserID) (api.PayrollFormulaRule, error) {
	db, err := requireDB(r.DB, "payroll repository")
	if err != nil {
		return api.PayrollFormulaRule{}, err
	}
	var result api.PayrollFormulaRule
	err = db.QueryRow(ctx, `UPDATE payroll_formula_rule_versions v
	SET status = 'published', published_by = $2, published_at = CURRENT_TIMESTAMP
	FROM payroll_formula_rules r
	WHERE v.rule_id = r.id AND r.id = $1 AND v.status = 'draft'
	RETURNING r.id, r.code, r.name, r.component_code, v.id, v.version_number, v.expression, v.effective_from::text, v.status`, uuid.UUID(ruleID), uuid.UUID(actorID)).Scan(
		&result.ID, &result.Code, &result.Name, &result.ComponentCode, &result.VersionID, &result.Version, &result.Expression, &result.EffectiveFrom, &result.Status)
	if errors.Is(err, pgx.ErrNoRows) {
		var exists bool
		if existsErr := db.QueryRow(ctx, `SELECT EXISTS (SELECT 1 FROM payroll_formula_rules WHERE id = $1)`, uuid.UUID(ruleID)).Scan(&exists); existsErr != nil {
			return api.PayrollFormulaRule{}, existsErr
		}
		if !exists {
			return api.PayrollFormulaRule{}, api.ErrRuleNotFound
		}
		return api.PayrollFormulaRule{}, api.ErrConflict
	}
	if err != nil {
		return api.PayrollFormulaRule{}, err
	}
	return result, nil
}
