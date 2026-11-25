//go:build integration

package repository

import (
	"context"
	"errors"
	"strings"
	"testing"

	"github.com/google/uuid"
	"github.com/jackc/pgx/v5"
	"github.com/vkaylee/leeattend/apiserver/internal/api"
	"github.com/vkaylee/leeattend/apiserver/internal/domain/ids"
)

func TestIntegrationAttendanceClosureCloseReopenReclosePreservesEvidence(t *testing.T) {
	pool, ctx := requireIntegrationDatabase(t)
	tx, err := pool.Begin(ctx)
	if err != nil {
		t.Fatalf("begin fixture transaction: %v", err)
	}
	defer func() { _ = tx.Rollback(ctx) }()

	fixture := createAttendanceClosureFixture(t, ctx, tx, false)
	repo := AttendanceRepository{DB: tx, BeginTx: func(context.Context) (TxBeginner, error) { return tx, nil }}

	first, err := repo.CloseAttendancePeriod(ctx, fixture.employerID, 2026, 11, 1, "first close", ids.UserID(fixture.actorID))
	if err != nil {
		t.Fatalf("close attendance period: %v", err)
	}
	if first.Status != "closed" || first.Version != 2 || first.ClosureNumber != 1 || first.DayCount != 1 || first.SnapshotHash == "" {
		t.Fatalf("unexpected first closure: %+v", first)
	}
	var firstSnapshotHash string
	if err := tx.QueryRow(ctx, `SELECT snapshot_hash FROM attendance_closures WHERE id = $1`, first.ClosureID).Scan(&firstSnapshotHash); err != nil {
		t.Fatalf("read first closure: %v", err)
	}

	reopenActorID := uuid.New()
	if _, err := tx.Exec(ctx, `INSERT INTO users (id, email, password_hash) VALUES ($1, $2, 'integration-test-hash')`, reopenActorID, reopenActorID.String()+"@invalid.test"); err != nil {
		t.Fatalf("insert reopen actor: %v", err)
	}
	reopened, err := repo.ReopenAttendancePeriod(ctx, fixture.employerID, 2026, 11, first.Version, "correct attendance", ids.UserID(reopenActorID))
	if err != nil {
		t.Fatalf("reopen attendance period: %v", err)
	}
	if reopened.Status != "reopened" || reopened.Version != 3 || reopened.PriorClosureID == nil || *reopened.PriorClosureID != *first.ClosureID {
		t.Fatalf("unexpected reopen response: %+v", reopened)
	}

	second, err := repo.CloseAttendancePeriod(ctx, fixture.employerID, 2026, 11, reopened.Version, "reclose after correction", ids.UserID(fixture.actorID))
	if err != nil {
		t.Fatalf("reclose attendance period: %v", err)
	}
	if second.Status != "closed" || second.Version != 4 || second.ClosureNumber != 2 || second.ClosureID == nil || *second.ClosureID == *first.ClosureID {
		t.Fatalf("unexpected second closure: %+v", second)
	}
	var closureCount, dayCount int
	if err := tx.QueryRow(ctx, `SELECT count(*) FROM attendance_closures WHERE period_id = $1`, first.PeriodID).Scan(&closureCount); err != nil {
		t.Fatalf("count closures: %v", err)
	}
	if err := tx.QueryRow(ctx, `SELECT count(*) FROM attendance_closure_days WHERE closure_id = $1`, first.ClosureID).Scan(&dayCount); err != nil {
		t.Fatalf("count first closure days: %v", err)
	}
	if closureCount != 2 || dayCount != 1 {
		t.Fatalf("closure evidence count = closures:%d first_days:%d", closureCount, dayCount)
	}
	var secondSnapshotHash string
	if err := tx.QueryRow(ctx, `SELECT snapshot_hash FROM attendance_closures WHERE id = $1`, second.ClosureID).Scan(&secondSnapshotHash); err != nil {
		t.Fatalf("read second closure: %v", err)
	}
	if firstSnapshotHash != secondSnapshotHash {
		t.Fatalf("same inputs should preserve snapshot hash: first=%s second=%s", firstSnapshotHash, secondSnapshotHash)
	}
}

