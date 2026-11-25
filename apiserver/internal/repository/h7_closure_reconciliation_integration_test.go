//go:build integration

package repository

import (
	"context"
	"encoding/json"
	"errors"
	"sync"
	"testing"
	"time"

	"github.com/google/uuid"
	"github.com/vkaylee/leeattend/apiserver/internal/api"
	"github.com/vkaylee/leeattend/apiserver/internal/domain/ids"
)

func TestIntegrationClosureBlocksSelfReopenAndRequiresIndependentApprover(t *testing.T) {
	pool, ctx := requireIntegrationDatabase(t)
	tx, err := pool.Begin(ctx)
	if err != nil {
		t.Fatalf("begin fixture transaction: %v", err)
	}
	defer func() { _ = tx.Rollback(ctx) }()

	fixture := createAttendanceClosureFixture(t, ctx, tx, false)
	independentApprover := uuid.New()
	if _, err := tx.Exec(ctx, `INSERT INTO users (id, email, password_hash) VALUES ($1, $2, 'integration-test-hash')`, independentApprover, independentApprover.String()+"@invalid.test"); err != nil {
		t.Fatalf("insert independent approver: %v", err)
	}

	repo := AttendanceRepository{DB: tx, BeginTx: func(context.Context) (TxBeginner, error) { return tx, nil }}
	closed, err := repo.CloseAttendancePeriod(ctx, fixture.employerID, 2026, 11, 1, "close by actor A", ids.UserID(fixture.actorID))
	if err != nil {
		t.Fatalf("close attendance period: %v", err)
	}

	// Self-reopen by the same actor who closed MUST be blocked.
	if _, err := repo.ReopenAttendancePeriod(ctx, fixture.employerID, 2026, 11, closed.Version, "self reopen attempt", ids.UserID(fixture.actorID)); !errors.Is(err, api.ErrAttendanceSelfReopenForbidden) {
		t.Fatalf("self-reopen error = %v, want ErrAttendanceSelfReopenForbidden", err)
	}

	// Period must remain closed at unchanged version.
	status, err := repo.GetAttendancePeriodStatus(ctx, fixture.employerID, 2026, 11)
	if err != nil {
		t.Fatalf("get period status: %v", err)
	}
	if status.Status != "closed" || status.Version != closed.Version {
		t.Fatalf("status after blocked self-reopen = %+v, want closed v%d", status, closed.Version)
	}

	// Independent approver succeeds.
	reopened, err := repo.ReopenAttendancePeriod(ctx, fixture.employerID, 2026, 11, closed.Version, "independent HR reopen", ids.UserID(independentApprover))
	if err != nil {
		t.Fatalf("independent reopen failed: %v", err)
	}
	if reopened.Status != "reopened" || reopened.Version != closed.Version+1 {
		t.Fatalf("unexpected reopen response: %+v", reopened)
	}
}

