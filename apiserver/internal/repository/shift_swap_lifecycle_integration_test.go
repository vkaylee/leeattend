//go:build integration

package repository

import (
	"context"
	"errors"
	"fmt"
	"strings"
	"sync"
	"testing"
	"time"

	"github.com/google/uuid"
	"github.com/jackc/pgx/v5/pgxpool"
	"github.com/vkaylee/leeattend/apiserver/internal/api"
)

func TestIntegrationApprovedShiftSwapRollsBackOnOfficialSchedule(t *testing.T) {
	pool, ctx := requireIntegrationDatabase(t)
	employerID, employeeA, employeeB := uuid.New(), uuid.New(), uuid.New()
	employmentA, employmentB, actorID, swapID := uuid.New(), uuid.New(), uuid.New(), uuid.New()
	templateA, templateB, versionA, versionB := uuid.New(), uuid.New(), uuid.New(), uuid.New()
	batchID, revisionA, revisionB := uuid.New(), uuid.New(), uuid.New()
	date := "2026-11-20"
	mustExec := func(query string, args ...any) {
		t.Helper()
		if _, err := pool.Exec(ctx, query, args...); err != nil {
			t.Fatalf("fixture query failed: %v", err)
		}
	}
	mustExec(`INSERT INTO employers (id, name) VALUES ($1, 'swap-lifecycle-employer')`, employerID)
	mustExec(`INSERT INTO employees (id, full_name) VALUES ($1, 'swap-employee-a'), ($2, 'swap-employee-b')`, employeeA, employeeB)
	relA, relB, tenureA, tenureB := uuid.New(), uuid.New(), uuid.New(), uuid.New()
	mustExec(`INSERT INTO employee_tenures (id, employee_id, employee_number, start_date) VALUES ($1, $2, $3, DATE '2026-01-01'), ($4, $5, $6, DATE '2026-01-01')`, tenureA, employeeA, employeeA.String(), tenureB, employeeB, employeeB.String())
	mustExec(`INSERT INTO employment_relationships (id, employee_id, employer_id, tenure_id, status, start_date) VALUES ($1, $2, $3, $4, 'active', DATE '2026-01-01'), ($5, $6, $3, $7, 'active', DATE '2026-01-01')`, relA, employeeA, employerID, tenureA, relB, employeeB, tenureB)
	mustExec(`INSERT INTO employment_segments (id, employee_id, employer_id, relationship_id, status, start_date) VALUES ($1, $2, $5, $6, 'active', DATE '2026-01-01'), ($3, $4, $5, $7, 'active', DATE '2026-01-01')`, employmentA, employeeA, employmentB, employeeB, employerID, relA, relB)
	mustExec(`INSERT INTO users (id, email, password_hash) VALUES ($1, $2, 'integration-test-hash')`, actorID, actorID.String()+"@invalid.test")
	mustExec(`INSERT INTO shift_templates (id, name) VALUES ($1, 'swap-template-a'), ($2, 'swap-template-b')`, templateA, templateB)
	mustExec(`INSERT INTO shift_versions (id, template_id, version_number, name, effective_from, change_type, created_by) VALUES ($1, $2, 1, 'swap-version-a', DATE '2026-01-01', 'create', $3), ($4, $5, 1, 'swap-version-b', DATE '2026-01-01', 'create', $3)`, versionA, templateA, actorID, versionB, templateB)
	mustExec(`INSERT INTO schedule_publication_batches (id, idempotency_key, request_hash, mode, status, requested_by, operation, response_snapshot, reason, timezone, range_start, range_end) VALUES ($1, $2, repeat('d', 64), 'rotating', 'published', $3, 'publish', '{}'::jsonb, 'swap fixture', 'UTC', $4::date, $4::date)`, batchID, batchID.String(), actorID, date)
	mustExec(`INSERT INTO schedule_day_revisions (id, employment_id, employer_id, business_date, timezone, revision_number, state, publication_batch_id, source, source_snapshot, resolver_version, reason, published_by, published_actor_type, published_at) VALUES ($1, $2, $3, $4::date, 'UTC', 1, 'draft', $5, 'manual', '{}'::jsonb, 'integration-test', 'swap fixture', $6, 'human', NULL), ($7, $8, $3, $4::date, 'UTC', 1, 'draft', $5, 'manual', '{}'::jsonb, 'integration-test', 'swap fixture', $6, 'human', NULL)`, revisionA, employmentA, employerID, date, batchID, actorID, revisionB, employmentB)
	mustExec(`SELECT set_config('schedule.lifecycle_publication', 'on', true)`)
	mustExec(`INSERT INTO work_segments (employment_id, date, sort_order, scheduled_start, scheduled_end, segment_type, segment_status, compensation_rate, gap_compensation_before, gap_compensation_rate_before, source, source_detail, generation_metadata, template_id, schedule_revision_id, selected_shift_version_id, scheduled_start_at, scheduled_end_at, breaks_snapshot) VALUES ($1, $2::date, 0, '08:00', '17:00', 'work', 'scheduled', 1, 'unpaid', 0, 'manual', '{}'::jsonb, '{}'::jsonb, $3, $4, $5, TIMESTAMPTZ '2026-11-20 08:00:00+00', TIMESTAMPTZ '2026-11-20 17:00:00+00', '[]'::jsonb), ($6, $2::date, 0, '09:00', '18:00', 'work', 'scheduled', 1, 'unpaid', 0, 'manual', '{}'::jsonb, '{}'::jsonb, $7, $8, $9, TIMESTAMPTZ '2026-11-20 09:00:00+00', TIMESTAMPTZ '2026-11-20 18:00:00+00', '[]'::jsonb)`, employmentA, date, templateA, revisionA, versionA, employmentB, templateB, revisionB, versionB)
	mustExec(`UPDATE schedule_day_revisions SET state = 'published', published_at = CURRENT_TIMESTAMP WHERE id IN ($1, $2)`, revisionA, revisionB)
	mustExec(`INSERT INTO shift_swap_requests (id, requester_id, target_id, swap_date, requester_template_id, target_template_id, requester_revision_id, requester_revision_number, target_revision_id, target_revision_number, status, reason, requested_by_user_id, created_at, updated_at) VALUES ($1, $2, $3, $4::date, $5, $6, $7, 1, $8, 1, 'pending_admin', 'swap official day', $9, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)`, swapID, employeeA, employeeB, date, templateA, templateB, revisionA, revisionB, actorID)

	if err := (ShiftSwapRepository{DB: pool}).ApproveSwap(ctx, swapID, actorID, api.ShiftSwapApproved); err != nil {
		t.Fatalf("approve official swap: %v", err)
	}
	var status string
	if err := pool.QueryRow(ctx, `SELECT status FROM shift_swap_requests WHERE id = $1`, swapID).Scan(&status); err != nil {
		t.Fatalf("read swap status: %v", err)
	}
	if status != string(api.ShiftSwapApproved) {
		t.Fatalf("swap status=%s, want approved", status)
	}
	var superseded, current int
	if err := pool.QueryRow(ctx, `SELECT count(*) FILTER (WHERE state = 'superseded'), count(*) FILTER (WHERE state IN ('published', 'explicit_off')) FROM schedule_day_revisions WHERE employment_id IN ($1, $2) AND business_date = $3::date`, employmentA, employmentB, date).Scan(&superseded, &current); err != nil {
		t.Fatalf("read revision states: %v", err)
	}
	if superseded != 2 || current != 2 {
		t.Fatalf("revision states superseded=%d current=%d, want 2/2", superseded, current)
	}
	var operation string
	if err := pool.QueryRow(ctx, `SELECT operation FROM schedule_publication_batches WHERE idempotency_key = $1`, "shift-swap:"+swapID.String()).Scan(&operation); err != nil {
		t.Fatalf("read swap publication: %v", err)
	}
	if operation != "replan" {
		t.Fatalf("swap publication operation=%s, want replan", operation)
	}
	var publicationCount, auditCount int
	if err := pool.QueryRow(ctx, `SELECT count(*) FROM schedule_publication_batches WHERE idempotency_key = $1`, "shift-swap:"+swapID.String()).Scan(&publicationCount); err != nil {
		t.Fatalf("count swap publications: %v", err)
	}
	if err := pool.QueryRow(ctx, `SELECT count(*) FROM audit_logs WHERE target_resource_id = (SELECT id FROM schedule_publication_batches WHERE idempotency_key = $1) AND action = 'schedule.swap_replanned'`, "shift-swap:"+swapID.String()).Scan(&auditCount); err != nil {
		t.Fatalf("count swap audits: %v", err)
	}
	if publicationCount != 1 || auditCount != 1 {
		t.Fatalf("swap evidence publications=%d audits=%d, want 1/1", publicationCount, auditCount)
	}
	for employmentID, expectedVersion := range map[uuid.UUID]uuid.UUID{employmentA: versionB, employmentB: versionA} {
		var selectedVersion uuid.UUID
		if err := pool.QueryRow(ctx, `SELECT selected_shift_version_id FROM work_segments ws JOIN schedule_day_revisions r ON r.id = ws.schedule_revision_id WHERE ws.employment_id = $1 AND r.business_date = $2::date AND r.state IN ('published', 'explicit_off')`, employmentID, date).Scan(&selectedVersion); err != nil {
			t.Fatalf("read swapped segment for %s: %v", employmentID, err)
		}
		if selectedVersion != expectedVersion {
			t.Fatalf("employment %s selected version=%s, want %s", employmentID, selectedVersion, expectedVersion)
		}
	}
}

