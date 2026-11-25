//go:build integration

package repository

import (
	"testing"

	"github.com/google/uuid"
	"github.com/shopspring/decimal"
	"github.com/vkaylee/leeattend/apiserver/internal/api"
)

func TestIntegrationZeroCompensationAndPartialLeave(t *testing.T) {
	pool, ctx := requireIntegrationDatabase(t)

	employerID := uuid.New()
	employeeID := uuid.New()
	employmentID := uuid.New()
	actorID := uuid.New()
	templateID := uuid.New()
	versionID := uuid.New()
	batchID := uuid.New()
	leaveTypeID := uuid.New()

	mustExec := func(query string, args ...any) {
		t.Helper()
		if _, err := pool.Exec(ctx, query, args...); err != nil {
			t.Fatalf("exec %q: %v", query, err)
		}
	}

	mustExec(`INSERT INTO employers (id, name) VALUES ($1, 'h5-employer')`, employerID)
	seedIntegrationEmployeeGraphWithRange(t, ctx, pool, employeeID, employerID, employmentID, "H5-"+employeeID.String()[:8], "H5 Worker", "2026-01-01", "2026-01-01", "2026-11-30")
	mustExec(`INSERT INTO users (id, email, password_hash) VALUES ($1, $2, 'integration-test-hash')`, actorID, actorID.String()+"@invalid.test")
	mustExec(`INSERT INTO shift_templates (id, name) VALUES ($1, 'h5-shift')`, templateID)
	mustExec(`INSERT INTO shift_versions (id, template_id, version_number, name, effective_from, change_type, created_by) VALUES ($1, $2, 1, 'h5-v1', DATE '2026-01-01', 'create', $3)`, versionID, templateID, actorID)
	mustExec(`INSERT INTO leave_types (id, code, name, is_paid, policy_config) VALUES ($1, $2, 'H5 Annual Leave', true, '{}'::jsonb)`, leaveTypeID, "H5_"+leaveTypeID.String()[:6])

	// Part A: WorkSegment explicit 0 compensation_rate vs omitted default (1).
	segRepo := WorkSegmentRepository{DB: pool}
	zero := decimal.Zero
	zeroSeg, err := segRepo.CreateSegment(ctx, api.CreateWorkSegmentRequest{
		EmploymentID:     employmentID,
		Date:             "2026-11-18",
		TemplateID:       templateID,
		SortOrder:        0,
		ScheduledStart:   "08:00:00",
		ScheduledEnd:     "12:00:00",
		SegmentType:      "on_call",
		CompensationRate: &zero,
	})
	if err != nil {
		t.Fatalf("create explicit zero compensation segment: %v", err)
	}
	if !zeroSeg.CompensationRate.Equal(decimal.Zero) {
		t.Fatalf("explicit zero segment compensation_rate = %s, want 0", zeroSeg.CompensationRate.String())
	}

	defaultSeg, err := segRepo.CreateSegment(ctx, api.CreateWorkSegmentRequest{
		EmploymentID:   employmentID,
		Date:           "2026-11-19",
		TemplateID:     templateID,
		SortOrder:      0,
		ScheduledStart: "08:00:00",
		ScheduledEnd:   "17:00:00",
	})
	if err != nil {
		t.Fatalf("create omitted compensation segment: %v", err)
	}
	if !defaultSeg.CompensationRate.Equal(decimal.NewFromInt(1)) {
		t.Fatalf("omitted compensation_rate = %s, want 1", defaultSeg.CompensationRate.String())
	}

	// Part B: Official schedule revisions + holidays + leave intersection.
	mustExec(`INSERT INTO schedule_publication_batches (id, idempotency_key, request_hash, mode, status, requested_by, operation, response_snapshot, reason, timezone, range_start, range_end) VALUES ($1, $2, repeat('a', 64), 'rotating', 'published', $3, 'publish', '{}'::jsonb, 'h5 schedule', 'UTC', DATE '2026-11-22', DATE '2026-11-27')`, batchID, batchID.String(), actorID)

	publishWorkDay := func(dateStr, startLocal, endLocal, startAt, endAt, breaksJSON string) {
		t.Helper()
		revID := uuid.New()
		tx, err := pool.Begin(ctx)
		if err != nil {
			t.Fatalf("begin publish tx: %v", err)
		}
		if _, err := tx.Exec(ctx, `SELECT set_config('schedule.lifecycle_publication', 'on', true)`); err != nil {
			_ = tx.Rollback(ctx)
			t.Fatalf("set publication context: %v", err)
		}
		if _, err := tx.Exec(ctx, `INSERT INTO schedule_day_revisions (id, employment_id, employer_id, business_date, timezone, revision_number, state, publication_batch_id, source, source_snapshot, resolver_version, reason, published_by, published_actor_type, published_at) VALUES ($1, $2, $3, $4::date, 'UTC', 1, 'draft', $5, 'pattern', '{}'::jsonb, 'schedule-resolver-v1', 'h5', $6, 'human', NULL)`, revID, employmentID, employerID, dateStr, batchID, actorID); err != nil {
			_ = tx.Rollback(ctx)
			t.Fatalf("insert draft rev %s: %v", dateStr, err)
		}
		if _, err := tx.Exec(ctx, `INSERT INTO work_segments (id, employment_id, date, sort_order, scheduled_start, scheduled_end, segment_type, segment_status, compensation_rate, gap_compensation_before, gap_compensation_rate_before, source, source_detail, generation_metadata, template_id, schedule_revision_id, selected_shift_version_id, scheduled_start_at, scheduled_end_at, breaks_snapshot) VALUES ($1, $2, $3::date, 0, $4::time, $5::time, 'work', 'scheduled', 1, 'unpaid', 0, 'pattern', '{}'::jsonb, '{}'::jsonb, $6, $7, $8, $9::timestamptz, $10::timestamptz, $11::jsonb)`, uuid.New(), employmentID, dateStr, startLocal, endLocal, templateID, revID, versionID, startAt, endAt, breaksJSON); err != nil {
			_ = tx.Rollback(ctx)
			t.Fatalf("insert official segment %s: %v", dateStr, err)
		}
		if _, err := tx.Exec(ctx, `UPDATE schedule_day_revisions SET state = 'published', published_at = CURRENT_TIMESTAMP WHERE id = $1`, revID); err != nil {
			_ = tx.Rollback(ctx)
			t.Fatalf("publish rev %s: %v", dateStr, err)
		}
		if err := tx.Commit(ctx); err != nil {
			t.Fatalf("commit rev %s: %v", dateStr, err)
		}
	}

	publishOffDay := func(dateStr string) {
		t.Helper()
		mustExec(`INSERT INTO schedule_day_revisions (id, employment_id, employer_id, business_date, timezone, revision_number, state, publication_batch_id, source, source_snapshot, resolver_version, reason, published_by, published_actor_type, published_at) VALUES ($1, $2, $3, $4::date, 'UTC', 1, 'explicit_off', $5, 'pattern', '{}'::jsonb, 'schedule-resolver-v1', 'h5 off', $6, 'human', CURRENT_TIMESTAMP)`, uuid.New(), employmentID, employerID, dateStr, batchID, actorID)
	}

	// 2026-11-22 is Sunday (working day with 2 breaks), 2026-11-23 is Monday (explicit_off).
	publishWorkDay("2026-11-22", "08:00", "17:00", "2026-11-22T08:00:00Z", "2026-11-22T17:00:00Z", `[{"start_at":"2026-11-22T10:00:00Z","end_at":"2026-11-22T10:15:00Z"},{"start_at":"2026-11-22T12:00:00Z","end_at":"2026-11-22T13:00:00Z"}]`)
	publishOffDay("2026-11-23")
	// 2026-11-24 Tuesday overnight shift 22:00-06:00 with 02:00-03:00 break (420m net).
	publishWorkDay("2026-11-24", "22:00", "06:00", "2026-11-24T22:00:00Z", "2026-11-25T06:00:00Z", `[{"start_at":"2026-11-25T02:00:00Z","end_at":"2026-11-25T03:00:00Z"}]`)
	// 2026-11-25 Wednesday national holiday (even with published shift, holiday must not deduct leave).
	publishWorkDay("2026-11-25", "08:00", "17:00", "2026-11-25T08:00:00Z", "2026-11-25T17:00:00Z", `[{"start_at":"2026-11-25T12:00:00Z","end_at":"2026-11-25T13:00:00Z"}]`)
	holidayID := uuid.New()
	mustExec(`INSERT INTO holidays (id, name, date, holiday_type, is_paid) VALUES ($1, 'H5 National Holiday', DATE '2026-11-25', 'national', true)`, holidayID)
	t.Cleanup(func() { _, _ = pool.Exec(ctx, `DELETE FROM holidays WHERE id = $1`, holidayID) })
	// 2026-11-26 Thursday 08:00-17:00 with 12:00-13:00 break (480m net).
	publishWorkDay("2026-11-26", "08:00", "17:00", "2026-11-26T08:00:00Z", "2026-11-26T17:00:00Z", `[{"start_at":"2026-11-26T12:00:00Z","end_at":"2026-11-26T13:00:00Z"}]`)
	// 2026-11-27 Friday 08:00-17:00 (half-day leave).
	publishWorkDay("2026-11-27", "08:00", "17:00", "2026-11-27T08:00:00Z", "2026-11-27T17:00:00Z", `[{"start_at":"2026-11-27T12:00:00Z","end_at":"2026-11-27T13:00:00Z"}]`)

	reqSunMon := uuid.New()
	reqOvernight := uuid.New()
	reqHoliday := uuid.New()
	reqCustom := uuid.New()
	reqHalf := uuid.New()

	mustExec(`INSERT INTO leave_requests (id, employment_id, leave_type_id, start_date, end_date, status, duration_type, requested_by_user_id, approved_by_user_id) VALUES ($1, $2, $3, DATE '2026-11-22', DATE '2026-11-23', 'approved', 'full_day', $4, $4)`, reqSunMon, employmentID, leaveTypeID, actorID)
	mustExec(`INSERT INTO leave_requests (id, employment_id, leave_type_id, start_date, end_date, status, duration_type, start_time, end_time, requested_by_user_id, approved_by_user_id) VALUES ($1, $2, $3, DATE '2026-11-24', DATE '2026-11-24', 'approved', 'custom_hours', TIME '01:00', TIME '04:00', $4, $4)`, reqOvernight, employmentID, leaveTypeID, actorID)
	mustExec(`INSERT INTO leave_requests (id, employment_id, leave_type_id, start_date, end_date, status, duration_type, requested_by_user_id, approved_by_user_id) VALUES ($1, $2, $3, DATE '2026-11-25', DATE '2026-11-25', 'approved', 'full_day', $4, $4)`, reqHoliday, employmentID, leaveTypeID, actorID)
	mustExec(`INSERT INTO leave_requests (id, employment_id, leave_type_id, start_date, end_date, status, duration_type, start_time, end_time, requested_by_user_id, approved_by_user_id) VALUES ($1, $2, $3, DATE '2026-11-26', DATE '2026-11-26', 'approved', 'custom_hours', TIME '11:00', TIME '14:00', $4, $4)`, reqCustom, employmentID, leaveTypeID, actorID)
	mustExec(`INSERT INTO leave_requests (id, employment_id, leave_type_id, start_date, end_date, status, duration_type, requested_by_user_id, approved_by_user_id) VALUES ($1, $2, $3, DATE '2026-11-27', DATE '2026-11-27', 'approved', 'first_half', $4, $4)`, reqHalf, employmentID, leaveTypeID, actorID)

	leaveRepo := LeaveRepository{DB: pool}
	history, err := leaveRepo.GetEmployeeLeaveHistory(ctx, employeeID, 2026, &leaveTypeID)
	if err != nil {
		t.Fatalf("get employee leave history: %v", err)
	}
	byRef := make(map[uuid.UUID]decimal.Decimal, len(history))
	for _, item := range history {
		if item.ReferenceID != nil {
			byRef[*item.ReferenceID] = item.Amount
		}
	}

	assertAmount := func(name string, id uuid.UUID, want string) {
		t.Helper()
		got, ok := byRef[id]
		if !ok {
			t.Fatalf("%s: missing history item for request %s", name, id)
		}
		wantDec := decimal.RequireFromString(want)
		if !got.Equal(wantDec) {
			t.Fatalf("%s: got amount %s, want %s", name, got.String(), wantDec.String())
		}
	}

	assertAmount("sunday-work + monday-off", reqSunMon, "-1")
	assertAmount("holiday-day", reqHoliday, "0")
	assertAmount("half-day", reqHalf, "-0.5")
	assertAmount("custom-hours-across-break", reqCustom, "-0.25")
	assertAmount("overnight-custom-hours-across-break", reqOvernight, "-0.2857")

	orgBalances, _, err := leaveRepo.GetOrganizationLeaveBalances(ctx, 2026, 1, 50, "H5 Worker")
	if err != nil {
		t.Fatalf("get organization leave balances: %v", err)
	}
	foundBalance := false
	for _, b := range orgBalances {
		if b.EmployeeID == employeeID && b.LeaveTypeID == leaveTypeID {
			foundBalance = true
			// reqSunMon(1) + reqOvernight(1 in day-level SQL) + reqHoliday(0) + reqCustom(1 in day-level SQL) + reqHalf(0.5) => 3.5 -> 4
			if b.Used == 0 {
				t.Fatalf("expected non-zero organization leave balance used for Sunday/weekday schedule, got 0")
			}
		}
	}
	if !foundBalance {
		t.Fatalf("expected H5 Worker balance in GetOrganizationLeaveBalances")
	}
}