func TestIntegrationClosureC2MissingCoverageAndC3OpenPunch(t *testing.T) {
	pool, ctx := requireIntegrationDatabase(t)

	t.Run("C2 missing_coverage blocks closure without writing snapshot", func(t *testing.T) {
		tx, err := pool.Begin(ctx)
		if err != nil {
			t.Fatalf("begin tx: %v", err)
		}
		defer func() { _ = tx.Rollback(ctx) }()

		fixture := createAttendanceClosureFixture(t, ctx, tx, false)
		// Extend employment to 2026-11-02 while only 2026-11-01 has an official revision.
		if _, err := tx.Exec(ctx, `UPDATE employments SET end_date = DATE '2026-11-02' WHERE id = $1`, fixture.employmentID); err != nil {
			t.Fatalf("extend employment: %v", err)
		}

		repo := AttendanceRepository{DB: tx, BeginTx: func(context.Context) (TxBeginner, error) { return tx, nil }}
		if _, err := repo.CloseAttendancePeriod(ctx, fixture.employerID, 2026, 11, 1, "close missing day", ids.UserID(fixture.actorID)); !errors.Is(err, api.ErrAttendanceMissingCoverage) {
			t.Fatalf("C2 close error = %v, want ErrAttendanceMissingCoverage", err)
		}

		var closureCount int
		if err := tx.QueryRow(ctx, `SELECT count(*) FROM attendance_closures WHERE employer_id = $1`, fixture.employerID).Scan(&closureCount); err != nil {
			t.Fatalf("count closures: %v", err)
		}
		if closureCount != 0 {
			t.Fatalf("C2 expected 0 closures on rollback, got %d", closureCount)
		}
	})

	t.Run("C3 open_punch blocks closure without writing snapshot", func(t *testing.T) {
		tx, err := pool.Begin(ctx)
		if err != nil {
			t.Fatalf("begin tx: %v", err)
		}
		defer func() { _ = tx.Rollback(ctx) }()

		fixture := createAttendanceClosureFixture(t, ctx, tx, false)
		// Insert an unclosed time_entry (punch_out_time IS NULL) on 2026-11-01.
		if _, err := tx.Exec(ctx, `INSERT INTO time_entries (id, employment_id, punch_in_time, entry_type) VALUES (gen_random_uuid(), $1, TIMESTAMPTZ '2026-11-01 08:05:00+00', 'work')`, fixture.employmentID); err != nil {
			t.Fatalf("insert open punch: %v", err)
		}

		repo := AttendanceRepository{DB: tx, BeginTx: func(context.Context) (TxBeginner, error) { return tx, nil }}
		if _, err := repo.CloseAttendancePeriod(ctx, fixture.employerID, 2026, 11, 1, "close with open punch", ids.UserID(fixture.actorID)); !errors.Is(err, api.ErrAttendanceOpenPunch) {
			t.Fatalf("C3 close error = %v, want ErrAttendanceOpenPunch", err)
		}

		var closureCount int
		if err := tx.QueryRow(ctx, `SELECT count(*) FROM attendance_closures WHERE employer_id = $1`, fixture.employerID).Scan(&closureCount); err != nil {
			t.Fatalf("count closures: %v", err)
		}
		if closureCount != 0 {
			t.Fatalf("C3 expected 0 closures on rollback, got %d", closureCount)
		}
	})
}

