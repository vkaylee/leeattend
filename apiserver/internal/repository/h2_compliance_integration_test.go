//go:build integration

package repository

import (
	"errors"
	"testing"

	"github.com/google/uuid"
	"github.com/vkaylee/leeattend/apiserver/internal/api"
	"github.com/vkaylee/leeattend/apiserver/internal/domain/compliance"
)

func TestIntegrationAdjacentNightShiftRestEffectiveVersionAndCrossEmployment(t *testing.T) {
	pool, ctx := requireIntegrationDatabase(t)

	employerA, employerB := uuid.New(), uuid.New()
	employeeID := uuid.New()
	empA, empB := uuid.New(), uuid.New()
	actorID := uuid.New()
	nightTemplate, nightV1 := uuid.New(), uuid.New()
	morningOverlapTemplate, morningOverlapV1 := uuid.New(), uuid.New()
	touchingTemplate, touchingV1 := uuid.New(), uuid.New()
	day10hTemplate, day10hV1 := uuid.New(), uuid.New()
	versionedTemplate, verOld, verNew := uuid.New(), uuid.New(), uuid.New()
	batchID := uuid.New()

	mustExec := func(query string, args ...any) {
		t.Helper()
		if _, err := pool.Exec(ctx, query, args...); err != nil {
			t.Fatalf("exec %q: %v", query, err)
		}
	}

	mustExec(`INSERT INTO employers (id, name) VALUES ($1, 'h2-emp-a'), ($2, 'h2-emp-b')`, employerA, employerB)
	tenureID, relA, relB := uuid.New(), uuid.New(), uuid.New()
	mustExec(`INSERT INTO employees (id, full_name) VALUES ($1, 'H2 Compliance Worker')`, employeeID)
	mustExec(`INSERT INTO employee_tenures (id, employee_id, employee_number, hire_type, start_date) VALUES ($1, $2, $3, 'initial', DATE '2026-01-01')`, tenureID, employeeID, "H2-"+employeeID.String()[:8])
	mustExec(`INSERT INTO employment_relationships (id, employee_id, employer_id, tenure_id, status, start_date) VALUES ($1, $2, $3, $4, 'active', DATE '2026-01-01'), ($5, $2, $6, $4, 'active', DATE '2026-01-01')`, relA, employeeID, employerA, tenureID, relB, employerB)
	mustExec(`INSERT INTO employment_segments (id, employee_id, employer_id, relationship_id, status, start_date, end_date) VALUES ($1, $2, $3, $4, 'active', DATE '2026-11-01', DATE '2026-12-31'), ($5, $2, $6, $7, 'active', DATE '2026-11-01', DATE '2026-12-31')`, empA, employeeID, employerA, relA, empB, employerB, relB)
	mustExec(`INSERT INTO users (id, email, password_hash) VALUES ($1, $2, 'integration-test-hash')`, actorID, actorID.String()+"@invalid.test")

	// Configure compliance rules with allow_manager_override = false and restore after test.
	mustExec(`DELETE FROM compliance_rules`)
	mustExec(`INSERT INTO compliance_rules (id, is_enabled, min_rest_period_hours, max_weekly_hours, max_monthly_ot_hours, allow_manager_override) VALUES (gen_random_uuid(), true, 12, 48, 40, false)`)
	t.Cleanup(func() { _, _ = pool.Exec(ctx, `DELETE FROM compliance_rules`) })

	createShift := func(tplID, verID uuid.UUID, name, startTime, endTime string) {
		t.Helper()
		mustExec(`INSERT INTO shift_templates (id, name) VALUES ($1, $2)`, tplID, name)
		mustExec(`INSERT INTO shift_versions (id, template_id, version_number, name, effective_from, change_type, created_by) VALUES ($1, $2, 1, $3, DATE '2026-01-01', 'create', $4)`, verID, tplID, name+" v1", actorID)
		mustExec(`UPDATE shift_templates SET current_version_id = $1 WHERE id = $2`, verID, tplID)
		mustExec(`INSERT INTO shift_default_intervals (id, version_id, sort_order, start_time, end_time) VALUES (gen_random_uuid(), $1, 0, $2::time, $3::time)`, verID, startTime, endTime)
	}

	createShift(nightTemplate, nightV1, "Night 22-06", "22:00:00", "06:00:00")
	createShift(morningOverlapTemplate, morningOverlapV1, "Morning 05-13", "05:00:00", "13:00:00")
	createShift(touchingTemplate, touchingV1, "Touching 06-14", "06:00:00", "14:00:00")
	createShift(day10hTemplate, day10hV1, "Day 10h", "08:00:00", "18:00:00")

	// Versioned template: V1 (08:00-17:00) effective through Nov 30, V2 (22:00-06:00) effective from Dec 1 (current_version_id = V2).
	mustExec(`INSERT INTO shift_templates (id, name) VALUES ($1, 'Versioned Shift')`, versionedTemplate)
	mustExec(`INSERT INTO shift_versions (id, template_id, version_number, name, effective_from, effective_to, change_type, created_by) VALUES ($1, $2, 1, 'v1', DATE '2026-01-01', DATE '2026-11-30', 'create', $3)`, verOld, versionedTemplate, actorID)
	mustExec(`INSERT INTO shift_versions (id, template_id, version_number, name, effective_from, change_type, created_by) VALUES ($1, $2, 2, 'v2', DATE '2026-12-01', 'update', $3)`, verNew, versionedTemplate, actorID)
	mustExec(`UPDATE shift_templates SET current_version_id = $1 WHERE id = $2`, verNew, versionedTemplate)
	mustExec(`INSERT INTO shift_default_intervals (id, version_id, sort_order, start_time, end_time) VALUES (gen_random_uuid(), $1, 0, '08:00:00'::time, '17:00:00'::time), (gen_random_uuid(), $2, 0, '22:00:00'::time, '06:00:00'::time)`, verOld, verNew)

	// Publish official schedule on empA:
	// - 2026-11-20: Night 22:00-06:00
	// - 2026-11-23..26 (Mon..Thu): 4 x 10h = 40h
	mustExec(`INSERT INTO schedule_publication_batches (id, idempotency_key, request_hash, mode, status, requested_by, operation, response_snapshot, reason, timezone, range_start, range_end) VALUES ($1, $2, repeat('c', 64), 'rotating', 'published', $3, 'publish', '{}'::jsonb, 'h2 fixture', 'UTC', DATE '2026-11-20', DATE '2026-11-26')`, batchID, batchID.String(), actorID)

	publishSeg := func(empID, employerID, tplID, verID uuid.UUID, dateStr, startLocal, endLocal, startAt, endAt string) {
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
		if _, err := tx.Exec(ctx, `INSERT INTO schedule_day_revisions (id, employment_id, employer_id, business_date, timezone, revision_number, state, publication_batch_id, source, source_snapshot, resolver_version, reason, published_by, published_actor_type, published_at) VALUES ($1, $2, $3, $4::date, 'UTC', 1, 'draft', $5, 'manual', '{}'::jsonb, 'schedule-resolver-v1', 'h2', $6, 'human', NULL)`, revID, empID, employerID, dateStr, batchID, actorID); err != nil {
			_ = tx.Rollback(ctx)
			t.Fatalf("insert rev: %v", err)
		}
		if _, err := tx.Exec(ctx, `INSERT INTO work_segments (id, employment_id, date, sort_order, scheduled_start, scheduled_end, segment_type, segment_status, compensation_rate, gap_compensation_before, gap_compensation_rate_before, source, source_detail, generation_metadata, template_id, schedule_revision_id, selected_shift_version_id, scheduled_start_at, scheduled_end_at, breaks_snapshot) VALUES ($1, $2, $3::date, 0, $4::time, $5::time, 'work', 'scheduled', 1, 'unpaid', 0, 'manual', '{}'::jsonb, '{}'::jsonb, $6, $7, $8, $9::timestamptz, $10::timestamptz, '[]'::jsonb)`, uuid.New(), empID, dateStr, startLocal, endLocal, tplID, revID, verID, startAt, endAt); err != nil {
			_ = tx.Rollback(ctx)
			t.Fatalf("insert seg: %v", err)
		}
		if _, err := tx.Exec(ctx, `UPDATE schedule_day_revisions SET state = 'published', published_at = CURRENT_TIMESTAMP WHERE id = $1`, revID); err != nil {
			_ = tx.Rollback(ctx)
			t.Fatalf("publish rev: %v", err)
		}
		if err := tx.Commit(ctx); err != nil {
			t.Fatalf("commit rev: %v", err)
		}
	}

	publishSeg(empA, employerA, nightTemplate, nightV1, "2026-11-20", "22:00", "06:00", "2026-11-20T22:00:00Z", "2026-11-21T06:00:00Z")
	publishSeg(empA, employerA, day10hTemplate, day10hV1, "2026-11-23", "08:00", "18:00", "2026-11-23T08:00:00Z", "2026-11-23T18:00:00Z")
	publishSeg(empA, employerA, day10hTemplate, day10hV1, "2026-11-24", "08:00", "18:00", "2026-11-24T08:00:00Z", "2026-11-24T18:00:00Z")
	publishSeg(empA, employerA, day10hTemplate, day10hV1, "2026-11-25", "08:00", "18:00", "2026-11-25T08:00:00Z", "2026-11-25T18:00:00Z")
	publishSeg(empA, employerA, day10hTemplate, day10hV1, "2026-11-26", "08:00", "18:00", "2026-11-26T08:00:00Z", "2026-11-26T18:00:00Z")
	// Add existing morning 05-13 roster on 2026-11-16 and 2026-12-02 to test effective shift version selection.
	mustExec(`INSERT INTO shift_rosters (id, employment_id, template_id, date) VALUES (gen_random_uuid(), $1, $2, DATE '2026-11-16'), (gen_random_uuid(), $1, $2, DATE '2026-12-02')`, empA, morningOverlapTemplate)

	shiftRepo := ShiftRepository{DB: pool}
	strPtr := func(s string) *string { return &s }

	// 1. Adjacent night 22-06 (published on Nov 20) vs 05-13 on Nov 21 => hard overlap blocked.
	err := shiftRepo.AssignShift(ctx, actorID, api.AssignShiftRequest{
		EmploymentID: empA, ShiftID: morningOverlapTemplate, StartDate: "2026-11-21", EndDate: strPtr("2026-11-21"),
	})
	var compErr *api.ShiftComplianceError
	if !errors.As(err, &compErr) || !compErr.Hard {
		t.Fatalf("expected hard compliance error for 22-06 vs 05-13 adjacent day, got %v", err)
	}

	// 2. Touching 06-14 on Nov 21 with allow_manager_override=false + AllowWarnings=true => still blocked!
	err = shiftRepo.AssignShift(ctx, actorID, api.AssignShiftRequest{
		EmploymentID: empA, ShiftID: touchingTemplate, StartDate: "2026-11-21", EndDate: strPtr("2026-11-21"),
		AllowWarnings: true, OverrideReason: strPtr("Attempt override when disabled"),
	})
	if !errors.As(err, &compErr) || !compErr.Hard {
		t.Fatalf("expected hard block when allow_manager_override=false, got %v", err)
	}

	// 3. Full weekly hours: Mon-Thu published 40h + Fri 10h = 50h > 48h => blocked!
	err = shiftRepo.AssignShift(ctx, actorID, api.AssignShiftRequest{
		EmploymentID: empA, ShiftID: day10hTemplate, StartDate: "2026-11-27", EndDate: strPtr("2026-11-27"),
	})
	if !errors.As(err, &compErr) {
		t.Fatalf("expected weekly hours compliance error (40h existing + 10h > 48h), got %v", err)
	}
	foundWeekly := false
	for _, v := range compErr.Violations {
		if v.Type == compliance.ViolationWeeklyHours {
			foundWeekly = true
		}
	}
	if !foundWeekly {
		t.Fatalf("expected ViolationWeeklyHours, got %+v", compErr.Violations)
	}

	// 4. Cross-employment overlap: empB assigning Day 10h on Nov 25 overlaps empA's published Day 10h on Nov 25!
	err = shiftRepo.AssignShift(ctx, actorID, api.AssignShiftRequest{
		EmploymentID: empB, ShiftID: day10hTemplate, StartDate: "2026-11-25", EndDate: strPtr("2026-11-25"),
	})
	if !errors.As(err, &compErr) || !compErr.Hard {
		t.Fatalf("expected cross-employment hard overlap on empB, got %v", err)
	}

	// 5. Effective shift version by date:
	// - Nov 15 uses V1 (08:00-17:00), followed by Nov 16 (05:00-13:00): 12h rest, NO overlap => succeeds!
	err = shiftRepo.AssignShift(ctx, actorID, api.AssignShiftRequest{
		EmploymentID: empA, ShiftID: versionedTemplate, StartDate: "2026-11-15", EndDate: strPtr("2026-11-15"),
	})
	if err != nil {
		t.Fatalf("expected V1 (08:00-17:00) on Nov 15 to pass compliance, got %v", err)
	}
	// - Dec 1 uses V2 (22:00-06:00), followed by Dec 2 (05:00-13:00): hard overlap => blocked!
	err = shiftRepo.AssignShift(ctx, actorID, api.AssignShiftRequest{
		EmploymentID: empA, ShiftID: versionedTemplate, StartDate: "2026-12-01", EndDate: strPtr("2026-12-01"),
	})
	if !errors.As(err, &compErr) || !compErr.Hard {
		t.Fatalf("expected V2 (22:00-06:00) on Dec 1 to fail overlap with Dec 2 05:00-13:00, got %v", err)
	}

	// 6. Out-of-employment date rejected.
	err = shiftRepo.AssignShift(ctx, actorID, api.AssignShiftRequest{
		EmploymentID: empA, ShiftID: touchingTemplate, StartDate: "2026-10-15", EndDate: strPtr("2026-10-15"),
	})
	if !errors.Is(err, api.ErrShiftValidation) {
		t.Fatalf("expected ErrShiftValidation for date outside employment, got %v", err)
	}

	// 7. Pattern compliance: rotation Day 0 = 22:00-06:00, Day 1 = 05:00-13:00 => hard overlap detected!
	patternID := uuid.New()
	mustExec(`INSERT INTO working_patterns (id, name, is_active, pattern_type, cycle_length_days) VALUES ($1, 'H2 Bad Rotation', true, 'rotation', 2)`, patternID)
	mustExec(`INSERT INTO working_pattern_details (id, pattern_id, schedule_day, template_id) VALUES (gen_random_uuid(), $1, 0, $2), (gen_random_uuid(), $1, 1, $3)`, patternID, nightTemplate, morningOverlapTemplate)

	patternRepo := WorkingPatternRepository{DB: pool}
	err = patternRepo.CheckPatternCompliance(ctx, empA, api.AssignPatternRequest{
		PatternID:  patternID,
		StartDate:  "2026-12-07",
		EndDate:    strPtr("2026-12-08"),
		AnchorDate: strPtr("2026-12-07"),
	})
	if !errors.As(err, &compErr) || !compErr.Hard {
		t.Fatalf("expected hard compliance overlap for rotating pattern 22-06 -> 05-13, got %v", err)
	}
}
