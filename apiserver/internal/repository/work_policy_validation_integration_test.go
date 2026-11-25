//go:build integration

package repository

import (
	"context"
	"errors"
	"testing"

	"github.com/google/uuid"
	"github.com/shopspring/decimal"
	"github.com/vkaylee/leeattend/apiserver/internal/api"
	"github.com/vkaylee/leeattend/apiserver/internal/domain/ids"
	"github.com/vkaylee/leeattend/apiserver/internal/domain/workpolicy"
)

func TestIntegrationValidateWorkPolicyVersionChecksLiveReferences(t *testing.T) {
	pool, ctx := requireIntegrationDatabase(t)
	tx, err := pool.Begin(ctx)
	if err != nil {
		t.Fatalf("begin fixture transaction: %v", err)
	}
	defer func() { _ = tx.Rollback(ctx) }()

	userID, patternID, otPolicyID, otVerID, groupID, versionID := uuid.New(), uuid.New(), uuid.New(), uuid.New(), uuid.New(), uuid.New()
	if _, err := tx.Exec(ctx, `INSERT INTO users (id, email, password_hash) VALUES ($1, $2, 'integration-test-hash')`, userID, userID.String()+"@invalid.test"); err != nil {
		t.Fatalf("insert user: %v", err)
	}
	if _, err := tx.Exec(ctx, `INSERT INTO working_patterns (id, name, is_active) VALUES ($1, 'validation-pattern', true)`, patternID); err != nil {
		t.Fatalf("insert working pattern: %v", err)
	}
	if _, err := tx.Exec(ctx, `INSERT INTO overtime_policy_definitions (id, name) VALUES ($1, 'validation-overtime')`, otPolicyID); err != nil {
		t.Fatalf("insert overtime policy: %v", err)
	}
	if _, err := tx.Exec(ctx, `INSERT INTO overtime_policy_versions (id, policy_id, version_no, status, rules, created_by, published_by, published_at) VALUES ($1, $2, 1, 'published', '[{"priority":10,"day_type":"any","days_of_week":127,"multiplier":"1.5"}]'::jsonb, $3, $3, NOW())`, otVerID, otPolicyID, userID); err != nil {
		t.Fatalf("insert overtime policy version: %v", err)
	}
	if _, err := tx.Exec(ctx, `INSERT INTO work_policy_groups (id, code, name, created_by) VALUES ($1, $2, 'validation-group', $3)`, groupID, "VAL-"+groupID.String()[:8], userID); err != nil {
		t.Fatalf("insert work policy group: %v", err)
	}
	if _, err := tx.Exec(ctx, `INSERT INTO work_policy_versions (id, group_id, version_no, status, effective_from, created_by, working_pattern_id, pay_method, monthly_denominator, monthly_standard_days, overtime_policy_version_id) VALUES ($1, $2, 1, 'draft', DATE '2026-01-01', $3, $4, 'monthly', 'fixed', 26, $5)`, versionID, groupID, userID, patternID, otVerID); err != nil {
		t.Fatalf("insert work policy version: %v", err)
	}

	repo := WorkPolicyRepository{DB: tx}
	detail, err := repo.GetWorkPolicyVersion(ctx, workpolicy.GroupID(groupID), workpolicy.VersionID(versionID))
	if err != nil {
		t.Fatalf("load persisted policy detail: %v", err)
	}
	if detail.ID != versionID || detail.GroupID != groupID || detail.EffectiveFrom != "2026-01-01" {
		t.Fatalf("unexpected persisted policy detail: %+v", detail)
	}
	versions, err := repo.ListWorkPolicyGroupVersions(ctx, workpolicy.GroupID(groupID), "draft", 1, 20)
	if err != nil || versions.Total != 1 || len(versions.Data) != 1 || versions.Data[0].ID != versionID {
		t.Fatalf("unexpected policy versions: %+v err=%v", versions, err)
	}

	valid, err := repo.ValidateWorkPolicyVersion(ctx, workpolicy.GroupID(groupID), workpolicy.VersionID(versionID))
	if err != nil || !valid.Valid || len(valid.Blockers) != 0 {
		t.Fatalf("active references should validate: %+v err=%v", valid, err)
	}

	if _, err := tx.Exec(ctx, `UPDATE working_patterns SET is_active = false WHERE id = $1`, patternID); err != nil {
		t.Fatalf("deactivate working pattern: %v", err)
	}
	blocked, err := repo.ValidateWorkPolicyVersion(ctx, workpolicy.GroupID(groupID), workpolicy.VersionID(versionID))
	if err != nil || blocked.Valid || len(blocked.Blockers) != 1 || blocked.Blockers[0].Code != "working_pattern_inactive_or_missing" {
		t.Fatalf("inactive pattern must block validation: %+v err=%v", blocked, err)
	}
}