type officialSwapFixture struct {
	employerID, employeeA, employeeB          uuid.UUID
	employmentA, employmentB, actorID, swapID uuid.UUID
	templateA, templateB, versionA, versionB  uuid.UUID
	batchID, revisionA, revisionB             uuid.UUID
	date                                      string
}

func seedOfficialSwapFixture(t *testing.T) (*pgxpool.Pool, context.Context, officialSwapFixture) {
	t.Helper()
	pool, ctx := requireIntegrationDatabase(t)
	fixture := officialSwapFixture{
		employerID: uuid.New(), employeeA: uuid.New(), employeeB: uuid.New(),
		employmentA: uuid.New(), employmentB: uuid.New(), actorID: uuid.New(), swapID: uuid.New(),
		templateA: uuid.New(), templateB: uuid.New(), versionA: uuid.New(), versionB: uuid.New(),
		batchID: uuid.New(), revisionA: uuid.New(), revisionB: uuid.New(), date: "2026-11-21",
	}
	exec := func(query string, args ...any) {
		t.Helper()
		if _, err := pool.Exec(ctx, query, args...); err != nil {
			t.Fatalf("fixture query failed: %v", err)
		}
	}
	exec(`INSERT INTO employers (id, name) VALUES ($1, 'atomic-swap-employer')`, fixture.employerID)
	exec(`INSERT INTO employees (id, full_name) VALUES ($1, 'atomic-swap-a'), ($2, 'atomic-swap-b')`, fixture.employeeA, fixture.employeeB)
	relA, relB, tenureA, tenureB := uuid.New(), uuid.New(), uuid.New(), uuid.New()
	exec(`INSERT INTO employee_tenures (id, employee_id, employee_number, start_date) VALUES ($1, $2, $3, DATE '2026-01-01'), ($4, $5, $6, DATE '2026-01-01')`, tenureA, fixture.employeeA, fixture.employeeA.String(), tenureB, fixture.employeeB, fixture.employeeB.String())
	exec(`INSERT INTO employment_relationships (id, employee_id, employer_id, tenure_id, status, start_date) VALUES ($1, $2, $3, $4, 'active', DATE '2026-01-01'), ($5, $6, $3, $7, 'active', DATE '2026-01-01')`, relA, fixture.employeeA, fixture.employerID, tenureA, relB, fixture.employeeB, tenureB)
	exec(`INSERT INTO employment_segments (id, employee_id, employer_id, relationship_id, status, start_date) VALUES ($1, $2, $5, $6, 'active', DATE '2026-01-01'), ($3, $4, $5, $7, 'active', DATE '2026-01-01')`, fixture.employmentA, fixture.employeeA, fixture.employmentB, fixture.employeeB, fixture.employerID, relA, relB)
	exec(`INSERT INTO users (id, email, password_hash) VALUES ($1, $2, 'integration-test-hash')`, fixture.actorID, fixture.actorID.String()+"@invalid.test")
	exec(`INSERT INTO shift_templates (id, name) VALUES ($1, $3), ($2, $4)`, fixture.templateA, fixture.templateB, "atomic-swap-template-a-"+fixture.templateA.String(), "atomic-swap-template-b-"+fixture.templateB.String())
	exec(`INSERT INTO shift_versions (id, template_id, version_number, name, effective_from, change_type, created_by) VALUES ($1, $2, 1, 'atomic-swap-version-a', DATE '2026-01-01', 'create', $3), ($4, $5, 1, 'atomic-swap-version-b', DATE '2026-01-01', 'create', $3)`, fixture.versionA, fixture.templateA, fixture.actorID, fixture.versionB, fixture.templateB)
	exec(`INSERT INTO schedule_publication_batches (id, idempotency_key, request_hash, mode, status, requested_by, operation, response_snapshot, reason, timezone, range_start, range_end) VALUES ($1, $2, repeat('e', 64), 'rotating', 'published', $3, 'publish', '{}'::jsonb, 'atomic swap fixture', 'UTC', $4::date, $4::date)`, fixture.batchID, fixture.batchID.String(), fixture.actorID, fixture.date)
	exec(`INSERT INTO schedule_day_revisions (id, employment_id, employer_id, business_date, timezone, revision_number, state, publication_batch_id, source, source_snapshot, resolver_version, reason, published_by, published_actor_type, published_at) VALUES ($1, $2, $3, $4::date, 'UTC', 1, 'draft', $5, 'manual', '{}'::jsonb, 'integration-test', 'atomic swap fixture', $6, 'human', NULL), ($7, $8, $3, $4::date, 'UTC', 1, 'draft', $5, 'manual', '{}'::jsonb, 'integration-test', 'atomic swap fixture', $6, 'human', NULL)`, fixture.revisionA, fixture.employmentA, fixture.employerID, fixture.date, fixture.batchID, fixture.actorID, fixture.revisionB, fixture.employmentB)
	exec(`INSERT INTO work_segments (employment_id, date, sort_order, scheduled_start, scheduled_end, segment_type, segment_status, compensation_rate, gap_compensation_before, gap_compensation_rate_before, source, source_detail, generation_metadata, template_id, schedule_revision_id, selected_shift_version_id, scheduled_start_at, scheduled_end_at, breaks_snapshot) VALUES ($1, $2::date, 0, '08:00', '17:00', 'work', 'scheduled', 1, 'unpaid', 0, 'manual', '{}'::jsonb, '{}'::jsonb, $3, $4, $5, TIMESTAMPTZ '2026-11-21 08:00:00+00', TIMESTAMPTZ '2026-11-21 17:00:00+00', '[]'::jsonb), ($6, $2::date, 0, '09:00', '18:00', 'work', 'scheduled', 1, 'unpaid', 0, 'manual', '{}'::jsonb, '{}'::jsonb, $7, $8, $9, TIMESTAMPTZ '2026-11-21 09:00:00+00', TIMESTAMPTZ '2026-11-21 18:00:00+00', '[]'::jsonb)`, fixture.employmentA, fixture.date, fixture.templateA, fixture.revisionA, fixture.versionA, fixture.employmentB, fixture.templateB, fixture.revisionB, fixture.versionB)
	exec(`UPDATE schedule_day_revisions SET state = 'published', published_at = CURRENT_TIMESTAMP WHERE id IN ($1, $2)`, fixture.revisionA, fixture.revisionB)
	exec(`INSERT INTO shift_swap_requests (id, requester_id, target_id, swap_date, requester_template_id, target_template_id, requester_revision_id, requester_revision_number, target_revision_id, target_revision_number, status, reason, requested_by_user_id, created_at, updated_at) VALUES ($1, $2, $3, $4::date, $5, $6, $7, 1, $8, 1, 'pending_admin', 'atomic swap fixture', $9, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)`, fixture.swapID, fixture.employeeA, fixture.employeeB, fixture.date, fixture.templateA, fixture.templateB, fixture.revisionA, fixture.revisionB, fixture.actorID)
	return pool, ctx, fixture
}

