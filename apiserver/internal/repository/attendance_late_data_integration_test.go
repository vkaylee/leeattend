//go:build integration

package repository

import (
	"errors"
	"testing"
	"time"

	"github.com/google/uuid"
	"github.com/vkaylee/leeattend/apiserver/internal/api"
)

func TestIntegrationPunchInPinsUniqueMaterializedSegmentSnapshot(t *testing.T) {
	pool, ctx := requireIntegrationDatabase(t)
	employerID, employeeID, employmentID, actorID := uuid.New(), uuid.New(), uuid.New(), uuid.New()
	templateID, versionID, segmentID := uuid.New(), uuid.New(), uuid.New()
	mustExec := func(query string, args ...any) {
		t.Helper()
		if _, err := pool.Exec(ctx, query, args...); err != nil {
			t.Fatalf("fixture query failed: %v", err)
		}
	}
	mustExec(`INSERT INTO employers (id, name) VALUES ($1, 'segment-snapshot-employer')`, employerID)
	mustExec(`INSERT INTO employees (id, employee_id_number, full_name, hire_date, original_hire_date) VALUES ($1, $2, 'segment-snapshot-employee', DATE '2026-01-01', DATE '2026-01-01')`, employeeID, employeeID.String())
	mustExec(`INSERT INTO employments (id, employee_id, employer_id, status, start_date) VALUES ($1, $2, $3, 'active', DATE '2026-01-01')`, employmentID, employeeID, employerID)
	mustExec(`INSERT INTO users (id, email, password_hash) VALUES ($1, $2, 'integration-test-hash')`, actorID, actorID.String()+"@invalid.test")
	mustExec(`INSERT INTO shift_templates (id, name) VALUES ($1, 'segment-snapshot-template')`, templateID)
	mustExec(`INSERT INTO shift_versions (id, template_id, version_number, name, effective_from, change_type, created_by) VALUES ($1, $2, 1, 'segment-snapshot-version', DATE '2026-01-01', 'create', $3)`, versionID, templateID, actorID)
	punch := time.Date(2026, 11, 20, 9, 0, 0, 0, time.UTC)
	breaks := []byte(`[{"break_punch_mode":"required"}]`)
	mustExec(`INSERT INTO work_segments (id, employment_id, date, sort_order, scheduled_start, scheduled_end, segment_type, segment_status, compensation_rate, gap_compensation_before, gap_compensation_rate_before, source, source_detail, generation_metadata, template_id, selected_shift_version_id, scheduled_start_at, scheduled_end_at, breaks_snapshot) VALUES ($1, $2, $3::date, 0, '08:00', '17:00', 'work', 'scheduled', 1, 'unpaid', 0, 'manual', '{}'::jsonb, '{}'::jsonb, $4, $5, $3::date + TIME '08:00', $3::date + TIME '17:00', $6::jsonb)`, segmentID, employmentID, punch.Format("2006-01-02"), templateID, versionID, breaks)

	record, err := (AttendanceRepository{DB: pool}).PunchIn(ctx, employmentID, nil, "work", nil, false, punch)
	if err != nil {
		t.Fatalf("punch-in failed: %v", err)
	}
	if record.SegmentID == nil || *record.SegmentID != segmentID {
		t.Fatalf("segment pin=%v, want %s", record.SegmentID, segmentID)
	}
	if record.ShiftVersionID == nil || *record.ShiftVersionID != versionID {
		t.Fatalf("version pin=%v, want %s", record.ShiftVersionID, versionID)
	}
	if string(record.BreaksSnapshot) != `[{"break_punch_mode": "required"}]` {
		t.Fatalf("break snapshot=%s, want materialized policy", record.BreaksSnapshot)
	}
}

func TestIntegrationRawPunchIngestRemainsAllowedAfterClosedAttendanceSnapshot(t *testing.T) {
	pool, ctx := requireIntegrationDatabase(t)
	employerID, employeeID, employmentID := uuid.New(), uuid.New(), uuid.New()
	if _, err := pool.Exec(ctx, `INSERT INTO employers (id, name) VALUES ($1, 'late-data-employer')`, employerID); err != nil {
		t.Fatalf("insert employer: %v", err)
	}
	if _, err := pool.Exec(ctx, `INSERT INTO employees (id, employee_id_number, full_name, hire_date, original_hire_date) VALUES ($1, $2, 'late-data-employee', DATE '2026-01-01', DATE '2026-01-01')`, employeeID, employeeID.String()); err != nil {
		t.Fatalf("insert employee: %v", err)
	}
	if _, err := pool.Exec(ctx, `INSERT INTO employments (id, employee_id, employer_id, status, start_date) VALUES ($1, $2, $3, 'active', DATE '2026-01-01')`, employmentID, employeeID, employerID); err != nil {
		t.Fatalf("insert employment: %v", err)
	}
	if _, err := pool.Exec(ctx, `INSERT INTO attendance_periods (id, employer_id, period_year, period_month, status, version) VALUES ($1, $2, 2026, 11, 'closed', 2)`, uuid.New(), employerID); err != nil {
		t.Fatalf("insert closed attendance period: %v", err)
	}
	punch := time.Date(2026, 11, 20, 9, 0, 0, 0, time.UTC)
	if _, err := pool.Exec(ctx, `INSERT INTO raw_attendance_logs (id, employment_id, punch_time, source, raw_data, punch, status, created_at) VALUES ($1, $2, $3, 'import', '{}'::jsonb, 1, 0, NOW())`, uuid.New(), employmentID, punch); err != nil {
		t.Fatalf("late raw punch rejected: %v", err)
	}
	var count int
	if err := pool.QueryRow(ctx, `SELECT count(*) FROM raw_attendance_logs WHERE employment_id = $1 AND punch_time = $2`, employmentID, punch).Scan(&count); err != nil {
		t.Fatalf("count late raw punch: %v", err)
	}
	if count != 1 {
		t.Fatalf("late raw punch count = %d, want 1", count)
	}

	punchIn, err := (AttendanceRepository{DB: pool}).PunchIn(ctx, employmentID, nil, "work", nil, false, punch.Add(time.Hour))
	if err != nil {
		t.Fatalf("late punch-in rejected after closure: %v", err)
	}
	if punchIn.EmploymentID != employmentID {
		t.Fatalf("late punch-in employment = %s, want %s", punchIn.EmploymentID, employmentID)
	}
}

