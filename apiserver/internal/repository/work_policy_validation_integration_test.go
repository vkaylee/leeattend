//go:build integration

package repository

import (
	"context"
	"errors"
	"testing"

	"github.com/google/uuid"
	"github.com/vkaylee/leeattend/apiserver/internal/api"
	"github.com/vkaylee/leeattend/apiserver/internal/domain/ids"
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
	detail, err := repo.GetWorkPolicyGroupVersion(ctx, workpolicy.GroupID(groupID), workpolicy.VersionID(versionID))
	if err != nil {
		t.Fatalf("load persisted policy detail: %v", err)
	}
	if detail.ID != versionID || detail.GroupID != groupID || detail.EffectiveFrom != "2026-01-01" {
		t.Fatalf("unexpected persisted policy detail: %+v", detail)
	}
	history, err := repo.ListWorkPolicyGroupHistory(ctx, workpolicy.GroupID(groupID), 1, 20)
	if err != nil {
		t.Fatalf("load policy history: %v", err)
	}
	if history.Total != 1 || len(history.Data) != 1 || history.Data[0].ID != versionID {
		t.Fatalf("unexpected policy history: %+v", history)
	}

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

func TestIntegrationCreateWorkPolicyGroupVersionRollsBackDefinitionsOnVersionConflict(t *testing.T) {
	pool, ctx := requireIntegrationDatabase(t)
	tx, err := pool.Begin(ctx)
	if err != nil {
		t.Fatalf("begin fixture transaction: %v", err)
	}
	defer func() { _ = tx.Rollback(ctx) }()

	userID, groupID := uuid.New(), uuid.New()
	attendanceID, patternID, overtimeID := uuid.New(), uuid.New(), uuid.New()
	if _, err := tx.Exec(ctx, `INSERT INTO users (id, email, password_hash) VALUES ($1, $2, 'integration-test-hash')`, userID, userID.String()+"@invalid.test"); err != nil {
		t.Fatalf("insert user: %v", err)
	}
	if _, err := tx.Exec(ctx, `INSERT INTO attendance_policies (id, name) VALUES ($1, 'rollback-attendance')`, attendanceID); err != nil {
		t.Fatalf("insert attendance policy: %v", err)
	}
	if _, err := tx.Exec(ctx, `INSERT INTO working_patterns (id, name, is_active) VALUES ($1, 'rollback-pattern', true)`, patternID); err != nil {
		t.Fatalf("insert working pattern: %v", err)
	}
	if _, err := tx.Exec(ctx, `INSERT INTO overtime_policy_definitions (id, name) VALUES ($1, 'rollback-overtime')`, overtimeID); err != nil {
		t.Fatalf("insert overtime policy: %v", err)
	}
	if _, err := tx.Exec(ctx, `INSERT INTO labor_policy_groups (id, name, created_by) VALUES ($1, 'rollback-group', $2)`, groupID, userID); err != nil {
		t.Fatalf("insert policy group: %v", err)
	}
	if _, err := tx.Exec(ctx, `INSERT INTO work_credit_policies (id, name, minutes_per_credit, attendance_policy_id, working_pattern_id, created_by) VALUES ($1, 'existing-credit', 480, $2, $3, $4)`, uuid.New(), attendanceID, patternID, userID); err != nil {
		t.Fatalf("insert existing credit policy: %v", err)
	}
	if _, err := tx.Exec(ctx, `INSERT INTO pay_basis_policies (id, name, method, monthly_denominator, monthly_standard_days, created_by) VALUES ($1, 'existing-pay', 'monthly', 'fixed', 26, $2)`, uuid.New(), userID); err != nil {
		t.Fatalf("insert existing pay policy: %v", err)
	}
	if _, err := tx.Exec(ctx, `INSERT INTO labor_policy_group_versions (id, group_id, work_credit_policy_id, pay_basis_policy_id, overtime_policy_id, effective_from, effective_to, created_by) SELECT $1, $2, wc.id, pb.id, $3, DATE '2026-01-01', DATE '2026-01-31', $4 FROM work_credit_policies wc CROSS JOIN pay_basis_policies pb WHERE wc.name = 'existing-credit' AND pb.name = 'existing-pay'`, uuid.New(), groupID, overtimeID, userID); err != nil {
		t.Fatalf("insert existing version: %v", err)
	}

	repo := WorkPolicyRepository{DB: tx, BeginTx: func(context.Context) (TxBeginner, error) { return tx, nil }}
	_, err = repo.CreateWorkPolicyGroupVersion(ctx, workpolicy.CreateGroupVersionInput{
		GroupID:            workpolicy.GroupID(groupID),
		Name:               "rollback-group",
		MinutesPerCredit:   450,
		AttendancePolicyID: ids.AttendancePolicyID(attendanceID),
		WorkingPatternID:   ids.WorkingPatternID(patternID),
		OTAggregationMode:  "aggregate",
		PayMethod:          workpolicy.PayMethodMonthly,
		OvertimePolicyID:   ids.OvertimePolicyID(overtimeID),
		EffectiveFrom:      "2026-01-01",
	}, ids.UserID(userID))
	if !errors.Is(err, api.ErrConflict) {
		t.Fatalf("create conflicting version error = %v, want conflict", err)
	}
	var count int
	if err := tx.QueryRow(ctx, `SELECT count(*) FROM work_credit_policies WHERE name = 'rollback-group'`).Scan(&count); err != nil {
		t.Fatalf("count rolled back credit policies: %v", err)
	}
	if count != 0 {
		t.Fatalf("rolled-back credit policy count = %d, want 0", count)
	}
	if err := tx.QueryRow(ctx, `SELECT count(*) FROM pay_basis_policies WHERE name = 'rollback-group'`).Scan(&count); err != nil {
		t.Fatalf("count rolled back pay policies: %v", err)
	}
	if count != 0 {
		t.Fatalf("rolled-back pay policy count = %d, want 0", count)
	}

	// Rollback leaves no partial definitions, so retry with a non-overlapping date can proceed.
	_, err = repo.CreateWorkPolicyGroupVersion(ctx, workpolicy.CreateGroupVersionInput{
		GroupID:            workpolicy.GroupID(groupID),
		Name:               "rollback-group",
		MinutesPerCredit:   450,
		AttendancePolicyID: ids.AttendancePolicyID(attendanceID),
		WorkingPatternID:   ids.WorkingPatternID(patternID),
		OTAggregationMode:  "aggregate",
		PayMethod:          workpolicy.PayMethodMonthly,
		OvertimePolicyID:   ids.OvertimePolicyID(overtimeID),
		EffectiveFrom:      "2026-02-01",
	}, ids.UserID(userID))
	if err != nil {
		t.Fatalf("retry after rolled-back conflict: %v", err)
	}
}