func assertSwapPendingWithoutEvidence(t *testing.T, pool *pgxpool.Pool, ctx context.Context, fixture officialSwapFixture) {
	t.Helper()
	var status string
	if err := pool.QueryRow(ctx, `SELECT status FROM shift_swap_requests WHERE id = $1`, fixture.swapID).Scan(&status); err != nil {
		t.Fatalf("read failed swap status: %v", err)
	}
	if status != string(api.ShiftSwapPendingAdmin) {
		t.Fatalf("failed swap status=%s, want pending_admin", status)
	}
	var count int
	if err := pool.QueryRow(ctx, `SELECT count(*) FROM schedule_publication_batches WHERE idempotency_key = $1`, "shift-swap:"+fixture.swapID.String()).Scan(&count); err != nil {
		t.Fatalf("count failed swap batch: %v", err)
	}
	if count != 0 {
		t.Fatalf("failed swap publication batches=%d, want 0", count)
	}
}

func TestIntegrationApprovedShiftSwapStaleRequesterRollsBack(t *testing.T) {
	pool, ctx, fixture := seedOfficialSwapFixture(t)
	newRevisionID := uuid.New()
	if _, err := pool.Exec(ctx, `INSERT INTO schedule_day_revisions (id, employment_id, employer_id, business_date, timezone, revision_number, state, publication_batch_id, source, source_snapshot, resolver_version, reason, published_by, published_actor_type, published_at) VALUES ($1, $2, $3, $4::date, 'UTC', 2, 'draft', $5, 'manual', '{}'::jsonb, 'integration-test', 'stale requester', $6, 'human', NULL)`, newRevisionID, fixture.employmentA, fixture.employerID, fixture.date, fixture.batchID, fixture.actorID); err != nil {
		t.Fatalf("insert stale requester revision: %v", err)
	}
	if _, err := pool.Exec(ctx, `SELECT set_config('schedule.lifecycle_publication', 'on', true)`); err != nil {
		t.Fatalf("enable lifecycle transition: %v", err)
	}
	if _, err := pool.Exec(ctx, `UPDATE schedule_day_revisions SET state = 'superseded' WHERE id = $1`, fixture.revisionA); err != nil {
		t.Fatalf("supersede requester revision: %v", err)
	}
	if _, err := pool.Exec(ctx, `UPDATE schedule_day_revisions SET state = 'published', published_at = CURRENT_TIMESTAMP WHERE id = $1`, newRevisionID); err != nil {
		t.Fatalf("publish requester replacement: %v", err)
	}
	err := (ShiftSwapRepository{DB: pool}).ApproveSwap(ctx, fixture.swapID, fixture.actorID, api.ShiftSwapApproved)
	if !errors.Is(err, api.ErrShiftSwapConflict) {
		t.Fatalf("stale requester error=%v, want conflict", err)
	}
	assertSwapPendingWithoutEvidence(t, pool, ctx, fixture)
}

