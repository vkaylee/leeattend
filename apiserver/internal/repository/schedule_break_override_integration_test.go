//go:build integration

package repository

import (
	"errors"
	"testing"

	"github.com/google/uuid"
	"github.com/vkaylee/leeattend/apiserver/internal/api"
)

type breakOverrideFixture struct {
	employmentID uuid.UUID
	employerID   uuid.UUID
	segmentID    uuid.UUID
	actorID      uuid.UUID
	date         string
}

func seedBreakOverrideFixture(t *testing.T) *breakOverrideFixture {
	t.Helper()
	pool, ctx := requireIntegrationDatabase(t)
	fixture := &breakOverrideFixture{employmentID: uuid.New(), employerID: uuid.New(), segmentID: uuid.New(), actorID: uuid.New(), date: "2026-11-20"}
	employeeID, tenureID, relationshipID := uuid.New(), uuid.New(), uuid.New()
	templateID, versionID, batchID, revisionID := uuid.New(), uuid.New(), uuid.New(), uuid.New()
	exec := func(query string, args ...any) {
		t.Helper()
		if _, err := pool.Exec(ctx, query, args...); err != nil {
			t.Fatalf("break override fixture query failed: %v", err)
		}
	}
	exec(`INSERT INTO employers (id, name) VALUES ($1, 'break-override-employer')`, fixture.employerID)
	exec(`INSERT INTO employees (id, employee_id_number, full_name, hire_date, original_hire_date) VALUES ($1, $2, 'break-override-employee', DATE '2026-01-01', DATE '2026-01-01')`, employeeID, employeeID.String())
	exec(`INSERT INTO employee_tenures (id, employee_id, employee_number, start_date) VALUES ($1, $2, $3, DATE '2026-01-01')`, tenureID, employeeID, employeeID.String())
	exec(`INSERT INTO employment_relationships (id, employee_id, employer_id, tenure_id, status, start_date) VALUES ($1, $2, $3, $4, 'active', DATE '2026-01-01')`, relationshipID, employeeID, fixture.employerID, tenureID)
	exec(`INSERT INTO employments (id, employee_id, employer_id, relationship_id, status, start_date) VALUES ($1, $2, $3, $4, 'active', DATE '2026-01-01')`, fixture.employmentID, employeeID, fixture.employerID, relationshipID)
	exec(`INSERT INTO users (id, email, password_hash) VALUES ($1, $2, 'integration-test-hash')`, fixture.actorID, fixture.actorID.String()+"@invalid.test")
	exec(`INSERT INTO payroll_periods (id, employer_id, period_year, period_month, status, version) VALUES ($1, $2, 2026, 11, 'open', 1)`, uuid.New(), fixture.employerID)
	exec(`INSERT INTO shift_templates (id, name) VALUES ($1, $2)`, templateID, "break-override-template-"+templateID.String())
	exec(`INSERT INTO shift_versions (id, template_id, version_number, name, effective_from, change_type, created_by) VALUES ($1, $2, 1, 'break-override-version', DATE '2026-01-01', 'create', $3)`, versionID, templateID, fixture.actorID)
	exec(`INSERT INTO schedule_publication_batches (id, idempotency_key, request_hash, mode, status, requested_by, operation, response_snapshot, reason, timezone, range_start, range_end) VALUES ($1, $2, repeat('b', 64), 'rotating', 'published', $3, 'publish', '{}'::jsonb, 'break override fixture', 'UTC', $4::date, $4::date)`, batchID, batchID.String(), fixture.actorID, fixture.date)
	exec(`INSERT INTO schedule_day_revisions (id, employment_id, employer_id, business_date, timezone, revision_number, state, publication_batch_id, source, source_snapshot, resolver_version, reason, published_by, published_actor_type, published_at) VALUES ($1, $2, $3, $4::date, 'UTC', 1, 'draft', $5, 'manual', '{}'::jsonb, 'integration-test', 'break override fixture', $6, 'human', NULL)`, revisionID, fixture.employmentID, fixture.employerID, fixture.date, batchID, fixture.actorID)
	breaks := []byte(`[{"start_at":"2026-11-20T12:00:00Z","end_at":"2026-11-20T13:00:00Z","break_punch_mode":"none"}]`)
	exec(`INSERT INTO work_segments (id, employment_id, date, sort_order, scheduled_start, scheduled_end, segment_type, segment_status, compensation_rate, gap_compensation_before, gap_compensation_rate_before, source, source_detail, generation_metadata, template_id, schedule_revision_id, selected_shift_version_id, scheduled_start_at, scheduled_end_at, breaks_snapshot) VALUES ($1, $2, $3::date, 0, '08:00', '17:00', 'work', 'scheduled', 1, 'unpaid', 0, 'manual', '{}'::jsonb, '{}'::jsonb, $4, $5, $6, TIMESTAMPTZ '2026-11-20 08:00:00+00', TIMESTAMPTZ '2026-11-20 17:00:00+00', $7::jsonb)`, fixture.segmentID, fixture.employmentID, fixture.date, templateID, revisionID, versionID, breaks)
	exec(`UPDATE schedule_day_revisions SET state = 'published', published_at = CURRENT_TIMESTAMP WHERE id = $1`, revisionID)
	return fixture
}

