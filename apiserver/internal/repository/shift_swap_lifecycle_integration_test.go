//go:build integration

package repository

import (
	"errors"
	"testing"

	"github.com/google/uuid"
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
	mustExec(`INSERT INTO employees (id, employee_id_number, full_name, hire_date, original_hire_date) VALUES ($1, $2, 'swap-employee-a', DATE '2026-01-01', DATE '2026-01-01'), ($3, $4, 'swap-employee-b', DATE '2026-01-01', DATE '2026-01-01')`, employeeA, employeeA.String(), employeeB, employeeB.String())
	relA, relB, tenureA, tenureB := uuid.New(), uuid.New(), uuid.New(), uuid.New()
	mustExec(`INSERT INTO employee_tenures (id, employee_id, employee_number, start_date) VALUES ($1, $2, $3, DATE '2026-01-01'), ($4, $5, $6, DATE '2026-01-01')`, tenureA, employeeA, employeeA.String(), tenureB, employeeB, employeeB.String())
	mustExec(`INSERT INTO employment_relationships (id, employee_id, employer_id, tenure_id, status, start_date) VALUES ($1, $2, $3, $4, 'active', DATE '2026-01-01'), ($5, $6, $3, $7, 'active', DATE '2026-01-01')`, relA, employeeA, employerID, tenureA, relB, employeeB, tenureB)
	mustExec(`INSERT INTO employments (id, employee_id, employer_id, relationship_id, status, start_date) VALUES ($1, $2, $5, $6, 'active', DATE '2026-01-01'), ($3, $4, $5, $7, 'active', DATE '2026-01-01')`, employmentA, employeeA, employmentB, employeeB, employerID, relA, relB)
	mustExec(`INSERT INTO users (id, email, password_hash) VALUES ($1, $2, 'integration-test-hash')`, actorID, actorID.String()+"@invalid.test")
	mustExec(`INSERT INTO shift_templates (id, name) VALUES ($1, 'swap-template-a'), ($2, 'swap-template-b')`, templateA, templateB)
	mustExec(`INSERT INTO shift_versions (id, template_id, version_number, name, effective_from, change_type, created_by) VALUES ($1, $2, 1, 'swap-version-a', DATE '2026-01-01', 'create', $3), ($4, $5, 1, 'swap-version-b', DATE '2026-01-01', 'create', $3)`, versionA, templateA, actorID, versionB, templateB)
	mustExec(`INSERT INTO schedule_publication_batches (id, idempotency_key, request_hash, mode, status, requested_by, operation, response_snapshot, reason, timezone, range_start, range_end) VALUES ($1, $2, repeat('d', 64), 'rotating', 'published', $3, 'publish', '{}'::jsonb, 'swap fixture', 'UTC', $4::date, $4::date)`, batchID, batchID.String(), actorID, date)
	mustExec(`INSERT INTO schedule_day_revisions (id, employment_id, employer_id, business_date, timezone, revision_number, state, publication_batch_id, source, source_snapshot, resolver_version, reason, published_by, published_actor_type, published_at) VALUES ($1, $2, $3, $4::date, 'UTC', 1, 'draft', $5, 'manual', '{}'::jsonb, 'integration-test', 'swap fixture', $6, 'human', NULL), ($7, $8, $3, $4::date, 'UTC', 1, 'draft', $5, 'manual', '{}'::jsonb, 'integration-test', 'swap fixture', $6, 'human', NULL)`, revisionA, employmentA, employerID, date, batchID, actorID, revisionB, employmentB)
	mustExec(`SELECT set_config('schedule.lifecycle_publication', 'on', true)`)
	mustExec(`INSERT INTO work_segments (employment_id, date, sort_order, scheduled_start, scheduled_end, segment_type, segment_status, compensation_rate, gap_compensation_before, gap_compensation_rate_before, source, source_detail, generation_metadata, template_id, schedule_revision_id, selected_shift_version_id, scheduled_start_at, scheduled_end_at, breaks_snapshot) VALUES ($1, $2::date, 0, '08:00', '17:00', 'work', 'scheduled', 1, 'unpaid', 0, 'manual', '{}'::jsonb, '{}'::jsonb, $3, $4, $5, TIMESTAMPTZ '2026-11-20 08:00:00+00', TIMESTAMPTZ '2026-11-20 17:00:00+00', '[]'::jsonb), ($6, $2::date, 0, '09:00', '18:00', 'work', 'scheduled', 1, 'unpaid', 0, 'manual', '{}'::jsonb, '{}'::jsonb, $7, $8, $9, TIMESTAMPTZ '2026-11-20 09:00:00+00', TIMESTAMPTZ '2026-11-20 18:00:00+00', '[]'::jsonb)`, employmentA, date, templateA, revisionA, versionA, employmentB, templateB, revisionB, versionB)
	mustExec(`UPDATE schedule_day_revisions SET state = 'published', published_at = CURRENT_TIMESTAMP WHERE id IN ($1, $2)`, revisionA, revisionB)
	mustExec(`INSERT INTO shift_swap_requests (id, requester_id, target_id, swap_date, requester_template_id, target_template_id, status, reason, requested_by_user_id, created_at, updated_at) VALUES ($1, $2, $3, $4::date, $5, $6, 'pending_admin', 'swap official day', $7, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)`, swapID, employeeA, employeeB, date, templateA, templateB, actorID)

	err := (ShiftSwapRepository{DB: pool}).ApproveSwap(ctx, swapID, actorID, api.ShiftSwapApproved)
	if !errors.Is(err, api.ErrShiftSwapConflict) {
		t.Fatalf("approve official swap error = %v, want conflict", err)
	}
	var status string
	if err := pool.QueryRow(ctx, `SELECT status FROM shift_swap_requests WHERE id = $1`, swapID).Scan(&status); err != nil {
		t.Fatalf("read swap status: %v", err)
	}
	if status != string(api.ShiftSwapPendingAdmin) {
		t.Fatalf("swap status=%s, want rollback to pending_admin", status)
	}
}