func TestIntegrationClosureC4ConcurrentCloseAndReplanSerializesCleanly(t *testing.T) {
	pool, ctx := requireIntegrationDatabase(t)

	employerID, employeeID, employmentID, actorID := uuid.New(), uuid.New(), uuid.New(), uuid.New()
	templateID, versionID, batchID, rev1ID := uuid.New(), uuid.New(), uuid.New(), uuid.New()
	mustExec := func(query string, args ...any) {
		t.Helper()
		if _, err := pool.Exec(ctx, query, args...); err != nil {
			t.Fatalf("exec %q: %v", query, err)
		}
	}
	mustExec(`INSERT INTO employers (id, name) VALUES ($1, 'c4-employer')`, employerID)
	seedIntegrationEmployeeGraphWithRange(t, ctx, pool, employeeID, employerID, employmentID, "C4-"+employeeID.String()[:8], "C4 Worker", "2026-01-01", "2026-11-01", "2026-11-01")
	mustExec(`INSERT INTO users (id, email, password_hash) VALUES ($1, $2, 'integration-test-hash')`, actorID, actorID.String()+"@invalid.test")
	mustExec(`INSERT INTO payroll_periods (id, employer_id, period_year, period_month, status) VALUES ($1, $2, 2026, 11, 'open')`, uuid.New(), employerID)
	mustExec(`INSERT INTO shift_templates (id, name) VALUES ($1, 'c4-shift')`, templateID)
	mustExec(`INSERT INTO shift_versions (id, template_id, version_number, name, effective_from, change_type, created_by) VALUES ($1, $2, 1, 'c4-v1', DATE '2026-01-01', 'create', $3)`, versionID, templateID, actorID)
	mustExec(`INSERT INTO schedule_publication_batches (id, idempotency_key, request_hash, mode, status, requested_by, operation, response_snapshot, reason, timezone, range_start, range_end) VALUES ($1, $2, repeat('e', 64), 'rotating', 'published', $3, 'publish', '{}'::jsonb, 'c4 fixture', 'UTC', DATE '2026-11-01', DATE '2026-11-01')`, batchID, batchID.String(), actorID)

	tx, err := pool.Begin(ctx)
	if err != nil {
		t.Fatalf("begin seed tx: %v", err)
	}
	if _, err := tx.Exec(ctx, `SELECT set_config('schedule.lifecycle_publication', 'on', true)`); err != nil {
		_ = tx.Rollback(ctx)
		t.Fatalf("set config: %v", err)
	}
	if _, err := tx.Exec(ctx, `INSERT INTO schedule_day_revisions (id, employment_id, employer_id, business_date, timezone, revision_number, state, publication_batch_id, source, source_snapshot, resolver_version, reason, published_by, published_actor_type, published_at) VALUES ($1, $2, $3, DATE '2026-11-01', 'UTC', 1, 'draft', $4, 'manual', '{}'::jsonb, 'schedule-resolver-v1', 'c4', $5, 'human', NULL)`, rev1ID, employmentID, employerID, batchID, actorID); err != nil {
		_ = tx.Rollback(ctx)
		t.Fatalf("insert rev: %v", err)
	}
	if _, err := tx.Exec(ctx, `INSERT INTO work_segments (id, employment_id, date, sort_order, scheduled_start, scheduled_end, segment_type, segment_status, compensation_rate, gap_compensation_before, gap_compensation_rate_before, source, source_detail, generation_metadata, template_id, schedule_revision_id, selected_shift_version_id, scheduled_start_at, scheduled_end_at, breaks_snapshot) VALUES (gen_random_uuid(), $1, DATE '2026-11-01', 0, '08:00', '17:00', 'work', 'scheduled', 1, 'unpaid', 0, 'manual', '{}'::jsonb, '{}'::jsonb, $2, $3, $4, TIMESTAMPTZ '2026-11-01 08:00:00+00', TIMESTAMPTZ '2026-11-01 17:00:00+00', '[]'::jsonb)`, employmentID, templateID, rev1ID, versionID); err != nil {
		_ = tx.Rollback(ctx)
		t.Fatalf("insert seg: %v", err)
	}
	if _, err := tx.Exec(ctx, `UPDATE schedule_day_revisions SET state = 'published', published_at = CURRENT_TIMESTAMP WHERE id = $1`, rev1ID); err != nil {
		_ = tx.Rollback(ctx)
		t.Fatalf("publish rev: %v", err)
	}
	if err := tx.Commit(ctx); err != nil {
		t.Fatalf("commit seed: %v", err)
	}

	schedRepo := ScheduleRepository{DB: pool}
	attRepo := AttendanceRepository{DB: pool}
	replacement := api.ScheduleReplanReplacement{
		Status:         "work",
		Source:         "manual",
		ShiftVersionID: &versionID,
		Intervals:      []api.ScheduleReplanInterval{{SortOrder: 0, Start: "09:00", End: "18:00"}},
	}
	preview, err := schedRepo.PreviewReplan(ctx, actorID, employmentID, "2026-11-01", api.ScheduleReplanPreviewRequest{
		ExpectedRevision: 1,
		Reason:           "concurrent replan",
		Replacement:      replacement,
	})
	if err != nil {
		t.Fatalf("preview replan: %v", err)
	}

	startGate := make(chan struct{})
	var closeRes api.AttendanceClosureResponse
	var closeErr error
	var replanRes api.ScheduleReplanResponse
	var replanErr error
	var wg sync.WaitGroup
	wg.Add(2)

	go func() {
		defer wg.Done()
		<-startGate
		closeRes, closeErr = attRepo.CloseAttendancePeriod(ctx, employerID, 2026, 11, 1, "concurrent close", ids.UserID(actorID))
	}()
	go func() {
		defer wg.Done()
		<-startGate
		replanRes, replanErr = schedRepo.CommitReplan(ctx, actorID, employmentID, "2026-11-01", "c4-key-"+uuid.NewString(), api.ScheduleReplanRequest{
			ExpectedRevision: 1,
			Reason:           "concurrent replan",
			Replacement:      replacement,
			PreviewID:        preview.PreviewID,
			PreviewHash:      preview.PreviewHash,
			ConfirmDiffHash:  preview.DiffHash,
		})
	}()
	close(startGate)
	wg.Wait()

	if closeErr != nil {
		t.Fatalf("concurrent close failed unexpectedly (deadlock/error): %v", closeErr)
	}

	var snappedRevID uuid.UUID
	var snappedJSON []byte
	if err := pool.QueryRow(ctx, `SELECT schedule_revision_id, schedule_snapshot FROM attendance_closure_days WHERE closure_id = $1 AND employment_id = $2 AND business_date = DATE '2026-11-01'`, *closeRes.ClosureID, employmentID).Scan(&snappedRevID, &snappedJSON); err != nil {
		t.Fatalf("read snapped closure day: %v", err)
	}
	var snapDoc struct {
		RevisionID string `json:"revision_id"`
		Segments   []struct {
			ScheduledStart string `json:"scheduled_start"`
		} `json:"segments"`
	}
	if err := json.Unmarshal(snappedJSON, &snapDoc); err != nil {
		t.Fatalf("unmarshal closure snapshot: %v", err)
	}
	if snapDoc.RevisionID != snappedRevID.String() || len(snapDoc.Segments) != 1 {
		t.Fatalf("inconsistent closure snapshot: rev=%s doc=%+v", snappedRevID, snapDoc)
	}

	if replanErr != nil {
		// Close won first: replan must fail with locked period, and snapshot must match rev1 (08:00).
		if !errors.Is(replanErr, ErrScheduleLockedPeriod) {
			t.Fatalf("replan after close error = %v, want ErrScheduleLockedPeriod", replanErr)
		}
		if snappedRevID != rev1ID || snapDoc.Segments[0].ScheduledStart != "08:00:00" {
			t.Fatalf("close-first snapshot mismatch: rev=%s seg=%+v", snappedRevID, snapDoc.Segments)
		}
	} else {
		// Replan won first: close must snapshot rev2 (09:00), never mixed rev1/rev2.
		if snappedRevID != replanRes.ReplacementRevisionID || snapDoc.Segments[0].ScheduledStart != "09:00:00" {
			t.Fatalf("replan-first snapshot mismatch: snapped=%s replacement=%s seg=%+v", snappedRevID, replanRes.ReplacementRevisionID, snapDoc.Segments)
		}
	}
}