func TestIntegrationApprovedShiftSwapStaleTargetRollsBack(t *testing.T) {
	pool, ctx, fixture := seedOfficialSwapFixture(t)
	newRevisionID := uuid.New()
	if _, err := pool.Exec(ctx, `INSERT INTO schedule_day_revisions (id, employment_id, employer_id, business_date, timezone, revision_number, state, publication_batch_id, source, source_snapshot, resolver_version, reason, published_by, published_actor_type, published_at) VALUES ($1, $2, $3, $4::date, 'UTC', 2, 'draft', $5, 'manual', '{}'::jsonb, 'integration-test', 'stale target', $6, 'human', NULL)`, newRevisionID, fixture.employmentB, fixture.employerID, fixture.date, fixture.batchID, fixture.actorID); err != nil {
		t.Fatalf("insert stale target revision: %v", err)
	}
	if _, err := pool.Exec(ctx, `SELECT set_config('schedule.lifecycle_publication', 'on', true)`); err != nil {
		t.Fatalf("enable lifecycle transition: %v", err)
	}
	if _, err := pool.Exec(ctx, `UPDATE schedule_day_revisions SET state = 'superseded' WHERE id = $1`, fixture.revisionB); err != nil {
		t.Fatalf("supersede target revision: %v", err)
	}
	if _, err := pool.Exec(ctx, `UPDATE schedule_day_revisions SET state = 'published', published_at = CURRENT_TIMESTAMP WHERE id = $1`, newRevisionID); err != nil {
		t.Fatalf("publish target replacement: %v", err)
	}
	err := (ShiftSwapRepository{DB: pool}).ApproveSwap(ctx, fixture.swapID, fixture.actorID, api.ShiftSwapApproved)
	if !errors.Is(err, api.ErrShiftSwapConflict) {
		t.Fatalf("stale target error=%v, want conflict", err)
	}
	assertSwapPendingWithoutEvidence(t, pool, ctx, fixture)
}