func TestIntegrationMarkBreakWorkedPersistsAuditAndRejectsOverlap(t *testing.T) {
	fixture := seedBreakOverrideFixture(t)
	pool, ctx := requireIntegrationDatabase(t)
	repo := ScheduleRepository{DB: pool}
	start := "2026-11-20T12:15:00Z"
	end := "2026-11-20T12:45:00Z"
	result, err := repo.MarkBreakWorked(ctx, fixture.actorID, fixture.employmentID, fixture.date, api.MarkBreakWorkedRequest{SegmentID: fixture.segmentID, Start: start, End: end, Reason: "Production coverage"})
	if err != nil {
		t.Fatalf("mark break worked: %v", err)
	}
	if result.Minutes != 30 || result.Status != "approved" {
		t.Fatalf("unexpected override result: %+v", result)
	}
	var overrideCount, auditCount int
	if err := pool.QueryRow(ctx, `SELECT count(*) FROM schedule_day_overrides WHERE id = $1`, result.ID).Scan(&overrideCount); err != nil {
		t.Fatalf("read override: %v", err)
	}
	if err := pool.QueryRow(ctx, `SELECT count(*) FROM audit_logs WHERE target_resource_id = $1 AND action = 'schedule.break_worked.marked'`, result.ID).Scan(&auditCount); err != nil {
		t.Fatalf("read audit: %v", err)
	}
	if overrideCount != 1 || auditCount != 1 {
		t.Fatalf("evidence override=%d audit=%d, want 1/1", overrideCount, auditCount)
	}
	_, err = repo.MarkBreakWorked(ctx, fixture.actorID, fixture.employmentID, fixture.date, api.MarkBreakWorkedRequest{SegmentID: fixture.segmentID, Start: "2026-11-20T12:30:00Z", End: "2026-11-20T12:50:00Z", Reason: "Overlap"})
	if !errors.Is(err, api.ErrScheduleBreakOverrideInvalid) {
		t.Fatalf("overlap error=%v, want invalid", err)
	}
}

func TestIntegrationMarkBreakWorkedRejectsOutsideBreak(t *testing.T) {
	fixture := seedBreakOverrideFixture(t)
	pool, ctx := requireIntegrationDatabase(t)
	_, err := (ScheduleRepository{DB: pool}).MarkBreakWorked(ctx, fixture.actorID, fixture.employmentID, fixture.date, api.MarkBreakWorkedRequest{SegmentID: fixture.segmentID, Start: "2026-11-20T11:00:00Z", End: "2026-11-20T12:30:00Z", Reason: "Outside break"})
	if !errors.Is(err, api.ErrScheduleBreakOverrideInvalid) {
		t.Fatalf("outside break error=%v, want invalid", err)
	}
}
