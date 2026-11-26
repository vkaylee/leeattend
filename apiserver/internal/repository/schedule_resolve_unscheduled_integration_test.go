//go:build integration

package repository

import (
	"errors"
	"sync"
	"testing"
	"time"

	"github.com/google/uuid"
	api "github.com/vkaylee/leeattend/apiserver/internal/api"
)

func TestIntegrationResolveUnscheduledPublishesPairsAndReplaysAtomically(t *testing.T) {
	pool, ctx := requireIntegrationDatabase(t)
	employerID, employeeID, employmentID, actorID := uuid.New(), uuid.New(), uuid.New(), uuid.New()
	templateID, versionID := uuid.New(), uuid.New()
	mustExec := func(query string, args ...any) {
		t.Helper()
		if _, err := pool.Exec(ctx, query, args...); err != nil {
			t.Fatalf("fixture exec failed: %v", err)
		}
	}
	mustExec(`INSERT INTO employers (id, name) VALUES ($1, 'resolve-unscheduled-employer')`, employerID)
	seedIntegrationEmployeeGraphWithRange(t, ctx, pool, employeeID, employerID, employmentID, employeeID.String(), "resolve-unscheduled-employee", "2026-01-01", "2026-01-01", "")
	mustExec(`INSERT INTO users (id, email, password_hash) VALUES ($1, $2, 'integration-test-hash')`, actorID, actorID.String()+"@invalid.test")
	patternID, otPolicyID, otVersionID, groupID, policyVersionID := uuid.New(), uuid.New(), uuid.New(), uuid.New(), uuid.New()
	var relationshipID uuid.UUID
	if err := pool.QueryRow(ctx, `SELECT relationship_id FROM employments WHERE id = $1`, employmentID).Scan(&relationshipID); err != nil {
		t.Fatalf("load relationship: %v", err)
	}
	mustExec(`INSERT INTO working_patterns (id, name, pattern_type, is_active) VALUES ($1, 'resolve policy pattern', 'weekly', true)`, patternID)
	mustExec(`INSERT INTO overtime_policy_definitions (id, name) VALUES ($1, 'resolve policy OT')`, otPolicyID)
	mustExec(`INSERT INTO overtime_policy_versions (id, policy_id, version_no, status, rules, created_by, published_by, published_at) VALUES ($1, $2, 1, 'published', '[{"priority":10,"day_type":"any","days_of_week":127,"multiplier":"1.5"}]'::jsonb, $3, $3, NOW())`, otVersionID, otPolicyID, actorID)
	mustExec(`INSERT INTO work_policy_groups (id, code, name, created_by) VALUES ($1, $2, 'resolve policy group', $3)`, groupID, "RES-"+groupID.String()[:8], actorID)
	mustExec(`INSERT INTO work_policy_versions (id, group_id, version_no, status, effective_from, created_by, published_by, published_at, minutes_per_credit, work_unit_minutes, rounding_policy, early_arrival_strategy, early_ot_enabled, early_ot_qualification_grace_minutes, early_ot_max_minutes, break_auto_deduct, break_is_overtime, ot_aggregation_mode, working_pattern_id, pay_method, monthly_denominator, monthly_standard_days, overtime_policy_version_id) VALUES ($1, $2, 1, 'published', DATE '2026-01-01', $3, $3, NOW(), 480, 30, 'floor', 'block', false, 0, 60, true, false, 'aggregate', $4, 'monthly', 'fixed', 22, $5)`, policyVersionID, groupID, actorID, patternID, otVersionID)
	mustExec(`INSERT INTO work_policy_memberships (id, employment_relationship_id, group_id, effective_from, effective_to, created_by) VALUES (gen_random_uuid(), $1, $2, DATE '2026-01-01', DATE '2027-01-01', $3)`, relationshipID, groupID, actorID)
	mustExec(`INSERT INTO shift_templates (id, name) VALUES ($1, 'resolve-unscheduled-shift')`, templateID)
	mustExec(`INSERT INTO shift_versions (id, template_id, version_number, name, effective_from, change_type, created_by) VALUES ($1, $2, 1, 'v1', DATE '2026-01-01', 'create', $3)`, versionID, templateID, actorID)
	mustExec(`INSERT INTO shift_default_intervals (id, version_id, sort_order, start_time, end_time) VALUES (gen_random_uuid(), $1, 0, '08:00', '17:00')`, versionID)

	// Unscheduled raw punches exist before any schedule revision.
	mustExec(`INSERT INTO raw_attendance_logs (id, employment_id, punch_time, source, raw_data, punch, status, created_at) VALUES (gen_random_uuid(), $1, TIMESTAMPTZ '2026-11-22 08:05:00+00', 'import', '{}'::jsonb, 0, 1, NOW()), (gen_random_uuid(), $1, TIMESTAMPTZ '2026-11-22 17:00:00+00', 'import', '{}'::jsonb, 1, 1, NOW())`, employmentID)

	repo := ScheduleRepository{DB: pool}
	req := api.ResolveUnscheduledScheduleRequest{ShiftID: templateID, Reason: "Manager confirmed unscheduled shift", Timezone: "UTC"}
	key := "resolve-unscheduled-" + uuid.NewString()

	res, err := repo.ResolveUnscheduled(ctx, actorID, employmentID, "2026-11-22", key, req)
	if err != nil {
		t.Fatalf("resolve unscheduled failed: %v", err)
	}
	if res.BatchID == uuid.Nil || res.RevisionID == uuid.Nil || res.Recalculated != 1 {
		t.Fatalf("unexpected resolve response: %+v", res)
	}

	// Verify official revision, work segment, audit log, paired time entry, and daily summary all persisted cleanly without legacy shift_rosters.
	var revState string
	if err := pool.QueryRow(ctx, `SELECT state FROM schedule_day_revisions WHERE id = $1`, res.RevisionID).Scan(&revState); err != nil || revState != "published" {
		t.Fatalf("revision state=%q err=%v, want published", revState, err)
	}
	var rosterCount, summaryMinutes int
	if err := pool.QueryRow(ctx, `SELECT count(*) FROM shift_rosters WHERE employment_id = $1 AND date = DATE '2026-11-22'`, employmentID).Scan(&rosterCount); err != nil || rosterCount != 0 {
		t.Fatalf("resolve-unscheduled must not write legacy shift_rosters: count=%d err=%v", rosterCount, err)
	}
	if err := pool.QueryRow(ctx, `SELECT actual_work_minutes FROM attendance_daily_summaries WHERE employment_id = $1 AND date = DATE '2026-11-22'`, employmentID).Scan(&summaryMinutes); err != nil || summaryMinutes <= 0 {
		t.Fatalf("daily summary not populated: minutes=%d err=%v", summaryMinutes, err)
	}

	// Idempotent replay with same key & payload returns exact original response.
	replay, err := repo.ResolveUnscheduled(ctx, actorID, employmentID, "2026-11-22", key, req)
	if err != nil || replay.BatchID != res.BatchID || replay.RevisionID != res.RevisionID {
		t.Fatalf("idempotent replay mismatch: got=%+v want=%+v err=%v", replay, res, err)
	}

	// Same key with different payload -> conflict.
	conflictReq := req
	conflictReq.Reason = "Different reason"
	if _, err := repo.ResolveUnscheduled(ctx, actorID, employmentID, "2026-11-22", key, conflictReq); !errors.Is(err, ErrScheduleIdempotencyConflict) {
		t.Fatalf("idempotency conflict err=%v, want ErrScheduleIdempotencyConflict", err)
	}

	// New key on already-published day -> ErrScheduleAlreadyPublished (must use replan).
	if _, err := repo.ResolveUnscheduled(ctx, actorID, employmentID, "2026-11-22", "resolve-unscheduled-"+uuid.NewString(), req); !errors.Is(err, ErrScheduleAlreadyPublished) {
		t.Fatalf("already published err=%v, want ErrScheduleAlreadyPublished", err)
	}

	// Closed attendance period blocks resolution with zero writes.
	mustExec(`INSERT INTO attendance_periods (id, employer_id, period_year, period_month, status, version) VALUES ($1, $2, 2026, 10, 'closed', 1)`, uuid.New(), employerID)
	if _, err := repo.ResolveUnscheduled(ctx, actorID, employmentID, "2026-10-15", "resolve-closed-"+uuid.NewString(), req); !errors.Is(err, ErrScheduleLockedPeriod) {
		t.Fatalf("closed period err=%v, want ErrScheduleLockedPeriod", err)
	}

	// Pre-existing unlinked time_entry (from ESS/PunchIn on an unscheduled day) gets reconciled to the newly published segment,
	// while a second employment of the same employee in a closed period is completely untouched.
	otherEmployerID, otherEmploymentID := uuid.New(), uuid.New()
	otherRelID := uuid.New()
	var tenureID uuid.UUID
	if err := pool.QueryRow(ctx, `SELECT tenure_id FROM employment_relationships WHERE id = $1`, relationshipID).Scan(&tenureID); err != nil {
		t.Fatalf("load tenure: %v", err)
	}
	mustExec(`INSERT INTO employers (id, name) VALUES ($1, 'resolve-other-employer')`, otherEmployerID)
	mustExec(`INSERT INTO employment_relationships (id, employee_id, employer_id, tenure_id, status, start_date) VALUES ($1, $2, $3, $4, 'active', DATE '2026-01-01')`, otherRelID, employeeID, otherEmployerID, tenureID)
	mustExec(`INSERT INTO employments (id, employee_id, employer_id, relationship_id, status, start_date) VALUES ($1, $2, $3, $4, 'active', DATE '2026-01-01')`, otherEmploymentID, employeeID, otherEmployerID, otherRelID)
	mustExec(`INSERT INTO attendance_periods (id, employer_id, period_year, period_month, status, version) VALUES ($1, $2, 2026, 11, 'closed', 1)`, uuid.New(), otherEmployerID)
	mustExec(`INSERT INTO raw_attendance_logs (id, employment_id, punch_time, source, raw_data, punch, status, created_at) VALUES (gen_random_uuid(), $1, TIMESTAMPTZ '2026-11-23 08:00:00+00', 'import', '{}'::jsonb, 0, 1, NOW())`, otherEmploymentID)

	preEntryID := uuid.New()
	mustExec(`INSERT INTO time_entries (id, employment_id, punch_in_time, punch_out_time, duration_minutes, entry_type) VALUES ($1, $2, TIMESTAMPTZ '2026-11-23 08:15:00+00', TIMESTAMPTZ '2026-11-23 17:00:00+00', 525, 'work')`, preEntryID, employmentID)

	res2, err := repo.ResolveUnscheduled(ctx, actorID, employmentID, "2026-11-23", "resolve-preexisting-"+uuid.NewString(), req)
	if err != nil {
		t.Fatalf("resolve with pre-existing unlinked time_entry failed: %v", err)
	}
	var linkedSegID, linkedVerID *uuid.UUID
	if err := pool.QueryRow(ctx, `SELECT segment_id, shift_version_id FROM time_entries WHERE id = $1`, preEntryID).Scan(&linkedSegID, &linkedVerID); err != nil {
		t.Fatalf("read reconciled time_entry: %v", err)
	}
	if linkedSegID == nil || linkedVerID == nil || *linkedVerID != versionID {
		t.Fatalf("pre-existing time_entry was not reconciled to published segment/version: seg=%v ver=%v", linkedSegID, linkedVerID)
	}
	var lateMins int
	if err := pool.QueryRow(ctx, `SELECT late_minutes FROM attendance_daily_summaries WHERE employment_id = $1 AND date = DATE '2026-11-23'`, employmentID).Scan(&lateMins); err != nil || lateMins != 15 {
		t.Fatalf("expected late_minutes=15 after reconciling 08:15 entry to 08:00 shift, got %d (err=%v, rev=%s)", lateMins, err, res2.RevisionID)
	}
	var otherSummaries int
	if err := pool.QueryRow(ctx, `SELECT count(*) FROM attendance_daily_summaries WHERE employment_id = $1`, otherEmploymentID).Scan(&otherSummaries); err != nil || otherSummaries != 0 {
		t.Fatalf("other employment must remain untouched, summaries=%d err=%v", otherSummaries, err)
	}

	// Unlinked time_entry outside the chosen shift's window must fail closed with ErrScheduleConflict and roll back the revision.
	mustExec(`INSERT INTO time_entries (id, employment_id, punch_in_time, punch_out_time, duration_minutes, entry_type) VALUES (gen_random_uuid(), $1, TIMESTAMPTZ '2026-11-24 22:30:00+00', TIMESTAMPTZ '2026-11-24 23:30:00+00', 60, 'work')`, employmentID)
	if _, err := repo.ResolveUnscheduled(ctx, actorID, employmentID, "2026-11-24", "resolve-mismatch-"+uuid.NewString(), req); !errors.Is(err, api.ErrScheduleConflict) {
		t.Fatalf("mismatched unlinked entry err=%v, want ErrScheduleConflict", err)
	}
	var rolledBackRevs int
	if err := pool.QueryRow(ctx, `SELECT count(*) FROM schedule_day_revisions WHERE employment_id = $1 AND business_date = DATE '2026-11-24'`, employmentID).Scan(&rolledBackRevs); err != nil || rolledBackRevs != 0 {
		t.Fatalf("mismatched resolution must roll back revision, found %d (err=%v)", rolledBackRevs, err)
	}

	// Day with no unscheduled attendance evidence must be rejected (cannot bypass normal schedule publication).
	if _, err := repo.ResolveUnscheduled(ctx, actorID, employmentID, "2026-11-28", "resolve-no-evidence-"+uuid.NewString(), req); !errors.Is(err, api.ErrScheduleValidation) {
		t.Fatalf("day without unscheduled attendance err=%v, want ErrScheduleValidation", err)
	}

	// Overnight shift (22:00-06:00) on 2026-11-25 reconciles an unlinked entry punched in after midnight (00:15 on 2026-11-26).
	nightTplID, nightVerID := uuid.New(), uuid.New()
	mustExec(`INSERT INTO shift_templates (id, name) VALUES ($1, 'resolve-night-shift')`, nightTplID)
	mustExec(`INSERT INTO shift_versions (id, template_id, version_number, name, effective_from, change_type, created_by) VALUES ($1, $2, 1, 'night-v1', DATE '2026-01-01', 'create', $3)`, nightVerID, nightTplID, actorID)
	mustExec(`INSERT INTO shift_default_intervals (id, version_id, sort_order, start_time, end_time) VALUES (gen_random_uuid(), $1, 0, '22:00', '06:00')`, nightVerID)
	nightEntryID := uuid.New()
	mustExec(`INSERT INTO time_entries (id, employment_id, punch_in_time, punch_out_time, duration_minutes, entry_type) VALUES ($1, $2, TIMESTAMPTZ '2026-11-26 00:15:00+00', TIMESTAMPTZ '2026-11-26 06:00:00+00', 345, 'work')`, nightEntryID, employmentID)
	nightReq := api.ResolveUnscheduledScheduleRequest{ShiftID: nightTplID, Reason: "Overnight unscheduled shift", Timezone: "UTC"}
	if _, err := repo.ResolveUnscheduled(ctx, actorID, employmentID, "2026-11-25", "resolve-night-"+uuid.NewString(), nightReq); err != nil {
		t.Fatalf("overnight unscheduled resolve failed: %v", err)
	}
	var nightSegID, nightLinkedVer *uuid.UUID
	if err := pool.QueryRow(ctx, `SELECT segment_id, shift_version_id FROM time_entries WHERE id = $1`, nightEntryID).Scan(&nightSegID, &nightLinkedVer); err != nil || nightSegID == nil || nightLinkedVer == nil || *nightLinkedVer != nightVerID {
		t.Fatalf("overnight entry not linked to night revision: seg=%v ver=%v err=%v", nightSegID, nightLinkedVer, err)
	}

	// Cross-employer shift version must be rejected.
	otherShiftTpl, otherShiftVer := uuid.New(), uuid.New()
	mustExec(`INSERT INTO shift_templates (id, name) VALUES ($1, 'other-employer-shift')`, otherShiftTpl)
	mustExec(`INSERT INTO shift_versions (id, template_id, version_number, name, employer_id, effective_from, change_type, created_by) VALUES ($1, $2, 1, 'other-v1', $3, DATE '2026-01-01', 'create', $4)`, otherShiftVer, otherShiftTpl, otherEmployerID, actorID)
	mustExec(`INSERT INTO shift_default_intervals (id, version_id, sort_order, start_time, end_time) VALUES (gen_random_uuid(), $1, 0, '08:00', '17:00')`, otherShiftVer)
	mustExec(`INSERT INTO raw_attendance_logs (id, employment_id, punch_time, source, raw_data, punch, status, created_at) VALUES (gen_random_uuid(), $1, TIMESTAMPTZ '2026-11-27 08:00:00+00', 'import', '{}'::jsonb, 0, 1, NOW())`, employmentID)
	crossReq := api.ResolveUnscheduledScheduleRequest{ShiftID: otherShiftTpl, Reason: "Cross employer shift", Timezone: "UTC"}
	if _, err := repo.ResolveUnscheduled(ctx, actorID, employmentID, "2026-11-27", "resolve-cross-"+uuid.NewString(), crossReq); !errors.Is(err, api.ErrScheduleValidation) {
		t.Fatalf("cross-employer shift err=%v, want ErrScheduleValidation", err)
	}

	// Protected approved OT summary blocks overwrite with ErrScheduleConflict.
	mustExec(`INSERT INTO attendance_daily_summaries (id, employment_id, date, actual_work_minutes, standard_minutes, approved_ot_minutes, ot_status) VALUES (gen_random_uuid(), $1, DATE '2026-11-27', 540, 480, 60, 'approved')`, employmentID)
	if _, err := repo.ResolveUnscheduled(ctx, actorID, employmentID, "2026-11-27", "resolve-ot-"+uuid.NewString(), req); !errors.Is(err, api.ErrScheduleConflict) {
		t.Fatalf("approved OT summary err=%v, want ErrScheduleConflict", err)
	}
}