func TestIntegrationApprovedShiftSwapRejectsPayrollAndAttendanceLocks(t *testing.T) {
	for _, testCase := range []struct {
		name  string
		setup string
	}{
		{name: "payroll", setup: `INSERT INTO payroll_periods (id, employer_id, period_year, period_month, status) VALUES ($1, $2, 2026, 11, 'locked')`},
		{name: "attendance", setup: `INSERT INTO attendance_periods (id, employer_id, period_year, period_month, status) VALUES ($1, $2, 2026, 11, 'closed')`},
	} {
		t.Run(testCase.name, func(t *testing.T) {
			pool, ctx, fixture := seedOfficialSwapFixture(t)
			if _, err := pool.Exec(ctx, testCase.setup, uuid.New(), fixture.employerID); err != nil {
				t.Fatalf("insert %s lock: %v", testCase.name, err)
			}
			err := (ShiftSwapRepository{DB: pool}).ApproveSwap(ctx, fixture.swapID, fixture.actorID, api.ShiftSwapApproved)
			if !errors.Is(err, api.ErrScheduleLockedPeriod) {
				t.Fatalf("%s lock error=%v, want locked period", testCase.name, err)
			}
			assertSwapPendingWithoutEvidence(t, pool, ctx, fixture)
		})
	}
}

