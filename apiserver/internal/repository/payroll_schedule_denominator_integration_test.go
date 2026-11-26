//go:build integration

package repository

import (
	"testing"

	"github.com/google/uuid"
)

func TestIntegrationLoadScheduledWorkdayCountsUsesCanonicalWorkSegments(t *testing.T) {
	pool, ctx := requireIntegrationDatabase(t)
	tx, err := pool.Begin(ctx)
	if err != nil {
		t.Fatalf("begin fixture transaction: %v", err)
	}
	defer func() { _ = tx.Rollback(ctx) }()

	employerID := uuid.New()
	employeeID := uuid.New()
	employmentID := uuid.New()
	templateID := uuid.New()
	if _, err := tx.Exec(ctx, `INSERT INTO employers (id, name) VALUES ($1, 'schedule-denominator-test-employer')`, employerID); err != nil {
		t.Fatalf("insert employer: %v", err)
	}
	seedIntegrationEmployeeGraphWithRange(t, ctx, tx, employeeID, employerID, employmentID, employeeID.String(), "schedule-denominator-test-employee", "2026-01-01", "2026-01-01", "")
	if _, err := tx.Exec(ctx, `INSERT INTO shift_templates (id, name) VALUES ($1, 'schedule-denominator-test-template')`, templateID); err != nil {
		t.Fatalf("insert shift template: %v", err)
	}

	sortOrder := 0
	insertSegment := func(date, status string) {
		t.Helper()
		sortOrder++
		_, err := tx.Exec(ctx, `INSERT INTO work_segments (employment_id, date, sort_order, scheduled_start, scheduled_end, segment_type, segment_status, template_id) VALUES ($1, $2::date, $3, '08:00', '12:00', 'work', $4::segment_status_type, $5)`, employmentID, date, sortOrder, status, templateID)
		if err != nil {
			t.Fatalf("insert work segment: %v", err)
		}
	}
	insertSegment("2026-01-05", "scheduled")
	insertSegment("2026-01-05", "scheduled")
	insertSegment("2026-01-06", "completed")
	insertSegment("2026-01-07", "cancelled")

	bundles, err := (PayrollBatchLoader{}).LoadForEmployments(ctx, tx, []uuid.UUID{employmentID}, 2026, 1)
	if err != nil {
		t.Fatalf("load batch period inputs: %v", err)
	}
	counts := bundles[employmentID].Input.ScheduledWorkdayCounts
	if len(counts) != 2 || counts["2026-01-05"] != 1 || counts["2026-01-06"] != 1 {
		t.Fatalf("unexpected scheduled workday counts: %#v", counts)
	}
	if _, exists := counts["2026-01-07"]; exists {
		t.Fatalf("cancelled work segment must not count: %#v", counts)
	}
}

