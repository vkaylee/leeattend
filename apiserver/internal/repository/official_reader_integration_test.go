//go:build integration

package repository

import (
	"testing"
	"time"

	"github.com/google/uuid"
	api "github.com/vkaylee/leeattend/apiserver/internal/api"
)

func TestIntegrationOfficialReaderAndNoShowUsePublishedRevisionSegments(t *testing.T) {
	pool, ctx := requireIntegrationDatabase(t)
	employerID, employeeID, employmentID, actorID := uuid.New(), uuid.New(), uuid.New(), uuid.New()
	templateID, versionID, batchID, revisionID, segmentID, overrideID := uuid.New(), uuid.New(), uuid.New(), uuid.New(), uuid.New(), uuid.New()
	if _, err := pool.Exec(ctx, `INSERT INTO employers (id, name) VALUES ($1, 'official-reader-employer')`, employerID); err != nil {
		t.Fatalf("insert employer: %v", err)
	}
	seedIntegrationEmployeeGraphWithRange(t, ctx, pool, employeeID, employerID, employmentID, employeeID.String(), "official-reader-employee", "2026-01-01", "2026-11-20", "2026-11-20")
	if _, err := pool.Exec(ctx, `INSERT INTO users (id, email, password_hash) VALUES ($1, $2, 'integration-test-hash')`, actorID, actorID.String()+"@invalid.test"); err != nil {
		t.Fatalf("insert actor: %v", err)
	}
	if _, err := pool.Exec(ctx, `INSERT INTO shift_templates (id, name) VALUES ($1, 'official-reader-shift')`, templateID); err != nil {
		t.Fatalf("insert template: %v", err)
	}
	if _, err := pool.Exec(ctx, `INSERT INTO shift_versions (id, template_id, version_number, name, effective_from, change_type, created_by) VALUES ($1, $2, 1, 'official-reader-version', DATE '2026-01-01', 'create', $3)`, versionID, templateID, actorID); err != nil {
		t.Fatalf("insert version: %v", err)
	}
	if _, err := pool.Exec(ctx, `INSERT INTO shift_default_intervals (id, version_id, sort_order, start_time, end_time) VALUES (gen_random_uuid(), $1, 0, '22:00', '06:00')`, versionID); err != nil {
		t.Fatalf("insert interval: %v", err)
	}
	if _, err := pool.Exec(ctx, `INSERT INTO schedule_publication_batches (id, idempotency_key, request_hash, mode, status, requested_by, operation, response_snapshot, reason, timezone, range_start, range_end) VALUES ($1, $2, repeat('b', 64), 'rotating', 'published', $3, 'publish', '{}'::jsonb, 'official reader integration', 'UTC', DATE '2026-11-20', DATE '2026-11-20')`, batchID, batchID.String(), actorID); err != nil {
		t.Fatalf("insert batch: %v", err)
	}
	if _, err := pool.Exec(ctx, `INSERT INTO schedule_day_revisions (id, employment_id, employer_id, business_date, timezone, revision_number, state, publication_batch_id, source, source_snapshot, resolver_version, reason, published_by, published_actor_type, published_at) VALUES ($1, $2, $3, DATE '2026-11-20', 'UTC', 1, 'draft', $4, 'pattern', '{}'::jsonb, 'schedule-resolver-v1', 'official reader integration', $5, 'human', NULL)`, revisionID, employmentID, employerID, batchID, actorID); err != nil {
		t.Fatalf("insert revision: %v", err)
	}
	tx, err := pool.Begin(ctx)
	if err != nil {
		t.Fatalf("begin publication fixture: %v", err)
	}
	if _, err := tx.Exec(ctx, `SELECT set_config('schedule.lifecycle_publication', 'on', true)`); err != nil {
		t.Fatalf("set publication context: %v", err)
	}
	if _, err := tx.Exec(ctx, `INSERT INTO work_segments (id, employment_id, date, sort_order, scheduled_start, scheduled_end, segment_type, segment_status, compensation_rate, gap_compensation_before, gap_compensation_rate_before, source, source_detail, generation_metadata, template_id, schedule_revision_id, selected_shift_version_id, scheduled_start_at, scheduled_end_at, breaks_snapshot) VALUES ($1, $2, DATE '2026-11-20', 0, '22:00', '06:00', 'work', 'scheduled', 1, 'unpaid', 0, 'pattern', '{}'::jsonb, '{}'::jsonb, $3, $4, $5, TIMESTAMPTZ '2026-11-20 22:00:00+00', TIMESTAMPTZ '2026-11-21 06:00:00+00', '[]'::jsonb)`, segmentID, employmentID, templateID, revisionID, versionID); err != nil {
		_ = tx.Rollback(ctx)
		t.Fatalf("insert segment: %v", err)
	}
	if _, err := tx.Exec(ctx, `UPDATE schedule_day_revisions SET state = 'published', published_at = CURRENT_TIMESTAMP WHERE id = $1`, revisionID); err != nil {
		_ = tx.Rollback(ctx)
		t.Fatalf("publish revision: %v", err)
	}
	if err := tx.Commit(ctx); err != nil {
		t.Fatalf("commit publication fixture: %v", err)
	}
	legacyID := uuid.New()
	if _, err := pool.Exec(ctx, `INSERT INTO work_segments (id, employment_id, date, sort_order, scheduled_start, scheduled_end, segment_type, segment_status, template_id) VALUES ($1, $2, DATE '2026-11-19', 0, '08:00', '17:00', 'work', 'scheduled', $3)`, legacyID, employmentID, templateID); err != nil {
		t.Fatalf("insert legacy segment: %v", err)
	}
	shadow, err := loadLegacyShadowDays(ctx, pool, []uuid.UUID{employmentID}, "2026-11-19", "2026-11-20")
	if err != nil {
		t.Fatalf("load legacy shadow: %v", err)
	}
	legacyDay := shadow[shadowDayKey(employmentID, "2026-11-19")]
	if legacyDay == nil || len(legacyDay.SegmentIDs) != 1 || legacyDay.SegmentIDs[0] != legacyID {
		t.Fatalf("legacy shadow missing legacy segment: %+v", legacyDay)
	}
	if day := shadow[shadowDayKey(employmentID, "2026-11-20")]; day != nil {
		t.Fatalf("official segment included in legacy shadow: %+v", day)
	}
	if _, err := pool.Exec(ctx, `INSERT INTO schedule_day_overrides (id, employment_id, business_date, work_segment_id, override_type, start_at, end_at, minutes, reason, status, created_by) VALUES ($1, $2, DATE '2026-11-20', $3, 'break_worked', TIMESTAMPTZ '2026-11-20 12:15:00+00', TIMESTAMPTZ '2026-11-20 12:45:00+00', 30, 'official reader integration', 'approved', $4)`, overrideID, employmentID, segmentID, actorID); err != nil {
		t.Fatalf("insert break worked override: %v", err)
	}

	schedules, err := (ScheduleRepository{DB: pool}).GetOfficialSchedules(ctx, time.Date(2026, 11, 20, 0, 0, 0, 0, time.UTC), time.Date(2026, 11, 20, 0, 0, 0, 0, time.UTC), []uuid.UUID{employmentID}, 1, 50)
	if err != nil {
		t.Fatalf("official schedules: %v", err)
	}
	if len(schedules.Data) != 1 || len(schedules.Data[0].Schedules) != 1 {
		t.Fatalf("official schedule rows = %+v", schedules.Data)
	}
	day := schedules.Data[0].Schedules[0]
	if day.RevisionID == nil || *day.RevisionID != revisionID || day.ScheduleStatus != "work" || day.StartTime == nil || *day.StartTime != "22:00:00" {
		t.Fatalf("unexpected official day: %+v", day)
	}
	if len(day.BreakWorkedOverrides) != 1 || day.BreakWorkedOverrides[0].ID != overrideID || day.BreakWorkedOverrides[0].SegmentID != segmentID || day.BreakWorkedOverrides[0].Minutes != 30 {
		t.Fatalf("official day missing break worked override: %+v", day.BreakWorkedOverrides)
	}

	noShow, err := (ReportRepository{DB: pool}).GetNoShowReport(ctx, api.NoShowReportFilter{Date: "2026-11-20", EmployeeIDs: []uuid.UUID{employeeID}, Page: 1, Limit: 50})
	if err != nil {
		t.Fatalf("no-show report: %v", err)
	}
	if noShow.Total != 1 {
		t.Fatalf("no-show total = %d, want 1", noShow.Total)
	}
}
