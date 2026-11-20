//go:build integration

package repository

import (
	"testing"
	"time"

	"github.com/google/uuid"
)

func TestIntegrationOfficialScheduleCoverageFiltersBeforePagination(t *testing.T) {
	pool, ctx := requireIntegrationDatabase(t)
	employerID, employeeMissing, employeeComplete, actorID := uuid.New(), uuid.New(), uuid.New(), uuid.New()
	employmentMissing, employmentComplete := uuid.New(), uuid.New()
	templateID, versionID, batchID, revisionID := uuid.New(), uuid.New(), uuid.New(), uuid.New()
	mustExec := func(query string, args ...any) {
		t.Helper()
		if _, err := pool.Exec(ctx, query, args...); err != nil {
			t.Fatalf("fixture query failed: %v", err)
		}
	}
	mustExec(`INSERT INTO employers (id, name) VALUES ($1, 'coverage-employer')`, employerID)
	mustExec(`INSERT INTO employees (id, employee_id_number, full_name, hire_date, original_hire_date) VALUES ($1, $2, 'coverage-missing', DATE '2026-01-01', DATE '2026-01-01'), ($3, $4, 'coverage-complete', DATE '2026-01-01', DATE '2026-01-01')`, employeeMissing, employeeMissing.String(), employeeComplete, employeeComplete.String())
	mustExec(`INSERT INTO employments (id, employee_id, employer_id, status, start_date, end_date) VALUES ($1, $2, $3, 'active', DATE '2026-11-16', DATE '2026-11-17'), ($4, $5, $3, 'active', DATE '2026-11-16', DATE '2026-11-16')`, employmentMissing, employeeMissing, employerID, employmentComplete, employeeComplete)
	mustExec(`INSERT INTO users (id, email, password_hash) VALUES ($1, $2, 'integration-test-hash')`, actorID, actorID.String()+"@invalid.test")
	mustExec(`INSERT INTO shift_templates (id, name) VALUES ($1, 'coverage-template')`, templateID)
	mustExec(`INSERT INTO shift_versions (id, template_id, version_number, name, effective_from, change_type, created_by) VALUES ($1, $2, 1, 'coverage-version', DATE '2026-01-01', 'create', $3)`, versionID, templateID, actorID)
	mustExec(`INSERT INTO shift_default_intervals (id, version_id, sort_order, start_time, end_time) VALUES (gen_random_uuid(), $1, 0, '08:00', '17:00')`, versionID)
	mustExec(`INSERT INTO schedule_publication_batches (id, idempotency_key, request_hash, mode, status, requested_by, operation, response_snapshot, reason, timezone, range_start, range_end) VALUES ($1, $2, repeat('e', 64), 'rotating', 'published', $3, 'publish', '{}'::jsonb, 'coverage fixture', 'UTC', DATE '2026-11-16', DATE '2026-11-17')`, batchID, batchID.String(), actorID)
	mustExec(`INSERT INTO schedule_day_revisions (id, employment_id, employer_id, business_date, timezone, revision_number, state, publication_batch_id, source, source_snapshot, resolver_version, reason, published_by, published_actor_type, published_at) VALUES ($1, $2, $3, DATE '2026-11-16', 'UTC', 1, 'draft', $4, 'manual', '{}'::jsonb, 'integration-test', 'coverage fixture', $5, 'human', NULL)`, revisionID, employmentComplete, employerID, batchID, actorID)
	mustExec(`SELECT set_config('schedule.lifecycle_publication', 'on', true)`)
	mustExec(`INSERT INTO work_segments (employment_id, date, sort_order, scheduled_start, scheduled_end, segment_type, segment_status, compensation_rate, gap_compensation_before, gap_compensation_rate_before, source, source_detail, generation_metadata, template_id, schedule_revision_id, selected_shift_version_id, scheduled_start_at, scheduled_end_at, breaks_snapshot) VALUES ($1, DATE '2026-11-16', 0, '08:00', '17:00', 'work', 'scheduled', 1, 'unpaid', 0, 'manual', '{}'::jsonb, '{}'::jsonb, $2, $3, $4, TIMESTAMPTZ '2026-11-16 08:00:00+00', TIMESTAMPTZ '2026-11-16 17:00:00+00', '[]'::jsonb)`, employmentComplete, templateID, revisionID, versionID)
	mustExec(`UPDATE schedule_day_revisions SET state = 'published', published_at = CURRENT_TIMESTAMP WHERE id = $1`, revisionID)

	result, err := (ScheduleRepository{DB: pool}).GetOfficialSchedulesWithCoverage(ctx, time.Date(2026, 11, 16, 0, 0, 0, 0, time.UTC), time.Date(2026, 11, 17, 0, 0, 0, 0, time.UTC), []uuid.UUID{employmentMissing, employmentComplete}, 1, 1, "missing")
	if err != nil {
		t.Fatalf("coverage query: %v", err)
	}
	if result.Total != 1 || len(result.Data) != 1 || result.Data[0].EmploymentID != employmentMissing {
		t.Fatalf("missing coverage filter result=%+v, want only missing employment", result)
	}
	coverage := result.Data[0].Coverage
	if coverage.ExpectedDays != 2 || coverage.PublishedWorkDays != 0 || coverage.PublishedOffDays != 0 || coverage.MissingOfficialDays != 2 || coverage.Complete {
		t.Fatalf("missing coverage counts=%+v", coverage)
	}
}
