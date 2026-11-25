//go:build integration

package repository

import (
	"errors"
	"testing"
	"time"

	"github.com/google/uuid"
	"github.com/vkaylee/leeattend/apiserver/internal/api"
)

func TestIntegrationRawLogsToSessionsSummariesAndClosureGuard(t *testing.T) {
	pool, ctx := requireIntegrationDatabase(t)

	employerID := uuid.New()
	actorID := uuid.New()
	empID, employmentID := uuid.New(), uuid.New()
	tplSplit, verSplit := uuid.New(), uuid.New()
	tplNight, verNight := uuid.New(), uuid.New()
	batchID := uuid.New()

	attPolicyID := uuid.New()
	patternID := uuid.New()
	workCreditID := uuid.New()
	payBasisID := uuid.New()
	otPolicyID := uuid.New()
	policyGroupID := uuid.New()
	policyVersionID := uuid.New()

	mustExec := func(query string, args ...any) {
		t.Helper()
		if _, err := pool.Exec(ctx, query, args...); err != nil {
			t.Fatalf("exec %q: %v", query, err)
		}
	}

	mustExec(`INSERT INTO employers (id, name) VALUES ($1, 'h6-employer')`, employerID)
	mustExec(`INSERT INTO users (id, email, password_hash) VALUES ($1, $2, 'integration-test-hash')`, actorID, actorID.String()+"@invalid.test")
	mustExec(`INSERT INTO employees (id, employee_id_number, full_name, hire_date, original_hire_date) VALUES ($1, $2, 'H6 Worker', DATE '2026-01-01', DATE '2026-01-01')`, empID, "H6-"+empID.String()[:8])
	mustExec(`INSERT INTO employments (id, employee_id, employer_id, status, start_date, end_date) VALUES ($1, $2, $3, 'active', DATE '2026-01-01', DATE '2026-12-31')`, employmentID, empID, employerID)

	mustExec(`INSERT INTO attendance_policies (id, name, is_default) VALUES ($1, 'H6 Att Policy', false)`, attPolicyID)
	mustExec(`INSERT INTO working_patterns (id, name, pattern_type, is_active) VALUES ($1, 'H6 Pattern', 'weekly', true)`, patternID)
	mustExec(`INSERT INTO work_credit_policies (id, name, minutes_per_credit, attendance_policy_id, working_pattern_id, early_ot_enabled, early_ot_qualification_grace_minutes, early_ot_max_minutes, break_auto_deduct, break_is_overtime, created_by) VALUES ($1, 'H6 Work Credit', 480, $2, $3, true, 0, 60, true, true, $4)`, workCreditID, attPolicyID, patternID, actorID)
	mustExec(`INSERT INTO pay_basis_policies (id, name, method, monthly_denominator, monthly_standard_days, created_by) VALUES ($1, 'H6 Pay Basis', 'monthly', 'fixed', 22, $2)`, payBasisID, actorID)
	mustExec(`INSERT INTO overtime_policy_definitions (id, name) VALUES ($1, 'H6 OT Policy')`, otPolicyID)
	mustExec(`INSERT INTO labor_policy_groups (id, name, created_by) VALUES ($1, 'H6 Group', $2)`, policyGroupID, actorID)
	mustExec(`INSERT INTO labor_policy_group_versions (id, group_id, work_credit_policy_id, pay_basis_policy_id, overtime_policy_id, effective_from, created_by) VALUES ($1, $2, $3, $4, $5, DATE '2026-01-01', $6)`, policyVersionID, policyGroupID, workCreditID, payBasisID, otPolicyID, actorID)
	mustExec(`INSERT INTO labor_policy_memberships (id, employment_id, group_id, effective_from, created_by) VALUES (gen_random_uuid(), $1, $2, DATE '2026-01-01', $3)`, employmentID, policyGroupID, actorID)

	for _, sh := range []struct {
		tpl, ver uuid.UUID
		name     string
	}{
		{tplSplit, verSplit, "H6 Split Shift"},
		{tplNight, verNight, "H6 Night Shift"},
	} {
		mustExec(`INSERT INTO shift_templates (id, name) VALUES ($1, $2)`, sh.tpl, sh.name)
		mustExec(`INSERT INTO shift_versions (id, template_id, version_number, name, effective_from, change_type, created_by) VALUES ($1, $2, 1, $3, DATE '2026-01-01', 'create', $4)`, sh.ver, sh.tpl, sh.name+" v1", actorID)
	}

	mustExec(`INSERT INTO schedule_publication_batches (id, idempotency_key, request_hash, mode, status, requested_by, operation, response_snapshot, reason, timezone, range_start, range_end) VALUES ($1, $2, repeat('f', 64), 'rotating', 'published', $3, 'publish', '{}'::jsonb, 'h6 fixture', 'Asia/Ho_Chi_Minh', DATE '2026-11-10', DATE '2026-11-12')`, batchID, batchID.String(), actorID)

	type segSpec struct {
		id       uuid.UUID
		tplID    uuid.UUID
		verID    uuid.UUID
		sort     int
		startLoc string
		endLoc   string
		startAt  string
		endAt    string
		breaks   string
	}
	publishDay := func(dateStr string, segs []segSpec) {
		t.Helper()
		revID := uuid.New()
		tx, err := pool.Begin(ctx)
		if err != nil {
			t.Fatalf("begin tx: %v", err)
		}
		if _, err := tx.Exec(ctx, `SELECT set_config('schedule.lifecycle_publication', 'on', true)`); err != nil {
			_ = tx.Rollback(ctx)
			t.Fatalf("set config: %v", err)
		}
		if _, err := tx.Exec(ctx, `INSERT INTO schedule_day_revisions (id, employment_id, employer_id, business_date, timezone, revision_number, state, publication_batch_id, source, source_snapshot, resolver_version, reason, published_by, published_actor_type, published_at) VALUES ($1, $2, $3, $4::date, 'Asia/Ho_Chi_Minh', 1, 'draft', $5, 'manual', '{}'::jsonb, 'schedule-resolver-v1', 'h6', $6, 'human', NULL)`, revID, employmentID, employerID, dateStr, batchID, actorID); err != nil {
			_ = tx.Rollback(ctx)
			t.Fatalf("insert rev %s: %v", dateStr, err)
		}
		for _, s := range segs {
			bJSON := s.breaks
			if bJSON == "" {
				bJSON = "[]"
			}
			if _, err := tx.Exec(ctx, `INSERT INTO work_segments (id, employment_id, date, sort_order, scheduled_start, scheduled_end, segment_type, segment_status, compensation_rate, gap_compensation_before, gap_compensation_rate_before, source, source_detail, generation_metadata, template_id, schedule_revision_id, selected_shift_version_id, scheduled_start_at, scheduled_end_at, breaks_snapshot) VALUES ($1, $2, $3::date, $4, $5::time, $6::time, 'work', 'scheduled', 1, 'unpaid', 0, 'manual', '{}'::jsonb, '{}'::jsonb, $7, $8, $9, $10::timestamptz, $11::timestamptz, $12::jsonb)`, s.id, employmentID, dateStr, s.sort, s.startLoc, s.endLoc, s.tplID, revID, s.verID, s.startAt, s.endAt, bJSON); err != nil {
				_ = tx.Rollback(ctx)
				t.Fatalf("insert seg %s: %v", dateStr, err)
			}
		}
		if _, err := tx.Exec(ctx, `UPDATE schedule_day_revisions SET state = 'published', published_at = CURRENT_TIMESTAMP WHERE id = $1`, revID); err != nil {
			_ = tx.Rollback(ctx)
			t.Fatalf("publish rev %s: %v", dateStr, err)
		}
		if err := tx.Commit(ctx); err != nil {
			t.Fatalf("commit rev %s: %v", dateStr, err)
		}
	}

	// Day 1 (2026-11-10): split shift with 2 segments; morning has TWO overlapping breaks (10:00-10:15 + 10:10-10:30 => 30m merged).
	segD1M, segD1A := uuid.New(), uuid.New()
	publishDay("2026-11-10", []segSpec{
		{
			id: segD1M, tplID: tplSplit, verID: verSplit, sort: 0,
			startLoc: "08:00:00", endLoc: "12:00:00",
			startAt: "2026-11-10T08:00:00+07:00", endAt: "2026-11-10T12:00:00+07:00",
			breaks: `[{"start_at":"2026-11-10T10:00:00+07:00","end_at":"2026-11-10T10:15:00+07:00","duration_minutes":15},{"start_at":"2026-11-10T10:10:00+07:00","end_at":"2026-11-10T10:30:00+07:00","duration_minutes":20}]`,
		},
		{
			id: segD1A, tplID: tplSplit, verID: verSplit, sort: 1,
			startLoc: "14:00:00", endLoc: "18:00:00",
			startAt: "2026-11-10T14:00:00+07:00", endAt: "2026-11-10T18:00:00+07:00",
		},
	})

	// Day 2 (2026-11-11): overnight shift 23:00-07:00 +07:00 (starts 16:00 UTC Nov 11, ends 00:00 UTC Nov 12).
	segD2N := uuid.New()
	publishDay("2026-11-11", []segSpec{
		{
			id: segD2N, tplID: tplNight, verID: verNight, sort: 0,
			startLoc: "23:00:00", endLoc: "07:00:00",
			startAt: "2026-11-11T23:00:00+07:00", endAt: "2026-11-12T07:00:00+07:00",
		},
	})

	// Day 3 (2026-11-12): split shift where morning is complete and afternoon has ONLY punch-in (missing OUT).
	segD3M, segD3A := uuid.New(), uuid.New()
	publishDay("2026-11-12", []segSpec{
		{
			id: segD3M, tplID: tplSplit, verID: verSplit, sort: 0,
			startLoc: "08:00:00", endLoc: "12:00:00",
			startAt: "2026-11-12T08:00:00+07:00", endAt: "2026-11-12T12:00:00+07:00",
		},
		{
			id: segD3A, tplID: tplSplit, verID: verSplit, sort: 1,
			startLoc: "14:00:00", endLoc: "18:00:00",
			startAt: "2026-11-12T14:00:00+07:00", endAt: "2026-11-12T18:00:00+07:00",
		},
	})

	// Insert out-of-order + bounce-duplicate raw logs across all 3 days.
	rawPunches := []string{
		// Day 1: morning 08:00 (+20s duplicate bounce), 12:00; afternoon 14:00, 18:00
		"2026-11-10T18:00:00+07:00",
		"2026-11-10T08:00:20+07:00",
		"2026-11-10T12:00:00+07:00",
		"2026-11-10T14:00:00+07:00",
		"2026-11-10T08:00:00+07:00",
		// Day 2: overnight 23:00 -> 07:00 next morning
		"2026-11-12T07:00:00+07:00",
		"2026-11-11T23:00:00+07:00",
		// Day 3: morning 08:00 -> 12:00; afternoon 14:05 only (missing OUT)
		"2026-11-12T08:00:00+07:00",
		"2026-11-12T12:00:00+07:00",
		"2026-11-12T14:05:00+07:00",
	}
	for _, ts := range rawPunches {
		mustExec(`INSERT INTO raw_attendance_logs (id, employment_id, punch_time, source, raw_data, punch, status, created_at) VALUES (gen_random_uuid(), $1, $2::timestamptz, 'import', '{}'::jsonb, 0, 0, NOW())`, employmentID, ts)
	}

	repo := AttendanceRepository{DB: pool}
	resp, err := repo.ReprocessAttendance(ctx, "2026-11-10", "2026-11-12", &empID)
	if err != nil {
		t.Fatalf("ReprocessAttendance failed: %v", err)
	}
	if resp.Days != 3 {
		t.Fatalf("expected 3 reprocessed business days, got %+v", resp)
	}

	// Re-running ReprocessAttendance must be idempotent (no duplicate time_entries).
	if _, err := repo.ReprocessAttendance(ctx, "2026-11-10", "2026-11-12", &empID); err != nil {
		t.Fatalf("second ReprocessAttendance failed: %v", err)
	}

	var totalEntries, closedEntries, openEntries int
	if err := pool.QueryRow(ctx, `SELECT count(*), count(*) FILTER (WHERE punch_out_time IS NOT NULL), count(*) FILTER (WHERE punch_out_time IS NULL) FROM time_entries WHERE employment_id = $1`, employmentID).Scan(&totalEntries, &closedEntries, &openEntries); err != nil {
		t.Fatalf("count time_entries: %v", err)
	}
	if totalEntries != 5 || closedEntries != 4 || openEntries != 1 {
		t.Fatalf("expected 5 time_entries (4 closed + 1 open missing-out), got total=%d closed=%d open=%d", totalEntries, closedEntries, openEntries)
	}

	// Day 1 summary: 480 actual, 30 merged break_ot (NOT double-counted as 35), net_paid must NOT include unapproved break_ot!
	var d1Actual, d1Standard, d1BreakOT, d1SysOT, d1ApprovedOT, d1NetPaid int
	if err := pool.QueryRow(ctx, `SELECT actual_work_minutes, standard_minutes, break_ot_minutes, system_detected_ot_minutes, approved_ot_minutes, net_paid_minutes FROM attendance_daily_summaries WHERE employment_id = $1 AND date = DATE '2026-11-10'`, employmentID).Scan(&d1Actual, &d1Standard, &d1BreakOT, &d1SysOT, &d1ApprovedOT, &d1NetPaid); err != nil {
		t.Fatalf("load Day 1 summary: %v", err)
	}
	if d1Actual != 480 || d1Standard != 480 || d1BreakOT != 30 || d1SysOT != 30 || d1ApprovedOT != 0 || d1NetPaid != 480 {
		t.Fatalf("unexpected Day 1 summary (must merge breaks to 30m and keep unapproved break OT out of net_paid): actual=%d standard=%d breakOT=%d sysOT=%d approvedOT=%d netPaid=%d", d1Actual, d1Standard, d1BreakOT, d1SysOT, d1ApprovedOT, d1NetPaid)
	}

	// Day 2 summary: overnight shift grouped under business date 2026-11-11 with 480 minutes.
	var d2Actual, d2Standard int
	if err := pool.QueryRow(ctx, `SELECT actual_work_minutes, standard_minutes FROM attendance_daily_summaries WHERE employment_id = $1 AND date = DATE '2026-11-11'`, employmentID).Scan(&d2Actual, &d2Standard); err != nil {
		t.Fatalf("load Day 2 summary: %v", err)
	}
	if d2Actual != 480 || d2Standard != 480 {
		t.Fatalf("expected Day 2 overnight summary 480/480, got actual=%d standard=%d", d2Actual, d2Standard)
	}

	// Day 3 summary: morning 240m completed; afternoon missing OUT stays open without crashing or stealing punches.
	var d3Actual int
	if err := pool.QueryRow(ctx, `SELECT actual_work_minutes FROM attendance_daily_summaries WHERE employment_id = $1 AND date = DATE '2026-11-12'`, employmentID).Scan(&d3Actual); err != nil {
		t.Fatalf("load Day 3 summary: %v", err)
	}
	if d3Actual != 240 {
		t.Fatalf("expected Day 3 completed morning 240m, got %d", d3Actual)
	}

	// Closed period guard: once attendance period is closed, ReprocessAttendance MUST return ErrAttendanceLockedPeriod.
	mustExec(`INSERT INTO attendance_periods (id, employer_id, period_year, period_month, status, version, created_at, updated_at) VALUES (gen_random_uuid(), $1, 2026, 11, 'closed', 1, NOW(), NOW())`, employerID)
	_, err = repo.ReprocessAttendance(ctx, "2026-11-10", "2026-11-12", &empID)
	if !errors.Is(err, api.ErrAttendanceLockedPeriod) {
		t.Fatalf("expected ErrAttendanceLockedPeriod when reprocessing closed attendance period, got %v", err)
	}
	_ = time.UTC
}