func TestIntegrationPublishWorkPolicyVersionRejectsOverlapWithoutAutoClose(t *testing.T) {
	pool, ctx := requireIntegrationDatabase(t)
	tx, err := pool.Begin(ctx)
	if err != nil {
		t.Fatalf("begin fixture transaction: %v", err)
	}
	defer func() { _ = tx.Rollback(ctx) }()

	userID, otPolicyID, otVerID := uuid.New(), uuid.New(), uuid.New()
	if _, err := tx.Exec(ctx, `INSERT INTO users (id, email, password_hash) VALUES ($1, $2, 'integration-test-hash')`, userID, userID.String()+"@invalid.test"); err != nil {
		t.Fatalf("insert user: %v", err)
	}
	if _, err := tx.Exec(ctx, `INSERT INTO overtime_policy_definitions (id, name) VALUES ($1, 'overlap-ot')`, otPolicyID); err != nil {
		t.Fatalf("insert ot policy: %v", err)
	}
	if _, err := tx.Exec(ctx, `INSERT INTO overtime_policy_versions (id, policy_id, version_no, status, rules, created_by, published_by, published_at) VALUES ($1, $2, 1, 'published', '[{"priority":10,"day_type":"any","days_of_week":127,"multiplier":"1.5"}]'::jsonb, $3, $3, NOW())`, otVerID, otPolicyID, userID); err != nil {
		t.Fatalf("insert ot version: %v", err)
	}

	repo := WorkPolicyRepository{DB: tx, BeginTx: func(context.Context) (TxBeginner, error) { return tx, nil }}
	denom := workpolicy.MonthlyDenominatorFixed
	days26 := decimal.NewFromInt(26)
	v1, err := repo.CreateWorkPolicyGroup(ctx, workpolicy.CreateGroupInput{
		Code: "OVL-" + uuid.NewString()[:8], Name: "Overlap Test",
		Draft: workpolicy.VersionDraftFields{
			EffectiveFrom: "2026-01-01", MinutesPerCredit: 480, WorkUnitMinutes: 1, RoundingPolicy: "none",
			EarlyArrivalStrategy: "clamp", BreakAutoDeduct: true, OTAggregationMode: "aggregate",
			PayMethod: workpolicy.PayMethodMonthly, MonthlyDenominator: &denom, MonthlyStandardDays: &days26,
			PaidLeaveCreditMode: workpolicy.PaidLeaveCreditStandardDay, OvertimePolicyVersionID: otVerID,
		},
	}, ids.UserID(userID))
	if err != nil {
		t.Fatalf("create group: %v", err)
	}
	if _, err := repo.PublishWorkPolicyVersion(ctx, workpolicy.PublishVersionInput{
		GroupID: workpolicy.GroupID(v1.GroupID), VersionID: workpolicy.VersionID(v1.ID), ExpectedRowVersion: v1.RowVersion, ActorID: ids.UserID(userID),
	}); err != nil {
		t.Fatalf("publish v1: %v", err)
	}

	days22 := decimal.NewFromInt(22)
	v2, err := repo.CreateWorkPolicyVersion(ctx, workpolicy.CreateVersionInput{
		GroupID: workpolicy.GroupID(v1.GroupID),
		Draft: &workpolicy.VersionDraftFields{
			EffectiveFrom: "2026-03-01", MinutesPerCredit: 480, WorkUnitMinutes: 1, RoundingPolicy: "none",
			EarlyArrivalStrategy: "clamp", BreakAutoDeduct: true, OTAggregationMode: "aggregate",
			PayMethod: workpolicy.PayMethodMonthly, MonthlyDenominator: &denom, MonthlyStandardDays: &days22,
			PaidLeaveCreditMode: workpolicy.PaidLeaveCreditStandardDay, OvertimePolicyVersionID: otVerID,
		},
	}, ids.UserID(userID))
	if err != nil {
		t.Fatalf("create draft v2: %v", err)
	}

	// Without auto_close_previous -> 409 conflict
	if _, err := repo.PublishWorkPolicyVersion(ctx, workpolicy.PublishVersionInput{
		GroupID: workpolicy.GroupID(v1.GroupID), VersionID: workpolicy.VersionID(v2.ID), ExpectedRowVersion: v2.RowVersion, AutoClosePrevious: false, ActorID: ids.UserID(userID),
	}); !errors.Is(err, api.ErrWorkPolicyVersionConflict) {
		t.Fatalf("expected conflict without auto_close_previous, got %v", err)
	}

	// With auto_close_previous -> succeeds and closes v1 at 2026-03-01
	pubV2, err := repo.PublishWorkPolicyVersion(ctx, workpolicy.PublishVersionInput{
		GroupID: workpolicy.GroupID(v1.GroupID), VersionID: workpolicy.VersionID(v2.ID), ExpectedRowVersion: v2.RowVersion, AutoClosePrevious: true, IdempotencyKey: "pub-v2-idem", ActorID: ids.UserID(userID),
	})
	if err != nil || pubV2.Status != workpolicy.StatusPublished {
		t.Fatalf("publish v2 with auto_close_previous failed: %+v err=%v", pubV2, err)
	}
	closedV1, err := repo.GetWorkPolicyVersion(ctx, workpolicy.GroupID(v1.GroupID), workpolicy.VersionID(v1.ID))
	if err != nil || closedV1.EffectiveTo == nil || *closedV1.EffectiveTo != "2026-03-01" {
		t.Fatalf("expected v1 closed at 2026-03-01, got %+v err=%v", closedV1.EffectiveTo, err)
	}

	// Idempotency replay returns same published v2 without error
	replayed, err := repo.PublishWorkPolicyVersion(ctx, workpolicy.PublishVersionInput{
		GroupID: workpolicy.GroupID(v1.GroupID), VersionID: workpolicy.VersionID(v2.ID), ExpectedRowVersion: v2.RowVersion, IdempotencyKey: "pub-v2-idem", ActorID: ids.UserID(userID),
	})
	if err != nil || replayed.ID != pubV2.ID {
		t.Fatalf("expected idempotent replay, got %+v err=%v", replayed, err)
	}
}
