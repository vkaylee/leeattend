//go:build integration

package repository

import (
	"context"
	"errors"
	"testing"

	"github.com/google/uuid"
	"github.com/vkaylee/leeattend/apiserver/internal/api"
)

func TestIntegrationCrossEmploymentOverlapIsRejected(t *testing.T) {
	pool, ctx := requireIntegrationDatabase(t)
	employerA, employerB, employeeID := uuid.New(), uuid.New(), uuid.New()
	employmentA, employmentB, actorID := uuid.New(), uuid.New(), uuid.New()
	templateID, versionID, batchID, revisionID := uuid.New(), uuid.New(), uuid.New(), uuid.New()
	mustExec := func(query string, args ...any) {
		t.Helper()
		if _, err := pool.Exec(ctx, query, args...); err != nil {
			t.Fatalf("fixture query failed: %v", err)
		}
	}
	mustExec(`INSERT INTO employers (id, name) VALUES ($1, 'overlap-employer-a'), ($2, 'overlap-employer-b')`, employerA, employerB)
	mustExec(`INSERT INTO employees (id, employee_id_number, full_name, hire_date, original_hire_date) VALUES ($1, $2, 'overlap-employee', DATE '2026-01-01', DATE '2026-01-01')`, employeeID, employeeID.String())
	mustExec(`INSERT INTO employments (id, employee_id, employer_id, status, start_date) VALUES ($1, $3, $2, 'active', DATE '2026-01-01'), ($4, $3, $5, 'active', DATE '2026-01-01')`, employmentA, employerA, employeeID, employmentB, employerB)
	mustExec(`INSERT INTO users (id, email, password_hash) VALUES ($1, $2, 'integration-test-hash')`, actorID, actorID.String()+"@invalid.test")
	mustExec(`INSERT INTO shift_templates (id, name) VALUES ($1, 'overlap-template')`, templateID)
	mustExec(`INSERT INTO shift_versions (id, template_id, version_number, name, effective_from, change_type, created_by) VALUES ($1, $2, 1, 'overlap-version', DATE '2026-01-01', 'create', $3)`, versionID, templateID, actorID)
	mustExec(`INSERT INTO schedule_publication_batches (id, idempotency_key, request_hash, mode, status, requested_by, operation, response_snapshot, reason, timezone, range_start, range_end) VALUES ($1, $2, repeat('c', 64), 'rotating', 'published', $3, 'publish', '{}'::jsonb, 'overlap fixture', 'UTC', DATE '2026-11-20', DATE '2026-11-20')`, batchID, batchID.String(), actorID)
	mustExec(`INSERT INTO schedule_day_revisions (id, employment_id, employer_id, business_date, timezone, revision_number, state, publication_batch_id, source, source_snapshot, resolver_version, reason, published_by, published_actor_type, published_at) VALUES ($1, $2, $3, DATE '2026-11-20', 'UTC', 1, 'draft', $4, 'manual', '{}'::jsonb, 'integration-test', 'overlap fixture', $5, 'human', NULL)`, revisionID, employmentA, employerA, batchID, actorID)
	mustExec(`SELECT set_config('schedule.lifecycle_publication', 'on', true)`)
	mustExec(`INSERT INTO work_segments (employment_id, date, sort_order, scheduled_start, scheduled_end, segment_type, segment_status, compensation_rate, gap_compensation_before, gap_compensation_rate_before, source, source_detail, generation_metadata, template_id, schedule_revision_id, selected_shift_version_id, scheduled_start_at, scheduled_end_at, breaks_snapshot) VALUES ($1, DATE '2026-11-20', 0, '08:00', '17:00', 'work', 'scheduled', 1, 'unpaid', 0, 'manual', '{}'::jsonb, '{}'::jsonb, $2, $3, $4, TIMESTAMPTZ '2026-11-20 08:00:00+00', TIMESTAMPTZ '2026-11-20 17:00:00+00', '[]'::jsonb)`, employmentA, templateID, revisionID, versionID)
	mustExec(`UPDATE schedule_day_revisions SET state = 'published', published_at = CURRENT_TIMESTAMP WHERE id = $1`, revisionID)

	tx, err := pool.Begin(ctx)
	if err != nil {
		t.Fatalf("begin overlap transaction: %v", err)
	}
	defer func() { _ = tx.Rollback(ctx) }()
	err = rejectConcreteScheduleOverlaps(ctx, tx, []map[string]any{{
		"employment_id": employmentB.String(),
		"business_date": "2026-11-20",
		"status":        "work",
		"intervals": []map[string]any{{
			"sort_order":       0,
			"start":            "08:00:00",
			"end":              "17:00:00",
			"start_at":         "2026-11-20T08:00:00Z",
			"end_at":           "2026-11-20T17:00:00Z",
			"shift_version_id": versionID.String(),
			"breaks":           []map[string]any{},
		}},
	}})
	if !errors.Is(err, api.ErrScheduleConflict) {
		t.Fatalf("cross-employment overlap error = %v, want schedule conflict", err)
	}
}

var _ context.Context
