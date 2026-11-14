package standalone

import (
	"context"
	"errors"
	"fmt"
	"strings"

	"github.com/google/uuid"
	"github.com/jackc/pgx/v5"
	"github.com/vkaylee/leeattend/apiserver/internal/auth"
)

var ErrInvalidBootstrap = errors.New("standalone: invalid bootstrap configuration")

// Bootstrap creates the singleton organization settings and its first administrator.
// It is idempotent for an already-complete installation and refuses conflicting
// existing state, preventing accidental takeover of a non-standalone database.
func ensurePayrollKPIFormula(ctx context.Context, tx pgx.Tx, actorID uuid.UUID) error {
	if _, err := tx.Exec(ctx, `INSERT INTO payroll_formula_rules (code, name, component_code, created_by)
VALUES ('kpi.bonus', 'KPI Bonus', 'KPI_BONUS', $1)
ON CONFLICT (code) DO NOTHING`, actorID); err != nil {
		return fmt.Errorf("standalone: seed KPI formula rule: %w", err)
	}
	if _, err := tx.Exec(ctx, `INSERT INTO payroll_formula_rule_versions (rule_id, version_number, expression, effective_from, status, created_by, published_by, published_at)
SELECT id, 1, 'min(target_bonus * (min(actual / target, score_cap) * output_weight + quality * quality_weight), bonus_cap)', CURRENT_DATE, 'published', $1, $1, CURRENT_TIMESTAMP
FROM payroll_formula_rules
WHERE code = 'kpi.bonus'
  AND NOT EXISTS (SELECT 1 FROM payroll_formula_rule_versions v WHERE v.rule_id = payroll_formula_rules.id)`, actorID); err != nil {
		return fmt.Errorf("standalone: seed KPI formula version: %w", err)
	}
	return nil
}

func Bootstrap(ctx context.Context, db interface {
	Begin(context.Context) (pgx.Tx, error)
}, organizationName, email, password string) error {
	if db == nil || strings.TrimSpace(organizationName) == "" || strings.TrimSpace(email) == "" || strings.TrimSpace(password) == "" {
		return ErrInvalidBootstrap
	}
	passwordHash, err := auth.HashPassword(password)
	if err != nil {
		return fmt.Errorf("standalone: hash bootstrap password: %w", err)
	}
	tx, err := db.Begin(ctx)
	if err != nil {
		return fmt.Errorf("standalone: begin bootstrap: %w", err)
	}
	defer func() { _ = tx.Rollback(ctx) }()

	if err := EnsureDefaultRolesAndPermissions(ctx, tx); err != nil {
		return err
	}

	var settingsCount int
	if err := tx.QueryRow(ctx, `SELECT count(*) FROM organization_settings`).Scan(&settingsCount); err != nil {
		return fmt.Errorf("standalone: count organization settings: %w", err)
	}
	if settingsCount > 1 {
		return fmt.Errorf("%w: multiple organization settings rows exist", ErrInvalidBootstrap)
	}
	if settingsCount == 0 {
		// The settings UUID is internal to the schema and never selected by clients.
		if _, err := tx.Exec(ctx, `INSERT INTO organization_settings (id, name) VALUES ($1, $2)`, uuid.New(), strings.TrimSpace(organizationName)); err != nil {
			return fmt.Errorf("standalone: create organization settings: %w", err)
		}
	}

	var userID uuid.UUID
	err = tx.QueryRow(ctx, `SELECT id FROM users WHERE lower(email) = lower($1) LIMIT 1`, strings.TrimSpace(email)).Scan(&userID)
	if errors.Is(err, pgx.ErrNoRows) {
		userID = uuid.New()
		if _, err := tx.Exec(ctx, `INSERT INTO users (id, email, password_hash, created_at, full_name) VALUES ($1, $2, $3, NOW(), $4)`, userID, strings.ToLower(strings.TrimSpace(email)), passwordHash, "Standalone Administrator"); err != nil {
			return fmt.Errorf("standalone: create administrator: %w", err)
		}
	} else if err != nil {
		return fmt.Errorf("standalone: read administrator: %w", err)
	}

	var roleID uuid.UUID
	if err := tx.QueryRow(ctx, `SELECT id FROM roles WHERE name = 'admin' LIMIT 1`).Scan(&roleID); err != nil {
		return fmt.Errorf("standalone: read admin role: %w", err)
	}
	if _, err := tx.Exec(ctx, `INSERT INTO user_roles (user_id, role_id) VALUES ($1, $2) ON CONFLICT DO NOTHING`, userID, roleID); err != nil {
		return fmt.Errorf("standalone: assign admin role: %w", err)
	}
	if err := ensurePayrollKPIFormula(ctx, tx, userID); err != nil {
		return err
	}
	if err := tx.Commit(ctx); err != nil {
		return fmt.Errorf("standalone: commit bootstrap: %w", err)
	}
	return nil
}
