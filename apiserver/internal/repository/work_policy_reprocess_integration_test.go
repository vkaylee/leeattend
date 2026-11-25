//go:build integration

package repository

import (
	"errors"
	"testing"

	"github.com/google/uuid"
	"github.com/vkaylee/leeattend/apiserver/internal/api"
)

func TestIntegrationWorkPolicyRulesLateGraceOTAggregationAndBreaks(t *testing.T) {
	pool, ctx := requireIntegrationDatabase(t)

	employerID := uuid.New()
	actorID := uuid.New()
	patternID := uuid.New()
	otPolicyID := uuid.New()
	otVerID := uuid.New()
	batchID := uuid.New()

	mustExec := func(query string, args ...any) {
		t.Helper()
		if _, err := pool.Exec(ctx, query, args...); err != nil {
			t.Fatalf("exec %q: %v", query, err)
		}
	}

	mustExec(`INSERT INTO employers (id, name) VALUES ($1, 'wp-reprocess-employer')`, employerID)
	mustExec(`INSERT INTO users (id, email, password_hash) VALUES ($1, $2, 'integration-test-hash')`, actorID, actorID.String()+"@invalid.test")
	mustExec(`INSERT INTO working_patterns (id, name, pattern_type, is_active) VALUES ($1, 'WP Pattern', 'weekly', true)`, patternID)
	mustExec(`INSERT INTO overtime_policy_definitions (id, name) VALUES ($1, 'WP OT Policy')`, otPolicyID)
	mustExec(`INSERT INTO overtime_policy_versions (id, policy_id, version_no, status, rules, created_by, published_by, published_at) VALUES ($1, $2, 1, 'published', '[{"priority":10,"day_type":"any","days_of_week":127,"multiplier":"1.5"}]'::jsonb, $3, $3, NOW())`, otVerID, otPolicyID, actorID)
	mustExec(`INSERT INTO schedule_publication_batches (id, idempotency_key, request_hash, mode, status, requested_by, operation, response_snapshot, reason, timezone, range_start, range_end) VALUES ($1, $2, repeat('e', 64), 'rotating', 'published', $3, 'publish', '{}'::jsonb, 'wp fixture', 'Asia/Ho_Chi_Minh', DATE '2026-11-10', DATE '2026-11-16')`, batchID, batchID.String(), actorID)

	tplID, verID := uuid.New(), uuid.New()
	mustExec(`INSERT INTO shift_templates (id, name) VALUES ($1, 'WP Shift')`, tplID)
	mustExec(`INSERT INTO shift_versions (id, template_id, version_number, name, effective_from, change_type, created_by) VALUES ($1, $2, 1, 'WP Shift v1', DATE '2026-01-01', 'create', $3)`, verID, tplID, actorID)

	createWorker := func(label string, lateGrace int, earlyEnabled bool, earlyGrace int, earlyMax *int, breakAuto, breakOT bool, aggMode string, assignPolicy bool) (uuid.UUID, uuid.UUID) {
		empID, employmentID := uuid.New(), uuid.New()
		seedIntegrationEmployeeGraphWithRange(t, ctx, pool, empID, employerID, employmentID, label+"-"+empID.String()[:8], label, "2026-01-01", "2026-01-01", "2026-12-31")
		if assignPolicy {
			var relID uuid.UUID
			if err := pool.QueryRow(ctx, `SELECT relationship_id FROM employments WHERE id = $1`, employmentID).Scan(&relID); err != nil {
				t.Fatalf("lookup relationship_id: %v", err)
			}
			groupID, verGroupID := uuid.New(), uuid.New()
			mustExec(`INSERT INTO work_policy_groups (id, code, name, created_by) VALUES ($1, $2, $3, $4)`, groupID, "GRP-"+groupID.String()[:8], "Grp-"+label, actorID)
			mustExec(`INSERT INTO work_policy_versions (id, group_id, version_no, status, effective_from, created_by, published_by, published_at, minutes_per_credit, work_unit_minutes, rounding_policy, early_arrival_strategy, late_arrival_grace_minutes, early_ot_enabled, early_ot_qualification_grace_minutes, early_ot_max_minutes, break_auto_deduct, break_is_overtime, ot_aggregation_mode, working_pattern_id, pay_method, monthly_denominator, monthly_standard_days, overtime_policy_version_id) VALUES ($1, $2, 1, 'published', DATE '2026-01-01', $3, $3, NOW(), 480, 15, 'floor', 'clamp', $4, $5, $6, $7, $8, $9, $10, $11, 'monthly', 'fixed', 22, $12)`, verGroupID, groupID, actorID, lateGrace, earlyEnabled, earlyGrace, earlyMax, breakAuto, breakOT, aggMode, patternID, otVerID)
			mustExec(`INSERT INTO work_policy_memberships (id, employment_relationship_id, group_id, effective_from, effective_to, created_by) VALUES (gen_random_uuid(), $1, $2, DATE '2026-01-01', DATE '2027-01-01', $3)`, relID, groupID, actorID)
		}
		return empID, employmentID
	}

	type segSpec struct {
		id       uuid.UUID
		sort     int
		startLoc string
		endLoc   string
		startAt  string
		endAt    string
		breaks   string
	}
	publishDay := func(employmentID uuid.UUID, dateStr string, segs []segSpec) {
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
		if _, err := tx.Exec(ctx, `INSERT INTO schedule_day_revisions (id, employment_id, employer_id, business_date, timezone, revision_number, state, publication_batch_id, source, source_snapshot, resolver_version, reason, published_by, published_actor_type, published_at) VALUES ($1, $2, $3, $4::date, 'Asia/Ho_Chi_Minh', 1, 'draft', $5, 'manual', '{}'::jsonb, 'schedule-resolver-v1', 'wp', $6, 'human', NULL)`, revID, employmentID, employerID, dateStr, batchID, actorID); err != nil {
			_ = tx.Rollback(ctx)
			t.Fatalf("insert rev: %v", err)
		}
		for _, s := range segs {
			bJSON := s.breaks
			if bJSON == "" {
				bJSON = "[]"
			}
			if _, err := tx.Exec(ctx, `INSERT INTO work_segments (id, employment_id, date, sort_order, scheduled_start, scheduled_end, segment_type, segment_status, compensation_rate, gap_compensation_before, gap_compensation_rate_before, source, source_detail, generation_metadata, template_id, schedule_revision_id, selected_shift_version_id, scheduled_start_at, scheduled_end_at, breaks_snapshot) VALUES ($1, $2, $3::date, $4, $5::time, $6::time, 'work', 'scheduled', 1, 'unpaid', 0, 'manual', '{}'::jsonb, '{}'::jsonb, $7, $8, $9, $10::timestamptz, $11::timestamptz, $12::jsonb)`, s.id, employmentID, dateStr, s.sort, s.startLoc, s.endLoc, tplID, revID, verID, s.startAt, s.endAt, bJSON); err != nil {
				_ = tx.Rollback(ctx)
				t.Fatalf("insert seg: %v", err)
			}
		}
		if _, err := tx.Exec(ctx, `UPDATE schedule_day_revisions SET state = 'published', published_at = CURRENT_TIMESTAMP WHERE id = $1`, revID); err != nil {
			_ = tx.Rollback(ctx)
			t.Fatalf("publish rev: %v", err)
		}
		if err := tx.Commit(ctx); err != nil {
			t.Fatalf("commit rev: %v", err)
		}
	}

	insertTimeEntry := func(employmentID, segID uuid.UUID, inTS, outTS string) uuid.UUID {
		entryID := uuid.New()
		mustExec(`INSERT INTO time_entries (id, employment_id, segment_id, template_id, shift_version_id, breaks_snapshot, punch_in_time, punch_out_time, duration_minutes, entry_type, is_break_overtime, created_at, updated_at) SELECT $1, $2, $3, ws.template_id, ws.selected_shift_version_id, ws.breaks_snapshot, $4::timestamptz, $5::timestamptz, GREATEST(0, FLOOR(EXTRACT(EPOCH FROM ($5::timestamptz - $4::timestamptz)) / 60)::integer), 'work', false, NOW(), NOW() FROM work_segments ws WHERE ws.id = $3`, entryID, employmentID, segID, inTS, outTS)
		return entryID
	}

	repo := AttendanceRepository{DB: pool}
	max30 := 30

	// Scenario 1: Late arrival grace boundary (<= 5m waived, 8m recorded) + Aggregate Early OT ((10+10)-15 = 5m) + break_worked override (60m - 30m = 30m deducted).
	empAgg, employmentAgg := createWorker("AggWorker", 5, true, 15, &max30, true, false, "aggregate", true)
	seg1A, seg1B := uuid.New(), uuid.New()
	publishDay(employmentAgg, "2026-11-10", []segSpec{
		{id: seg1A, sort: 0, startLoc: "08:00:00", endLoc: "12:00:00", startAt: "2026-11-10T08:00:00+07:00", endAt: "2026-11-10T12:00:00+07:00"},
		{id: seg1B, sort: 1, startLoc: "14:00:00", endLoc: "18:00:00", startAt: "2026-11-10T14:00:00+07:00", endAt: "2026-11-10T18:00:00+07:00"},
	})
	insertTimeEntry(employmentAgg, seg1A, "2026-11-10T07:50:00+07:00", "2026-11-10T12:00:00+07:00")
	insertTimeEntry(employmentAgg, seg1B, "2026-11-10T13:50:00+07:00", "2026-11-10T18:00:00+07:00")

	// Day 2: late 5m (within grace => 0) & break_worked override 30m inside 60m scheduled break.
	seg2 := uuid.New()
	publishDay(employmentAgg, "2026-11-11", []segSpec{
		{
			id: seg2, sort: 0, startLoc: "08:00:00", endLoc: "17:00:00",
			startAt: "2026-11-11T08:00:00+07:00", endAt: "2026-11-11T17:00:00+07:00",
			breaks: `[{"start_at":"2026-11-11T12:00:00+07:00","end_at":"2026-11-11T13:00:00+07:00","duration_minutes":60,"break_punch_mode":"none"}]`,
		},
	})
	insertTimeEntry(employmentAgg, seg2, "2026-11-11T08:05:00+07:00", "2026-11-11T17:00:00+07:00")
	mustExec(`INSERT INTO schedule_day_overrides (id, employment_id, business_date, work_segment_id, override_type, start_at, end_at, minutes, reason, status, created_by) VALUES (gen_random_uuid(), $1, DATE '2026-11-11', $2, 'break_worked', TIMESTAMPTZ '2026-11-11T12:00:00+07:00', TIMESTAMPTZ '2026-11-11T12:30:00+07:00', 30, 'coverage', 'approved', $3)`, employmentAgg, seg2, actorID)

	// Day 3: late 8m (> 5m grace => late_minutes = 8).
	seg3 := uuid.New()
	publishDay(employmentAgg, "2026-11-12", []segSpec{
		{id: seg3, sort: 0, startLoc: "08:00:00", endLoc: "17:00:00", startAt: "2026-11-12T08:00:00+07:00", endAt: "2026-11-12T17:00:00+07:00"},
	})
	insertTimeEntry(employmentAgg, seg3, "2026-11-12T08:08:00+07:00", "2026-11-12T17:00:00+07:00")

	if _, err := repo.ReprocessAttendance(ctx, "2026-11-10", "2026-11-12", &empAgg); err != nil {
		t.Fatalf("reprocess AggWorker: %v", err)
	}

	var earlyOT, netPaid int
	if err := pool.QueryRow(ctx, `SELECT early_ot_minutes, net_paid_minutes FROM attendance_daily_summaries WHERE employment_id = $1 AND date = DATE '2026-11-10'`, employmentAgg).Scan(&earlyOT, &netPaid); err != nil {
		t.Fatalf("query agg day 1: %v", err)
	}
	if earlyOT != 5 || netPaid != 480 {
		t.Fatalf("aggregate day 1: want earlyOT=5 netPaid=480, got earlyOT=%d netPaid=%d", earlyOT, netPaid)
	}

	var lateD2, deductedD2, stdD2 int
	if err := pool.QueryRow(ctx, `SELECT late_minutes, deducted_break_minutes, standard_minutes FROM attendance_daily_summaries WHERE employment_id = $1 AND date = DATE '2026-11-11'`, employmentAgg).Scan(&lateD2, &deductedD2, &stdD2); err != nil {
		t.Fatalf("query agg day 2: %v", err)
	}
	if lateD2 != 0 || deductedD2 != 30 || stdD2 != 505 {
		t.Fatalf("day 2 grace+override: want late=0 deducted=30 std=505, got late=%d deducted=%d std=%d", lateD2, deductedD2, stdD2)
	}

	var lateD3 int
	if err := pool.QueryRow(ctx, `SELECT late_minutes FROM attendance_daily_summaries WHERE employment_id = $1 AND date = DATE '2026-11-12'`, employmentAgg).Scan(&lateD3); err != nil {
		t.Fatalf("query agg day 3: %v", err)
	}
	if lateD3 != 8 {
		t.Fatalf("day 3 tardiness: want late=8, got %d", lateD3)
	}

	// Scenario 2: per_segment Early OT (10m + 10m with 15m grace => 0m) + break_punch_mode=required (45m actual vs missing pair).
	empPerSeg, employmentPerSeg := createWorker("PerSegWorker", 0, true, 15, &max30, true, false, "per_segment", true)
	pSeg1A, pSeg1B := uuid.New(), uuid.New()
	publishDay(employmentPerSeg, "2026-11-13", []segSpec{
		{id: pSeg1A, sort: 0, startLoc: "08:00:00", endLoc: "12:00:00", startAt: "2026-11-13T08:00:00+07:00", endAt: "2026-11-13T12:00:00+07:00"},
		{id: pSeg1B, sort: 1, startLoc: "14:00:00", endLoc: "18:00:00", startAt: "2026-11-13T14:00:00+07:00", endAt: "2026-11-13T18:00:00+07:00"},
	})
	insertTimeEntry(employmentPerSeg, pSeg1A, "2026-11-13T07:50:00+07:00", "2026-11-13T12:00:00+07:00")
	insertTimeEntry(employmentPerSeg, pSeg1B, "2026-11-13T13:50:00+07:00", "2026-11-13T18:00:00+07:00")

	// Day 14: break_punch_mode=required with 45m actual break punch.
	pSegReqOK := uuid.New()
	publishDay(employmentPerSeg, "2026-11-14", []segSpec{
		{
			id: pSegReqOK, sort: 0, startLoc: "08:00:00", endLoc: "17:00:00",
			startAt: "2026-11-14T08:00:00+07:00", endAt: "2026-11-14T17:00:00+07:00",
			breaks: `[{"start_at":"2026-11-14T12:00:00+07:00","end_at":"2026-11-14T13:00:00+07:00","duration_minutes":60,"break_punch_mode":"required"}]`,
		},
	})
	reqEntryOK := insertTimeEntry(employmentPerSeg, pSegReqOK, "2026-11-14T08:00:00+07:00", "2026-11-14T17:00:00+07:00")
	mustExec(`INSERT INTO time_entry_breaks (id, time_entry_id, break_start_at, break_end_at, source, close_reason) VALUES (gen_random_uuid(), $1, TIMESTAMPTZ '2026-11-14T12:00:00+07:00', TIMESTAMPTZ '2026-11-14T12:45:00+07:00', 'punch', 'punch')`, reqEntryOK)

	// Day 15: break_punch_mode=required missing break punch -> provisional + missing_break_pair.
	pSegReqMiss := uuid.New()
	publishDay(employmentPerSeg, "2026-11-15", []segSpec{
		{
			id: pSegReqMiss, sort: 0, startLoc: "08:00:00", endLoc: "17:00:00",
			startAt: "2026-11-15T08:00:00+07:00", endAt: "2026-11-15T17:00:00+07:00",
			breaks: `[{"start_at":"2026-11-15T12:00:00+07:00","end_at":"2026-11-15T13:00:00+07:00","duration_minutes":60,"break_punch_mode":"required"}]`,
		},
	})
	insertTimeEntry(employmentPerSeg, pSegReqMiss, "2026-11-15T08:00:00+07:00", "2026-11-15T17:00:00+07:00")

	if _, err := repo.ReprocessAttendance(ctx, "2026-11-13", "2026-11-15", &empPerSeg); err != nil {
		t.Fatalf("reprocess PerSegWorker: %v", err)
	}

	var perSegEarlyOT int
	if err := pool.QueryRow(ctx, `SELECT early_ot_minutes FROM attendance_daily_summaries WHERE employment_id = $1 AND date = DATE '2026-11-13'`, employmentPerSeg).Scan(&perSegEarlyOT); err != nil {
		t.Fatalf("query per_segment day: %v", err)
	}
	if perSegEarlyOT != 0 {
		t.Fatalf("per_segment day 13: want earlyOT=0, got %d", perSegEarlyOT)
	}

	var actBreak, dedBreak, stdReqOK int
	var provOK bool
	var excOK *string
	if err := pool.QueryRow(ctx, `SELECT actual_break_minutes, deducted_break_minutes, standard_minutes, break_provisional, break_exception_code FROM attendance_daily_summaries WHERE employment_id = $1 AND date = DATE '2026-11-14'`, employmentPerSeg).Scan(&actBreak, &dedBreak, &stdReqOK, &provOK, &excOK); err != nil {
		t.Fatalf("query required break ok: %v", err)
	}
	if actBreak != 45 || dedBreak != 45 || stdReqOK != 495 || provOK || excOK != nil {
		t.Fatalf("required break ok: want act=45 ded=45 std=495 prov=false exc=nil, got act=%d ded=%d std=%d prov=%v exc=%v", actBreak, dedBreak, stdReqOK, provOK, excOK)
	}

	var provMiss bool
	var excMiss *string
	var dedMiss, stdMiss int
	if err := pool.QueryRow(ctx, `SELECT break_provisional, break_exception_code, deducted_break_minutes, standard_minutes FROM attendance_daily_summaries WHERE employment_id = $1 AND date = DATE '2026-11-15'`, employmentPerSeg).Scan(&provMiss, &excMiss, &dedMiss, &stdMiss); err != nil {
		t.Fatalf("query required break missing: %v", err)
	}
	if !provMiss || excMiss == nil || *excMiss != "missing_break_pair" || dedMiss != 0 || stdMiss != 540 {
		t.Fatalf("required break missing: want prov=true exc=missing_break_pair ded=0 std=540, got prov=%v exc=%v ded=%d std=%d", provMiss, excMiss, dedMiss, stdMiss)
	}

	// Scenario 3: Missing Work Policy -> ErrAttendancePolicyMissing + zero rows written (atomic TX rollback).
	empNoPol, employmentNoPol := createWorker("NoPolicyWorker", 0, false, 0, nil, true, false, "aggregate", false)
	segNoPol := uuid.New()
	publishDay(employmentNoPol, "2026-11-16", []segSpec{
		{id: segNoPol, sort: 0, startLoc: "08:00:00", endLoc: "17:00:00", startAt: "2026-11-16T08:00:00+07:00", endAt: "2026-11-16T17:00:00+07:00"},
	})
	insertTimeEntry(employmentNoPol, segNoPol, "2026-11-16T08:00:00+07:00", "2026-11-16T17:00:00+07:00")

	if _, err := repo.ReprocessAttendance(ctx, "2026-11-16", "2026-11-16", &empNoPol); !errors.Is(err, api.ErrAttendancePolicyMissing) {
		t.Fatalf("expected ErrAttendancePolicyMissing, got %v", err)
	}
	var noPolRows int
	if err := pool.QueryRow(ctx, `SELECT count(*) FROM attendance_daily_summaries WHERE employment_id = $1`, employmentNoPol).Scan(&noPolRows); err != nil {
		t.Fatalf("count no-policy summaries: %v", err)
	}
	if noPolRows != 0 {
		t.Fatalf("expected 0 summaries after missing-policy rollback, got %d", noPolRows)
	}
}
