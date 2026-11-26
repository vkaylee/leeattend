//go:build integration

package repository

import (
	"context"
	"testing"
	"time"

	"github.com/google/uuid"
	"github.com/jackc/pgx/v5/pgconn"
)

func TestIntegrationOfficialScheduleParityAcrossReaderNoShowPunchAndPayroll(t *testing.T) {
	pool, ctx := requireIntegrationDatabase(t)
	repo := ScheduleRepository{DB: pool}
	actorID := uuid.New()
	templateID, versionID := uuid.New(), uuid.New()
	if _, err := pool.Exec(ctx, `INSERT INTO users (id, email, password_hash) VALUES ($1, $2, 'integration-test-hash')`, actorID, actorID.String()+"@invalid.test"); err != nil {
		t.Fatalf("insert actor: %v", err)
	}
	if _, err := pool.Exec(ctx, `INSERT INTO shift_templates (id, name) VALUES ($1, 'official-parity-shift')`, templateID); err != nil {
		t.Fatalf("insert template: %v", err)
	}
	if _, err := pool.Exec(ctx, `INSERT INTO shift_versions (id, template_id, version_number, name, effective_from, change_type, created_by) VALUES ($1, $2, 1, 'parity-v1', DATE '2026-01-01', 'create', $3)`, versionID, templateID, actorID); err != nil {
		t.Fatalf("insert version: %v", err)
	}

	employerID, employmentID := insertActivationAwareEmployment(t, pool, ctx, "2026-01-01", "2026-12-31")
	var employeeID uuid.UUID
	if err := pool.QueryRow(ctx, `SELECT employee_id FROM employment_segments WHERE id = $1`, employmentID).Scan(&employeeID); err != nil {
		t.Fatalf("load employee: %v", err)
	}

	// 2026-12-01: published work day (08:00-17:00 UTC)
	batchID, workRevID, workSegID := uuid.New(), uuid.New(), uuid.New()
	if _, err := pool.Exec(ctx, `INSERT INTO schedule_publication_batches (id, idempotency_key, request_hash, mode, status, requested_by, operation, response_snapshot, reason, timezone, range_start, range_end) VALUES ($1, $2, repeat('a', 64), 'rotating', 'published', $3, 'publish', '{}'::jsonb, 'parity test', 'UTC', DATE '2026-12-01', DATE '2026-12-02')`, batchID, batchID.String(), actorID); err != nil {
		t.Fatalf("insert batch: %v", err)
	}
	tx, err := pool.Begin(ctx)
	if err != nil {
		t.Fatalf("begin tx: %v", err)
	}
	if _, err := tx.Exec(ctx, `SELECT set_config('schedule.lifecycle_publication', 'on', true)`); err != nil {
		_ = tx.Rollback(ctx)
		t.Fatalf("set config: %v", err)
	}
	if _, err := tx.Exec(ctx, `INSERT INTO schedule_day_revisions (id, employment_id, employer_id, business_date, timezone, revision_number, state, publication_batch_id, source, source_snapshot, resolver_version, reason, published_by, published_actor_type, published_at) VALUES ($1, $2, $3, DATE '2026-12-01', 'UTC', 1, 'draft', $4, 'manual', '{}'::jsonb, 'schedule-resolver-v1', 'parity work', $5, 'human', NULL)`, workRevID, employmentID, employerID, batchID, actorID); err != nil {
		_ = tx.Rollback(ctx)
		t.Fatalf("insert work rev: %v", err)
	}
	if _, err := tx.Exec(ctx, `INSERT INTO work_segments (id, employment_id, date, sort_order, scheduled_start, scheduled_end, segment_type, segment_status, source, template_id, schedule_revision_id, selected_shift_version_id, scheduled_start_at, scheduled_end_at, breaks_snapshot) VALUES ($1, $2, DATE '2026-12-01', 0, '08:00', '17:00', 'work', 'scheduled', 'manual', $3, $4, $5, TIMESTAMPTZ '2026-12-01 08:00:00+00', TIMESTAMPTZ '2026-12-01 17:00:00+00', '[]'::jsonb)`, workSegID, employmentID, templateID, workRevID, versionID); err != nil {
		_ = tx.Rollback(ctx)
		t.Fatalf("insert work seg: %v", err)
	}
	if _, err := tx.Exec(ctx, `UPDATE schedule_day_revisions SET state = 'published', published_at = CURRENT_TIMESTAMP WHERE id = $1`, workRevID); err != nil {
		_ = tx.Rollback(ctx)
		t.Fatalf("publish work rev: %v", err)
	}
	if err := tx.Commit(ctx); err != nil {
		t.Fatalf("commit work rev: %v", err)
	}
	// 2026-12-02: explicit_off; 2026-12-03: missing (no revision). Even with a legacy roster on 2026-12-03, official reader must report missing (not on-the-fly work).
	insertPublishedRevision(t, pool, ctx, actorID, employmentID, employerID, "2026-12-02", 1)
	if _, err := pool.Exec(ctx, `INSERT INTO shift_rosters (id, employment_id, date, template_id) VALUES (gen_random_uuid(), $1, DATE '2026-12-03', $2)`, employmentID, templateID); err != nil {
		t.Fatalf("insert un-published roster: %v", err)
	}

	// 1. Official schedule reader returns exact states & revision_number without falling back to roster on 2026-12-03.
	official, err := repo.GetOfficialSchedules(ctx, date("2026-12-01"), date("2026-12-03"), []uuid.UUID{employmentID}, 1, 50)
	if err != nil {
		t.Fatalf("official schedules: %v", err)
	}
	if len(official.Data) != 1 || len(official.Data[0].Schedules) != 3 {
		t.Fatalf("unexpected official schedules: %+v", official.Data)
	}
	days := official.Data[0].Schedules
	if days[0].ScheduleStatus != "work" || days[0].RevisionNumber == nil || *days[0].RevisionNumber != 1 || days[0].Provisional {
		t.Fatalf("day 1 want published work rev=1, got %+v", days[0])
	}
	if days[1].ScheduleStatus != "explicit_off" || days[1].RevisionNumber == nil || *days[1].RevisionNumber != 1 {
		t.Fatalf("day 2 want explicit_off rev=1, got %+v", days[1])
	}
	if days[2].ScheduleStatus != "missing" || days[2].RevisionID != nil || days[2].Provisional {
		t.Fatalf("day 3 must remain missing (never fallback to legacy roster): %+v", days[2])
	}

	// 2. Payroll denominator counts ONLY 2026-12-01 (published work), not explicit_off or un-published roster.
	bundles, err := (PayrollBatchLoader{}).LoadForEmployments(ctx, pool, []uuid.UUID{employmentID}, 2026, 12)
	if err != nil {
		t.Fatalf("payroll loader: %v", err)
	}
	if got := bundles[employmentID].Input.ScheduledWorkdayCounts; len(got) != 1 || got["2026-12-01"] != 1 {
		t.Fatalf("payroll scheduled workday counts = %#v, want only 2026-12-01", got)
	}

	// 3. PunchIn on 2026-12-01 pins exact official workSegID; PunchIn on 2026-12-03 stays unscheduled (segment_id=nil).
	entryWork, err := (AttendanceRepository{DB: pool}).PunchIn(ctx, employmentID, nil, "work", nil, false, time.Date(2026, 12, 1, 7, 45, 0, 0, time.UTC))
	if err != nil || entryWork.SegmentID == nil || *entryWork.SegmentID != workSegID {
		t.Fatalf("punch-in on published work day seg=%v err=%v, want %s", entryWork.SegmentID, err, workSegID)
	}
	if _, err := (AttendanceRepository{DB: pool}).PunchOut(ctx, employeeID, nil, time.Date(2026, 12, 1, 17, 0, 0, 0, time.UTC)); err != nil {
		t.Fatalf("punch-out day 1: %v", err)
	}
	entryMissing, err := (AttendanceRepository{DB: pool}).PunchIn(ctx, employmentID, nil, "work", nil, false, time.Date(2026, 12, 3, 8, 0, 0, 0, time.UTC))
	if err != nil || entryMissing.SegmentID != nil {
		t.Fatalf("punch-in on unassigned day must remain unscheduled: seg=%v err=%v", entryMissing.SegmentID, err)
	}
}

