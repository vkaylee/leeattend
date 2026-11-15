//go:build integration

package repository

import (
	"testing"

	"github.com/google/uuid"
	"github.com/vkaylee/leeattend/apiserver/internal/domain/workpolicy"
)

func TestIntegrationValidateWorkPolicyGroupVersionChecksLiveReferences(t *testing.T) {
	pool, ctx := requireIntegrationDatabase(t)
	tx, err := pool.Begin(ctx)
	if err != nil {
		t.Fatalf("begin fixture transaction: %v", err)
	}
	defer func() { _ = tx.Rollback(ctx) }()

	userID := uuid.New()
	groupID := uuid.New()
	versionID := uuid.New()
	workCreditID := uuid.New()
	payBasisID := uuid.New()
	attendanceID := uuid.New()
	patternID := uuid.New()
	overtimeID := uuid.New()
	if _, err := tx.Exec(ctx, `INSERT INTO users (id, email, password_hash) VALUES ($1, $2, 'integration-test-hash')`, userID, userID.String()+"@invalid.test"); err != nil {
		t.Fatalf("insert user: %v", err)
	}
	if _, err := tx.Exec(ctx, `INSERT INTO attendance_policies (id, name) VALUES ($1, 'validation-attendance')`, attendanceID); err != nil {
		t.Fatalf("insert attendance policy: %v", err)
	}
	if _, err := tx.Exec(ctx, `INSERT INTO working_patterns (id, name, is_active) VALUES ($1, 'validation-pattern', true)`, patternID); err != nil {
		t.Fatalf("insert working pattern: %v", err)
	}
	if _, err := tx.Exec(ctx, `INSERT INTO overtime_policy_definitions (id, name) VALUES ($1, 'validation-overtime')`, overtimeID); err != nil {
		t.Fatalf("insert overtime policy: %v", err)
	}
	if _, err := tx.Exec(ctx, `INSERT INTO labor_policy_groups (id, name, created_by) VALUES ($1, 'validation-group', $2)`, groupID, userID); err != nil {
		t.Fatalf("insert labor policy group: %v", err)
	}
	if _, err := tx.Exec(ctx, `INSERT INTO work_credit_policies (id, name, minutes_per_credit, attendance_policy_id, working_pattern_id, created_by) VALUES ($1, 'validation-credit', 480, $2, $3, $4)`, workCreditID, attendanceID, patternID, userID); err != nil {
		t.Fatalf("insert work credit policy: %v", err)
	}
	if _, err := tx.Exec(ctx, `INSERT INTO pay_basis_policies (id, name, method, monthly_denominator, monthly_standard_days, created_by) VALUES ($1, 'validation-pay', 'monthly', 'fixed', 26, $2)`, payBasisID, userID); err != nil {
		t.Fatalf("insert pay basis policy: %v", err)
	}
	if _, err := tx.Exec(ctx, `INSERT INTO labor_policy_group_versions (id, group_id, work_credit_policy_id, pay_basis_policy_id, overtime_policy_id, effective_from, created_by) VALUES ($1, $2, $3, $4, $5, DATE '2026-01-01', $6)`, versionID, groupID, workCreditID, payBasisID, overtimeID, userID); err != nil {
		t.Fatalf("insert policy version: %v", err)
	}

	repo := WorkPolicyRepository{DB: tx}
	valid, err := repo.ValidateWorkPolicyGroupVersion(ctx, workpolicy.GroupID(groupID), workpolicy.VersionID(versionID))
	if err != nil {
		t.Fatalf("validate active references: %v", err)
	}
	if !valid.Valid || len(valid.Blockers) != 0 {
		t.Fatalf("active references should validate: %+v", valid)
	}

	if _, err := tx.Exec(ctx, `UPDATE working_patterns SET is_active = false WHERE id = $1`, patternID); err != nil {
		t.Fatalf("deactivate working pattern: %v", err)
	}
	blocked, err := repo.ValidateWorkPolicyGroupVersion(ctx, workpolicy.GroupID(groupID), workpolicy.VersionID(versionID))
	if err != nil {
		t.Fatalf("validate inactive pattern: %v", err)
	}
	if blocked.Valid || len(blocked.Blockers) != 1 || blocked.Blockers[0].Code != "reference_missing_or_inactive" {
		t.Fatalf("inactive pattern must block validation: %+v", blocked)
	}
}