func TestIntegrationAttendanceClosureRejectsStaleVersionAndLockedPayroll(t *testing.T) {
	pool, ctx := requireIntegrationDatabase(t)
	tx, err := pool.Begin(ctx)
	if err != nil {
		t.Fatalf("begin fixture transaction: %v", err)
	}
	defer func() { _ = tx.Rollback(ctx) }()

	fixture := createAttendanceClosureFixture(t, ctx, tx, true)
	repo := AttendanceRepository{DB: tx, BeginTx: func(context.Context) (TxBeginner, error) { return tx, nil }}
	if _, err := repo.CloseAttendancePeriod(ctx, fixture.employerID, 2026, 11, 1, "locked close", ids.UserID(fixture.actorID)); !errors.Is(err, api.ErrAttendanceLockedPeriod) {
		t.Fatalf("locked payroll close error = %v, want locked period", err)
	}
}

func TestIntegrationAttendanceClosureRejectsStaleVersion(t *testing.T) {
	pool, ctx := requireIntegrationDatabase(t)
	tx, err := pool.Begin(ctx)
	if err != nil {
		t.Fatalf("begin fixture transaction: %v", err)
	}
	defer func() { _ = tx.Rollback(ctx) }()

	fixture := createAttendanceClosureFixture(t, ctx, tx, false)
	repo := AttendanceRepository{DB: tx, BeginTx: func(context.Context) (TxBeginner, error) { return tx, nil }}
	if _, err := repo.CloseAttendancePeriod(ctx, fixture.employerID, 2026, 11, 1, "close once", ids.UserID(fixture.actorID)); err != nil {
		t.Fatalf("first close: %v", err)
	}
	if _, err := repo.CloseAttendancePeriod(ctx, fixture.employerID, 2026, 11, 1, "stale close", ids.UserID(fixture.actorID)); !errors.Is(err, api.ErrAttendancePeriodConflict) {
		t.Fatalf("stale close error = %v, want period conflict", err)
	}
}

func TestIntegrationAttendanceClosureEvidenceIsImmutable(t *testing.T) {
	pool, ctx := requireIntegrationDatabase(t)
	tx, err := pool.Begin(ctx)
	if err != nil {
		t.Fatalf("begin fixture transaction: %v", err)
	}
	defer func() { _ = tx.Rollback(ctx) }()

	fixture := createAttendanceClosureFixture(t, ctx, tx, false)
	repo := AttendanceRepository{DB: tx, BeginTx: func(context.Context) (TxBeginner, error) { return tx, nil }}
	closed, err := repo.CloseAttendancePeriod(ctx, fixture.employerID, 2026, 11, 1, "immutable evidence", ids.UserID(fixture.actorID))
	if err != nil {
		t.Fatalf("close attendance period: %v", err)
	}
	if _, err := tx.Exec(ctx, `UPDATE attendance_closures SET snapshot_hash = repeat('b', 64) WHERE id = $1`, closed.ClosureID); err == nil || !strings.Contains(err.Error(), "attendance closure evidence is immutable") {
		t.Fatalf("closure update error = %v, want immutable evidence rejection", err)
	}
}