func TestIntegrationConcurrentResolveUnscheduledWithESSPunchAndReprocessHasNoDeadlockOrDuplicates(t *testing.T) {
	pool, ctx := requireIntegrationDatabase(t)
	employerID, employeeID, employmentID, actorID := uuid.New(), uuid.New(), uuid.New(), uuid.New()
	templateID, versionID := uuid.New(), uuid.New()
	mustExec := func(query string, args ...any) {
		t.Helper()
		if _, err := pool.Exec(ctx, query, args...); err != nil {
			t.Fatalf("fixture exec failed: %v", err)
		}
	}
	mustExec(`INSERT INTO employers (id, name) VALUES ($1, 'concurrent-resolve-employer')`, employerID)
	seedIntegrationEmployeeGraphWithRange(t, ctx, pool, employeeID, employerID, employmentID, employeeID.String(), "concurrent-resolve-employee", "2026-01-01", "2026-01-01", "")
	mustExec(`INSERT INTO users (id, email, password_hash) VALUES ($1, $2, 'integration-test-hash')`, actorID, actorID.String()+"@invalid.test")
	var relationshipID uuid.UUID
	if err := pool.QueryRow(ctx, `SELECT relationship_id FROM employments WHERE id = $1`, employmentID).Scan(&relationshipID); err != nil {
		t.Fatalf("load relationship: %v", err)
	}
	patternID, otPolicyID, otVersionID, groupID, policyVersionID := uuid.New(), uuid.New(), uuid.New(), uuid.New(), uuid.New()
	mustExec(`INSERT INTO working_patterns (id, name, pattern_type, is_active) VALUES ($1, 'conc pattern', 'weekly', true)`, patternID)
	mustExec(`INSERT INTO overtime_policy_definitions (id, name) VALUES ($1, 'conc OT')`, otPolicyID)
	mustExec(`INSERT INTO overtime_policy_versions (id, policy_id, version_no, status, rules, created_by, published_by, published_at) VALUES ($1, $2, 1, 'published', '[{"priority":10,"day_type":"any","days_of_week":127,"multiplier":"1.5"}]'::jsonb, $3, $3, NOW())`, otVersionID, otPolicyID, actorID)
	mustExec(`INSERT INTO work_policy_groups (id, code, name, created_by) VALUES ($1, $2, 'conc group', $3)`, groupID, "CNC-"+groupID.String()[:8], actorID)
	mustExec(`INSERT INTO work_policy_versions (id, group_id, version_no, status, effective_from, created_by, published_by, published_at, minutes_per_credit, work_unit_minutes, rounding_policy, early_arrival_strategy, early_ot_enabled, early_ot_qualification_grace_minutes, early_ot_max_minutes, break_auto_deduct, break_is_overtime, ot_aggregation_mode, working_pattern_id, pay_method, monthly_denominator, monthly_standard_days, overtime_policy_version_id) VALUES ($1, $2, 1, 'published', DATE '2026-01-01', $3, $3, NOW(), 480, 30, 'floor', 'block', false, 0, 60, true, false, 'aggregate', $4, 'monthly', 'fixed', 22, $5)`, policyVersionID, groupID, actorID, patternID, otVersionID)
	mustExec(`INSERT INTO work_policy_memberships (id, employment_relationship_id, group_id, effective_from, effective_to, created_by) VALUES (gen_random_uuid(), $1, $2, DATE '2026-01-01', DATE '2027-01-01', $3)`, relationshipID, groupID, actorID)
	mustExec(`INSERT INTO shift_templates (id, name) VALUES ($1, 'concurrent-shift')`, templateID)
	mustExec(`INSERT INTO shift_versions (id, template_id, version_number, name, effective_from, change_type, created_by) VALUES ($1, $2, 1, 'v1', DATE '2026-01-01', 'create', $3)`, versionID, templateID, actorID)
	mustExec(`INSERT INTO shift_default_intervals (id, version_id, sort_order, start_time, end_time) VALUES (gen_random_uuid(), $1, 0, '08:00', '17:00')`, versionID)

	schedRepo := ScheduleRepository{DB: pool}
	attRepo := AttendanceRepository{DB: pool}
	req := api.ResolveUnscheduledScheduleRequest{ShiftID: templateID, Reason: "Concurrent resolve test", Timezone: "UTC"}

	// Case 1: ResolveUnscheduled racing with ESSAttendanceAction + ReprocessAttendance on 2026-11-18.
	mustExec(`INSERT INTO raw_attendance_logs (id, employment_id, punch_time, source, raw_data, punch, status, created_at) VALUES (gen_random_uuid(), $1, TIMESTAMPTZ '2026-11-18 08:10:00+00', 'import', '{}'::jsonb, 0, 1, NOW()), (gen_random_uuid(), $1, TIMESTAMPTZ '2026-11-18 17:00:00+00', 'import', '{}'::jsonb, 1, 1, NOW())`, employmentID)
	start := make(chan struct{})
	var wg sync.WaitGroup
	errs := make(chan error, 3)
	wg.Add(3)
	go func() {
		defer wg.Done()
		<-start
		_, err := schedRepo.ResolveUnscheduled(ctx, actorID, employmentID, "2026-11-18", "conc-resolve-"+uuid.NewString(), req)
		errs <- err
	}()
	go func() {
		defer wg.Done()
		<-start
		_, err := attRepo.ESSAttendanceAction(ctx, employeeID, "work_in", nil, "conc-ess-"+uuid.NewString(), time.Date(2026, 11, 18, 8, 12, 0, 0, time.UTC))
		errs <- err
	}()
	go func() {
		defer wg.Done()
		<-start
		_, err := attRepo.ReprocessAttendance(ctx, "2026-11-18", "2026-11-18", &employeeID)
		errs <- err
	}()
	close(start)
	wg.Wait()
	close(errs)
	for err := range errs {
		if err != nil {
			t.Fatalf("concurrent resolve/ess/reprocess returned error (deadlock or failure): %v", err)
		}
	}
	var unlinkedCount int
	if err := pool.QueryRow(ctx, `SELECT count(*) FROM time_entries WHERE employment_id = $1 AND segment_id IS NULL AND punch_in_time >= TIMESTAMPTZ '2026-11-18 00:00:00+00' AND punch_in_time < TIMESTAMPTZ '2026-11-19 00:00:00+00'`, employmentID).Scan(&unlinkedCount); err != nil || unlinkedCount != 0 {
		t.Fatalf("concurrent resolve + ESS left %d unlinked time entries (err=%v)", unlinkedCount, err)
	}

	// Case 2: Two concurrent ReprocessAttendance runs must not insert duplicate time entries.
	start2 := make(chan struct{})
	var wg2 sync.WaitGroup
	errs2 := make(chan error, 2)
	wg2.Add(2)
	for range 2 {
		go func() {
			defer wg2.Done()
			<-start2
			_, err := attRepo.ReprocessAttendance(ctx, "2026-11-18", "2026-11-18", &employeeID)
			errs2 <- err
		}()
	}
	close(start2)
	wg2.Wait()
	close(errs2)
	for err := range errs2 {
		if err != nil {
			t.Fatalf("concurrent reprocess error: %v", err)
		}
	}
	var reprocessEntries int
	if err := pool.QueryRow(ctx, `SELECT count(*) FROM time_entries WHERE employment_id = $1 AND punch_in_time = TIMESTAMPTZ '2026-11-18 08:10:00+00'`, employmentID).Scan(&reprocessEntries); err != nil || reprocessEntries > 1 {
		t.Fatalf("concurrent reprocess created duplicate paired entries: count=%d err=%v", reprocessEntries, err)
	}
}
