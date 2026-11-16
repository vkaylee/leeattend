//go:build integration

package repository

import (
	"testing"

	"github.com/google/uuid"
)

func TestIntegrationScheduleEvidencePreservesAmendmentChain(t *testing.T) {
	pool, ctx := requireIntegrationDatabase(t)
	employerID, employeeID, employmentID, actorID := uuid.New(), uuid.New(), uuid.New(), uuid.New()
	templateID, versionID := uuid.New(), uuid.New()
	oldBatchID, newBatchID, oldRevisionID, newRevisionID := uuid.New(), uuid.New(), uuid.New(), uuid.New()
	mustExec := func(query string, args ...any) {
		t.Helper()
		if _, err := pool.Exec(ctx, query, args...); err != nil {
			t.Fatalf("fixture query failed: %v", err)
		}
	}

	mustExec(`INSERT INTO employers (id, name) VALUES ($1, 'evidence-chain-employer')`, employerID)
	mustExec(`INSERT INTO employees (id, employee_id_number, full_name, hire_date, original_hire_date) VALUES ($1, $2, 'evidence-chain-employee', DATE '2026-01-01', DATE '2026-01-01')`, employeeID, employeeID.String())
	mustExec(`INSERT INTO employments (id, employee_id, employer_id, status, start_date, end_date) VALUES ($1, $2, $3, 'active', DATE '2026-11-20', DATE '2026-11-20')`, employmentID, employeeID, employerID)
	mustExec(`INSERT INTO users (id, email, password_hash) VALUES ($1, $2, 'integration-test-hash')`, actorID, actorID.String()+"@invalid.test")
	mustExec(`INSERT INTO shift_templates (id, name) VALUES ($1, 'evidence-chain-shift')`, templateID)
	mustExec(`INSERT INTO shift_versions (id, template_id, version_number, name, effective_from, change_type, created_by) VALUES ($1, $2, 1, 'evidence-chain-version', DATE '2026-01-01', 'create', $3)`, versionID, templateID, actorID)
	mustExec(`INSERT INTO schedule_publication_batches (id, idempotency_key, request_hash, mode, status, requested_by, operation, response_snapshot, reason, timezone, range_start, range_end) VALUES ($1, $2, repeat('a', 64), 'rotating', 'published', $3, 'publish', '{}'::jsonb, 'first publication', 'UTC', DATE '2026-11-20', DATE '2026-11-20')`, oldBatchID, oldBatchID.String(), actorID)
	mustExec(`INSERT INTO schedule_publication_batches (id, idempotency_key, request_hash, mode, status, requested_by, operation, response_snapshot, reason, timezone, range_start, range_end) VALUES ($1, $2, repeat('b', 64), 'rotating', 'published', $3, 'replan', '{}'::jsonb, 'amendment', 'UTC', DATE '2026-11-20', DATE '2026-11-20')`, newBatchID, newBatchID.String(), actorID)

	mustExec(`INSERT INTO schedule_day_revisions (id, employment_id, employer_id, business_date, timezone, revision_number, state, publication_batch_id, source, source_snapshot, resolver_version, reason, published_by, published_actor_type, published_at) VALUES ($1, $2, $3, DATE '2026-11-20', 'UTC', 1, 'draft', $4, 'pattern', '{}'::jsonb, 'schedule-resolver-v1', 'first publication', $5, 'human', NULL)`, oldRevisionID, employmentID, employerID, oldBatchID, actorID)
	mustExec(`SELECT set_config('schedule.lifecycle_publication', 'on', true)`)
	mustExec(`INSERT INTO work_segments (employment_id, date, sort_order, scheduled_start, scheduled_end, segment_type, segment_status, compensation_rate, gap_compensation_before, gap_compensation_rate_before, source, source_detail, generation_metadata, template_id, schedule_revision_id, selected_shift_version_id, scheduled_start_at, scheduled_end_at, breaks_snapshot) VALUES ($1, DATE '2026-11-20', 0, '08:00', '17:00', 'work', 'scheduled', 1, 'unpaid', 0, 'pattern', '{}'::jsonb, '{}'::jsonb, $2, $3, $4, TIMESTAMPTZ '2026-11-20 08:00:00+00', TIMESTAMPTZ '2026-11-20 17:00:00+00', '[]'::jsonb)`, employmentID, templateID, oldRevisionID, versionID)
	mustExec(`UPDATE schedule_day_revisions SET state = 'published', published_at = CURRENT_TIMESTAMP WHERE id = $1`, oldRevisionID)

	mustExec(`INSERT INTO schedule_day_revisions (id, employment_id, employer_id, business_date, timezone, revision_number, state, previous_revision_id, publication_batch_id, source, source_snapshot, resolver_version, reason, published_by, published_actor_type, published_at) VALUES ($1, $2, $3, DATE '2026-11-20', 'UTC', 2, 'draft', $4, $5, 'manual', '{}'::jsonb, 'schedule-resolver-v1', 'amendment', $6, 'human', NULL)`, newRevisionID, employmentID, employerID, oldRevisionID, newBatchID, actorID)
	mustExec(`INSERT INTO work_segments (employment_id, date, sort_order, scheduled_start, scheduled_end, segment_type, segment_status, compensation_rate, gap_compensation_before, gap_compensation_rate_before, source, source_detail, generation_metadata, template_id, schedule_revision_id, selected_shift_version_id, scheduled_start_at, scheduled_end_at, breaks_snapshot) VALUES ($1, DATE '2026-11-20', 0, '09:00', '18:00', 'work', 'scheduled', 1, 'unpaid', 0, 'manual', '{}'::jsonb, '{}'::jsonb, $2, $3, $4, TIMESTAMPTZ '2026-11-20 09:00:00+00', TIMESTAMPTZ '2026-11-20 18:00:00+00', '[]'::jsonb)`, employmentID, templateID, newRevisionID, versionID)
	mustExec(`UPDATE schedule_day_revisions SET state = 'superseded', published_at = COALESCE(published_at, CURRENT_TIMESTAMP) WHERE id = $1`, oldRevisionID)
	mustExec(`UPDATE schedule_day_revisions SET state = 'published', published_at = CURRENT_TIMESTAMP WHERE id = $1`, newRevisionID)

	evidence, err := (ScheduleRepository{DB: pool}).GetScheduleEvidence(ctx, actorID, employmentID, "2026-11-20")
	if err != nil {
		t.Fatalf("get schedule evidence: %v", err)
	}
	if len(evidence.Revisions) != 2 || len(evidence.Segments) != 2 || len(evidence.DiffChain) != 2 {
		t.Fatalf("evidence counts revisions=%d segments=%d diff_chain=%d", len(evidence.Revisions), len(evidence.Segments), len(evidence.DiffChain))
	}
	if evidence.Revisions[1].PreviousRevisionID == nil || *evidence.Revisions[1].PreviousRevisionID != oldRevisionID {
		t.Fatalf("replacement revision lost lineage: %+v", evidence.Revisions)
	}
	if evidence.DiffChain[1].PreviousRevisionID == nil || *evidence.DiffChain[1].PreviousRevisionID != oldRevisionID {
		t.Fatalf("evidence diff chain lost lineage: %+v", evidence.DiffChain)
	}
}