func TestIntegrationApprovedShiftSwapFailureAfterFirstReplacementRollsBack(t *testing.T) {
	pool, ctx, fixture := seedOfficialSwapFixture(t)
	triggerName := "trg_test_swap_fail_" + strings.ReplaceAll(uuid.NewString(), "-", "_")
	functionName := "fn_test_swap_fail_" + strings.ReplaceAll(uuid.NewString(), "-", "_")
	statement := fmt.Sprintf(`CREATE FUNCTION %s() RETURNS trigger LANGUAGE plpgsql AS $fn$ BEGIN IF NEW.employment_id = '%s'::uuid THEN RAISE EXCEPTION 'test replacement insertion failure'; END IF; RETURN NEW; END; $fn$; CREATE TRIGGER %s BEFORE INSERT ON work_segments FOR EACH ROW EXECUTE FUNCTION %s()`, functionName, fixture.employmentB, triggerName, functionName)
	if _, err := pool.Exec(ctx, statement); err != nil {
		t.Fatalf("create failure trigger: %v", err)
	}
	defer func() {
		_, _ = pool.Exec(ctx, fmt.Sprintf(`DROP TRIGGER IF EXISTS %s ON work_segments; DROP FUNCTION IF EXISTS %s()`, triggerName, functionName))
	}()
	err := (ShiftSwapRepository{DB: pool}).ApproveSwap(ctx, fixture.swapID, fixture.actorID, api.ShiftSwapApproved)
	if err == nil {
		t.Fatal("replacement insertion failure returned nil")
	}
	assertSwapPendingWithoutEvidence(t, pool, ctx, fixture)
	var superseded int
	if err := pool.QueryRow(ctx, `SELECT count(*) FROM schedule_day_revisions WHERE id IN ($1, $2) AND state = 'superseded'`, fixture.revisionA, fixture.revisionB).Scan(&superseded); err != nil {
		t.Fatalf("read rollback states: %v", err)
	}
	if superseded != 0 {
		t.Fatalf("old revisions superseded=%d after rollback, want 0", superseded)
	}
}