func TestIntegrationLoadScheduledWorkdayCountsIgnoresSupersededAndLegacyOnOfficialDay(t *testing.T) {
	pool, ctx := requireIntegrationDatabase(t)
	tx, err := pool.Begin(ctx)
	if err != nil {
		t.Fatalf("begin fixture transaction: %v", err)
	}
	defer func() { _ = tx.Rollback(ctx) }()

	employerID, employeeID, employmentID, actorID := uuid.New(), uuid.New(), uuid.New(), uuid.New()
	templateID, versionID, batchID := uuid.New(), uuid.New(), uuid.New()
	mustExec := func(query string, args ...any) {
		t.Helper()
		if _, err := tx.Exec(ctx, query, args...); err != nil {
			t.Fatalf("fixture exec failed: %v", err)
		}
	}
	mustExec(`INSERT INTO employers (id, name) VALUES ($1, 'schedule-denominator-official-employer')`, employerID)
	seedIntegrationEmployeeGraphWithRange(t, ctx, tx, employeeID, employerID, employmentID, employeeID.String(), "schedule-denominator-official-employee", "2026-01-01", "2026-01-01", "")
	mustExec(`INSERT INTO users (id, email, password_hash) VALUES ($1, $2, 'integration-test-hash')`, actorID, actorID.String()+"@invalid.test")
	mustExec(`INSERT INTO shift_templates (id, name) VALUES ($1, 'schedule-denominator-official-template')`, templateID)
	mustExec(`INSERT INTO shift_versions (id, template_id, version_number, name, effective_from, change_type, created_by) VALUES ($1, $2, 1, 'v1', DATE '2026-01-01', 'create', $3)`, versionID, templateID, actorID)
	mustExec(`INSERT INTO schedule_publication_batches (id, idempotency_key, request_hash, mode, status, requested_by, operation, response_snapshot, reason, timezone, range_start, range_end) VALUES ($1, $2, repeat('c', 64), 'rotating', 'published', $3, 'publish', '{}'::jsonb, 'payroll official filter test', 'UTC', DATE '2026-01-10', DATE '2026-01-12')`, batchID, batchID.String(), actorID)
	mustExec(`SELECT set_config('schedule.lifecycle_publication', 'on', true)`)

	// Seed the legacy segment before publication; guards reject later legacy mutations.
	mustExec(`INSERT INTO work_segments (employment_id, date, sort_order, scheduled_start, scheduled_end, segment_type, segment_status, template_id) VALUES ($1, DATE '2026-01-10', 9, '08:00', '17:00', 'work', 'scheduled', $2)`, employmentID, templateID)

	// 2026-01-10: superseded work revision + current explicit_off revision + legacy unlinked segment -> must NOT count as scheduled workday.
	supOffRev, curOffRev, supOffSeg := uuid.New(), uuid.New(), uuid.New()
	mustExec(`INSERT INTO schedule_day_revisions (id, employment_id, employer_id, business_date, timezone, revision_number, state, publication_batch_id, source, source_snapshot, resolver_version, reason, published_by, published_actor_type, published_at) VALUES ($1, $2, $3, DATE '2026-01-10', 'UTC', 1, 'draft', $4, 'pattern', '{}'::jsonb, 'schedule-resolver-v1', 'rev1', $5, 'human', NULL)`, supOffRev, employmentID, employerID, batchID, actorID)
	mustExec(`INSERT INTO work_segments (id, employment_id, date, sort_order, scheduled_start, scheduled_end, segment_type, segment_status, source, template_id, schedule_revision_id, selected_shift_version_id, scheduled_start_at, scheduled_end_at, breaks_snapshot) VALUES ($1, $2, DATE '2026-01-10', 0, '08:00', '17:00', 'work', 'scheduled', 'pattern', $3, $4, $5, TIMESTAMPTZ '2026-01-10 08:00:00+00', TIMESTAMPTZ '2026-01-10 17:00:00+00', '[]'::jsonb)`, supOffSeg, employmentID, templateID, supOffRev, versionID)
	mustExec(`UPDATE schedule_day_revisions SET state = 'published', published_at = CURRENT_TIMESTAMP WHERE id = $1`, supOffRev)
	mustExec(`UPDATE schedule_day_revisions SET state = 'superseded' WHERE id = $1`, supOffRev)
	mustExec(`INSERT INTO schedule_day_revisions (id, employment_id, employer_id, business_date, timezone, revision_number, state, publication_batch_id, source, source_snapshot, resolver_version, reason, published_by, published_actor_type, published_at, previous_revision_id) VALUES ($1, $2, $3, DATE '2026-01-10', 'UTC', 2, 'explicit_off', $4, 'roster', '{}'::jsonb, 'schedule-resolver-v1', 'replan off', $5, 'human', CURRENT_TIMESTAMP, $6)`, curOffRev, employmentID, employerID, batchID, actorID, supOffRev)

	// 2026-01-11: superseded completed segment with punch + replanned published scheduled segment -> denominator is 1 and historical completed shift is preserved.
	supWorkRev, curWorkRev, supWorkSeg, curWorkSeg := uuid.New(), uuid.New(), uuid.New(), uuid.New()
	mustExec(`INSERT INTO schedule_day_revisions (id, employment_id, employer_id, business_date, timezone, revision_number, state, publication_batch_id, source, source_snapshot, resolver_version, reason, published_by, published_actor_type, published_at) VALUES ($1, $2, $3, DATE '2026-01-11', 'UTC', 1, 'draft', $4, 'pattern', '{}'::jsonb, 'schedule-resolver-v1', 'rev1', $5, 'human', NULL)`, supWorkRev, employmentID, employerID, batchID, actorID)
	mustExec(`INSERT INTO work_segments (id, employment_id, date, sort_order, scheduled_start, scheduled_end, segment_type, segment_status, source, template_id, schedule_revision_id, selected_shift_version_id, scheduled_start_at, scheduled_end_at, breaks_snapshot) VALUES ($1, $2, DATE '2026-01-11', 0, '08:00', '17:00', 'work', 'completed', 'pattern', $3, $4, $5, TIMESTAMPTZ '2026-01-11 08:00:00+00', TIMESTAMPTZ '2026-01-11 17:00:00+00', '[]'::jsonb)`, supWorkSeg, employmentID, templateID, supWorkRev, versionID)
	mustExec(`UPDATE schedule_day_revisions SET state = 'published', published_at = CURRENT_TIMESTAMP WHERE id = $1`, supWorkRev)
	mustExec(`UPDATE schedule_day_revisions SET state = 'superseded' WHERE id = $1`, supWorkRev)
	mustExec(`INSERT INTO schedule_day_revisions (id, employment_id, employer_id, business_date, timezone, revision_number, state, publication_batch_id, source, source_snapshot, resolver_version, reason, published_by, published_actor_type, published_at, previous_revision_id) VALUES ($1, $2, $3, DATE '2026-01-11', 'UTC', 2, 'draft', $4, 'pattern', '{}'::jsonb, 'schedule-resolver-v1', 'rev2', $5, 'human', NULL, $6)`, curWorkRev, employmentID, employerID, batchID, actorID, supWorkRev)
	mustExec(`INSERT INTO work_segments (id, employment_id, date, sort_order, scheduled_start, scheduled_end, segment_type, segment_status, source, template_id, schedule_revision_id, selected_shift_version_id, scheduled_start_at, scheduled_end_at, breaks_snapshot) VALUES ($1, $2, DATE '2026-01-11', 0, '09:00', '18:00', 'work', 'scheduled', 'pattern', $3, $4, $5, TIMESTAMPTZ '2026-01-11 09:00:00+00', TIMESTAMPTZ '2026-01-11 18:00:00+00', '[]'::jsonb)`, curWorkSeg, employmentID, templateID, curWorkRev, versionID)
	mustExec(`UPDATE schedule_day_revisions SET state = 'published', published_at = CURRENT_TIMESTAMP WHERE id = $1`, curWorkRev)
	mustExec(`INSERT INTO time_entries (id, employment_id, segment_id, punch_in_time, punch_out_time, duration_minutes, entry_type) VALUES (gen_random_uuid(), $1, $2, TIMESTAMPTZ '2026-01-11 08:00:00+00', TIMESTAMPTZ '2026-01-11 17:00:00+00', 540, 'work')`, employmentID, supWorkSeg)

	// 2026-01-12: draft revision only -> must NOT count.
	draftRev, draftSeg := uuid.New(), uuid.New()
	mustExec(`INSERT INTO schedule_day_revisions (id, employment_id, employer_id, business_date, timezone, revision_number, state, publication_batch_id, source, source_snapshot, resolver_version, reason, published_by, published_actor_type, published_at) VALUES ($1, $2, $3, DATE '2026-01-12', 'UTC', 1, 'draft', $4, 'pattern', '{}'::jsonb, 'schedule-resolver-v1', 'draft', $5, 'human', NULL)`, draftRev, employmentID, employerID, batchID, actorID)
	mustExec(`INSERT INTO work_segments (id, employment_id, date, sort_order, scheduled_start, scheduled_end, segment_type, segment_status, source, template_id, schedule_revision_id, selected_shift_version_id, scheduled_start_at, scheduled_end_at, breaks_snapshot) VALUES ($1, $2, DATE '2026-01-12', 0, '08:00', '17:00', 'work', 'scheduled', 'pattern', $3, $4, $5, TIMESTAMPTZ '2026-01-12 08:00:00+00', TIMESTAMPTZ '2026-01-12 17:00:00+00', '[]'::jsonb)`, draftSeg, employmentID, templateID, draftRev, versionID)

	bundles, err := (PayrollBatchLoader{}).LoadForEmployments(ctx, tx, []uuid.UUID{employmentID}, 2026, 1)
	if err != nil {
		t.Fatalf("load batch period inputs: %v", err)
	}
	input := bundles[employmentID].Input
	if _, exists := input.ScheduledWorkdayCounts["2026-01-10"]; exists {
		t.Fatalf("superseded/legacy segment counted on explicit_off day: %#v", input.ScheduledWorkdayCounts)
	}
	if got := input.ScheduledWorkdayCounts["2026-01-11"]; got != 1 {
		t.Fatalf("2026-01-11 scheduled workday count = %d, want 1", got)
	}
	if got := input.CompletedShiftCounts["2026-01-11"]; got != 1 {
		t.Fatalf("2026-01-11 completed shift count = %d, want 1 (superseded segment must be excluded)", got)
	}
	if _, exists := input.ScheduledWorkdayCounts["2026-01-12"]; exists {
		t.Fatalf("draft revision segment must not count: %#v", input.ScheduledWorkdayCounts)
	}
}