func TestIntegrationN1LatePunchPreservesClosureAndCreatesReconciliationSignal(t *testing.T) {
	pool, ctx := requireIntegrationDatabase(t)

	employerID, employeeID, employmentID, actorID := uuid.New(), uuid.New(), uuid.New(), uuid.New()
	templateID, versionID, batchID, revID := uuid.New(), uuid.New(), uuid.New(), uuid.New()
	mustExec := func(query string, args ...any) {
		t.Helper()
		if _, err := pool.Exec(ctx, query, args...); err != nil {
			t.Fatalf("exec %q: %v", query, err)
		}
	}
	mustExec(`INSERT INTO employers (id, name) VALUES ($1, 'n1-employer')`, employerID)
	seedIntegrationEmployeeGraphWithRange(t, ctx, pool, employeeID, employerID, employmentID, "N1-"+employeeID.String()[:8], "N1 Worker", "2026-01-01", "2026-11-01", "2026-11-01")
	mustExec(`INSERT INTO users (id, email, password_hash) VALUES ($1, $2, 'integration-test-hash')`, actorID, actorID.String()+"@invalid.test")
	mustExec(`INSERT INTO payroll_periods (id, employer_id, period_year, period_month, status) VALUES ($1, $2, 2026, 11, 'open')`, uuid.New(), employerID)
	mustExec(`INSERT INTO shift_templates (id, name) VALUES ($1, 'n1-shift')`, templateID)
	mustExec(`INSERT INTO shift_versions (id, template_id, version_number, name, effective_from, change_type, created_by) VALUES ($1, $2, 1, 'n1-v1', DATE '2026-01-01', 'create', $3)`, versionID, templateID, actorID)
	mustExec(`INSERT INTO schedule_publication_batches (id, idempotency_key, request_hash, mode, status, requested_by, operation, response_snapshot, reason, timezone, range_start, range_end) VALUES ($1, $2, repeat('f', 64), 'rotating', 'published', $3, 'publish', '{}'::jsonb, 'n1 fixture', 'UTC', DATE '2026-11-01', DATE '2026-11-01')`, batchID, batchID.String(), actorID)
	mustExec(`INSERT INTO schedule_day_revisions (id, employment_id, employer_id, business_date, timezone, revision_number, state, publication_batch_id, source, source_snapshot, resolver_version, reason, published_by, published_actor_type, published_at) VALUES ($1, $2, $3, DATE '2026-11-01', 'UTC', 1, 'published', $4, 'manual', '{}'::jsonb, 'schedule-resolver-v1', 'n1', $5, 'human', CURRENT_TIMESTAMP)`, revID, employmentID, employerID, batchID, actorID)

	tx, err := pool.Begin(ctx)
	if err != nil {
		t.Fatalf("begin seg tx: %v", err)
	}
	if _, err := tx.Exec(ctx, `SELECT set_config('schedule.lifecycle_publication', 'on', true)`); err != nil {
		_ = tx.Rollback(ctx)
		t.Fatalf("set config: %v", err)
	}
	if _, err := tx.Exec(ctx, `INSERT INTO work_segments (id, employment_id, date, sort_order, scheduled_start, scheduled_end, segment_type, segment_status, compensation_rate, gap_compensation_before, gap_compensation_rate_before, source, source_detail, generation_metadata, template_id, schedule_revision_id, selected_shift_version_id, scheduled_start_at, scheduled_end_at, breaks_snapshot) VALUES (gen_random_uuid(), $1, DATE '2026-11-01', 0, '08:00', '17:00', 'work', 'scheduled', 1, 'unpaid', 0, 'manual', '{}'::jsonb, '{}'::jsonb, $2, $3, $4, TIMESTAMPTZ '2026-11-01 08:00:00+00', TIMESTAMPTZ '2026-11-01 17:00:00+00', '[]'::jsonb)`, employmentID, templateID, revID, versionID); err != nil {
		_ = tx.Rollback(ctx)
		t.Fatalf("insert seg: %v", err)
	}
	if err := tx.Commit(ctx); err != nil {
		t.Fatalf("commit seg: %v", err)
	}

	attRepo := AttendanceRepository{DB: pool}
	closed, err := attRepo.CloseAttendancePeriod(ctx, employerID, 2026, 11, 1, "n1 close", ids.UserID(actorID))
	if err != nil {
		t.Fatalf("close period: %v", err)
	}

	// Late punch arrives AFTER period is closed.
	lateTime := time.Date(2026, 11, 1, 8, 10, 0, 0, time.UTC)
	entry, err := attRepo.PunchIn(ctx, employmentID, nil, "work", nil, false, lateTime)
	if err != nil {
		t.Fatalf("late punch-in rejected: %v", err)
	}
	if entry.ID == uuid.Nil {
		t.Fatal("expected late time entry ID")
	}

	// Closed snapshot MUST remain completely unchanged.
	var hashAfter string
	var snappedEntryCount int
	if err := pool.QueryRow(ctx, `SELECT c.snapshot_hash, COALESCE(array_length(cd.time_entry_ids, 1), 0) FROM attendance_closures c JOIN attendance_closure_days cd ON cd.closure_id = c.id WHERE c.id = $1`, *closed.ClosureID).Scan(&hashAfter, &snappedEntryCount); err != nil {
		t.Fatalf("read closure after late punch: %v", err)
	}
	if hashAfter != closed.SnapshotHash || snappedEntryCount != 0 {
		t.Fatalf("closure mutated by late punch: hash=%s want=%s snappedEntries=%d", hashAfter, closed.SnapshotHash, snappedEntryCount)
	}

	// Reconciliation signal MUST exist in outbox_jobs for this closed period + employment.
	var signalCount int
	var payloadRaw []byte
	if err := pool.QueryRow(ctx, `SELECT count(*), MAX(payload::text)::jsonb FROM outbox_jobs WHERE job_type = 'AttendanceLatePunchReconciliation' AND status = 'pending' AND payload->>'employment_id' = $1 AND payload->>'period_id' = $2`, employmentID.String(), closed.PeriodID.String()).Scan(&signalCount, &payloadRaw); err != nil {
		t.Fatalf("query reconciliation outbox signal: %v", err)
	}
	if signalCount != 1 {
		t.Fatalf("late punch reconciliation signals = %d, want 1", signalCount)
	}
	var payload map[string]any
	if err := json.Unmarshal(payloadRaw, &payload); err != nil {
		t.Fatalf("unmarshal reconciliation payload: %v", err)
	}
	if payload["closure_id"] != closed.ClosureID.String() {
		t.Fatalf("reconciliation closure_id = %v, want %s", payload["closure_id"], closed.ClosureID.String())
	}
}