func TestIntegrationOfficialScheduleReaderClipsEmploymentAndExcludesSupersededLegacyRows(t *testing.T) {
	pool, ctx := requireIntegrationDatabase(t)
	actorID := uuid.New()
	if _, err := pool.Exec(ctx, `INSERT INTO users (id, email, password_hash) VALUES ($1, $2, 'integration-test-hash')`, actorID, actorID.String()+"@invalid.test"); err != nil {
		t.Fatalf("insert actor: %v", err)
	}
	employerID, employmentID := insertActivationAwareEmployment(t, pool, ctx, "2099-04-02", "2099-04-03")
	insertActivation(t, pool, ctx, employerID, "2099-04-01", "2099-04-04")
	insertPublishedRevision(t, pool, ctx, actorID, employmentID, employerID, "2099-04-02", 1)
	insertSupersededAndLegacyRows(t, pool, ctx, actorID, employmentID, employerID, "2099-04-02")
	insertPublishedRevision(t, pool, ctx, actorID, employmentID, employerID, "2099-04-03", 1)

	result, err := (ScheduleRepository{DB: pool}).GetOfficialSchedules(ctx, date("2099-04-01"), date("2099-04-04"), []uuid.UUID{employmentID}, 1, 50)
	if err != nil {
		t.Fatalf("official reader: %v", err)
	}
	if len(result.Data) != 1 || len(result.Data[0].Schedules) != 2 {
		t.Fatalf("reader must clip to employment dates: %+v", result.Data)
	}
	for _, day := range result.Data[0].Schedules {
		if day.RevisionID == nil || day.ScheduleStatus != "explicit_off" {
			t.Fatalf("superseded and legacy rows must not be current official rows: %+v", day)
		}
	}
}