func TestIntegrationRequiredBreakWorkOutRejectsOpenOrMissingBreak(t *testing.T) {
	pool, ctx := requireIntegrationDatabase(t)
	employerID, employeeID, employmentID, actorID := uuid.New(), uuid.New(), uuid.New(), uuid.New()
	tenureID, relationshipID := uuid.New(), uuid.New()
	templateID, versionID, segmentID := uuid.New(), uuid.New(), uuid.New()
	mustExec := func(query string, args ...any) {
		t.Helper()
		if _, err := pool.Exec(ctx, query, args...); err != nil {
			t.Fatalf("fixture query failed: %v", err)
		}
	}
	mustExec(`INSERT INTO employers (id, name) VALUES ($1, 'required-break-employer')`, employerID)
	mustExec(`INSERT INTO employees (id, employee_id_number, full_name, hire_date, original_hire_date) VALUES ($1, $2, 'required-break-employee', DATE '2026-01-01', DATE '2026-01-01')`, employeeID, employeeID.String())
	mustExec(`INSERT INTO employee_tenures (id, employee_id, employee_number, start_date) VALUES ($1, $2, $3, DATE '2026-01-01')`, tenureID, employeeID, employeeID.String())
	mustExec(`INSERT INTO employment_relationships (id, employee_id, employer_id, tenure_id, status, start_date) VALUES ($1, $2, $3, $4, 'active', DATE '2026-01-01')`, relationshipID, employeeID, employerID, tenureID)
	mustExec(`INSERT INTO employments (id, employee_id, employer_id, relationship_id, status, start_date) VALUES ($1, $2, $3, $4, 'active', DATE '2026-01-01')`, employmentID, employeeID, employerID, relationshipID)
	mustExec(`INSERT INTO users (id, email, password_hash) VALUES ($1, $2, 'integration-test-hash')`, actorID, actorID.String()+"@invalid.test")
	mustExec(`INSERT INTO shift_templates (id, name) VALUES ($1, 'required-break-template')`, templateID)
	mustExec(`INSERT INTO shift_versions (id, template_id, version_number, name, effective_from, change_type, created_by) VALUES ($1, $2, 1, 'required-break-version', DATE '2026-01-01', 'create', $3)`, versionID, templateID, actorID)
	breaks := []byte(`[{"break_punch_mode":"required"}]`)
	mustExec(`INSERT INTO work_segments (id, employment_id, date, sort_order, scheduled_start, scheduled_end, segment_type, segment_status, compensation_rate, gap_compensation_before, gap_compensation_rate_before, source, source_detail, generation_metadata, template_id, selected_shift_version_id, scheduled_start_at, scheduled_end_at, breaks_snapshot) VALUES ($1, $2, DATE '2026-11-20', 0, '08:00', '17:00', 'work', 'scheduled', 1, 'unpaid', 0, 'manual', '{}'::jsonb, '{}'::jsonb, $3, $4, TIMESTAMPTZ '2026-11-20 08:00:00+00', TIMESTAMPTZ '2026-11-20 17:00:00+00', $5::jsonb)`, segmentID, employmentID, templateID, versionID, breaks)

	repo := AttendanceRepository{DB: pool}
	if _, err := repo.ESSAttendanceAction(ctx, employeeID, "work_in", nil, "required-break-in", time.Date(2026, 11, 20, 9, 0, 0, 0, time.UTC)); err != nil {
		t.Fatalf("work-in failed: %v", err)
	}
	if _, err := repo.ESSAttendanceAction(ctx, employeeID, "work_out", nil, "required-break-out", time.Date(2026, 11, 20, 17, 0, 0, 0, time.UTC)); !errors.Is(err, api.ErrAttendanceInvalidTransition) {
		t.Fatalf("work-out error=%v, want invalid transition", err)
	}
	var openCount, eventCount int
	if err := pool.QueryRow(ctx, `SELECT count(*) FROM time_entries WHERE employment_id = $1 AND punch_out_time IS NULL`, employmentID).Scan(&openCount); err != nil {
		t.Fatalf("read open entry: %v", err)
	}
	if err := pool.QueryRow(ctx, `SELECT count(*) FROM attendance_punch_events WHERE employment_id = $1`, employmentID).Scan(&eventCount); err != nil {
		t.Fatalf("read punch events: %v", err)
	}
	if openCount != 1 || eventCount != 1 {
		t.Fatalf("state after rejected work-out: open_entries=%d events=%d, want 1/1", openCount, eventCount)
	}
}
