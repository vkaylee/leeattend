//go:build integration

package repository

import (
	"encoding/json"
	"testing"

	"github.com/google/uuid"
	"github.com/vkaylee/leeattend/apiserver/internal/api"
)

func TestIntegrationSplitShiftAttendanceConsumers(t *testing.T) {
	pool, ctx := requireIntegrationDatabase(t)

	employerID := uuid.New()
	actorID := uuid.New()
	templateSplit, verSplit := uuid.New(), uuid.New()
	templateNight, verNight := uuid.New(), uuid.New()
	templateEarlyTZ, verEarlyTZ := uuid.New(), uuid.New()
	batchID := uuid.New()
	leaveTypeID := uuid.New()

	mustExec := func(query string, args ...any) {
		t.Helper()
		if _, err := pool.Exec(ctx, query, args...); err != nil {
			t.Fatalf("exec %q: %v", query, err)
		}
	}

	mustExec(`INSERT INTO employers (id, name) VALUES ($1, 'h3-employer')`, employerID)
	mustExec(`INSERT INTO users (id, email, password_hash) VALUES ($1, $2, 'integration-test-hash')`, actorID, actorID.String()+"@invalid.test")
	mustExec(`INSERT INTO leave_types (id, code, name, is_paid, policy_config) VALUES ($1, $2, 'H3 Leave', true, '{}'::jsonb)`, leaveTypeID, "H3_"+leaveTypeID.String()[:6])

	createShift := func(tplID, verID uuid.UUID, name string) {
		t.Helper()
		mustExec(`INSERT INTO shift_templates (id, name) VALUES ($1, $2)`, tplID, name)
		mustExec(`INSERT INTO shift_versions (id, template_id, version_number, name, effective_from, change_type, created_by) VALUES ($1, $2, 1, $3, DATE '2026-01-01', 'create', $4)`, verID, tplID, name+" v1", actorID)
	}
	createShift(templateSplit, verSplit, "Split Shift")
	createShift(templateNight, verNight, "Night Shift")
	createShift(templateEarlyTZ, verEarlyTZ, "HCM Early Shift")

	mustExec(`INSERT INTO schedule_publication_batches (id, idempotency_key, request_hash, mode, status, requested_by, operation, response_snapshot, reason, timezone, range_start, range_end) VALUES ($1, $2, repeat('d', 64), 'rotating', 'published', $3, 'publish', '{}'::jsonb, 'h3 fixture', 'UTC', DATE '2026-11-20', DATE '2026-11-20')`, batchID, batchID.String(), actorID)

	createWorker := func(name string) (uuid.UUID, uuid.UUID) {
		t.Helper()
		empID, employmentID := uuid.New(), uuid.New()
		mustExec(`INSERT INTO employees (id, employee_id_number, full_name, hire_date, original_hire_date) VALUES ($1, $2, $3, DATE '2026-01-01', DATE '2026-01-01')`, empID, "H3-"+empID.String()[:8], name)
		mustExec(`INSERT INTO employments (id, employee_id, employer_id, status, start_date, end_date) VALUES ($1, $2, $3, 'active', DATE '2026-01-01', DATE '2026-11-30')`, employmentID, empID, employerID)
		return empID, employmentID
	}

	type segSpec struct {
		id       uuid.UUID
		tplID    uuid.UUID
		verID    uuid.UUID
		sort     int
		startLoc string
		endLoc   string
		startAt  string
		endAt    string
		status   string
	}
	publishDay := func(employmentID uuid.UUID, tz string, segs []segSpec) uuid.UUID {
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
		if _, err := tx.Exec(ctx, `INSERT INTO schedule_day_revisions (id, employment_id, employer_id, business_date, timezone, revision_number, state, publication_batch_id, source, source_snapshot, resolver_version, reason, published_by, published_actor_type, published_at) VALUES ($1, $2, $3, DATE '2026-11-20', $4, 1, 'draft', $5, 'manual', '{}'::jsonb, 'schedule-resolver-v1', 'h3', $6, 'human', NULL)`, revID, employmentID, employerID, tz, batchID, actorID); err != nil {
			_ = tx.Rollback(ctx)
			t.Fatalf("insert rev: %v", err)
		}
		for _, s := range segs {
			st := s.status
			if st == "" {
				st = "scheduled"
			}
			if _, err := tx.Exec(ctx, `INSERT INTO work_segments (id, employment_id, date, sort_order, scheduled_start, scheduled_end, segment_type, segment_status, compensation_rate, gap_compensation_before, gap_compensation_rate_before, source, source_detail, generation_metadata, template_id, schedule_revision_id, selected_shift_version_id, scheduled_start_at, scheduled_end_at, breaks_snapshot) VALUES ($1, $2, DATE '2026-11-20', $3, $4::time, $5::time, 'work', $6, 1, 'unpaid', 0, 'manual', '{}'::jsonb, '{}'::jsonb, $7, $8, $9, $10::timestamptz, $11::timestamptz, '[]'::jsonb)`, s.id, employmentID, s.sort, s.startLoc, s.endLoc, st, s.tplID, revID, s.verID, s.startAt, s.endAt); err != nil {
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
		return revID
	}

	// Worker 1: 2 segments x 2 sessions (grain-safe daily report + not in no-show).
	empFull, empFullID := createWorker("H3 Full Split")
	segFull1, segFull2 := uuid.New(), uuid.New()
	publishDay(empFullID, "UTC", []segSpec{
		{id: segFull1, tplID: templateSplit, verID: verSplit, sort: 0, startLoc: "08:00", endLoc: "12:00", startAt: "2026-11-20T08:00:00Z", endAt: "2026-11-20T12:00:00Z"},
		{id: segFull2, tplID: templateSplit, verID: verSplit, sort: 1, startLoc: "14:00", endLoc: "18:00", startAt: "2026-11-20T14:00:00Z", endAt: "2026-11-20T18:00:00Z"},
	})
	mustExec(`INSERT INTO time_entries (id, employment_id, segment_id, template_id, punch_in_time, punch_out_time, duration_minutes, entry_type) VALUES (gen_random_uuid(), $1, $2, $4, '2026-11-20T08:00:00Z', '2026-11-20T12:00:00Z', 240, 'work'), (gen_random_uuid(), $1, $3, $4, '2026-11-20T14:00:00Z', '2026-11-20T18:00:00Z', 240, 'work')`, empFullID, segFull1, segFull2, templateSplit)
	mustExec(`INSERT INTO attendance_daily_summaries (id, employment_id, date, actual_work_minutes, standard_minutes, approved_ot_minutes, ot_status) VALUES (gen_random_uuid(), $1, DATE '2026-11-20', 480, 480, 0, 'none')`, empFullID)

	// Worker 2: Split shift, attended morning ONLY, missed afternoon (must appear in no-show with 14:00-18:00!).
	empHalfMiss, empHalfMissID := createWorker("H3 Miss Afternoon")
	segHalf1, segHalf2 := uuid.New(), uuid.New()
	publishDay(empHalfMissID, "UTC", []segSpec{
		{id: segHalf1, tplID: templateSplit, verID: verSplit, sort: 0, startLoc: "08:00", endLoc: "12:00", startAt: "2026-11-20T08:00:00Z", endAt: "2026-11-20T12:00:00Z"},
		{id: segHalf2, tplID: templateSplit, verID: verSplit, sort: 1, startLoc: "14:00", endLoc: "18:00", startAt: "2026-11-20T14:00:00Z", endAt: "2026-11-20T18:00:00Z"},
	})
	mustExec(`INSERT INTO time_entries (id, employment_id, segment_id, template_id, punch_in_time, punch_out_time, duration_minutes, entry_type) VALUES (gen_random_uuid(), $1, $2, $3, '2026-11-20T08:00:00Z', '2026-11-20T12:00:00Z', 240, 'work')`, empHalfMissID, segHalf1, templateSplit)

	// Worker 3: Early shift in Asia/Ho_Chi_Minh (05:00-13:00+07 = 2026-11-19T22:00Z..2026-11-20T06:00Z).
	// Unlinked punch at 2026-11-19T22:05:00Z (previous UTC date!) must match business date 2026-11-20 in BOTH daily and no-show!
	empTZ, empTZID := createWorker("H3 HCM Early")
	segTZ := uuid.New()
	publishDay(empTZID, "Asia/Ho_Chi_Minh", []segSpec{
		{id: segTZ, tplID: templateEarlyTZ, verID: verEarlyTZ, sort: 0, startLoc: "05:00", endLoc: "13:00", startAt: "2026-11-19T22:00:00Z", endAt: "2026-11-20T06:00:00Z"},
	})
	mustExec(`INSERT INTO time_entries (id, employment_id, segment_id, template_id, punch_in_time, punch_out_time, duration_minutes, entry_type) VALUES (gen_random_uuid(), $1, NULL, $2, '2026-11-19T22:05:00Z', '2026-11-20T06:00:00Z', 475, 'work')`, empTZID, templateEarlyTZ)
	mustExec(`INSERT INTO attendance_daily_summaries (id, employment_id, date, actual_work_minutes, standard_minutes, approved_ot_minutes, ot_status) VALUES (gen_random_uuid(), $1, DATE '2026-11-20', 475, 475, 0, 'none')`, empTZID)

	// Worker 4: Approved full-day leave on 2026-11-20 (must NOT be flagged no-show!).
	empLeave, empLeaveID := createWorker("H3 Approved Leave")
	publishDay(empLeaveID, "UTC", []segSpec{
		{id: uuid.New(), tplID: templateSplit, verID: verSplit, sort: 0, startLoc: "08:00", endLoc: "17:00", startAt: "2026-11-20T08:00:00Z", endAt: "2026-11-20T17:00:00Z"},
	})
	mustExec(`INSERT INTO leave_requests (id, employment_id, leave_type_id, start_date, end_date, status, duration_type, requested_by_user_id, approved_by_user_id) VALUES (gen_random_uuid(), $1, $2, DATE '2026-11-20', DATE '2026-11-20', 'approved', 'full_day', $3, $3)`, empLeaveID, leaveTypeID, actorID)

	// Worker 5: Cancelled segment (must NOT be flagged no-show!).
	empCancelled, empCancelledID := createWorker("H3 Cancelled Seg")
	publishDay(empCancelledID, "UTC", []segSpec{
		{id: uuid.New(), tplID: templateNight, verID: verNight, sort: 0, startLoc: "22:00", endLoc: "06:00", startAt: "2026-11-20T22:00:00Z", endAt: "2026-11-21T06:00:00Z", status: "cancelled"},
	})

	repo := ReportRepository{DB: pool}

	// Verify daily report: grain-safe 2 seg x 2 sessions AND timezone-aware punch matching.
	dailyResp, err := repo.GetAttendanceReport(ctx, api.AttendanceReportFilter{
		ReportType:  "daily",
		StartDate:   "2026-11-20",
		EndDate:     "2026-11-20",
		EmployeeIDs: []uuid.UUID{empFull, empTZ},
		Page:        1,
		Limit:       10,
	})
	if err != nil {
		t.Fatalf("daily report: %v", err)
	}
	if dailyResp.Total != 2 {
		t.Fatalf("daily report total = %d, want 2 (no grain multiplication)", dailyResp.Total)
	}
	var dailyRows []api.DailyAttendanceRow
	if err := json.Unmarshal(dailyResp.Data, &dailyRows); err != nil {
		t.Fatalf("unmarshal daily rows: %v", err)
	}
	if len(dailyRows) != 2 {
		t.Fatalf("daily rows len = %d, want 2", len(dailyRows))
	}
	dailyByEmp := map[uuid.UUID]api.DailyAttendanceRow{}
	for _, r := range dailyRows {
		dailyByEmp[r.EmployeeID] = r
	}
	if dailyByEmp[empFull].ActualWorkMinutes != 480 {
		t.Fatalf("2 seg x 2 sessions actual_work_minutes = %d, want 480", dailyByEmp[empFull].ActualWorkMinutes)
	}
	if dailyByEmp[empTZ].ActualPunchIn == nil || dailyByEmp[empTZ].ActualPunchOut == nil {
		t.Fatalf("Asia/Ho_Chi_Minh early shift punch was missed due to UTC ::date mismatch: %+v", dailyByEmp[empTZ])
	}

	// Verify no-show report across all 5 workers: ONLY Worker 2's afternoon segment (14:00:00-18:00:00) should appear!
	noShowResp, err := repo.GetNoShowReport(ctx, api.NoShowReportFilter{
		Date:        "2026-11-20",
		EmployeeIDs: []uuid.UUID{empFull, empHalfMiss, empTZ, empLeave, empCancelled},
		Page:        1,
		Limit:       20,
	})
	if err != nil {
		t.Fatalf("no-show report: %v", err)
	}
	var noShowRows []api.NoShowReportRow
	if err := json.Unmarshal(noShowResp.Data, &noShowRows); err != nil {
		t.Fatalf("unmarshal no-show rows: %v", err)
	}
	if noShowResp.Total != 1 || len(noShowRows) != 1 {
		t.Fatalf("no-show total=%d len=%d rows=%+v; want ONLY 1 row (Worker 2 afternoon segment)", noShowResp.Total, len(noShowRows), noShowRows)
	}
	if noShowRows[0].EmployeeID != empHalfMiss || noShowRows[0].ScheduledStart == nil || *noShowRows[0].ScheduledStart != "14:00:00" || noShowRows[0].ScheduledEnd == nil || *noShowRows[0].ScheduledEnd != "18:00:00" {
		t.Fatalf("unexpected no-show row: %+v; want Worker 2 afternoon 14:00:00-18:00:00", noShowRows[0])
	}
}