func createAttendanceClosureFixture(t *testing.T, ctx context.Context, tx pgx.Tx, payrollLocked bool) struct {
	employerID   uuid.UUID
	employmentID uuid.UUID
	actorID      uuid.UUID
} {
	t.Helper()
	employerID, employeeID, employmentID, actorID := uuid.New(), uuid.New(), uuid.New(), uuid.New()
	mustExec := func(query string, args ...any) {
		t.Helper()
		if _, err := tx.Exec(ctx, query, args...); err != nil {
			t.Fatalf("fixture query failed: %v", err)
		}
	}
	mustExec(`INSERT INTO employers (id, name) VALUES ($1, 'closure-test-employer')`, employerID)
	mustExec(`INSERT INTO employees (id, employee_id_number, full_name, hire_date, original_hire_date) VALUES ($1, $2, 'closure-test-employee', DATE '2026-01-01', DATE '2026-01-01')`, employeeID, employeeID.String())
	mustExec(`INSERT INTO employments (id, employee_id, employer_id, status, start_date, end_date) VALUES ($1, $2, $3, 'active', DATE '2026-01-01', DATE '2026-11-01')`, employmentID, employeeID, employerID)
	mustExec(`INSERT INTO users (id, email, password_hash) VALUES ($1, $2, 'integration-test-hash')`, actorID, actorID.String()+"@invalid.test")
	payrollStatus := "open"
	if payrollLocked {
		payrollStatus = "locked"
	}
	mustExec(`INSERT INTO payroll_periods (id, employer_id, period_year, period_month, status) VALUES ($1, $2, 2026, 11, $3)`, uuid.New(), employerID, payrollStatus)
	batchID, revisionID, templateID, versionID := uuid.New(), uuid.New(), uuid.New(), uuid.New()
	mustExec(`INSERT INTO schedule_publication_batches (id, idempotency_key, request_hash, mode, status, requested_by, operation, response_snapshot, reason, timezone, range_start, range_end) VALUES ($1, $2, repeat('a', 64), 'rotating', 'published', $3, 'publish', '{}'::jsonb, 'closure fixture', 'UTC', DATE '2026-11-01', DATE '2026-11-01')`, batchID, batchID.String(), actorID)
	mustExec(`INSERT INTO shift_templates (id, name) VALUES ($1, 'closure-template')`, templateID)
	mustExec(`INSERT INTO shift_versions (id, template_id, version_number, name, effective_from, change_type, created_by) VALUES ($1, $2, 1, 'closure-version', DATE '2026-01-01', 'create', $3)`, versionID, templateID, actorID)
	mustExec(`INSERT INTO schedule_day_revisions (id, employment_id, employer_id, business_date, timezone, revision_number, state, publication_batch_id, source, source_snapshot, resolver_version, reason, published_by, published_actor_type, published_at) VALUES ($1, $2, $3, DATE '2026-11-01', 'UTC', 1, 'published', $4, 'manual', '{}'::jsonb, 'integration-test', 'closure fixture', $5, 'human', CURRENT_TIMESTAMP)`, revisionID, employmentID, employerID, batchID, actorID)
	mustExec(`SELECT set_config('schedule.lifecycle_publication', 'on', true)`)
	mustExec(`INSERT INTO work_segments (employment_id, date, sort_order, scheduled_start, scheduled_end, segment_type, segment_status, compensation_rate, gap_compensation_before, gap_compensation_rate_before, source, source_detail, generation_metadata, template_id, schedule_revision_id, selected_shift_version_id, scheduled_start_at, scheduled_end_at, breaks_snapshot) VALUES ($1, DATE '2026-11-01', 0, '08:00', '17:00', 'work', 'scheduled', 1, 'unpaid', 0, 'manual', '{}'::jsonb, '{}'::jsonb, $2, $3, $4, TIMESTAMPTZ '2026-11-01 08:00:00+00', TIMESTAMPTZ '2026-11-01 17:00:00+00', '[]'::jsonb)`, employmentID, templateID, revisionID, versionID)
	mustExec(`INSERT INTO attendance_daily_summaries (id, date, employment_id, actual_work_minutes, standard_minutes) VALUES ($1, DATE '2026-11-01', $2, 480, 480)`, uuid.New(), employmentID)
	return struct {
		employerID   uuid.UUID
		employmentID uuid.UUID
		actorID      uuid.UUID
	}{employerID: employerID, employmentID: employmentID, actorID: actorID}
}