func insertActivationAwareEmployment(t *testing.T, pool interface {
	Exec(context.Context, string, ...any) (pgconn.CommandTag, error)
}, ctx context.Context, start, end string) (uuid.UUID, uuid.UUID) {
	t.Helper()
	employerID, employeeID, employmentID := uuid.New(), uuid.New(), uuid.New()
	mustExec := func(query string, args ...any) {
		if _, err := pool.Exec(ctx, query, args...); err != nil {
			t.Fatalf("fixture query failed: %v", err)
		}
	}
	mustExec(`INSERT INTO employers (id, name) VALUES ($1, $2)`, employerID, "activation-aware-employer-"+employerID.String())
	seedIntegrationEmployeeGraphWithRange(t, ctx, pool, employeeID, employerID, employmentID, employeeID.String(), "activation-aware-employee-"+employeeID.String(), start, start, end)
	return employerID, employmentID
}

func insertActivation(t *testing.T, pool interface {
	Exec(context.Context, string, ...any) (pgconn.CommandTag, error)
}, ctx context.Context, employerID uuid.UUID, start, end string) {
	t.Helper()
	if _, err := pool.Exec(ctx, `INSERT INTO config_entries (scope_type, scope_id, config_key, value_bool, effective_from, effective_to) VALUES ('employer', $1, 'schedule.lifecycle.enabled', true, $2::date, $3::date)`, employerID, start, end); err != nil {
		t.Fatalf("insert activation: %v", err)
	}
}

func insertPublishedRevision(t *testing.T, pool interface {
	Exec(context.Context, string, ...any) (pgconn.CommandTag, error)
}, ctx context.Context, actorID, employmentID, employerID uuid.UUID, businessDate string, revision int) {
	t.Helper()
	batchID, revisionID := uuid.New(), uuid.New()
	if _, err := pool.Exec(ctx, `INSERT INTO schedule_publication_batches (id, idempotency_key, request_hash, mode, status, requested_by, operation, response_snapshot, reason, timezone, range_start, range_end) VALUES ($1, $2, repeat('a', 64), 'rotating', 'published', $3, 'publish', '{}'::jsonb, 'activation-aware reader fixture', 'UTC', $4::date, $4::date)`, batchID, batchID.String(), actorID, businessDate); err != nil {
		t.Fatalf("insert publication batch: %v", err)
	}
	if _, err := pool.Exec(ctx, `INSERT INTO schedule_day_revisions (id, employment_id, employer_id, business_date, timezone, revision_number, state, publication_batch_id, source, source_snapshot, resolver_version, reason, published_by, published_actor_type, published_at) VALUES ($1, $2, $3, $4::date, 'UTC', $5, 'explicit_off', $6, 'manual', '{}'::jsonb, 'test', 'activation-aware reader fixture', $7, 'human', CURRENT_TIMESTAMP)`, revisionID, employmentID, employerID, businessDate, revision, batchID, actorID); err != nil {
		t.Fatalf("insert official revision: %v", err)
	}
}

func insertSupersededAndLegacyRows(t *testing.T, pool interface {
	Exec(context.Context, string, ...any) (pgconn.CommandTag, error)
}, ctx context.Context, actorID, employmentID, employerID uuid.UUID, businessDate string) {
	t.Helper()
	batchID, supersededID, legacyID := uuid.New(), uuid.New(), uuid.New()
	if _, err := pool.Exec(ctx, `INSERT INTO schedule_publication_batches (id, idempotency_key, request_hash, mode, status, requested_by, operation, response_snapshot, reason, timezone, range_start, range_end) VALUES ($1, $2, repeat('b', 64), 'rotating', 'published', $3, 'publish', '{}'::jsonb, 'superseded fixture', 'UTC', $4::date, $4::date)`, batchID, batchID.String(), actorID, businessDate); err != nil {
		t.Fatalf("insert superseded batch: %v", err)
	}
	if _, err := pool.Exec(ctx, `INSERT INTO schedule_day_revisions (id, employment_id, employer_id, business_date, timezone, revision_number, state, publication_batch_id, source, source_snapshot, resolver_version, reason, published_by, published_actor_type, published_at) VALUES ($1, $2, $3, $4::date, 'UTC', 2, 'superseded', $5, 'manual', '{}'::jsonb, 'test', 'superseded fixture', $6, 'human', CURRENT_TIMESTAMP), ($7, $2, $3, $4::date, 'UTC', 3, 'legacy_unverified', NULL, 'legacy_unverified', '{"provenance":"legacy_unverified"}'::jsonb, 'test', 'legacy fixture', NULL, 'system', NULL)`, supersededID, employmentID, employerID, businessDate, batchID, actorID, legacyID); err != nil {
		t.Fatalf("insert superseded and legacy rows: %v", err)
	}
}

func date(value string) time.Time {
	parsed, _ := time.Parse("2006-01-02", value)
	return parsed
}