func TestIntegrationOpposingOfficialSwapsSerializeWithoutLostUpdate(t *testing.T) {
	pool, ctx, fixture := seedOfficialSwapFixture(t)
	opposingID := uuid.New()
	if _, err := pool.Exec(ctx, `INSERT INTO shift_swap_requests (id, requester_id, target_id, swap_date, requester_template_id, target_template_id, requester_revision_id, requester_revision_number, target_revision_id, target_revision_number, status, reason, requested_by_user_id, created_at, updated_at) VALUES ($1, $2, $3, $4::date, $5, $6, $7, 1, $8, 1, 'pending_admin', 'opposing atomic swap', $9, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)`, opposingID, fixture.employeeB, fixture.employeeA, fixture.date, fixture.templateB, fixture.templateA, fixture.revisionB, fixture.revisionA, fixture.actorID); err != nil {
		t.Fatalf("insert opposing swap: %v", err)
	}
	ctx, cancel := context.WithTimeout(ctx, 15*time.Second)
	defer cancel()
	results := make(chan error, 2)
	var wait sync.WaitGroup
	for _, requestID := range []uuid.UUID{fixture.swapID, opposingID} {
		wait.Add(1)
		go func(id uuid.UUID) {
			defer wait.Done()
			results <- (ShiftSwapRepository{DB: pool}).ApproveSwap(ctx, id, fixture.actorID, api.ShiftSwapApproved)
		}(requestID)
	}
	wait.Wait()
	close(results)
	approved, conflicts := 0, 0
	for err := range results {
		if err == nil {
			approved++
		} else if errors.Is(err, api.ErrShiftSwapConflict) {
			conflicts++
		} else {
			t.Fatalf("opposing swap error=%v", err)
		}
	}
	if approved != 1 || conflicts != 1 {
		t.Fatalf("opposing swaps approved=%d conflicts=%d, want 1/1", approved, conflicts)
	}
	var batchCount, currentCount int
	if err := pool.QueryRow(ctx, `SELECT count(*) FROM schedule_publication_batches WHERE operation = 'replan' AND idempotency_key IN ($1, $2)`, "shift-swap:"+fixture.swapID.String(), "shift-swap:"+opposingID.String()).Scan(&batchCount); err != nil {
		t.Fatalf("count opposing batches: %v", err)
	}
	if err := pool.QueryRow(ctx, `SELECT count(*) FROM schedule_day_revisions WHERE employment_id IN ($1, $2) AND business_date = $3::date AND state IN ('published', 'explicit_off')`, fixture.employmentA, fixture.employmentB, fixture.date).Scan(&currentCount); err != nil {
		t.Fatalf("count opposing current revisions: %v", err)
	}
	if batchCount != 1 || currentCount != 2 {
		t.Fatalf("opposing evidence batches=%d current_revisions=%d, want 1/2", batchCount, currentCount)
	}
}
